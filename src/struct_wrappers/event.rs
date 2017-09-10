use bitmasks::event::EventType;
// use bitmasks::event::EventTypes::CRITICAL_XID_ERROR;
use device::Device;
use enums::event::XidError;
use error::{Result, Bits, ErrorKind};
use ffi::bindings::*;

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
    pub event_type: EventType,
    /**
    Stores the last XID error for the device for the
    nvmlEventTypeXidCriticalError event.
    
    `None` in the case of any other event type.
    */
    pub event_data: Option<XidError>
}

impl<'nvml> EventData<'nvml> {
    /// Waiting for `TryFrom` to be stable. In the meantime, we do this.
    pub fn try_from(struct_: nvmlEventData_t) -> Result<Self> {
        let event_type = match EventType::from_bits(struct_.eventType) {
            Some(t) => t,
            None => bail!(ErrorKind::IncorrectBits(Bits::U64(struct_.eventType))),
        };

        Ok(EventData {
            device: struct_.device.into(),
            event_type,
            event_data: if event_type.contains(EventType::CRITICAL_XID_ERROR) {
                Some(match struct_.eventData {
                    999 => XidError::Unknown,
                    v => XidError::Value(v),
                })
            } else {
                None
            }
        })
    }
}
