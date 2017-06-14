use ffi::bindings::*;
use device::Device;
use bitmasks::event::*;
use enums::event::XidError;
use error::*;

// TODO: Should this be higher level. It probably should
/// Information about an event that has occurred.
// Checked against local
#[derive(Debug)]
pub struct EventData<'nvml> {
    /**
    Device where the event occurred.
    
    See `Device.uuid()` for a way to compare this `Device` to another `Device`
    and find out if they represent the same physical device.
    */
    pub device: Device<'nvml>,
    /// Information about what specific event occurred.
    pub event_type: EventTypes,
    /**
    Stores the last XID error for the device for the
    nvmlEventTypeXidCriticalError event.
    
    `None` in the case of any other event type.
    */
    pub event_data: Option<XidError>,
}

impl<'nvml> EventData<'nvml> {
    /// Waiting for `TryFrom` to be stable. In the meantime, we do this.
    pub fn try_from(struct_: nvmlEventData_t) -> Result<Self> {
        let event_type = match EventTypes::from_bits(struct_.eventType) {
            Some(t) => t,
            None    => bail!(ErrorKind::IncorrectBits(Bits::U64(struct_.eventType))),
        };

        Ok(EventData {
            device: struct_.device.into(),
            event_type,
            event_data: if event_type.contains(CRITICAL_XID_ERROR) {
                Some(match struct_.eventData {
                    999 => XidError::Unknown,
                    v => XidError::Value(v),
                })
            } else {
                None
            },
        })
    }
}
