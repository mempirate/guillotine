use embedded_graphics::{pixelcolor::Rgb565, prelude::Size};

use crate::Constraints;

/// Type for top, left, bottom and right values.
#[derive(Clone, Copy)]
pub struct Insets {
    pub top: u32,
    pub left: u32,
    pub bottom: u32,
    pub right: u32,
}

impl Insets {
    /// Create zero insets.
    pub fn zero() -> Self {
        Self { top: 0, left: 0, bottom: 0, right: 0 }
    }

    /// Create uniform insets from the given `value`.
    pub fn uniform(value: u32) -> Self {
        Self::from(value)
    }
}

impl From<u32> for Insets {
    /// Converts a `u32` value to uniform [`Insets`].
    fn from(value: u32) -> Self {
        Self { top: value, left: value, bottom: value, right: value }
    }
}

impl From<usize> for Insets {
    /// Converts a `usize` value to uniform [`Insets`].
    fn from(value: usize) -> Self {
        let value = value as u32;
        Self::from(value)
    }
}

impl From<(u32, u32)> for Insets {
    /// Converts a `(u32, u32)` pair to [`Insets`], interpreting it as (vertical, horizontal).
    fn from((y, x): (u32, u32)) -> Self {
        Self { top: y, left: x, bottom: y, right: x }
    }
}

impl From<(usize, usize)> for Insets {
    /// Converts a `(usize, usize)` pair to `Insets`, interpreting it as (vertical, horizontal).
    fn from((y, x): (usize, usize)) -> Self {
        let value = (y as u32, x as u32);
        Self::from(value)
    }
}

impl From<(u32, u32, u32, u32)> for Insets {
    /// Converts a `(u32, u32, u32, u32)` quad to [`Insets`], interpreting it as (top, left, bottom,
    /// right).
    fn from((top, left, bottom, right): (u32, u32, u32, u32)) -> Self {
        Self { top, left, bottom, right }
    }
}

impl From<(usize, usize, usize, usize)> for Insets {
    /// Converts a `(usize, usize, usize, usize)` quad to [`Insets`], interpreting it as (top, left,
    /// bottom, right).
    fn from((top, left, bottom, right): (usize, usize, usize, usize)) -> Self {
        let value = (top as u32, left as u32, bottom as u32, right as u32);
        Self::from(value)
    }
}

impl Insets {
    /// Returns the horizontal insets (left + right).
    pub const fn horizontal(&self) -> u32 {
        self.left + self.right
    }

    /// Returns the vertical insets (top + bottom).
    pub const fn vertical(&self) -> u32 {
        self.top + self.bottom
    }
}

pub struct BoxStyle {
    pub margin: u32,
    pub border: u32,
    pub padding: u32,
    pub size: Option<Size>,
}

impl BoxStyle {
    /// Returns the constraints for the border box by subtracting the margin from
    /// the given constraints.
    pub(crate) fn border_constraints(&self, constraints: Constraints) -> Constraints {
        constraints.deflate(self.margin.saturating_mul(2))
    }

    /// Returns loose constraints for the contents of a box, which lives inside the border
    /// and padding.
    pub(crate) fn content_constraints(&self, constraints: Constraints) -> Constraints {
        let border_constraints = self.border_constraints(constraints);

        let border_constraints = match self.size {
            Some(size) => {
                let border_size = border_constraints.constrain(size);

                Constraints::exact(border_size)
            }
            None => border_constraints,
        };

        let content_inset = self.border.saturating_add(self.padding);

        border_constraints.deflate(content_inset.saturating_mul(2)).loosen()
    }

    /// Returns the content inset, i.e. `border + padding`.
    pub(crate) fn content_inset(&self) -> u32 {
        self.border.saturating_add(self.padding)
    }
}

/// Common style type shared between all [`Element`] variants.
#[derive(Default, PartialEq, Eq)]
pub struct Style<S: Default, C = Rgb565> {
    // Common properties (including Box-model properties).
    // TODO: Replace with `Insets`
    pub margin: u32,
    pub padding: u32,
    pub border: u32,
    pub border_color: Option<C>,
    pub background: Option<C>,
    pub size: Option<Size>,
    // Specific style properties for each node kind.
    pub specific: S,
}

impl<S: Default, C> Style<S, C> {
    /// Derive a [`BoxStyle`] for layout.
    pub fn box_style(&self) -> BoxStyle {
        BoxStyle {
            margin: self.margin,
            padding: self.padding,
            border: self.border,
            size: self.size,
        }
    }
}

impl<S: Default, C> core::ops::Deref for Style<S, C> {
    type Target = S;

    fn deref(&self) -> &Self::Target {
        &self.specific
    }
}

impl<S: Default, C> core::ops::DerefMut for Style<S, C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.specific
    }
}
