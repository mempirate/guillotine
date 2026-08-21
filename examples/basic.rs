//! Basic flexbox UI example.
use std::env;

use embedded_graphics::{
    mono_font::ascii::{FONT_6X10, FONT_9X18_BOLD},
    pixelcolor::Rgb565,
    prelude::Size,
};
use embedded_graphics_simulator::{OutputSettingsBuilder, SimulatorDisplay, Window};
use guillotine::{style::JustifyContent, *};

const CANVAS: Rgb565 = Rgb565::new(2, 4, 6);
const PANEL: Rgb565 = Rgb565::new(4, 9, 12);
const ACCENT: Rgb565 = Rgb565::new(7, 47, 25);

struct BasicView {
    greeting: &'static str,
}

impl Render for BasicView {
    // Render lets you declaratively build your UI tree.
    fn render(&self, cx: &Context<'_>) -> impl ElementBuilder {
        cx.column()
            .size(Size::new(320, 172))
            .padding(12)
            .gap(8)
            .background(CANVAS)
            .justify_content(JustifyContent::Center)
            .child(
                cx.row()
                    .child(cx.text("GUILLOTINE").flex_grow(1).font(Font::mono(&FONT_9X18_BOLD)))
                    .child(
                        cx.text("READY")
                            .padding((4, 9))
                            .background(ACCENT)
                            .font(Font::mono(&FONT_6X10)),
                    ),
            )
            .child(
                cx.row()
                    .gap(8)
                    .child(
                        cx.text(self.greeting)
                            .font(Font::mono(&FONT_6X10))
                            .flex(3)
                            .padding(10)
                            .background(PANEL),
                    )
                    .child(
                        cx.text("7 nodes, 54 bytes")
                            .font(Font::mono(&FONT_6X10))
                            .flex(2)
                            .padding(6)
                            .background(ACCENT),
                    ),
            )
    }
}

fn main() {
    // Display should implement embedded_graphics DrawTarget
    let display = SimulatorDisplay::<Rgb565>::new(Size::new(320, 172));

    let view = BasicView { greeting: "Build tiny interfaces." };

    // Initialize stack-based storage for the frame. Capacity: 32 elements
    // and 128 bytes of UTF-8 text.
    let storage = FrameStorage::<Rgb565, 32, 128>::default();

    // Create a new UI with a direct display target. Used here for brevity;
    // if you have memory available, use `BufferedTarget`.
    let mut ui = Ui::new(DirectTarget::new(display), storage).with_background(CANVAS);

    // Render the view
    let start = std::time::Instant::now();
    ui.render(&view).unwrap();
    println!("render time: {:?}", start.elapsed());

    let output_settings = OutputSettingsBuilder::new().build();
    let title = format!("Guillotine: {}", env!("CARGO_BIN_NAME"));
    Window::new(&title, &output_settings).show_static(ui.display());
}
