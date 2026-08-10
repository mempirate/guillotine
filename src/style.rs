//! Styling utilities.
use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::{PixelColor, Size},
};

use crate::Constraints;

/// Physical top, right, bottom, and left insets in pixels.
///
/// Guillotine uses the same one-to-four-value shorthand ordering as CSS:
///
/// - `10`: all sides
/// - `(4, 8)`: vertical, horizontal
/// - `(4, 8, 12)`: top, horizontal, bottom
/// - `(4, 8, 12, 16)`: top, right, bottom, left
///
/// Insets are non-negative pixel lengths. Percentages, `auto`, logical edges, negative margins,
/// and margin collapsing are not supported.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Insets {
    /// Top inset.
    pub top: u32,
    /// Right inset.
    pub right: u32,
    /// Bottom inset.
    pub bottom: u32,
    /// Left inset.
    pub left: u32,
}

impl Insets {
    /// Insets with every edge set to zero.
    pub const ZERO: Self = Self::uniform(0);

    /// Creates insets in CSS order: top, right, bottom, left.
    pub const fn new(top: u32, right: u32, bottom: u32, left: u32) -> Self {
        Self { top, right, bottom, left }
    }

    /// Creates uniform insets from the given `value`.
    pub const fn uniform(value: u32) -> Self {
        Self::new(value, value, value, value)
    }

    /// Creates zero insets.
    pub const fn zero() -> Self {
        Self::ZERO
    }

    /// Returns the horizontal insets (`left + right`).
    pub const fn horizontal(self) -> u32 {
        self.left.saturating_add(self.right)
    }

    /// Returns the vertical insets (`top + bottom`).
    pub const fn vertical(self) -> u32 {
        self.top.saturating_add(self.bottom)
    }

    /// Returns the total horizontal and vertical inset as a size.
    pub const fn total_size(self) -> Size {
        Size::new(self.horizontal(), self.vertical())
    }

    /// Adds two sets of insets edge by edge using saturating arithmetic.
    pub const fn saturating_add(self, other: Self) -> Self {
        Self::new(
            self.top.saturating_add(other.top),
            self.right.saturating_add(other.right),
            self.bottom.saturating_add(other.bottom),
            self.left.saturating_add(other.left),
        )
    }
}

macro_rules! impl_insets_from {
    ($type:ty, $convert:expr) => {
        impl From<$type> for Insets {
            fn from(value: $type) -> Self {
                let convert = $convert;
                Self::uniform(convert(value))
            }
        }

        impl From<($type, $type)> for Insets {
            fn from((vertical, horizontal): ($type, $type)) -> Self {
                let convert = $convert;
                Self::new(
                    convert(vertical),
                    convert(horizontal),
                    convert(vertical),
                    convert(horizontal),
                )
            }
        }

        impl From<($type, $type, $type)> for Insets {
            fn from((top, horizontal, bottom): ($type, $type, $type)) -> Self {
                let convert = $convert;
                Self::new(convert(top), convert(horizontal), convert(bottom), convert(horizontal))
            }
        }

        impl From<($type, $type, $type, $type)> for Insets {
            fn from((top, right, bottom, left): ($type, $type, $type, $type)) -> Self {
                let convert = $convert;
                Self::new(convert(top), convert(right), convert(bottom), convert(left))
            }
        }
    };
}

impl_insets_from!(u32, |value: u32| value);
impl_insets_from!(usize, |value: usize| u32::try_from(value).unwrap_or(u32::MAX));
impl_insets_from!(i32, |value: i32| {
    assert!(value >= 0, "insets cannot be negative");
    value as u32
});

pub(crate) struct BoxStyle {
    pub margin: Insets,
    pub border: Insets,
    pub padding: Insets,
    pub size: Option<Size>,
}

impl BoxStyle {
    /// Returns the constraints for the border box by subtracting the margin from
    /// the given constraints.
    pub(crate) fn border_constraints(&self, constraints: Constraints) -> Constraints {
        constraints.deflate(self.margin.total_size())
    }

    /// Returns loose constraints for the contents of a box, which lives inside the border
    /// and padding.
    pub(crate) fn content_constraints(&self, constraints: Constraints) -> Constraints {
        let border_constraints = self.border_constraints(constraints);
        let content_size = self.content_insets().total_size();

        let border_constraints = match self.size {
            Some(size) => {
                // `size` describes the border box. Like CSS `border-box`, it grows to the minimum
                // needed by its padding and border unless hard parent constraints prevent that.
                let desired = Size::new(
                    size.width.max(content_size.width),
                    size.height.max(content_size.height),
                );
                let border_size = border_constraints.constrain(desired);

                Constraints::exact(border_size)
            }
            None => border_constraints,
        };

        border_constraints.deflate(content_size).loosen()
    }

    /// Returns the content insets, i.e. `border + padding` on each edge.
    pub(crate) const fn content_insets(&self) -> Insets {
        self.border.saturating_add(self.padding)
    }
}

/// Common style type shared between all [`crate::Element`] variants.
///
/// # Box model
///
/// ```text
/// ┌─────────────────────────┐
/// │         margin          │
/// │  ┌───────────────────┐  │
/// │  │      border       │  │
/// │  │  ┌─────────────┐  │  │
/// │  │  │   padding   │  │  │
/// │  │  │  ┌───────┐  │  │  │
/// │  │  │  │content│  │  │  │
/// │  │  │  └───────┘  │  │  │
/// │  │  └─────────────┘  │  │
/// │  └───────────────────┘  │
/// └─────────────────────────┘
/// ```
///
/// [`Self::margin`], [`Self::padding`], and [`Self::border`] use physical top/right/bottom/left
/// edges and accept CSS-like one-to-four-value shorthands through [`StyledElement`]. Adjacent
/// margins in rows and columns add together; they do not collapse.
///
/// [`Self::size`] sets the border-box size. Padding and border are placed inside that size, while
/// margin is added outside it. A configured border box grows to contain its padding and border
/// when parent constraints allow. Guillotine supports non-negative pixel insets only, with one
/// border color and no border styles.
#[derive(PartialEq, Eq)]
pub struct Style<S: Default, C = Rgb565> {
    /// Margin insets: transparent space outside the border box.
    pub margin: Insets,
    /// Padding insets: space between the border and content.
    pub padding: Insets,
    /// Border widths.
    pub border: Insets,
    /// Border color shared by all four edges.
    pub border_color: Option<C>,
    /// Background color painted across the complete border box, beneath the border.
    pub background: Option<C>,
    /// Size of the border box.
    pub size: Option<Size>,
    /// Specific style properties for each element kind.
    pub specific: S,
}

impl<S: Default, C> Default for Style<S, C> {
    fn default() -> Self {
        Self {
            margin: Insets::ZERO,
            padding: Insets::ZERO,
            border: Insets::ZERO,
            border_color: None,
            background: None,
            size: None,
            specific: S::default(),
        }
    }
}

impl<S: Default, C> Style<S, C> {
    /// Derive a [`BoxStyle`] for layout.
    pub(crate) fn box_style(&self) -> BoxStyle {
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

/// A blanket trait for elements that can be styled. Import [`StyledElement`] to use.
pub trait StyledElement: Sized {
    /// The pixel color used by this element.
    type Color: PixelColor;

    /// The specific style type for this element.
    type Specific: Default;

    /// Returns a mutable reference to the element's specific style.
    fn style_mut(&mut self) -> &mut Style<Self::Specific, Self::Color>;

    /// Sets the padding using a CSS-like one-to-four-value inset shorthand.
    fn padding(mut self, padding: impl Into<Insets>) -> Self {
        self.style_mut().padding = padding.into();
        self
    }

    /// Sets the margin using a CSS-like one-to-four-value inset shorthand.
    fn margin(mut self, margin: impl Into<Insets>) -> Self {
        self.style_mut().margin = margin.into();
        self
    }

    /// Sets the background color of the element.
    fn background(mut self, color: Self::Color) -> Self {
        self.style_mut().background = Some(color);
        self
    }

    /// Sets the border widths using a CSS-like one-to-four-value inset shorthand.
    fn border(mut self, border: impl Into<Insets>) -> Self {
        self.style_mut().border = border.into();
        self
    }

    /// Sets the size of the element's border box.
    fn size(mut self, size: Size) -> Self {
        self.style_mut().size = Some(size);
        self
    }

    /// Sets the border color of the element.
    fn border_color(mut self, color: Self::Color) -> Self {
        self.style_mut().border_color = Some(color);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn css_shorthands_expand_in_trbl_order() {
        assert_eq!(Insets::from(10), Insets::new(10, 10, 10, 10));
        assert_eq!(Insets::from((4, 8)), Insets::new(4, 8, 4, 8));
        assert_eq!(Insets::from((4, 8, 12)), Insets::new(4, 8, 12, 8));
        assert_eq!(Insets::from((4, 8, 12, 16)), Insets::new(4, 8, 12, 16));
    }

    #[test]
    fn typed_unsigned_inputs_are_supported() {
        assert_eq!(Insets::from(3_u32), Insets::uniform(3));
        assert_eq!(Insets::from((1_usize, 2, 3)), Insets::new(1, 2, 3, 2));
    }

    #[test]
    fn usize_conversion_saturates() {
        assert_eq!(Insets::from(usize::MAX), Insets::uniform(u32::MAX));
    }

    #[test]
    #[should_panic(expected = "insets cannot be negative")]
    fn signed_conversion_rejects_negative_values() {
        let _ = Insets::from((1, -2));
    }

    #[test]
    fn inset_arithmetic_saturates() {
        let insets = Insets::new(u32::MAX, u32::MAX, 3, 4);

        assert_eq!(insets.horizontal(), u32::MAX);
        assert_eq!(insets.vertical(), u32::MAX);
        assert_eq!(
            insets.saturating_add(Insets::new(1, 2, u32::MAX, u32::MAX)),
            Insets::new(u32::MAX, u32::MAX, u32::MAX, u32::MAX),
        );
    }
}
