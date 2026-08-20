use embedded_graphics::{
    pixelcolor::PixelColor,
    prelude::{Point, Size},
    primitives::Rectangle,
};

use crate::{
    common::{NodeIndex, SizeExt as _, to_i32},
    style::{FlexDirection, FlexLayout},
    tree::{FrameTree, NodeKind},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Constraints {
    min: Size,
    max: Size,
}

impl Constraints {
    /// Returns [`Constraints`] that is exact in both dimensions.
    pub(crate) const fn exact(size: Size) -> Self {
        Self { min: size, max: size }
    }

    /// Returns [`Constraints`] with zero minimum size and the given maximum size.
    pub(crate) const fn max(size: Size) -> Self {
        Self { min: Size::zero(), max: size }
    }

    pub(crate) fn constrain(self, desired: Size) -> Size {
        Size::new(
            desired.width.clamp(self.min.width, self.max.width),
            desired.height.clamp(self.min.height, self.max.height),
        )
    }

    /// Returns [`Constraints`] with zero minimum size and the same maximum size as the original.
    pub(crate) const fn loosen(self) -> Self {
        Self { min: Size::zero(), max: self.max }
    }

    pub(crate) const fn deflate(self, by: Size) -> Self {
        Self { min: self.min.saturating_sub(by), max: self.max.saturating_sub(by) }
    }
}

#[derive(PartialEq, Eq)]
pub(crate) struct Layout {
    /// Position relative to the parent.
    pub(crate) offset: Point,
    /// Position of the border relative to the parent.
    pub(crate) border_offset: Point,
    /// Position of the content relative to the parent.
    pub(crate) content_offset: Point,
    /// Size of the node (after constraints are applied).
    pub(crate) outer_size: Size,
    /// Size of the border.
    pub(crate) border_size: Size,
    /// Size of the actual content.
    pub(crate) content_size: Size,
}

impl Layout {
    /// Creates an empty unresolved layout.
    pub(crate) const fn empty() -> Self {
        Self {
            offset: Point::new(0, 0),
            border_offset: Point::new(0, 0),
            content_offset: Point::new(0, 0),

            outer_size: Size::zero(),
            border_size: Size::zero(),
            content_size: Size::zero(),
        }
    }

    pub(crate) const fn set_offset(&mut self, offset: Point) {
        self.offset = offset;
    }

    /// Resolves the absolute bounds of the complete box layout.
    pub(crate) fn resolve(&self, parent_origin: Point) -> BoxLayout {
        let outer_origin = parent_origin + self.offset;
        let border_origin = outer_origin + self.border_offset;
        let content_origin = outer_origin + self.content_offset;

        BoxLayout {
            border: Rectangle::new(border_origin, self.border_size),
            content: Rectangle::new(content_origin, self.content_size),
        }
    }
}

/// Fully resolved box layout. Contains absolute positioned rectangles for margin, border and
/// content.
pub(crate) struct BoxLayout {
    pub(crate) border: Rectangle,
    pub(crate) content: Rectangle,
}

impl FlexDirection {
    /// Splits a physical size into its main-axis and cross-axis components.
    ///
    /// The main axis is the direction in which children are laid out; the cross axis is
    /// perpendicular to it. A row therefore uses width as its main size and height as its cross
    /// size, while a column uses height as its main size and width as its cross size.
    const fn split(self, size: Size) -> (u32, u32) {
        match self {
            Self::Row => (size.width, size.height),
            Self::Column => (size.height, size.width),
        }
    }

    /// Creates a physical offset from a logical main-axis offset.
    const fn offset(self, main: u32) -> Point {
        let main = to_i32(main);

        match self {
            Self::Row => Point::new(main, 0),
            Self::Column => Point::new(0, main),
        }
    }

    /// Creates a physical size from logical main-axis and cross-axis sizes.
    const fn size(self, main: u32, cross: u32) -> Size {
        match self {
            Self::Row => Size::new(main, cross),
            Self::Column => Size::new(cross, main),
        }
    }
}

impl<'frame, C> FrameTree<'frame, C>
where
    C: PixelColor,
{
    /// Lays out the full tree, starting with the root node.
    pub(crate) fn layout(&mut self, root: NodeIndex, constraints: Constraints) {
        self.root = Some(root);
        self.layout_node(root, constraints);
    }

    // TODO: Clean up
    fn layout_node(&mut self, index: NodeIndex, constraints: Constraints) {
        // Extract the common box style properties.
        let box_style = self.node(index).box_style();

        let border_constraints = box_style.border_constraints(constraints);

        let content_constraints = box_style.content_constraints(constraints);

        // Determine the content's intrinsic size without retaining a
        // borrow of `self.nodes[index]` while layout_children mutates children.
        enum ContentLayout {
            /// Flexbox layout.
            Flex(FlexLayout),
            // A leaf with an inherent size.
            Leaf(Size),
        }

        let content_layout = match &self.node(index).kind {
            NodeKind::Div(style) => ContentLayout::Flex(style.specific.clone().into()),
            NodeKind::Text(text) => ContentLayout::Leaf(content_constraints.constrain(text.size)),
        };

        let intrinsic_content_size = match content_layout {
            ContentLayout::Flex(layout) => self.layout_children(index, content_constraints, layout),
            ContentLayout::Leaf(size) => size,
        };

        let content_insets = box_style.content_insets();
        let content_inset_size = content_insets.total_size();
        let border_size = {
            // The "natural" border size, i.e. derived from its children.
            let natural_border_size = intrinsic_content_size.inflate(content_inset_size);

            // A configured size describes the border box, but it grows to fit border and padding
            // when the parent constraints allow it.
            let desired_border_size = box_style.size.map_or(natural_border_size, |size| {
                Size::new(
                    size.width.max(content_inset_size.width),
                    size.height.max(content_inset_size.height),
                )
            });

            border_constraints.constrain(desired_border_size)
        };

        let outer_size = border_size.inflate(box_style.margin.total_size());

        let content_size = border_size.deflate(content_inset_size);

        let content_offset = box_style.margin.saturating_add(content_insets);

        self.node_mut(index).layout = Layout {
            // The parent assigns this later. The root remains at zero.
            offset: Point::new(0, 0),

            // Both offsets are relative to the node's outer-box origin.
            border_offset: Point::new(to_i32(box_style.margin.left), to_i32(box_style.margin.top)),

            content_offset: Point::new(to_i32(content_offset.left), to_i32(content_offset.top)),

            outer_size,
            border_size,
            content_size,
        };
    }

    /// Lays out the container at `index` along `axis`:
    /// - Traverses the children and sets their offsets.
    /// - Sums the children's sizes on the main axis and takes their maximum size on the cross axis.
    /// - Returns the constrained intrinsic size.
    fn layout_children(
        &mut self,
        index: NodeIndex,
        constraints: Constraints,
        layout: FlexLayout,
    ) -> Size {
        let direction = layout.direction;
        let (gap, _) = direction.split(layout.gap);

        // Main is the cursor (x or y depending on direction).
        let mut main: u32 = 0;
        // Cross is the maximum size on the perpendicular axis / direction.
        let mut cross: u32 = 0;

        // Get the optional child.
        let mut next = self.node(index).child;

        while let Some(next_idx) = next {
            self.layout_node(next_idx, constraints);

            let size = self.node(next_idx).layout.outer_size;
            // Get the child's main and cross sizes based on direction.
            let (child_main, child_cross) = direction.split(size);

            // Calculate the next offset based on the main cursor.
            let offset = direction.offset(main);
            self.node_mut(next_idx).layout.set_offset(offset);

            main = main.saturating_add(child_main);
            cross = cross.max(child_cross);

            next = self.node(next_idx).sibling;

            // Only add the gap if there's a next sibling.
            if next.is_some() {
                main = main.saturating_add(gap);
            }
        }

        constraints.constrain(direction.size(main, cross))
    }
}
