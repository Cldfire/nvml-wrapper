use ffi::bindings::*;
use error::*;
use struct_wrappers::event::*;
use std::mem;
use std::marker::PhantomData;
use std::io;
use std::io::Write;
use NVML;

// TODO: This should most probably only be compiled on linux

/**
Handle to a set of events.

**Operations on a set are not thread-safe.** It does not, therefore, implement `Sync`.

You can get yourself an `EventSet` via `NVML.create_event_set()`. Once again, Rust's
lifetimes will ensure that it does not outlive the `NVML` instance that it was created
from.
*/
// Checked against local
#[derive(Debug)]
pub struct EventSet<'nvml> {
    set: nvmlEventSet_t,
    _phantom: PhantomData<&'nvml NVML>,
}

unsafe impl<'nvml> Send for EventSet<'nvml> {}

impl<'nvml> From<nvmlEventSet_t> for EventSet<'nvml> {
    fn from(set: nvmlEventSet_t) -> Self {
        EventSet {
            set,
            _phantom: PhantomData,
        }
    }
}

impl<'nvml> EventSet<'nvml> {
    /**
    Use this to release the set's events if you care about handling
    potential errors (*the `Drop` implementation ignores errors!*).
    
    # Errors
    * `Uninitialized`, if the library has not been successfully initialized
    * `Unknown`, on any unexpected error
    */
    // Checked against local
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
            nvml_try(nvmlEventSetWait(self.set, &mut data, timeout_ms))?;

            Ok(EventData::try_from(data)?)
        }
    }

    /// Consume the struct and obtain the raw set handle that it contains.
    #[inline]
    pub fn into_raw(self) -> nvmlEventSet_t {
        let set = self.set;
        mem::forget(self);
        set
    }

    /// Obtain a reference to the raw set handle contained in the struct.
    #[inline]
    pub fn as_raw(&self) -> &nvmlEventSet_t {
        &(self.set)
    }

    /// Obtain a mutable reference to the raw set handle contained in the struct.
    #[inline]
    pub fn as_mut_raw(&mut self) -> &mut nvmlEventSet_t {
        &mut (self.set)
    }

    #[inline]
    /// Sometimes necessary for C interop. Use carefully.
    pub unsafe fn unsafe_raw(&self) -> nvmlEventSet_t {
        self.set
    }
}

/// This `Drop` implementation ignores errors! Use the `.release_events()` method on the `EventSet`
/// struct if you care about handling them.
impl<'nvml> Drop for EventSet<'nvml> {
    fn drop(&mut self) {
        #[allow(unused_must_use)]
        unsafe {
            match nvml_try(nvmlEventSetFree(self.set)) {
                Ok(()) => (),
                Err(e) => {
                    io::stderr().write(format!("WARNING: Error returned by \
                        `nmvlEventSetFree()` in Drop implementation: {:?}", e).as_bytes());
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::EventSet;
    use test_utils::*;
    #[cfg(target_os = "linux")]
    use bitmasks::event::*;

    // Ensuring that double-free issues don't crop up here.
    #[test]
    fn into_raw() {
        let nvml = nvml();
        let raw;

        {
            let set = nvml.create_event_set().expect("set");
            raw = set.into_raw();
        }

        EventSet::from(raw);
    }

    #[cfg(target_os = "linux")]
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

    #[cfg(target_os = "linux")]
    #[cfg(feature = "test-local")]
    #[test]
    fn wait() {
        use error::*;

        let nvml = nvml();
        let device = device(&nvml);
        let set = nvml.create_event_set().expect("event set");
        let set = device.register_events(PSTATE_CHANGE |
                                         CRITICAL_XID_ERROR |
                                         CLOCK_CHANGE,
                                         set).expect("registration");

        let data = match set.wait(10_000) {
            Err(Error(ErrorKind::Timeout, _)) => return (),
            Ok(d) => d,
            _ => panic!("An error other than `Timeout` occurred")
        };

        print!("{:?} ...", data);
    }
}
