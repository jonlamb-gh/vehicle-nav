#![deny(warnings)]

use std::env;
use std::path::PathBuf;

fn main() {
    let raylib_config = pkg_config::Config::new()
        .atleast_version("3.5.0")
        .probe("raylib")
        .unwrap();

    println!("cargo:rerun-if-changed=include/wrapper.h");

    let bindings = bindgen::Builder::default()
        .header("include/wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .clang_args(
            raylib_config
                .include_paths
                .iter()
                .map(|p| format!("-I{}", p.display())),
        )
        .rustified_enum(".+") // NOTE: this could generate UB, should be specific
        .blocklist_item("PI")
        .blocklist_item("FP_NAN")
        .blocklist_item("FP_INFINITE")
        .blocklist_item("FP_ZERO")
        .blocklist_item("FP_SUBNORMAL")
        .blocklist_item("FP_NORMAL")
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
