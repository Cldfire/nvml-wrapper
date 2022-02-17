pub mod device;
pub mod event;
pub mod nv_link;
pub mod unit;

use self::device::PciInfo;
use crate::error::NvmlError;
use crate::ffi::bindings::*;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::{convert::TryFrom, ffi::CStr};

/// Information about a blacklisted device
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BlacklistDeviceInfo {
    pci_info: PciInfo,
    uuid: String,
}

impl TryFrom<nvmlExcludedDeviceInfo_t> for BlacklistDeviceInfo {
    type Error = NvmlError;

    /**
    Construct `BlacklistDeviceInfo` from the corresponding C struct.

    # Errors

    * `UnexpectedVariant`, for which you can read the docs for
    */
    fn try_from(value: nvmlExcludedDeviceInfo_t) -> Result<Self, Self::Error> {
        unsafe {
            let uuid_raw = CStr::from_ptr(value.uuid.as_ptr());

            Ok(Self {
                pci_info: PciInfo::try_from(value.pciInfo, true)?,
                uuid: uuid_raw.to_str()?.into(),
            })
        }
    }
}
