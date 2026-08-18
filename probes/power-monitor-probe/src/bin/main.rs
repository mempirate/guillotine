#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use core::{cell::UnsafeCell, convert::Infallible, hint::black_box, mem::MaybeUninit};
use embedded_graphics::{
    Pixel,
    mono_font::ascii::{FONT_6X10, FONT_9X18_BOLD, FONT_10X20},
    pixelcolor::Rgb565,
    prelude::{DrawTarget, OriginDimensions, RgbColor as _, Size},
};
use esp_hal::{clock::CpuClock, main};
use guillotine::{style::StyledElement, *};

const WIDTH: u32 = 320;
const HEIGHT: u32 = 172;
const NODE_CAPACITY: usize = 64;
const TEXT_CAPACITY: usize = 1024;

// A deliberately restrained palette: neutral surfaces and one status accent.
const CANVAS: Rgb565 = Rgb565::new(2, 3, 4);
const PANEL: Rgb565 = Rgb565::new(4, 8, 9);
const PANEL_STRONG: Rgb565 = Rgb565::new(5, 11, 11);
const EDGE: Rgb565 = Rgb565::new(8, 17, 16);
const ACCENT: Rgb565 = Rgb565::new(12, 50, 27);
const MUTED: Rgb565 = Rgb565::new(17, 34, 25);

/// A display that exercises layout and drawing without retaining a framebuffer.
struct NoopDisplay;

impl OriginDimensions for NoopDisplay {
    fn size(&self) -> Size {
        Size::new(WIDTH, HEIGHT)
    }
}

impl DrawTarget for NoopDisplay {
    type Color = Rgb565;
    type Error = Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        // Consume the iterator so the probe exercises Guillotine's complete draw path.
        for pixel in pixels {
            black_box(pixel);
        }
        Ok(())
    }

    fn fill_solid(
        &mut self,
        area: &embedded_graphics::primitives::Rectangle,
        color: Self::Color,
    ) -> Result<(), Self::Error> {
        black_box((area, color));
        Ok(())
    }

    fn clear(&mut self, color: Self::Color) -> Result<(), Self::Error> {
        black_box(color);
        Ok(())
    }
}

struct PowerMonitor<'a> {
    power: &'a str,
    voltage: &'a str,
    current: &'a str,
    energy: &'a str,
    cost: &'a str,
}

impl Render for PowerMonitor<'_> {
    fn render(&self, cx: &Context<'_>) -> impl ElementBuilder {
        cx.column()
            .padding(8)
            .background(CANVAS)
            .size(Size::new(WIDTH, HEIGHT))
            .child(header(cx))
            .child(live_power(cx, self.power).margin((4, 0, 0, 0)))
            .child(
                cx.row()
                    .margin((4, 0, 0, 0))
                    .child(metric_card(cx, "VOLTAGE", self.voltage).margin((0, 2, 0, 0)))
                    .child(metric_card(cx, "CURRENT", self.current).margin((0, 2, 0, 0)))
                    .child(metric_card(cx, "REFRESH", "5 SEC")),
            )
            .child(
                cx.row()
                    .margin((4, 0, 0, 0))
                    .child(summary_card(cx, "ENERGY / RESET", self.energy).margin((0, 2, 0, 0)))
                    .child(summary_card(cx, "EST. COST", self.cost)),
            )
    }
}

fn header(
    cx: &Context<'_>,
) -> impl ElementBuilder + StyledElement<Color = Rgb565, Specific = RowStyle> {
    cx.row()
        .size(Size::new(304, 18))
        .child(
            cx.text("SHELLY POWER")
                .size(Size::new(238, 18))
                .font(Font::mono(&FONT_9X18_BOLD))
                .text_color(Rgb565::WHITE),
        )
        .child(
            cx.text("ONLINE")
                .padding((4, 12))
                .background(ACCENT)
                .size(Size::new(66, 18))
                .font(Font::mono(&FONT_6X10))
                .text_color(CANVAS),
        )
}

fn live_power(
    cx: &Context<'_>,
    power: &str,
) -> impl ElementBuilder + StyledElement<Color = Rgb565, Specific = RowStyle> {
    cx.row()
        .border((0, 0, 0, 2))
        .border_color(ACCENT)
        .background(PANEL_STRONG)
        .size(Size::new(304, 44))
        .child(
            cx.text("LIVE LOAD")
                .padding((17, 12))
                .size(Size::new(108, 42))
                .font(Font::mono(&FONT_6X10))
                .text_color(MUTED),
        )
        .child(
            cx.text(power)
                .padding((11, 14))
                .size(Size::new(196, 42))
                .font(Font::mono(&FONT_10X20))
                .text_color(Rgb565::WHITE),
        )
}

fn metric_card(
    cx: &Context<'_>,
    label: &str,
    value: &str,
) -> impl ElementBuilder + StyledElement<Color = Rgb565, Specific = ColumnStyle> {
    cx.column()
        .padding((4, 6))
        .border(1)
        .border_color(EDGE)
        .background(PANEL)
        .size(Size::new(100, 40))
        .child(
            cx.text(label).size(Size::new(86, 12)).font(Font::mono(&FONT_6X10)).text_color(MUTED),
        )
        .child(
            cx.text(value)
                .size(Size::new(86, 18))
                .font(Font::mono(&FONT_9X18_BOLD))
                .text_color(Rgb565::WHITE),
        )
}

fn summary_card(
    cx: &Context<'_>,
    label: &str,
    value: &str,
) -> impl ElementBuilder + StyledElement<Color = Rgb565, Specific = ColumnStyle> {
    cx.column()
        .padding((4, 6))
        .border(1)
        .border_color(EDGE)
        .background(PANEL)
        .size(Size::new(151, 40))
        .child(
            cx.text(label).size(Size::new(137, 12)).font(Font::mono(&FONT_6X10)).text_color(MUTED),
        )
        .child(
            cx.text(value)
                .size(Size::new(137, 18))
                .font(Font::mono(&FONT_9X18_BOLD))
                .text_color(Rgb565::WHITE),
        )
}

type ProbeUi = Ui<NoopDisplay, NODE_CAPACITY, TEXT_CAPACITY>;

/// A small no-dependency equivalent of `StaticCell` for this single, boot-time initialization.
#[repr(transparent)]
struct StaticUi(UnsafeCell<MaybeUninit<ProbeUi>>);

// SAFETY: `POWER_MONITOR_UI` is initialized once and only main accesses the unique reference
// returned by `init_ui`; no interrupt handler or second core can access it.
unsafe impl Sync for StaticUi {}

/// Keeping the complete UI in a named static makes its fixed-capacity storage visible in the ELF.
#[used]
#[unsafe(no_mangle)]
static POWER_MONITOR_UI: StaticUi = StaticUi(UnsafeCell::new(MaybeUninit::uninit()));

unsafe fn init_ui() -> &'static mut ProbeUi {
    let slot = POWER_MONITOR_UI.0.get();

    // SAFETY: this function is called exactly once from `main`; the slot is valid, aligned storage
    // for `ProbeUi`, and no other references to it exist.
    unsafe { (*slot).write(Ui::new(NoopDisplay, FrameStorage::default()).with_background(CANVAS)) }
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {
        core::hint::spin_loop();
    }
}

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

#[allow(
    clippy::large_stack_frames,
    reason = "it's not unusual to allocate larger buffers etc. in main"
)]
#[main]
fn main() -> ! {
    // generator version: 1.3.0
    // generator parameters: --headless --chip esp32c3

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let _peripherals = esp_hal::init(config);

    // SAFETY: this is the only initialization and access site for the static UI.
    let ui = unsafe { init_ui() };
    let monitor = PowerMonitor {
        power: "86.4 W",
        voltage: "232.1 V",
        current: "0.373 A",
        energy: "14.18 kWh",
        cost: "1.84 EUR",
    };

    if ui.render(&monitor).is_err() {
        loop {
            core::hint::spin_loop();
        }
    }

    // Keep both the fixed reservation and peak frame usage observable to the optimizer/debugger.
    black_box((ui.storage().size(), ui.storage().capacity(), ui.storage().usage()));

    loop {
        core::hint::spin_loop();
    }
}
