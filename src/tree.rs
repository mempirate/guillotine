//! Frame tree.

use embedded_graphics::{
    draw_target::DrawTarget,
    geometry::{Point, Size},
    pixelcolor::PixelColor,
    primitives::Rectangle,
};

use crate::{
    ColumnStyle, DisplayTarget, RowStyle, StorageView, Style, TextStyle, Theme,
    common::{NodeIndex, SizeExt as _, TextRange, to_i32},
    layout::{BoxLayout, Constraints, Layout},
    style::BoxStyle,
};

/// The axis along which a container lays out its children.
#[derive(Clone, Copy)]
enum Axis {
    Horizontal,
    Vertical,
}

/// Operations, single pass:
/// - Recursively walk the element tree to build the frame tree. For each element, calculate hard
///   constraints (`inner_constraints`) and push them down to the children.
/// - Once the leafs are resolved, push sizes back up the tree. For container elements, also
///   calculate relative offsets for each child.
pub(crate) struct FrameTree<'frame, C>
where
    C: PixelColor,
{
    pub root: Option<NodeIndex>,
    pub storage: StorageView<'frame, C>,
}

/// A node in a [`FrameTree`].
pub(crate) struct Node<C>
where
    C: PixelColor,
{
    /// The node's element.
    pub kind: NodeKind<C>,
    /// The layout of this node.
    pub layout: Layout,

    /// Index of the first child node, if any.
    pub child: Option<NodeIndex>,
    /// Index of the next sibling node, if any.
    pub sibling: Option<NodeIndex>,
}

impl<C: PixelColor> Node<C> {
    pub(crate) const fn box_style(&self) -> BoxStyle {
        self.kind.box_style()
    }

    pub(crate) fn draw<D>(&self, layout: &BoxLayout, target: &mut D) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = C>,
    {
        self.kind.draw(layout, target)
    }

    /// Sets the index of the next sibling node.
    pub(crate) const fn set_sibling(&mut self, sibling: NodeIndex) {
        self.sibling = Some(sibling);
    }
}

impl<'frame, C> FrameTree<'frame, C>
where
    C: PixelColor,
{
    /// Creates a new (unresolved) frame tree from the given storage.
    pub(crate) const fn new(storage: StorageView<'frame, C>) -> Self {
        Self { root: None, storage }
    }

    /// Returns the number of nodes in the frame tree.
    #[allow(unused)]
    pub(crate) fn len(&self) -> usize {
        self.storage.nodes.len()
    }

    /// Returns a reference to the node at the given index.
    #[inline]
    pub(crate) fn node(&self, index: NodeIndex) -> &Node<C> {
        &self.storage.nodes[index]
    }

    /// Returns a mutable reference to the node at the given index.
    #[inline]
    pub(crate) fn node_mut(&mut self, index: NodeIndex) -> &mut Node<C> {
        &mut self.storage.nodes[index]
    }

    /// Resolves the tree: lays out the root node and its children recursively.
    pub(crate) fn resolve(&mut self, root: NodeIndex, constraints: Constraints) {
        self.root = Some(root);
        self.layout_node(root, constraints);
    }

    /// Draws the frame tree onto the given target, starting with `root` at `offset`.
    pub(crate) fn draw<D>(&mut self, target: &mut D, theme: &Theme<C>) -> Result<(), D::Error>
    where
        D: DisplayTarget<Color = C>,
    {
        let Some(root) = self.root else { return target.clear(theme.background) };

        let viewport = target.bounding_box();

        if self.needs_clear(viewport.size) {
            // A transparent or partial root requires the complete viewport to be
            // initialized. Prefer doing that invisibly in the framebuffer.
            if target.try_begin(viewport, theme.background) {
                self.draw_subtree(root, Point::zero(), target, theme)?;
                return target.flush();
            }

            // The viewport doesn't fit, so a physical clear is unavoidable.
            target.clear(theme.background)?;
        }

        self.draw_adaptive(root, Point::zero(), theme.background, viewport, target, theme)
    }

    /// Returns whether the display must be cleared before drawing the frame.
    ///
    /// Clearing can be skipped only when the root node paints a background across the entire
    /// viewport. An empty tree, a transparent root, or a root that leaves any part of the viewport
    /// uncovered requires a clear to remove pixels from the previous frame.
    pub(crate) fn needs_clear(&self, viewport: Size) -> bool {
        let Some(root) = self.root else {
            return true;
        };

        let root = self.node(root);

        if !root.kind.has_background() {
            return true;
        }

        let root_bounds = root.layout.resolve(Point::zero()).border;
        let viewport = Rectangle::new(Point::zero(), viewport);

        root_bounds.intersection(&viewport) != viewport
    }

    /// Adaptive drawing that supports frame buffering through [`DisplayTarget`].
    /// For each subtree, it checks whether the first node's bounds fits within the buffer using
    /// [`DisplayTarget::try_begin`]. If it does, that whole subtree is drawn into the buffer.
    ///
    /// If the subtree does not fit within the buffer, it is drawn directly into the target without
    /// using the buffer. Each child node will be tested recursively for frame buffer capacity.
    fn draw_adaptive<D>(
        &mut self,
        index: NodeIndex,
        parent_origin: Point,
        inherited_background: C,
        viewport: Rectangle,
        target: &mut D,
        theme: &Theme<C>,
    ) -> Result<(), D::Error>
    where
        D: DisplayTarget<Color = C>,
    {
        let layout = self.node(index).layout.resolve(parent_origin);
        let bounds = layout.border.intersection(&viewport);

        if bounds.size != Size::zero() && target.try_begin(bounds, inherited_background) {
            self.draw_subtree(index, parent_origin, target, theme)?;
            target.flush()?;
            return Ok(());
        }

        // This node didn't fit, so it is painted directly. The children will be tried to fit
        // into the remaining space in the next recursive call.
        self.draw_node(index, &layout, target, theme)?;

        let child_background = self.node(index).kind.background().unwrap_or(inherited_background);
        let mut child = self.node(index).child;

        while let Some(index) = child {
            self.draw_adaptive(
                index,
                layout.content.top_left,
                child_background,
                viewport,
                target,
                theme,
            )?;

            child = self.node(index).sibling;
        }

        Ok(())
    }

    fn draw_node<D>(
        &self,
        index: NodeIndex,
        layout: &BoxLayout,
        target: &mut D,
        theme: &Theme<C>,
    ) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = C>,
    {
        let node = self.node(index);

        if let NodeKind::Text(text) = &node.kind {
            // Extract the content of the text node.
            let content = text.content(self.storage.text);

            text.draw(content, layout, target, theme)
        } else {
            node.draw(layout, target)
        }
    }

    fn draw_subtree<D>(
        &mut self,
        index: NodeIndex,
        parent_origin: Point,
        target: &mut D,
        theme: &Theme<C>,
    ) -> Result<(), D::Error>
    where
        D: DisplayTarget<Color = C>,
    {
        let layout = self.node(index).layout.resolve(parent_origin);

        self.draw_node(index, &layout, target, theme)?;

        let mut child = self.node(index).child;

        while let Some(index) = child {
            self.draw_subtree(index, layout.content.top_left, target, theme)?;
            child = self.node(index).sibling;
        }

        Ok(())
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

pub(crate) enum NodeKind<C> {
    Row(Style<RowStyle, C>),
    Column(Style<ColumnStyle, C>),
    Text(TextNode<C>),
}

impl<C: PixelColor> NodeKind<C> {
    pub(crate) const fn box_style(&self) -> BoxStyle {
        match self {
            Self::Row(style) => style.box_style(),
            Self::Column(style) => style.box_style(),
            Self::Text(text) => text.style.box_style(),
        }
    }

    /// Returns the background color of this node, if any.
    pub(crate) const fn background(&self) -> Option<C> {
        match self {
            Self::Row(style) => style.background,
            Self::Column(style) => style.background,
            Self::Text(text) => text.style.background,
        }
    }
}

pub(crate) struct TextNode<C> {
    pub(crate) range: TextRange,
    pub(crate) size: Size,
    pub(crate) style: Style<TextStyle<C>, C>,
}

impl<C: PixelColor> TextNode<C> {
    /// Extracts the content of this text node as a `&str` slice.
    pub(crate) fn content<'a>(&self, storage: &'a [u8]) -> &'a str {
        let end = self.range.offset + self.range.len;

        unsafe { core::str::from_utf8_unchecked(&storage[self.range.offset..end]) }
    }
}
