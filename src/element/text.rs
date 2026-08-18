use embedded_graphics::{
    mono_font::{MonoFont, MonoTextStyle, ascii::FONT_10X20},
    pixelcolor::{BinaryColor, Rgb565},
    prelude::{Dimensions as _, PixelColor, Point, Size},
    text::{Baseline, Text as GraphicsText},
};

use crate::{
    Context, Node, NodeIndex, NodeKind, Style, TextNode,
    element::{BuildError, ElementBuilder},
    layout::Layout,
    style::StyledElement,
};

/// Text style.
#[derive(PartialEq, Eq)]
pub struct TextStyle<C = Rgb565> {
    pub(crate) font: Font,
    pub(crate) color: Option<C>,
}

impl<C> Default for TextStyle<C> {
    fn default() -> Self {
        Self { font: Font::mono(&FONT_10X20), color: None }
    }
}

impl<'frame, C> Context<'frame, C>
where
    C: PixelColor,
{
    /// Creates a new text element with the given content.
    pub fn text<'cx, 't>(&'cx self, content: &'t str) -> TextBuilder<'cx, 'frame, 't, C> {
        TextBuilder::new(self, content)
    }
}

pub struct TextBuilder<'cx, 'frame, 't, C: PixelColor> {
    content: &'t str,
    pub(crate) style: Style<TextStyle<C>, C>,
    cx: &'cx Context<'frame, C>,
}

impl<'cx, 'frame, 't, C: PixelColor> TextBuilder<'cx, 'frame, 't, C> {
    pub fn new(cx: &'cx Context<'frame, C>, content: &'t str) -> Self {
        Self { content, style: Style::default(), cx }
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
}

impl<C> StyledElement for TextBuilder<'_, '_, '_, C>
where
    C: PixelColor,
{
    type Color = C;
    type Specific = TextStyle<C>;

    fn style(&self) -> &Style<Self::Specific, Self::Color> {
        &self.style
    }

    fn style_mut(&mut self) -> &mut Style<Self::Specific, Self::Color> {
        &mut self.style
    }
}

impl<C> ElementBuilder for TextBuilder<'_, '_, '_, C>
where
    C: PixelColor,
{
    fn try_build(self) -> Result<NodeIndex, BuildError> {
        let range = self.cx.store_text(self.content)?;

        // Measure the size of the text.
        let size = self.measure();

        self.cx.insert(Node {
            kind: NodeKind::Text(TextNode { size, range, style: self.style }),
            layout: Layout::empty(),
            child: None,
            sibling: None,
        })
    }
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
