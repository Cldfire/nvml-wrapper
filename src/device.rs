use ffi::*;
use super::nvml_errors::*;
use super::structs::*;
use super::struct_wrappers::*;
use super::enum_wrappers::*;
use std::marker::PhantomData;
use std::ffi::CStr;
use std::mem;
use std::os::raw::{c_uint, c_ulong, c_ulonglong};
use std::slice;
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

unsafe impl<'nvml> Send for Device<'nvml> {}
unsafe impl<'nvml> Sync for Device<'nvml> {}

impl<'nvml> Device<'nvml> {
    #[doc(hidden)]
    pub fn new(device: nvmlDevice_t) -> Self {
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
    #[cfg(target_os = "linux")]
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

            Ok(AutoBoostClocksEnabledInfo{ is_enabled: bool_from_state(is_enabled), 
                                           is_enabled_default: bool_from_state(is_enabled_default) })
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

    /// Gets information about processes with a compute context running on this `Device`,
    /// allocating `size` amount of space.
    ///
    /// This only returns information about running compute processes (such as a CUDA application
    /// with an active context). Graphics applications (OpenGL, DirectX) won't be listed by this
    /// function.
    ///
    /// Keep in mind that information returned by this call is dynamic and the number of elements
    /// may change over time. Allocate more space for information in case new compute processes
    /// are spawned.
    ///
    /// NVIDIA doesn't say anything about compute shaders causing a process to show up here.
    // TODO: Docs and function need work, not sure if what I'm doing is even safe or correct
    // TODO: Handle passing 0 to just query with enum
    ///
    /// # Errors
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `InsufficientSize`, TODO: This
    /// * `InvalidArg`, if the device is invalid
    /// * `NotSupported`, if this `Device` TODO: and this
    /// * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    /// * `Unknown`, on any unexpected error
    // TODO: And, handle INSUFFICIENT_SIZE infocount being size needed to fill array... lots of todo here
    pub fn running_compute_processes(&self, size: usize) -> Result<Vec<ProcessInfo>> {
        unsafe {
            let mut first_item: nvmlProcessInfo_t = mem::zeroed();
            // Passed in to designate size of returned array and after call is count of returned elements
            let mut count: c_uint = size as c_uint;
            nvml_try(nvmlDeviceGetComputeRunningProcesses(self.device, &mut count, &mut first_item))?;

            // TODO: Is passing a reference to `first_item` safe? Am I doing this right?
            let array = slice::from_raw_parts(&first_item as *const nvmlProcessInfo_t, count as usize);
            let vec = array.iter()
                           .map(|info| ProcessInfo::from(*info))
                           .collect();

            Ok(vec)
        }
    }

    /// Gets a `Vec` of bitmasks with the ideal CPU affinity for the device.
    ///
    /// For example, if processors 0, 1, 32, and 33 are ideal for the device and `size` == 2,
    /// result[0] = 0x3, result[1] = 0x3.
    ///
    /// # Errors
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `InvalidArg`, if the device is invalid or `size` is 0
    /// * `NotSupported`, if this `Device` does not support this feature
    /// * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    /// * `Unknown`, on any unexpected error
    ///
    /// # Device Support
    /// Supports Kepler or newer fully supported devices.
    ///
    /// # Platform Support
    /// Only supports Linux.
    #[cfg(target_os = "linux")]
    pub fn cpu_affinity(&self, size: usize) -> Result<Vec<u64>> {
        unsafe {
            let mut first_item: c_ulong = mem::zeroed();
            nvml_try(nvmlDeviceGetCpuAffinity(self.device, size as c_uint, &mut first_item))?;

            // TODO: same as running_compute_processes, is this safe
            let array = slice::from_raw_parts(&first_item as *const c_ulong, size);
            Ok(array.to_vec())
        }
    }

    /// Gets the current PCIe link generation.
    ///
    /// # Errors
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `InvalidArg`, if the device is invalid
    /// * `NotSupported`, if PCIe link information is not available
    /// * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    /// * `Unknown`, on any unexpected error
    ///
    /// # Device Support
    /// Supports Fermi or newer fully supported devices.
    pub fn current_pcie_link_gen(&self) -> Result<u32> {
        unsafe {
            let mut link_gen: c_uint = mem::zeroed();
            nvml_try(nvmlDeviceGetCurrPcieLinkGeneration(self.device, &mut link_gen))?;

            Ok(link_gen as u32)
        }
    }

    /// Gets the current PCIe link width.
    ///
    /// # Errors
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `InvalidArg`, if the device is invalid
    /// * `NotSupported`, if PCIe link information is not available
    /// * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    /// * `Unknown`, on any unexpected error
    ///
    /// # Device Support
    /// Supports Fermi or newer fully supported devices.
    pub fn current_pcie_link_width(&self) -> Result<u32> {
        unsafe {
            let mut link_width: c_uint = mem::zeroed();
            nvml_try(nvmlDeviceGetCurrPcieLinkWidth(self.device, &mut link_width))?;

            Ok(link_width as u32)
        }
    }

    // TODO: GetCurrentClocksThrottleReasons. It returns a bitmask and I've never worked with those

    /// Gets the current utilization and sampling size (sampling size in μs) for the Decoder.
    ///
    /// # Errors
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `InvalidArg`, if the device is invalid
    /// * `NotSupported`, if this `Device` does not support this feature
    /// * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    /// * `Unknown`, on any unexpected error
    ///
    /// # Device Support
    /// Supports Kepler or newer fully supported devices.
    pub fn decoder_utilization(&self) -> Result<DecoderUtilizationInfo> {
        unsafe {
            let mut utilization: c_uint = mem::zeroed();
            let mut sampling_period: c_uint = mem::zeroed();
            nvml_try(nvmlDeviceGetDecoderUtilization(self.device, &mut utilization, &mut sampling_period))?;

            Ok(DecoderUtilizationInfo {
                utilization: utilization as u32,
                sampling_period: sampling_period as u32,
            })
        }
    }

    /// Gets the default applications clock that this `Device` boots with or defaults to after
    /// `reset_applications_clocks()`.
    ///
    /// # Errors
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `InvalidArg`, if the device is invalid
    /// * `NotSupported`, if this `Device` does not support this feature
    /// * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    /// * `Unknown`, on any unexpected error
    ///
    /// # Device Support
    /// Supports Kepler or newer fully supported devices.
    pub fn default_applications_clock(&self, clock_type: Clock) -> Result<u32> {
        unsafe {
            let mut clock: c_uint = mem::zeroed();
            nvml_try(nvmlDeviceGetDefaultApplicationsClock(self.device, clock_type.eq_c_variant(), &mut clock))?;

            Ok(clock as u32)
        }
    }

    /// Not documenting this because it's deprecated. Read NVIDIA's docs if you must use it.
    #[deprecated(note="use `Device.memory_error_counter()`")]
    pub fn detailed_ecc_errors(&self, error_type: MemoryError, counter_type: EccCounter) -> Result<EccErrorCounts> {
        unsafe {
            let mut counts: nvmlEccErrorCounts_t = mem::zeroed();
            nvml_try(nvmlDeviceGetDetailedEccErrors(self.device, 
                                                    error_type.eq_c_variant(), 
                                                    counter_type.eq_c_variant(), 
                                                    &mut counts))?;

            Ok(counts.into())
        }
    }

    /// Gets the display active state for the device. 
    ///
    /// This method indicates whether a display is initialized on this `Device`.
    /// For example, whether or not an X Server is attached to this device and
    /// has allocated memory for the screen.
    ///
    /// # Errors
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `InvalidArg`, if the device is invalid
    /// * `NotSupported`, if this `Device` does not support this feature
    /// * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    /// * `Unknown`, on any unexpected error
    pub fn is_display_active(&self) -> Result<bool> {
        unsafe {
            let mut state: nvmlEnableState_t = mem::zeroed();
            nvml_try(nvmlDeviceGetDisplayActive(self.device, &mut state))?;

            Ok(bool_from_state(state))
        }
    }

    /// Gets whether a physical display is currently connected to any of this `Device`'s
    /// connectors.
    ///
    /// # Errors
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `InvalidArg`, if the device is invalid
    /// * `NotSupported`, if this `Device` does not support this feature
    /// * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    /// * `Unknown`, on any unexpected error
    pub fn is_display_connected(&self) -> Result<bool> {
        unsafe {
            let mut state: nvmlEnableState_t = mem::zeroed();
            nvml_try(nvmlDeviceGetDisplayMode(self.device, &mut state))?;

            Ok(bool_from_state(state))
        }
    }

    /// Gets the current and pending driver model for this `Device`.
    ///
    /// On Windows, the device driver can run in either WDDM or WDM (TCC) modes.
    /// If a display is attached to the device it must run in WDDM mode. TCC mode
    /// is preferred if a display is not attached.
    ///
    /// # Errors
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `InvalidArg`, if the device is invalid
    /// * `NotSupported`, if the platform is not Windows
    /// * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    /// * `Unknown`, on any unexpected error
    ///
    /// # Device Support
    /// Supports Fermi and newer fully supported devices.
    ///
    /// # Platform Support
    /// Only supports Windows.
    #[cfg(target_os = "windows")]
    pub fn driver_model(&self) -> Result<DriverModels> {
        unsafe {
            let current: nvmlDriverModel_t = mem::zeroed();
            let pending: nvmlDriverModel_t = mem::zeroed();
            nvml_try(nvmlDeviceGetDriverModel(self.device, &mut current, &mut pending))?;

            Ok(DriverModels{ current: current.into(), pending: pending.into() })
        }
    }

    /// Get the current and pending ECC modes for the device.
    ///
    /// Changing ECC modes requires a reboot. The "pending" ECC mode refers to the target
    /// mode following the next reboot.
    ///
    /// # Errors
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `InvalidArg`, if the device is invalid
    /// * `NotSupported`, if this `Device` does not support this feature
    /// * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    /// * `Unknown`, on any unexpected error
    ///
    /// # Device Support
    /// Supports Fermi and newer fully supported devices. Only applicable to devices with
    /// ECC. Requires NVML_INFOROM_ECC version 1.0 or higher.
    // TODO: Expose that somehow? ^
    pub fn is_ecc_enabled(&self) -> Result<EccModeInfo> {
        unsafe {
            let mut current: nvmlEnableState_t = mem::zeroed();
            let mut pending: nvmlEnableState_t = mem::zeroed();
            nvml_try(nvmlDeviceGetEccMode(self.device, &mut current, &mut pending))?;

            Ok(EccModeInfo{ currently_enabled: bool_from_state(current), 
                            pending_enabled: bool_from_state(pending) })
        }
    }

    /// Gets the current utilization and sampling size (sampling size in μs) for the Encoder.
    ///
    /// # Errors
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `InvalidArg`, if the device is invalid
    /// * `NotSupported`, if this `Device` does not support this feature
    /// * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    /// * `Unknown`, on any unexpected error
    ///
    /// # Device Support
    /// Supports Kepler or newer fully supported devices.
    pub fn encoder_utilization(&self) -> Result<EncoderUtilizationInfo> {
        unsafe {
            let mut utilization: c_uint = mem::zeroed();
            let mut sampling_period: c_uint = mem::zeroed();
            nvml_try(nvmlDeviceGetEncoderUtilization(self.device, &mut utilization, &mut sampling_period))?;

            Ok(EncoderUtilizationInfo{ utilization: utilization as u32, 
                                       sampling_period: sampling_period as u32 })
        }
    }

    /// Gets the effective power limit in milliwatts that the driver enforces after taking
    /// into account all limiters.
    ///
    /// Note: This can be different from the `.power_management_limit()` if other limits
    /// are set elswhere. This includes the out-of-band power limit interface.
    ///
    /// # Errors
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `InvalidArg`, if the device is invalid
    /// * `NotSupported`, if this `Device` does not support this feature
    /// * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    /// * `Unknown`, on any unexpected error
    ///
    /// # Device Support
    /// Supports Kepler or newer fully supported devices.
    pub fn enforced_power_limit(&self) -> Result<u32> {
        unsafe {
            let mut limit: c_uint = mem::zeroed();
            nvml_try(nvmlDeviceGetEnforcedPowerLimit(self.device, &mut limit))?;

            Ok(limit as u32)
        }
    }

    /// Gets the intended operating speed of this `Device`'s fan as a percentage of the
    /// maximum fan speed.
    ///
    /// Note: The reported speed is the intended fan speed. If the fan is physically blocked
    /// and unable to spin, the output will not match the actual fan speed.
    ///
    /// # Errors
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `InvalidArg`, if the device is invalid
    /// * `NotSupported`, if this `Device` does not have a fan
    /// * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    /// * `Unknown`, on any unexpected error
    ///
    /// # Device Support
    /// Supports all discrete products with dedicated fans.
    pub fn fan_speed(&self) -> Result<u32> {
        unsafe {
            let mut speed: c_uint = mem::zeroed();
            nvml_try(nvmlDeviceGetFanSpeed(self.device, &mut speed))?;

            Ok(speed as u32)
        }
    }

    /// Gets the current GPU operation mode and the pending one (that it will switch to
    /// after a reboot).
    ///
    /// # Errors
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `InvalidArg`, if the device is invalid
    /// * `NotSupported`, if this `Device` does not support this feature
    /// * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    /// * `Unknown`, on any unexpected error
    ///
    /// # Device Support
    /// Supports GK110 M-class Tesla products from the Kepler family. Modes `LowDP`
    /// and `AllOn` are supported on fully supported GeForce products. Not supported
    /// on Quadro and Tesla C-class products.
    pub fn gpu_operation_mode(&self) -> Result<OperationModeInfo> {
        unsafe {
            let mut current: nvmlGpuOperationMode_t = mem::zeroed();
            let mut pending: nvmlGpuOperationMode_t = mem::zeroed();
            nvml_try(nvmlDeviceGetGpuOperationMode(self.device, &mut current, &mut pending))?;

            Ok(OperationModeInfo{ current: current.into(),
                                  pending: pending.into() })
        }
    }

    /// Gets information about processes with a graphics context running on this `Device`,
    /// allocating `size` amount of space.
    ///
    /// This only returns information about graphics based processes (OpenGL, DirectX).
    ///
    /// Keep in mind that information returned by this call is dynamic and the number of elements
    /// may change over time. Allocate more space for information in case new compute processes
    /// are spawned.
    ///
    /// # Errors
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `InsufficientSize`, TODO: This
    /// * `InvalidArg`, if the device is invalid
    /// * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    /// * `Unknown`, on any unexpected error
    pub fn running_graphics_processes(&self, size: usize) -> Result<Vec<ProcessInfo>> {
        unsafe {
            let mut first_item: nvmlProcessInfo_t = mem::zeroed();
            // Passed in to designate size of returned array and after call is count of returned elements
            let mut count: c_uint = size as c_uint;
            nvml_try(nvmlDeviceGetGraphicsRunningProcesses(self.device, &mut count, &mut first_item))?;

            // TODO: Check along with compute
            let array = slice::from_raw_parts(&first_item as *const nvmlProcessInfo_t, count as usize);
            let vec = array.iter()
                           .map(|info| ProcessInfo::from(*info))
                           .collect();

            Ok(vec)
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

    /// Gets the checksum of the config stored in the device's infoROM.
    ///
    /// Can be used to make sure that two GPUs have the exact same configuration.
    /// The current checksum takes into account configuration stored in PWR and ECC
    /// infoROM objects. The checksum can change between driver released or when the
    /// user changes the configuration (e.g. disabling/enabling ECC).
    ///
    /// # Errors
    /// * `CorruptedInfoROM`, if the device's checksum couldn't be retrieved due to infoROM corruption
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `InvalidArg`, if the device is invalid
    /// * `NotSupported`, if this `Device` does not support this feature
    /// * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    /// * `Unknown`, on any unexpected error
    ///
    /// # Device Support
    /// Supports all devices with an infoROM.
    pub fn config_checksum(&self) -> Result<u32> {
        unsafe {
            let mut checksum: c_uint = mem::zeroed();
            nvml_try(nvmlDeviceGetInforomConfigurationChecksum(self.device, &mut checksum))?;

            Ok(checksum as u32)
        }
    }

    /// Gets the global infoROM image version.
    ///
    /// This image version, just like the VBIOS version, uniquely describes the exact version
    /// of the infoROM flashed on the board, in contrast to the infoROM object version which
    /// is only an indicator of supported features.
    ///
    /// # Errors
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `InvalidArg`, if the device is invalid
    /// * `NotSupported`, if this `Device` does not have an infoROM
    /// * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    /// * `Utf8Error`, if the string obtained from the C function is not valid Utf8
    /// * `Unknown`, on any unexpected error
    ///
    /// Why is `CorruptedInfoROM` not mentioned? Your guess is as good as mine. While we're
    /// at it, why is this one of two functions I've seen so far that does not say that
    /// it will return `InvalidArg` if the device is invalid, like every other device 
    /// function? Hmm.
    ///
    /// # Device Support
    /// Supports all devices with an infoROM.
    pub fn info_rom_image_version(&self) -> Result<String> {
        unsafe {
            let mut version_vec = Vec::with_capacity(NVML_DEVICE_INFOROM_VERSION_BUFFER_SIZE as usize);
            nvml_try(nvmlDeviceGetInforomImageVersion(self.device, 
                                                      version_vec.as_mut_ptr(), 
                                                      NVML_DEVICE_INFOROM_VERSION_BUFFER_SIZE))?;
            
            let version_raw = CStr::from_ptr(version_vec.as_ptr());
            Ok(version_raw.to_str()?.into())
        }
    }

    /// Gets the version information for this `Device`'s infoROM object, for the passed in 
    /// object type.
    ///
    /// # Errors
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `InvalidArg`, if the device is invalid
    /// * `NotSupported`, if this `Device` does not have an infoROM
    /// * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    /// * `Utf8Error`, if the string obtained from the C function is not valid Utf8
    /// * `Unknown`, on any unexpected error
    ///
    /// # Device Support
    /// Supports all devices with an infoROM.
    ///
    /// Fermi and higher parts have non-volatile on-board memory for persisting device info,
    /// such as aggregate ECC counts. The version of the data structures in this memory may
    /// change from time to time.
    pub fn info_rom_version(&self, object: InfoROM) -> Result<String> {
        unsafe {
            let mut version_vec = Vec::with_capacity(NVML_DEVICE_INFOROM_VERSION_BUFFER_SIZE as usize);
            nvml_try(nvmlDeviceGetInforomVersion(self.device,
                                                 object.eq_c_variant(),
                                                 version_vec.as_mut_ptr(),
                                                 NVML_DEVICE_INFOROM_VERSION_BUFFER_SIZE))?;
            
            let version_raw = CStr::from_ptr(version_vec.as_ptr());
            Ok(version_raw.to_str()?.into())
        }
    }

    /// Gets the maximum clock speeds for this `Device`.
    ///
    /// # Errors
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `InvalidArg`, if the device is invalid
    /// * `NotSupported`, if this `Device` cannot report the specified `Clock`
    /// * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    /// * `Unknown`, on any unexpected error
    ///
    /// # Device Support
    /// Supports Fermi and newer fully supported devices.
    ///
    /// Note: On GPUs from the Fermi family, current P0 clocks (reported by `.clock_info()`)
    /// can differ from max clocks by a few MHz.
    pub fn max_clock_info(&self, clock_type: Clock) -> Result<u32> {
        unsafe {
            let mut clock: c_uint = mem::zeroed();
            nvml_try(nvmlDeviceGetMaxClockInfo(self.device, clock_type.eq_c_variant(), &mut clock))?;

            Ok(clock as u32)
        }
    }

    /// Gets the max PCIe link generation possible with this `Device` and system.
    ///
    /// For a gen 2 PCIe device attached to a gen 1 PCIe bus, the max link generation
    /// this function will report is generation 1.
    ///
    /// # Errors
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `InvalidArg`, if the device is invalid
    /// * `NotSupported`, if PCIe link information is not available
    /// * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    /// * `Unknown`, on any unexpected error
    ///
    /// # Device Support
    /// Supports Fermi and newer fully supported devices.
    pub fn max_pcie_link_gen(&self) -> Result<u32> {
        unsafe {
            let mut max_gen: c_uint = mem::zeroed();
            nvml_try(nvmlDeviceGetMaxPcieLinkGeneration(self.device, &mut max_gen))?;

            Ok(max_gen as u32)
        }
    }

    /// Gets the maximum PCIe link width possible with this `Device` and system.
    ///
    /// For a device with a 16x PCie bus width attached to an 8x PCIe system bus,
    /// this method will report a max link width of 8.
    ///
    /// # Errors
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `InvalidArg`, if the device is invalid
    /// * `NotSupported`, if PCIe link information is not available
    /// * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    /// * `Unknown`, on any unexpected error
    ///
    /// # Device Support
    /// Supports Fermi and newer fully supported devices.
    pub fn max_pcie_link_width(&self) -> Result<u32> {
        unsafe {
            let mut max_width: c_uint = mem::zeroed();
            nvml_try(nvmlDeviceGetMaxPcieLinkWidth(self.device, &mut max_width))?;

            Ok(max_width as u32)
        }
    }

    /// Gets the requested memory error counter for this `Device`.
    ///
    /// Only applicable to devices with ECC. Requires ECC mode to be enabled.
    ///
    /// # Errors
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `InvalidArg`, if `error_type`, `counter_type`, or `location` is invalid (shouldn't occur?)
    /// * `NotSupported`, if this `Device` does not support ECC error reporting for the specified
    /// memory
    /// * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    /// * `Unknown`, on any unexpected error
    ///
    /// # Device Support
    /// Supports Fermi and newer fully supported devices. Requires `NVML_INFOROM_ECC` version
    /// 2.0 or higher to report aggregate location-based memory error counts. Requires
    /// `NVML_INFOROM_ECC` version 1.0 or higher to report all other memory error counts.
    pub fn memory_error_counter(&self,
                                error_type: MemoryError,
                                counter_type: EccCounter,
                                location: MemoryLocation) 
                                -> Result<u64> {
        unsafe {
            let mut count: c_ulonglong = mem::zeroed();
            nvml_try(nvmlDeviceGetMemoryErrorCounter(self.device,
                                                     error_type.eq_c_variant(),
                                                     counter_type.eq_c_variant(),
                                                     location.eq_c_variant(),
                                                     &mut count))?;
            
            Ok(count as u64)
        }
    }

    /// Gets the amount of used, free and total memory available on the device, in bytes.
    ///
    /// Note that enabling ECC reduces the amount of total available memory due to the
    /// extra required parity bits.
    ///
    /// Also note that on Windows, most device memory is allocated and managed on startup
    /// by Windows.
    ///
    /// Under Linux and Windows TCC (no physical display connected), the reported amount 
    /// of used memory is equal to the sum of memory allocated by all active channels on 
    /// the device.
    // TODO: is the above accurate?
    ///
    /// # Errors
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `InvalidArg`, if the device is invalid
    /// * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    /// * `Unknown`, on any unexpected error
    pub fn memory_info(&self) -> Result<MemoryInfo> {
        unsafe {
            let mut info: nvmlMemory_t = mem::zeroed();
            nvml_try(nvmlDeviceGetMemoryInfo(self.device, &mut info))?;

            Ok(info.into())
        }
    }

    /// Gets the minor number for this `Device`.
    ///
    /// The minor number is such that the NVIDIA device node file for each GPU will
    /// have the form `/dev/nvidia/[minor number]`.
    ///
    /// # Errors
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `InvalidArg`, if the device is invalid
    /// * `NotSupported`, if this query is not supported by this `Device`
    /// * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    /// * `Unknown`, on any unexpected error
    ///
    /// # Platform Support
    /// Only supports Linux.
    #[cfg(target_os = "linux")]
    pub fn minor_number(&self) -> Result<u32> {
        unsafe {
            let mut number: c_uint = mem::zeroed();
            nvml_try(nvmlDeviceGetMinorNumber(self.device, &mut number))?;

            Ok(number as u32)
        }
    }

    /// Identifies whether or not this `Device` is on a multi-GPU board.
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

    /// Gets the PCIe replay counter and rollover information.
    ///
    /// # Errors
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `InvalidArg`, if the device is invalid
    /// * `NotSupported`, if this `Device` does not support this feature
    /// * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    /// * `Unknown`, on any unexpected error
    ///
    /// # Device Support
    /// Supports Kepler or newer fully supported devices.
    pub fn pcie_replay_counter(&self) -> Result<u32> {
        unsafe {
            let mut value: c_uint = mem::zeroed();
            nvml_try(nvmlDeviceGetPcieReplayCounter(self.device, &mut value))?;

            Ok(value as u32)
        }
    }

    /// Gets PCIe utilization information in KB/s.
    ///
    /// The function called within this method is querying a byte counter over a 20ms
    /// interval and thus is the PCIE throughput over that interval.
    ///
    /// # Errors
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `InvalidArg`, if the device is invalid or `counter` is invalid (shouldn't occur?)
    /// * `NotSupported`, if this `Device` does not support this feature
    /// * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    /// * `Unknown`, on any unexpected error
    ///
    /// # Device Support
    /// Supports Maxwell and newer fully supported devices.
    pub fn pcie_throughput(&self, counter: PcieUtilCounter) -> Result<u32> {
        unsafe {
            let mut throughput: c_uint = mem::zeroed();
            nvml_try(nvmlDeviceGetPcieThroughput(self.device, counter.eq_c_variant(), &mut throughput))?;

            Ok(throughput as u32)
        }
    }

    /// Gets the current performance state for this `Device`. 0 == max, 15 == min.
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
    pub fn performance_state(&self) -> Result<PerformanceState> {
        unsafe {
            let mut state: nvmlPstates_t = mem::zeroed();
            nvml_try(nvmlDeviceGetPerformanceState(self.device, &mut state))?;

            Ok(state.into())
        }
    } 

    /// Gets whether or not persistent mode is enabled for this `Device`.
    ///
    /// When driver persistence mode is enabled the driver software is not torn down
    /// when the last client disconnects. This feature is disabled by default.
    ///
    /// # Errors
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `InvalidArg`, if the device is invalid
    /// * `NotSupported`, if this `Device` does not support this feature
    /// * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    /// * `Unknown`, on any unexpected error
    ///
    /// # Platform Support
    /// Only supports Linux.
    #[cfg(target_os = "linux")]
    pub fn is_in_persistent_mode(&self) -> Result<bool> {
        unsafe {
            let mut state: nvmlEnableState_t = mem::zeroed();
            nvml_try(nvmlDeviceGetPersistenceMode(self.device, &mut state))?;

            Ok(bool_from_state(state))
        }
    }

    /// Gets the default power management limit for this `Device`, in milliwatts.
    ///
    /// This is the limit that this `Device` boots with.
    ///
    /// # Errors
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `InvalidArg`, if the device is invalid
    /// * `NotSupported`, if this `Device` does not support this feature
    /// * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    /// * `Unknown`, on any unexpected error
    ///
    /// # Device Support
    /// Supports Kepler or newer fully supported devices.
    pub fn power_management_limit_default(&self) -> Result<u32> {
        unsafe {
            let mut limit: c_uint = mem::zeroed();
            nvml_try(nvmlDeviceGetPowerManagementDefaultLimit(self.device, &mut limit))?;

            Ok(limit as u32)
        }
    }

    /// Gets the power management limit associated with this `Device`.
    ///
    /// The power limit defines the upper boundary for the card's power draw. If the card's
    /// total power draw reaches this limit, the power management algorithm kicks in.
    ///
    /// This reading is only supported if power management mode is supported. See
    /// `.power_management_mode()`.
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
    pub fn power_management_limit(&self) -> Result<u32> {
        unsafe {
            let mut limit: c_uint = mem::zeroed();
            nvml_try(nvmlDeviceGetPowerManagementLimit(self.device, &mut limit))?;

            Ok(limit as u32)
        }
    }

    /// Gets information about possible power management limit values for this `Device`, in milliwatts.
    ///
    /// # Errors
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `InvalidArg`, if the device is invalid
    /// * `NotSupported`, if this `Device` does not support this feature
    /// * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    /// * `Unknown`, on any unexpected error
    ///
    /// # Device Support
    /// Supports Kepler or newer fully supported devices.
    pub fn power_management_limit_constraints(&self) -> Result<PowerManagementConstraints> {
        unsafe {
            let mut min: c_uint = mem::zeroed();
            let mut max: c_uint = mem::zeroed();
            nvml_try(nvmlDeviceGetPowerManagementLimitConstraints(self.device, &mut min, &mut max))?;

            Ok(PowerManagementConstraints {
                min_limit: min as u32,
                max_limit: max as u32,
            })
        }
    }

    /// Not documenting this because it's deprecated. Read NVIDIA's docs if you must use it.
    #[deprecated(note = "NVIDIA states that \"this API has been deprecated.\"")]
    pub fn is_power_management_algo_active(&self) -> Result<bool> {
        unsafe {
            let mut state: nvmlEnableState_t = mem::zeroed();
            nvml_try(nvmlDeviceGetPowerManagementMode(self.device, &mut state))?;

            Ok(bool_from_state(state))
        }
    }

    /// Not documenting this because it's deprecated. Read NVIDIA's docs if you must use it.
    #[deprecated(note = "use `.performance_state()`.")]
    pub fn power_state(&self) -> Result<PerformanceState> {
        unsafe {
            let mut state: nvmlPstates_t = mem::zeroed();
            nvml_try(nvmlDeviceGetPowerState(self.device, &mut state))?;

            Ok(state.into())
        }
    }
}

#[cfg(feature = "test")]
#[cfg(feature = "test-local")]
#[allow(unused_variables, unused_imports)]
mod test {
    use super::*;

    #[test]
    fn running_compute_processes() {
        let test = NVML::init().expect("init call failed");
        let device = test.device_by_index(0).expect("Could not get a device by index 0");

        println!("{:?}", device.running_compute_processes(32).expect("You've failed"));
    }

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