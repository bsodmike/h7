// cbindgen: Rust -> C
// bindgen: C -> Rust

fn main() {
    println!("cargo:rerun-if-changed=src/*");

    let crate_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let cbind_conf = cbindgen::Config::from_file("cbindgen.toml").unwrap();
    cbindgen::Builder::new()
        .with_config(cbind_conf)
        .with_crate(crate_dir)
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file("gen/h7api.h");
}
