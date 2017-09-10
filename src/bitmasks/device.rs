#![allow(deprecated)]

use ffi::bindings::*;

bitflags! {
    /// Flags used to specify why a GPU is throttling.
    // Checked against local
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub struct ThrottleReasons: u64 {
        /// Nothing is running on the GPU.
        ///
        /// This limiter may be removed in a future release.
        const GPU_IDLE                    = nvmlClocksThrottleReasonGpuIdle as u64;
        /// GPU clocks are limited by the current applications clocks setting.
        const APPLICATIONS_CLOCKS_SETTING = nvmlClocksThrottleReasonApplicationsClocksSetting as u64;
        /// **This flag is deprecated.** It has been renamed to `APPLICATIONS_CLOCKS_SETTING`.
        // TODO: Use the #[deprecated] attribute again when the fix gets released
        const USER_DEFINED_CLOCKS         = nvmlClocksThrottleReasonUserDefinedClocks as u64;
        /// Software power scaling algorithm is reducing clocks.
        const SW_POWER_CAP                = nvmlClocksThrottleReasonSwPowerCap as u64;
        /**
        Hardware slowdown (reducing the core clocks by a factor of 2 or more)
        is engaged.
        
        This is an indicator of one of the following:
        * temperature being too high
        * External Power Brake Asseration is triggered (e.g. by the system power supply)
        * Power draw is too high and Fast Trigger protection is reducing the clocks
        * May also be reported during PState or clock change
            * This behavior may be removed in a later release.
        */
        const HW_SLOWDOWN                 = nvmlClocksThrottleReasonHwSlowdown as u64;
        /**
        This GPU is being throttled by another GPU in its sync boost group.
        
        Sync boost groups can be used to maximize performance per watt. All GPUs
        in a sync boost group will boost to the minimum possible clocks across
        the entire group. Look at the throttle reasons for other GPUs in the
        system to find out why this GPU is being held at lower clocks.
        */
        const SYNC_BOOST                  = nvmlClocksThrottleReasonSyncBoost as u64;
        /// Some other unspecified factor is reducing the clocks.
        const UNKNOWN                     = nvmlClocksThrottleReasonUnknown as u64;
        /// Clocks are as high as possible and are not being throttled.
        const NONE                        = nvmlClocksThrottleReasonNone as u64;
    }
}
