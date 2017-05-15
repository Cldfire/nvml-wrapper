use ffi::bindings::*;

/// Unit fan state.
// Checked against local
#[derive(EnumWrapper, Debug, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[wrap(c_enum = "nvmlFanState_t")]
pub enum FanState {
    /// Working properly
    #[wrap(c_variant = "NVML_FAN_NORMAL")]
    Normal,
    #[wrap(c_variant = "NVML_FAN_FAILED")]
    Failed,
}

// Checked against local
#[derive(EnumWrapper, Debug, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[wrap(c_enum = "nvmlLedColor_t")]
pub enum LedColor {
    /// Used to indicate good health.
    #[wrap(c_variant = "NVML_LED_COLOR_GREEN")]
    Green,
    /// Used to indicate a problem.
    #[wrap(c_variant = "NVML_LED_COLOR_AMBER")]
    Amber,
}