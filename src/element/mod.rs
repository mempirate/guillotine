extern crate alloc;

mod column;
mod row;
mod text;

use alloc::vec::Vec;
use core::convert::Infallible;
use embedded_graphics::{
    Drawable as _,
    pixelcolor::Rgb565,
    prelude::{DrawTarget, PixelColor, Primitive as _},
    primitives::PrimitiveStyleBuilder,
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
    pub(crate) fn children(&self) -> Option<&[Element<'a, C>]> {
        match self {
            Element::Column(column) => Some(&column.children),
            Element::Row(row) => Some(&row.children),
            _ => None,
        }
    }

    /// Takes this element's children, setting them to null.
    pub(crate) fn take_children(&mut self) -> Option<Vec<Element<'a, C>>> {
        match self {
            Element::Column(column) => Some(core::mem::take(&mut column.children)),
            Element::Row(row) => Some(core::mem::take(&mut row.children)),
            _ => None,
        }
    }

    /// Returns the box style of this element.
    pub(crate) fn box_style(&self) -> BoxStyle {
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
    let mut primitive_style = PrimitiveStyleBuilder::new().stroke_width(style.border);

    if let Some(color) = style.border_color {
        primitive_style = primitive_style.stroke_color(color);
    }

    if let Some(color) = style.background {
        primitive_style = primitive_style.fill_color(color);
    }

    layout.border.into_styled(primitive_style.build()).draw(target)
}

impl<'a, C> IntoElement for Element<'a, C>
where
    C: PixelColor,
{
    type Element = Element<'a, C>;

    fn into_element(self) -> Element<'a, C> {
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
        ElementKey::Number(value)
    }
}

impl From<&'static str> for ElementKey {
    fn from(value: &'static str) -> Self {
        ElementKey::String(value)
    }
}
