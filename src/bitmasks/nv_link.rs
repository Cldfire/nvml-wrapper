use ffi::nvmlNvLinkUtilizationCountPktTypes_t::*;

bitflags! {
    /// Represents the NvLink utilization counter packet types that can be counted.
    ///
    /// Only applicable when `UtilizationCountUnit`s are packets or bytes. All 
    /// packet filter descriptions are target GPU centric.
    ///
    /// This can be "OR'd" together.
    // Checked against local
    pub flags PacketTypes: u32 {
        const NO_OP      = NVML_NVLINK_COUNTER_PKTFILTER_NOP as u32,
        const READ       = NVML_NVLINK_COUNTER_PKTFILTER_READ as u32,
        const WRITE      = NVML_NVLINK_COUNTER_PKTFILTER_WRITE as u32,
        /// Reduction atomic requests.
        const RATOM      = NVML_NVLINK_COUNTER_PKTFILTER_RATOM as u32,
        /// Non-reduction atomic requests.
        const NON_RATOM  = NVML_NVLINK_COUNTER_PKTFILTER_NRATOM as u32,
        /// Flush requests.
        const FLUSH      = NVML_NVLINK_COUNTER_PKTFILTER_FLUSH as u32,
        /// Responses with data.
        const WITH_DATA  = NVML_NVLINK_COUNTER_PKTFILTER_RESPDATA as u32,
        /// Responses without data.
        const NO_DATA    = NVML_NVLINK_COUNTER_PKTFILTER_RESPNODATA as u32,
    }
}