use super::ffi::*;

/// API types that allow changes to default permission restrictions.
pub enum RestrictedApi {
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

impl RestrictedApi {
    /// Returns the C enum variant equivalent for the given Rust enum variant. 
    pub fn eq_c_variant(&self) -> nvmlRestrictedAPI_enum {
        match *self {
            RestrictedApi::ApplicationClocks
                => nvmlRestrictedAPI_enum::NVML_RESTRICTED_API_SET_APPLICATION_CLOCKS,
            RestrictedApi::AutoBoostedClocks
                => nvmlRestrictedAPI_enum::NVML_RESTRICTED_API_SET_AUTO_BOOSTED_CLOCKS,
        }
    }
}

/// Clock types. All speeds are in Mhz. 
pub enum ClockType {
    /// Graphics clock domain.
    Graphics,
    /// SM clock domain.
    // TODO: Improve that ^
    SM,
    /// Memory clock domain.
    Memory,
}

impl ClockType {
    /// Returns the C enum variant equivalent for the given Rust enum variant. 
    pub fn eq_c_variant(&self) -> nvmlClockType_enum {
        match *self {
            ClockType::Graphics => nvmlClockType_enum::NVML_CLOCK_GRAPHICS,
            ClockType::SM       => nvmlClockType_enum::NVML_CLOCK_SM,
            ClockType::Memory   => nvmlClockType_enum::NVML_CLOCK_MEM,
        }
    }
}