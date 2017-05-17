pub use self::bindings::*;

#[cfg(target_os = "linux")]
#[path = "linux.rs"]
mod bindings;