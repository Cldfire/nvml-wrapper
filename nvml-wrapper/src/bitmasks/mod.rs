pub mod device;
pub mod event;
pub mod nv_link;

use crate::ffi::bindings::*;
use bitflags::bitflags;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

bitflags! {
    /// Generic flags used to specify the default behavior of some functions.
    // Checked against local
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub struct Behavior: u32 {
        const DEFAULT = nvmlFlagDefault;
        const FORCE   = nvmlFlagForce;
    }
}

bitflags! {
    /// Flags that can be passed to `NVML::init_with_flags()`.
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[derive(Default)]
    pub struct InitFlags: u32 {
        /// Don't fail to initialize when no NVIDIA GPUs are found.
        const NO_GPUS = NVML_INIT_FLAG_NO_GPUS;
        /// Don't attach GPUs during initialization.
        const NO_ATTACH = NVML_INIT_FLAG_NO_ATTACH;
    }
}
