extern crate alloc;

mod column;
mod row;
mod text;

use alloc::vec::Vec;
use core::convert::Infallible;
use embedded_graphics::{
    Drawable as _,
    pixelcolor::Rgb565,
    prelude::{DrawTarget, PixelColor, Point, Primitive as _, Size},
    primitives::{PrimitiveStyle, Rectangle},
};

pub use column::*;
pub use row::*;
pub use text::*;

use crate::{Style, Theme, layout::BoxLayout, style::BoxStyle};

/// A value that can be converted into Guillotine's closed element enum.
pub trait IntoElement {
    /// The element type produced by this value.
    type Element;

    /// Converts this value into an element declaration.
    fn into_element(self) -> Self::Element;
}

/// An ephemeral element declaration produced while rendering a frame.
///
/// This is deliberately a closed enum rather than a type-erased `AnyElement`. It is consumed
/// during reconciliation and must not be retained beyond the render call: variants such as
/// [`Text`] may borrow their content from application state.
#[derive(PartialEq, Eq)]
pub enum Element<'a, C = Rgb565, CE = Infallible>
where
    C: PixelColor,
{
    /// A vertical container.
    Column(Column<'a, C, CE>),
    /// A horizontal container.
    Row(Row<'a, C, CE>),
    /// A text element.
    Text(Text<'a, C>),
    /// A custom element.
    Custom(CE),
}

impl<'a, C> Element<'a, C>
where
    C: PixelColor,
{
    /// Returns the optional key of this element. The key is used to identify the element during
    /// reconciliation and building a new frame, allowing for incremental redrawing.
    #[allow(unused)]
    pub(crate) const fn key(&self) -> Option<ElementKey> {
        // None for now. No incremental redrawing yet.
        // TODO: Add incremental redrawing support
        None
    }

    /// Returns the optional children of this element.
    #[allow(unused)]
    pub(crate) fn children(&self) -> Option<&[Self]> {
        match self {
            Element::Column(column) => Some(&column.children),
            Element::Row(row) => Some(&row.children),
            _ => None,
        }
    }

    /// Takes this element's children, setting them to null.
    pub(crate) fn take_children(&mut self) -> Option<Vec<Self>> {
        match self {
            Element::Column(column) => Some(core::mem::take(&mut column.children)),
            Element::Row(row) => Some(core::mem::take(&mut row.children)),
            _ => None,
        }
    }

    /// Returns the box style of this element.
    pub(crate) const fn box_style(&self) -> BoxStyle {
        match self {
            Element::Column(column) => column.style().box_style(),
            Element::Row(row) => row.style().box_style(),
            Element::Text(text) => text.style().box_style(),
            Element::Custom(never) => match *never {},
        }
    }

    pub(crate) fn draw<D>(
        &self,
        layout: &BoxLayout,
        target: &mut D,
        theme: &Theme<C>,
    ) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = C>,
    {
        match self {
            Element::Column(column) => column.draw(layout, target),
            Element::Row(row) => row.draw(layout, target),
            Element::Text(text) => text.draw(layout, target, theme),
            Element::Custom(_) => unimplemented!("custom elements are not implemented"),
        }
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

impl<'a, C> IntoElement for Element<'a, C>
where
    C: PixelColor,
{
    type Element = Self;

    fn into_element(self) -> Self {
        self
    }
}

/// This is a helper trait to provide a uniform interface for constructing elements that
/// can accept any number of any kind of child elements
pub trait ParentElement<'a, C>
where
    C: PixelColor,
{
    /// Extend this element's children with the given child elements.
    fn extend(&mut self, elements: impl IntoIterator<Item = Element<'a, C>>);

    /// Add a single child element to this element.
    fn child(mut self, child: impl IntoElement<Element = Element<'a, C>>) -> Self
    where
        Self: Sized,
    {
        self.extend(core::iter::once(child.into_element()));
        self
    }

    /// Add multiple child elements to this element.
    fn children(
        mut self,
        children: impl IntoIterator<Item = impl IntoElement<Element = Element<'a, C>>>,
    ) -> Self
    where
        Self: Sized,
    {
        self.extend(children.into_iter().map(|child| child.into_element()));
        self
    }
}

/// User-provided, persistent element keys for cross-frame tracking.
#[derive(PartialEq, Eq)]
pub(crate) enum ElementKey {
    /// A numeric ID.
    Number(usize),
    /// A static string ID.
    String(&'static str),
}

impl From<usize> for ElementKey {
    fn from(value: usize) -> Self {
        Self::Number(value)
    }
}

impl From<&'static str> for ElementKey {
    fn from(value: &'static str) -> Self {
        Self::String(value)
    }
}
