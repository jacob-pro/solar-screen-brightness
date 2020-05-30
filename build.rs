use cmake::Config;
use std::env;
use std::path::PathBuf;

fn main() {
    // Builds the project in the directory located in `sunrise-sunset-calculator`, installing it into $OUT_DIR
    let dst = Config::new("sunrise-sunset-calculator")
        .build_target("ssc")
        .build();

    println!("cargo:rustc-link-search=native={}", format!("{}/build/lib", dst.display()));
    println!("cargo:rustc-link-lib=static=ssc");


    let bindings = bindgen::Builder::default()
        .header("sunrise-sunset-calculator/brightness.h")
        .header("sunrise-sunset-calculator/adapter.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings.write_to_file(out_path.join("ssc.rs"))
        .expect("Couldn't write bindings!");

}
