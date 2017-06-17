extern crate nvml_wrapper as nvml;

// Don't mind any of this; just making sure that this example prints a helpful
// message on platforms other than Linux, where it is not supported

#[cfg(target_os = "linux")]
fn main() {
    println!("{:?}", example::actual_main());
}

#[cfg(target_os = "windows")]
fn main() {
    println!(
        "Whoops! The event_loop sample only works on Linux (NVML does not support events on any \
         other platform)."
    );
}

// The part that concerns you starts below.

#[cfg(target_os = "linux")]
mod example {
    use nvml::NVML;
    // You may want to use your own error-chain setup in your own code
    use nvml::error::{Error, ErrorKind, Result};
    // Bringing this in allows us to use `NVML.create_event_loop()`
    use nvml::high_level::EventLoopProvider;
    // Bringing these in for brevity (Event::SomeEvent vs. SomeEvent)
    use nvml::high_level::Event::*;

    // We write a function so that we can return a `Result` and use `?`
    pub fn actual_main() -> Result<()> {
        let nvml = NVML::init()?;
        let device = nvml.device_by_index(0)?;

        // Create an event loop, registering the single device we obtained above
        let mut event_loop = nvml.create_event_loop(vec![&device])?;

        // Start handling events
        event_loop.run_forever(|event, state| match event {
            // If there were no errors, extract the event
            Ok(event) => match event {
                ClockChange(device) => if let Ok(uuid) = device.uuid() { 
                    println!("ClockChange      event for device with UUID {:?}", uuid);
                } else {
                    // Your error-handling strategy here
                },

                PowerStateChange(device) => if let Ok(uuid) = device.uuid() { 
                    println!("PowerStateChange event for device with UUID {:?}", uuid);
                } else {
                    // Your error-handling strategy here
                },

                _ => println!("A different event occurred: {:?}", event)
            },

            // If there was an error, handle it
            Err(Error(error, _)) => match error {
                // If the error is `Unknown`, continue looping and hope for the best
                ErrorKind::Unknown => {},
                // The other errors that can occur are almost guaranteed to mean that
                // further looping will never be successful (`GpuLost` and
                // `Uninitialized`), so we stop looping
                _ => state.interrupt()
            }
        });

        Ok(())
    }
}
