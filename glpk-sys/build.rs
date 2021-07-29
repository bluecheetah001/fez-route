extern crate bindgen;

use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    // copy the pre-built glpk binary to the output
    let out_path = env::var("OUT_DIR").unwrap();
    for ext in &["def", "dll", "exp", "lib", "pdb"] {
        fs::copy(
            format!("glpk/glpk_4_65.{}", ext),
            format!("{}/glpk_4_65.{}", out_path, ext),
        )
        .expect("fs::copy failed");
    }

    // Tell cargo to tell rustc to link the pre-build glpk binary
    println!("cargo:rustc-link-search=native={}", out_path);
    println!("cargo:rustc-link-lib=dylib=glpk_4_65");

    // Tell cargo to invalidate the built crate whenever the wrapper changes
    println!("cargo:rerun-if-changed=glpk.h");

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header("glpk.h")
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    bindings
        .write_to_file(PathBuf::from(out_path).join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
