use embedded_graphics::{
    pixelcolor::PixelColor,
    prelude::{Point, Size},
    primitives::Rectangle,
};

use crate::{
    common::{NodeIndex, SizeExt as _, to_i32},
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

/// The axis along which a container lays out its children.
#[derive(Clone, Copy)]
enum Axis {
    Horizontal,
    Vertical,
}

impl<'frame, C> FrameTree<'frame, C>
where
    C: PixelColor,
{
    // TODO: Clean up
    pub(crate) fn layout_node(&mut self, index: NodeIndex, constraints: Constraints) {
        // Extract the common box style properties.
        let box_style = self.node(index).box_style();

        let border_constraints = box_style.border_constraints(constraints);

        let content_constraints = box_style.content_constraints(constraints);

        // Determine the content's intrinsic size without retaining a
        // borrow of `self.nodes[index]` while layout_children mutates children.
        enum ContentLayout {
            Container(Axis),
            // A leaf with an inherent size.
            Leaf(Size),
        }

        let content_layout = match &self.node(index).kind {
            NodeKind::Column(_) => ContentLayout::Container(Axis::Vertical),
            NodeKind::Row(_) => ContentLayout::Container(Axis::Horizontal),
            NodeKind::Text(text) => ContentLayout::Leaf(content_constraints.constrain(text.size)),
        };

        let intrinsic_content_size = match content_layout {
            ContentLayout::Container(axis) => {
                self.layout_children(index, content_constraints, axis)
            }

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
    fn layout_children(&mut self, index: NodeIndex, constraints: Constraints, axis: Axis) -> Size {
        let mut main_size: u32 = 0;
        let mut cross_size: u32 = 0;

        // Get the optional child.
        let mut child = self.node(index).child;

        while let Some(child_idx) = child {
            self.layout_node(child_idx, constraints);

            let size = self.node(child_idx).layout.outer_size;

            let main_offset = i32::try_from(main_size).unwrap_or(i32::MAX);

            // TODO: Only allows for start alignment, and no gap.
            let offset = match axis {
                Axis::Horizontal => Point::new(main_offset, 0),
                Axis::Vertical => Point::new(0, main_offset),
            };

            self.node_mut(child_idx).layout.set_offset(offset);

            let (child_main_size, child_cross_size) = match axis {
                Axis::Horizontal => (size.width, size.height),
                Axis::Vertical => (size.height, size.width),
            };

            main_size = main_size.saturating_add(child_main_size);
            cross_size = cross_size.max(child_cross_size);

            child = self.node(child_idx).sibling;
        }

        let intrinsic_size = match axis {
            Axis::Horizontal => Size::new(main_size, cross_size),
            Axis::Vertical => Size::new(cross_size, main_size),
        };

        constraints.constrain(intrinsic_size)
    }
}
