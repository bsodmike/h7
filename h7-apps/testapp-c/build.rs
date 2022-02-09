fn main() {
    println!("cargo:rerun-if-changed=src/*");
    cc::Build::new()
        .cpp(false)
        // TODO: Find file automatically
        .files(&["src/main.c"])
        .compile(concat!("lib", env!("CARGO_PKG_NAME")));
}
