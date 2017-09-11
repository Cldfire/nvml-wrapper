use bitmasks::nv_link::PacketTypes;
use enum_wrappers::nv_link::UtilizationCountUnit;
use error::Result;
use ffi::bindings::*;

/// Defines NvLink counter controls.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct UtilizationControl {
    pub units: UtilizationCountUnit,
    pub packet_filter: PacketTypes
}

impl UtilizationControl {
    /**
    Waiting for `TryFrom` to be stable. In the meantime, we do this.

    The `packet_filter` bitmask is created via the `PacketTypes::from_bits_truncate`
    method, meaning that any bits that don't correspond to flags present in this
    version of the wrapper will be dropped.

    # Errors

    * `UnexpectedVariant`, for which you can read the docs for
    */
    pub fn try_from(struct_: nvmlNvLinkUtilizationControl_t) -> Result<Self> {
        let bits = struct_.pktfilter as u32;

        Ok(UtilizationControl {
            units: UtilizationCountUnit::try_from(struct_.units)?,
            packet_filter: PacketTypes::from_bits_truncate(bits)
        })
    }

    /// Obtain this struct's C counterpart.
    pub fn as_c(&self) -> nvmlNvLinkUtilizationControl_t {
        nvmlNvLinkUtilizationControl_t {
            units: self.units.as_c(),
            pktfilter: self.packet_filter.bits()
        }
    }
}
