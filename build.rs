extern crate bindgen;

use bindgen::EnumVariation;
use std::env;
use std::path::PathBuf;
use EnumVariation::NewType;

extern crate simple_logger;

fn main() {
    // Tell cargo to invalidate the built crate whenever the wrapper changes
    println!("cargo:rerun-if-changed=rtconfig.h");

    simple_logger::init().unwrap();

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        .use_core()
        // The input header we would like to generate
        // bindings for.
        .header("rt-thread/include/rtthread.h")
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .default_enum_style(NewType { is_bitfield: false })
        .ctypes_prefix("cty")
        .detect_include_paths(false)
        .clang_args(vec![
            "-ffreestanding",
            "-target",
            "thumbv7em-none-eabihf",
            "-isystem",
            "./",
            "-isystem",
            "rt-thread/include/",
            "-isystem",
            "rt-thread/components/finsh",
            "-isystem",
            "rt-thread/components/libc/compilers/minilibc",
            "-DRT_USING_MINILIBC",
        ])
        // .clang_args(include_args.iter())
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
