use std::sync::atomic::AtomicBool;
use enums::event::XidError;
use bitmasks::event::*;
use bitmasks::event::EventTypes;
use struct_wrappers::event::EventData;
use error::*;
use Device;
use NVML;

/// Represents the event types that an `EventLoop` can gather for you.
///
/// These are analagous to the constants in `bitmasks::event`.
pub enum Event<'nvml> {
    ClockChange(Device<'nvml>),
    CriticalXidError(Device<'nvml>, XidError),
    DoubleBitEccError(Device<'nvml>),
    PowerStateChange(Device<'nvml>),
    SingleBitEccError(Device<'nvml>),
    /// If an unsupported event value is encountered, this is returned.
    Unknown,
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

#[derive(Debug)]
pub struct EventLoopState {
    interrupted: bool,
}

impl EventLoopState {
    #[inline]
    pub fn interrupt(&mut self) {
        self.interrupted = true;
    }
}

pub trait EventLoopProvider<'nvml> {
    fn event_loop<F>(&self, nvml: &'nvml NVML, callback: F) -> Result<()>
        where F: FnMut(Result<Event<'nvml>>, &mut EventLoopState);
    // fn poll_events<F>(&self, nvml: &NVML, callback: F) -> Result<()>
    //     where F: FnMut(Result<Event<'nvml>>, &mut EventLoopState);
}

impl<'nvml> EventLoopProvider<'nvml> for Device<'nvml> {
    #[inline]
    fn event_loop<F>(&self, nvml: &'nvml NVML, mut callback: F) -> Result<()>
        where F: FnMut(Result<Event<'nvml>>, &mut EventLoopState) {

        let mut state = EventLoopState{ interrupted: false };

        let set = nvml.create_event_set()?;
        let set = self.register_events(EventTypes::all(), set)?;

        loop {
            if state.interrupted {
                break;
            };

            match set.wait(1) {
                Ok(data) => {
                    callback(Ok(data.into()), &mut state);
                },
                Err(Error(ErrorKind::Timeout, _)) => continue,
                value => callback(value.map(|d| d.into()), &mut state),
            };
        }

        Ok(())
    }  
}
