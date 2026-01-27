# Compile command:
``` bash
cargo +nightly build -Z build-std=std,core,alloc,panic_abort -Z build-std-features="optimize_for_size" --release
```
[read about specified compiler flags](https://github.com/johnthagen/min-sized-rust)

# Cargo.toml profile:
``` toml
[profile.release]
opt-level = "z"
lto = true
codegen-units = 1
strip = true
```
