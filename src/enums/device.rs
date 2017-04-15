use ffi::bindings::*;

// TODO: document try_froms

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
// Checked
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
            v if v == not_available => UsedGpuMemory::Unavailable,
            _ => UsedGpuMemory::Used(value),
        }
    }
}

/// Represents different types of sample values.
// Checked against local
#[cfg(feature = "nightly")]
#[derive(Debug)]
pub enum SampleValue {
    F64(f64),
    U32(u32),
    U64(u64),
}

#[cfg(feature = "nightly")]
impl SampleValue {
    pub fn from_tag_and_union(tag: &SampleValueType, union: nvmlValue_t) -> Self {
        use SampleValueType::*;

        unsafe {
            match *tag {
                Double            => SampleValue::F64(union.dVal as f64),
                UnsignedInt       => SampleValue::U32(union.uiVal as u32),
                // TODO: Is it okay to map ul => u32
                UnsignedLong      => SampleValue::U32(union.ulVal as u32),
                UnsignedLongLong  => SampleValue::U64(union.ullVal as u64),
            }
        }
    }
}
