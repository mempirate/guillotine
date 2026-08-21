//! A compact flexbox showcase for a 320 × 240 display.

use embedded_graphics::{
    mono_font::ascii::{FONT_6X10, FONT_9X18_BOLD},
    pixelcolor::Rgb565,
    prelude::{RgbColor as _, Size},
};
use embedded_graphics_simulator::{OutputSettingsBuilder, SimulatorDisplay, Window};
use guillotine::{
    style::{JustifyContent, StyledElement},
    *,
};

const CANVAS: Rgb565 = Rgb565::new(1, 3, 5);
const PANEL: Rgb565 = Rgb565::new(3, 7, 10);
const PANEL_RAISED: Rgb565 = Rgb565::new(4, 10, 14);
const EDGE: Rgb565 = Rgb565::new(7, 17, 20);
const MUTED: Rgb565 = Rgb565::new(15, 32, 27);
const CYAN: Rgb565 = Rgb565::new(4, 48, 29);
const GREEN: Rgb565 = Rgb565::new(8, 52, 17);
const AMBER: Rgb565 = Rgb565::new(30, 43, 5);

struct FlexboxShowcase;

impl Render for FlexboxShowcase {
    fn render(&self, cx: &Context<'_>) -> impl ElementBuilder {
        cx.column()
            .padding(8)
            .gap(8)
            .size(Size::new(320, 240))
            .background(CANVAS)
            .child(header(cx))
            .child(
                cx.row()
                    .height(48)
                    .justify_content(JustifyContent::SpaceBetween)
                    .child(stat(cx, "DIRECTION", "ROW", CYAN))
                    .child(stat(cx, "ITEMS", "06", GREEN))
                    .child(stat(cx, "GAP", "02 PX", AMBER)),
            )
            .child(
                cx.row()
                    .height(136)
                    .justify_content(JustifyContent::SpaceBetween)
                    .child(sidebar(cx))
                    .child(justify_panel(cx)),
            )
    }
}

fn header(
    cx: &Context<'_>,
) -> impl ElementBuilder + StyledElement<Color = Rgb565, Specific = DivStyle> {
    cx.row()
        .padding((3, 6))
        .background(PANEL)
        .justify_content(JustifyContent::SpaceBetween)
        .child(cx.text("FLEX LAB").font(Font::mono(&FONT_9X18_BOLD)).text_color(Rgb565::WHITE))
        .child(
            cx.text("LIVE")
                .padding((4, 10))
                .width(48)
                .background(CYAN)
                .font(Font::mono(&FONT_6X10))
                .text_color(CANVAS),
        )
}

fn stat(
    cx: &Context<'_>,
    label: &str,
    value: &str,
    accent: Rgb565,
) -> impl ElementBuilder + StyledElement<Color = Rgb565, Specific = DivStyle> {
    cx.column()
        .padding((4, 6))
        .border(1)
        .border_color(accent)
        .width(96)
        .background(PANEL_RAISED)
        .justify_content(JustifyContent::SpaceBetween)
        .child(cx.text(label).font(Font::mono(&FONT_6X10)).text_color(MUTED))
        .child(cx.text(value).font(Font::mono(&FONT_9X18_BOLD)).text_color(Rgb565::WHITE))
}

fn sidebar(
    cx: &Context<'_>,
) -> impl ElementBuilder + StyledElement<Color = Rgb565, Specific = DivStyle> {
    cx.column()
        .padding(6)
        .border(1)
        .border_color(EDGE)
        .width(82)
        .background(PANEL)
        .justify_content(JustifyContent::SpaceBetween)
        .child(cx.text("MODULES").font(Font::mono(&FONT_6X10)).text_color(MUTED))
        .child(nav_item(cx, "LAYOUT", true))
        .child(nav_item(cx, "STYLE", false))
        .child(nav_item(cx, "PAINT", false))
        .child(cx.text("v0.2").font(Font::mono(&FONT_6X10)).text_color(MUTED))
}

fn nav_item(
    cx: &Context<'_>,
    label: &str,
    active: bool,
) -> impl ElementBuilder + StyledElement<Color = Rgb565, Specific = DivStyle> {
    let (background, color) = if active { (CYAN, CANVAS) } else { (PANEL_RAISED, Rgb565::WHITE) };

    cx.row()
        .padding((7, 6))
        .background(background)
        .child(cx.text(label).font(Font::mono(&FONT_6X10)).text_color(color))
}

fn justify_panel(
    cx: &Context<'_>,
) -> impl ElementBuilder + StyledElement<Color = Rgb565, Specific = DivStyle> {
    cx.column()
        .padding(6)
        .border(1)
        .border_color(EDGE)
        .gap(2)
        .width(214)
        .background(PANEL)
        .child(cx.text("JUSTIFY CONTENT").font(Font::mono(&FONT_6X10)).text_color(MUTED))
        .child(justify_lane(cx, "START", JustifyContent::Start))
        .child(justify_lane(cx, "END", JustifyContent::End))
        .child(justify_lane(cx, "CENTER", JustifyContent::Center))
        .child(justify_lane(cx, "BETWEEN", JustifyContent::SpaceBetween))
        .child(justify_lane(cx, "AROUND", JustifyContent::SpaceAround))
        .child(justify_lane(cx, "EVENLY", JustifyContent::SpaceEvenly))
}

fn justify_lane(
    cx: &Context<'_>,
    label: &str,
    justify: JustifyContent,
) -> impl ElementBuilder + StyledElement<Color = Rgb565, Specific = DivStyle> {
    cx.row()
        .padding((3, 5))
        .background(PANEL_RAISED)
        .justify_content(JustifyContent::SpaceBetween)
        .child(cx.text(label).font(Font::mono(&FONT_6X10)).text_color(Rgb565::WHITE))
        .child(
            cx.row()
                .gap(2)
                .width(140)
                .justify_content(justify)
                .child(swatch(cx, CYAN))
                .child(swatch(cx, GREEN))
                .child(swatch(cx, AMBER)),
        )
}

fn swatch(
    cx: &Context<'_>,
    color: Rgb565,
) -> impl ElementBuilder + StyledElement<Color = Rgb565, Specific = DivStyle> {
    cx.div().size(Size::new(10, 10)).background(color)
}

fn main() {
    let display = SimulatorDisplay::<Rgb565>::new(Size::new(320, 240));
    let storage = FrameStorage::<Rgb565>::default();
    let mut ui = Ui::new(DirectTarget::new(display), storage).with_background(CANVAS);

    ui.render(&FlexboxShowcase).unwrap();

    let output_settings = OutputSettingsBuilder::new().scale(3).build();
    let title = format!("Guillotine: {}", env!("CARGO_BIN_NAME"));
    Window::new(&title, &output_settings).show_static(ui.display());
}
