// Generate bindings: bindgen --constified-enum "nvml.+" --ctypes-prefix raw
// --no-doc-comments --raw-line "#![allow(non_upper_case_globals)]" --raw-line
// "#![allow(non_camel_case_types)]" --raw-line "#![allow(non_snake_case)]"
// --raw-line "#![allow(dead_code)]"  --raw-line "use std::os::raw;" -o
// genned_bindings.rs nvml.h

fn main() {
    #[cfg(not(feature = "ci"))]
    {
        #[cfg(target_os = "windows")]
        {
            println!("cargo:rustc-link-lib=nvml");
            println!("cargo:rustc-link-search=C:\\Program Files\\NVIDIA Corporation\\NVSMI");
        }

        #[cfg(target_os = "linux")]
        {
            println!("cargo:rustc-link-lib=nvidia-ml");
        }
    }
}