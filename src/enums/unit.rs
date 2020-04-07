use crate::error::{Result, ErrorKind, Error};
use crate::ffi::bindings::*;
use std::ffi::CStr;

/// LED states for an S-class unit.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum LedState {
    /// Indicates good health.
    Green,
    /// Indicates a problem along with the accompanying cause.
    Amber(String)
}

impl LedState {
    /**
    Waiting for `TryFrom` to be stable. In the meantime, we do this.
    
    # Errors
    
    * `Utf8Error`, if the string obtained from the C function is not valid Utf8
    */
    pub fn try_from(struct_: nvmlLedState_t) -> Result<Self> {
        let color = struct_.color;

        match color {
            nvmlLedColor_enum_NVML_LED_COLOR_GREEN => Ok(LedState::Green),
            nvmlLedColor_enum_NVML_LED_COLOR_AMBER => unsafe {
                let cause_raw = CStr::from_ptr(struct_.cause.as_ptr());
                Ok(LedState::Amber(cause_raw.to_str()?.into()))
            },
            _ => Err(Error::from_kind(ErrorKind::UnexpectedVariant(color))),
        }
    }
}

/// The type of temperature reading to take for a `Unit`.
///
/// Available readings depend on the product.
#[repr(u32)]
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum TemperatureReading {
    Intake = 0,
    Exhaust = 1,
    Board = 2
}
