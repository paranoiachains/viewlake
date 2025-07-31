fn main() {
    println!(r"cargo:rustc-link-search=native=C:\Users\admin\rust\viewlake\build-win");

    println!("cargo:rustc-link-lib=static=mbedtls");
    println!("cargo:rustc-link-lib=static=mbedx509");
    println!("cargo:rustc-link-lib=static=tfpsacrypto");
}
