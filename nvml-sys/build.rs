extern crate pkg_config;

fn main() {
    pkg_config::Config::new().atleast_version("8.0")
                             .probe("nvml-8.0")
                             .expect("NVML library >= 8.0 not found");
}
