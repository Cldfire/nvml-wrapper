use ffi::*;
use nvml_errors::*;

/// Represents the NvLink utilization counter packet units.
// Checked against local
#[derive(EnumWrapper, Debug)]
#[wrap(c_enum = "nvmlNvLinkUtilizationCountUnits_t")]
#[wrap(has_count = "NVML_NVLINK_COUNTER_UNIT_COUNT")]
pub enum UtilizationCountUnits {
    #[wrap(c_variant = "NVML_NVLINK_COUNTER_UNIT_CYCLES")]
    Cycles,
    #[wrap(c_variant = "NVML_NVLINK_COUNTER_UNIT_PACKETS")]
    Packets,
    #[wrap(c_variant = "NVML_NVLINK_COUNTER_UNIT_BYTES")]
    Bytes,
}

/// Represents the NvLink utilization counter packet types that can be counted.
///
/// Only applica
#[derive(EnumWrapper, Debug)]
#[wrap(c_enum = "nvmlNvLinkUtilizationCountPktTypes_t")]
pub enum UtilizationCountPacketTypes {
    #[wrap(c_variant = "NVML_NVLINK_COUNTER_PKTFILTER_NOP")]
    NoOp,
    #[wrap(c_variant = "NVML_NVLINK_COUNTER_PKTFILTER_READ")]
    Read,
    #[wrap(c_variant = "NVML_NVLINK_COUNTER_PKTFILTER_WRITE")]
    Write,
    #[wrap(c_variant = "NVML_NVLINK_COUNTER_PKTFILTER_RATOM")]
    Ratom,
    #[wrap(c_variant = "NVML_NVLINK_COUNTER_PKTFILTER_NRATOM")]
    NRatom,
    #[wrap(c_variant = "NVML_NVLINK_COUNTER_PKTFILTER_FLUSH")]
    Flush,
    #[wrap(c_variant = "NVML_NVLINK_COUNTER_PKTFILTER_RESPDATA")]
    WithData,
    #[wrap(c_variant = "NVML_NVLINK_COUNTER_PKTFILTER_RESPNODATA")]
    NoData,
    #[wrap(c_variant = "NVML_NVLINK_COUNTER_PKTFILTER_ALL")]
    All,
}
