use std::{env, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=wrapper.h");

    let header_path = "wrapper.h";
    let bindings = bindgen::Builder::default()
        .header(header_path)
        .clang_arg(r"-IC:\Users\admin\rust\viewlake\build-win\headers")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("unable to gen bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldnt write bindings");

    println!(r"cargo:rustc-link-search=native=C:\Users\admin\rust\viewlake\build-win");

    println!("cargo:rustc-link-lib=static=mbedtls");
    println!("cargo:rustc-link-lib=static=mbedx509");
    println!("cargo:rustc-link-lib=static=tfpsacrypto");
}
