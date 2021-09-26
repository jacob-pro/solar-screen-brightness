fn main() {
    if cfg!(windows) {
        let mut res = winres::WindowsResource::new();
        res.set_icon("../assets/icon-256.ico");
        res.compile().unwrap();
    }
    bearlib();
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
