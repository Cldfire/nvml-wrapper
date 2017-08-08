use enum_wrappers::device::SampleValueType;
use ffi::bindings::*;

/// Respresents possible variants for a firmware version.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum FirmwareVersion {
    /// The version is unavailable.
    Unavailable,
    Version(u32)
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
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum UsedGpuMemory {
    /// Under WDDM, `NVML_VALUE_NOT_AVAILABLE` is always reported because
    /// Windows KMD manages all the memory, not the NVIDIA driver.
    Unavailable,
    /// Memory used in bytes.
    Used(u64)
}

impl From<u64> for UsedGpuMemory {
    fn from(value: u64) -> Self {
        let not_available = (NVML_VALUE_NOT_AVAILABLE) as u64;

        match value {
            v if v == not_available => UsedGpuMemory::Unavailable,
            _ => UsedGpuMemory::Used(value),
        }
    }
}

/// Represents different types of sample values.
// Checked against local
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum SampleValue {
    F64(f64),
    U32(u32),
    U64(u64)
}

impl SampleValue {
    pub fn from_tag_and_union(tag: &SampleValueType, union: nvmlValue_t) -> Self {
        use self::SampleValueType::*;

        unsafe {
            match *tag {
                Double => SampleValue::F64(union.dVal),
                UnsignedInt => SampleValue::U32(union.uiVal),
                // Methodology: NVML supports 32-bit Linux. UL is u32 on that platform.
                // NVML wouldn't return anything larger
                UnsignedLong => SampleValue::U32(union.ulVal as u32),
                UnsignedLongLong => SampleValue::U64(union.ullVal),
            }
        }
    }
}
