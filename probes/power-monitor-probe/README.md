# Power-monitor layout probe

This standalone ESP32-C3 binary copies the declaration from `examples/power_monitor.rs` and
renders one frame into a zero-sized `NoopDisplay`. It does not enable an allocator or retain a
framebuffer.

The complete `Ui` is allocated in the named `POWER_MONITOR_UI` static. This makes its fixed-capacity
`FrameStorage<Rgb565, 64, 1024>` reservation visible in the linked ELF instead of hiding it in an
unknown stack frame.

Run Cargo from this directory so it loads `.cargo/config.toml` and selects
`riscv32imc-unknown-none-elf`:

```sh
cargo check
cargo build --release
cargo size --release -- -A
cargo nm --release -- --print-size | rg POWER_MONITOR_UI
```

The last two commands require `cargo-binutils` and the Rust `llvm-tools` component. CI can run the
underlying `llvm-size` and `llvm-nm` programs directly instead.

## Pull-request measurements

After `.github/workflows/probe-memory.yml` is present on the default branch, a repository owner,
member, or collaborator can comment `/probe` on a pull request. CI builds both the pull-request and
base revisions, posts a byte-level comparison, and retains the JSON, symbol report, and head ELF as
a workflow artifact for 30 days.

This project was initially scaffolded with `esp-generate 1.3.0`:

```sh
esp-generate --headless --chip esp32c3 --skip-update-check \
  --output-path probes power-monitor-probe
```
