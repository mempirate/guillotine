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
use guillotine::{
    style::{AlignItems, JustifyContent, StyledElement},
    *,
};

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
            .gap(4)
            .background(CANVAS)
            .size(Size::new(320, 172))
            .child(header(cx))
            .child(live_power(cx, self.power))
            .child(
                cx.row()
                    .height(40)
                    .justify_content(JustifyContent::SpaceBetween)
                    .child(metric_card(cx, "VOLTAGE", self.voltage))
                    .child(metric_card(cx, "CURRENT", self.current))
                    .child(metric_card(cx, "REFRESH", "5 SEC")),
            )
            .child(
                cx.row()
                    .height(40)
                    .justify_content(JustifyContent::SpaceBetween)
                    .child(summary_card(cx, "ENERGY / RESET", self.energy))
                    .child(summary_card(cx, "EST. COST", self.cost)),
            )
    }
}

fn header(
    cx: &Context<'_>,
) -> impl ElementBuilder + StyledElement<Color = Rgb565, Specific = DivStyle> {
    cx.row()
        .justify_content(JustifyContent::SpaceBetween)
        .child(cx.text("SHELLY POWER").font(Font::mono(&FONT_9X18_BOLD)).text_color(Rgb565::WHITE))
        .child(
            cx.text("ONLINE")
                .padding((4, 12))
                .background(ACCENT)
                .width(66)
                .font(Font::mono(&FONT_6X10))
                .text_color(CANVAS),
        )
}

fn live_power(
    cx: &Context<'_>,
    power: &str,
) -> impl ElementBuilder + StyledElement<Color = Rgb565, Specific = DivStyle> {
    cx.row()
        .border((0, 0, 0, 2))
        .border_color(ACCENT)
        .background(PANEL_STRONG)
        .height(44)
        .align_items(AlignItems::Center)
        .child(
            cx.text("LIVE LOAD")
                .padding((0, 12))
                .width(108)
                .font(Font::mono(&FONT_6X10))
                .text_color(MUTED),
        )
        .child(
            cx.text(power)
                .padding((0, 14))
                .width(196)
                .font(Font::mono(&FONT_10X20))
                .text_color(Rgb565::WHITE),
        )
}

fn metric_card(
    cx: &Context<'_>,
    label: &str,
    value: &str,
) -> impl ElementBuilder + StyledElement<Color = Rgb565, Specific = DivStyle> {
    cx.column()
        .padding((4, 6))
        .border(1)
        .border_color(EDGE)
        .background(PANEL)
        .width(100)
        .justify_content(JustifyContent::SpaceBetween)
        .child(cx.text(label).font(Font::mono(&FONT_6X10)).text_color(MUTED))
        .child(cx.text(value).font(Font::mono(&FONT_9X18_BOLD)).text_color(Rgb565::WHITE))
}

fn summary_card(
    cx: &Context<'_>,
    label: &str,
    value: &str,
) -> impl ElementBuilder + StyledElement<Color = Rgb565, Specific = DivStyle> {
    cx.column()
        .padding((4, 6))
        .border(1)
        .border_color(EDGE)
        .background(PANEL)
        .width(151)
        .justify_content(JustifyContent::SpaceBetween)
        .child(cx.text(label).font(Font::mono(&FONT_6X10)).text_color(MUTED))
        .child(cx.text(value).font(Font::mono(&FONT_9X18_BOLD)).text_color(Rgb565::WHITE))
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
    println!("Storage size: {}", storage.size());

    let mut ui = Ui::new(DirectTarget::new(display), storage).with_background(CANVAS);
    ui.render(&monitor).unwrap();

    let usage = ui.storage().usage();
    let cap = ui.storage().capacity();
    println!("Usage: nodes={} text={}", usage.nodes, usage.text);
    println!("Node utilization: {:.2}%", usage.nodes as f32 / cap.nodes as f32 * 100.0);
    println!("Text utilization: {:.2}%", usage.text as f32 / cap.text as f32 * 100.0);

    let output_settings = OutputSettingsBuilder::new().scale(3).build();
    let title = format!("Guillotine: {}", env!("CARGO_BIN_NAME"));
    Window::new(&title, &output_settings).show_static(ui.display());
}
