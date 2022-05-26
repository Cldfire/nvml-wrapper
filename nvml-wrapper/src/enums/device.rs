use std::convert::TryFrom;
use std::fmt::Display;
use std::os::raw::c_uint;

use crate::enum_wrappers::device::{ClockLimitId, SampleValueType};
use crate::error::NvmlError;
use crate::ffi::bindings::*;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Respresents possible variants for a firmware version.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum FirmwareVersion {
    /// The version is unavailable.
    Unavailable,
    Version(u32),
}

impl From<u32> for FirmwareVersion {
    fn from(value: u32) -> Self {
        match value {
            0 => FirmwareVersion::Unavailable,
            _ => FirmwareVersion::Version(value),
        }
    }
}

/// Represents possible variants for used GPU memory.
// Checked
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum UsedGpuMemory {
    /// Under WDDM, `NVML_VALUE_NOT_AVAILABLE` is always reported because
    /// Windows KMD manages all the memory, not the NVIDIA driver.
    Unavailable,
    /// Memory used in bytes.
    Used(u64),
}

impl From<u64> for UsedGpuMemory {
    fn from(value: u64) -> Self {
        let not_available = (NVML_VALUE_NOT_AVAILABLE) as u64;

        match value {
            v if v == not_available => UsedGpuMemory::Unavailable,
            _ => UsedGpuMemory::Used(value),
        }
    }
}

/// Represents different types of sample values.
// Checked against local
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum SampleValue {
    F64(f64),
    U32(u32),
    U64(u64),
    I64(i64),
}

impl SampleValue {
    pub fn from_tag_and_union(tag: &SampleValueType, union: nvmlValue_t) -> Self {
        use self::SampleValueType::*;

        unsafe {
            match *tag {
                Double => SampleValue::F64(union.dVal),
                UnsignedInt => SampleValue::U32(union.uiVal),
                // Methodology: NVML supports 32-bit Linux. UL is u32 on that platform.
                // NVML wouldn't return anything larger
                UnsignedLong => SampleValue::U32(union.ulVal as u32),
                UnsignedLongLong => SampleValue::U64(union.ullVal),
                SignedLongLong => SampleValue::I64(union.sllVal),
            }
        }
    }
}

/// Represents different types of sample values.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum GpuLockedClocksSetting {
    /// Numeric setting that allows you to explicitly define minimum and
    /// maximum clock frequencies.
    Numeric {
        min_clock_mhz: u32,
        max_clock_mhz: u32,
    },
    /// Symbolic setting that allows you to define lower and upper bounds for
    /// clock speed with various possibilities.
    ///
    /// Not all combinations of `lower_bound` and `upper_bound` are valid.
    /// Please see the docs for `nvmlDeviceSetGpuLockedClocks` in `nvml.h` to
    /// learn more.
    Symbolic {
        lower_bound: ClockLimitId,
        upper_bound: ClockLimitId,
    },
}

impl GpuLockedClocksSetting {
    /// Returns `(min_clock_mhz, max_clock_mhz)`.
    pub fn into_min_and_max_clocks(self) -> (u32, u32) {
        match self {
            GpuLockedClocksSetting::Numeric {
                min_clock_mhz,
                max_clock_mhz,
            } => (min_clock_mhz, max_clock_mhz),
            GpuLockedClocksSetting::Symbolic {
                lower_bound,
                upper_bound,
            } => (lower_bound.as_c(), upper_bound.as_c()),
        }
    }
}

/// Returned by [`crate::Device::bus_type()`].
// TODO: technically this is an "enum wrapper" but the type on the C side isn't
// an enum
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum BusType {
    /// Unknown bus type.
    Unknown,
    /// PCI (Peripheral Component Interconnect) bus type.
    Pci,
    /// PCIE (Peripheral Component Interconnect Express) bus type.
    ///
    /// This is the most common bus type.
    Pcie,
    /// FPCI (Fast Peripheral Component Interconnect) bus type.
    Fpci,
    /// AGP (Accelerated Graphics Port) bus type.
    ///
    /// This is old and was dropped in favor of PCIE.
    Agp,
}

impl BusType {
    /// Returns the C constant equivalent for the given Rust enum variant.
    pub fn as_c(&self) -> nvmlBusType_t {
        match *self {
            Self::Unknown => NVML_BUS_TYPE_UNKNOWN,
            Self::Pci => NVML_BUS_TYPE_PCI,
            Self::Pcie => NVML_BUS_TYPE_PCIE,
            Self::Fpci => NVML_BUS_TYPE_FPCI,
            Self::Agp => NVML_BUS_TYPE_AGP,
        }
    }
}

impl TryFrom<nvmlBusType_t> for BusType {
    type Error = NvmlError;

    fn try_from(data: nvmlBusType_t) -> Result<Self, Self::Error> {
        match data {
            NVML_BUS_TYPE_UNKNOWN => Ok(Self::Unknown),
            NVML_BUS_TYPE_PCI => Ok(Self::Pci),
            NVML_BUS_TYPE_PCIE => Ok(Self::Pcie),
            NVML_BUS_TYPE_FPCI => Ok(Self::Fpci),
            NVML_BUS_TYPE_AGP => Ok(Self::Agp),
            _ => Err(NvmlError::UnexpectedVariant(data)),
        }
    }
}

/// Returned by [`crate::Device::power_source()`].
// TODO: technically this is an "enum wrapper" but the type on the C side isn't
// an enum
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum PowerSource {
    /// AC power (receiving power from some external source).
    Ac,
    /// Battery power.
    Battery,
}

impl PowerSource {
    /// Returns the C constant equivalent for the given Rust enum variant.
    pub fn as_c(&self) -> nvmlPowerSource_t {
        match *self {
            Self::Ac => NVML_POWER_SOURCE_AC,
            Self::Battery => NVML_POWER_SOURCE_BATTERY,
        }
    }
}

impl TryFrom<nvmlPowerSource_t> for PowerSource {
    type Error = NvmlError;

    fn try_from(data: nvmlPowerSource_t) -> Result<Self, Self::Error> {
        match data {
            NVML_POWER_SOURCE_AC => Ok(Self::Ac),
            NVML_POWER_SOURCE_BATTERY => Ok(Self::Battery),
            _ => Err(NvmlError::UnexpectedVariant(data)),
        }
    }
}

/// Returned by [`crate::Device::architecture()`].
///
/// This is the simplified chip architecture of the device.
// TODO: technically this is an "enum wrapper" but the type on the C side isn't
// an enum
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum DeviceArchitecture {
    /// <https://en.wikipedia.org/wiki/Kepler_(microarchitecture)>
    Kepler,
    /// <https://en.wikipedia.org/wiki/Maxwell_(microarchitecture)>
    Maxwell,
    /// <https://en.wikipedia.org/wiki/Pascal_(microarchitecture)>
    Pascal,
    /// <https://en.wikipedia.org/wiki/Volta_(microarchitecture)>
    Volta,
    /// <https://en.wikipedia.org/wiki/Turing_(microarchitecture)>
    Turing,
    /// <https://en.wikipedia.org/wiki/Ampere_(microarchitecture)>
    Ampere,
    /// Unknown device architecture (most likely something newer).
    Unknown,
}

impl DeviceArchitecture {
    /// Returns the C constant equivalent for the given Rust enum variant.
    pub fn as_c(&self) -> nvmlDeviceArchitecture_t {
        match *self {
            Self::Kepler => NVML_DEVICE_ARCH_KEPLER,
            Self::Maxwell => NVML_DEVICE_ARCH_MAXWELL,
            Self::Pascal => NVML_DEVICE_ARCH_PASCAL,
            Self::Volta => NVML_DEVICE_ARCH_VOLTA,
            Self::Turing => NVML_DEVICE_ARCH_TURING,
            Self::Ampere => NVML_DEVICE_ARCH_AMPERE,
            Self::Unknown => NVML_DEVICE_ARCH_UNKNOWN,
        }
    }
}

impl TryFrom<nvmlDeviceArchitecture_t> for DeviceArchitecture {
    type Error = NvmlError;

    fn try_from(data: nvmlDeviceArchitecture_t) -> Result<Self, Self::Error> {
        match data {
            NVML_DEVICE_ARCH_KEPLER => Ok(Self::Kepler),
            NVML_DEVICE_ARCH_MAXWELL => Ok(Self::Maxwell),
            NVML_DEVICE_ARCH_PASCAL => Ok(Self::Pascal),
            NVML_DEVICE_ARCH_VOLTA => Ok(Self::Volta),
            NVML_DEVICE_ARCH_TURING => Ok(Self::Turing),
            NVML_DEVICE_ARCH_AMPERE => Ok(Self::Ampere),
            NVML_DEVICE_ARCH_UNKNOWN => Ok(Self::Unknown),
            _ => Err(NvmlError::UnexpectedVariant(data)),
        }
    }
}

impl Display for DeviceArchitecture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Kepler => f.write_str("Kepler"),
            Self::Maxwell => f.write_str("Maxwell"),
            Self::Pascal => f.write_str("Pascal"),
            Self::Volta => f.write_str("Volta"),
            Self::Turing => f.write_str("Turing"),
            Self::Ampere => f.write_str("Ampere"),
            Self::Unknown => f.write_str("Unknown"),
        }
    }
}

/// Returned by [`crate::Device::pcie_link_max_speed()`].
///
/// Note, the NVML header says these are all MBPS (Megabytes Per Second) but
/// they don't line up with the throughput numbers on this page:
/// <https://en.wikipedia.org/wiki/PCI_Express>
///
/// They _do_ line up with the "transfer rate per lane" numbers, though.
// TODO: technically this is an "enum wrapper" but the type on the C side isn't
// an enum
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum PcieLinkMaxSpeed {
    Invalid,
    MegabytesPerSecond2500,
    MegabytesPerSecond5000,
    MegabytesPerSecond8000,
    MegabytesPerSecond16000,
    MegabytesPerSecond32000,
}

impl PcieLinkMaxSpeed {
    /// Returns the C constant equivalent for the given Rust enum variant.
    pub fn as_c(&self) -> c_uint {
        match *self {
            Self::Invalid => NVML_PCIE_LINK_MAX_SPEED_INVALID,
            Self::MegabytesPerSecond2500 => NVML_PCIE_LINK_MAX_SPEED_2500MBPS,
            Self::MegabytesPerSecond5000 => NVML_PCIE_LINK_MAX_SPEED_5000MBPS,
            Self::MegabytesPerSecond8000 => NVML_PCIE_LINK_MAX_SPEED_8000MBPS,
            Self::MegabytesPerSecond16000 => NVML_PCIE_LINK_MAX_SPEED_16000MBPS,
            Self::MegabytesPerSecond32000 => NVML_PCIE_LINK_MAX_SPEED_32000MBPS,
        }
    }
}

impl TryFrom<c_uint> for PcieLinkMaxSpeed {
    type Error = NvmlError;

    fn try_from(data: c_uint) -> Result<Self, Self::Error> {
        match data {
            NVML_PCIE_LINK_MAX_SPEED_INVALID => Ok(Self::Invalid),
            NVML_PCIE_LINK_MAX_SPEED_2500MBPS => Ok(Self::MegabytesPerSecond2500),
            NVML_PCIE_LINK_MAX_SPEED_5000MBPS => Ok(Self::MegabytesPerSecond5000),
            NVML_PCIE_LINK_MAX_SPEED_8000MBPS => Ok(Self::MegabytesPerSecond8000),
            NVML_PCIE_LINK_MAX_SPEED_16000MBPS => Ok(Self::MegabytesPerSecond16000),
            NVML_PCIE_LINK_MAX_SPEED_32000MBPS => Ok(Self::MegabytesPerSecond32000),
            _ => Err(NvmlError::UnexpectedVariant(data)),
        }
    }
}
