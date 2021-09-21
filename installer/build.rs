use std::env::consts::EXE_EXTENSION;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    compile_and_copy_app();
    if cfg!(windows) {
        let mut res = winres::WindowsResource::new();
        res.set_icon("../assets/icon-256.ico");
        res.compile().unwrap();
    }
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    bearlib(manifest_dir);
}

fn compile_and_copy_app() {
    let profile = std::env::var("PROFILE").unwrap();
    let app_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("app");
    let mut args = vec!["build"];
    if profile == "release" {
        args.push("--release");
    }
    // Due to https://github.com/rust-lang/cargo/issues/6412
    // And https://github.com/rust-lang/cargo/issues/8938
    // This repo is being built as individual crates not in a workspace
    let output = Command::new("cargo")
        .current_dir(&app_dir)
        .args(args)
        .output()
        .expect("Failed to run cargo");
    if !output.status.success() {
        panic!("{}", std::str::from_utf8(&output.stderr).unwrap())
    }
    let exe = app_dir
        .join("target")
        .join(&profile)
        .join("solar-screen-brightness")
        .with_extension(EXE_EXTENSION);
    assert!(exe.is_file());
    println!("cargo:rerun-if-changed={:?}", exe);
    let dest = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("build-assets");
    std::fs::create_dir_all(&dest).expect(&format!("Failed to create {:?}", dest));
    std::fs::copy(exe, dest.join("solar-screen-brightness")).expect("Failed to copy binary");
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
