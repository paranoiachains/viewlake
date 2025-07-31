fn main() {
    println!("cargo:rustc-link-search=native=/home/keira/rust/viewlake/build-win/library");

    println!("cargo:rustc-link-lib=static=mbedtls");
    println!("cargo:rustc-link-lib=static=mbedx509");
    println!("cargo:rustc-link-lib=static=tfpsacrypto");
}
