use alloc::borrow::Cow;
use embedded_graphics::{
    Drawable as _,
    mono_font::{MonoFont, MonoTextStyle, MonoTextStyleBuilder, ascii::FONT_10X20},
    pixelcolor::{BinaryColor, Rgb565},
    prelude::{Dimensions as _, DrawTarget, PixelColor, Point, Size},
    text::{Baseline, Text as GraphicsText},
};

use crate::{
    Style, Theme,
    element::{Element, IntoElement, draw_box},
    layout::BoxLayout,
    style::StyledElement,
};

/// Text style.
#[derive(PartialEq, Eq)]
pub struct TextStyle<C = Rgb565> {
    font: Font,
    color: Option<C>,
}

impl<C> Default for TextStyle<C> {
    fn default() -> Self {
        Self { font: Font::mono(&FONT_10X20), color: None }
    }
}

/// An ephemeral text declaration.
#[derive(PartialEq, Eq)]
pub struct Text<'a, C = Rgb565>
where
    C: PixelColor,
{
    content: Cow<'a, str>,
    style: Style<TextStyle<C>, C>,
}

impl<'a, C> Text<'a, C>
where
    C: PixelColor,
{
    /// Creates a text declaration that borrows its content for this render.
    pub fn new(content: impl Into<Cow<'a, str>>) -> Self {
        Self { content: content.into(), style: Style::default() }
    }

    /// Sets the style of this text.
    pub const fn with_style(mut self, style: Style<TextStyle<C>, C>) -> Self {
        self.style = style;
        self
    }

    /// Returns the text content.
    pub fn content(&self) -> &str {
        self.content.as_ref()
    }

    /// Returns a reference to this text's style.
    pub const fn style(&self) -> &Style<TextStyle<C>, C> {
        &self.style
    }

    /// Measures the size of this text.
    pub fn measure(&self) -> Size {
        match self.style.font {
            Font::Mono(font) => {
                // Text color doesn't affect geometry, so measurement doesn't need a theme.
                let character_style = MonoTextStyle::new(font, BinaryColor::On);

                let bounds = GraphicsText::with_baseline(
                    self.content.as_ref(),
                    Point::new(0, 0),
                    character_style,
                    Baseline::Top,
                )
                .bounding_box();

                bounds.size
            }
        }
    }

    /// Draws this text box onto the given target, using the provided layout.
    pub(crate) fn draw<D>(
        &self,
        layout: &BoxLayout,
        target: &mut D,
        theme: &Theme<C>,
    ) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = C>,
    {
        draw_box(&self.style, layout, target)?;

        match self.style.font {
            Font::Mono(font) => {
                let character_style = MonoTextStyleBuilder::new()
                    .font(font)
                    .text_color(self.style.color.unwrap_or(theme.foreground))
                    .build();

                GraphicsText::with_baseline(
                    self.content.as_ref(),
                    layout.content.top_left,
                    character_style,
                    Baseline::Top,
                )
                .draw(target)?;
            }
        }
        Ok(())
    }
}

impl<'a, C> IntoElement for Text<'a, C>
where
    C: PixelColor,
{
    type Element = Element<'a, C>;

    fn into_element(self) -> Element<'a, C> {
        Element::Text(self)
    }
}

impl<C> StyledElement for Text<'_, C>
where
    C: PixelColor,
{
    type Color = C;
    type Specific = TextStyle<C>;

    fn style_mut(&mut self) -> &mut Style<Self::Specific, Self::Color> {
        &mut self.style
    }
}

/// Creates a text declaration borrowing `content` for this render.
pub fn text<'a, C>(content: impl Into<Cow<'a, str>>) -> Text<'a, C>
where
    C: PixelColor,
{
    Text::new(content)
}

/// Font for text rendering.
#[derive(PartialEq)]
pub enum Font {
    /// A monospaced font.
    Mono(&'static MonoFont<'static>),
}

impl Font {
    /// Creates a [`Font::Mono`] font from the given [`MonoFont`].
    #[inline]
    pub const fn mono(font: &'static MonoFont<'static>) -> Self {
        Self::Mono(font)
    }
}

impl Eq for Font {}

impl Default for Font {
    fn default() -> Self {
        Self::mono(&FONT_10X20)
    }
}

/// A trait for elements that can be styled with text properties.
pub trait TextStyledElement<C>: StyledElement<Color = C, Specific = TextStyle<C>>
where
    C: PixelColor,
{
    /// Sets the font.
    fn font(mut self, font: Font) -> Self {
        self.style_mut().specific.font = font;
        self
    }

    /// Sets the text color.
    fn text_color(mut self, color: C) -> Self {
        self.style_mut().specific.color = Some(color);
        self
    }
}

impl<T, C> TextStyledElement<C> for T
where
    T: StyledElement<Color = C, Specific = TextStyle<C>>,
    C: PixelColor,
{
}
