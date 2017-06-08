pub mod device;
pub mod nv_link;
pub mod event;

use ffi::bindings::*;

bitflags! {
    /// Generic flags used to specify the default behavior of some functions.
    // Checked against local
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub struct Behavior: u32 {
        const DEFAULT = nvmlFlagDefault;
        const FORCE   = nvmlFlagForce;
    }
}
