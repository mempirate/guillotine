//! Shelly Plug power-monitor dashboard for a 320 × 172 display.
//!
//! The sample data mirrors `mempirate/shellyctl::PowerStatus`. A real integration can keep the
//! formatted strings in its application state and rerender this view after each Shelly poll.
use std::env;

use embedded_graphics::{
    mono_font::ascii::{FONT_6X10, FONT_9X18_BOLD, FONT_10X20},
    pixelcolor::Rgb565,
    prelude::{RgbColor as _, Size},
};
use embedded_graphics_simulator::{OutputSettingsBuilder, SimulatorDisplay, Window};
use guillotine::{style::StyledElement, *};

const CANVAS: Rgb565 = Rgb565::new(1, 3, 7);
const PANEL: Rgb565 = Rgb565::new(2, 7, 11);
const PANEL_HIGHLIGHT: Rgb565 = Rgb565::new(1, 13, 16);
const CYAN: Rgb565 = Rgb565::new(0, 51, 27);
const GREEN: Rgb565 = Rgb565::new(7, 53, 12);
const AMBER: Rgb565 = Rgb565::new(31, 36, 2);
const MUTED: Rgb565 = Rgb565::new(14, 31, 18);

struct PowerMonitor<'a> {
    device: &'a str,
    power: &'a str,
    voltage: &'a str,
    current: &'a str,
    energy: &'a str,
    cost: &'a str,
}

impl Render for PowerMonitor<'_> {
    fn render<'a>(&'a self, _cx: &mut Context) -> impl IntoElement<Element = Element<'a>> {
        column()
            .margin(4)
            .padding(3)
            .border(1)
            .border_color(CYAN)
            .background(CANVAS)
            .size(Size::new(312, 164))
            .child(header(self.device))
            .child(live_power(self.power).margin((0, 0, 2, 0)))
            .child(
                row()
                    .margin((0, 0, 2, 0))
                    .child(metric_card("VOLTAGE", self.voltage, CYAN).margin((0, 2, 0, 0)))
                    .child(metric_card("CURRENT", self.current, GREEN).margin((0, 2, 0, 0)))
                    .child(metric_card("POLL", "5 sec", AMBER)),
            )
            .child(
                row()
                    .child(summary_card("ENERGY / RESET", self.energy, CYAN).margin((0, 2, 0, 0)))
                    .child(summary_card("EST. COST", self.cost, AMBER)),
            )
    }
}

fn header<'a>(device: &'a str) -> Row<'a> {
    row()
        .size(Size::new(304, 20))
        .child(
            text(device)
                .padding(1)
                .size(Size::new(244, 20))
                .font(Font::mono(&FONT_9X18_BOLD))
                .text_color(Rgb565::WHITE),
        )
        .child(
            text("[ ON ]")
                .padding(5)
                .background(GREEN)
                .size(Size::new(60, 20))
                .font(Font::mono(&FONT_6X10))
                .text_color(CANVAS),
        )
}

fn live_power<'a>(power: &'a str) -> Row<'a> {
    row()
        .background(PANEL_HIGHLIGHT)
        .size(Size::new(304, 40))
        .child(
            text("LIVE LOAD")
                .padding(15)
                .size(Size::new(120, 40))
                .font(Font::mono(&FONT_6X10))
                .text_color(CYAN),
        )
        .child(
            text(power)
                .padding(10)
                .background(CYAN)
                .size(Size::new(184, 40))
                .font(Font::mono(&FONT_10X20))
                .text_color(CANVAS),
        )
}

fn metric_card<'a>(label: &'a str, value: &'a str, accent: Rgb565) -> Column<'a> {
    column()
        .padding(3)
        .border(1)
        .border_color(accent)
        .background(PANEL)
        .size(Size::new(100, 46))
        .child(
            text(label)
                .padding(2)
                .size(Size::new(92, 14))
                .font(Font::mono(&FONT_6X10))
                .text_color(MUTED),
        )
        .child(
            text(value)
                .padding(2)
                .size(Size::new(92, 24))
                .font(Font::mono(&FONT_10X20))
                .text_color(accent),
        )
}

fn summary_card<'a>(label: &'a str, value: &'a str, accent: Rgb565) -> Column<'a> {
    column()
        .padding(3)
        .border(1)
        .border_color(PANEL_HIGHLIGHT)
        .background(PANEL)
        .size(Size::new(151, 46))
        .child(
            text(label)
                .padding(2)
                .size(Size::new(143, 14))
                .font(Font::mono(&FONT_6X10))
                .text_color(MUTED),
        )
        .child(
            text(value)
                .padding(2)
                .size(Size::new(143, 24))
                .font(Font::mono(&FONT_10X20))
                .text_color(accent),
        )
}

fn main() {
    let display = SimulatorDisplay::<Rgb565>::new(Size::new(320, 172));
    let monitor = PowerMonitor {
        device: "SHELLY // MINI RACK",
        power: "86.4 W",
        voltage: "232.1 V",
        current: "0.373 A",
        energy: "14.18 kWh",
        cost: "EUR 1.84",
    };

    let storage = FrameStorage::<Rgb565>::default();
    let mut ui = Ui::new(display, storage).with_background(CANVAS);
    ui.render(&monitor).unwrap();

    let output_settings = OutputSettingsBuilder::new().scale(3).build();
    if let Some(path) = env::args_os().nth(1) {
        ui.display().to_rgb_output_image(&output_settings).save_png(path).unwrap();
    } else {
        let title = format!("Guillotine: {}", env!("CARGO_BIN_NAME"));
        Window::new(&title, &output_settings).show_static(ui.display());
    }
}
