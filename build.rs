use cmake::Config;

fn main() {
    // Builds the project in the directory located in `sunrise-sunset-calculator`, installing it into $OUT_DIR
    let dst = Config::new("sunrise-sunset-calculator")
        .build_target("ssc")
        .build();

    println!("cargo:rustc-link-search=native={}", format!("{}/build/lib", dst.display()));
    println!("cargo:rustc-link-lib=static=ssc");
}
