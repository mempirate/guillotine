mod column;
mod row;
mod text;

use embedded_graphics::{
    Drawable as _,
    prelude::{DrawTarget, PixelColor, Point, Primitive as _, Size},
    primitives::{PrimitiveStyle, Rectangle},
};

pub use column::*;
pub use row::*;
pub use text::*;

use crate::{NodeIndex, Style, layout::BoxLayout};

/// An error encountered while building a frame in fixed-capacity storage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum BuildError {
    /// The frame's node arena has no remaining capacity.
    #[error("node capacity exceeded")]
    NodeCapacity,
    /// The frame's text arena has no remaining capacity.
    #[error("text capacity exceeded")]
    TextCapacity,
}

/// A trait for element builders such as [`RowBuilder`] and [`ColumnBuilder`].
pub trait ElementBuilder {
    /// Finalizes this builder and returns the index of its node in frame storage.
    fn try_build(self) -> Result<NodeIndex, BuildError>;
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

/// This is a helper trait to provide a uniform interface for constructing elements that
/// can accept any number of any kind of child elements
pub trait ParentElement {
    /// Extend this element's children with the given child elements.
    fn extend<E: ElementBuilder>(&mut self, elements: impl IntoIterator<Item = E>);

    /// Add a single child element to this element.
    fn child<E: ElementBuilder>(mut self, child: E) -> Self
    where
        Self: Sized,
    {
        self.extend(core::iter::once(child));
        self
    }

    /// Add multiple child elements of the same type to this element.
    fn children<E: ElementBuilder>(mut self, children: impl IntoIterator<Item = E>) -> Self
    where
        Self: Sized,
    {
        self.extend(children);
        self
    }
}
