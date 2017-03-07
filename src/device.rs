use ffi::*;
use super::errors::*;
use super::structs::*;
use super::struct_wrappers::*;
use super::enum_wrappers::*;
use std::marker::PhantomData;
use std::ffi::CStr;
use std::mem;
use std::os::raw::c_uint;
use NVML;

// TODO: Investigate #[inline] and find out whether or not I should use it.
// TODO: Mark stuff that works on my 980 Ti but NVIDIA does not state should.

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
    /// * `InvalidArg`, if the device is invalid
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
    /// * `InvalidArg`, if the device is invalid or the apiType is invalid (may occur if 
    /// the C lib changes dramatically?)
    /// * `NotSupported`, if this query is not supported by this `Device` or this `Device`
    /// does not support the feature that is being queried (e.g. enabling/disabling auto
    /// boosted clocks is not supported by this `Device`).
    /// * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    /// * `Unknown`, on any unexpected error
    ///
    /// # Device Support
    /// Supports all _fully supported_ products.
    // TODO: Figure out how to test this on platforms it supports
    // TODO: Make sure there's a test case for when an error is returned and the mem::zeroed() values may be dropped
    // Checked against local nvml.h
    pub fn is_api_restricted(&self, api: Api) -> Result<bool> {
        unsafe {
            let mut restricted_state: nvmlEnableState_t = mem::zeroed();
            nvml_try(nvmlDeviceGetAPIRestriction(self.device, api.eq_c_variant(), &mut restricted_state))?;

            match restricted_state {
                nvmlEnableState_enum::NVML_FEATURE_ENABLED
                    => Ok(true),
                nvmlEnableState_enum::NVML_FEATURE_DISABLED
                    => Ok(false),
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
    /// * `InvalidArg`, if the device is invalid or the clockType is invalid (may occur 
    /// if the C lib changes dramatically?)
    /// * `NotSupported`, if this `Device` does not support this feature
    /// * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    /// * `Unknown`, on any unexpected error
    ///
    /// # Device Support
    /// Supports Kepler or newer fully supported devices.
    // TODO: Figure out how to test this on platforms it supports
    // Checked against local nvml.h
    pub fn applications_clock(&self, clock_type: Clock) -> Result<u32> {
        unsafe {
            let mut clock: c_uint = mem::zeroed();
            nvml_try(nvmlDeviceGetApplicationsClock(self.device, clock_type.eq_c_variant(), &mut clock))?;

            Ok(clock as u32)
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
    /// * `InvalidArg`, if the device is invalid
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
            nvml_try(nvmlDeviceGetAutoBoostedClocksEnabled(self.device, &mut is_enabled, &mut is_enabled_default))?;

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
        }
    }

    /// Gets the total, available and used size of BAR1 memory. 
    ///
    /// BAR1 memory is used to map the FB (device memory) so that it can be directly accessed
    /// by the CPU or by 3rd party devices (peer-to-peer on the PCIe bus).
    ///
    /// # Errors 
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `InvalidArg`, if the device is invalid
    /// * `NotSupported`, if this `Device` does not support this query
    /// * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    /// * `Unknown`, on any unexpected error
    ///
    /// # Device Support
    /// Supports Kepler or newer fully supported devices.
    // Checked against local nvml.h
    pub fn bar1_memory_info(&self) -> Result<BAR1MemoryInfo> {
        unsafe {
            let mut mem_info: nvmlBAR1Memory_t = mem::zeroed();
            nvml_try(nvmlDeviceGetBAR1MemoryInfo(self.device, &mut mem_info))?;

            Ok(mem_info.into())
        }
    }

    /// Gets the board ID for this `Device`, from 0-N. 
    ///
    /// Devices with the same boardID indicate GPUs connected to the same PLX. Use in
    /// conjunction with `.is_multi_gpu_board()` to determine if they are on the same
    /// board as well. 
    // TODO: Check that when I write it ^
    ///
    /// The boardID returned is a unique ID for the current config. Uniqueness and
    /// ordering across reboots and system configs is not guaranteed (i.e if a Tesla
    /// K40c returns 0x100 and the two GPUs on a Tesla K10 in the same system return
    /// 0x200, it is not guaranteed that they will always return those values. They will,
    /// however, always be different from each other).
    ///
    /// # Errors
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `InvalidArg`, if the device is invalid
    /// * `NotSupported`, if this `Device` does not support this feature
    /// * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    /// * `Unknown`, on any unexpected error
    ///
    /// # Device Support
    /// Supports Fermi or newer fully supported devices.
    // Checked against local nvml.h
    pub fn board_id(&self) -> Result<u32> {
        unsafe {
            let mut id: c_uint = mem::zeroed();
            nvml_try(nvmlDeviceGetBoardId(self.device, &mut id))?;

            Ok(id as u32)
        }
    }
    
    /// Gets the brand of this `Device`.
    ///
    /// See the `Brand` enum for documentation of possible values.
    ///
    /// # Errors
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `InvalidArg`, if the device is invalid
    /// * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    /// * `UnexpectedVariant`, check that error's docs for more info
    /// * `Unknown`, on any unexpected error
    // Checked against local nvml.h
    pub fn brand(&self) -> Result<Brand> {
        unsafe {
            let mut brand: nvmlBrandType_t = mem::zeroed();
            nvml_try(nvmlDeviceGetBrand(self.device, &mut brand))?;

            Ok(Brand::try_from(brand)?)
        }
    }

    /// Gets bridge chip information for all bridge chips on the board. 
    ///
    /// Only applicable to multi-GPU devices.
    ///
    /// # Errors
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `InvalidArg`, if the device is invalid
    /// * `NotSupported`, if this `Device` does not support this feature
    /// * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    /// * `Unknown`, on any unexpected error
    ///
    /// # Device Support
    /// Supports all _fully supported_ devices.
    // Checked against local nvml.h
    pub fn bridge_chip_info(&self) -> Result<BridgeChipHierarchy> {
        unsafe {
            let mut info: nvmlBridgeChipHierarchy_t = mem::zeroed();
            nvml_try(nvmlDeviceGetBridgeChipInfo(self.device, &mut info))?;

            Ok(BridgeChipHierarchy::from(info))
        }
    }

    /// Gets this `Device`'s current clock speed for the given `Clock` type.
    ///
    /// # Errors
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `InvalidArg`, if the device is invalid
    /// * `NotSupported`, if this `Device` cannot report the specified clock
    /// * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    /// * `Unknown`, on any unexpected error
    ///
    /// # Device Support
    /// Supports Kepler or newer fully supported devices.
    // Checked against local nvml.h
    // TODO: Uh... doesn't appear to do what it says? Investigate?
    pub fn clock_info(&self, clock_type: Clock) -> Result<u32> {
        unsafe {
            let mut clock: c_uint = mem::zeroed();
            nvml_try(nvmlDeviceGetClockInfo(self.device, clock_type.eq_c_variant(), &mut clock))?;

            Ok(clock as u32)
        }
    }

    // This is in progress.
    // pub fn running_compute_processes(&self) {
    //     unsafe {
    //         let mut info_array: *mut nvmlProcessInfo_t = ::std::ptr::null_mut();
    //         let mut count: c_uint = 0;
    //         nvml_try(nvmlDeviceGetComputeRunningProcesses(self.device, &mut count, info_array)).expect("Test failed");

    //         println!("{:?}", count as u32);
    //     }
    // }



    /// Gets the PCI attributes of this `Device`.
    /// 
    /// See `PciInfo` for details about the returned attributes.
    ///
    /// # Errors
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `InvalidArg`, if the device is invalid
    /// * `GpuLost`, if the GPU has fallen off the bus or is otherwise inaccessible
    /// * `Utf8Error`, if a string obtained from the C function is not valid Utf8
    /// * `Unknown`, on any unexpected error
    // Checked against local nvml.h
    pub fn pci_info(&self) -> Result<PciInfo> {
        unsafe {
            let mut pci_info: nvmlPciInfo_t = mem::zeroed();
            nvml_try(nvmlDeviceGetPciInfo_v2(self.device, &mut pci_info))?;

            Ok(PciInfo::try_from(pci_info)?)
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
    /// * `InvalidArg`, if the device is invalid
    /// * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    // Checked against local nvml.h 
    pub fn index(&self) -> Result<u32> {
        unsafe {
            let mut index: c_uint = mem::zeroed();
            nvml_try(nvmlDeviceGetIndex(self.device, &mut index))?;

            Ok(index as u32)
        }
    }

    /// The name of this `Device`, e.g. "Tesla C2070".
    ///
    /// The name is an alphanumeric string that denotes a particular product. 
    ///
    /// # Errors
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `InvalidArg`, if the device is invalid
    /// * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    /// * `Utf8Error`, if the string obtained from the C function is not valid Utf8
    /// * `Unknown`, on any unexpected error
    // Checked against local nvml.h
    pub fn name(&self) -> Result<String> {
        unsafe {
            let mut name_vec = Vec::with_capacity(NVML_DEVICE_NAME_BUFFER_SIZE as usize);
            nvml_try(nvmlDeviceGetName(self.device, name_vec.as_mut_ptr(), NVML_DEVICE_NAME_BUFFER_SIZE))?;

            let name_raw = CStr::from_ptr(name_vec.as_ptr());
            Ok(name_raw.to_str()?.into())
        }
    }

    /// Identifies whether or not the `Device` is on a multi-GPU board.
    ///
    /// # Errors
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `InvalidArg`, if the device is invalid
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
            nvml_try(nvmlDeviceGetMultiGpuBoard(self.device, &mut int_bool))?;

            match int_bool as u32 {
                0 => Ok(false),
                _ => Ok(true),
            }
        }
    }
}

#[cfg(feature = "test")]
#[cfg(feature = "test-local")]
#[allow(unused_variables, unused_imports)]
mod test {
    use super::*;

    // In progress
    // #[test]
    // fn test_thing() {
    //     let test = NVML::init().expect("init call failed");
    //     let device = test.device_by_index(0).expect("Could not get a device by index 0");

    //     device.running_compute_processes();
    // }

    // TODO: Look into generating tests via proc macros

    #[test]
    fn clock() {
        let test = NVML::init().expect("init call failed");
        let device = test.device_by_index(0).expect("Could not get a device by index 0");
        let gfx_clock = device.clock_info(Clock::Graphics);
        let mem_clock = device.clock_info(Clock::Memory);
        let sm_clock = device.clock_info(Clock::SM);
        let vid_clock = device.clock_info(Clock::Video);

        println!("{:?} MHz, {:?} MHz, {:?} MHz, {:?} MHz", gfx_clock, mem_clock, sm_clock, vid_clock);
    }

    #[ignore]
    #[test]
    // TODO: This is not supported for my GPU
    fn is_api_restricted() {
        let test = NVML::init().expect("init call failed");
        let device = test.device_by_index(0).expect("Could not get a device by index 0");
        let is_restricted = device.is_api_restricted(Api::ApplicationClocks)
            .expect("Failed to check ApplicationClocks");
        let is_restricted2 = device.is_api_restricted(Api::AutoBoostedClocks)
            .expect("Failed to check AutoBoostedClocks");
    }

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
        let clock = device.applications_clock(Clock::Graphics).expect("Could not get applications clock");
    }
}