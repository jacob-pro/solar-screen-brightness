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
    bearlib();
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
    std::fs::create_dir_all(&dest).unwrap_or_else(|_| panic!("Failed to create {:?}", dest));
    std::fs::copy(exe, dest.join("solar-screen-brightness")).expect("Failed to copy binary");
}

#[cfg(unix)]
fn bearlib() {
    use cmake::Config;
    let dst = Config::new("../bearlibterminal/Terminal")
        .define("BUILD_SHARED_LIBS", "OFF")
        .build();
    println!("cargo:rustc-link-search={}", dst.join("lib").display());
    println!("cargo:rustc-link-arg=-lstdc++");
    println!("cargo:rustc-link-arg=-lGL");
    println!("cargo:rustc-link-arg=-lX11");

    let deps = dst.join("build").join("Dependencies");

    let freetype2 = deps.join("FreeType").join("libfreetype2.a");
    assert!(freetype2.is_file());
    println!("cargo:rustc-link-arg={}", freetype2.display());

    let picopng = deps.join("PicoPNG").join("libpicopng.a");
    assert!(picopng.is_file());
    println!("cargo:rustc-link-arg={}", picopng.display());
}

#[cfg(windows)]
fn bearlib() {}
