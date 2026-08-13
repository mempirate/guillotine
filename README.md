# Guillotine

A `no-std` graphical user interface framework for embedded devices prioritizing resource efficiency and ergonomics. The UI declaration API is heavily inspired by [GPUI](https://github.com/zed-industries/zed/tree/main/crates/gpui).

Works everywhere `embedded-graphics` works.

## Quickstart

```rust
use guillotine::*; 

struct BasicView {}

impl Render for BasicView {
    // Render lets you declaratively build your UI tree.
    fn render<'a>(&'a self, _cx: &mut Context) -> impl IntoElement<Element = Element<'a>> {
        column()
            .padding(10)
            .margin(10)
            .border(2)
            .border_color(Rgb565::BLUE)
            .child(text("Hello, World!").background(Rgb565::RED).margin(5))
            .child(text("GUILLOTINE").margin(5).font(Font::mono(&FONT_9X18_BOLD)))
    }
}

fn main() {
    // Display should implement embedded_graphics DrawTarget
    let display = ....;

    let view = BasicView {};

    let mut ui = Ui::new(display);

    // Render the view
    ui.render(&view);
}

```

## Status
In progress, working towards an alpha v0.0.1. Check the [roadmap](#roadmap).

## Core Concepts

### Declarative Definition
- Explain the idea of declaratively building your UI

### TODO: Hybrid Immediate & Retained Mode

### TODO: Similar tree-based layout to X
- GPUI

### State Management

### Layout Engine
- Conceptually similar to Flutter (i.e. constraints go down, sizes go up)
  - constraints flow downward, sizes flow upward, positions flow downward
- Requirement: single pass.

## Features
- [x] Declarative, interactive UI building blocks
- [x] Statefulness
- [ ] Custom components & widgets (with custom style)
- [ ] Modals / floating windows
- [ ] Alignments (center, right, bottom, etc)
- [x] Basic foreground/background color themes
- [ ] Support for interaction (touch, hover, click)


## API

```rust
use guillotine::prelude::*;

struct Home {
    show_button: bool,
    header: &'static str,
}

impl Render for Home {
    fn render(&self, _cx: &mut Context<'_>) -> impl IntoElement {
        row()
            .style(Style::default().bg(Color::Red))
            .child(text(self.header).style(Style::default().bold()))
            .when(self.show_button, |row| row.child(button("Click me")))
            .children([text("Copyright"), text("ACME Corp")])
    }
}

struct Page {
    power: f32,
    current: f32,
    voltage: f32,
}

impl Render for Page {
    fn render(&self, _cx: &mut Context<'_>) -> impl IntoElement {
        column()
            .child(text("Power: {}", self.power))
            .child(text("Current: {}", self.current))
            .child(text("Voltage: {}", self.voltage))
    }
}

fn main() {
    let display = SimulatorDisplay::<Rgb565>::new(Size::new(320, 172));

    let mut ui = Ui::new(display);

    let mut home = Home {
        show_button: false,
        header: "Some Title",
    }

    ui.render(&home).unwrap();

    home.show_button = true;

    ui.render(&home).unwrap();

    let mut page = Page { .. };

    ui.render(&page);
}
```

Element trees use the display's `PixelColor` type throughout. `Rgb565` views keep the API shown
above; other targets select their color once at the `Render<Color>` boundary, after which element
constructors and style methods infer it. See the [binary-color example](examples/binary.rs).

### Insets and the box model

Margin, padding, and border widths accept CSS-like physical-edge shorthands:

```rust
column()
    .margin(10)                 // all edges
    .padding((4, 8))            // vertical, horizontal
    .border((1, 2, 3))          // top, horizontal, bottom
    .margin((4, 8, 12, 16));    // top, right, bottom, left
```

Use `Insets::new(top, right, bottom, left)` when a named value is clearer. Insets are non-negative
pixel lengths. Guillotine doesn't currently support percentages, `auto`, logical edges, negative
margins, margin collapsing, per-edge border colors, or border styles. Adjacent margins in rows and
columns add together.

`Style::size` is the border-box size: padding and border are placed inside it, and margin is added
outside it. The box grows to contain its padding and border when parent constraints allow.

### Examples

To run the examples, you need to enable the `simulator` feature. This pulls in a bundled [sdl2](https://github.com/Rust-SDL2/rust-sdl2) for opening windows. You will need cmake to compile it.

```sh
cargo run --example power_monitor --features simulator
```

## Roadmap

### v0.0.1
- [x] Try to mimic GPUI declaration style: <https://github.com/zed-industries/zed/blob/main/crates/gpui/examples/hello_world.rs>
- [x] Low-level `Element` / `ParentElement` trait for custom elements and widgets
- [x] Support generic `PixelColor`
- [x] Full immediate mode redrawing
- [ ] Make repo ready for publishing:
  - [ ] README documentation (a la Dioxus)
  - [ ] Rustdoc documentation
  - [ ] Examples
    - Sizing (insets)
    - Fonts
    - Cool
    - ESP32
  - [x] Fix exports
  - [x] Dual Apache / MIT license
  - [x] Fix sdl2 vendoring for embedded-graphics-simulator
- [ ] Benchmarks for Frame building
- [x] `TextStyle` fonts
- [ ] Support for non-interactive elements:
  - [x] Row
  - [x] (formatted) Text
  - [x] Column
  - [ ] Spinner
- [ ] Container gaps

### v0.1.0
- [ ] Custom render modes: `Incremental` (only repaint changed regions, requires more memory), 
      or `Redraw` (full redraw on every render, lowest memory footprint). Either as a feature
      or runtime flag.
- [ ] Define inremental redrawing triggers / states:
```rs
enum DrawState {
    Clean,
    Paint,     // same geometry; repaint old/current bounds
    Layout,    // size or position changed
    Structure, // child added, removed, moved, or keyed differently
    Full,      // theme, rotation, display reset, etc.
}
```
- [ ] New elements
  - [ ] Dialogs / Modals (floating containers)
  - [ ] Charts
- [ ] Overflow behaviour:
  - [x] Visible
  - [ ] Clip
- [ ] `profile` feature with `defmt` logs
- [ ] No alloc
- [ ] Custom elements
- [ ] Alignment
- [ ] Support for interaction
- [ ] Support interactive elements:
  - [ ] Button
  - [ ] Slider

## Why?

I was trying to build a clean-looking dashboard on a small LCD screen powered by an ESP32-C6, that's supposed to monitor and display the power consumption of my home lab (project [here](https://github.com/mempirate/shellyctl)). I wanted to do this in Rust, with the esp-rs ecosystem. The ecosystem is quite mature, but I couldn't really find a UI framework that was:
1. Performant
2. Very low memory footprint (no Slint / LVGL)
3. Beautiful

Additionally, I wanted to learn what it would take to build something like this.

## Prior Work & Inspiration
- [Clay by Nic Barker](https://github.com/nicbarker/clay#retained-mode-rendering)
- [Kolibri by Yandrik](https://github.com/Yandrik/kolibri)
- [GPUI by Zed](https://github.com/zed-industries/zed/tree/main/crates/gpui)
