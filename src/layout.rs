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

    /// Resolves the absolute bounds of the complete outer box.
    pub(crate) fn set_absolute_bounds(&mut self, parent_origin: Point) {
        let origin = parent_origin + self.offset;

        self.bounds = Some(Rectangle::new(origin, self.outer_size));
    }

    pub(crate) fn set_offset(&mut self, offset: Point) {
        self.offset = offset;
    }

    pub(crate) fn origin(&self) -> Option<Point> {
        self.bounds.map(|bounds| bounds.top_left)
    }
}
