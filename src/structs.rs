use super::enum_wrappers::*;

/// Returned from `Device.auto_boosted_clocks_enabled()`
#[derive(Debug)]
pub struct AutoBoostClocksEnabledInfo {
    /// Current state of auto boosted clocks for the `Device`
    pub is_enabled: bool,
    /// Default auto boosted clocks behavior for the `Device`
    ///
    /// The GPU will revert to this default when no applications are using the GPU.
    pub is_enabled_default: bool,
}

/// Returned from `Device.decoder_utilization()`
#[derive(Debug)]
pub struct UtilizationInfo {
    pub utilization: u32,
    /// Sampling period in μs.
    pub sampling_period: u32,
}

pub type DecoderUtilizationInfo = UtilizationInfo;
pub type EncoderUtilizationInfo = UtilizationInfo;

/// Returned from `Device.driver_model()`
#[derive(Debug)]
#[cfg(target_os = "windows")]
// TODO: Maybe a better name?
pub struct DriverModels {
    pub current: DriverModel,
    pub pending: DriverModel,
}

/// Returned from `Device.is_ecc_enabled()`
#[derive(Debug)]
pub struct EccModeInfo {
    pub currently_enabled: bool,
    pub pending_enabled: bool,
}

/// Returned from `Device.gpu_operation_mode()`
#[derive(Debug)]
pub struct OperationModeInfo {
    pub current: OperationMode,
    pub pending: OperationMode,
}

/// Returned from `Device.power_management_limit_constraints()`
///
/// Values are in milliwatts.
#[derive(Debug)]
pub struct PowerManagementConstraints {
    pub min_limit: u32,
    pub max_limit: u32,
}