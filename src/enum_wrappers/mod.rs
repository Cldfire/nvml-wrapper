use crate::error::{Result, ErrorKind, Error};
use crate::ffi::bindings::*;

pub mod nv_link;
pub mod device;
pub mod unit;

pub fn bool_from_state(state: nvmlEnableState_t) -> Result<bool> {
    match state {
        nvmlEnableState_enum_NVML_FEATURE_DISABLED => Ok(false),
        nvmlEnableState_enum_NVML_FEATURE_ENABLED => Ok(true),
        _ => Err(Error::from_kind(ErrorKind::UnexpectedVariant(state))),
    }
}

pub fn state_from_bool(enabled: bool) -> nvmlEnableState_t {
    if enabled {
        nvmlEnableState_enum_NVML_FEATURE_ENABLED
    } else {
        nvmlEnableState_enum_NVML_FEATURE_DISABLED
    }
}
