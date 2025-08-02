fn main() {
    println!("cargo::rustc-link-search=native=lib");
    println!("cargo::rustc-link-lib=static=bearssl");
    println!("cargo:rerun-if-changed=lib/libbearssl.a");
    println!("cargo:rerun-if-changed=include/bearssl.h");
}
