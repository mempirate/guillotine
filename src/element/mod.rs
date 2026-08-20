mod div;
mod text;

pub use div::*;
pub use text::*;

use crate::NodeIndex;

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
