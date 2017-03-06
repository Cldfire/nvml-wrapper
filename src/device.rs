use ffi::*;
use super::errors::*;
use super::structs::AutoBoostClocksEnabledInfo;
use super::struct_wrappers::PciInfo;
use super::enum_wrappers::*;
use std::marker::PhantomData;
use std::ffi::CStr;
use std::mem;
use std::os::raw::c_uint;
use NVML;

// TODO: Investigate #[inline] and find out whether or not I should use it.

/// Struct that represents a device on the system. 
///
/// Obtain a `Device` with the various functions available to you on the `NVML`
/// struct.
/// 
/// Rust's lifetimes will ensure that the NVML instance this `Device` was created from is
/// not allowed to be shutdown until this `Device` is dropped, meaning you shouldn't
/// have to worry about calls returning `Uninitialized` errors. 
// TODO: Use compiletest to ensure lifetime guarantees
pub struct Device<'nvml> {
    device: nvmlDevice_t,
    _phantom: PhantomData<&'nvml NVML>,
}

impl<'nvml> Device<'nvml> {
    #[doc(hidden)]
    pub fn _new(device: nvmlDevice_t) -> Self {
        Device {
            device: device,
            _phantom: PhantomData,
        }
    }

    /// Clear all affinity bindings for the calling thread.
    ///
    /// Note that this was changed as of version 8.0; older versions cleared affinity for 
    /// the calling process and all children. 
    ///
    /// # Errors
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `InvalidArg`, if the device is invalid (this shouldn't ever be the case?)
    /// * `Unknown`, on any unexpected error
    /// 
    /// That's it according to NVIDIA's docs. No clue why GPU_IS_LOST and NOT_SUPPORTED
    /// are not mentioned. I would recommend planning for those as well, I've seen other 
    /// mistakes in the errors listed by their docs. 
    ///
    /// # Platform Support
    /// Only supports Linux. 
    ///
    /// # Device Support
    /// Supports Kepler or newer fully supported devices.
    // TODO: Figure out how to test this on platforms it supports
    // Checked against local nvml.h
    pub fn clear_cpu_affinity(&self) -> Result<()> {
        unsafe {
            nvml_try(nvmlDeviceClearCpuAffinity(self.device)) 
        }
    }

    /// Gets the root/admin permissions for the target API.
    ///
    /// Only root users are able to call functions belonging to restricted APIs. See 
    /// the documentation for the `RestrictedApi` enum for a list of those functions.
    // TODO: Document how to change perms
    ///
    /// # Errors
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `InvalidArg`, if the device is invalid (this shouldn't ever be the case?) or the 
    /// apiType is invalid (may occur if C lib changes dramatically?)
    /// * `NotSupported`, if this query is not supported by this `Device` or this `Device`
    /// does not support the feature that is being queried (e.g. enabling/disabling auto
    /// boosted clocks is not supported by this `Device`).
    /// * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    /// * `Unknown`, on any unexpected error
    ///
    /// # Device Support
    /// Supports all _fully supported_ products.
    // TODO: Figure out how to test this on platforms it supports
    // Checked against local nvml.h
    pub fn is_api_restricted(&self, api: RestrictedApi) -> Result<bool> {
        unsafe {
            let mut restricted_state: nvmlEnableState_t = mem::zeroed();
            match nvml_try(nvmlDeviceGetAPIRestriction(self.device, api.eq_c_variant(), &mut restricted_state)) {
                Ok(()) => match restricted_state {
                    nvmlEnableState_enum::NVML_FEATURE_ENABLED
                        => Ok(true),
                    nvmlEnableState_enum::NVML_FEATURE_DISABLED
                        => Ok(false),
                },
                Err(e) => Err(e),
            }
        }
    }

    /// Gets the current clock setting that all applications will use unless an overspec 
    /// situation occurs.
    ///
    /// This setting can be changed using `.set_applications_clocks()`.
    // TODO: Check that name is correct after I write the method ^
    ///
    /// # Errors 
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `InvalidArg`, if the device is invalid (this shouldn't ever be the case?) or the 
    /// clockType is invalid (may occur if C lib changes dramatically?)
    /// * `NotSupported`, if this `Device` does not support this feature
    /// * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    /// * `Unknown`, on any unexpected error
    ///
    /// # Device Support
    /// Supports Kepler or newer fully supported devices.
    // TODO: Figure out how to test this on platforms it supports
    // Checked against local nvml.h
    pub fn applications_clock(&self, clock_type: ClockType) -> Result<u32> {
        unsafe {
            let mut clock: c_uint = mem::zeroed();
            match nvml_try(nvmlDeviceGetApplicationsClock(self.device, clock_type.eq_c_variant(), &mut clock)) {
                Ok(()) => Ok(clock as u32),
                Err(e) => Err(e),
            }
        }
    }

    /// Gets the current state and default state of auto boosted clocks.
    ///
    /// Auto boosted clocks are enabled by default on some hardware, allowing the GPU to run
    /// as fast as thermals will allow it to. 
    ///
    /// On Pascal and newer hardware, auto boosted clocks are controlled through application
    /// clocks. Use `.set_applications_clocks()` and `.reset_applications_clocks()` to control
    /// auto boost behavior.
    // TODO: Check these method names after I write them ^
    /// 
    /// # Errors 
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `InvalidArg`, if the device is invalid (this shouldn't ever be the case?)
    /// * `NotSupported`, if this `Device` does not support auto boosted clocks
    /// * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    /// * `Unknown`, on any unexpected error
    ///
    /// # Device Support
    /// Supports Kepler or newer fully supported devices.
    // TODO: Figure out how to test this on platforms it supports
    // Checked against local nvml.h
    pub fn auto_boosted_clocks_enabled(&self) -> Result<AutoBoostClocksEnabledInfo> {
        unsafe {
            let mut is_enabled: nvmlEnableState_t = mem::zeroed();
            let mut is_enabled_default: nvmlEnableState_t = mem::zeroed();
            match nvml_try(nvmlDeviceGetAutoBoostedClocksEnabled(self.device, &mut is_enabled, &mut is_enabled_default)) {
                Ok(()) => {
                    let is_enabled = match is_enabled {
                        nvmlEnableState_enum::NVML_FEATURE_ENABLED
                            => true,
                        nvmlEnableState_enum::NVML_FEATURE_DISABLED
                            => false,
                    };

                    let is_enabled_default = match is_enabled_default {
                        nvmlEnableState_enum::NVML_FEATURE_ENABLED
                            => true,
                        nvmlEnableState_enum::NVML_FEATURE_DISABLED
                            => false,
                    };

                    Ok(AutoBoostClocksEnabledInfo{ is_enabled: is_enabled, is_enabled_default: is_enabled_default })
                },
                Err(e) => Err(e),
            }
        }
    }

    /// Gets the PCI attributes of this `Device`.
    /// 
    /// See `PciInfo` for details about the returned attributes.
    ///
    /// # Errors
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `InvalidArg`, if the device is invalid (this shouldn't ever be the case?)
    /// * `GpuLost`, if the GPU has fallen off the bus or is otherwise inaccessible
    /// * `Utf8Error`, if a string obtained from the C function is not valid Utf8
    /// * `Unknown`, on any unexpected error
    // Checked against local nvml.h
    pub fn pci_info(&self) -> Result<PciInfo> {
        unsafe {
            let mut pci_info: nvmlPciInfo_t = mem::zeroed();
            match nvml_try(nvmlDeviceGetPciInfo_v2(self.device, &mut pci_info)) {
                Ok(()) => {
                    let bus_id_raw = CStr::from_ptr(pci_info.busId.as_ptr());
                    Ok(PciInfo {
                        bus: pci_info.bus as u32,
                        bus_id: bus_id_raw.to_str()?.into(),
                        device: pci_info.device as u32,
                        domain: pci_info.domain as u32,
                        pci_device_id: pci_info.pciDeviceId as u32,
                        pci_sub_system_id: pci_info.pciSubSystemId as u32,
                    })
                },
                Err(e) => Err(e)
            }
        }
    }

    /// Gets the NVML index of this `Device`. 
    /// 
    /// Keep in mind that the order in which NVML enumerates devices has no guarantees of
    /// consistency between reboots. Also, the NVML index may not correlate with other APIs,
    /// such as the CUDA device index.
    ///
    /// # Errors 
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `InvalidArg`, if the device is invalid (this shouldn't ever be the case?)
    /// * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    // Checked against local nvml.h 
    pub fn index(&self) -> Result<u32> {
        unsafe {
            let mut index: c_uint = mem::zeroed();
            match nvml_try(nvmlDeviceGetIndex(self.device, &mut index)) {
                Ok(()) => Ok(index as u32),
                Err(e) => Err(e),
            }
        }
    }

    /// The name of this `Device`, e.g. "Tesla C2070".
    ///
    /// The name is an alphanumeric string that denotes a particular product. 
    ///
    /// # Errors
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `InvalidArg`, if the device is invalid (this shouldn't ever be the case?)
    /// * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    /// * `Utf8Error`, if the string obtained from the C function is not valid Utf8
    /// * `Unknown`, on any unexpected error
    // Checked against local nvml.h
    pub fn name(&self) -> Result<String> {
        unsafe {
            let mut name_vec = Vec::with_capacity(NVML_DEVICE_NAME_BUFFER_SIZE as usize);
            match nvml_try(nvmlDeviceGetName(self.device, name_vec.as_mut_ptr(), NVML_DEVICE_NAME_BUFFER_SIZE)) {
                Ok(()) => {
                    let name_raw = CStr::from_ptr(name_vec.as_ptr());
                    Ok(name_raw.to_str()?.into())
                }, 
                Err(e) => Err(e),
            }
        }
    }

    /// Identifies whether or not the `Device` is on a multi-GPU board.
    ///
    /// # Errors
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `InvalidArg`, if the device is invalid (this shouldn't ever be the case?)
    /// * `NotSupported`, if the `Device` does not support this feature
    /// * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    /// * `Unknown`, on any unexpected error
    ///
    /// # Device Support
    /// Supports Fermi or newer fully supported devices.
    // TODO: Figure out how to test this on platforms it supports
    // Checked against local nvml.h
    pub fn is_multi_gpu_board(&self) -> Result<bool> {
        unsafe {
            let mut int_bool: c_uint = mem::zeroed();
            match nvml_try(nvmlDeviceGetMultiGpuBoard(self.device, &mut int_bool)) {
                Ok(()) => {
                    match int_bool as u32 {
                        0 => Ok(false),
                        _ => Ok(true),
                    }
                },
                Err(e) => Err(e),
            }
        }
    }
}

#[cfg(feature = "test")]
#[cfg(feature = "test-local")]
#[allow(unused_variables, unused_imports)]
mod test {
    use super::*;

    // TODO: Gen tests for EVERYTHING IN THIS FILE (!!!!!!!!)

    // TODO: Gen tests for pci_info
    #[test]
    fn pci_info() {
        let test = NVML::init().expect("init call failed");
        let device = test.device_by_index(0).expect("Could not get a device by index 0");
        let pci_info = device.pci_info().expect("Could not get pci info");
    }

    // TODO: Gen tests for index
    #[test]
    fn index() {
        let test = NVML::init().expect("init call failed");
        let device = test.device_by_index(0).expect("Could not get a device by index 0");
        let index = device.index().expect("Could not get device index");
    }

    // TODO: Gen tests for name
    #[test]
    fn name() {
        let test = NVML::init().expect("init call failed");
        let device = test.device_by_index(0).expect("Could not get a device by index 0");
        let name = device.name().expect("Could not get device name");
    }

    #[test]
    fn applications_clock() {
        let test = NVML::init().expect("init call failed");
        let device = test.device_by_index(0).expect("Could not get a device by index 0");
        let clock = device.applications_clock(ClockType::Graphics).expect("Could not get applications clock");
    }
}