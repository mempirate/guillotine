//! Frame tree.

use embedded_graphics::{
    draw_target::DrawTarget,
    geometry::{Point, Size},
    pixelcolor::PixelColor,
    primitives::Rectangle,
};

use crate::{
    ColumnStyle, DisplayTarget, RowStyle, StorageView, Style, TextStyle, Theme,
    common::{NodeIndex, TextRange},
    layout::{BoxLayout, Constraints, Layout},
    style::BoxStyle,
};

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
