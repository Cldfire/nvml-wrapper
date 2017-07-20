use bitmasks::nv_link::*;
use enum_wrappers::nv_link::*;
use error::*;
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

    # Errors
    
    * `UnexpectedVariant`, for which you can read the docs for
    * `IncorrectBits`, if bits obtained in this method cannot be interpreted
    as `PacketTypes`
    */
    pub fn try_from(struct_: nvmlNvLinkUtilizationControl_t) -> Result<Self> {
        let bits = struct_.pktfilter as u32;

        Ok(UtilizationControl {
            units: UtilizationCountUnit::try_from(struct_.units)?,
            packet_filter: match PacketTypes::from_bits(bits) {
                Some(t) => t,
                None => bail!(ErrorKind::IncorrectBits(Bits::U32(bits))),
            }
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
