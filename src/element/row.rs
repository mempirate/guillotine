extern crate alloc;

use core::convert::Infallible;

use alloc::vec::Vec;
use embedded_graphics::{
    Drawable as _,
    pixelcolor::Rgb565,
    prelude::{DrawTarget, Primitive as _},
    primitives::PrimitiveStyleBuilder,
};

use crate::{
    Style,
    element::{Element, IntoElement, ParentElement},
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
/// caller-provided arena storage through [`Context`].
#[derive(Default, PartialEq, Eq)]
pub struct Row<'a, CE = Infallible> {
    pub(crate) children: Vec<Element<'a, CE>>,
    pub(crate) style: Style<RowStyle>,
}

impl Row<'_> {
    /// Creates an empty row.
    pub fn new() -> Self {
        Self { children: Vec::new(), style: Style::default() }
    }

    /// Sets the style of this row.
    pub const fn with_style(mut self, style: Style<RowStyle>) -> Self {
        self.style = style;
        self
    }

    /// Returns a reference to this row's style.
    pub fn style(&self) -> &Style<RowStyle> {
        &self.style
    }

    /// Draws this row onto the given target, using the provided layout. Does NOT draw children,
    /// that responsibility is delegated to the children themselves.
    pub(crate) fn draw<D>(&self, layout: &BoxLayout, target: &mut D) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        // Build the primitive style for the border.
        let mut style = PrimitiveStyleBuilder::new().stroke_width(self.style.border);

        if let Some(color) = self.style.border_color {
            style = style.stroke_color(color);
        }

        if let Some(color) = self.style.background {
            style = style.fill_color(color);
        }

        // Draw the border box.
        layout.border.into_styled(style.build()).draw(target)?;

        Ok(())
    }
}

impl<'a> IntoElement for Row<'a> {
    type Element = Element<'a>;

    fn into_element(self) -> Element<'a> {
        Element::Row(self)
    }
}

impl<'a> ParentElement<'a> for Row<'a> {
    fn extend(&mut self, elements: impl IntoIterator<Item = Element<'a>>) {
        self.children.extend(elements);
    }
}

impl<'a> StyledElement for Row<'a> {
    type Specific = RowStyle;

    fn style_mut(&mut self) -> &mut Style<Self::Specific> {
        &mut self.style
    }
}

/// Creates an empty horizontal container.
pub fn row<'a>() -> Row<'a> {
    Row::new()
}
