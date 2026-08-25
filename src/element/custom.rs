//! Custom element builder.

use embedded_graphics::pixelcolor::PixelColor;

use crate::{
    BuildError, Context, ElementBuilder, Style, StyledElement,
    common::NodeIndex,
    layout::Layout,
    tree::{Node, NodeKind},
};

impl<'frame, C, CE> Context<'frame, C, CE>
where
    C: PixelColor,
{
    /// Creates a builder for application-defined leaf content.
    pub fn custom<E>(&self, element: E) -> CustomBuilder<'_, 'frame, C, CE>
    where
        E: Into<CE>,
    {
        CustomBuilder::new(self, element.into())
    }
}

/// A builder that adds application-defined leaf content to a frame.
pub struct CustomBuilder<'cx, 'frame, C, CE>
where
    C: PixelColor,
{
    cx: &'cx Context<'frame, C, CE>,
    style: Style<(), C>,
    element: CE,
}

impl<'cx, 'frame, C, CE> CustomBuilder<'cx, 'frame, C, CE>
where
    C: PixelColor,
{
    /// Creates a custom element builder for the given context and element.
    pub fn new(cx: &'cx Context<'frame, C, CE>, element: CE) -> Self {
        Self { cx, style: Style::default(), element }
    }
}

impl<C, CE> StyledElement for CustomBuilder<'_, '_, C, CE>
where
    C: PixelColor,
{
    type Color = C;
    type Specific = ();

    fn style(&self) -> &Style<Self::Specific, Self::Color> {
        &self.style
    }

    fn style_mut(&mut self) -> &mut Style<Self::Specific, Self::Color> {
        &mut self.style
    }
}

impl<C, CE> ElementBuilder for CustomBuilder<'_, '_, C, CE>
where
    C: PixelColor,
{
    fn try_build(self) -> Result<NodeIndex, BuildError> {
        self.cx.insert(Node {
            kind: NodeKind::Custom(self.style, self.element),
            layout: Layout::empty(),
            child: None,
            sibling: None,
        })
    }
}
