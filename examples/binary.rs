//! Static UI example using a binary-color display.
use std::env;

use embedded_graphics::{pixelcolor::BinaryColor, prelude::Size};
use embedded_graphics_simulator::{OutputSettingsBuilder, SimulatorDisplay, Window};
use guillotine::{style::StyledElement as _, *};

type Color = BinaryColor;

struct BinaryView;

impl Render<Color> for BinaryView {
    fn render(&self, cx: &Context<'_, Color>) -> impl ElementBuilder {
        cx.column()
            .padding(10)
            .margin(10)
            .border(2)
            .border_color(Color::On)
            .child(cx.text("Generic PixelColor").margin(5))
            .child(cx.text("Binary display").margin(5).text_color(Color::On))
    }
}

fn main() {
    let display = SimulatorDisplay::<Color>::new(Size::new(320, 172));
    let storage = FrameStorage::<Color>::default();
    println!("storage size: {}", storage.size());

    let mut ui = Ui::new(display, storage);

    ui.render(&BinaryView).unwrap();

    let output_settings = OutputSettingsBuilder::new().build();
    let title = format!("Guillotine: {}", env!("CARGO_BIN_NAME"));
    Window::new(&title, &output_settings).show_static(ui.display());
}
