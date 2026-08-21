use embedded_graphics::{geometry::Size, prelude::PixelColor};

use crate::{
    Context, Style,
    common::{Gap, NodeIndex},
    element::{BuildError, ElementBuilder, ParentElement},
    layout::Layout,
    style::{FlexDirection, StyledElement},
    tree::{Node, NodeKind},
};

#[cfg(feature = "flexbox")]
use crate::style::{AlignItems, JustifyContent};

/// Style for a div element.
#[derive(Default, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct DivStyle {
    /// Gap between child elements.
    pub(crate) gap: Size,
    /// Flex direction.
    pub(crate) direction: FlexDirection,
    #[cfg(feature = "flexbox")]
    /// Justification of the flex items along the main axis.
    pub(crate) justify_content: JustifyContent,
    #[cfg(feature = "flexbox")]
    /// Alignment of the flex items along the cross axis.
    pub(crate) align_items: AlignItems,
}

pub struct DivBuilder<'cx, 'frame, C: PixelColor> {
    style: Style<DivStyle, C>,
    cx: &'cx Context<'frame, C>,
    first_child: Option<NodeIndex>,
    last_child: Option<NodeIndex>,

    /// Optional error that occurred during build.
    error: Option<BuildError>,
}

impl<'cx, 'frame, C: PixelColor> DivBuilder<'cx, 'frame, C> {
    pub fn new(cx: &'cx Context<'frame, C>) -> Self {
        Self { style: Style::default(), cx, first_child: None, last_child: None, error: None }
    }
}

impl<'frame, C: PixelColor> Context<'frame, C> {
    /// Creates a new div container builder, with a default row flex direction.
    pub fn div(&self) -> DivBuilder<'_, 'frame, C> {
        DivBuilder::new(self)
    }

    /// Creates an empty row container builder.
    pub fn row(&self) -> DivBuilder<'_, 'frame, C> {
        DivBuilder::new(self).flex_direction(FlexDirection::Row)
    }

    /// Creates an empty column container builder.
    pub fn column(&self) -> DivBuilder<'_, 'frame, C> {
        DivBuilder::new(self).flex_direction(FlexDirection::Column)
    }
}

impl<C> StyledElement for DivBuilder<'_, '_, C>
where
    C: PixelColor,
{
    type Color = C;
    type Specific = DivStyle;

    fn style(&self) -> &Style<Self::Specific, Self::Color> {
        &self.style
    }

    fn style_mut(&mut self) -> &mut Style<Self::Specific, Self::Color> {
        &mut self.style
    }
}

impl<C: PixelColor> ParentElement for DivBuilder<'_, '_, C> {
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

impl<C: PixelColor> ElementBuilder for DivBuilder<'_, '_, C> {
    fn try_build(self) -> Result<NodeIndex, BuildError> {
        if let Some(err) = self.error {
            return Err(err);
        }

        let node = Node {
            kind: NodeKind::Div(self.style),
            layout: Layout::empty(),
            child: self.first_child,
            sibling: None,
        };

        self.cx.insert(node)
    }
}

/// A trait for elements that can be styled as a flex container.
pub trait StyledFlexContainer<C>: StyledElement<Color = C, Specific = DivStyle>
where
    C: PixelColor,
{
    /// Sets the gap between child elements.
    fn gap(mut self, gap: impl Into<Gap>) -> Self {
        self.style_mut().specific.gap = gap.into().0;
        self
    }

    /// Sets the flex direction of the container.
    fn flex_direction(mut self, direction: FlexDirection) -> Self {
        self.style_mut().specific.direction = direction;
        self
    }

    #[cfg(feature = "flexbox")]
    /// Sets the justification of the flex items along the main axis.
    fn justify_content(mut self, justify_content: JustifyContent) -> Self {
        self.style_mut().specific.justify_content = justify_content;
        self
    }

    #[cfg(feature = "flexbox")]
    /// Sets the alignment of the flex items along the cross axis.
    fn align_items(mut self, align_items: AlignItems) -> Self {
        self.style_mut().specific.align_items = align_items;
        self
    }
}

impl<T, C> StyledFlexContainer<C> for T
where
    T: StyledElement<Color = C, Specific = DivStyle>,
    C: PixelColor,
{
}
