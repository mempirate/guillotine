//! Shelly Plug power-monitor dashboard for a 320 × 172 display.
//!
//! This mirrors the dashboard in `mempirate/shellyctl`. A real integration can keep formatted
//! readings in application state and rerender this view after each Shelly poll.
use embedded_graphics::{
    mono_font::ascii::{FONT_6X10, FONT_9X18_BOLD, FONT_10X20},
    pixelcolor::Rgb565,
    prelude::{RgbColor as _, Size},
};
use embedded_graphics_simulator::{OutputSettingsBuilder, SimulatorDisplay, Window};
use guillotine::{style::StyledElement, *};

// A deliberately restrained palette: neutral surfaces and one status accent.
const CANVAS: Rgb565 = Rgb565::new(2, 3, 4);
const PANEL: Rgb565 = Rgb565::new(4, 8, 9);
const PANEL_STRONG: Rgb565 = Rgb565::new(5, 11, 11);
const EDGE: Rgb565 = Rgb565::new(8, 17, 16);
const ACCENT: Rgb565 = Rgb565::new(12, 50, 27);
const MUTED: Rgb565 = Rgb565::new(17, 34, 25);

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
            .size(Size::new(320, 172))
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

fn main() {
    let display = SimulatorDisplay::<Rgb565>::new(Size::new(320, 172));
    let monitor = PowerMonitor {
        power: "86.4 W",
        voltage: "232.1 V",
        current: "0.373 A",
        energy: "14.18 kWh",
        cost: "1.84 EUR",
    };

    let storage = FrameStorage::<Rgb565>::default();
    let mut ui = Ui::new(display, storage).with_background(CANVAS);
    ui.render(&monitor).unwrap();

    let output_settings = OutputSettingsBuilder::new().scale(3).build();
    let title = format!("Guillotine: {}", env!("CARGO_BIN_NAME"));
    Window::new(&title, &output_settings).show_static(ui.display());
}
