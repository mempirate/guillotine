//! # Guillotine
#![no_std]

mod element;
mod layout;
pub mod style;

extern crate alloc;

use alloc::vec::Vec;
use embedded_graphics::{
    pixelcolor::{
        Bgr555, Bgr565, Bgr666, Bgr888, BinaryColor, Gray2, Gray4, Gray8, Rgb555, Rgb565, Rgb666,
        Rgb888,
    },
    prelude::{DrawTarget, GrayColor, OriginDimensions, PixelColor, Point, RgbColor, Size},
};

pub use element::*;
pub use style::{Insets, Style};

use crate::layout::{Constraints, Layout};

/// Colors used by the UI when an element doesn't specify a color explicitly.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Theme<C>
where
    C: PixelColor,
{
    /// Color used to clear the display before drawing a frame.
    pub background: C,
    /// Default text color.
    pub foreground: C,
}

impl<C> Theme<C>
where
    C: PixelColor,
{
    /// Creates a theme from its background and foreground colors.
    pub const fn new(background: C, foreground: C) -> Self {
        Self { background, foreground }
    }
}

macro_rules! impl_rgb_theme {
    ($($color:ty),+ $(,)?) => {
        $(
            impl Default for Theme<$color> {
                fn default() -> Self {
                    Self::new(<$color>::BLACK, <$color>::WHITE)
                }
            }
        )+
    };
}

impl_rgb_theme!(Rgb555, Bgr555, Rgb565, Bgr565, Rgb666, Bgr666, Rgb888, Bgr888);

macro_rules! impl_gray_theme {
    ($($color:ty),+ $(,)?) => {
        $(
            impl Default for Theme<$color> {
                fn default() -> Self {
                    Self::new(<$color>::BLACK, <$color>::WHITE)
                }
            }
        )+
    };
}

impl_gray_theme!(Gray2, Gray4, Gray8);

impl Default for Theme<BinaryColor> {
    fn default() -> Self {
        Self::new(BinaryColor::Off, BinaryColor::On)
    }
}

/// The [`Ui`] struct is the main entrypoint for the Guillotine UI framework.
/// It manages the display and takes care of rendering the UI from a tree of [`Element`]s,
/// with [`Self::render`].
pub struct Ui<D>
where
    D: DrawTarget + OriginDimensions,
{
    display: D,
    cx: Context,
    theme: Theme<D::Color>,
}

impl<D> Ui<D>
where
    D: DrawTarget + OriginDimensions,
{
    /// Creates a new [`Ui`] instance with an explicit theme.
    ///
    /// This constructor supports arbitrary custom [`PixelColor`] implementations. For the
    /// standard embedded-graphics color types, [`Ui::new`] supplies a black and white theme.
    pub fn with_theme(display: D, theme: Theme<D::Color>) -> Self {
        Self { display, cx: Context::default(), theme }
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
    pub fn render<V>(&mut self, view: &V) -> Result<(), D::Error>
    where
        V: Render<D::Color>,
    {
        let root = view.render(&mut self.cx).into_element();

        let viewport = Constraints::exact(self.display.size());

        // Resolve the root node and build the tree.
        let mut tree = FrameTree::new();
        let _ = tree.resolve(root, viewport);

        self.display.clear(self.theme.background)?;

        tree.draw(&mut self.display, &self.theme)
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
}

impl<D> Ui<D>
where
    D: DrawTarget + OriginDimensions,
    Theme<D::Color>: Default,
{
    /// Creates a new [`Ui`] instance with a black background and white foreground.
    pub fn new(display: D) -> Self {
        Self::with_theme(display, Theme::default())
    }
}

type NodeIndex = usize;

/// The axis along which a container lays out its children.
#[derive(Clone, Copy)]
enum Axis {
    Horizontal,
    Vertical,
}

/// Operations, single pass:
/// - Recursively walk the element tree to build the frame tree. For each element,
/// calculate hard constraints (inner_constraints) and push them down to the children.
/// - Once the leafs are resolved, push sizes back up the tree. For container elements,
/// also calculate relative offsets for each child.
struct FrameTree<'a, C>
where
    C: PixelColor,
{
    nodes: Vec<Node<'a, C>>,
    root: Option<NodeIndex>,
}

impl<'a, C> FrameTree<'a, C>
where
    C: PixelColor,
{
    /// Creates a new, empty frame tree.
    fn new() -> Self {
        Self { nodes: Vec::new(), root: None }
    }

    /// Inserts a new node into the frame tree and returns its index.
    fn insert(&mut self, node: Node<'a, C>) -> NodeIndex {
        let index = self.nodes.len();
        self.nodes.push(node);
        index
    }

    /// Returns the number of nodes in the frame tree.
    #[allow(unused)]
    fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Resolves the root of the frame tree with the given element and constraints.
    pub(crate) fn resolve(&mut self, root: Element<'a, C>, constraints: Constraints) -> NodeIndex {
        let root = resolve(self, root, constraints, None);
        self.root = Some(root);
        root
    }

    /// Draws the frame tree onto the given display, starting with `root` at `offset`.
    pub(crate) fn draw<D>(&mut self, display: &mut D, theme: &Theme<C>) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = C>,
    {
        let Some(root) = self.root else {
            return Ok(());
        };

        self.draw_node(root, Point::zero(), display, theme)
    }

    fn draw_node<D>(
        &mut self,
        index: NodeIndex,
        parent_origin: Point,
        display: &mut D,
        theme: &Theme<C>,
    ) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = C>,
    {
        let node = &mut self.nodes[index];
        let layout = node.layout.resolve(parent_origin);

        node.element.draw(&layout, display, theme)?;

        let mut child = node.child;

        while let Some(index) = child {
            self.draw_node(index, layout.content.top_left, display, theme)?;
            child = self.nodes[index].sibling;
        }

        Ok(())
    }

    // TODO: Clean up
    fn layout_node(&mut self, index: NodeIndex, constraints: Constraints) {
        // Extract the common box style properties.
        let box_style = self.nodes[index].element.box_style();

        let border_constraints = box_style.border_constraints(constraints);

        let content_constraints = box_style.content_constraints(constraints);

        // Determine the content's intrinsic size without retaining a
        // borrow of `self.nodes[index]` while layout_children mutates children.
        enum ContentLayout {
            Container(Axis),
            // A leaf with an inherent size.
            Leaf(Size),
        }

        let content_layout = match &self.nodes[index].element {
            Element::Column(_) => ContentLayout::Container(Axis::Vertical),
            Element::Row(_) => ContentLayout::Container(Axis::Horizontal),
            Element::Text(text) => {
                ContentLayout::Leaf(content_constraints.constrain(text.measure()))
            }
            Element::Custom(never) => match *never {},
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

        self.nodes[index].layout = Layout {
            // The parent assigns this later. The root remains at zero.
            offset: Point::new(0, 0),

            // Both offsets are relative to the node's outer-box origin.
            border_offset: Point::new(to_i32(box_style.margin.left), to_i32(box_style.margin.top)),

            content_offset: Point::new(to_i32(content_offset.left), to_i32(content_offset.top)),

            outer_size,
            border_size,
            content_size,

            // Absolute bounds are resolved while traversing for paint
            // or hit testing.
            bounds: None,
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
        let mut child = self.nodes[index].child;

        while let Some(child_idx) = child {
            let size = self.nodes[child_idx].layout.outer_size;

            let main_offset = i32::try_from(main_size).unwrap_or(i32::MAX);

            // TODO: Only allows for start alignment, and no gap.
            let offset = match axis {
                Axis::Horizontal => Point::new(main_offset, 0),
                Axis::Vertical => Point::new(0, main_offset),
            };
            self.nodes[child_idx].layout.set_offset(offset);

            let (child_main_size, child_cross_size) = match axis {
                Axis::Horizontal => (size.width, size.height),
                Axis::Vertical => (size.height, size.width),
            };

            main_size = main_size.saturating_add(child_main_size);
            cross_size = cross_size.max(child_cross_size);

            child = self.nodes[child_idx].sibling;
        }

        let intrinsic_size = match axis {
            Axis::Horizontal => Size::new(main_size, cross_size),
            Axis::Vertical => Size::new(cross_size, main_size),
        };

        constraints.constrain(intrinsic_size)
    }
}

trait SizeExt {
    /// Inflates the size by a horizontal and vertical amount.
    fn inflate(self, by: Size) -> Self;

    /// Deflates the size by a horizontal and vertical amount.
    fn deflate(self, by: Size) -> Self;
}

impl SizeExt for Size {
    fn inflate(self, by: Size) -> Self {
        Self::new(self.width.saturating_add(by.width), self.height.saturating_add(by.height))
    }

    fn deflate(self, by: Size) -> Self {
        self.saturating_sub(by)
    }
}

const fn to_i32(value: u32) -> i32 {
    if value > i32::MAX as u32 { i32::MAX } else { value as i32 }
}

/// Recursively populates the frame tree with the given element and parent node.
fn resolve<'a, C>(
    tree: &mut FrameTree<'a, C>,
    mut element: Element<'a, C>,
    constraints: Constraints,
    parent: Option<NodeIndex>,
) -> NodeIndex
where
    C: PixelColor,
{
    // Calculate the content constraints for the current element. These are effectively the
    // constraints that will be pushed down to the children.
    let content_constraints = element.box_style().content_constraints(constraints);

    let children = element.take_children().unwrap_or_default();

    // Insert the parent node into the tree. Not every field can be fully populated yet.
    let index =
        tree.insert(Node { element, layout: Layout::empty(), parent, child: None, sibling: None });

    let mut previous_child: Option<NodeIndex> = None;

    for child in children {
        let child = resolve(tree, child, content_constraints, Some(index));

        // If there is a previous child, set its sibling to the current child.
        // Otherwise, set the child of the parent node to the current child (as this is the first
        // child).
        if let Some(prev) = previous_child {
            tree.nodes[prev].sibling = Some(child);
        } else {
            tree.nodes[index].child = Some(child);
        }

        previous_child = Some(child);
    }

    tree.layout_node(index, constraints);

    index
}

/// A node in a [`FrameTree`].
struct Node<'a, C>
where
    C: PixelColor,
{
    /// The node's element.
    element: Element<'a, C>,
    /// The layout of this node.
    layout: Layout,

    /// Index of the parent node, if any.
    #[allow(unused)]
    parent: Option<NodeIndex>,
    /// Index of the first child node, if any.
    child: Option<NodeIndex>,
    /// Index of the next sibling node, if any.
    sibling: Option<NodeIndex>,
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
impl<T: IntoElement> FluentBuilder for T {}

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
    fn render<'a>(&'a self, cx: &mut Context) -> impl IntoElement<Element = Element<'a, C>>;
}

/// For now, unused. In the future, will be used for context management, such as:
/// - Allocating and managing retained resources
/// - Interactivity (from UI upstream)
#[derive(Default)]
pub struct Context {}

#[cfg(test)]
mod tests {
    use embedded_graphics::{
        mock_display::MockDisplay,
        pixelcolor::{BinaryColor, Rgb565},
        prelude::{Point, RgbColor as _, Size},
        primitives::Rectangle,
    };

    use super::*;

    struct Dashboard {
        text: &'static str,
    }

    impl Render for Dashboard {
        fn render<'a>(&'a self, _cx: &mut Context) -> impl IntoElement<Element = Element<'a>> {
            row()
                .child(text(self.text))
                .child(row().child(text("Nested")))
                .when(true, |row| row.child(text("Conditional")))
                .children([text("Copyright"), text("ACME Corp")])
        }
    }

    #[test]
    fn row_composes_heterogeneous_and_conditional_children() {
        let dashboard = Dashboard { text: "Hello, World!" };

        let element = dashboard.render(&mut Context::default()).into_element();

        let Element::Row(row) = element else {
            panic!("expected a row");
        };

        assert_eq!(row.children.len(), 5);
        assert!(matches!(
            &row.children[2],
            Element::Text(text) if text.content() == "Conditional"
        ));
    }

    #[test]
    fn column_composes_heterogeneous_and_conditional_children() {
        let element: Element<'_> = column()
            .child(text("First"))
            .child(row().child(text("Nested")))
            .when(true, |column| column.child(text("Conditional")))
            .children([text("Fourth"), text("Fifth")])
            .into_element();

        let Element::Column(column) = element else {
            panic!("expected a column");
        };

        assert_eq!(column.children.len(), 5);
        assert!(matches!(&column.children[1], Element::Row(_)));
        assert!(matches!(
            &column.children[2],
            Element::Text(text) if text.content() == "Conditional"
        ));
    }

    #[test]
    fn column_stacks_children_vertically_and_uses_the_widest_child() {
        fn sized_text(content: &str, size: Size) -> Text<'_> {
            text(content).with_style(Style { size: Some(size), ..Style::default() })
        }

        let element = column()
            .child(
                row()
                    .child(sized_text("a", Size::new(10, 5)))
                    .child(sized_text("b", Size::new(20, 7))),
            )
            .child(sized_text("c", Size::new(15, 9)))
            .into_element();

        let mut tree = FrameTree::new();
        let root = tree.resolve(element, Constraints::exact(Size::new(100, 100)).loosen());

        assert_eq!(tree.nodes[root].layout.outer_size, Size::new(30, 16));

        let row = tree.nodes[root].child.expect("column should have a row child");
        let last_text = tree.nodes[row].sibling.expect("column should have a text child");
        assert_eq!(tree.nodes[row].layout.offset, Point::zero());
        assert_eq!(tree.nodes[last_text].layout.offset, Point::new(0, 7));

        let first_text = tree.nodes[row].child.expect("row should have a text child");
        let second_text = tree.nodes[first_text].sibling.expect("row should have two children");
        assert_eq!(tree.nodes[first_text].layout.offset, Point::zero());
        assert_eq!(tree.nodes[second_text].layout.offset, Point::new(10, 0));
    }

    #[test]
    fn column_draws_its_styled_border_box() {
        let column = column().with_style(Style {
            border: 1.into(),
            border_color: Some(Rgb565::BLUE),
            background: Some(Rgb565::RED),
            ..Style::default()
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
        let element: Element<'_> = text("")
            .with_style(Style {
                margin: Insets::new(1, 2, 3, 4),
                border: Insets::new(1, 2, 3, 4),
                padding: Insets::new(5, 6, 7, 8),
                size: Some(Size::new(30, 25)),
                ..Style::default()
            })
            .into_element();

        let mut tree = FrameTree::new();
        let root = tree.resolve(element, Constraints::exact(Size::new(100, 100)).loosen());
        let layout = &tree.nodes[root].layout;

        assert_eq!(layout.border_size, Size::new(30, 25));
        assert_eq!(layout.outer_size, Size::new(36, 29));
        assert_eq!(layout.content_size, Size::new(10, 9));
        assert_eq!(layout.border_offset, Point::new(4, 1));
        assert_eq!(layout.content_offset, Point::new(16, 7));
    }

    #[test]
    fn adjacent_margins_add_in_rows() {
        fn box_with_margin(margin: Insets) -> Text<'static> {
            text("").with_style(Style { margin, size: Some(Size::new(10, 10)), ..Style::default() })
        }

        let element = row()
            .child(box_with_margin(Insets::new(0, 2, 0, 0)))
            .child(box_with_margin(Insets::new(0, 0, 0, 3)))
            .into_element();

        let mut tree = FrameTree::new();
        let root = tree.resolve(element, Constraints::exact(Size::new(100, 100)).loosen());
        let first = tree.nodes[root].child.expect("row should have children");
        let second = tree.nodes[first].sibling.expect("row should have two children");

        assert_eq!(tree.nodes[first].layout.outer_size, Size::new(12, 10));
        assert_eq!(tree.nodes[second].layout.offset, Point::new(12, 0));

        let first_box = tree.nodes[first].layout.resolve(Point::zero()).border;
        let second_box = tree.nodes[second].layout.resolve(Point::zero()).border;
        assert_eq!(first_box.top_left.x + 10, 10);
        assert_eq!(second_box.top_left.x, 15);
    }

    #[test]
    fn explicit_size_grows_for_insets_but_hard_constraints_win() {
        fn inset_box() -> Element<'static> {
            text("")
                .with_style(Style {
                    padding: 4.into(),
                    border: 2.into(),
                    size: Some(Size::new(5, 5)),
                    ..Style::default()
                })
                .into_element()
        }

        let mut loose_tree = FrameTree::new();
        let loose =
            loose_tree.resolve(inset_box(), Constraints::exact(Size::new(100, 100)).loosen());
        assert_eq!(loose_tree.nodes[loose].layout.border_size, Size::new(12, 12));
        assert_eq!(loose_tree.nodes[loose].layout.content_size, Size::zero());

        let mut constrained_tree = FrameTree::new();
        let constrained =
            constrained_tree.resolve(inset_box(), Constraints::exact(Size::new(8, 8)));
        assert_eq!(constrained_tree.nodes[constrained].layout.border_size, Size::new(8, 8));
        assert_eq!(constrained_tree.nodes[constrained].layout.content_size, Size::zero());
    }

    #[test]
    fn asymmetric_borders_are_painted_inside_the_border_box() {
        let column = column().with_style(Style {
            border: Insets::new(1, 2, 3, 4),
            border_color: Some(Rgb565::BLUE),
            background: Some(Rgb565::RED),
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
        let column: Column<'_, BinaryColor> = column().with_style(Style {
            border: (1, 2, 1, 2).into(),
            border_color: Some(BinaryColor::On),
            background: Some(BinaryColor::Off),
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
