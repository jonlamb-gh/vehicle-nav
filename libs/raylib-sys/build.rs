#![deny(warnings)]

use std::env;
use std::path::PathBuf;

fn main() {
    // TODO for now, add env PKG_CONFIG_PATH=/tmp/raylib_install/lib/pkgconfig/
    // ultimately this will be setup to work in the bitbake cross compilation build env
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
        .blocklist_item("PI")
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
