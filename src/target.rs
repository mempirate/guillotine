use embedded_graphics::{
    draw_target::DrawTarget,
    geometry::{OriginDimensions, Size},
    primitives::Rectangle,
};

#[cfg(feature = "simulator")]
use embedded_graphics::pixelcolor::PixelColor;
#[cfg(feature = "simulator")]
use embedded_graphics_simulator::SimulatorDisplay;

/// A display target that optionally supports buffered drawing.
pub trait DisplayTarget: DrawTarget + OriginDimensions {
    /// Begins a new framebuffer mode with the given bounds and background color. A corresponding
    /// call to [`flush`](Self::flush) is required to update the display.
    fn try_begin(&mut self, bounds: Rectangle, background: Self::Color) -> bool;

    /// Flushes the buffer to the display, filling any remaining space with the given background
    /// color. Resets the mode to direct drawing.
    fn flush(&mut self) -> Result<(), Self::Error>;
}

/// A display target that only supports direct drawing without buffering. For a buffered display,
/// use [`BufferedDisplay`](crate::buffered::BufferedDisplay) (behind the `framebuffer` feature).
pub struct DirectTarget<D>(D);

impl<D> core::ops::Deref for DirectTarget<D> {
    type Target = D;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<D> DirectTarget<D> {
    /// Creates a new [`DirectTarget`] with the given display target.
    pub const fn new(display: D) -> Self {
        Self(display)
    }
}

impl<D> OriginDimensions for DirectTarget<D>
where
    D: OriginDimensions,
{
    fn size(&self) -> Size {
        self.0.size()
    }
}

impl<D> DrawTarget for DirectTarget<D>
where
    D: DrawTarget + OriginDimensions,
{
    type Color = D::Color;

    type Error = D::Error;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = embedded_graphics::prelude::Pixel<Self::Color>>,
    {
        self.0.draw_iter(pixels)
    }

    fn fill_contiguous<I>(&mut self, area: &Rectangle, colors: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Self::Color>,
    {
        self.0.fill_contiguous(area, colors)
    }

    fn fill_solid(&mut self, area: &Rectangle, color: Self::Color) -> Result<(), Self::Error> {
        self.0.fill_solid(area, color)
    }
}

impl<D> DisplayTarget for DirectTarget<D>
where
    D: DrawTarget + OriginDimensions,
{
    fn try_begin(&mut self, _: Rectangle, _: Self::Color) -> bool {
        false
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[cfg(feature = "simulator")]
impl<C: PixelColor> DisplayTarget for SimulatorDisplay<C> {
    fn try_begin(&mut self, _: Rectangle, _: Self::Color) -> bool {
        false
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}
