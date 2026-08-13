//! Basic static UI example.
use std::env;

use embedded_graphics::{
    mono_font::ascii::FONT_9X18_BOLD,
    pixelcolor::Rgb565,
    prelude::{RgbColor as _, Size},
};
use embedded_graphics_simulator::{OutputSettingsBuilder, SimulatorDisplay, Window};
use guillotine::*;

struct BasicView {}

impl Render for BasicView {
    fn render<'a>(&'a self, _cx: &mut Context) -> impl IntoElement<Element = Element<'a>> {
        column()
            .padding(10)
            .margin(10)
            .border(2)
            .border_color(Rgb565::BLUE)
            .child(text("Hello, World!").background(Rgb565::RED).margin(5))
            .child(text("GUILLOTINE").margin(5).font(Font::mono(&FONT_9X18_BOLD)))
    }
}

fn main() {
    let display = SimulatorDisplay::<Rgb565>::new(Size::new(320, 172));

    let view = BasicView {};

    let mut ui = Ui::new(display);

    ui.render(&view);

    let output_settings = OutputSettingsBuilder::new().build();
    let title = format!("Guillotine: {}", env!("CARGO_BIN_NAME"));
    Window::new(&title, &output_settings).show_static(ui.display());
}
