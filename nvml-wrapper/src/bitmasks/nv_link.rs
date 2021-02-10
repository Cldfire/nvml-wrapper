use crate::ffi::bindings::*;
use bitflags::bitflags;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

bitflags! {
    /**
    Represents the NvLink utilization counter packet types that can be counted.

    Only applicable when `UtilizationCountUnit`s are packets or bytes. All
    packet filter descriptions are target GPU centric.
    */
    // Checked against local
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub struct PacketTypes: u32 {
        const NO_OP      = nvmlNvLinkUtilizationCountPktTypes_enum_NVML_NVLINK_COUNTER_PKTFILTER_NOP as u32;
        const READ       = nvmlNvLinkUtilizationCountPktTypes_enum_NVML_NVLINK_COUNTER_PKTFILTER_READ as u32;
        const WRITE      = nvmlNvLinkUtilizationCountPktTypes_enum_NVML_NVLINK_COUNTER_PKTFILTER_WRITE as u32;
        /// Reduction atomic requests.
        const RATOM      = nvmlNvLinkUtilizationCountPktTypes_enum_NVML_NVLINK_COUNTER_PKTFILTER_RATOM as u32;
        /// Non-reduction atomic requests.
        const NON_RATOM  = nvmlNvLinkUtilizationCountPktTypes_enum_NVML_NVLINK_COUNTER_PKTFILTER_NRATOM as u32;
        /// Flush requests.
        const FLUSH      = nvmlNvLinkUtilizationCountPktTypes_enum_NVML_NVLINK_COUNTER_PKTFILTER_FLUSH as u32;
        /// Responses with data.
        const WITH_DATA  = nvmlNvLinkUtilizationCountPktTypes_enum_NVML_NVLINK_COUNTER_PKTFILTER_RESPDATA as u32;
        /// Responses without data.
        const NO_DATA    = nvmlNvLinkUtilizationCountPktTypes_enum_NVML_NVLINK_COUNTER_PKTFILTER_RESPNODATA as u32;
    }
}
