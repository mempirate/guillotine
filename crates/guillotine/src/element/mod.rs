mod custom;
mod div;
mod text;

pub use custom::*;
pub use div::*;
use embedded_graphics::{
    draw_target::DrawTarget, geometry::Size, pixelcolor::PixelColor, primitives::Rectangle,
};
pub use text::*;

use crate::{NodeIndex, Theme};

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

/// A value that can add an element node to the current frame.
pub trait ElementBuilder {
    /// Finalizes this builder and returns the index of its node in frame storage.
    fn try_build(self) -> Result<NodeIndex, BuildError>;
}

/// A builder for an element that can contain any number of child elements.
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

/// Application-defined leaf content that participates in layout and drawing.
pub trait CustomElement<C: PixelColor> {
    /// Returns the element's natural content size before common style and parent constraints.
    fn intrinsic_size(&self) -> Size;

    /// Draws the element inside its absolute content bounds.
    ///
    /// Guillotine clips `target` to the element's content box and supplies the UI's active
    /// `theme`.
    fn draw<D>(&self, bounds: &Rectangle, theme: &Theme<C>, target: &mut D) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = C>;
}

/// An uninhabited marker used when a UI doesn't support custom elements.
pub enum NoCustomElement {}

impl<C: PixelColor> CustomElement<C> for NoCustomElement {
    fn intrinsic_size(&self) -> Size {
        match *self {}
    }

    fn draw<D>(
        &self,
        _bounds: &Rectangle,
        _theme: &Theme<C>,
        _target: &mut D,
    ) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = C>,
    {
        match *self {}
    }
}
