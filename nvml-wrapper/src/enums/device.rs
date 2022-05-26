use std::convert::TryFrom;

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
            BusType::Unknown => NVML_BUS_TYPE_UNKNOWN,
            BusType::Pci => NVML_BUS_TYPE_PCI,
            BusType::Pcie => NVML_BUS_TYPE_PCIE,
            BusType::Fpci => NVML_BUS_TYPE_FPCI,
            BusType::Agp => NVML_BUS_TYPE_AGP,
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
