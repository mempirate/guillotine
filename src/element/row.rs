use embedded_graphics::prelude::PixelColor;

use crate::{
    Context, Style,
    common::NodeIndex,
    element::{BuildError, ElementBuilder, ParentElement},
    layout::Layout,
    style::StyledElement,
    tree::{Node, NodeKind},
};

/// Style for this row.
#[derive(Default, PartialEq, Eq)]
pub struct RowStyle {}

impl<'frame, C: PixelColor> Context<'frame, C> {
    /// Creates an empty horizontal container builder.
    pub fn row(&self) -> RowBuilder<'_, 'frame, C> {
        RowBuilder::new(self)
    }
}

pub struct RowBuilder<'cx, 'frame, C: PixelColor> {
    pub(crate) style: Style<RowStyle, C>,
    cx: &'cx Context<'frame, C>,
    first_child: Option<NodeIndex>,
    last_child: Option<NodeIndex>,

    /// Optional error that occurred during build.
    error: Option<BuildError>,
}

impl<'cx, 'frame, C: PixelColor> RowBuilder<'cx, 'frame, C> {
    pub fn new(cx: &'cx Context<'frame, C>) -> Self {
        Self { style: Style::default(), cx, first_child: None, last_child: None, error: None }
    }
}

impl<C> StyledElement for RowBuilder<'_, '_, C>
where
    C: PixelColor,
{
    type Color = C;
    type Specific = RowStyle;

    fn style(&self) -> &Style<Self::Specific, Self::Color> {
        &self.style
    }

    fn style_mut(&mut self) -> &mut Style<Self::Specific, Self::Color> {
        &mut self.style
    }
}

impl<C: PixelColor> ParentElement for RowBuilder<'_, '_, C> {
    fn extend<E: ElementBuilder>(&mut self, elements: impl IntoIterator<Item = E>) {
        // Short-circuit if an error occurred during a previous extend call.
        if self.error.is_some() {
            return;
        }

        for element in elements {
            match element.try_build() {
                Ok(idx) => {
                    // Link the child: if no child is set as the first child, we set it.
                    self.first_child.get_or_insert(idx);

                    if let Some(last) = self.last_child {
                        // Link the child to the last child, if any.
                        self.cx.link_sibling(last, idx);
                    }

                    // Record last child.
                    self.last_child = Some(idx);
                }
                Err(err) => {
                    // Record the error and break early.
                    self.error = Some(err);
                    break;
                }
            }
        }
    }
}

impl<C: PixelColor> ElementBuilder for RowBuilder<'_, '_, C> {
    fn try_build(self) -> Result<NodeIndex, BuildError> {
        if let Some(err) = self.error {
            return Err(err);
        }

        let node = Node {
            kind: NodeKind::Row(self.style),
            layout: Layout::empty(),
            child: self.first_child,
            sibling: None,
        };

        self.cx.insert(node)
    }
}
