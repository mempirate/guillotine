//! Color theming.
use embedded_graphics::{
    pixelcolor::{
        Bgr555, Bgr565, Bgr666, Bgr888, BinaryColor, Gray2, Gray4, Gray8, GrayColor as _, Rgb555,
        Rgb565, Rgb666, Rgb888, RgbColor as _,
    },
    prelude::PixelColor,
};

/// Colors used by the UI when an element doesn't specify a color explicitly.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Theme<C>
where
    C: PixelColor,
{
    /// Color used to clear the display before drawing a frame.
    pub background: C,
    /// Default text color.
    pub foreground: C,
}

impl<C> Theme<C>
where
    C: PixelColor,
{
    /// Creates a theme from its background and foreground colors.
    pub const fn new(background: C, foreground: C) -> Self {
        Self { background, foreground }
    }
}

macro_rules! impl_rgb_theme {
    ($($color:ty),+ $(,)?) => {
        $(
            impl Default for Theme<$color> {
                fn default() -> Self {
                    Self::new(<$color>::BLACK, <$color>::WHITE)
                }
            }
        )+
    };
}

impl_rgb_theme!(Rgb555, Bgr555, Rgb565, Bgr565, Rgb666, Bgr666, Rgb888, Bgr888);

macro_rules! impl_gray_theme {
    ($($color:ty),+ $(,)?) => {
        $(
            impl Default for Theme<$color> {
                fn default() -> Self {
                    Self::new(<$color>::BLACK, <$color>::WHITE)
                }
            }
        )+
    };
}

impl_gray_theme!(Gray2, Gray4, Gray8);

impl Default for Theme<BinaryColor> {
    fn default() -> Self {
        Self::new(BinaryColor::Off, BinaryColor::On)
    }
}
