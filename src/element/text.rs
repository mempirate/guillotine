use embedded_graphics::{
    Drawable as _,
    mono_font::{MonoFont, MonoTextStyle, MonoTextStyleBuilder, ascii::FONT_10X20},
    pixelcolor::Rgb565,
    prelude::{Dimensions as _, DrawTarget, Point, RgbColor as _, Size},
    text::{Baseline, Text as GraphicsText},
};

use crate::{
    Style,
    element::{Element, IntoElement, draw_box},
    layout::BoxLayout,
    style::StyledElement,
};

/// Text style.
#[derive(PartialEq, Eq)]
pub struct TextStyle {
    font: Font,
    color: Rgb565,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self { font: Font::mono(&FONT_10X20), color: Rgb565::WHITE }
    }
}

/// An ephemeral text declaration.
#[derive(PartialEq, Eq)]
pub struct Text<'a> {
    content: &'a str,
    style: Style<TextStyle>,
}

impl<'a> Text<'a> {
    /// Creates a text declaration that borrows its content for this render.
    pub fn new(content: &'a str) -> Self {
        Self { content, style: Style::default() }
    }

    /// Sets the style of this text.
    pub const fn with_style(mut self, style: Style<TextStyle>) -> Self {
        self.style = style;
        self
    }

    /// Returns the text content.
    pub const fn content(&self) -> &'a str {
        self.content
    }

    /// Returns a reference to this text's style.
    pub fn style(&self) -> &Style<TextStyle> {
        &self.style
    }

    /// Measures the size of this text.
    pub fn measure(&self) -> Size {
        // Temporary default until TextStyle owns a font and text color.
        //
        match self.style.font {
            Font::Mono(font) => {
                let character_style = MonoTextStyle::new(font, self.style.color);

                let bounds = GraphicsText::with_baseline(
                    self.content,
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
    pub(crate) fn draw<D>(&self, layout: &BoxLayout, target: &mut D) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        draw_box(&self.style, layout, target)?;

        match self.style.font {
            Font::Mono(font) => {
                let character_style =
                    MonoTextStyleBuilder::new().font(font).text_color(self.style.color).build();

                GraphicsText::with_baseline(
                    self.content,
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

impl<'a> IntoElement for Text<'a> {
    type Element = Element<'a>;

    fn into_element(self) -> Element<'a> {
        Element::Text(self)
    }
}

impl<'a> StyledElement for Text<'a> {
    type Specific = TextStyle;

    fn style_mut(&mut self) -> &mut Style<Self::Specific> {
        &mut self.style
    }
}

/// Creates a text declaration borrowing `content` for this render.
pub fn text(content: &str) -> Text<'_> {
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
    pub fn mono(font: &'static MonoFont<'static>) -> Self {
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
pub trait TextStyledElement: StyledElement<Specific = TextStyle> {
    /// Sets the font.
    fn font(mut self, font: Font) -> Self {
        self.style_mut().specific.font = font;
        self
    }

    /// Sets the text color.
    fn text_color(mut self, color: Rgb565) -> Self {
        self.style_mut().specific.color = color;
        self
    }
}

impl<T> TextStyledElement for T where T: StyledElement<Specific = TextStyle> {}
