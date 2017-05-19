use ffi::bindings::*;
use error::*;
use enum_wrappers::*;
use enum_wrappers::nv_link::*;
use Device;
use std::mem;
use std::os::raw::{c_uint, c_ulonglong};

/**
Struct that represents a `Device`'s NvLink.

Obtain this via `Device.link_wrapper_for()`.

Rust's lifetimes will ensure both that the contained `Device` is valid for the
lifetime of the `NvLink` struct and that the `NVML` instance will be valid for
the duration of both.
*/
#[derive(Debug)]
pub struct NvLink<'device, 'nvml: 'device> {
    pub(crate) device: &'device Device<'nvml>,
    pub(crate) link: c_uint,
}

impl<'device, 'nvml: 'device> NvLink<'device, 'nvml> {
    /**
    Gets whether or not this `Device`'s NvLink is active.

    # Errors
    * `Uninitialized`, if the library has not been successfully initialized
    * `InvalidArg`, if the `link` within this `NvLink` struct instance is invalid
    * `NotSupported`, if this `Device` doesn't support this feature
    * `Unknown`, on any unexpected error

    # Device Support
    Supports Maxwell or newer fully supported devices.
    */
    #[inline]
    pub fn is_active(&self) -> Result<bool> {
        unsafe {
            let mut state: nvmlEnableState_t = mem::zeroed();

            nvml_try(nvmlDeviceGetNvLinkState(self.device.unsafe_raw(),
                                              self.link,
                                              &mut state))?;

            Ok(bool_from_state(state))
        }
    }

    /**
    Gets the NvLink version of this `Device` / `NvLink`.

    # Errors
    * `Uninitialized`, if the library has not been successfully initialized
    * `InvalidArg`, if the `link` within this `NvLink` struct instance is invalid
    * `NotSupported`, if this `Device` doesn't support this feature
    * `Unknown`, on any unexpected error

    # Device Support
    Supports Maxwell or newer fully supported devices.
    */
    #[inline]
    pub fn version(&self) -> Result<u32> {
        unsafe {
            let mut version: c_uint = mem::zeroed();

            nvml_try(nvmlDeviceGetNvLinkVersion(self.device.unsafe_raw(),
                                                self.link,
                                                &mut version))?;

            Ok(version)
        }
    }

    /**
    Gets whether or not this `Device` / `NvLink` has a `Capability`.

    # Errors
    * `Uninitialized`, if the library has not been successfully initialized
    * `InvalidArg`, if the `link` within this `NvLink` struct instance is invalid
    * `NotSupported`, if this `Device` doesn't support this feature
    * `Unknown`, on any unexpected error

    # Device Support
    Supports Maxwell or newer fully supported devices.
    */
    #[inline]
    pub fn has_capability(&self, cap_type: Capability) -> Result<bool> {
        unsafe {
            // NVIDIA says that this should be interpreted as a boolean
            let mut capability: c_uint = mem::zeroed();

            nvml_try(nvmlDeviceGetNvLinkCapability(self.device.unsafe_raw(),
                                                   self.link,
                                                   cap_type.as_c(),
                                                   &mut capability))?;

            Ok(match capability {
                0 => false,
                1 => true,
                // Not worth an error, certainly not a panic...
                _ => true,
            })
        }
    }

    // TODO: remotePciInfo

    /**
    Gets the specified `ErrorCounter` value.

    # Errors
    * `Uninitialized`, if the library has not been successfully initialized
    * `InvalidArg`, if the `link` within this `NvLink` struct instance is invalid
    * `NotSupported`, if this `Device` doesn't support this feature
    * `Unknown`, on any unexpected error

    # Device Support
    Supports Maxwell or newer fully supported devices.
    */
    #[inline]
    pub fn error_counter(&self, counter: ErrorCounter) -> Result<u64> {
        unsafe {
            let mut value: c_ulonglong = mem::zeroed();

            nvml_try(nvmlDeviceGetNvLinkErrorCounter(self.device.unsafe_raw(),
                                                     self.link,
                                                     counter.as_c(),
                                                     &mut value))?;

            Ok(value)
        }
    }
}
