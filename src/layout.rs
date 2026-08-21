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
    #[cfg(test)]
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

    /// Makes configured dimensions exact while preserving the constraints of automatic axes.
    pub(crate) fn with_exact_dimensions(self, width: Option<u32>, height: Option<u32>) -> Self {
        let width = width.map(|width| width.clamp(self.min.width, self.max.width));
        let height = height.map(|height| height.clamp(self.min.height, self.max.height));

        Self {
            min: Size::new(width.unwrap_or(self.min.width), height.unwrap_or(self.min.height)),
            max: Size::new(width.unwrap_or(self.max.width), height.unwrap_or(self.max.height)),
        }
    }

    #[cfg(feature = "flexbox")]
    /// Creates loose flex-item constraints with either axis optionally made exact.
    fn flex_item(
        direction: FlexDirection,
        available_main: u32,
        available_cross: u32,
        exact_main: Option<u32>,
        exact_cross: Option<u32>,
    ) -> Self {
        Self {
            min: direction.size(exact_main.unwrap_or(0), exact_cross.unwrap_or(0)),
            max: direction
                .size(exact_main.unwrap_or(available_main), exact_cross.unwrap_or(available_cross)),
        }
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
    const fn offset(self, main: u32, cross: u32) -> Point {
        let main = to_i32(main);
        let cross = to_i32(cross);

        match self {
            Self::Row => Point::new(main, cross),
            Self::Column => Point::new(cross, main),
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

/// Helper type to avoid borrows while `layout_children` mutates children.
enum ContentLayout {
    /// Flexbox layout.
    Flex(FlexLayout),
    // A leaf with an inherent size.
    Leaf(Size),
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

        let content_layout = match &self.node(index).kind {
            NodeKind::Div(style) => ContentLayout::Flex(style.specific.clone().into()),
            NodeKind::Text(text) => ContentLayout::Leaf(content_constraints.constrain(text.size)),
        };

        #[cfg(feature = "flexbox")]
        let mut pending_flex = None;

        let intrinsic_content_size = match content_layout {
            #[cfg(not(feature = "flexbox"))]
            ContentLayout::Flex(layout) => self.layout_children(index, content_constraints, layout),
            #[cfg(feature = "flexbox")]
            ContentLayout::Flex(layout) => {
                // Run the double pass flexbox layout algorithm: measure first, then position.
                let measurements = self.measure_items(index, content_constraints, &layout);

                // This is the intrinsic size of the content, before any flexbox layout is applied.
                let intrinsic = layout.direction.size(measurements.main, measurements.cross);
                pending_flex = Some((measurements, layout));

                intrinsic
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
            let desired_border_size = Size::new(
                box_style.width.unwrap_or(natural_border_size.width).max(content_inset_size.width),
                box_style
                    .height
                    .unwrap_or(natural_border_size.height)
                    .max(content_inset_size.height),
            );

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

        // Apply flexbox layout after all the other operations.
        #[cfg(feature = "flexbox")]
        if let Some((measurements, layout)) = pending_flex {
            let measurements = if layout.align_items.is_stretch() || measurements.needs_flexing() {
                self.grow_and_stretch_items(index, content_size, &layout, measurements)
            } else {
                measurements
            };

            self.position_items(index, content_size, &layout, measurements);
        }
    }

    /// Lays out the container at `index` along `axis`:
    /// - Traverses the children and sets their offsets.
    /// - Sums the children's sizes on the main axis and takes their maximum size on the cross axis.
    /// - Returns the constrained intrinsic size.
    #[cfg(not(feature = "flexbox"))]
    fn layout_children(
        &mut self,
        parent: NodeIndex,
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
        let mut next = self.node(parent).child;

        while let Some(child) = next {
            // First we lay out the child node, as layout calculations flow upwards
            // (with constraints pushing downwards).
            self.layout_node(child, constraints);

            let size = self.node(child).layout.outer_size;
            // Get the child's main and cross sizes based on direction.
            let (child_main, child_cross) = direction.split(size);

            // Calculate the next offset based on the main cursor.
            let offset = direction.offset(main, 0);
            self.node_mut(child).layout.set_offset(offset);

            main = main.saturating_add(child_main);
            cross = cross.max(child_cross);

            next = self.node(child).sibling;

            // Only add the gap if there's a next sibling.
            if next.is_some() {
                main = main.saturating_add(gap);
            }
        }

        constraints.constrain(direction.size(main, cross))
    }
}

/// The measurements of a flex container's children.
#[cfg(feature = "flexbox")]
#[derive(Clone, Copy, Default)]
struct FlexMeasurements {
    main: u32,
    cross: u32,
    count: u32,
    /// Main-axis space occupied by gaps, margins, and flex bases.
    flex_base_main: u32,
    total_grow: u32,
    has_zero_basis: bool,
}

#[cfg(feature = "flexbox")]
#[derive(Clone, Copy)]
/// Measurements needed to resolve one flex item.
struct FlexItemMetrics {
    outer_size: Size,
    main: u32,
    cross: u32,
    /// Flex basis plus fixed main-axis margins.
    base_main: u32,
    grow: u32,
    has_zero_basis: bool,
    has_auto_cross_size: bool,
}

#[cfg(feature = "flexbox")]
/// Distributes integer free space without losing remainder pixels.
struct GrowDistribution {
    free: u32,
    total_grow: u32,
    accumulated_grow: u32,
    distributed: u32,
}

#[cfg(feature = "flexbox")]
impl GrowDistribution {
    const fn new(free: u32, total_grow: u32) -> Self {
        Self { free, total_grow, accumulated_grow: 0, distributed: 0 }
    }

    const fn share(&mut self, grow: u32) -> u32 {
        self.accumulated_grow = self.accumulated_grow.saturating_add(grow);

        let distributed = if self.total_grow == 0 {
            0
        } else {
            ((self.free as u64 * self.accumulated_grow as u64) / self.total_grow as u64) as u32
        };
        let share = distributed - self.distributed;
        self.distributed = distributed;

        share
    }
}

#[cfg(feature = "flexbox")]
impl FlexMeasurements {
    fn add(&mut self, size: Size, direction: FlexDirection, gap: u32) {
        let (main, cross) = direction.split(size);

        if self.count > 0 {
            self.main = self.main.saturating_add(gap);
        }

        self.main = self.main.saturating_add(main);
        self.cross = self.cross.max(cross);
        self.count = self.count.saturating_add(1);
    }

    const fn needs_flexing(&self) -> bool {
        self.total_grow > 0 || self.has_zero_basis
    }

    fn add_measured(&mut self, item: FlexItemMetrics, direction: FlexDirection, gap: u32) {
        if self.count > 0 {
            self.flex_base_main = self.flex_base_main.saturating_add(gap);
        }

        self.flex_base_main = self.flex_base_main.saturating_add(item.base_main);
        self.total_grow = self.total_grow.saturating_add(item.grow);
        self.has_zero_basis |= item.has_zero_basis;
        self.add(item.outer_size, direction, gap);
    }
}

#[cfg(feature = "flexbox")]
impl<'frame, C> FrameTree<'frame, C>
where
    C: PixelColor,
{
    /// Measures the items of a flex container, including the main and cross sizes, and the total
    /// count of items.
    fn measure_items(
        &mut self,
        parent: NodeIndex,
        constraints: Constraints,
        layout: &FlexLayout,
    ) -> FlexMeasurements {
        let direction = layout.direction;
        // Get the main-axis gap.
        let (gap, _) = direction.split(layout.gap);

        let mut measurements = FlexMeasurements::default();
        let mut next = self.node(parent).child;

        while let Some(item) = next {
            // First we lay out the child node, as layout calculations flow upwards
            // (with constraints pushing downwards).
            self.layout_node(item, constraints);

            measurements.add_measured(self.flex_item_metrics(item, direction), direction, gap);
            next = self.node(item).sibling;
        }

        measurements
    }

    /// Positions the items within the parent node based on [`FlexLayout`], particularly
    /// [`FlexLayout::justify_content`] and [`FlexLayout::align_items`].
    fn position_items(
        &mut self,
        parent: NodeIndex,
        content_size: Size,
        layout: &FlexLayout,
        measurements: FlexMeasurements,
    ) {
        let direction = layout.direction;

        // Get the available space for the items based on direction.
        let (main_available, cross_available) = direction.split(content_size);
        // Get the gap size based on direction.
        let (gap, _) = direction.split(layout.gap);

        let main_free = main_available.saturating_sub(measurements.main);

        let mut cursor = 0u32;
        let mut index = 0;
        let mut next = self.node(parent).child;

        while let Some(item) = next {
            let size = self.node(item).layout.outer_size;
            let (item_main, item_cross) = direction.split(size);

            // Calculate the values to shift the offset by along both axes
            let main_shift = layout.justify_content.shift(main_free, index, measurements.count);

            let cross_free = cross_available.saturating_sub(item_cross);
            let cross_shift = layout.align_items.shift(cross_free);

            let offset = direction.offset(cursor.saturating_add(main_shift), cross_shift);

            self.node_mut(item).layout.set_offset(offset);

            cursor = cursor.saturating_add(item_main);
            next = self.node(item).sibling;

            if next.is_some() {
                cursor = cursor.saturating_add(gap);
            }

            index += 1;
        }
    }

    fn grow_and_stretch_items(
        &mut self,
        parent: NodeIndex,
        content_size: Size,
        layout: &FlexLayout,
        initial: FlexMeasurements,
    ) -> FlexMeasurements {
        let direction = layout.direction;
        let (main_available, cross_available) = direction.split(content_size);
        let (gap, _) = direction.split(layout.gap);

        let free = main_available.saturating_sub(initial.flex_base_main);
        let mut growth = GrowDistribution::new(free, initial.total_grow);

        let mut resolved = FlexMeasurements::default();
        let mut next = self.node(parent).child;

        while let Some(item) = next {
            let metrics = self.flex_item_metrics(item, direction);
            let target_main = metrics.base_main.saturating_add(growth.share(metrics.grow));

            let exact_main = (metrics.main != target_main).then_some(target_main);
            let exact_cross = (layout.align_items.is_stretch()
                && metrics.has_auto_cross_size
                && metrics.cross != cross_available)
                .then_some(cross_available);

            if exact_main.is_some() || exact_cross.is_some() {
                self.layout_node(
                    item,
                    Constraints::flex_item(
                        direction,
                        main_available,
                        cross_available,
                        exact_main,
                        exact_cross,
                    ),
                );
            }

            let final_size = self.node(item).layout.outer_size;
            resolved.add(final_size, direction, gap);

            next = self.node(item).sibling;
        }

        resolved
    }

    fn flex_item_metrics(&self, item: NodeIndex, direction: FlexDirection) -> FlexItemMetrics {
        let node = self.node(item);
        let box_style = node.box_style();
        let flex = node.flex_item_style();
        let outer_size = node.layout.outer_size;
        let (main, cross) = direction.split(outer_size);
        let (border_main, _) = direction.split(node.layout.border_size);
        let (margin_main, _) = direction.split(box_style.margin.total_size());
        let has_auto_cross_size = match direction {
            FlexDirection::Row => box_style.height.is_none(),
            FlexDirection::Column => box_style.width.is_none(),
        };

        FlexItemMetrics {
            outer_size,
            main,
            cross,
            base_main: margin_main.saturating_add(flex.flex_basis.resolve(border_main)),
            grow: u32::from(flex.flex_grow),
            has_zero_basis: flex.flex_basis.is_zero(),
            has_auto_cross_size,
        }
    }
}
