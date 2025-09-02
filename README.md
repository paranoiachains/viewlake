Compile command:
$env:RUSTFLAGS="-C panic=abort -Zlocation-detail=none -Zfmt-debug=none"; cargo +nightly build -Z build-std-features="optimize_for_size,panic_immediate_abort" --target x86_64-pc-windows-msvc --release

Tests:
$env:RUSTFLAGS=""; cargo t


