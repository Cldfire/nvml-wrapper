extern crate pkg_config;

#[cfg(target_os = "windows")]
fn main() {
    println!("cargo:rustc-link-lib=dylib=nvml");
    println!("cargo:rustc-link-search=C:\\Program Files\\NVIDIA Corporation\\NVSMI");
}

#[cfg(target_os = "linux")]
fn main() {

}