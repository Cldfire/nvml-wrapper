use ffi::bindings::*;
use error::*;
use structs::device::*;
use struct_wrappers::device::*;
use enum_wrappers::*;
use enum_wrappers::device::*;
use event::EventSet;
use bitmasks::event::EventTypes;
use bitmasks::device::*;
use bitmasks::Behavior;
use NVML;
use std::marker::PhantomData;
use std::ffi::CStr;
use std::mem;
use std::os::raw::{c_uint, c_ulong, c_ulonglong, c_int};
use std::slice;

// TODO: Mark stuff that works on my 980 Ti but NVIDIA does not state should.
// TODO: A number of things here that return Utf8Errors I have not documented.

/// Struct that represents a device on the system. 
///
/// Obtain a `Device` with the various methods available to you on the `NVML`
/// struct.
/// 
/// Rust's lifetimes will ensure that the NVML instance this `Device` was created from is
/// not allowed to be shutdown until this `Device` is dropped, meaning you shouldn't
/// have to worry about calls returning `Uninitialized` errors.
// TODO: Use compiletest to ensure lifetime guarantees
#[derive(Debug)]
pub struct Device<'nvml> {
    device: nvmlDevice_t,
    // Storage for PCI info used in drain state calls
    pci_info: Option<nvmlPciInfo_t>,
    _phantom: PhantomData<&'nvml NVML>,
}

unsafe impl<'nvml> Send for Device<'nvml> {}
unsafe impl<'nvml> Sync for Device<'nvml> {}

impl<'nvml> From<nvmlDevice_t> for Device<'nvml> {
    fn from(device: nvmlDevice_t) -> Self {
        Device {
            device: device,
            pci_info: None,
            _phantom: PhantomData,
        }
    }
}

impl<'nvml> Device<'nvml> {
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
    /// # Device Support
    /// Supports Kepler or newer fully supported devices.
    ///
    /// # Platform Support
    /// Only supports Linux. 
    // Checked against local
    #[cfg(target_os = "linux")]
    #[inline]
    pub fn clear_cpu_affinity(&self) -> Result<()> {
        unsafe {
            nvml_try(nvmlDeviceClearCpuAffinity(self.device)) 
        }
    }

    /// Gets the root/admin permissions for the target API.
    ///
    /// Only root users are able to call functions belonging to restricted APIs. See 
    /// the documentation for the `RestrictedApi` enum for a list of those functions.
    ///
    /// Non-root users can be granted access to these APIs through use of
    /// `.set_api_restricted()`.
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
    // TODO: Make sure there's a test case for when an error is returned and the mem::zeroed() values may be dropped
    // Checked against local
    #[inline]
    pub fn is_api_restricted(&self, api: Api) -> Result<bool> {
        unsafe {
            let mut restricted_state: nvmlEnableState_t = mem::zeroed();
            nvml_try(nvmlDeviceGetAPIRestriction(self.device, api.into_c(), &mut restricted_state))?;

            Ok(bool_from_state(restricted_state))
        }
    }

    /// Gets the current clock setting that all applications will use unless an overspec 
    /// situation occurs.
    ///
    /// This setting can be changed using `.set_applications_clocks()`.
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
    // Checked against local
    #[inline]
    pub fn applications_clock(&self, clock_type: Clock) -> Result<u32> {
        unsafe {
            let mut clock: c_uint = mem::zeroed();
            nvml_try(nvmlDeviceGetApplicationsClock(self.device, clock_type.into_c(), &mut clock))?;

            Ok(clock as u32)
        }
    }

    /// Gets the current and default state of auto boosted clocks.
    ///
    /// Auto boosted clocks are enabled by default on some hardware, allowing the GPU to run
    /// as fast as thermals will allow it to. 
    ///
    /// On Pascal and newer hardware, auto boosted clocks are controlled through application
    /// clocks. Use `.set_applications_clocks()` and `.reset_applications_clocks()` to control
    /// auto boost behavior.
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
    // Checked against local
    #[inline]
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
    // Checked against local
    #[inline]
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
    // Checked against local
    #[inline]
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
    #[inline]
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
    // Checked against local
    #[inline]
    pub fn bridge_chip_info(&self) -> Result<BridgeChipHierarchy> {
        unsafe {
            let mut info: nvmlBridgeChipHierarchy_t = mem::zeroed();
            nvml_try(nvmlDeviceGetBridgeChipInfo(self.device, &mut info))?;

            Ok(BridgeChipHierarchy::from(info))
        }
    }

    /// Gets this `Device`'s current clock speed for the given `Clock` type and `ClockId`.
    ///
    /// # Errors
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `InvalidArg`, if this `Device` is invalid or `clock_type` is invalid (shouldn't occur?)
    /// * `NotSupported`, if this `Device` does not support this feature
    /// * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    /// * `Unknown`, on any unexpected error
    ///
    /// # Device Support
    /// Supports Kepler and newer fully supported devices.
    // Checked against local
    #[inline]
    pub fn clock(&self, clock_type: Clock, clock_id: ClockId) -> Result<u32> {
        unsafe {
            let mut clock: c_uint = mem::zeroed();
            nvml_try(nvmlDeviceGetClock(self.device,
                                        clock_type.into_c(),
                                        clock_id.into_c(),
                                        &mut clock))?;

            Ok(clock as u32)
        }
    }

    /// Gets this `Device`'s customer-defined maximum boost clock speed for the
    /// given `Clock` type.
    ///
    /// # Errors
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `InvalidArg`, if this `Device` is invalid or `clock_type` is invalid (shouldn't occur?)
    /// * `NotSupported`, if this `Device` or the `clock_type` on this `Device`
    /// does not support this feature
    /// * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    /// * `Unknown`, on any unexpected error
    ///
    /// # Device Support
    /// Supports Maxwell and newer fully supported devices.
    // Checked against local
    #[inline]
    pub fn max_customer_boost_clock(&self, clock_type: Clock) -> Result<u32> {
        unsafe {
            let mut clock: c_uint = mem::zeroed();
            nvml_try(nvmlDeviceGetMaxCustomerBoostClock(self.device,
                                                        clock_type.into_c(),
                                                        &mut clock))?;

            Ok(clock as u32)
        }
    }

    /// Gets the current compute mode for this `Device`.
    ///
    /// # Errors
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `InvalidArg`, if this `Device` is invalid
    /// * `NotSupported`, if this `Device` does not support this feature
    /// * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    /// * `Unknown`, on any unexpected error
    // Checked against local
    #[inline]
    pub fn compute_mode(&self) -> Result<ComputeMode> {
        unsafe {
            let mut mode: nvmlComputeMode_t = mem::zeroed();
            nvml_try(nvmlDeviceGetComputeMode(self.device, &mut mode))?;

            Ok(ComputeMode::try_from(mode)?)
        }
    }

    /// Gets this `Device`'s current clock speed for the given `Clock` type.
    ///
    /// # Errors
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `InvalidArg`, if this `Device` is invalid
    /// * `NotSupported`, if this `Device` cannot report the specified clock
    /// * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    /// * `Unknown`, on any unexpected error
    ///
    /// # Device Support
    /// Supports Fermi or newer fully supported devices.
    // Checked against local
    #[inline]
    pub fn clock_info(&self, clock_type: Clock) -> Result<u32> {
        unsafe {
            let mut clock: c_uint = mem::zeroed();
            nvml_try(nvmlDeviceGetClockInfo(self.device, clock_type.into_c(), &mut clock))?;

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
    #[inline]
    pub fn running_compute_processes(&self, size: usize) -> Result<Vec<ProcessInfo>> {
        unsafe {
            let mut first_item: nvmlProcessInfo_t = mem::zeroed();
            // Passed in to designate size of returned array and after call is count of returned elements
            let mut count: c_uint = size as c_uint;
            nvml_try(nvmlDeviceGetComputeRunningProcesses(self.device, &mut count, &mut first_item))?;

            Ok(slice::from_raw_parts(&first_item as *const nvmlProcessInfo_t,
                                     count as usize)
                                     .iter()
                                     .map(|info| ProcessInfo::from(*info))
                                     .collect())
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
    // Checked against local
    #[cfg(target_os = "linux")]
    #[inline]
    pub fn cpu_affinity(&self, size: usize) -> Result<Vec<u64>> {
        unsafe {
            let mut first_item: c_ulong = mem::zeroed();
            nvml_try(nvmlDeviceGetCpuAffinity(self.device, size as c_uint, &mut first_item))?;

            // TODO: same as running_compute_processes, is this safe
            let array = slice::from_raw_parts(first_item as *const c_ulong, size);
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
    // Checked against local
    #[inline]
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
    // Checked against local
    #[inline]
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
    // Checked against local
    #[inline]
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
    // Checked against local
    #[inline]
    pub fn default_applications_clock(&self, clock_type: Clock) -> Result<u32> {
        unsafe {
            let mut clock: c_uint = mem::zeroed();
            nvml_try(nvmlDeviceGetDefaultApplicationsClock(self.device, clock_type.into_c(), &mut clock))?;

            Ok(clock as u32)
        }
    }

    /// Not documenting this because it's deprecated. Read NVIDIA's docs if you must use it.
    #[deprecated(note="use `Device.memory_error_counter()`")]
    #[inline]
    pub fn detailed_ecc_errors(&self, error_type: MemoryError, counter_type: EccCounter) -> Result<EccErrorCounts> {
        unsafe {
            let mut counts: nvmlEccErrorCounts_t = mem::zeroed();
            nvml_try(nvmlDeviceGetDetailedEccErrors(self.device, 
                                                    error_type.into_c(), 
                                                    counter_type.into_c(), 
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
    /// A display can be active even when no monitor is physically attached to this `Device`.
    ///
    /// # Errors
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `InvalidArg`, if the device is invalid
    /// * `NotSupported`, if this `Device` does not support this feature
    /// * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    /// * `Unknown`, on any unexpected error
    // Checked against local
    #[inline]
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
    /// This calls the C function `nvmlDeviceGetDisplayMode`.
    ///
    /// # Errors
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `InvalidArg`, if the device is invalid
    /// * `NotSupported`, if this `Device` does not support this feature
    /// * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    /// * `Unknown`, on any unexpected error
    // Checked against local
    #[inline]
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
    // Checked against local
    #[cfg(target_os = "windows")]
    #[inline]
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
    // Checked against local
    #[inline]
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
    // Checked against local
    #[inline]
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
    // Checked against local
    #[inline]
    pub fn enforced_power_limit(&self) -> Result<u32> {
        unsafe {
            let mut limit: c_uint = mem::zeroed();
            nvml_try(nvmlDeviceGetEnforcedPowerLimit(self.device, &mut limit))?;

            Ok(limit as u32)
        }
    }

    /// Gets the intended operating speed of this `Device`'s fan as a percentage of the
    /// maximum fan speed (100%).
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
    // Checked against local
    #[inline]
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
    /// Supports GK110 M-class and X-class Tesla products from the Kepler family. Modes `LowDP`
    /// and `AllOn` are supported on fully supported GeForce products. Not supported
    /// on Quadro and Tesla C-class products.
    // Checked against local
    #[inline]
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
    /// may change over time. Allocate more space for information in case new graphics processes
    /// are spawned.
    ///
    /// # Errors
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `InsufficientSize`, TODO: This
    /// * `InvalidArg`, if the device is invalid
    /// * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    /// * `Unknown`, on any unexpected error
    #[inline]
    pub fn running_graphics_processes(&self, size: usize) -> Result<Vec<ProcessInfo>> {
        unsafe {
            let mut first_item: nvmlProcessInfo_t = mem::zeroed();
            // Passed in to designate size of returned array and after call is count of returned elements
            let mut count: c_uint = size as c_uint;
            nvml_try(nvmlDeviceGetGraphicsRunningProcesses(self.device, &mut count, &mut first_item))?;

            // TODO: Check along with compute
            Ok(slice::from_raw_parts(&first_item as *const nvmlProcessInfo_t,
                                     count as usize)
                                     .iter()
                                     .map(|info| ProcessInfo::from(*info))
                                     .collect())
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
    // Checked against local
    #[inline]
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
    /// * `NotSupported`, if this `Device` does not support this feature
    /// * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    /// * `Unknown`, on any unexpected error
    ///
    /// # Device Support
    /// Supports all devices with an infoROM.
    // Checked against local
    #[inline]
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
    // Checked against local
    #[inline]
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
    /// * `Utf8Error`, if the string obtained from the C function is not valid UTF-8
    /// * `Unknown`, on any unexpected error
    ///
    /// # Device Support
    /// Supports all devices with an infoROM.
    ///
    /// Fermi and higher parts have non-volatile on-board memory for persisting device info,
    /// such as aggregate ECC counts. The version of the data structures in this memory may
    /// change from time to time.
    // Checked against local
    #[inline]
    pub fn info_rom_version(&self, object: InfoROM) -> Result<String> {
        unsafe {
            let mut version_vec = Vec::with_capacity(NVML_DEVICE_INFOROM_VERSION_BUFFER_SIZE as usize);
            nvml_try(nvmlDeviceGetInforomVersion(self.device,
                                                 object.into_c(),
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
    /// Note: On GPUs from the Fermi family, current P0 (Performance state 0?) clocks 
    /// (reported by `.clock_info()`) can differ from max clocks by a few MHz.
    // Checked against local
    #[inline]
    pub fn max_clock_info(&self, clock_type: Clock) -> Result<u32> {
        unsafe {
            let mut clock: c_uint = mem::zeroed();
            nvml_try(nvmlDeviceGetMaxClockInfo(self.device, clock_type.into_c(), &mut clock))?;

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
    // Checked against local
    #[inline]
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
    // Checked against local
    #[inline]
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
    // Checked against local
    #[inline]
    pub fn memory_error_counter(&self,
                                error_type: MemoryError,
                                counter_type: EccCounter,
                                location: MemoryLocation) 
                                -> Result<u64> {
        unsafe {
            let mut count: c_ulonglong = mem::zeroed();
            nvml_try(nvmlDeviceGetMemoryErrorCounter(self.device,
                                                     error_type.into_c(),
                                                     counter_type.into_c(),
                                                     location.into_c(),
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
    ///
    /// # Errors
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `InvalidArg`, if the device is invalid
    /// * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    /// * `Unknown`, on any unexpected error
    // Checked against local
    #[inline]
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
    /// have the form `/dev/nvidia[minor number]`.
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
    // Checked against local
    #[cfg(target_os = "linux")]
    #[inline]
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
    // Checked against local
    #[inline]
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
    // Checked against local
    #[inline]
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
    // Checked against local
    #[inline]
    pub fn pci_info(&self) -> Result<PciInfo> {
        unsafe {
            let mut pci_info: nvmlPciInfo_t = mem::zeroed();
            nvml_try(nvmlDeviceGetPciInfo_v2(self.device, &mut pci_info))?;

            Ok(PciInfo::try_from(pci_info)?)
        }
    }

    /// Gets the PCIe replay counter.
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
    // Checked against local
    #[inline]
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
    ///
    /// # Environment Support
    /// This method is not supported in virtualized GPU environments.
    // Checked against local
    #[inline]
    pub fn pcie_throughput(&self, counter: PcieUtilCounter) -> Result<u32> {
        unsafe {
            let mut throughput: c_uint = mem::zeroed();
            nvml_try(nvmlDeviceGetPcieThroughput(self.device, counter.into_c(), &mut throughput))?;

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
    // Checked against local
    #[inline]
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
    // Checked against local
    #[cfg(target_os = "linux")]
    #[inline]
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
    // Checked against local
    #[inline]
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
    /// # Errors
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `InvalidArg`, if the device is invalid
    /// * `NotSupported`, if this `Device` does not support this feature
    /// * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    /// * `Unknown`, on any unexpected error
    ///
    /// # Device Support
    /// Supports Fermi or newer fully supported devices.
    ///
    /// This reading is only supported if power management mode is supported. See
    /// `.is_power_management_algo_active()`. Yes, it's deprecated, but that's what
    /// NVIDIA's docs said to see.
    // Checked against local
    #[inline]
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
    // Checked against local
    #[inline]
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
    #[inline]
    pub fn is_power_management_algo_active(&self) -> Result<bool> {
        unsafe {
            let mut state: nvmlEnableState_t = mem::zeroed();
            nvml_try(nvmlDeviceGetPowerManagementMode(self.device, &mut state))?;

            Ok(bool_from_state(state))
        }
    }

    /// Not documenting this because it's deprecated. Read NVIDIA's docs if you must use it.
    #[deprecated(note = "use `.performance_state()`.")]
    #[inline]
    pub fn power_state(&self) -> Result<PerformanceState> {
        unsafe {
            let mut state: nvmlPstates_t = mem::zeroed();
            nvml_try(nvmlDeviceGetPowerState(self.device, &mut state))?;

            Ok(state.into())
        }
    }

    /// Gets the power usage for this GPU and its associated circuitry (memory) in milliwatts.
    ///
    /// # Errors
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `InvalidArg`, if the device is invalid
    /// * `NotSupported`, if this `Device` does not support power readings
    /// * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    /// * `Unknown`, on any unexpected error
    ///
    /// # Device Support
    /// Supports Fermi and newer fully supported devices.
    ///
    /// This reading is accurate to within +/- 5% of current power draw on Fermi and Kepler GPUs.
    /// It is only supported if power management mode is supported. See `.is_power_management_algo_active()`.
    /// Yes, that is deperecated, but that's what NVIDIA's docs say to see.
    // Checked against local
    #[inline]
    pub fn power_usage(&self) -> Result<u32> {
        unsafe {
            let mut usage: c_uint = mem::zeroed();
            nvml_try(nvmlDeviceGetPowerUsage(self.device, &mut usage))?;

            Ok(usage as u32)
        }
    }

    /// Gets the list of retired pages filtered by `cause`, including pages pending retirement.
    ///
    /// The address information provided by this API is the hardware address of the page that was
    /// retired. Note that this does not match the virtual address used in CUDA, but it will
    /// match the address information in XID 63.
    ///
    /// # Errors
    /// * `InsufficientSize`, if
    // TODO: That ^
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `InvalidArg`, if the device is invalid
    /// * `NotSupported`, if this `Device` doesn't support this feature
    /// * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    /// * `Unknown`, on any unexpected error
    ///
    /// # Device Support
    /// Supports Kepler and newer fully supported devices.
    // Checked against local
    #[inline]
    pub fn retired_pages(&self, cause: RetirementCause, size: usize) -> Result<Vec<u64>> {
        unsafe {
            // TODO: Encode into the type system that you can pass 0 to query
            // TODO: This is also supposed to be the size required if the call fails. Ugh.
            let mut page_count: c_uint = size as c_uint;
            let mut first_item: c_ulonglong = mem::zeroed();
            nvml_try(nvmlDeviceGetRetiredPages(self.device, 
                                               cause.into_c(), 
                                               &mut page_count, 
                                               &mut first_item))?;
            
            // TODO: I really don't think I'm doing this right.
            let array = slice::from_raw_parts(&first_item as *const c_ulonglong, page_count as usize);
            Ok(array.to_vec())
            
        }
    }

    /// Gets whether there are pages pending retirement (they need a reboot to fully retire).
    ///
    /// # Errors
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `InvalidArg`, if the device is invalid
    /// * `NotSupported`, if this `Device` doesn't support this feature
    /// * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    /// * `Unknown`, on any unexpected error
    ///
    /// # Device Support
    /// Supports Kepler and newer fully supported devices.
    // Checked against local
    #[inline]
    pub fn are_pages_pending_retired(&self) -> Result<bool> {
        unsafe {
            let mut state: nvmlEnableState_t = mem::zeroed();
            nvml_try(nvmlDeviceGetRetiredPagesPendingStatus(self.device, &mut state))?;

            Ok(bool_from_state(state))
        }
    }

    /// Gets recent samples for this `Device`.
    ///
    /// `last_seen_timestamp` represents the CPU timestamp in μs. Passing in `None`
    /// will fetch all samples maintained in the underlying buffer; you can
    /// alternatively pass in a timestamp retrieved from the date of the previous
    /// query in order to obtain more recent samples.
    ///
    /// The advantage of using this method for samples in contrast to polling via
    /// existing methods is to get higher frequency data at a lower polling cost.
    ///
    /// # Errors
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `InvalidArg`, TODO: <---
    /// * `NotSupported`, if this query is not supported by this `Device`
    /// * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    /// * `NotFound`, if sample entries are not found
    /// * `Unknown`, on any unexpected error
    ///
    /// # Device Support
    /// Supports Kepler and newer fully supported devices.
    ///
    /// # Rustc Support
    /// Only compiles on nightly due to use of the `untagged_unions` feature. See
    /// [the tracking issue](https://github.com/rust-lang/rust/issues/32836).
    // TODO: Complicated, figure out how to handle array allocation
    #[cfg(feature = "nightly")]
    pub fn samples<T>(&self, sample_type: Sampling, last_seen_timestamp: T) -> Result<Vec<Sample>>
        where T: Into<Option<u64>> {
            let timestamp = last_seen_timestamp.into().unwrap_or(0);

            unsafe {
                let mut value_type: nvmlValueType_t = mem::zeroed();
                let mut count: c_uint = mem::zeroed();
                // TODO: Fairly sure this is completely wrong
                let mut first_item: nvmlSample_t = mem::zeroed();

                nvml_try(nvmlDeviceGetSamples(self.device, 
                                              sample_type.into_c(), 
                                              timestamp as c_ulonglong,
                                              &mut value_type,
                                              &mut count,
                                              &mut first_item))?;

                let value_type_rust = SampleValueType::try_from(value_type)?;
                let array = slice::from_raw_parts(&first_item as *const nvmlSample_t, count as usize);
                Ok(array.iter()
                        .map(|s| Sample::from_tag_and_struct(&value_type_rust, *s))
                        .collect())
            }
    }

    /// Gets the globally unique board serial number associated with this `Device`'s board
    /// as an alphanumeric string.
    ///
    /// This serial number matches the serial number tag that is physically attached to the board.
    ///
    /// # Errors
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `InvalidArg`, if the device is invalid
    /// * `NotSupported`, if this `Device` doesn't support this feature
    /// * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    /// * `Unknown`, on any unexpected error
    ///
    /// # Device Support
    /// Supports all products with an infoROM.
    // Checked against local
    #[inline]
    pub fn serial(&self) -> Result<String> {
        unsafe {
            let mut serial_vec = Vec::with_capacity(NVML_DEVICE_SERIAL_BUFFER_SIZE as usize);
            nvml_try(nvmlDeviceGetSerial(self.device, serial_vec.as_mut_ptr(), NVML_DEVICE_SERIAL_BUFFER_SIZE))?;

            let serial_raw = CStr::from_ptr(serial_vec.as_ptr());
            Ok(serial_raw.to_str()?.into())
        }
    }

    /// Gets the board part number for this `Device`.
    ///
    /// The board part number is programmed into the board's infoROM.
    ///
    /// # Errors
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `NotSupported`, if the necessary VBIOS fields have not been filled
    /// * `GpuLost`, if the target GPU has fellen off the bus or is otherwise inaccessible
    /// * `Unknown`, on any unexpected error
    // Checked against local
    #[inline]
    pub fn board_part_number(&self) -> Result<String> {
        unsafe {
            let mut part_num_vec = Vec::with_capacity(NVML_DEVICE_PART_NUMBER_BUFFER_SIZE as usize);
            nvml_try(nvmlDeviceGetBoardPartNumber(self.device, 
                                                  part_num_vec.as_mut_ptr(), 
                                                  NVML_DEVICE_PART_NUMBER_BUFFER_SIZE))?;

            let part_num_raw = CStr::from_ptr(part_num_vec.as_ptr());
            Ok(part_num_raw.to_str()?.into())
        }
    }

    /// Gets current throttling reasons.
    ///
    /// Note that multiple reasons can be affecting clocks at once.
    ///
    /// # Errors
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `NotSupported`, if this `Device` does not support this feature
    /// * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    /// * `Unknown`, on any unexpected error
    ///
    /// # Device Support
    /// Supports all _fully supported_ devices.
    // Checked against local.
    #[inline]
    pub fn current_throttle_reasons(&self) -> Result<ThrottleReasons> {
        unsafe {
            let mut reasons: c_ulonglong = mem::zeroed();
            nvml_try(nvmlDeviceGetCurrentClocksThrottleReasons(self.device, &mut reasons))?;

            ThrottleReasons::from_bits(reasons as u64)
                .ok_or(Error::from_kind(ErrorKind::IncorrectBits))
        }
    } 

    /// Gets a bitmask of the supported throttle reasons.
    ///
    /// These reasons can be returned by `.current_throttle_reasons()`.
    ///
    /// # Errors
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    /// * `Unknown`, on any unexpected error
    ///
    /// # Device Support
    /// Supports all _fully supported_ devices.
    ///
    /// # Environment Support
    /// This method is not supported in virtualized GPU environments.
    // Checked against local
    #[inline]
    pub fn supported_throttle_reasons(&self) -> Result<ThrottleReasons> {
        unsafe {
            let mut reasons: c_ulonglong = mem::zeroed();
            nvml_try(nvmlDeviceGetSupportedClocksThrottleReasons(self.device, &mut reasons))?;

            ThrottleReasons::from_bits(reasons as u64)
                .ok_or(Error::from_kind(ErrorKind::IncorrectBits))
        }
    }

    /// Gets a `Vec` of possible graphics clocks that can be used as an arg for
    /// `set_applications_clocks()`.
    ///
    /// # Errors
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `NotFound`, if the specified `for_mem_clock` is not a supported frequency
    /// * `InvalidArg`, if the device is invalid
    /// * `NotSupported`, if this `Device` doesn't support this feature
    /// * `InsufficientSize`, if `size` is too small
    /// * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    /// * `Unknown`, on any unexpected error
    ///
    /// # Device Support
    /// Supports Kepler and newer fully supported devices.
    // Checked against local
    #[inline]
    pub fn supported_graphics_clocks(&self, for_mem_clock: u32, size: c_uint) -> Result<Vec<u32>> {
        unsafe {
            let mut first_item: c_uint = mem::zeroed();
            // TODO: Convert all other function calls like this to take `size` param as c_uint
            let mut count: c_uint = size;
            nvml_try(nvmlDeviceGetSupportedGraphicsClocks(self.device, 
                                                          for_mem_clock as c_uint, 
                                                          &mut count, 
                                                          &mut first_item))?;

            let array = slice::from_raw_parts(&first_item as *const c_uint, count as usize);
            Ok(array.to_vec())
        }
    }

    /// Gets a `Vec` of possible memory clocks that can be used as an arg for
    /// `set_applications_clocks()`.
    ///
    /// # Errors
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `InvalidArg`, if the device is invalid
    /// * `NotSupported`, if this `Device` doesn't support this feature
    /// * `InsufficientSize`, if `size` is too small
    // TODO: ^ read below
    /// * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    /// * `Unknown`, on any unexpected error
    ///
    /// # Device Support
    /// Supports Kepler and newer fully supported devices.
    // Checked against local
    #[inline]
    pub fn supported_memory_clocks(&self, size: c_uint) -> Result<Vec<u32>> {
        unsafe {
            let mut first_item: c_uint = mem::zeroed();
            // TODO: Convert all other function calls like this to take `size` param as c_uint
            // TODO: says count is set to the number of required elements if `InsufficientSize`?
            let mut count: c_uint = size;
            nvml_try(nvmlDeviceGetSupportedMemoryClocks(self.device, 
                                                          &mut count, 
                                                          &mut first_item))?;

            let array = slice::from_raw_parts(&first_item as *const c_uint, count as usize);
            Ok(array.to_vec())
        }
    }

    /// Gets the current temperature readings for the given sensor, in °C.
    ///
    /// # Errors
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `InvalidArg`, if the device is invalid or `sensor` is invalid (shouldn't occur?)
    /// * `NotSupported`, if this `Device` does not have the specified sensor
    /// * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    /// * `Unknown`, on any unexpected error
    // Checked against local
    #[inline]
    pub fn temperature(&self, sensor: TemperatureSensor) -> Result<u32> {
        unsafe {
            let mut temp: c_uint = mem::zeroed();
            nvml_try(nvmlDeviceGetTemperature(self.device, sensor.into_c(), &mut temp))?;

            Ok(temp as u32)
        }
    }

    /// Gets the temperature threshold for this `Device` and the specified `threshold_type`, in °C.
    ///
    /// # Errors
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `InvalidArg`, if the device is invalid or `threshold_type` is invalid (shouldn't occur?)
    /// * `NotSupported`, if this `Device` does not have a temperature sensor or is unsupported
    /// * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    /// * `Unknown`, on any unexpected error
    ///
    /// # Device Support
    /// Supports Kepler and newer fully supported devices.
    // Checked against local
    #[inline]
    pub fn temperature_threshold(&self, threshold_type: TemperatureThreshold) -> Result<u32> {
        unsafe {
            let mut temp: c_uint = mem::zeroed();
            nvml_try(nvmlDeviceGetTemperatureThreshold(self.device, threshold_type.into_c(), &mut temp))?;

            Ok(temp as u32)
        }
    }

    /// Gets the common ancestor for two devices.
    ///
    /// # Errors
    /// * `InvalidArg`, if either `Device` is invalid
    /// * `NotSupported`, if this `Device` or the OS does not support this feature
    /// * `Unknown`, an error has occured in the underlying topology discovery
    ///
    /// # Platform Support
    /// Only supports Linux.
    // Checked against local
    #[cfg(target_os = "linux")]
    #[inline]
    pub fn topology_common_ancestor(&self, other_device: Device) -> Result<TopologyLevel> {
        unsafe {
            let mut level: nvmlGpuTopologyLevel_t = mem::zeroed();
            nvml_try(nvmlDeviceGetTopologyCommonAncestor(self.device, other_device.device, &mut level))?;

            Ok(level.into())
        }
    }

    /// Gets the set of GPUs that are nearest to this `Device` at a specific interconnectivity level.
    ///
    /// # Errors
    /// * `InvalidArg`, if the device is invalid or `level` is invalid (shouldn't occur?)
    /// * `NotSupported`, if this `Device` or the OS does not support this feature
    /// * `Unknown`, an error has occured in the underlying topology discovery
    ///
    /// # Platform Support
    /// Only supports Linux.
    // Checked against local
    #[cfg(target_os = "linux")]
    #[inline]
    pub fn topology_nearest_gpus(&self, level: TopologyLevel) -> Result<Vec<Device<'nvml>>> {
        unsafe {
            let mut first_item: nvmlDevice_t = mem::zeroed();
            // TODO: Fails if I pass 0? What?
            let mut count: c_uint = 0;
            nvml_try(nvmlDeviceGetTopologyNearestGpus(self.device, 
                                                      level.into_c(), 
                                                      &mut count, 
                                                      &mut first_item))?;
            
            // TODO: Again I believe I'm doing every single one of these wrong. The array has
            // already been malloc'd on the C side according to NVIDIA, meaning I'm probably
            // responsible for freeing the memory or something? Which I'm not doing here?
            // Investigate?
            Ok(slice::from_raw_parts(&first_item as *const nvmlDevice_t, 
                                     count as usize)
                                     .iter()
                                     .map(|d| Device::from(*d))
                                     .collect())
        }
    }

    /// Gets the total ECC error counts for this `Device`.
    ///
    /// Only applicable to devices with ECC. The total error count is the sum of errors across
    /// each of the separate memory systems, i.e. the total set of errors across the entire device.
    ///
    /// # Errors
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `InvalidArg`, if the device is invalid or either enum is invalid (shouldn't occur?)
    /// * `NotSupported`, if this `Device` does not support this feature
    /// * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    /// * `Unknown`, on any unexpected error
    ///
    /// # Device Support
    /// Supports Fermi and newer fully supported devices. Requires NVML_INFOROM_ECC version 1.0
    /// or higher. Requires ECC mode to be enabled.
    // Checked against local
    #[inline]
    pub fn total_ecc_errors(&self, error_type: MemoryError, counter_type: EccCounter) -> Result<u64> {
        unsafe {
            let mut count: c_ulonglong = mem::zeroed();
            nvml_try(nvmlDeviceGetTotalEccErrors(self.device, 
                                                 error_type.into_c(), 
                                                 counter_type.into_c(), 
                                                 &mut count))?;

            Ok(count as u64)
        }
    }

    /// Gets the globally unique immutable UUID associated with this `Device` as a 5 part
    /// hexadecimal string.
    ///
    /// This UUID augments the immutable, board serial identifier. It is a globally unique
    /// identifier and is the _only_ available identifier for pre-Fermi-architecture products.
    /// It does NOT correspond to any identifier printed on the board.
    ///
    /// # Errors
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `InvalidArg`, if the device is invalid
    /// * `NotSupported`, if this `Device` does not support this feature
    /// * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    /// * `Utf8Error`, if the string obtained from the C function is not valid Utf8
    /// * `Unknown`, on any unexpected error
    // Checked against local
    #[inline]
    pub fn uuid(&self) -> Result<String> {
        unsafe {
            let mut uuid_vec = Vec::with_capacity(NVML_DEVICE_UUID_BUFFER_SIZE as usize);
            nvml_try(nvmlDeviceGetUUID(self.device, uuid_vec.as_mut_ptr(), NVML_DEVICE_UUID_BUFFER_SIZE))?;

            let uuid_raw = CStr::from_ptr(uuid_vec.as_ptr());
            Ok(uuid_raw.to_str()?.into())
        }
    }

    /// Gets the current utilization rates for this `Device`'s major subsystems.
    ///
    /// Note: During driver initialization when ECC is enabled, one can see high GPU
    /// and memory utilization readings. This is caused by the ECC memory scrubbing
    /// mechanism that is performed during driver initialization.
    ///
    /// # Errors
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `InvalidArg`, if the device is invalid
    /// * `NotSupported`, if this `Device` does not support this feature
    /// * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    /// * `Unknown`, on any unexpected error
    ///
    /// # Device Support
    /// Supports Fermi and newer fully supported devices.
    // Checked against local
    #[inline]
    pub fn utilization_rates(&self) -> Result<Utilization> {
        unsafe {
            let mut utilization: nvmlUtilization_t = mem::zeroed();
            nvml_try(nvmlDeviceGetUtilizationRates(self.device, &mut utilization))?;

            Ok(utilization.into())
        }
    }

    /// Gets the VBIOS version of this `Device`.
    ///
    /// The VBIOS version may change from time to time.
    ///
    /// # Errors
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `InvalidArg`, if the device is invalid
    /// * `NotSupported`, if this `Device` does not support this feature
    /// * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    /// * `Utf8Error`, if the string obtained from the C function is not valid UTF-8
    /// * `Unknown`, on any unexpected error
    // Checked against local
    #[inline]
    pub fn vbios_version(&self) -> Result<String> {
        unsafe {
            let mut version_vec = Vec::with_capacity(NVML_DEVICE_VBIOS_VERSION_BUFFER_SIZE as usize);
            nvml_try(nvmlDeviceGetVbiosVersion(self.device, 
                                               version_vec.as_mut_ptr(), 
                                               NVML_DEVICE_VBIOS_VERSION_BUFFER_SIZE))?;

            let version_raw = CStr::from_ptr(version_vec.as_ptr());
            Ok(version_raw.to_str()?.into())
        }
    }

    /// Gets the duration of time during which this `Device` was throttled (lower than the
    /// requested clocks) due to power or thermal constraints.
    ///
    /// This is important to users who are trying to understand if their GPUs throttle at any
    /// point while running applications. The difference in violation times at two different
    /// reference times gives the indication of a GPU throttling event.
    ///
    /// Violation for thermal capping is not supported at this time.
    ///
    /// # Errors
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `InvalidArg`, if the device is invalid or `perf_policy` is invalid (shouldn't occur?)
    /// * `NotSupported`, if this query is not supported by this `Device`
    /// * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    ///
    /// ...and this for some reason is not documented to return `Unknown`. Okay?
    ///
    /// # Device Support
    /// Supports Kepler or newer fully supported devices.
    // Checked against local
    #[inline]
    pub fn violation_status(&self, perf_policy: PerformancePolicy) -> Result<ViolationTime> {
        unsafe {
            let mut viol_time: nvmlViolationTime_t = mem::zeroed();
            nvml_try(nvmlDeviceGetViolationStatus(self.device, perf_policy.into_c(), &mut viol_time))?;

            Ok(viol_time.into())
        }
    }

    /// Checks if this `Device` and the passed-in device are on the same physical board.
    ///
    /// # Errors
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `InvalidArg`, if either `Device` is invalid
    /// * `NotSupported`, if this check is not supported by this `Device`
    /// * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    /// * `Unknown`, on any unexpected error
    // Checked against local
    #[inline]
    pub fn is_on_same_board_as(&self, other_device: &Device) -> Result<bool> {
        unsafe {
            let mut bool_int: c_int = mem::zeroed();
            nvml_try(nvmlDeviceOnSameBoard(self.device, other_device.c_device(), &mut bool_int))?;

            match bool_int {
                0 => Ok(false),
                _ => Ok(true),
            }
        }
    }

    /// Resets the application clock to the default value.
    ///
    /// This is the applications clock that will be used after a system reboot or a driver
    /// reload. The default value is a constant, but the current value be changed with
    /// `.set_applications_clocks()`.
    ///
    /// On Pascal and newer hardware, if clocks were previously locked with 
    /// `.set_applications_clocks()`, this call will unlock clocks. This returns clocks
    /// to their default behavior of automatically boosting above base clocks as
    /// thermal limits allow.
    ///
    /// # Errors
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `InvalidArg`, if the `Device` is invalid
    /// * `NotSupported`, if this `Device` does not support this feature
    /// * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    /// * `Unknown`, on any unexpected error
    ///
    /// # Device Support
    /// Supports Fermi and newer non-GeForce fully supported devices and Maxwell or newer
    /// GeForce devices.
    // Checked against local
    #[inline]
    pub fn reset_applications_clocks(&self) -> Result<()> {
        unsafe {
            nvml_try(nvmlDeviceResetApplicationsClocks(self.device))
        }
    }

    /// Try to set the current state of auto boosted clocks on this `Device`.
    ///
    /// Auto boosted clocks are enabled by default on some hardware, allowing the GPU to run
    /// as fast as thermals will allow it to. Auto boosted clocks should be disabled if fixed
    /// clock rates are desired.
    ///
    /// On Pascal and newer hardware, auto boosted clocks are controlled through application
    /// clocks. Use `.set_applications_clocks()` and `.reset_applications_clocks()` to control
    /// auto boost behavior.
    ///
    /// Non-root users may use this API by default, but access can be restricted by root using 
    /// `.set_api_restriction()`.
    ///
    /// Note: persistence mode is required to modify the curent auto boost settings and
    /// therefore must be enabled.
    ///
    /// # Errors
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `InvalidArg`, if the `Device` is invalid
    /// * `NotSupported`, if this `Device` does not support auto boosted clocks
    /// * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    /// * `Unknown`, on any unexpected error
    ///
    /// Not sure why nothing is said about `NoPermission`.
    ///
    /// # Device Support
    /// Supports Kepler and newer fully supported devices.
    // Checked against local
    #[inline]
    pub fn set_auto_boosted_clocks(&self, enabled: bool) -> Result<()> {
        unsafe {
            nvml_try(nvmlDeviceSetAutoBoostedClocksEnabled(self.device, state_from_bool(enabled)))
        }
    }

    /// Sets the ideal affinity for the calling thread and `Device` based on the guidelines given in
    /// `.cpu_affinity()`.
    ///
    /// Currently supports up to 64 processors.
    ///
    /// # Errors
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `InvalidArg`, if the `Device` is invalid
    /// * `NotSupported`, if this `Device` does not support this feature
    /// * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    /// * `Unknown`, on any unexpected error
    ///
    /// # Device Support
    /// Supports Kepler and newer fully supported devices.
    ///
    /// # Platform Support
    /// Only supports Linux.
    // Checked against local
    #[cfg(target_os = "linux")]
    #[inline]
    pub fn set_cpu_affinity(&self) -> Result<()> {
        unsafe {
            nvml_try(nvmlDeviceSetCpuAffinity(self.device))
        }
    }

    /// Try to set the default state of auto boosted clocks on this `Device`.
    ///
    /// This is the default state that auto boosted clocks will return to when no compute
    /// processes (e.g. CUDA application with an active context) are running.
    ///
    /// Requires root/admin permissions.
    ///
    /// Auto boosted clocks are enabled by default on some hardware, allowing the GPU to run
    /// as fast as thermals will allow it to. Auto boosted clocks should be disabled if fixed
    /// clock rates are desired.
    ///
    /// On Pascal and newer hardware, auto boosted clocks are controlled through application
    /// clocks. Use `.set_applications_clocks()` and `.reset_applications_clocks()` to control
    /// auto boost behavior.
    ///
    /// # Errors
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `NoPermission`, if the calling user does not have permission to change the default state
    /// * `InvalidArg`, if the `Device` is invalid
    /// * `NotSupported`, if this `Device` does not support auto boosted clocks
    /// * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    /// * `Unknown`, on any unexpected error
    ///
    /// # Device Support
    /// Supports Kepler or newer non-GeForce fully supported devices and Maxwell or newer
    /// GeForce devices.
    // Checked against local
    #[inline]
    pub fn set_auto_boosted_clocks_default(&self, enabled: bool) -> Result<()> {
        unsafe {
            // passing 0 because NVIDIA says flags are not supported yet
            nvml_try(nvmlDeviceSetDefaultAutoBoostedClocksEnabled(self.device, 
                                                                  state_from_bool(enabled), 
                                                                  0))
        }
    }

    /// Reads the infoROM from this `Device`'s flash and verifies the checksum.
    ///
    /// # Errors
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `CorruptedInfoROM`, if this `Device`'s infoROM is corrupted
    /// * `NotSupported`, if this `Device` does not support this feature
    /// * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    /// * `Unknown`, on any unexpected error
    ///
    /// Not sure why `InvalidArg` is not mentioned.
    ///
    /// # Device Support
    /// Supports all devices with an infoROM.
    // Checked against local
    #[inline]
    pub fn validate_info_rom(&self) -> Result<()> {
        unsafe {
            nvml_try(nvmlDeviceValidateInforom(self.device))
        }
    }

    // Wrappers for things from Accounting Statistics now

    /// Clears accounting information about all processes that have already terminated.
    ///
    /// Requires root/admin permissions.
    ///
    /// # Errors
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `InvalidArg`, if the `Device` is invalid
    /// * `NotSupported`, if this `Device` does not support this feature
    /// * `NoPermission`, if the user doesn't have permission to perform this operation
    /// * `Unknown`, on any unexpected error
    ///
    /// # Device Support
    /// Supports Kepler and newer fully supported devices.
    // Checked against local
    #[inline]
    pub fn clear_accounting_pids(&self) -> Result<()> {
        unsafe {
            nvml_try(nvmlDeviceClearAccountingPids(self.device))
        }
    }

    /// Gets the number of processes that the circular buffer with accounting PIDs can hold
    /// (in number of elements).
    ///
    /// This is the max number of processes that accounting information will be stored for
    /// before the oldest process information will get overwritten by information
    /// about new processes.
    ///
    /// # Errors
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `InvalidArg`, if the `Device` is invalid
    /// * `NotSupported`, if this `Device` does not support this feature or accounting mode
    /// is disabled
    /// * `Unknown`, on any unexpected error
    ///
    /// # Device Support
    /// Supports Kepler and newer fully supported devices.
    // Checked against local
    #[inline]
    pub fn accounting_buffer_size(&self) -> Result<u32> {
        unsafe {
            let mut count: c_uint = mem::zeroed();
            nvml_try(nvmlDeviceGetAccountingBufferSize(self.device, &mut count))?;

            Ok(count as u32)
        }
    }

    /// Gets whether or not per-process accounting mode is enabled.
    ///
    /// # Errors
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `InvalidArg`, if the `Device` is invalid
    /// * `NotSupported`, if this `Device` does not support this feature
    /// * `Unknown`, on any unexpected error
    ///
    /// # Device Support
    /// Supports Kepler and newer fully supported devices.
    // Checked against local
    #[inline]
    pub fn is_accounting_enabled(&self) -> Result<bool> {
        unsafe {
            let mut state: nvmlEnableState_t = mem::zeroed();
            nvml_try(nvmlDeviceGetAccountingMode(self.device, &mut state))?;

            Ok(bool_from_state(state))
        }
    }

    /// Gets the list of processes that can be queried for accounting stats.
    ///
    /// The list of processes returned can be in running or terminated state. Note that
    /// in the case of a PID collision some processes might not be accessible before
    /// the circular buffer is full.
    ///
    /// # Errors
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `InvalidArg`, if the `Device` is invalid
    /// * `NotSupported`, if this `Device` does not support this feature or accounting
    /// mode is disabled
    /// * `InsufficientSize`,
    // TODO: ^
    /// * `Unknown`, on any unexpected error
    // Checked against local
    #[inline]
    pub fn accounting_pids(&self, size: usize) -> Result<Vec<u32>> {
        unsafe {
            let mut first_item: c_uint = mem::zeroed();
            // TODO: Again, query with 0, if insufficientsize, count is supposed to be
            // required size...
            let mut count: c_uint = size as c_uint;
            nvml_try(nvmlDeviceGetAccountingPids(self.device, &mut count, &mut first_item))?;

            // TODO: is this safe, correct, is mapping this to u32 stupid
            Ok(slice::from_raw_parts(first_item as *const c_uint,
                                     count as usize)
                                     .iter()
                                     .map(|p| *p as u32)
                                     .collect())
        }
    }

    /// Gets a process's accounting stats.
    ///
    /// Accounting stats capture GPU utilization and other statistics across the lifetime
    /// of a process. Accounting stats can be queried during the lifetime of the process
    /// and after its termination. The `time` field in `AccountingStats` is reported as
    /// zero during the lifetime of the process and updated to the actual running time
    /// after its termination.
    ///
    /// Accounting stats are kept in a circular buffer; newly created processes overwrite
    /// information regarding old processes.
    ///
    /// Note:
    /// * Accounting mode needs to be on. See `.is_accounting_enabled()`.
    /// * Only compute and graphics applications stats can be queried. Monitoring
    /// applications can't be queried since they don't contribute to GPU utilization.
    /// * If a PID collision occurs, the stats of the latest process (the one that
    /// terminated last) will be reported.
    ///
    /// # Errors
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `InvalidArg`, if the `Device` is invalid
    /// * `NotFound`, if the process stats were not found
    /// * `NotSupported`, if this `Device` does not support this feature or accounting
    /// mode is disabled
    /// * `Unknown`, on any unexpected error
    ///
    /// # Device Support
    /// Suports Kepler and newer fully supported devices.
    ///
    /// # Warning
    /// On Kepler devices, per-process stats are accurate _only if_ there's one process
    /// running on this `Device`.
    // Checked against local
    #[inline]
    pub fn accounting_stats_for(&self, process_id: u32) -> Result<AccountingStats> {
        unsafe {
            let mut stats: nvmlAccountingStats_t = mem::zeroed();
            nvml_try(nvmlDeviceGetAccountingStats(self.device, process_id as c_uint, &mut stats))?;

            Ok(stats.into())
        }
    }

    /// Enables or disables per-process accounting.
    ///
    /// Requires root/admin permissions.
    ///
    /// Note:
    /// * This setting is not persistent and will default to disabled after the driver
    /// unloads. Enable persistence mode to be sure the setting doesn't switch off
    /// to disabled.
    /// * Enabling accounting mode has no negative impact on GPU performance.
    /// * Disabling accounting clears accounting information for all PIDs
    ///
    /// # Errors
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `InvalidArg`, if the `Device` is invalid
    /// * `NotSupported`, if this `Device` does not support this feature
    /// * `NoPermission`, if the user doesn't have permission to perform this operation
    /// * `Unknown`, on any unexpected error
    ///
    /// # Device Support
    /// Supports Kepler and newer fully supported devices.
    // Checked against local
    #[inline]
    pub fn set_accounting(&self, enabled: bool) -> Result<()> {
        unsafe {
            nvml_try(nvmlDeviceSetAccountingMode(self.device, state_from_bool(enabled)))
        }
    }

    // Device commands starting here

    /// Clears the ECC error and other memory error counts for this `Device`.
    ///
    /// Sets all of the specified ECC counters to 0, including both detailed and total counts.
    /// This operation takes effect immediately.
    ///
    /// Requires root/admin permissions and ECC mode to be enabled.
    ///
    /// # Errors
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `InvalidArg`, if the `Device` is invalid or `counter_type` is invalid (shouldn't occur?)
    /// * `NotSupported`, if this `Device` does not support this feature
    /// * `NoPermission`, if the user doesn't have permission to perform this operation
    /// * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    /// * `Unknown`, on any unexpected error
    ///
    /// # Device Support
    /// Supports Kepler and newer fully supported devices. Only applicable to devices with
    /// ECC. Requires `NVML_INFOROM_ECC` version 2.0 or higher to clear aggregate
    /// location-based ECC counts. Requires `NVML_INFOROM_ECC` version 1.0 or higher to
    /// clear all other ECC counts.
    // Checked against local
    #[inline]
    pub fn clear_ecc_error_counts(&self, counter_type: EccCounter) -> Result<()> {
        unsafe {
            nvml_try(nvmlDeviceClearEccErrorCounts(self.device, counter_type.into_c()))
        }
    }

    /// Changes the root/admin restrictions on certain APIs.
    ///
    /// This method can be used by a root/admin user to give non root/admin users access
    /// to certain otherwise-restricted APIs. The new setting lasts for the lifetime of
    /// the NVIDIA driver; it is not persistent. See `.is_api_restricted()` to query
    /// current settings.
    ///
    /// # Errors
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `InvalidArg`, if the `Device` is invalid or `api_type` is invalid (shouldn't occur?)
    /// * `NotSupported`, if this `Device` does not support changing API restrictions or
    /// this `Device` does not support the feature that API restrictions are being set for
    /// (e.g. enabling/disabling auto boosted clocks is not supported by this `Device`).
    /// * `NoPermission`, if the user doesn't have permission to perform this operation
    /// * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    /// * `Unknown`, on any unexpected error
    ///
    /// # Device Support
    /// Supports Kepler and newer fully supported devices.
    // Checked against local
    #[inline]
    pub fn set_api_restricted(&self, api_type: Api, restricted: bool) -> Result<()> {
        unsafe {
            nvml_try(nvmlDeviceSetAPIRestriction(self.device, 
                                                 api_type.into_c(), 
                                                 state_from_bool(restricted)))
        }
    }

    /// Sets clocks that applications will lock to.
    ///
    /// Sets the clocks that compute and graphics applications will be running at. e.g.
    /// CUDA driver requests these clocks during context creation which means this
    /// property defines clocks at which CUDA applications will be running unless some
    /// overspec event occurs (e.g. over power, over thermal or external HW brake).
    ///
    /// Can be used as a setting to request constant performance. Requires root/admin
    /// permissions.
    ///
    /// On Pascal and newer hardware, this will automatically disable automatic boosting
    /// of clocks. On K80 and newer Kepler and Maxwell GPUs, users desiring fixed performance
    /// should also call `.set_auto_boosted_clocks(false)` to prevent clocks from automatically
    /// boosting above the clock value being set here.
    ///
    /// Note that after a system reboot or driver reload applications clocks go back
    /// to their default value.
    ///
    /// # Errors
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `InvalidArg`, if the `Device` is invalid or the clocks are not a valid combo
    /// * `NotSupported`, if this `Device` does not support this feature
    /// * `NoPermission`, if the user doesn't have permission to perform this operation
    /// * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    /// * `Unknown`, on any unexpected error
    ///
    /// # Device Support
    /// Supports Kepler and newer non-GeForce fully supported devices and Maxwell or newer
    /// GeForce devices.
    // Checked against local
    #[inline]
    pub fn set_applications_clocks(&self, mem_clock: u32, graphics_clock: u32) -> Result<()> {
        unsafe {
            nvml_try(nvmlDeviceSetApplicationsClocks(self.device, 
                                                     mem_clock as c_uint, 
                                                     graphics_clock as c_uint))
        }
    }

    /// Sets the compute mode for this `Device`.
    ///
    /// The compute mode determines whether a GPU can be used for compute operations
    /// and whether it can be shared across contexts.
    ///
    /// This operation takes effect immediately. Under Linux it is not persistent
    /// across reboots and always resets to `Default`. Under Windows it is
    /// persistent.
    ///
    /// Under Windows, compute mode may only be set to `Default` when running in WDDM
    /// (physical display connected).
    ///
    /// Requires root/admin permissions.
    ///
    /// # Errors
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `InvalidArg`, if the `Device` is invalid or `mode` is invalid (shouldn't occur?)
    /// * `NotSupported`, if this `Device` does not support this feature
    /// * `NoPermission`, if the user doesn't have permission to perform this operation
    /// * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    /// * `Unknown`, on any unexpected error
    // Checked against local
    #[inline]
    pub fn set_compute_mode(&self, mode: ComputeMode) -> Result<()> {
        unsafe {
            nvml_try(nvmlDeviceSetComputeMode(self.device, mode.into_c()))
        }
    }

    /// Sets the driver model for this `Device`.
    ///
    /// This operation takes effect after the next reboot. The model may only be
    /// set to WDDM when running in DEFAULT compute mode. Changing the model to
    /// WDDM is not supported then the GPU doesn't support graphics acceleration
    /// or will not support it after a reboot.
    ///
    /// On Windows platforms the device driver can run in either WDDM or WDM (TCC)
    /// mode. If a physical display is attached to a device it must run in WDDM mode.
    ///
    /// It is possible to force the change to WDM (TCC) while the display is still
    /// attached with a `Behavior` of `FORCE`. This should only be done if the host
    /// is subsequently powered down and the display is detached from this `Device`
    /// before the next reboot.
    ///
    /// Requires root/admin permissions.
    ///
    /// # Errors
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `InvalidArg`, if the `Device` is invalid or `model` is invalid (shouldn't occur?)
    /// * `NotSupported`, if this `Device` does not support this feature
    /// * `NoPermission`, if the user doesn't have permission to perform this operation
    /// * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    /// * `Unknown`, on any unexpected error
    ///
    /// # Device Support
    /// Supports Fermi and newer fully supported devices.
    ///
    /// # Platform Support
    /// Only supports Windows.
    // Checked against local
    #[cfg(target_os = "windows")]
    #[inline]
    pub fn set_driver_model(&self, model: DriverModel, flags: Behavior) -> Result<()> {
        unsafe {
            nvml_try(nvmlDeviceSetDriverModel(self.device, model.into(), flags.bits()))
        }
    }

    /// Set whether or not ECC mode is enabled for this `Device`.
    ///
    /// Requires root/admin permissions. Only applicable to devices with ECC.
    ///
    /// This operation takes effect after the next reboot.
    ///
    /// # Errors
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `InvalidArg`, if the `Device` is invalid
    /// * `NotSupported`, if this `Device` does not support this feature
    /// * `NoPermission`, if the user doesn't have permission to perform this operation
    /// * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    /// * `Unknown`, on any unexpected error
    ///
    /// # Device Support
    /// Supports Kepler and newer fully supported devices. Requires NVML_INFOROM_ECC version
    /// 1.0 or higher.
    // Checked against local
    #[inline]
    pub fn set_ecc(&self, enabled: bool) -> Result<()> {
        unsafe {
            nvml_try(nvmlDeviceSetEccMode(self.device, state_from_bool(enabled)))
        }
    }

    /// Sets the GPU operation mode for this `Device`.
    ///
    /// Requires root/admin permissions. Chaning GOMs requires a reboot, a requirement
    /// that may be removed in the future.
    ///
    /// Compute only GOMs don't support graphics acceleration. Under Windows switching
    /// to these GOMs when the pending driver model is WDDM (physical display attached)
    /// is not supported.
    ///
    /// # Errors
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `InvalidArg`, if the `Device` is invalid or `mode` is invalid (shouldn't occur?)
    /// * `NotSupported`, if this `Device` does not support GOMs or a specific mode
    /// * `NoPermission`, if the user doesn't have permission to perform this operation
    /// * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    /// * `Unknown`, on any unexpected error
    ///
    /// # Device Support
    /// Supports GK110 M-class and X-class Tesla products from the Kepler family. Modes
    /// `LowDP` and `AllOn` are supported on fully supported GeForce products. Not
    /// supported on Quadro and Tesla C-class products.
    // Checked against local
    #[inline]
    pub fn set_gpu_op_mode(&self, mode: OperationMode) -> Result<()> {
        unsafe {
            nvml_try(nvmlDeviceSetGpuOperationMode(self.device, mode.into_c()))
        }
    }

    /// Sets the persistence mode for this `Device`.
    ///
    /// The persistence mode determines whether the GPU driver software is torn down
    /// after the last client exits.
    ///
    /// This operation takes effect immediately and requires root/admin permissions.
    /// It is not persistent across reboots; after each reboot it will default to
    /// disabled.
    ///
    /// # Errors
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `InvalidArg`, if the `Device` is invalid
    /// * `NotSupported`, if this `Device` does not support this feature
    /// * `NoPermission`, if the user doesn't have permission to perform this operation
    /// * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    /// * `Unknown`, on any unexpected error
    ///
    /// # Platform Support
    /// Only supports Linux.
    // Checked against local
    #[cfg(target_os = "linux")]
    #[inline]
    pub fn set_persistent(&self, enabled: bool) -> Result<()> {
        unsafe {
            nvml_try(nvmlDeviceSetPersistenceMode(self.device, state_from_bool(enabled)))
        }
    }

    /// Sets the power limit for this `Device`, in milliwatts.
    ///
    /// This limit is not persistent across reboots or driver unloads. Enable
    /// persistent mode to prevent the driver from unloading when no application
    /// is using this `Device`.
    ///
    /// Requires root/admin permissions. See `.power_management_limit_constraints()`
    /// to check the allowed range of values.
    ///
    /// # Errors
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `InvalidArg`, if the `Device` is invalid or `limit` is out of range
    /// * `NotSupported`, if this `Device` does not support this feature
    /// * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    /// * `Unknown`, on any unexpected error
    ///
    /// For some reason NVIDIA does not mention `NoPermission`.
    ///
    /// # Device Support
    /// Supports Kepler and newer fully supported devices.
    // Checked against local
    #[inline]
    pub fn set_power_management_limit(&self, limit: u32) -> Result<()> {
        unsafe {
            nvml_try(nvmlDeviceSetPowerManagementLimit(self.device, limit as c_uint))
        }
    }

    // Event handling methods
    // TODO: Figure out what to do about platform support situation for these

    /// Starts recording the given `EventTypes` for this `Device` and adding them
    /// to the specified `EventSet`.
    ///
    /// All events that occurred before this call was made will not be recorded.
    ///
    /// ECC events are only available on `Device`s with ECC enabled. Power capping events
    /// are only available on `Device`s with power management enabled.
    ///
    /// # Errors
    /// `Uninitialized`, if the library has not been successfully initialized
    /// `InvalidArg`, if `events` is invalid (shouldn't occur?)
    /// `NotSupported`, if the platform does not support this feature or some of the
    /// requested event types.
    /// `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    /// `Unknown`, on any unexpected error. **If this error is returned, the `set` you
    /// passed in has had its resources freed and will not be returned to you**. NVIDIA's
    /// docs say that this error means that the set is in an invalid state.
    ///
    /// # Device Support
    /// Supports Fermi and newer fully supported devices.
    ///
    /// # Platform Support
    /// Only supports Linux.
    // Checked against local
    // TODO: Is this a good way to handle the error cases here? (Unknown = should be freed)
    #[cfg(target_os = "linux")]
    #[inline]
    pub fn register_events(&self, events: EventTypes, set: EventSet) -> Result<EventSet> {
        unsafe {
            match nvml_try(nvmlDeviceRegisterEvents(self.device, 
                                                    events.bits() as c_ulonglong, 
                                                    set.c_set())) {
                // TODO: This is gonna be fun to figure out.
                // Ok(()) => Ok(set),
                Ok(()) => panic!(),
                Err(Error(ErrorKind::Unknown, _)) => {
                    // NVIDIA says that if an Unknown error is returned, `set` will
                    // be in an undefined state and should be freed.
                    // TODO: Something better to match on instead of string?
                    set.release_events().chain_err(|| "Error is from release call")?;
                    bail!(ErrorKind::Unknown)
                },
                // TODO: Figure out how to return set in error case
                Err(e) => Err(e)
            }
        }
    }

    /// Gets the `EventTypes` that this `Device` supports.
    ///
    /// # Errors
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    /// * `Unknown`, on any unexpected error
    ///
    /// # Device Support
    /// Supports Fermi and newer fully supported devices.
    ///
    /// # Platform Support
    /// Only supports Linux.
    // TODO: examples of interpreting the returned flags
    #[cfg(target_os = "linux")]
    #[inline]
    pub fn supported_event_types(&self) -> Result<EventTypes> {
        unsafe {
            let mut flags: c_ulonglong = mem::zeroed();
            nvml_try(nvmlDeviceGetSupportedEventTypes(self.device, &mut flags))?;

            if let Some(f) = EventTypes::from_bits(flags as u64) {
                Ok(f)
            } else {
                bail!(ErrorKind::IncorrectBits)
            }
        }
    }

    // Drain states

    /// Enable or disable drain state for this `Device`.
    ///
    /// Enabling drain state forces this `Device` to no longer accept new incoming requests.
    /// Any new NVML processes will no longer see this `Device`.
    /// 
    /// **`nvmlDeviceGetPciInfo` is also called the first time a drain-state-related 
    /// method is called in order to provide the drain call with the necessary PCI 
    /// information.** After the first call it is stored in this `Device` struct for future 
    /// calls. If you need to update the stored value for some reason, pass `true` for the 
    /// `update_storage` argument.
    ///
    /// Must be called as administrator. Persistence mode for this `Device` must be turned
    /// off before this call is made.
    ///
    /// # Errors
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `NotSupported`, if this `Device` doesn't support this feature
    /// * `NoPermission`, if the calling process has insufficient permissions to perform
    /// this operation
    /// * `InUse`, if this `Device` has persistence mode turned on
    /// * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    /// * `Unknown`, on any unexpected error
    ///
    /// # Device Support
    /// Supports Maxwell and newer fully supported devices.
    ///
    /// Some Kepler devices are also supported (that's all NVIDIA says, no specifics).
    ///
    /// # Platform Support
    /// Only supports Linux.
    // TODO: Should there be a separate method to update storage
    // Checked against local
    #[cfg(target_os = "linux")]
    #[inline]
    pub fn set_drain(&mut self, enabled: bool, update_storage: bool) -> Result<()> {
        unsafe {
            if update_storage || self.pci_info.is_none() {
                let mut pci_info: nvmlPciInfo_t = mem::zeroed();
                nvml_try(nvmlDeviceGetPciInfo_v2(self.device, &mut pci_info))
                    // TODO: Something better to match on here
                    .chain_err(|| "Error from nvmlDeviceGetPciInfo call")?;

                self.pci_info = Some(pci_info);
            }

            // Due to the above if, self.pci_info must be Some(), so we are free to unwrap here
            nvml_try(nvmlDeviceModifyDrainState(&mut self.pci_info.unwrap(), state_from_bool(enabled)))
        }
    }

    /// Query the drain state of this `Device`.
    ///
    /// **`nvmlDeviceGetPciInfo` is also called the first time a drain-state-related 
    /// method is called in order to provide the drain call with the necessary PCI 
    /// information.** After the first call it is stored in this `Device` struct for future 
    /// calls. If you need to update the stored value for some reason, pass `true` for the 
    /// `update_storage` argument.
    ///
    /// # Errors
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `NotSupported`, if this `Device` doesn't support this feature
    /// * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    /// * `Unknown`, on any unexpected error
    ///
    /// # Device Support
    /// Supports Maxwell and newer fully supported devices.
    ///
    /// Some Kepler devices are also supported (that's all NVIDIA says, no specifics).
    ///
    /// # Platform Support
    /// Only supports Linux.
    // Checked against local
    #[cfg(target_os = "linux")]
    #[inline]
    pub fn is_drain_enabled(&mut self, update_storage: bool) -> Result<bool> {
        unsafe {
            if update_storage || self.pci_info.is_none() {
                let mut pci_info: nvmlPciInfo_t = mem::zeroed();
                nvml_try(nvmlDeviceGetPciInfo_v2(self.device, &mut pci_info))
                    // TODO: Something better to match on here
                    .chain_err(|| "Error from nvmlDeviceGetPciInfo call")?;

                self.pci_info = Some(pci_info);
            }
            
            let mut state: nvmlEnableState_t = mem::zeroed();
            // Due to the above if, self.pci_info must be Some(), so we are free to unwrap here
            nvml_try(nvmlDeviceQueryDrainState(&mut self.pci_info.unwrap(), &mut state))?;

            Ok(bool_from_state(state))
        }
    }

    /// Removes this `Device` from the view of both NVML and the NVIDIA kernal driver.
    ///
    /// This call only works if no other processes are attached. If other processes
    /// are attached when this is called, the `InUse` error will be returned and
    /// this `Device` will return to its original draining state. The only situation
    /// where this can occur is if a process was and is still using this `Device`
    /// before the call to `set_drain()` was made and it was enabled. Note that
    /// persistence mode counts as an attachment to this `Device` and thus must be
    /// disabled prior to this call.
    ///
    /// For long-running NVML processes, please note that this will change the
    /// enumeration of current GPUs. As an example, if there are four GPUs present
    /// and the first is removed, the new enumeration will be 0-2. Device handles
    /// for the removed GPU will be invalid.
    ///
    /// Must be run as administrator.
    ///
    /// # Errors
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `NotSupported`, if this `Device` doesn't support this feature
    /// * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    /// * `InUse`, if this `Device` is still in use and cannot be removed
    ///
    /// # Device Support
    /// Supports Maxwell and newer fully supported devices.
    ///
    /// Some Kepler devices are also supported (that's all NVIDIA says, no specifics).
    ///
    /// # Platform Support
    /// Only supports Linux.
    // TODO: Figure out how to return device if handle is still valid
    // Not a blocker for release
    // Checked against local
    #[cfg(target_os = "linux")]
    #[inline]
    pub fn remove(mut self, update_storage: bool) -> Result<()> {
        unsafe {
            if update_storage || self.pci_info.is_none() {
                let mut pci_info: nvmlPciInfo_t = mem::zeroed();
                nvml_try(nvmlDeviceGetPciInfo_v2(self.device, &mut pci_info))
                    // TODO: Something better to match on here
                    .chain_err(|| "Error from nvmlDeviceGetPciInfo call")?;

                self.pci_info = Some(pci_info);
            }

            // Due to the above if, self.pci_info must be Some(), so we are free to unwrap here
            nvml_try(nvmlDeviceRemoveGpu(&mut self.pci_info.unwrap()))
        }
    }

    /// Only use this if it's absolutely necessary. 
    #[inline]
    pub fn c_device(&self) -> nvmlDevice_t {
        self.device
    }
}

#[cfg(test)]
#[cfg(feature = "test-local")]
#[allow(unused_variables, unused_imports)]
mod test {
    use NVML;
    use enum_wrappers::device::*;
    use test_utils::*;

    #[test]
    fn running_graphics_processes() {
        single(|nvml| {
            let device = device(&nvml, 0);
            device.running_graphics_processes(32).expect("graphics processes")
        });
    }

    #[test]
    fn running_graphics_processes_multiple() {
        multi(3, |nvml, i| {
            let device = device(&nvml, i);
            device.running_graphics_processes(32).expect(&format!("graphics processes {}", i));
        })
    }

    #[test]
    fn running_graphics_processes_multiple_threads() {
        multi_thread(3, |nvml, i| {
            let device = device(&nvml, i);
            device.running_graphics_processes(32).expect(&format!("graphics processes {}", i));
        });
    }

    #[test]
    fn running_graphics_processes_multiple_threads_arc() {
        multi_thread_arc(3, |nvml, i| {
            let device = device(&nvml, i);
            device.running_graphics_processes(32).expect(&format!("graphics processes {}", i));
        });
    }

    #[test]
    fn vbios_version() {
        single(|nvml| {
            let device = device(&nvml, 0);
            device.vbios_version().expect("version")
        });
    }

    #[test]
    fn vbios_version_multiple() {
        multi(3, |nvml, i| {
            let device = device(&nvml, i);
            device.vbios_version().expect(&format!("version {}", i));
        });
    }

    #[test]
    fn vbios_version_multiple_threads() {
        multi_thread(3, |nvml, i| {
            let device = device(&nvml, i);
            device.vbios_version().expect(&format!("version {}", i));
        });
    }

    #[test]
    fn vbios_version_multiple_threads_arc() {
        multi_thread_arc(3, |nvml, i| {
            let device = device(&nvml, i);
            device.vbios_version().expect(&format!("version {}", i));
        });
    }

    #[test]
    fn name() {
        single(|nvml| {
            let device = device(&nvml, 0);
            device.name().expect("Could not get name")
        });
    }

    #[test]
    fn name_multiple() {
        multi(3, |nvml, i| {
            let device = device(&nvml, i);
            device.name().expect(&format!("Could not get name{}", i));
        })
    }

    #[test]
    fn name_multiple_threads() {
        multi_thread(3, |nvml, i| {
            let device = device(&nvml, i);
            device.name().expect(&format!("Could not get name{}", i));
        });
    }

    #[test]
    fn name_multiple_threads_arc() {
        multi_thread_arc(3, |nvml, i| {
            let device = device(&nvml, i);
            device.name().expect(&format!("Could not get name{}", i));
        });
    }

    #[test]
    fn uuid() {
        single(|nvml| {
            let device = device(&nvml, 0);
            device.uuid().expect("uuid")
        });
    }

    #[test]
    fn uuid_multiple() {
        multi(3, |nvml, i| {
            let device = device(&nvml, i);
            device.uuid().expect(&format!("uuid {}", i));
        });
    }

    #[test]
    fn uuid_multiple_threads() {
        multi_thread(3, |nvml, i| {
            let device = device(&nvml, i);
            device.uuid().expect(&format!("uuid {}", i));
        });
    }

    #[test]
    fn uuid_multiple_threads_arc() {
        multi_thread_arc(3, |nvml, i| {
            let device = device(&nvml, i);
            device.uuid().expect(&format!("uuid {}", i));
        });
    }

    #[test]
    fn utilization_rates() {
        single(|nvml| {
            let device = device(&nvml, 0);
            device.utilization_rates().expect("rates")
        });
    }

    #[test]
    fn utilization_rates_multiple() {
        multi(3, |nvml, i| {
            let device = device(&nvml, i);
            device.utilization_rates().expect(&format!("rates {}", i));
        });
    }

    #[test]
    fn utilization_rates_multiple_threads() {
        multi_thread(3, |nvml, i| {
            let device = device(&nvml, i);
            device.utilization_rates().expect(&format!("rates {}", i));
        });
    }

    #[test]
    fn utilization_rates_multiple_threads_arc() {
        multi_thread_arc(3, |nvml, i| {
            let device = device(&nvml, i);
            device.utilization_rates().expect(&format!("rates {}", i));
        });
    }

    #[test]
    fn temperature_thresholds() {
        single(|nvml| {
            let device = device(&nvml, 0);
            let slowdown_temp = device.temperature_threshold(TemperatureThreshold::Slowdown)
                .expect("slowdown temp");
            let shutdown_temp = device.temperature_threshold(TemperatureThreshold::Shutdown)
                .expect("shutdown temp");

            #[cfg(feature = "test-local")]
            {
                assert_eq!(92, slowdown_temp);
                assert_eq!(97, shutdown_temp);
            }

            print!("\n\n\tGPU slows down at: {} \
                    \n\tGPU shuts down at: {}
                    \n\t... ",
                    slowdown_temp, shutdown_temp)
        });
    }

    #[test]
    fn temperature_thresholds_multiple() {
        multi(3, |nvml, i| {
            let device = device(&nvml, i);
            let slowdown_temp = device.temperature_threshold(TemperatureThreshold::Slowdown)
                .expect(&format!("slowdown temp {}", i));
            let shutdown_temp = device.temperature_threshold(TemperatureThreshold::Shutdown)
                .expect(&format!("shutdown temp {}", i));

            #[cfg(feature = "test-local")]
            {
                assert_eq!(92, slowdown_temp);
                assert_eq!(97, shutdown_temp);
            }
        });
    }

    #[test]
    fn temperature_thresholds_multiple_threads() {
        multi_thread(3, |nvml, i| {
            let device = device(&nvml, i);
            let slowdown_temp = device.temperature_threshold(TemperatureThreshold::Slowdown)
                .expect(&format!("slowdown temp {}", i));
            let shutdown_temp = device.temperature_threshold(TemperatureThreshold::Shutdown)
                .expect(&format!("shutdown temp {}", i));

            #[cfg(feature = "test-local")]
            {
                assert_eq!(92, slowdown_temp);
                assert_eq!(97, shutdown_temp);
            }
        });
    }

    #[test]
    fn temperature_thresholds_multiple_threads_arc() {
        multi_thread_arc(3, |nvml, i| {
            let device = device(&nvml, i);
            let slowdown_temp = device.temperature_threshold(TemperatureThreshold::Slowdown)
                .expect(&format!("slowdown temp {}", i));
            let shutdown_temp = device.temperature_threshold(TemperatureThreshold::Shutdown)
                .expect(&format!("shutdown temp {}", i));

            #[cfg(feature = "test-local")]
            {
                assert_eq!(92, slowdown_temp);
                assert_eq!(97, shutdown_temp);
            }
        });
    }

    #[test]
    fn temperature() {
        single(|nvml| {
            let device = device(&nvml, 0);
            device.temperature(TemperatureSensor::Gpu).expect("temp")
        });
    }

    #[test]
    fn temperature_multiple() {
        multi(3, |nvml, i| {
            let device = device(&nvml, i);
            device.temperature(TemperatureSensor::Gpu).expect(&format!("temp {}", i));
        });
    }

    #[test]
    fn temperature_multiple_threads() {
        multi_thread(3, |nvml, i| {
            let device = device(&nvml, i);
            device.temperature(TemperatureSensor::Gpu).expect(&format!("temp {}", i));
        });
    }

    #[test]
    fn temperature_multiple_threads_arc() {
        multi_thread_arc(3, |nvml, i| {
            let device = device(&nvml, i);
            device.temperature(TemperatureSensor::Gpu).expect(&format!("temp {}", i));
        });
    }

    #[test]
    fn brand() {
        single(|nvml| {
            let device = device(&nvml, 0);
            device.brand().expect("Could not get brand")
        });
    }

    #[test]
    fn brand_multiple() {
        multi(3, |nvml, i| {
            let device = device(&nvml, i);
            device.brand().expect(&format!("Could not get brand{}", i));
        });
    }

    #[test]
    fn brand_multiple_threads() {
        multi_thread(3, |nvml, i| {
            let device = device(&nvml, i);
            device.brand().expect(&format!("Could not get brand{}", i));
        });
    }

    #[test]
    fn brand_multiple_threads_arc() {
        multi_thread_arc(3, |nvml, i| {
            let device = device(&nvml, i);
            device.brand().expect(&format!("Could not get brand{}", i));
        });
    }

    #[test]
    fn clock_info() {
        single(|nvml| {
            let device = device(&nvml, 0);
            let gfx_clock = device.clock_info(Clock::Graphics).expect("Could not get gfx clock");
            let mem_clock = device.clock_info(Clock::Memory).expect("Could not get mem clock");
            let sm_clock = device.clock_info(Clock::SM).expect("Could not get sm clock");
            let vid_clock = device.clock_info(Clock::Video).expect("Could not get vid clock");

            print!("\n\n\tGraphics Clock: {:?} MHz \
                    \n\tMemory Clock: {:?} MHz \
                    \n\tStreaming Multiprocessor Clock: {:?} MHz \
                    \n\tVideo Clock: {:?} MHz
                    \n\t... ",
                    gfx_clock,
                    mem_clock,
                    sm_clock,
                    vid_clock)
        });
    }

    #[test]
    fn clock_info_multiple() {
        multi(3, |nvml, i| {
            let device = device(&nvml, 0);
            let gfx_clock = device.clock_info(Clock::Graphics)
                .expect(&format!("Could not get gfx clock{}", i));
            let mem_clock = device.clock_info(Clock::Memory)
                .expect(&format!("Could not get mem clock{}", i));
            let sm_clock = device.clock_info(Clock::SM)
                .expect(&format!("Could not get sm clock{}", i));
            let vid_clock = device.clock_info(Clock::Video)
                .expect(&format!("Could not get vid clock{}", i));
        });
    }

    #[test]
    fn clock_info_multiple_threads() {
        multi_thread(3, |nvml, i| {
            let device = device(&nvml, 0);
            let gfx_clock = device.clock_info(Clock::Graphics)
                .expect(&format!("Could not get gfx clock{}", i));
            let mem_clock = device.clock_info(Clock::Memory)
                .expect(&format!("Could not get mem clock{}", i));
            let sm_clock = device.clock_info(Clock::SM)
                .expect(&format!("Could not get sm clock{}", i));
            let vid_clock = device.clock_info(Clock::Video)
                .expect(&format!("Could not get vid clock{}", i));
        });
    }

    #[test]
    fn clock_info_multiple_threads_arc() {
        multi_thread_arc(3, |nvml, i| {
            let device = device(&nvml, 0);
            let gfx_clock = device.clock_info(Clock::Graphics)
                .expect(&format!("Could not get gfx clock{}", i));
            let mem_clock = device.clock_info(Clock::Memory)
                .expect(&format!("Could not get mem clock{}", i));
            let sm_clock = device.clock_info(Clock::SM)
                .expect(&format!("Could not get sm clock{}", i));
            let vid_clock = device.clock_info(Clock::Video)
                .expect(&format!("Could not get vid clock{}", i));
        });
    }

    #[test]
    fn max_clock_info() {
        single(|nvml| {
            let device = device(&nvml, 0);
            let max_gfx_clock = device.max_clock_info(Clock::Graphics).expect("Could not get gfx clock");
            let max_mem_clock = device.max_clock_info(Clock::Memory).expect("Could not get mem clock");
            let max_sm_clock = device.max_clock_info(Clock::SM).expect("Could not get sm clock");
            let max_vid_clock = device.max_clock_info(Clock::Video).expect("Could not get vid clock");

            print!("\n\n\tMax Graphics Clock: {:?} MHz \
                    \n\tMax Memory Clock: {:?} MHz \
                    \n\tMax Streaming Multiprocessor Clock: {:?} MHz \
                    \n\tMax Video Clock: {:?} MHz
                    \n\t... ",
                    max_gfx_clock,
                    max_mem_clock,
                    max_sm_clock,
                    max_vid_clock)
        });
    }

    #[test]
    fn max_clock_info_multiple() {
        multi(3, |nvml, i| {
            let device = device(&nvml, 0);
            let max_gfx_clock = device.max_clock_info(Clock::Graphics)
                .expect(&format!("Could not get gfx clock{}", i));
            let max_mem_clock = device.max_clock_info(Clock::Memory)
                .expect(&format!("Could not get mem clock{}", i));
            let max_sm_clock = device.max_clock_info(Clock::SM)
                .expect(&format!("Could not get sm clock{}", i));
            let max_vid_clock = device.max_clock_info(Clock::Video)
                .expect(&format!("Could not get vid clock{}", i));
        })
    }

    #[test]
    fn max_clock_info_multiple_threads() {
        multi_thread(3, |nvml, i| {
            let device = device(&nvml, 0);
            let max_gfx_clock = device.max_clock_info(Clock::Graphics)
                .expect(&format!("Could not get gfx clock{}", i));
            let max_mem_clock = device.max_clock_info(Clock::Memory)
                .expect(&format!("Could not get mem clock{}", i));
            let max_sm_clock = device.max_clock_info(Clock::SM)
                .expect(&format!("Could not get sm clock{}", i));
            let max_vid_clock = device.max_clock_info(Clock::Video)
                .expect(&format!("Could not get vid clock{}", i));
        });
    }

    #[test]
    fn max_clock_info_multiple_threads_arc() {
        multi_thread_arc(3, |nvml, i| {
            let device = device(&nvml, 0);
            let max_gfx_clock = device.max_clock_info(Clock::Graphics)
                .expect(&format!("Could not get gfx clock{}", i));
            let max_mem_clock = device.max_clock_info(Clock::Memory)
                .expect(&format!("Could not get mem clock{}", i));
            let max_sm_clock = device.max_clock_info(Clock::SM)
                .expect(&format!("Could not get sm clock{}", i));
            let max_vid_clock = device.max_clock_info(Clock::Video)
                .expect(&format!("Could not get vid clock{}", i));
        });
    }

    #[test]
    fn is_api_restricted() {
        single(|nvml| {
            let device = device(&nvml, 0);
            device.is_api_restricted(Api::ApplicationClocks).expect("boolean")

            // My GPU apparently does not support AutoBoostedClocks...
        });
    }

    #[test]
    fn is_api_restricted_multiple() {
        multi(3, |nvml, i| {
            let device = device(&nvml, i);
            device.is_api_restricted(Api::ApplicationClocks).expect(&format!("boolean {}", i));
        });
    }

    #[test]
    fn is_api_restricted_multiple_threads() {
        multi_thread(3, |nvml, i| {
            let device = device(&nvml, i);
            device.is_api_restricted(Api::ApplicationClocks).expect(&format!("boolean {}", i));
        });
    }

    #[test]
    fn is_api_restricted_multiple_threads_arc() {
        multi_thread_arc(3, |nvml, i| {
            let device = device(&nvml, i);
            device.is_api_restricted(Api::ApplicationClocks).expect(&format!("boolean {}", i));
        });
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

    #[test]
    fn applications_clock() {
        let test = NVML::init().expect("init call failed");
        let device = test.device_by_index(0).expect("Could not get a device by index 0");
        let clock = device.applications_clock(Clock::Graphics).expect("Could not get applications clock");
    }
}
