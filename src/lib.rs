//! # Guillotine
#![no_std]
#![doc = include_str!("../README.md")]

#[cfg(feature = "framebuffer")]
pub mod buffered;
mod common;
mod draw;
mod element;
mod layout;
pub mod style;
mod target;
mod theme;
mod tree;

use embedded_graphics::{pixelcolor::Rgb565, prelude::PixelColor};

pub use element::{
    BuildError, DivStyle, ElementBuilder, Font, ParentElement, StyledFlexContainer, TextStyle,
    TextStyledElement,
};
use heapless::VecView;
pub use style::{Insets, Style, StyledElement};
pub use target::{DirectTarget, DisplayTarget};
pub use theme::Theme;

use crate::{
    common::{NodeIndex, TextRange},
    layout::Constraints,
    tree::{FrameTree, Node},
};

/// Storage backed by [`heapless::Vec`] that holds frame data for rendering. Capacity is fixed at
/// `N` items.
#[derive(Default)]
pub struct FrameStorage<C: PixelColor, const N: usize = 64, const T: usize = 1024> {
    nodes: heapless::Vec<Node<C>, N>,
    /// A buffer for UTF-8 encoded text content. The reason we don't store this inside
    /// of [`Node`] ([`TextNode`]) is to reduce memory usage. Since [`Node`] is a fixed-size
    /// struct, storing text with capacity `N` bytes would carry over to all [`Node`] instances,
    /// even if they don't contain text.
    text: heapless::Vec<u8, T>,
}

/// Tracks the usage of a [`FrameStorage`] buffer.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct FrameUsage {
    /// The number of used nodes.
    pub nodes: usize,
    /// The number of bytes used for text content.
    pub text: usize,
}

/// Tracks the capacity of a [`FrameStorage`] buffer.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct FrameCapacity {
    /// The maximum number of nodes.
    pub nodes: usize,
    /// The maximum number of bytes for text content.
    pub text: usize,
}

impl<C: PixelColor, const N: usize, const T: usize> FrameStorage<C, N, T> {
    /// Returns a mutable view into this storage buffer.
    pub const fn view(&mut self) -> StorageView<'_, C> {
        StorageView { nodes: &mut self.nodes, text: &mut self.text }
    }

    /// Clears all nodes and text from this storage buffer.
    pub fn clear(&mut self) {
        self.nodes.clear();
        self.text.clear();
    }

    /// Returns the size of this storage buffer in bytes.
    pub const fn size(&self) -> usize {
        core::mem::size_of::<Self>()
    }

    /// Returns the number of elements currently used in this storage buffer.
    pub fn usage(&self) -> FrameUsage {
        FrameUsage { nodes: self.nodes.len(), text: self.text.len() }
    }

    /// Returns the capacity of this storage buffer.
    pub const fn capacity(&self) -> FrameCapacity {
        FrameCapacity { nodes: N, text: T }
    }
}

/// A capacity-erased mutable view into a [`FrameStorage`] buffer.
pub struct StorageView<'frame, C: PixelColor> {
    nodes: &'frame mut VecView<Node<C>>,
    text: &'frame mut VecView<u8>,
}

/// The [`Ui`] struct is the main entrypoint for the Guillotine UI framework.
/// It manages the display and takes care of rendering the UI from a tree of [`Element`]s,
/// with [`Self::render`].
pub struct Ui<D, const N: usize = 64, const T: usize = 1024>
where
    D: DisplayTarget,
{
    display: D,
    storage: FrameStorage<D::Color, N, T>,
    theme: Theme<D::Color>,
}

/// An error encountered while building or drawing a frame.
#[derive(Debug, thiserror::Error)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum RenderError<E> {
    /// Building the frame exceeded one of its fixed-capacity arenas.
    #[error(transparent)]
    Build(#[from] BuildError),
    /// The display returned an error while clearing or drawing the frame.
    #[error("display drawing failed")]
    Draw(E),
}

impl<D, const N: usize, const T: usize> Ui<D, N, T>
where
    D: DisplayTarget,
{
    /// Creates a new [`Ui`] instance with an explicit theme.
    ///
    /// This constructor supports arbitrary custom [`PixelColor`] implementations. For the
    /// standard embedded-graphics color types, [`Ui::new`] supplies a black and white theme.
    pub const fn with_theme(
        display: D,
        storage: FrameStorage<D::Color, N, T>,
        theme: Theme<D::Color>,
    ) -> Self {
        Self { display, storage, theme }
    }

    /// Returns a new [`Ui`] instance with the given background color. This background color will
    /// be used to clear dirty regions before rendering.
    pub const fn with_background(mut self, background: D::Color) -> Self {
        self.theme.background = background;
        self
    }

    /// Returns a new [`Ui`] instance with the given default text color.
    pub const fn with_foreground(mut self, foreground: D::Color) -> Self {
        self.theme.foreground = foreground;
        self
    }

    /// Renders the given `view` onto the display.
    pub fn render<V>(&mut self, view: &V) -> Result<(), RenderError<D::Error>>
    where
        V: Render<D::Color>,
    {
        self.storage.clear();

        let frame = self.storage.view();
        let cx = Context::new(frame);

        let root = view.render(&cx).try_build()?;

        // Create the viewport constraints.
        let viewport = Constraints::max(self.display.size());

        // Build and resolve the frame tree.
        let mut tree = FrameTree::new(cx.storage.into_inner());
        tree.layout(root, viewport);

        tree.draw(&self.theme, &mut self.display).map_err(RenderError::Draw)?;

        Ok(())
    }

    /// Returns a reference to the display.
    pub const fn display(&self) -> &D {
        &self.display
    }

    /// Returns a mutable reference to the display.
    pub const fn display_mut(&mut self) -> &mut D {
        &mut self.display
    }

    /// Returns the display and consumes the [`Ui`] instance.
    pub fn into_display(self) -> D {
        self.display
    }

    /// Returns the UI theme.
    pub const fn theme(&self) -> &Theme<D::Color> {
        &self.theme
    }

    /// Returns a reference to the frame storage.
    pub const fn storage(&self) -> &FrameStorage<D::Color, N, T> {
        &self.storage
    }
}

impl<D, const N: usize, const T: usize> Ui<D, N, T>
where
    D: DisplayTarget,
    Theme<D::Color>: Default,
{
    /// Creates a new [`Ui`] instance with a black background and white foreground.
    pub fn new(display: D, storage: FrameStorage<D::Color, N, T>) -> Self {
        Self::with_theme(display, storage, Theme::default())
    }
}

/// A helper trait for building complex objects with imperative conditionals in a fluent style.
pub trait FluentBuilder {
    /// Imperatively modify self with the given closure.
    fn map<U>(self, f: impl FnOnce(Self) -> U) -> U
    where
        Self: Sized,
    {
        f(self)
    }

    /// Conditionally modify self with the given closure.
    fn when(self, condition: bool, then: impl FnOnce(Self) -> Self) -> Self
    where
        Self: Sized,
    {
        self.map(|this| if condition { then(this) } else { this })
    }

    /// Conditionally modify self with the given closure.
    fn when_else(
        self,
        condition: bool,
        then: impl FnOnce(Self) -> Self,
        else_fn: impl FnOnce(Self) -> Self,
    ) -> Self
    where
        Self: Sized,
    {
        self.map(|this| if condition { then(this) } else { else_fn(this) })
    }

    /// Conditionally unwrap and modify self with the given closure, if the given option is Some.
    fn when_some<T>(self, option: Option<T>, then: impl FnOnce(Self, T) -> Self) -> Self
    where
        Self: Sized,
    {
        self.map(|this| if let Some(value) = option { then(this, value) } else { this })
    }
    /// Conditionally unwrap and modify self with the given closure, if the given option is None.
    fn when_none<T>(self, option: &Option<T>, then: impl FnOnce(Self) -> Self) -> Self
    where
        Self: Sized,
    {
        self.map(|this| if option.is_some() { this } else { then(this) })
    }
}

// Transparent implementation of FluentBuilder for all IntoElement types.
impl<T: ElementBuilder> FluentBuilder for T {}

/// The [`Render`] trait is implemented by types that can be rendered into an [`Element`]. Use this
/// trait to define UI elements.
///
/// `C` defaults to [`Rgb565`] to preserve the simple API for existing views. A view for another
/// display color declares that color once in its implementation, for example
/// `impl Render<BinaryColor> for MyView`. Element constructors inside `render` infer the color from
/// its return type and don't need explicit generic arguments.
pub trait Render<C = Rgb565>
where
    C: PixelColor,
{
    /// Renders this element into an [`Element`] using the given [`Context`].
    fn render(&self, cx: &Context<'_, C>) -> impl ElementBuilder;
}

/// For now, unused. In the future, will be used for context management, such as:
/// - Allocating and managing retained resources
/// - Interactivity (from UI upstream)
pub struct Context<'frame, C: PixelColor = Rgb565> {
    storage: core::cell::RefCell<StorageView<'frame, C>>,
}

impl<'frame, C: PixelColor> Context<'frame, C> {
    /// Creates a new [`Context`] with the given [`FrameStorage`].
    const fn new(storage: StorageView<'frame, C>) -> Self {
        Self { storage: core::cell::RefCell::new(storage) }
    }

    /// Links the sibling nodes of the given indices (i.e., sets the `sibling` field of the previous
    /// node to the next node's index)
    fn link_sibling(&self, node: NodeIndex, sibling: NodeIndex) {
        let mut storage = self.storage.borrow_mut();

        storage.nodes[node].set_sibling(sibling);
    }

    /// Inserts a node into the storage, returning its index.
    /// Returns `None` if storage is full.
    fn insert(&self, node: Node<C>) -> Result<NodeIndex, BuildError> {
        let mut storage = self.storage.borrow_mut();
        let index = storage.nodes.len();
        storage.nodes.push(node).map_err(|_| BuildError::NodeCapacity)?;

        Ok(index)
    }

    /// Stores the given text content in the storage, returning a [`TextRange`] that can be used to
    /// retrieve the text later.
    fn store_text(&self, content: &str) -> Result<TextRange, BuildError> {
        let mut storage = self.storage.borrow_mut();

        let offset = storage.text.len();
        let len = content.len();
        let end = offset.checked_add(len).ok_or(BuildError::TextCapacity)?;

        if end > storage.text.capacity() {
            return Err(BuildError::TextCapacity);
        }

        // Store the content as UTF-8 bytes.
        storage.text.extend_from_slice(content.as_bytes()).map_err(|_| BuildError::TextCapacity)?;

        Ok(TextRange { offset, len })
    }
}

#[cfg(test)]
mod tests {
    use embedded_graphics::{
        mock_display::MockDisplay,
        pixelcolor::{BinaryColor, Rgb565},
        prelude::{Point, RgbColor as _, Size},
        primitives::Rectangle,
    };

    use crate::{element::DivStyle, style::FlexDirection, tree::NodeKind};

    use super::*;

    struct Dashboard {
        text: &'static str,
    }

    impl Render for Dashboard {
        fn render(&self, cx: &Context<'_>) -> impl ElementBuilder {
            cx.row()
                .child(cx.text(self.text))
                .child(cx.row().child(cx.text("Nested")))
                .when(true, |row| row.child(cx.text("Conditional")))
                .children([cx.text("Copyright"), cx.text("ACME Corp")])
        }
    }

    fn child_count<C: PixelColor>(tree: &FrameTree<'_, C>, parent: NodeIndex) -> usize {
        let mut count = 0;
        let mut child = tree.node(parent).child;

        while let Some(index) = child {
            count += 1;
            child = tree.node(index).sibling;
        }

        count
    }

    fn nth_child<C: PixelColor>(
        tree: &FrameTree<'_, C>,
        parent: NodeIndex,
        position: usize,
    ) -> Option<NodeIndex> {
        let mut child = tree.node(parent).child;

        for _ in 0..position {
            child = child.and_then(|index| tree.node(index).sibling);
        }

        child
    }

    fn text_content<'a, C: PixelColor>(tree: &'a FrameTree<'_, C>, index: NodeIndex) -> &'a str {
        match &tree.node(index).kind {
            NodeKind::Text(text) => text.content(tree.storage.text),
            _ => panic!("expected a text node"),
        }
    }

    #[test]
    fn transparent_root_requires_clear() {
        let viewport = Size::new(20, 20);
        let mut storage = FrameStorage::<Rgb565, 1, 1>::default();
        let cx = Context::new(storage.view());
        let root = cx.column().size(viewport).try_build().unwrap();
        let mut tree = FrameTree::new(cx.storage.into_inner());

        tree.layout(root, Constraints::max(viewport));

        assert!(tree.needs_clear(viewport));
    }

    #[test]
    fn partial_opaque_root_requires_clear() {
        let viewport = Size::new(20, 20);
        let mut storage = FrameStorage::<Rgb565, 1, 1>::default();
        let cx = Context::new(storage.view());
        let root =
            cx.column().background(Rgb565::BLACK).size(Size::new(10, 20)).try_build().unwrap();
        let mut tree = FrameTree::new(cx.storage.into_inner());

        tree.layout(root, Constraints::max(viewport));

        assert!(tree.needs_clear(viewport));
    }

    #[test]
    fn full_viewport_opaque_root_does_not_require_clear() {
        let viewport = Size::new(20, 20);
        let mut storage = FrameStorage::<Rgb565, 1, 1>::default();
        let cx = Context::new(storage.view());
        let root = cx.column().background(Rgb565::BLACK).size(viewport).try_build().unwrap();
        let mut tree = FrameTree::new(cx.storage.into_inner());

        tree.layout(root, Constraints::max(viewport));

        assert!(!tree.needs_clear(viewport));
    }

    #[test]
    fn row_composes_heterogeneous_and_conditional_children() {
        let dashboard = Dashboard { text: "Hello, World!" };
        let mut storage = FrameStorage::<Rgb565, 10, 64>::default();
        let cx = Context::new(storage.view());

        let root = dashboard.render(&cx).try_build().unwrap();
        let tree = FrameTree::new(cx.storage.into_inner());

        assert!(matches!(tree.node(root).kind, NodeKind::Div(_)));
        assert_eq!(child_count(&tree, root), 5);

        let nested = nth_child(&tree, root, 1).unwrap();
        assert!(matches!(tree.node(nested).kind, NodeKind::Div(_)));

        let conditional = nth_child(&tree, root, 2).unwrap();
        assert_eq!(text_content(&tree, conditional), "Conditional");
    }

    #[test]
    fn column_composes_heterogeneous_and_conditional_children() {
        let mut storage = FrameStorage::<Rgb565, 10, 64>::default();
        let cx = Context::new(storage.view());

        let root = cx
            .column()
            .child(cx.text("First"))
            .child(cx.row().child(cx.text("Nested")))
            .when(true, |column| column.child(cx.text("Conditional")))
            .children([cx.text("Fourth"), cx.text("Fifth")])
            .try_build()
            .unwrap();
        let tree = FrameTree::new(cx.storage.into_inner());

        assert!(matches!(tree.node(root).kind, NodeKind::Div(_)));
        assert_eq!(child_count(&tree, root), 5);

        let nested = nth_child(&tree, root, 1).unwrap();
        assert!(matches!(tree.node(nested).kind, NodeKind::Div(_)));

        let conditional = nth_child(&tree, root, 2).unwrap();
        assert_eq!(text_content(&tree, conditional), "Conditional");
    }

    #[test]
    fn column_stacks_children_vertically_and_uses_the_widest_child() {
        let mut storage = FrameStorage::<Rgb565, 8, 8>::default();
        let cx = Context::new(storage.view());

        let root = cx
            .column()
            .child(
                cx.row()
                    .child(cx.text("a").size(Size::new(10, 5)))
                    .child(cx.text("b").size(Size::new(20, 7))),
            )
            .child(cx.text("c").size(Size::new(15, 9)))
            .try_build()
            .unwrap();

        let mut tree = FrameTree::new(cx.storage.into_inner());
        tree.layout(root, Constraints::exact(Size::new(100, 100)).loosen());

        assert_eq!(tree.node(root).layout.outer_size, Size::new(30, 16));

        let row = tree.node(root).child.expect("column should have a row child");
        let last_text = tree.node(row).sibling.expect("column should have a text child");
        assert_eq!(tree.node(row).layout.offset, Point::zero());
        assert_eq!(tree.node(last_text).layout.offset, Point::new(0, 7));

        let first_text = tree.node(row).child.expect("row should have a text child");
        let second_text = tree.node(first_text).sibling.expect("row should have two children");
        assert_eq!(tree.node(first_text).layout.offset, Point::zero());
        assert_eq!(tree.node(second_text).layout.offset, Point::new(10, 0));
    }

    #[test]
    fn flex_gaps_apply_on_the_main_axis_and_contribute_to_size() {
        let mut row_storage = FrameStorage::<Rgb565, 3, 1>::default();
        let row_cx = Context::new(row_storage.view());
        let row = row_cx
            .row()
            .gap(4)
            .child(row_cx.text("").size(Size::new(10, 5)))
            .child(row_cx.text("").size(Size::new(20, 7)))
            .try_build()
            .unwrap();
        let mut row_tree = FrameTree::new(row_cx.storage.into_inner());
        row_tree.layout(row, Constraints::exact(Size::new(100, 100)).loosen());

        let row_first = row_tree.node(row).child.unwrap();
        let row_second = row_tree.node(row_first).sibling.unwrap();
        assert_eq!(row_tree.node(row_second).layout.offset, Point::new(14, 0));
        assert_eq!(row_tree.node(row).layout.outer_size, Size::new(34, 7));

        let mut column_storage = FrameStorage::<Rgb565, 3, 1>::default();
        let column_cx = Context::new(column_storage.view());
        let column = column_cx
            .column()
            .gap(3)
            .child(column_cx.text("").size(Size::new(10, 5)))
            .child(column_cx.text("").size(Size::new(20, 7)))
            .try_build()
            .unwrap();
        let mut column_tree = FrameTree::new(column_cx.storage.into_inner());
        column_tree.layout(column, Constraints::exact(Size::new(100, 100)).loosen());

        let column_first = column_tree.node(column).child.unwrap();
        let column_second = column_tree.node(column_first).sibling.unwrap();
        assert_eq!(column_tree.node(column_second).layout.offset, Point::new(0, 8));
        assert_eq!(column_tree.node(column).layout.outer_size, Size::new(20, 15));
    }

    #[test]
    fn column_draws_its_styled_border_box() {
        let column = NodeKind::<Rgb565>::Div(Style {
            border: 1.into(),
            border_color: Some(Rgb565::BLUE),
            background: Some(Rgb565::RED),
            specific: DivStyle { gap: Size::zero(), direction: FlexDirection::Column },
            ..Default::default()
        });
        let layout = layout::BoxLayout {
            border: Rectangle::new(Point::new(1, 1), Size::new(4, 4)),
            content: Rectangle::new(Point::new(2, 2), Size::new(2, 2)),
        };
        let mut display = MockDisplay::new();
        display.set_allow_overdraw(true);

        column.draw(&layout, &mut display).unwrap();

        assert_eq!(display.get_pixel(Point::new(1, 1)), Some(Rgb565::BLUE));
        assert_eq!(display.get_pixel(Point::new(2, 2)), Some(Rgb565::RED));
        assert_eq!(display.get_pixel(Point::zero()), None);
    }

    #[test]
    fn asymmetric_insets_drive_size_and_offsets() {
        let mut storage = FrameStorage::<Rgb565, 1, 1>::default();
        let cx = Context::new(storage.view());

        let root = cx
            .text("")
            .margin(Insets::new(1, 2, 3, 4))
            .border(Insets::new(1, 2, 3, 4))
            .padding(Insets::new(5, 6, 7, 8))
            .size(Size::new(30, 25))
            .try_build()
            .unwrap();

        let mut tree = FrameTree::new(cx.storage.into_inner());
        tree.layout(root, Constraints::exact(Size::new(100, 100)).loosen());
        let layout = &tree.node(root).layout;

        assert_eq!(layout.border_size, Size::new(30, 25));
        assert_eq!(layout.outer_size, Size::new(36, 29));
        assert_eq!(layout.content_size, Size::new(10, 9));
        assert_eq!(layout.border_offset, Point::new(4, 1));
        assert_eq!(layout.content_offset, Point::new(16, 7));
    }

    #[test]
    fn adjacent_margins_add_in_rows() {
        let mut storage = FrameStorage::<Rgb565, 3, 1>::default();
        let cx = Context::new(storage.view());

        let root = cx
            .row()
            .child(cx.text("").margin(Insets::new(0, 2, 0, 0)).size(Size::new(10, 10)))
            .child(cx.text("").margin(Insets::new(0, 0, 0, 3)).size(Size::new(10, 10)))
            .try_build()
            .unwrap();

        let mut tree = FrameTree::new(cx.storage.into_inner());
        tree.layout(root, Constraints::exact(Size::new(100, 100)).loosen());

        let first = tree.node(root).child.expect("row should have children");
        let second = tree.node(first).sibling.expect("row should have two children");

        assert_eq!(tree.node(first).layout.outer_size, Size::new(12, 10));
        assert_eq!(tree.node(second).layout.offset, Point::new(12, 0));

        let first_box = tree.node(first).layout.resolve(Point::zero()).border;
        let second_box = tree.node(second).layout.resolve(Point::zero()).border;
        assert_eq!(first_box.top_left.x + 10, 10);
        assert_eq!(second_box.top_left.x, 15);
    }

    #[test]
    fn explicit_size_grows_for_insets_but_hard_constraints_win() {
        let mut loose_storage = FrameStorage::<Rgb565, 1, 1>::default();
        let loose_cx = Context::new(loose_storage.view());
        let loose =
            loose_cx.text("").padding(4).border(2).size(Size::new(5, 5)).try_build().unwrap();
        let mut loose_tree = FrameTree::new(loose_cx.storage.into_inner());
        loose_tree.layout(loose, Constraints::exact(Size::new(100, 100)).loosen());

        assert_eq!(loose_tree.node(loose).layout.border_size, Size::new(12, 12));
        assert_eq!(loose_tree.node(loose).layout.content_size, Size::zero());

        let mut constrained_storage = FrameStorage::<Rgb565, 1, 1>::default();
        let constrained_cx = Context::new(constrained_storage.view());
        let constrained =
            constrained_cx.text("").padding(4).border(2).size(Size::new(5, 5)).try_build().unwrap();
        let mut constrained_tree = FrameTree::new(constrained_cx.storage.into_inner());
        constrained_tree.layout(constrained, Constraints::exact(Size::new(8, 8)));

        assert_eq!(constrained_tree.node(constrained).layout.border_size, Size::new(8, 8));
        assert_eq!(constrained_tree.node(constrained).layout.content_size, Size::zero());
    }

    #[test]
    fn asymmetric_borders_are_painted_inside_the_border_box() {
        let column = NodeKind::<Rgb565>::Div(Style {
            border: Insets::new(1, 2, 3, 4),
            border_color: Some(Rgb565::BLUE),
            background: Some(Rgb565::RED),
            specific: DivStyle { gap: Size::zero(), direction: FlexDirection::Column },
            ..Style::default()
        });
        let layout = layout::BoxLayout {
            border: Rectangle::new(Point::new(1, 1), Size::new(7, 7)),
            content: Rectangle::new(Point::new(5, 2), Size::new(1, 3)),
        };
        let mut display = MockDisplay::new();
        display.set_allow_overdraw(true);

        column.draw(&layout, &mut display).unwrap();

        assert_eq!(display.get_pixel(Point::new(5, 1)), Some(Rgb565::BLUE));
        assert_eq!(display.get_pixel(Point::new(6, 3)), Some(Rgb565::BLUE));
        assert_eq!(display.get_pixel(Point::new(5, 5)), Some(Rgb565::BLUE));
        assert_eq!(display.get_pixel(Point::new(4, 3)), Some(Rgb565::BLUE));
        assert_eq!(display.get_pixel(Point::new(5, 3)), Some(Rgb565::RED));
        assert_eq!(display.get_pixel(Point::zero()), None);
        assert_eq!(display.get_pixel(Point::new(8, 3)), None);
    }

    #[test]
    fn box_painting_supports_binary_color() {
        let column = NodeKind::<BinaryColor>::Div(Style {
            border: (1, 2, 1, 2).into(),
            border_color: Some(BinaryColor::On),
            background: Some(BinaryColor::Off),
            specific: DivStyle { gap: Size::zero(), direction: FlexDirection::Column },
            ..Style::default()
        });
        let layout = layout::BoxLayout {
            border: Rectangle::new(Point::zero(), Size::new(6, 4)),
            content: Rectangle::new(Point::new(2, 1), Size::new(2, 2)),
        };
        let mut display = MockDisplay::new();
        display.set_allow_overdraw(true);

        column.draw(&layout, &mut display).unwrap();

        assert_eq!(display.get_pixel(Point::zero()), Some(BinaryColor::On));
        assert_eq!(display.get_pixel(Point::new(3, 2)), Some(BinaryColor::Off));
    }
}
