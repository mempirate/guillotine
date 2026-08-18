//! Common types, traits and utilities.

use embedded_graphics::geometry::Size;
pub(crate) type NodeIndex = usize;

pub(crate) trait SizeExt {
    /// Inflates the size by a horizontal and vertical amount.
    fn inflate(self, by: Size) -> Self;

    /// Deflates the size by a horizontal and vertical amount.
    fn deflate(self, by: Size) -> Self;
}

impl SizeExt for Size {
    fn inflate(self, by: Size) -> Self {
        Self::new(self.width.saturating_add(by.width), self.height.saturating_add(by.height))
    }

    fn deflate(self, by: Size) -> Self {
        self.saturating_sub(by)
    }
}

/// Converts a `u32` value to an `i32`, saturating at `i32::MAX` if the value is too large.
pub(crate) const fn to_i32(value: u32) -> i32 {
    if value > i32::MAX as u32 { i32::MAX } else { value as i32 }
}

#[derive(Clone, Copy)]
pub(crate) struct TextRange {
    pub offset: usize,
    pub len: usize,
}
