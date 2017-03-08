use super::ffi::*;
use super::errors::*;

// TODO: Test everything in this module.
// TODO: Check all of these things against local nvml.h
// TODO: Write a derive macro to get rid of all the boilerplate
// TODO: Should platform-specific things be in their own modules?

/// API types that allow changes to default permission restrictions.
#[derive(Debug)]
pub enum Api {
    /// APIs that change application clocks.
    ///
    /// Applicable methods on `Device`: `.set_applications_clocks()`, 
    /// `.reset_applications_clocks()`
    // TODO: Come back and make sure these names are right when I actually write them. And below
    ApplicationClocks,
    /// APIs that enable/disable auto boosted clocks.
    ///
    /// Applicable methods on `Device`: `.set_auto_boosted_clocks_enabled()`
    AutoBoostedClocks,
}

impl Api {
    /// Returns the C enum variant equivalent for the given Rust enum variant. 
    pub fn eq_c_variant(&self) -> nvmlRestrictedAPI_t {
        match *self {
            Api::ApplicationClocks
                => nvmlRestrictedAPI_t::NVML_RESTRICTED_API_SET_APPLICATION_CLOCKS,
            Api::AutoBoostedClocks
                => nvmlRestrictedAPI_t::NVML_RESTRICTED_API_SET_AUTO_BOOSTED_CLOCKS,
        }
    }

    /// Waiting for `TryFrom` to be stable. In the meantime, we do this.
    pub fn try_from(enum_: nvmlRestrictedAPI_t) -> Result<Self> {
        match enum_ {
            nvmlRestrictedAPI_t::NVML_RESTRICTED_API_SET_APPLICATION_CLOCKS
                => Ok(Api::ApplicationClocks),
            nvmlRestrictedAPI_t::NVML_RESTRICTED_API_SET_AUTO_BOOSTED_CLOCKS
                => Ok(Api::AutoBoostedClocks),
            nvmlRestrictedAPI_t::NVML_RESTRICTED_API_COUNT
                => bail!(ErrorKind::UnexpectedVariant),
        }
    }
}

/// Clock types. All speeds are in Mhz. 
// impl and enum checked against local nvml.h
#[derive(Debug)]
pub enum Clock {
    /// Graphics clock domain.
    Graphics,
    /// SM clock domain.
    // TODO: Improve that ^
    SM,
    /// Memory clock domain.
    Memory,
    /// Video encoder/decoder clock domain.
    Video,
}

impl Clock {
    /// Returns the C enum variant equivalent for the given Rust enum variant. 
    pub fn eq_c_variant(&self) -> nvmlClockType_t {
        match *self {
            Clock::Graphics => nvmlClockType_t::NVML_CLOCK_GRAPHICS,
            Clock::SM       => nvmlClockType_t::NVML_CLOCK_SM,
            Clock::Memory   => nvmlClockType_t::NVML_CLOCK_MEM,
            Clock::Video    => nvmlClockType_t::NVML_CLOCK_VIDEO,
        }
    }
    
    /// Waiting for `TryFrom` to be stable. In the meantime, we do this.
    pub fn try_from(enum_: nvmlClockType_t) -> Result<Self> {
        match enum_ {
            nvmlClockType_t::NVML_CLOCK_GRAPHICS => Ok(Clock::Graphics),
            nvmlClockType_t::NVML_CLOCK_SM       => Ok(Clock::SM),
            nvmlClockType_t::NVML_CLOCK_MEM      => Ok(Clock::Memory),
            nvmlClockType_t::NVML_CLOCK_VIDEO    => Ok(Clock::Video),
            nvmlClockType_t::NVML_CLOCK_COUNT    => bail!(ErrorKind::UnexpectedVariant),
        }
    }
}

/// GPU brand.
#[derive(Debug)]
pub enum Brand {
    Unknown,
    /// Targeted at workstations.
    Quadro,
    /// Targeted at high-end compute.
    Tesla,
    /// NVIDIA's multi-display cards.
    NVS,
    /// vGPUs
    GRID,
    /// Targeted at gaming.
    GeForce,
}

impl Brand {
    /// Returns the C enum variant equivalent for the given Rust enum variant.
    pub fn eq_c_variant(&self) -> nvmlBrandType_t {
        match *self {
            Brand::Unknown => nvmlBrandType_t::NVML_BRAND_UNKNOWN,
            Brand::Quadro  => nvmlBrandType_t::NVML_BRAND_QUADRO,
            Brand::Tesla   => nvmlBrandType_t::NVML_BRAND_TESLA,
            Brand::NVS     => nvmlBrandType_t::NVML_BRAND_NVS,
            Brand::GRID    => nvmlBrandType_t::NVML_BRAND_GRID,
            Brand::GeForce => nvmlBrandType_t::NVML_BRAND_GEFORCE,
        }
    }

    /// Waiting for `TryFrom` to be stable. In the meantime, we do this.
    pub fn try_from(enum_: nvmlBrandType_t) -> Result<Self> {
        match enum_ {
            nvmlBrandType_t::NVML_BRAND_UNKNOWN => Ok(Brand::Unknown),
            nvmlBrandType_t::NVML_BRAND_QUADRO  => Ok(Brand::Quadro),
            nvmlBrandType_t::NVML_BRAND_TESLA   => Ok(Brand::Tesla),
            nvmlBrandType_t::NVML_BRAND_NVS     => Ok(Brand::NVS),
            nvmlBrandType_t::NVML_BRAND_GRID    => Ok(Brand::GRID),
            nvmlBrandType_t::NVML_BRAND_GEFORCE => Ok(Brand::GeForce),
            nvmlBrandType_t::NVML_BRAND_COUNT   => bail!(ErrorKind::UnexpectedVariant),
        }
    }
}

/// Represents type of a bridge chip.
///
/// NVIDIA does not provide docs (in the code, that is) explaining what each chip
/// type is, so you're on your own there.
#[derive(Debug)]
pub enum BridgeChip {
    PLX,
    BRO4,
}

impl BridgeChip {
    /// Returns the C enum variant equivalent for the given Rust enum variant.
    pub fn eq_c_variant(&self) -> nvmlBridgeChipType_t {
        match *self {
            BridgeChip::PLX  => nvmlBridgeChipType_t::NVML_BRIDGE_CHIP_PLX,
            BridgeChip::BRO4 => nvmlBridgeChipType_t::NVML_BRIDGE_CHIP_BRO4,
        }
    }
}

impl From<nvmlBridgeChipType_t> for BridgeChip {
    fn from(enum_: nvmlBridgeChipType_t) -> Self {
        match enum_ {
            nvmlBridgeChipType_t::NVML_BRIDGE_CHIP_PLX  => BridgeChip::PLX,
            nvmlBridgeChipType_t::NVML_BRIDGE_CHIP_BRO4 => BridgeChip::BRO4,
        }
    }
}

/// Memory error types.
#[derive(Debug)]
pub enum MemoryError {
    /// A memory error that was corrected for ECC errors.
    ///
    /// These are single bit errors for texture memory and are fixed by a resend.
    Corrected,
    /// A memory error that was not corrected for ECC errors.
    ///
    /// These are double bit errors for texture memory where the resend failed.
    Uncorrected,
}

impl MemoryError {
    /// Returns the C enum variant equivalent for the given Rust enum variant.
    pub fn eq_c_variant(&self) -> nvmlMemoryErrorType_t {
        match *self {
            MemoryError::Corrected 
                => nvmlMemoryErrorType_t::NVML_MEMORY_ERROR_TYPE_CORRECTED,
            MemoryError::Uncorrected
                => nvmlMemoryErrorType_t::NVML_MEMORY_ERROR_TYPE_UNCORRECTED,
        }
    }

    /// Waiting for `TryFrom` to be stable. In the meantime, we do this.
    pub fn try_from(enum_: nvmlMemoryErrorType_t) -> Result<Self> {
        match enum_ {
            nvmlMemoryErrorType_t::NVML_MEMORY_ERROR_TYPE_CORRECTED
                => Ok(MemoryError::Corrected),
            nvmlMemoryErrorType_t::NVML_MEMORY_ERROR_TYPE_UNCORRECTED
                => Ok(MemoryError::Uncorrected),
            nvmlMemoryErrorType_t::NVML_MEMORY_ERROR_TYPE_COUNT
                => bail!(ErrorKind::UnexpectedVariant)
        }
    }
}

/// ECC counter types.
///
/// Note: Volatile counts are reset each time the driver loads. On Windows this is
/// once per boot. On Linux this can be more frequent; the driver unloads when no
/// active clients exist. If persistence mode is enabled or there is always a
/// driver client active (such as X11), then Linux also sees per-boot behavior.
/// If not, volatile counts are reset each time a compute app is run.
#[derive(Debug)]
pub enum EccCounter {
    /// Volatile counts are reset each time the driver loads.
    Volatile,
    /// Aggregate counts persist across reboots (i.e. for the lifetime of the device).
    Aggregate,
}

impl EccCounter {
    /// Returns the C enum variant equivalent for the given Rust enum variant.
    pub fn eq_c_variant(&self) -> nvmlEccCounterType_t {
        match *self {
            EccCounter::Volatile => nvmlEccCounterType_t::NVML_VOLATILE_ECC,
            EccCounter::Aggregate => nvmlEccCounterType_t::NVML_AGGREGATE_ECC,
        }
    }

    /// Waiting for `TryFrom` to be stable. In the meantime, we do this.
    pub fn try_from(enum_: nvmlEccCounterType_t) -> Result<Self> {
        match enum_ {
            nvmlEccCounterType_t::NVML_VOLATILE_ECC
                => Ok(EccCounter::Volatile),
            nvmlEccCounterType_t::NVML_AGGREGATE_ECC
                => Ok(EccCounter::Aggregate),
            nvmlEccCounterType_t::NVML_ECC_COUNTER_TYPE_COUNT
                => bail!(ErrorKind::UnexpectedVariant),
        }
    }
}

/// Driver models, Windows only.
#[derive(Debug)]
#[cfg(target_os = "windows")]
pub enum DriverModel {
    /// GPU treated as a display device.
    WDDM,
    /// (TCC model) GPU treated as a generic device (recommended).
    WDM,
}

#[cfg(target_os = "windows")]
impl DriverModel {
    /// Returns the C enum variant equivalent for the given Rust enum variant.
    pub fn eq_c_variant(&self) -> nvmlDriverModel_t {
        match *self {
            DriverModel::WDDM => nvmlDriverModel_t::NVML_DRIVER_WDDM,
            DriverModel::WDM => nvmlDriverModel_t::NVML_DRIVER_WDM,
        }
    }
}

#[cfg(target_os = "windows")]
impl From<nvmlDriverModel_t> for DriverModel {
    fn from(enum_: nvmlDriverModel_t) -> Self {
        match enum_ {
            nvmlDriverModel_t::NVML_DRIVER_WDDM => DriverModel::WDDM,
            nvmlDriverModel_t::NVML_DRIVER_WDM => DriverModel::WDM,
        }
    }
}

/// GPU operation mode.
///
/// Allows for the reduction of power usage and optimization of GPU throughput
/// by disabling GPU features.
#[derive(Debug)]
pub enum OperationMode {
    AllOn,
    Compute,
    LowDP,
}

impl OperationMode {
    /// Returns the C enum variant equivalent for the given Rust enum variant.
    pub fn eq_c_variant(&self) -> nvmlGpuOperationMode_t {
        match *self {
            OperationMode::AllOn => nvmlGpuOperationMode_t::NVML_GOM_ALL_ON,
            OperationMode::Compute => nvmlGpuOperationMode_t::NVML_GOM_COMPUTE,
            OperationMode::LowDP => nvmlGpuOperationMode_t::NVML_GOM_LOW_DP,
            
        }
    }
}

impl From<nvmlGpuOperationMode_t> for OperationMode {
    fn from(enum_: nvmlGpuOperationMode_t) -> Self {
        match enum_ {
            nvmlGpuOperationMode_t::NVML_GOM_ALL_ON => OperationMode::AllOn,
            nvmlGpuOperationMode_t::NVML_GOM_COMPUTE => OperationMode::Compute,
            nvmlGpuOperationMode_t::NVML_GOM_LOW_DP => OperationMode::Compute,
        }
    }
}

pub fn bool_from_state(state: nvmlEnableState_t) -> bool {
    match state {
        nvmlEnableState_t::NVML_FEATURE_DISABLED => false,
        nvmlEnableState_t::NVML_FEATURE_ENABLED => true,
    }
}