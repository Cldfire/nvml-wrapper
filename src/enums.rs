use super::ffi::*;

/// Respresents possible variants for a firmware version.
#[derive(Debug)]
pub enum FirmwareVersion {
    /// The version is unavailable.
    Unavailable,
    Version(u32),
}

impl From<u32> for FirmwareVersion {
    fn from(value: u32) -> Self {
        match value {
            0 => FirmwareVersion::Unavailable,
            _ => FirmwareVersion::Version(value),
        }
    }
}

/// Represents possible variants for used GPU memory.
#[derive(Debug)]
pub enum UsedGpuMemory {
    /// Under WDDM, `NVML_VALUE_NOT_AVAILABLE` is always reported because Windows KMD
    /// manages all the memory, not the NVIDIA driver.
    Unavailable,
    /// Memory used in bytes.
    Used(u64),
}

impl From<u64> for UsedGpuMemory {
    fn from(value: u64) -> Self {
        let not_available = (NVML_VALUE_NOT_AVAILABLE) as u64;

        match value {
            // Believe it or not, it took me half an hour to figure out how to do this.
            // Wow. Maybe write a blog post about that? ¯\_(ツ)_/¯
            _ if value == not_available => UsedGpuMemory::Unavailable,
            _ => UsedGpuMemory::Used(value),
        }
    }
}