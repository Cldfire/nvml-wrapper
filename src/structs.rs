/// Returned from `Device.auto_boosted_clocks_enabled()`
pub struct AutoBoostClocksEnabledInfo {
    /// Current state of auto boosted clocks for the `Device`
    pub is_enabled: bool,
    /// Default auto boosted clocks behavior for the `Device`
    ///
    /// The GPU will revert to this default when no applications are using the GPU.
    pub is_enabled_default: bool,
}