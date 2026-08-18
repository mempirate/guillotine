use embedded_graphics::{
    Drawable as _,
    draw_target::DrawTarget,
    geometry::{Point, Size},
    mono_font::MonoTextStyleBuilder,
    pixelcolor::PixelColor,
    primitives::{Primitive as _, PrimitiveStyle, Rectangle},
    text::{Baseline, Text},
};

use crate::{
    Font, Style, Theme,
    layout::BoxLayout,
    tree::{NodeKind, TextNode},
};

impl<C: PixelColor> NodeKind<C> {
    pub(crate) fn draw<D>(&self, layout: &BoxLayout, display: &mut D) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = C>,
    {
        match self {
            Self::Row(style) => draw_box(style, layout, display),
            Self::Column(style) => draw_box(style, layout, display),
            _ => unimplemented!("text drawing uses a different code path"),
        }
    }
}

impl<C: PixelColor> TextNode<C> {
    #[inline]
    pub(crate) fn draw<D>(
        &self,
        content: &str,
        layout: &BoxLayout,
        display: &mut D,
        theme: &Theme<C>,
    ) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = C>,
    {
        draw_box(&self.style, layout, display)?;

        match self.style.font {
            Font::Mono(font) => {
                let character_style = MonoTextStyleBuilder::new()
                    .font(font)
                    .text_color(self.style.color.unwrap_or(theme.foreground))
                    .build();

                Text::with_baseline(
                    content,
                    layout.content.top_left,
                    character_style,
                    Baseline::Top,
                )
                .draw(display)?;
            }
        }
        Ok(())
    }
}

/// Draws the common border box shared by all built-in elements.
pub(crate) fn draw_box<S, C, D>(
    style: &Style<S, C>,
    layout: &BoxLayout,
    target: &mut D,
) -> Result<(), D::Error>
where
    S: Default,
    C: PixelColor,
    D: DrawTarget<Color = C>,
{
    if let Some(color) = style.background {
        draw_filled_rectangle(layout.border, color, target)?;
    }

    let Some(color) = style.border_color else {
        return Ok(());
    };

    let origin = layout.border.top_left;
    let size = layout.border.size;
    let top = style.border.top.min(size.height);
    let right = style.border.right.min(size.width);
    let bottom = style.border.bottom.min(size.height);
    let left = style.border.left.min(size.width);

    // Border bands are painted inside the border box. Opposing bands may overlap when the box is
    // smaller than its border widths; all edges share one color, so overlap order is immaterial.
    draw_filled_rectangle(Rectangle::new(origin, Size::new(size.width, top)), color, target)?;
    draw_filled_rectangle(
        Rectangle::new(
            Point::new(
                saturating_coordinate_add(origin.x, size.width.saturating_sub(right)),
                origin.y,
            ),
            Size::new(right, size.height),
        ),
        color,
        target,
    )?;
    draw_filled_rectangle(
        Rectangle::new(
            Point::new(
                origin.x,
                saturating_coordinate_add(origin.y, size.height.saturating_sub(bottom)),
            ),
            Size::new(size.width, bottom),
        ),
        color,
        target,
    )?;
    draw_filled_rectangle(Rectangle::new(origin, Size::new(left, size.height)), color, target)
}

fn draw_filled_rectangle<C, D>(
    rectangle: Rectangle,
    color: C,
    target: &mut D,
) -> Result<(), D::Error>
where
    C: PixelColor,
    D: DrawTarget<Color = C>,
{
    if rectangle.size.width == 0 || rectangle.size.height == 0 {
        return Ok(());
    }

    rectangle.into_styled(PrimitiveStyle::with_fill(color)).draw(target)
}

const fn saturating_coordinate_add(coordinate: i32, offset: u32) -> i32 {
    coordinate.saturating_add(if offset > i32::MAX as u32 { i32::MAX } else { offset as i32 })
}
