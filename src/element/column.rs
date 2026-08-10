extern crate alloc;

use core::convert::Infallible;

use alloc::vec::Vec;
use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::{DrawTarget, PixelColor},
};

use crate::{
    Style,
    element::{Element, IntoElement, ParentElement, draw_box},
    layout::BoxLayout,
    style::StyledElement,
};

/// Style for this column.
#[derive(Default, PartialEq, Eq)]
pub struct ColumnStyle {}

/// A vertical container declaration.
///
/// `Vec` is useful while exploring the public API, but it is not intended to be the final
/// storage strategy. An allocation-free implementation will reconcile children directly into
/// caller-provided arena storage through [`crate::Context`].
#[derive(Default, PartialEq, Eq)]
pub struct Column<'a, C = Rgb565, CE = Infallible>
where
    C: PixelColor,
{
    pub(crate) children: Vec<Element<'a, C, CE>>,
    pub(crate) style: Style<ColumnStyle, C>,
}

impl<C> Column<'_, C>
where
    C: PixelColor,
{
    /// Creates an empty column.
    pub fn new() -> Self {
        Self { children: Vec::new(), style: Style::default() }
    }

    /// Sets the style of this column.
    pub const fn with_style(mut self, style: Style<ColumnStyle, C>) -> Self {
        self.style = style;
        self
    }

    /// Returns a reference to this column's style.
    pub const fn style(&self) -> &Style<ColumnStyle, C> {
        &self.style
    }

    /// Draws this column onto the given target, using the provided layout. Does NOT draw children,
    /// that responsibility is delegated to the children themselves.
    pub(crate) fn draw<D>(&self, layout: &BoxLayout, target: &mut D) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = C>,
    {
        draw_box(&self.style, layout, target)
    }
}

impl<'a, C> IntoElement for Column<'a, C>
where
    C: PixelColor,
{
    type Element = Element<'a, C>;

    fn into_element(self) -> Element<'a, C> {
        Element::Column(self)
    }
}

impl<'a, C> ParentElement<'a, C> for Column<'a, C>
where
    C: PixelColor,
{
    fn extend(&mut self, elements: impl IntoIterator<Item = Element<'a, C>>) {
        self.children.extend(elements);
    }
}

impl<C> StyledElement for Column<'_, C>
where
    C: PixelColor,
{
    type Color = C;
    type Specific = ColumnStyle;

    fn style_mut(&mut self) -> &mut Style<Self::Specific, Self::Color> {
        &mut self.style
    }
}

/// Creates an empty vertical container.
pub fn column<'a, C>() -> Column<'a, C>
where
    C: PixelColor,
{
    Column::new()
}
