extern crate bindgen;
extern crate pkg_config;

use std::env;
use std::path::PathBuf;

// TODO: Clean this up.

fn main() {
    match pkg_config::Config::new().atleast_version("8.0").probe("nvml-8.0") {
        Ok(info) => {
            if info.include_paths.len() == 1 {
                // println!("cargo:warning={:?}", info.include_paths[0].to_str().unwrap());
                let bindings = bindgen::Builder::default()
                    .no_unstable_rust()
                    // Doesn't work until bindgen processes doc comments
                    .generate_comments(false)
                    .derive_default(true)
                    .header(format!("{}{}", info.include_paths[0].to_str().unwrap(), "/nvml.h"))
                    .generate()
                    .expect("Unable to generate bindings");

                let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
                bindings.write_to_file(out_path.join("bindings.rs")).expect("Couldn't write bindings!");
            } else {
                println!("cargo:warning=Include paths != 1");
            }
        },
        Err(err) => println!("{:?}", err)
    }
}

