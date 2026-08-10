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

/// Style for this row.
#[derive(Default, PartialEq, Eq)]
pub struct RowStyle {}

/// A horizontal container declaration.
///
/// `Vec` is useful while exploring the public API, but it is not intended to be the final
/// storage strategy. An allocation-free implementation will reconcile children directly into
/// caller-provided arena storage through [`crate::Context`].
#[derive(Default, PartialEq, Eq)]
pub struct Row<'a, C = Rgb565, CE = Infallible>
where
    C: PixelColor,
{
    pub(crate) children: Vec<Element<'a, C, CE>>,
    pub(crate) style: Style<RowStyle, C>,
}

impl<C> Row<'_, C>
where
    C: PixelColor,
{
    /// Creates an empty row.
    pub fn new() -> Self {
        Self { children: Vec::new(), style: Style::default() }
    }

    /// Sets the style of this row.
    pub const fn with_style(mut self, style: Style<RowStyle, C>) -> Self {
        self.style = style;
        self
    }

    /// Returns a reference to this row's style.
    pub const fn style(&self) -> &Style<RowStyle, C> {
        &self.style
    }

    /// Draws this row onto the given target, using the provided layout. Does NOT draw children,
    /// that responsibility is delegated to the children themselves.
    pub(crate) fn draw<D>(&self, layout: &BoxLayout, target: &mut D) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = C>,
    {
        draw_box(&self.style, layout, target)
    }
}

impl<'a, C> IntoElement for Row<'a, C>
where
    C: PixelColor,
{
    type Element = Element<'a, C>;

    fn into_element(self) -> Element<'a, C> {
        Element::Row(self)
    }
}

impl<'a, C> ParentElement<'a, C> for Row<'a, C>
where
    C: PixelColor,
{
    fn extend(&mut self, elements: impl IntoIterator<Item = Element<'a, C>>) {
        self.children.extend(elements);
    }
}

impl<C> StyledElement for Row<'_, C>
where
    C: PixelColor,
{
    type Color = C;
    type Specific = RowStyle;

    fn style_mut(&mut self) -> &mut Style<Self::Specific, Self::Color> {
        &mut self.style
    }
}

/// Creates an empty horizontal container.
pub fn row<'a, C>() -> Row<'a, C>
where
    C: PixelColor,
{
    Row::new()
}
