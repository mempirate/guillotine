use embedded_graphics::{
    prelude::{Point, Size},
    primitives::Rectangle,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Constraints {
    min: Size,
    max: Size,
}

impl Constraints {
    pub(crate) fn constrain(self, desired: Size) -> Size {
        Size::new(
            desired.width.clamp(self.min.width, self.max.width),
            desired.height.clamp(self.min.height, self.max.height),
        )
    }

    /// Returns [`Constraints`] with zero minimum size and the same maximum size as the original.
    pub(crate) fn loosen(self) -> Self {
        Self { min: Size::zero(), max: self.max }
    }

    /// Returns [`Constraints`] that is exact in both dimensions.
    pub(crate) fn exact(size: Size) -> Self {
        Self { min: size, max: size }
    }

    pub(crate) fn deflate(self, by: u32) -> Self {
        Self {
            min: self.min.saturating_sub(Size::new_equal(by)),
            max: self.max.saturating_sub(Size::new_equal(by)),
        }
    }
}

#[derive(PartialEq, Eq)]
pub(crate) struct Layout {
    /// Position relative to the parent.
    pub(crate) offset: Point,
    /// Position of the border relative to the parent.
    pub(crate) border_offset: Point,
    /// Position of the content relative to the parent.
    pub(crate) content_offset: Point,
    /// Size of the node (after constraints are applied).
    pub(crate) outer_size: Size,
    /// Size of the border.
    pub(crate) border_size: Size,
    /// Size of the actual content.
    pub(crate) content_size: Size,
    /// Absolute bounds of the node.
    pub(crate) bounds: Option<Rectangle>,
}

impl Layout {
    /// Creates an empty unresolved layout.
    pub(crate) const fn empty() -> Self {
        Self {
            offset: Point::new(0, 0),
            border_offset: Point::new(0, 0),
            content_offset: Point::new(0, 0),

            outer_size: Size::zero(),
            border_size: Size::zero(),
            content_size: Size::zero(),

            bounds: None,
        }
    }

    pub(crate) fn set_offset(&mut self, offset: Point) {
        self.offset = offset;
    }

    /// Resolves the absolute bounds of the complete box layout.
    pub(crate) fn resolve(&self, parent_origin: Point) -> BoxLayout {
        let outer_origin = parent_origin + self.offset;
        let border_origin = outer_origin + self.border_offset;
        let content_origin = outer_origin + self.content_offset;

        BoxLayout {
            border: Rectangle::new(border_origin, self.border_size),
            content: Rectangle::new(content_origin, self.content_size),
        }
    }
}

/// Fully resolved box layout. Contains absolute positioned rectangles for margin, border and
/// content.
pub(crate) struct BoxLayout {
    pub(crate) border: Rectangle,
    pub(crate) content: Rectangle,
}
