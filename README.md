# Guillotine

A `no-std` graphical user interface framework for embedded devices prioritizing resource efficiency and ergonomics. The UI declaration API is heavily inspired by [GPUI](https://github.com/zed-industries/zed/tree/main/crates/gpui).

Works everywhere `mipidsi` and `embedded-graphics` work. 

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

```rs
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

## Roadmap

### v0.0.1
- [x] Try to mimic GPUI declaration style: https://github.com/zed-industries/zed/blob/main/crates/gpui/examples/hello_world.rs
- [x] Low-level `Element` / `ParentElement` trait for custom elements and widgets
- [x] Support generic `PixelColor`
- [ ] Full immediate mode redrawing
  - [ ] Clipping / overflow behaviour
- [ ] Make repo ready for publishing:
  - [ ] README documentation (a la Dioxus)
  - [ ] Rustdoc documentation
  - [ ] Examples
    - Sizing (insets)
    - Fonts
    - Cool
  - [ ] prelude for exporting
  - [ ] Dual Apache / MIT license
  - [ ] Fix sdl2 vendoring for embedded-graphics-simulator
- [ ] Benchmarks for Frame building
- [ ] `defmt` feature
- [ ] Profiling feature that prints render times with `defmt`
- [x] TextStyle fonts
- [ ] Support for non-interactive elements:
  - [x] Row
  - [x] Text
  - [ ] Column
  - [ ] Spinner

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
- [ ] Dialogs & modals
- [ ] No alloc
- [ ] Custom elements
- [ ] Alignment
- [ ] Support for interaction
- [ ] Support interactive elements:
  - [ ] Button
  - [ ] Slider

## Prior Work & Inspiration
- [Clay by Nic Barker](https://github.com/nicbarker/clay#retained-mode-rendering)
- [Kolibri by Yandrik](https://github.com/Yandrik/kolibri)
- [GPUI by Zed](https://github.com/zed-industries/zed/tree/main/crates/gpui)
