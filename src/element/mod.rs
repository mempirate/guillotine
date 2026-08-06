extern crate alloc;

mod row;
mod text;

use alloc::vec::Vec;
use core::convert::Infallible;

pub use row::*;
pub use text::*;

use crate::style::BoxStyle;

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
pub enum Element<'a, CE = Infallible> {
    /// A horizontal container.
    Row(Row<'a, CE>),
    /// A text element.
    Text(Text<'a>),
    /// A custom element.
    Custom(CE),
}

impl<'a> Element<'a> {
    /// Returns the optional key of this element. The key is used to identify the element during
    /// reconciliation and building a new frame, allowing for incremental redrawing.
    const fn key(&self) -> Option<ElementKey> {
        // None for now. No incremental redrawing yet.
        // TODO: Add incremental redrawing support
        None
    }

    /// Returns the optional children of this element.
    pub fn children(&self) -> Option<&[Element<'a>]> {
        match self {
            Element::Row(row) => Some(&row.children),
            _ => None,
        }
    }

    /// Takes this element's children, setting them to null.
    pub fn take_children(&mut self) -> Option<Vec<Element<'a>>> {
        match self {
            Element::Row(row) => Some(core::mem::take(&mut row.children)),
            _ => None,
        }
    }

    /// Returns the box style of this element.
    pub fn box_style(&self) -> BoxStyle {
        match self {
            Element::Row(row) => row.style().box_style(),
            Element::Text(text) => text.style().box_style(),
            Element::Custom(never) => match *never {},
        }
    }
}

impl<'a> IntoElement for Element<'a> {
    type Element = Element<'a>;

    fn into_element(self) -> Element<'a> {
        self
    }
}

/// This is a helper trait to provide a uniform interface for constructing elements that
/// can accept any number of any kind of child elements
pub trait ParentElement<'a> {
    /// Extend this element's children with the given child elements.
    fn extend(&mut self, elements: impl IntoIterator<Item = Element<'a>>);

    /// Add a single child element to this element.
    fn child(mut self, child: impl IntoElement<Element = Element<'a>>) -> Self
    where
        Self: Sized,
    {
        self.extend(core::iter::once(child.into_element()));
        self
    }

    /// Add multiple child elements to this element.
    fn children(
        mut self,
        children: impl IntoIterator<Item = impl IntoElement<Element = Element<'a>>>,
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
enum ElementKey {
    /// A numeric ID.
    Number(usize),
    /// A static string ID.
    String(&'static str),
}

impl From<usize> for ElementKey {
    fn from(value: usize) -> Self {
        ElementKey::Number(value)
    }
}

impl From<&'static str> for ElementKey {
    fn from(value: &'static str) -> Self {
        ElementKey::String(value)
    }
}
