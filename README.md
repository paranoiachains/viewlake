Compile command:
``` bash
$env:RUSTFLAGS="-Zunstable-options -Cpanic=immediate-abort -Zlocation-detail=none -Zfmt-debug=none"; cargo +nightly build -Z build-std=std,core,alloc,panic_abort -Z build-std-features="optimize_for_size,panic_immediate_abort" --target x86_64-pc-windows-msvc --release
```

Tests:
``` bash
$env:RUSTFLAGS=""; cargo t
```

Cargo.toml profile:
``` toml
[profile.release]
opt-level = "z"
lto = true
codegen-units = 1
panic = "abort"
strip = true
```
TBA (i can't remember all commands/flags/configs needed for min sized binary)

