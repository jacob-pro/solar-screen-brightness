use std::path::PathBuf;

fn main() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("lib");
    println!("cargo:rustc-env=LD_LIBRARY_PATH={}", path.to_str().unwrap());
    println!("cargo:rustc-flags=-L {}", path.to_str().unwrap());
}
