use ffi::bindings::*;
use device::Device;
use bitmasks::event::*;
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
    /// Stores the last XID error for the device in the event of nvmlEventTypeXidCriticalError,
    /// is 0 for any other event. Is 999 for an unknown XID error.
    pub event_data: u64,
}

impl<'nvml> EventData<'nvml> {
    /// Waiting for `TryFrom` to be stable. In the meantime, we do this.
    pub fn try_from(struct_: nvmlEventData_t) -> Result<Self> {
        Ok(EventData {
            device: struct_.device.into(),
            event_type: match EventTypes::from_bits(struct_.eventType as u64) {
                Some(t) => t,
                // TODO: Send incorrect bits back in error
                None    => bail!(ErrorKind::IncorrectBits),
            },
            event_data: struct_.eventData as u64,
        })
    }
}
