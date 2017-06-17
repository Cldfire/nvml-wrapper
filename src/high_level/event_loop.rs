/*!
The functionality in this module is only available on Linux platforms; NVML does
not support events on any other platform.
*/

use Device;
use EventSet;
use NVML;
use bitmasks::event::*;
use enums::event::XidError;
use error::*;
use struct_wrappers::event::EventData;

// TODO: Tests

/**
Represents the event types that an `EventLoop` can gather for you.

These are analagous to the constants in `bitmasks::event`.

Checking to see if the `Device` within an `Event` is the same physical device as
another `Device` that you have on hand can be accomplished via `Device.uuid()`.
*/
#[derive(Debug)]
pub enum Event<'nvml> {
    ClockChange(Device<'nvml>),
    CriticalXidError(Device<'nvml>, XidError),
    DoubleBitEccError(Device<'nvml>),
    PowerStateChange(Device<'nvml>),
    SingleBitEccError(Device<'nvml>),
    /// Returned if none of the above event types are contained in the
    /// `EventData` the `EventLoop` processes.
    Unknown
}

impl<'nvml> From<EventData<'nvml>> for Event<'nvml> {
    fn from(struct_: EventData<'nvml>) -> Self {
        if struct_.event_type.contains(CLOCK_CHANGE) {
            Event::ClockChange(struct_.device)
        } else if struct_.event_type.contains(CRITICAL_XID_ERROR) {
            // We can unwrap here because we know `event_data` will be `Some`
            // since the error is `CRITICAL_XID_ERROR`
            Event::CriticalXidError(struct_.device, struct_.event_data.unwrap())
        } else if struct_.event_type.contains(DOUBLE_BIT_ECC_ERROR) {
            Event::DoubleBitEccError(struct_.device)
        } else if struct_.event_type.contains(PSTATE_CHANGE) {
            Event::PowerStateChange(struct_.device)
        } else if struct_.event_type.contains(SINGLE_BIT_ECC_ERROR) {
            Event::SingleBitEccError(struct_.device)
        } else {
            Event::Unknown
        }
    }
}

/// Holds the `EventSet` utilized within an event loop.
///
/// A usage example can be found in the `examples` directory at the root.
// TODO: Example name ^
pub struct EventLoop<'nvml> {
    set: EventSet<'nvml>
}

impl<'nvml> EventLoop<'nvml> {
    /**
    Register another device that this `EventLoop` should receive events for.

    This method takes ownership of this struct and then hands it back to you if
    everything went well with the registration process.

    # Errors
    * `Uninitialized`, if the library has not been successfully initialized
    * `GpuLost`, if a GPU has fallen off the bus or is otherwise inaccessible
    * `Unknown`, on any unexpected error

    # Platform Support
    Only supports Linux.
    */
    #[inline]
    pub fn register_device(mut self, device: &'nvml Device<'nvml>) -> Result<Self> {
        self.set = device.register_events(device.supported_event_types()?, self.set)?;

        Ok(self)
    }

    /**
    Handle events with the given callback until the loop is manually interrupted.

    # Errors
    The function itself does not return anything. You will be given errors to
    handle within your closure if they occur; events are handed to you wrapped
    in a `Result`.

    The errors that you will need to handle are:

    * `Uninitialized`, if the library has not been successfully initialized
    * `GpuLost`, if a GPU has fallen off the bus or is otherwise inaccessible
    * `Unknown`, on any unexpected error

    # Examples
    See the `event_loop` example in the `examples` directory at the root.

    # Platform Support
    Only supports Linux.
    */
    // TODO: example name
    #[inline]
    pub fn run_forever<F>(&mut self, mut callback: F)
    where
        F: FnMut(Result<Event<'nvml>>, &mut EventLoopState),
    {

        let mut state = EventLoopState {
            interrupted: false
        };

        loop {
            if state.interrupted {
                break;
            };

            match self.set.wait(1) {
                Ok(data) => {
                    callback(Ok(data.into()), &mut state);
                },
                Err(Error(ErrorKind::Timeout, _)) => continue,
                value => callback(value.map(|d| d.into()), &mut state),
            };
        }
    }
}

/// Keeps track of whether an `EventLoop` is interrupted or not.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct EventLoopState {
    interrupted: bool
}

impl EventLoopState {
    /// Call this to mark the loop as interrupted.
    #[inline]
    pub fn interrupt(&mut self) {
        self.interrupted = true;
    }
}

/// Adds a method to obtain an `EventLoop` to the `NVML` struct.
///
/// `use` it at your leisure.
pub trait EventLoopProvider {
    // Thanks to Thinkofname for lifetime help, again :)
    fn create_event_loop<'nvml>(
        &'nvml self,
        devices: Vec<&'nvml Device<'nvml>>,
    ) -> Result<EventLoop>;
}

impl EventLoopProvider for NVML {
    /**
    Create an event loop that will register itself to recieve events for the given
    `Device`s.

    This function creates an event set and registers each devices' supported event
    types for it. The returned `EventLoop` struct then has methods that you can
    call to actually utilize it.

    # Errors
    * `Uninitialized`, if the library has not been successfully initialized
    * `GpuLost`, if any of the given `Device`s have fallen off the bus or are
    otherwise inaccessible
    * `Unknown`, on any unexpected error

    # Platform Support
    Only supports Linux.
    */
    #[inline]
    fn create_event_loop<'nvml>(
        &'nvml self,
        devices: Vec<&'nvml Device<'nvml>>,
    ) -> Result<EventLoop> {

        let mut set = self.create_event_set()?;

        for d in devices {
            set = d.register_events(d.supported_event_types()?, set)?;
        }

        Ok(EventLoop {
            set
        })
    }
}
