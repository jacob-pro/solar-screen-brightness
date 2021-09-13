use std::env::consts::EXE_EXTENSION;
use std::path::PathBuf;
use std::process::Command;

fn main() {
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
