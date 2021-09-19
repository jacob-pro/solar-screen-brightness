use std::path::PathBuf;

fn main() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    if cfg!(windows) {
        let mut res = winres::WindowsResource::new();
        res.set_icon("../assets/icon-256.ico");
        res.compile().unwrap();
    }
    bearlib(manifest_dir.join("../lib"));
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
fn bearlib(path: PathBuf) {
    let path = path.join("linux_x64");
    println!("cargo:rustc-env=LD_LIBRARY_PATH={}", path.to_str().unwrap());
    println!("cargo:rustc-flags=-L {}", path.to_str().unwrap());
}

#[cfg(target_os = "macos")]
fn bearlib(path: PathBuf) {
    let path = path.join("macos");
    println!("cargo:rustc-env=LD_LIBRARY_PATH={}", path.to_str().unwrap());
    println!("cargo:rustc-flags=-L {}", path.to_str().unwrap());
}

#[cfg(target_os = "windows")]
fn bearlib(_path: PathBuf) {}
