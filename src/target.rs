use embedded_graphics::{
    draw_target::DrawTarget,
    geometry::{OriginDimensions, Size},
    pixelcolor::PixelColor,
    primitives::Rectangle,
};
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
/// use [`BufferedTarget`](buffered::BufferedTarget) (behind the `framebuffer` feature).
pub struct DirectTarget<D: DisplayTarget<Color = C>, C: PixelColor>(D);

impl<D, C> core::ops::Deref for DirectTarget<D, C>
where
    D: DisplayTarget<Color = C>,
    C: PixelColor,
{
    type Target = D;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<D, C> DirectTarget<D, C>
where
    D: DisplayTarget<Color = C>,
    C: PixelColor,
{
    /// Creates a new [`DirectTarget`] with the given display target.
    pub fn new(display: D) -> Self {
        Self(display)
    }
}

impl<D, C> OriginDimensions for DirectTarget<D, C>
where
    D: DisplayTarget<Color = C>,
    C: PixelColor,
{
    fn size(&self) -> Size {
        self.0.size()
    }
}

impl<D, C> DrawTarget for DirectTarget<D, C>
where
    D: DisplayTarget<Color = C>,
    C: PixelColor,
{
    type Color = C;

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

impl<D, C> DisplayTarget for DirectTarget<D, C>
where
    D: DisplayTarget<Color = C>,
    C: PixelColor,
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
