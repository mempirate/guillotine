use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::FONT_6X10},
    pixelcolor::Rgb565,
    prelude::{Dimensions as _, Point, RgbColor as _, Size},
    text::{Baseline, Text as GraphicsText},
};

use crate::{
    Style,
    element::{Element, IntoElement},
};

/// Text style.
#[derive(Default, PartialEq, Eq)]
pub struct TextStyle {}

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
        let character_style = MonoTextStyle::new(&FONT_6X10, Rgb565::WHITE);

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

impl<'a> IntoElement for Text<'a> {
    type Element = Element<'a>;

    fn into_element(self) -> Element<'a> {
        Element::Text(self)
    }
}

/// Creates a text declaration borrowing `content` for this render.
pub fn text(content: &str) -> Text<'_> {
    Text::new(content)
}
