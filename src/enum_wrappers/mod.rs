use ffi::*;

pub mod nv_link;
pub mod device;
pub mod unit;

pub fn bool_from_state(state: nvmlEnableState_t) -> bool {
    match state {
        nvmlEnableState_t::NVML_FEATURE_DISABLED => false,
        nvmlEnableState_t::NVML_FEATURE_ENABLED => true,
    }
}

pub fn state_from_bool(bool_: bool) -> nvmlEnableState_t {
    match bool_ {
        false => nvmlEnableState_t::NVML_FEATURE_DISABLED,
        true => nvmlEnableState_t::NVML_FEATURE_ENABLED,
    }
}