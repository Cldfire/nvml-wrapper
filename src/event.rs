use ffi::bindings::*;
use error::*;
use struct_wrappers::event::*;
use std::mem;
use std::marker::PhantomData;
use std::os::raw::c_uint;
use std::io;
use std::io::Write;
use NVML;

/**
Handle to a set of events.

**Operations on a set are not thread-safe.** It does not, therefore, implement `Sync`.

Once again, Rust's lifetimes will ensure that this EventSet does not outlive the
`NVML` instance that it was created from.
*/
// Checked against local
#[derive(Debug, Clone)]
pub struct EventSet<'nvml> {
    set: nvmlEventSet_t,
    _phantom: PhantomData<&'nvml NVML>,
}

unsafe impl<'nvml> Send for EventSet<'nvml> {}

impl<'nvml> From<nvmlEventSet_t> for EventSet<'nvml> {
    fn from(set: nvmlEventSet_t) -> Self {
        EventSet {
            set: set,
            _phantom: PhantomData,
        }
    }
}

impl<'nvml> EventSet<'nvml> {
    /**
    Use this to release events in the set if you care about handling
    potential errors (*the `Drop` implementation ignores errors!*).
    
    # Errors
    * `Uninitialized`, if the library has not been successfully initialized
    * `Unknown`, on any unexpected error
    */
    // Checked against local
    // TODO: Should this be a weaker name?
    #[inline]
    pub fn release_events(self) -> Result<()> {
        unsafe {
            nvml_try(nvmlEventSetFree(self.set))?;
        }
        Ok(mem::forget(self))
    }

    /**
    Waits on events for the given timeout (in ms) and delivers one when it arrives.
    
    This method returns immediately if an event is ready to be delivered when it
    is called. If no events are ready it will sleep until an event arrives, but
    not longer than the specified timeout. In certain conditions, this method
    could return before the timeout passes (e.g. when an interrupt arrives).
    
    In the case of an XID error, the function returns the most recent XID error
    type seen by the system. If there are multiple XID errors generated before
    this method is called, the last seen XID error type will be returned for
    all XID error events.
    
    # Errors
    * `Uninitialized`, if the library has not been successfully initialized
    * `Timeout`, if no event arrived in the specified timeout or an interrupt
    arrived
    * `GpuLost`, if a GPU has fallen off the bus or is otherwise inaccessible
    * `Unknown`, on any unexpected error
    
    # Device Support
    Supports Fermi and newer fully supported devices.
    */
    // Checked against local
    // TODO: Should I go higher level with this?
    // Should it be tied to the device and managed for you
    #[inline]
    pub fn wait(&self, timeout_ms: u32) -> Result<EventData<'nvml>> {
        unsafe {
            let mut data: nvmlEventData_t = mem::zeroed();
            nvml_try(nvmlEventSetWait(self.set, &mut data, timeout_ms as c_uint))?;

            Ok(EventData::try_from(data)?)
        }
    }

    /// Only use this if it's absolutely necessary.
    // TODO: This should consume self
    #[inline]
    pub fn c_set(&self) -> nvmlEventSet_t {
        self.set
    }
}

/// This `Drop` implementation ignores errors! Use the `.release_events()` method on the `EventSet`
/// struct if you care about handling them.
impl<'nvml> Drop for EventSet<'nvml> {
    fn drop(&mut self) {
        unsafe {
            match nvml_try(nvmlEventSetFree(self.set)) {
                Ok(()) => (),
                Err(e) => {
                    io::stderr().write(&format!("WARNING: Error returned by \
                        `nmvlEventSetFree()` in Drop implementation: {:?}", e).as_bytes());
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use EventSet;
    use test_utils::*;
    use bitmasks::event::*;

    #[test]
    fn release_events() {
        let nvml = nvml();
        test_with_device(3, &nvml, |device| {
            let set = nvml.create_event_set()?;
            let set = device.register_events(PSTATE_CHANGE |
                                             CRITICAL_XID_ERROR |
                                             CLOCK_CHANGE,
                                             set)?;

            set.release_events()
        })
    }

    #[cfg(feature = "test-local")]
    #[test]
    fn wait() {
        let nvml = nvml();
        let device = device(&nvml);
        let set = nvml.create_event_set().expect("event set");
        let set = device.register_events(PSTATE_CHANGE |
                                         CRITICAL_XID_ERROR |
                                         CLOCK_CHANGE,
                                         set).expect("registration");

        let data = set.wait(10_000).expect("event data");
        print!("{:?} ...", data)
    }
}
