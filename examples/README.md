## Examples

To run the [examples](./examples), you need to enable the `simulator` feature. This pulls in a bundled [sdl2](https://github.com/Rust-SDL2/rust-sdl2) for opening windows. You will need cmake to compile it.

```sh
cargo run --example power_monitor --features simulator,flexbox

# Showcase the supported flexbox alignment modes.
cargo run --example flexbox --features simulator,flexbox
```
