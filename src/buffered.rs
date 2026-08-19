//! Buffered display operations.
use embedded_graphics::{
    Pixel,
    draw_target::DrawTarget,
    geometry::{OriginDimensions, Size},
    pixelcolor::PixelColor,
    primitives::{PointsIter as _, Rectangle},
};

use crate::DisplayTarget;

/// A target wrapper that uses an internal buffer for drawing.
pub struct BufferedTarget<'a, D, C>
where
    D: DrawTarget<Color = C>,
    C: PixelColor,
{
    display: &'a mut D,
    pixels: &'a mut [C],
    mode: Mode,
}

enum Mode {
    /// Direct mode (no buffer, pure pass-through)
    Direct,
    /// Buffered mode (uses an internal buffer for drawing)
    Buffered {
        /// The bounds of the buffer (position and size)
        bounds: Rectangle,
        /// The number of pixels in the buffer
        len: usize,
    },
}

impl<'a, D, C> BufferedTarget<'a, D, C>
where
    D: DrawTarget<Color = C>,
    C: PixelColor,
{
    /// Initializes a new [`BufferedDisplay`] with the given display and buffer.
    pub const fn new(display: &'a mut D, buffer: &'a mut [C]) -> Self {
        Self { display, pixels: buffer, mode: Mode::Direct }
    }

    /// Returns whether the given bounds can be buffered using the current buffer capacity.
    #[allow(unused)]
    fn can_buffer(&self, bounds: Rectangle) -> bool {
        pixel_count(bounds).is_some_and(|len| len <= self.pixels.len())
    }
}

impl<'a, D, C> DisplayTarget for BufferedTarget<'a, D, C>
where
    D: DrawTarget<Color = C> + OriginDimensions,
    C: PixelColor,
{
    fn try_begin(&mut self, bounds: Rectangle, background: Self::Color) -> bool {
        let Some(len) = pixel_count(bounds) else {
            return false;
        };

        assert!(
            matches!(self.mode, Mode::Direct),
            "transaction already in progress, call flush first"
        );

        // Check that the buffer is large enough
        if len > self.pixels.len() {
            return false;
        }

        self.pixels[..len].fill(background);
        self.mode = Mode::Buffered { bounds, len };

        true
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        match core::mem::replace(&mut self.mode, Mode::Direct) {
            Mode::Direct => Ok(()),
            Mode::Buffered { bounds, len } => {
                self.display.fill_contiguous(&bounds, self.pixels[..len].iter().copied())
            }
        }
    }
}

impl<'a, D, C> OriginDimensions for BufferedTarget<'a, D, C>
where
    D: DrawTarget<Color = C> + OriginDimensions,
    C: PixelColor,
{
    fn size(&self) -> Size {
        // TODO: Is this correct? Or should we use the buffer size instead?
        self.display.size()
    }
}

impl<'a, D, C> DrawTarget for BufferedTarget<'a, D, C>
where
    D: DrawTarget<Color = C> + OriginDimensions,
    C: PixelColor,
{
    type Color = C;
    type Error = D::Error;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        match &mut self.mode {
            Mode::Direct => self.display.draw_iter(pixels),
            Mode::Buffered { bounds, len } => {
                let width = bounds.size.width;
                let height = bounds.size.height;

                debug_assert_eq!(*len, (width * height) as usize);
                debug_assert!(*len <= self.pixels.len());

                let framebuffer = &mut self.pixels[..*len];

                // For each pixel, we translate the point to the framebuffer index (relative to the
                // bounds)
                for Pixel(point, color) in pixels {
                    // Translate to local coordinates
                    let Some(x) = point.x.checked_sub(bounds.top_left.x) else { continue };
                    let Some(y) = point.y.checked_sub(bounds.top_left.y) else { continue };

                    // Clip negative coordinates (these are outside the bounds)
                    let Ok(x) = u32::try_from(x) else { continue };
                    let Ok(y) = u32::try_from(y) else { continue };

                    if x >= width || y >= height {
                        continue;
                    }

                    let index = (y * width + x) as usize;
                    framebuffer[index] = color;
                }

                Ok(())
            }
        }
    }

    fn fill_contiguous<I>(&mut self, area: &Rectangle, colors: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Self::Color>,
    {
        match self.mode {
            Mode::Direct => self.display.fill_contiguous(area, colors),
            Mode::Buffered { .. } => {
                // TODO: Optimized implementation.
                self.draw_iter(area.points().zip(colors).map(|(point, color)| Pixel(point, color)))
            }
        }
    }

    fn fill_solid(&mut self, area: &Rectangle, color: C) -> Result<(), Self::Error> {
        match &mut self.mode {
            Mode::Direct => self.display.fill_solid(area, color),
            Mode::Buffered { bounds, len } => {
                let area = area.intersection(bounds);
                if area.size == Size::zero() {
                    return Ok(());
                }

                let buffer_width = bounds.size.width as usize;
                let area_width = area.size.width as usize;
                let x = (area.top_left.x - bounds.top_left.x) as usize;
                let y = (area.top_left.y - bounds.top_left.y) as usize;
                let framebuffer = &mut self.pixels[..*len];

                for row in y..y + area.size.height as usize {
                    let start = row * buffer_width + x;
                    framebuffer[start..start + area_width].fill(color);
                }

                Ok(())
            }
        }
    }
}

const fn pixel_count(bounds: Rectangle) -> Option<usize> {
    (bounds.size.width as usize).checked_mul(bounds.size.height as usize)
}

#[cfg(test)]
mod tests {
    use embedded_graphics::{geometry::Point, mock_display::MockDisplay, pixelcolor::BinaryColor};

    use super::*;

    #[test]
    fn direct_mode_draws_to_display() {
        let mut display = MockDisplay::new();
        let mut buffer = [BinaryColor::Off; 4];

        {
            let mut target = BufferedTarget::new(&mut display, &mut buffer);
            target
                .fill_solid(&Rectangle::new(Point::new(1, 1), Size::new(2, 1)), BinaryColor::On)
                .unwrap();
        }

        assert_eq!(display.get_pixel(Point::new(1, 1)), Some(BinaryColor::On));
        assert_eq!(display.get_pixel(Point::new(2, 1)), Some(BinaryColor::On));
        assert_eq!(buffer, [BinaryColor::Off; 4]);
    }

    #[test]
    fn buffered_mode_composes_and_flushes_a_region() {
        let mut display = MockDisplay::new();
        let mut buffer = [BinaryColor::Off; 4];
        let bounds = Rectangle::new(Point::new(2, 3), Size::new(2, 2));

        {
            let mut target = BufferedTarget::new(&mut display, &mut buffer);
            assert!(target.can_buffer(bounds));
            assert!(!target.can_buffer(Rectangle::new(Point::zero(), Size::new(3, 2))));

            target.try_begin(bounds, BinaryColor::Off);
            target
                .fill_solid(&Rectangle::new(Point::new(1, 3), Size::new(2, 1)), BinaryColor::On)
                .unwrap();
            target
                .fill_contiguous(
                    &Rectangle::new(Point::new(2, 4), Size::new(2, 1)),
                    [BinaryColor::Off, BinaryColor::On],
                )
                .unwrap();
            target
                .draw_iter([
                    Pixel(Point::new(2, 4), BinaryColor::On),
                    Pixel(Point::new(1, 3), BinaryColor::On),
                ])
                .unwrap();

            target.flush().unwrap();
            target.draw_iter([Pixel(Point::zero(), BinaryColor::On)]).unwrap();
        }

        assert_eq!(display.get_pixel(Point::zero()), Some(BinaryColor::On));
        assert_eq!(display.get_pixel(Point::new(2, 3)), Some(BinaryColor::On));
        assert_eq!(display.get_pixel(Point::new(3, 3)), Some(BinaryColor::Off));
        assert_eq!(display.get_pixel(Point::new(2, 4)), Some(BinaryColor::On));
        assert_eq!(display.get_pixel(Point::new(3, 4)), Some(BinaryColor::On));
    }
}
