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
        .rustified_enum(".+") // NOTE: this may generate UB, should be specific...
        .blocklist_item("PI")
        .blocklist_item("FP_NAN")
        .blocklist_item("FP_INFINITE")
        .blocklist_item("FP_ZERO")
        .blocklist_item("FP_SUBNORMAL")
        .blocklist_item("FP_NORMAL")
        // TODO - there has to be a better way to blacklist/deal-with long doubles...
        // just not including raymath.h for now
        /*
        .blocklist_item("*scalbl*")
        .blocklist_item("*fmal*")
        .blocklist_item("*fminl*")
        .blocklist_item("*fmaxl*")
        .blocklist_item("*fdiml*")
        .blocklist_item("*roundl*")
        .blocklist_item("*rintl*")
        .blocklist_item("*remquol*")
        .blocklist_item("*truncl*")
        .blocklist_item("*nearbyintl*")
        .blocklist_item("*scalblnl*")
        .blocklist_item("*ilogbl*")
        .blocklist_item("*scalbnl*")
        .blocklist_item("*remainderl*")
        .blocklist_item("*nexttowardl*")
        .blocklist_item("*nextafterl*")
        .blocklist_item("*lgamma*")
        .blocklist_item("*gammal*")
        .blocklist_item("*lgammal_r*")
        .blocklist_item("*erfcl*")
        .blocklist_item("*erfl*")
        .blocklist_item("*ynl*")
        .blocklist_item("*y1l*")
        .blocklist_item("*y0l*")
        .blocklist_item("*jnl*")
        .blocklist_item("*j1l*")
        .blocklist_item("*j0l*")
        .blocklist_item("*isnanl*")
        .blocklist_item("*copysignl*")
        .blocklist_item("*nanl*")
        .blocklist_item("*significandl*")
        .blocklist_item("*dreml*")
        .blocklist_item("*fmodl*")
        .blocklist_item("*finitel*")
        .blocklist_item("*isinfl*")
        .blocklist_item("*floorl*")
        .blocklist_item("*fabsl*")
        .blocklist_item("*ceill*")
        .blocklist_item("*cbrtl*")
        .blocklist_item("*hypotl*")
        .blocklist_item("*sqrtl*")
        .blocklist_item("*powl*")
        .blocklist_item("*log2l*")
        .blocklist_item("*exp2*")
        .blocklist_item("*logbl*")
        .blocklist_item("*log1pl*")
        .blocklist_item("*modfl*")
        .blocklist_item("*exp2l*")
        .blocklist_item("*expm1l*")
        .blocklist_item("*log10l*")
        .blocklist_item("*logl*")
        .blocklist_item("*ldexpl*")
        .blocklist_item("*frexpl*")
        .blocklist_item("*expl*")
        .blocklist_item("*atanhl*")
        .blocklist_item("*asinhl*")
        .blocklist_item("*acoshl*")
        .blocklist_item("*tanhl*")
        .blocklist_item("*sinhl*")
        .blocklist_item("*coshl*")
        .blocklist_item("*tanl*")
        .blocklist_item("*sinl*")
        .blocklist_item("*cosl*")
        .blocklist_item("*atan2l*")
        .blocklist_item("*issignalingl*")
        .blocklist_item("*iseqsigl*")
        .blocklist_item("*signbitl*")
        .blocklist_item("*fpclassify*")
        .blocklist_item("*nexttowardf*")
        .blocklist_item("*fpclassifyl*")
        */
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
