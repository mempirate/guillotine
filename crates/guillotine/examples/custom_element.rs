//! Custom element example that draws a battery gauge with embedded-graphics primitives.

use embedded_graphics::{
    Drawable as _,
    geometry::{Point, Size},
    mono_font::ascii::FONT_6X10,
    pixelcolor::Rgb565,
    primitives::{Primitive as _, PrimitiveStyle, Rectangle},
};
use embedded_graphics_simulator::{OutputSettingsBuilder, SimulatorDisplay, Window};
use guillotine::{style::StyledElement as _, *};

type Color = Rgb565;

const CANVAS: Color = Color::new(2, 4, 6);
const PANEL: Color = Color::new(4, 9, 12);
const ACCENT: Color = Color::new(7, 47, 25);
const WARNING: Color = Color::new(31, 20, 2);

/// All custom element kinds used by this application.
enum AppElement {
    Battery(BatteryGauge),
}

impl From<BatteryGauge> for AppElement {
    fn from(gauge: BatteryGauge) -> Self {
        Self::Battery(gauge)
    }
}

impl CustomElement<Color> for AppElement {
    fn intrinsic_size(&self) -> Size {
        match self {
            Self::Battery(gauge) => gauge.intrinsic_size(),
        }
    }

    fn draw<D>(
        &self,
        bounds: &Rectangle,
        theme: &Theme<Color>,
        target: &mut D,
    ) -> Result<(), D::Error>
    where
        D: embedded_graphics::draw_target::DrawTarget<Color = Color>,
    {
        match self {
            Self::Battery(gauge) => gauge.draw(bounds, theme, target),
        }
    }
}

/// A custom leaf element that renders a battery outline and its current charge level.
struct BatteryGauge {
    charge_percent: u8,
}

impl BatteryGauge {
    fn new(charge_percent: u8) -> Self {
        Self { charge_percent: charge_percent.min(100) }
    }
}

impl CustomElement<Color> for BatteryGauge {
    fn intrinsic_size(&self) -> Size {
        Size::new(96, 32)
    }

    fn draw<D>(
        &self,
        bounds: &Rectangle,
        theme: &Theme<Color>,
        target: &mut D,
    ) -> Result<(), D::Error>
    where
        D: embedded_graphics::draw_target::DrawTarget<Color = Color>,
    {
        const OUTER_INSET: u32 = 1;
        const TERMINAL_GAP: u32 = 2;
        const TERMINAL_WIDTH: u32 = 4;
        const INNER_INSET: u32 = 4;

        let body_size = bounds
            .size
            .saturating_sub(Size::new(OUTER_INSET * 2 + TERMINAL_GAP + TERMINAL_WIDTH, 2));

        if body_size.width == 0 || body_size.height == 0 {
            return Ok(());
        }

        let body_origin = bounds.top_left + Point::new(OUTER_INSET as i32, OUTER_INSET as i32);
        let body = Rectangle::new(body_origin, body_size);

        body.into_styled(PrimitiveStyle::with_stroke(theme.foreground, 2)).draw(target)?;

        let terminal_height = (body_size.height / 3).max(1);
        let terminal_origin = Point::new(
            body_origin.x + body_size.width as i32 + TERMINAL_GAP as i32,
            body_origin.y + body_size.height.saturating_sub(terminal_height) as i32 / 2,
        );
        target.fill_solid(
            &Rectangle::new(terminal_origin, Size::new(TERMINAL_WIDTH, terminal_height)),
            theme.foreground,
        )?;

        let inner_origin = body_origin + Point::new(INNER_INSET as i32, INNER_INSET as i32);
        let inner_size = body_size.saturating_sub(Size::new(INNER_INSET * 2, INNER_INSET * 2));
        let fill_width = inner_size.width.saturating_mul(self.charge_percent as u32) / 100;
        let fill_color = if self.charge_percent <= 20 { WARNING } else { ACCENT };

        if fill_width != 0 && inner_size.height != 0 {
            target.fill_solid(
                &Rectangle::new(inner_origin, Size::new(fill_width, inner_size.height)),
                fill_color,
            )?;
        }

        Ok(())
    }
}

struct BatteryView {
    charge_percent: u8,
}

impl Render<Color, AppElement> for BatteryView {
    fn render(&self, cx: &Context<'_, Color, AppElement>) -> impl ElementBuilder {
        cx.column()
            .size(Size::new(180, 90))
            .padding(12)
            .gap(8)
            .background(CANVAS)
            .child(cx.text("CUSTOM BATTERY").font(Font::mono(&FONT_6X10)))
            .child(
                cx.custom(BatteryGauge::new(self.charge_percent))
                    .padding(4)
                    .border(1)
                    .border_color(ACCENT)
                    .background(PANEL),
            )
    }
}

fn main() {
    let display = SimulatorDisplay::<Color>::new(Size::new(180, 90));
    let storage = FrameStorage::<Color, 8, 32, AppElement>::default();
    let mut ui =
        Ui::new(DirectTarget::new(display), storage).with_foreground(Color::new(31, 63, 31));

    ui.render(&BatteryView { charge_percent: 72 }).unwrap();

    let output_settings = OutputSettingsBuilder::new().build();
    Window::new("Guillotine: custom element", &output_settings).show_static(ui.display());
}
