use std::fs;

#[cfg(target_os = "linux")]
fn main() {
    let paths = fs::read_dir("/usr/lib").unwrap();

    for path in paths {
        let entry = path.unwrap().path();
        if entry.is_dir() {
            let entry_string = entry.to_string_lossy();
            if entry_string.contains("nvidia") && !entry_string.ends_with("prime") {
                println!("cargo:rustc-link-search=native={}", entry_string);
                break;
            }
        }
    }
    // println!("cargo:rustc-link-search=native={}", "/usr/lib/nvidia-415");
}

#[cfg(target_os = "windows")]
fn main() {}