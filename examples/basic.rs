//! Basic static UI example.
use embedded_graphics::{
    mono_font::ascii::FONT_9X18_BOLD,
    pixelcolor::Rgb565,
    prelude::{RgbColor as _, Size},
};
use embedded_graphics_simulator::{OutputSettingsBuilder, SimulatorDisplay, Window};
use guillotine::{style::StyledElement as _, *};

struct BasicView {}

impl Render for BasicView {
    fn render<'a>(&'a self, _cx: &mut Context) -> impl IntoElement<Element = Element<'a>> {
        row()
            .padding(50)
            .margin(10)
            .border(5)
            .border_color(Rgb565::BLUE)
            .child(text("Hello, World!").background(Rgb565::RED).margin(20))
            .child(text("GUILLOTINE").margin(20).font(Font::mono(&FONT_9X18_BOLD)))
    }
}

fn main() {
    let display = SimulatorDisplay::<Rgb565>::new(Size::new(800, 600));

    let view = BasicView {};

    let mut ui = Ui::new(display);

    ui.render(&view);

    let output_settings = OutputSettingsBuilder::new().build();
    Window::new("Guillotine: Basic", &output_settings).show_static(ui.display());
}
