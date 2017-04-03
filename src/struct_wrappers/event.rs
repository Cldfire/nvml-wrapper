use ffi::*;
use device::Device;

// TODO: Should this be higher level. It probably should
// Store specific event flag ^
/// Information about an event that has occurred.
// Checked against local
#[derive(Debug)]
pub struct EventData<'nvml> {
    /// Device where the event occurred.
    // TODO: Need to be able to compare device handles for equality due to this
    pub device: Device<'nvml>,
    /// Information about what specific event occurred.
    pub event_type: u64,
    /// Stores the last XID error for the device in the event of nvmlEventTypeXidCriticalError,
    /// is 0 for any other event. Is 999 for an unknown XID error.
    pub event_data: u64,
}

impl<'nvml> From<nvmlEventData_t> for EventData<'nvml> {
    fn from(struct_: nvmlEventData_t) -> Self {
        EventData {
            device: struct_.device.into(),
            event_type: struct_.eventType as u64,
            event_data: struct_.eventData as u64,
        }
    }
}
