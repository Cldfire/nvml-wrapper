use super::ffi::*;
use super::errors::*;

// TODO: Test everything in this module.
// TODO: Check all of these things against local nvml.h

/// API types that allow changes to default permission restrictions.
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
pub enum BridgeChip {
    PLX,
    BRO4,
}

impl BridgeChip {
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