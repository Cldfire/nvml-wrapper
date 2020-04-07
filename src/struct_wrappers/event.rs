use crate::bitmasks::event::EventTypes;
use crate::device::Device;
use crate::enums::event::XidError;
use crate::ffi::bindings::*;

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
    pub event_data: Option<XidError>
}

impl<'nvml> From<nvmlEventData_t> for EventData<'nvml> {
    /**
    Performs the conversion.
    
    The `event_type` bitmask is created via the `EventTypes::from_bits_truncate`
    method, meaning that any bits that don't correspond to flags present in this
    version of the wrapper will be dropped.
    */
    fn from(struct_: nvmlEventData_t) -> Self {
        let event_type = EventTypes::from_bits_truncate(struct_.eventType);

        EventData {
            device: struct_.device.into(),
            event_type,
            event_data: if event_type.contains(EventTypes::CRITICAL_XID_ERROR) {
                Some(match struct_.eventData {
                    999 => XidError::Unknown,
                    v => XidError::Value(v),
                })
            } else {
                None
            }
        }
    }
}
