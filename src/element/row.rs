extern crate alloc;

use core::convert::Infallible;

use alloc::vec::Vec;

use crate::{
    Style,
    element::{Element, IntoElement, ParentElement},
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

/// Creates an empty horizontal container.
pub fn row<'a>() -> Row<'a> {
    Row::new()
}
