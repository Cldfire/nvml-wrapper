use super::ffi::*;
use super::nvml_errors::*;

// TODO: Test everything in this module.
// TODO: Check all of these things against local nvml.h
// TODO: Improve the derive macro
// TODO: Should platform-specific things be in their own modules?

/// API types that allow changes to default permission restrictions.
#[derive(EnumWrapper, Debug)]
#[wrap(c_enum = "nvmlRestrictedAPI_t")]
#[wrap(has_count = "NVML_RESTRICTED_API_COUNT")]
pub enum Api {
    /// APIs that change application clocks.
    ///
    /// Applicable methods on `Device`: `.set_applications_clocks()`, 
    /// `.reset_applications_clocks()`
    // TODO: Come back and make sure these names are right when I actually write them. And below
    #[wrap(c_variant = "NVML_RESTRICTED_API_SET_APPLICATION_CLOCKS")]
    ApplicationClocks,
    /// APIs that enable/disable auto boosted clocks.
    ///
    /// Applicable methods on `Device`: `.set_auto_boosted_clocks_enabled()`
    #[wrap(c_variant = "NVML_RESTRICTED_API_SET_AUTO_BOOSTED_CLOCKS")]
    AutoBoostedClocks,
}

/// Clock types. All speeds are in Mhz. 
// impl and enum checked against local nvml.h
#[derive(EnumWrapper, Debug)]
#[wrap(c_enum = "nvmlClockType_t")]
#[wrap(has_count = "NVML_CLOCK_COUNT")]
pub enum Clock {
    /// Graphics clock domain.
    #[wrap(c_variant = "NVML_CLOCK_GRAPHICS")]
    Graphics,
    /// SM clock domain.
    // TODO: Improve that ^
    #[wrap(c_variant = "NVML_CLOCK_SM")]
    SM,
    /// Memory clock domain.
    #[wrap(c_variant = "NVML_CLOCK_MEM")]
    Memory,
    /// Video encoder/decoder clock domain.
    #[wrap(c_variant = "NVML_CLOCK_VIDEO")]
    Video,
}

/// GPU brand.
#[derive(EnumWrapper, Debug)]
#[wrap(c_enum = "nvmlBrandType_t")]
#[wrap(has_count = "NVML_BRAND_COUNT")]
pub enum Brand {
    #[wrap(c_variant = "NVML_BRAND_UNKNOWN")]
    Unknown,
    /// Targeted at workstations.
    #[wrap(c_variant = "NVML_BRAND_QUADRO")]
    Quadro,
    /// Targeted at high-end compute.
    #[wrap(c_variant = "NVML_BRAND_TESLA")]
    Tesla,
    /// NVIDIA's multi-display cards.
    #[wrap(c_variant = "NVML_BRAND_NVS")]
    NVS,
    /// vGPUs
    #[wrap(c_variant = "NVML_BRAND_GRID")]
    GRID,
    /// Targeted at gaming.
    #[wrap(c_variant = "NVML_BRAND_GEFORCE")]
    GeForce,
}

/// Represents type of a bridge chip.
///
/// NVIDIA does not provide docs (in the code, that is) explaining what each chip
/// type is, so you're on your own there.
#[derive(EnumWrapper, Debug)]
#[wrap(c_enum = "nvmlBridgeChipType_t")]
pub enum BridgeChip {
    #[wrap(c_variant = "NVML_BRIDGE_CHIP_PLX")]
    PLX,
    #[wrap(c_variant = "NVML_BRIDGE_CHIP_BRO4")]
    BRO4,
}

/// Memory error types.
#[derive(EnumWrapper, Debug)]
#[wrap(c_enum = "nvmlMemoryErrorType_t")]
#[wrap(has_count = "NVML_MEMORY_ERROR_TYPE_COUNT")]
pub enum MemoryError {
    /// A memory error that was corrected for ECC errors.
    ///
    /// These are single bit errors for texture memory and are fixed by a resend.
    #[wrap(c_variant = "NVML_MEMORY_ERROR_TYPE_CORRECTED")]
    Corrected,
    /// A memory error that was not corrected for ECC errors.
    ///
    /// These are double bit errors for texture memory where the resend failed.
    #[wrap(c_variant = "NVML_MEMORY_ERROR_TYPE_UNCORRECTED")]
    Uncorrected,
}

/// ECC counter types.
///
/// Note: Volatile counts are reset each time the driver loads. On Windows this is
/// once per boot. On Linux this can be more frequent; the driver unloads when no
/// active clients exist. If persistence mode is enabled or there is always a
/// driver client active (such as X11), then Linux also sees per-boot behavior.
/// If not, volatile counts are reset each time a compute app is run.
#[derive(EnumWrapper, Debug)]
#[wrap(c_enum = "nvmlEccCounterType_t")]
#[wrap(has_count = "NVML_ECC_COUNTER_TYPE_COUNT")]
pub enum EccCounter {
    /// Volatile counts are reset each time the driver loads.
    #[wrap(c_variant = "NVML_VOLATILE_ECC")]
    Volatile,
    /// Aggregate counts persist across reboots (i.e. for the lifetime of the device).
    #[wrap(c_variant = "NVML_AGGREGATE_ECC")]
    Aggregate,
}

/// Driver models, Windows only.
#[derive(EnumWrapper, Debug)]
#[wrap(c_enum = "nvmlDriverModel_t")]
#[cfg(target_os = "windows")]
pub enum DriverModel {
    /// GPU treated as a display device.
    #[wrap(c_variant = "NVML_DRIVER_WDDM")]
    WDDM,
    /// (TCC model) GPU treated as a generic device (recommended).
    #[wrap(c_variant = "NVML_DRIVER_WDM")]
    WDM,
}

/// GPU operation mode.
///
/// Allows for the reduction of power usage and optimization of GPU throughput
/// by disabling GPU features.
#[derive(EnumWrapper, Debug)]
#[wrap(c_enum = "nvmlGpuOperationMode_t")]
pub enum OperationMode {
    #[wrap(c_variant = "NVML_GOM_ALL_ON")]
    AllOn,
    #[wrap(c_variant = "NVML_GOM_COMPUTE")]
    Compute,
    #[wrap(c_variant = "NVML_GOM_LOW_DP")]
    LowDP,
}

pub fn bool_from_state(state: nvmlEnableState_t) -> bool {
    match state {
        nvmlEnableState_t::NVML_FEATURE_DISABLED => false,
        nvmlEnableState_t::NVML_FEATURE_ENABLED => true,
    }
}