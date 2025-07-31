fn main() {
    println!("cargo:rustc-link-search=native=/home/keira/rust/mbedtls/build-win/library");

    println!("cargo:rustc-link-lib=static=mbedtls");
    println!("cargo:rustc-link-lib=static=mbedx509");
    println!("cargo:rustc-link-lib=static=tfpsacrypto");

    println!("cargo:rustc-link-lib=dylib=crypto"); // example
    println!("cargo:rustc-link-lib=dylib=pthread"); // example
}
