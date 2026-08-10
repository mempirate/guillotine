//! # Guillotine
#![no_std]

mod element;
mod layout;
pub mod style;

extern crate alloc;

use alloc::vec::Vec;
use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::{DrawTarget, OriginDimensions, Point, RgbColor, Size},
};

pub use element::*;
pub use style::{Insets, Style};

use crate::layout::{Constraints, Layout};

/// The [`Ui`] struct is the main entrypoint for the Guillotine UI framework.
/// It manages the display and takes care of rendering the UI from a tree of [`Element`]s,
/// with [`Self::render`].
pub struct Ui<D>
where
    D: DrawTarget<Color = Rgb565> + OriginDimensions,
{
    display: D,
    cx: Context,

    background: Rgb565,
}

impl<D> Ui<D>
where
    D: DrawTarget<Color = Rgb565> + OriginDimensions,
{
    /// Creates a new [`Ui`] instance with the given display.
    pub fn new(display: D) -> Self {
        Self { display, cx: Context::default(), background: D::Color::BLACK }
    }

    /// Returns a new [`Ui`] instance with the given background color. This background color will
    /// be used to clear dirty regions before rendering.
    pub fn with_background(mut self, background: Rgb565) -> Self {
        self.background = background;
        self
    }

    /// Renders the given `view` onto the display.
    pub fn render<V>(&mut self, view: &V) -> Result<(), D::Error>
    where
        V: Render,
    {
        let root = view.render(&mut self.cx).into_element();

        let viewport = Constraints::exact(self.display.size());

        // Resolve the root node and build the tree.
        let mut tree = FrameTree::new();
        let _ = tree.resolve(root, viewport);

        self.display.clear(self.background)?;

        tree.draw(&mut self.display)
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
}

type NodeIndex = usize;

/// Operations, single pass:
/// - Recursively walk the element tree to build the frame tree. For each element,
/// calculate hard constraints (inner_constraints) and push them down to the children.
/// - Once the leafs are resolved, push sizes back up the tree. For container elements,
/// also calculate relative offsets for each child.
struct FrameTree<'a> {
    nodes: Vec<Node<'a>>,
    root: Option<NodeIndex>,
}

impl<'a> FrameTree<'a> {
    /// Creates a new, empty frame tree.
    fn new() -> Self {
        Self { nodes: Vec::new(), root: None }
    }

    /// Inserts a new node into the frame tree and returns its index.
    fn insert(&mut self, node: Node<'a>) -> NodeIndex {
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
    pub(crate) fn resolve(&mut self, root: Element<'a>, constraints: Constraints) -> NodeIndex {
        let root = resolve(self, root, constraints, None);
        self.root = Some(root);
        root
    }

    /// Draws the frame tree onto the given display, starting with `root` at `offset`.
    pub(crate) fn draw<D>(&mut self, display: &mut D) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        let Some(root) = self.root else {
            return Ok(());
        };

        self.draw_node(root, Point::zero(), display)
    }

    fn draw_node<D>(
        &mut self,
        index: NodeIndex,
        parent_origin: Point,
        display: &mut D,
    ) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        let node = &mut self.nodes[index];
        let layout = node.layout.resolve(parent_origin);

        node.element.draw(&layout, display)?;

        let mut child = node.child;

        while let Some(index) = child {
            self.draw_node(index, layout.content.top_left, display)?;
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
        // borrow of `self.nodes[index]` while layout_row mutates children.
        enum ContentLayout {
            Row,
            // A leaf with an inherent size.
            Leaf(Size),
        }

        let content_layout = match &self.nodes[index].element {
            Element::Row(_) => ContentLayout::Row,
            Element::Text(text) => {
                ContentLayout::Leaf(content_constraints.constrain(text.measure()))
            }
            Element::Custom(never) => match *never {},
        };

        let intrinsic_content_size = match content_layout {
            ContentLayout::Row => self.layout_row(index, content_constraints),

            ContentLayout::Leaf(size) => size,
        };

        let content_inset = box_style.content_inset();
        let border_size = {
            // The "natural" border size, i.e. derived from its children.
            let border_size = intrinsic_content_size.inflate(content_inset.saturating_mul(2));

            // Adopt the configured border size if set.
            let border_size = box_style.size.unwrap_or(border_size);

            border_constraints.constrain(border_size)
        };

        let outer_size = border_size.inflate(box_style.margin.saturating_mul(2));

        let content_size = border_size.deflate(content_inset.saturating_mul(2));

        let margin_offset = i32::try_from(box_style.margin).unwrap_or(i32::MAX);

        let content_offset =
            box_style.margin.saturating_add(box_style.border).saturating_add(box_style.padding);

        let content_offset = i32::try_from(content_offset).unwrap_or(i32::MAX);

        self.nodes[index].layout = Layout {
            // The parent assigns this later. The root remains at zero.
            offset: Point::new(0, 0),

            // Both offsets are relative to the node's outer-box origin.
            border_offset: Point::new(margin_offset, margin_offset),

            content_offset: Point::new(content_offset, content_offset),

            outer_size,
            border_size,
            content_size,

            // Absolute bounds are resolved while traversing for paint
            // or hit testing.
            bounds: None,
        };
    }

    /// Lays out the row at `index`:
    /// - Traverses the children and sets their offsets.
    /// - Records intrinsic width and height, and returns the constrained size.
    fn layout_row(&mut self, index: NodeIndex, constraints: Constraints) -> Size {
        let mut width: u32 = 0;
        let mut height: u32 = 0;

        // Get the optional child.
        let mut child = self.nodes[index].child;

        while let Some(child_idx) = child {
            let size = self.nodes[child_idx].layout.outer_size;

            let x = i32::try_from(width).unwrap_or(i32::MAX);

            // TODO: Only allows for top alignment, and no gap.
            self.nodes[child_idx].layout.set_offset(Point::new(x, 0));

            width = width.saturating_add(size.width);

            height = height.max(size.height);

            child = self.nodes[child_idx].sibling;
        }

        constraints.constrain(Size::new(width, height))
    }
}

trait SizeExt {
    /// Uniformly inflates the size with `by`.
    fn inflate(self, by: u32) -> Self;

    /// Uniformly deflates the size with `by`.
    fn deflate(self, by: u32) -> Self;
}

impl SizeExt for Size {
    fn inflate(self, by: u32) -> Self {
        Size::new(self.width.saturating_add(by), self.height.saturating_add(by))
    }

    fn deflate(self, by: u32) -> Self {
        self.saturating_sub(Size::new_equal(by))
    }
}

/// Recursively populates the frame tree with the given element and parent node.
fn resolve<'a>(
    tree: &mut FrameTree<'a>,
    mut element: Element<'a>,
    constraints: Constraints,
    parent: Option<NodeIndex>,
) -> NodeIndex {
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
struct Node<'a> {
    /// The node's element.
    element: Element<'a>,
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
pub trait Render {
    /// Renders this element into an [`Element`] using the given [`Context`].
    fn render<'a>(&'a self, cx: &mut Context) -> impl IntoElement<Element = Element<'a>>;
}

/// For now, unused. In the future, will be used for context management, such as:
/// - Allocating and managing retained resources
/// - Interactivity (from UI upstream)
#[derive(Default)]
pub struct Context {}

#[cfg(test)]
mod tests {
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
}
