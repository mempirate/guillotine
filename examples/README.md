## Examples

To run these examples, enable the `simulator` feature. This pulls in a bundled
[SDL2](https://github.com/Rust-SDL2/rust-sdl2) for opening windows. You will need CMake to compile
it.

```sh
cargo run --example basic --features simulator,flexbox

cargo run --example power_monitor --features simulator,flexbox

# Draw an application-defined battery widget through the custom element API.
cargo run --example custom_element --features simulator

# Showcase the supported flexbox alignment modes.
cargo run --example flexbox --features simulator,flexbox
```
