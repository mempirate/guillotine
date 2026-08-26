## Examples

To run these examples, enable the `simulator` feature. This pulls in a bundled
[SDL2](https://github.com/Rust-SDL2/rust-sdl2) for opening windows. You will need CMake to compile
it.

```sh
cargo run -p guillotine --example basic --features simulator,flexbox

cargo run -p guillotine --example power_monitor --features simulator,flexbox

# Draw an application-defined battery widget through the custom element API.
cargo run -p guillotine --example custom_element --features simulator

# Showcase the supported flexbox alignment modes.
cargo run -p guillotine --example flexbox --features simulator,flexbox
```
