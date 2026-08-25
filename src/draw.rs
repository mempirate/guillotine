use embedded_graphics::{
    Drawable as _,
    draw_target::{DrawTarget, DrawTargetExt},
    geometry::{Point, Size},
    mono_font::MonoTextStyleBuilder,
    pixelcolor::PixelColor,
    primitives::Rectangle,
    text::{Baseline, Text},
};

use crate::{
    Font, Style, Theme,
    common::to_i32,
    element::CustomElement,
    layout::BorderBox,
    tree::{NodeKind, TextNode},
};

impl<C, CE> NodeKind<C, CE>
where
    C: PixelColor,
    CE: CustomElement<C>,
{
    pub(crate) fn draw<D>(
        &self,
        border_box: &BorderBox,
        theme: &Theme<C>,
        target: &mut D,
    ) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = C>,
    {
        match self {
            Self::Div(style) => border_box.draw(style, target),
            Self::Custom(style, element) => {
                // First draw the border box, then the custom element content.
                border_box.draw(style, target)?;

                // Clip the target to the content area.
                let mut target = target.clipped(&border_box.content);

                element.draw(&border_box.content, theme, &mut target)
            }
            _ => unimplemented!("text drawing uses a different code path"),
        }
    }
}

impl<C: PixelColor> TextNode<C> {
    #[inline]
    pub(crate) fn draw<D>(
        &self,
        content: &str,
        border_box: &BorderBox,
        theme: &Theme<C>,
        target: &mut D,
    ) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = C>,
    {
        border_box.draw(&self.style, target)?;

        match self.style.font {
            Font::Mono(font) => {
                let character_style = MonoTextStyleBuilder::new()
                    .font(font)
                    .text_color(self.style.color.unwrap_or(theme.foreground))
                    .build();

                Text::with_baseline(
                    content,
                    border_box.content.top_left,
                    character_style,
                    Baseline::Top,
                )
                .draw(target)?;
            }
        }
        Ok(())
    }
}

impl BorderBox {
    /// Draws the common border box shared by all elements.
    fn draw<D, S, C>(&self, style: &Style<S, C>, target: &mut D) -> Result<(), D::Error>
    where
        S: Default,
        D: DrawTarget<Color = C>,
        C: PixelColor,
    {
        if let Some(color) = style.background {
            fill_rectangle(self.border, color, target)?;
        }

        let Some(color) = style.border_color else {
            return Ok(());
        };

        let origin = self.border.top_left;
        let size = self.border.size;

        let top = style.border.top.min(size.height);
        let right = style.border.right.min(size.width);
        let bottom = style.border.bottom.min(size.height);
        let left = style.border.left.min(size.width);

        // Construct the border bands.
        let bands = [
            // x, y, width, height
            (0, 0, size.width, top),
            (size.width.saturating_sub(right), 0, right, size.height),
            (0, size.height.saturating_sub(bottom), size.width, bottom),
            (0, 0, left, size.height),
        ];

        for (x, y, width, height) in bands {
            // Compute the band's absolute origin.
            let top_left =
                Point::new(origin.x.saturating_add(to_i32(x)), origin.y.saturating_add(to_i32(y)));

            fill_rectangle(Rectangle::new(top_left, Size::new(width, height)), color, target)?;
        }

        Ok(())
    }
}

/// Fills a rectangle with the given color and draws it to the target.
fn fill_rectangle<C, D>(rectangle: Rectangle, color: C, target: &mut D) -> Result<(), D::Error>
where
    C: PixelColor,
    D: DrawTarget<Color = C>,
{
    if rectangle.size.width == 0 || rectangle.size.height == 0 {
        return Ok(());
    }

    target.fill_solid(&rectangle, color)
}
