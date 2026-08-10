extern crate alloc;

use core::convert::Infallible;

use alloc::vec::Vec;
use embedded_graphics::{pixelcolor::Rgb565, prelude::DrawTarget};

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
pub struct Column<'a, CE = Infallible> {
    pub(crate) children: Vec<Element<'a, CE>>,
    pub(crate) style: Style<ColumnStyle>,
}

impl Column<'_> {
    /// Creates an empty column.
    pub fn new() -> Self {
        Self { children: Vec::new(), style: Style::default() }
    }

    /// Sets the style of this column.
    pub const fn with_style(mut self, style: Style<ColumnStyle>) -> Self {
        self.style = style;
        self
    }

    /// Returns a reference to this column's style.
    pub const fn style(&self) -> &Style<ColumnStyle> {
        &self.style
    }

    /// Draws this column onto the given target, using the provided layout. Does NOT draw children,
    /// that responsibility is delegated to the children themselves.
    pub(crate) fn draw<D>(&self, layout: &BoxLayout, target: &mut D) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        draw_box(&self.style, layout, target)
    }
}

impl<'a> IntoElement for Column<'a> {
    type Element = Element<'a>;

    fn into_element(self) -> Element<'a> {
        Element::Column(self)
    }
}

impl<'a> ParentElement<'a> for Column<'a> {
    fn extend(&mut self, elements: impl IntoIterator<Item = Element<'a>>) {
        self.children.extend(elements);
    }
}

impl<'a> StyledElement for Column<'a> {
    type Specific = ColumnStyle;

    fn style_mut(&mut self) -> &mut Style<Self::Specific> {
        &mut self.style
    }
}

/// Creates an empty vertical container.
pub fn column<'a>() -> Column<'a> {
    Column::new()
}
