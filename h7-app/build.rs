use std::{env, fs::File, io::Write, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=h7-app.ld");
    let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let linker_script = include_bytes!("h7-app.ld");
    let mut f = File::create(out.join("h7-app.ld")).unwrap();
    f.write_all(linker_script).unwrap();
    println!("cargo:rustc-link-search={}", out.display());
}
