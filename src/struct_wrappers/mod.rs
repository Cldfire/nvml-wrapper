pub mod device;
pub mod unit;
pub mod event;
pub mod nv_link;

use self::device::PciInfo;
use error::Result;
use ffi::bindings::*;
use std::ffi::CStr;

/// Information about a blacklisted device
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BlacklistDeviceInfo {
    pci_info: PciInfo,
    uuid: String
}

impl BlacklistDeviceInfo {
    /**
    Try converting the C structure into a rustified one.

    # Errors

    * `Utf8Error`, if strings obtained from the C function are not valid Utf8
    */
    pub fn try_from(
        struct_: nvmlBlacklistDeviceInfo_t,
    ) -> Result<Self> {

        unsafe {
            let uuid_raw = CStr::from_ptr(struct_.uuid.as_ptr());

            Ok(Self {
                pci_info: PciInfo::try_from(struct_.pciInfo, true)?,
                uuid: uuid_raw.to_str()?.into()
            })
        }
    }
}
