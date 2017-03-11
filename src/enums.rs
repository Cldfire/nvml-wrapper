use ffi::*;
use nvml_errors::*;
use std::ffi::CStr;

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

/// LED states for an S-class unit.
#[derive(Debug)]
pub enum UnitLedState {
    /// Indicates good health.
    Green,
    /// Indicates a problem along with the accompanying cause
    Amber(String),
}

impl UnitLedState {
    /// Waiting for `TryFrom` to be stable. In the meantime, we do this.
    pub fn try_from(struct_: nvmlLedState_t) -> Result<Self> {
        match struct_.color {
            nvmlLedColor_t::NVML_LED_COLOR_GREEN => Ok(UnitLedState::Green),
            nvmlLedColor_t::NVML_LED_COLOR_AMBER => {
                unsafe {
                    let cause_raw = CStr::from_ptr(struct_.cause.as_ptr());
                    Ok(UnitLedState::Amber(cause_raw.to_str()?.into()))
                }
            },
        }
    }
}

/// THe type of temperature reading to take for a `Unit`.
///
/// Available readings depend on the product.
#[repr(u32)]
#[derive(Debug)]
pub enum UnitTemperatureReading {
    Intake = 0,
    Exhaust = 1,
    Board = 2,
}