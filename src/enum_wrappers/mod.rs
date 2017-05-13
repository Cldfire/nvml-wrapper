use ffi::bindings::*;

pub mod nv_link;
pub mod device;
pub mod unit;

pub fn bool_from_state(state: nvmlEnableState_t) -> bool {
    match state {
        nvmlEnableState_t::NVML_FEATURE_DISABLED => false,
        nvmlEnableState_t::NVML_FEATURE_ENABLED => true,
    }
}

pub fn state_from_bool(enabled: bool) -> nvmlEnableState_t {
    if enabled {
        nvmlEnableState_t::NVML_FEATURE_ENABLED
    } else {
        nvmlEnableState_t::NVML_FEATURE_DISABLED
    }
}