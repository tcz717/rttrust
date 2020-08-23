extern crate bindgen;

use bindgen::EnumVariation;
use std::env;
use std::path::{PathBuf};
use EnumVariation::NewType;

extern crate simple_logger;

fn main() {
    simple_logger::init_by_env();

    let target = env::var("TARGET").unwrap();
    let config_dir = env::var("RT_CONFIG_DIR").unwrap_or("./".to_owned());
    let rt_thread_root = PathBuf::from(env::var_os("RTT_ROOT").unwrap_or("rt-thread".into()));

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
            &target,
            "-isystem",
            &config_dir,
            "-isystem",
            rt_thread_root.join("include").to_str().unwrap(),
            "-isystem",
            rt_thread_root.join("components/finsh").to_str().unwrap(),
            "-isystem",
            rt_thread_root
                .join("components/libc/compilers/minilibc")
                .to_str()
                .unwrap(),
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
