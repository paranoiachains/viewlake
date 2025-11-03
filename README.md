# Compile command:
``` bash
$env:RUSTFLAGS="-Zunstable-options -Cpanic=immediate-abort -Zlocation-detail=none -Zfmt-debug=none"; cargo +nightly build -Z build-std=std,core,alloc -Z build-std-features="optimize_for_size,panic_immediate_abort" --target x86_64-pc-windows-msvc --release
```

---

- `-Zunstable-options -Cpanic=immediate-abort` excludes panic strings and formatting code from final binary.
- `-Zlocation-detail=none` removes information for `panic!()` and `[track_caller]` excludes panic strings and formatting code from final binary.
- `-Zfmt-debug=none` turns `#[derive(Debug)]` and `{:?}` formatting into no-ops (ruins output of `dbg!()`, `assert!()`).
- `-Z build-std=...` builds rust's `libstd` from source, providing such features as optimizing build for speed.

# Cargo.toml profile:
``` toml
[profile.release]
opt-level = "z"
lto = true
codegen-units = 1
strip = true
```
TBA (i can't remember all commands/flags/configs needed for min sized binary)

