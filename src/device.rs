#[cfg(target_os = "linux")]
use EventSet;
use NVML;
use NvLink;
#[cfg(target_os = "windows")]
use bitmasks::Behavior;
use bitmasks::device::ThrottleReasons;
#[cfg(target_os = "linux")]
use bitmasks::event::EventTypes;
use enum_wrappers::{state_from_bool, bool_from_state};
use enum_wrappers::device::*;
#[cfg(target_os = "linux")]
use error::ResultExt;
use error::{Bits, nvml_try, Result, ErrorKind, Error};
use ffi::bindings::*;
use std::ffi::CStr;
use std::marker::PhantomData;
use std::mem;
#[cfg(target_os = "linux")]
use std::os::raw::c_ulong;
use std::os::raw::{c_int, c_uint, c_ulonglong};
use std::ptr;
use struct_wrappers::device::*;
use structs::device::*;

/**
Struct that represents a device on the system. 

Obtain a `Device` with the various methods available to you on the `NVML`
struct.

Rust's lifetimes will ensure that the NVML instance this `Device` was created from is
not allowed to be shutdown until this `Device` is dropped, meaning you shouldn't
have to worry about calls returning `Uninitialized` errors.
*/
// TODO: Use compiletest to ensure lifetime guarantees
#[derive(Debug)]
pub struct Device<'nvml> {
    device: nvmlDevice_t,
    _phantom: PhantomData<&'nvml NVML>
}

unsafe impl<'nvml> Send for Device<'nvml> {}
unsafe impl<'nvml> Sync for Device<'nvml> {}

impl<'nvml> From<nvmlDevice_t> for Device<'nvml> {
    fn from(device: nvmlDevice_t) -> Self {
        Device {
            device,
            _phantom: PhantomData
        }
    }
}

impl<'nvml> Device<'nvml> {
    /**
    Clear all affinity bindings for the calling thread.
    
    Note that this was changed as of version 8.0; older versions cleared affinity for 
    the calling process and all children. 
    
    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `InvalidArg`, if this `Device` is invalid
    * `Unknown`, on any unexpected error
    
    # Device Support

    Supports Kepler or newer fully supported devices.
    
    # Platform Support

    Only supports Linux. 
    */
    // Checked against local
    // Tested (no-run)
    #[cfg(target_os = "linux")]
    #[inline]
    pub fn clear_cpu_affinity(&mut self) -> Result<()> {
        unsafe { nvml_try(nvmlDeviceClearCpuAffinity(self.device)) }
    }

    /**
    Gets the root/admin permissions for the target API.
    
    Only root users are able to call functions belonging to restricted APIs. See 
    the documentation for the `RestrictedApi` enum for a list of those functions.
    
    Non-root users can be granted access to these APIs through use of
    `.set_api_restricted()`.
    
    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `InvalidArg`, if this `Device` is invalid or the apiType is invalid (may occur if 
    the C lib changes dramatically?)
    * `NotSupported`, if this query is not supported by this `Device` or this `Device`
    does not support the feature that is being queried (e.g. enabling/disabling auto
    boosted clocks is not supported by this `Device`).
    * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    * `UnexpectedVariant`, for which you can read the docs for
    * `Unknown`, on any unexpected error
    
    # Device Support

    Supports all _fully supported_ products.
    */
    // Checked against local
    // Tested (except for AutoBoostedClocks)
    #[inline]
    pub fn is_api_restricted(&self, api: Api) -> Result<bool> {
        unsafe {
            let mut restricted_state: nvmlEnableState_t = mem::zeroed();

            nvml_try(nvmlDeviceGetAPIRestriction(
                self.device,
                api.as_c(),
                &mut restricted_state
            ))?;

            Ok(bool_from_state(restricted_state)?)
        }
    }

    /**
    Gets the current clock setting that all applications will use unless an overspec 
    situation occurs.
    
    This setting can be changed using `.set_applications_clocks()`.
    
    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `InvalidArg`, if this `Device` is invalid or the clockType is invalid (may occur 
    if the C lib changes dramatically?)
    * `NotSupported`, if this `Device` does not support this feature
    * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    * `Unknown`, on any unexpected error
    
    # Device Support

    Supports Kepler or newer fully supported devices.
    */
    // Checked against local
    // Tested
    #[inline]
    pub fn applications_clock(&self, clock_type: Clock) -> Result<u32> {
        unsafe {
            let mut clock: c_uint = mem::zeroed();

            nvml_try(nvmlDeviceGetApplicationsClock(
                self.device,
                clock_type.as_c(),
                &mut clock
            ))?;

            Ok(clock)
        }
    }

    /**
    Gets the current and default state of auto boosted clocks.
    
    Auto boosted clocks are enabled by default on some hardware, allowing the GPU to run
    as fast as thermals will allow it to. 
    
    On Pascal and newer hardware, auto boosted clocks are controlled through application
    clocks. Use `.set_applications_clocks()` and `.reset_applications_clocks()` to control
    auto boost behavior.
    
    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `InvalidArg`, if this `Device` is invalid
    * `NotSupported`, if this `Device` does not support auto boosted clocks
    * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    * `UnexpectedVariant`, for which you can read the docs for
    * `Unknown`, on any unexpected error
    
    # Device Support

    Supports Kepler or newer fully supported devices.
    */
    // Checked against local
    // Tested on machines other than my own
    #[inline]
    pub fn auto_boosted_clocks_enabled(&self) -> Result<AutoBoostClocksEnabledInfo> {
        unsafe {
            let mut is_enabled: nvmlEnableState_t = mem::zeroed();
            let mut is_enabled_default: nvmlEnableState_t = mem::zeroed();

            nvml_try(nvmlDeviceGetAutoBoostedClocksEnabled(
                self.device,
                &mut is_enabled,
                &mut is_enabled_default
            ))?;

            Ok(AutoBoostClocksEnabledInfo {
                is_enabled: bool_from_state(is_enabled)?,
                is_enabled_default: bool_from_state(is_enabled_default)?
            })
        }
    }

    /**
    Gets the total, available and used size of BAR1 memory. 
    
    BAR1 memory is used to map the FB (device memory) so that it can be directly accessed
    by the CPU or by 3rd party devices (peer-to-peer on the PCIe bus).
    
    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `InvalidArg`, if this `Device` is invalid
    * `NotSupported`, if this `Device` does not support this query
    * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    * `Unknown`, on any unexpected error
    
    # Device Support

    Supports Kepler or newer fully supported devices.
    */
    // Checked against local
    // Tested
    #[inline]
    pub fn bar1_memory_info(&self) -> Result<BAR1MemoryInfo> {
        unsafe {
            let mut mem_info: nvmlBAR1Memory_t = mem::zeroed();
            nvml_try(nvmlDeviceGetBAR1MemoryInfo(self.device, &mut mem_info))?;

            Ok(mem_info.into())
        }
    }

    /**
    Gets the board ID for this `Device`, from 0-N. 
    
    Devices with the same boardID indicate GPUs connected to the same PLX. Use in
    conjunction with `.is_multi_gpu_board()` to determine if they are on the same
    board as well. 
    
    The boardID returned is a unique ID for the current config. Uniqueness and
    ordering across reboots and system configs is not guaranteed (i.e if a Tesla
    K40c returns 0x100 and the two GPUs on a Tesla K10 in the same system return
    0x200, it is not guaranteed that they will always return those values. They will,
    however, always be different from each other).
    
    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `InvalidArg`, if this `Device` is invalid
    * `NotSupported`, if this `Device` does not support this feature
    * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    * `Unknown`, on any unexpected error
    
    # Device Support

    Supports Fermi or newer fully supported devices.
    */
    // Checked against local
    // Tested
    #[inline]
    pub fn board_id(&self) -> Result<u32> {
        unsafe {
            let mut id: c_uint = mem::zeroed();
            nvml_try(nvmlDeviceGetBoardId(self.device, &mut id))?;

            Ok(id)
        }
    }

    /**
    Gets the brand of this `Device`.
    
    See the `Brand` enum for documentation of possible values.
    
    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `InvalidArg`, if this `Device` is invalid
    * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    * `UnexpectedVariant`, check that error's docs for more info
    * `Unknown`, on any unexpected error
    */
    // Checked against local nvml.h
    // Tested
    #[inline]
    pub fn brand(&self) -> Result<Brand> {
        unsafe {
            let mut brand: nvmlBrandType_t = mem::zeroed();
            nvml_try(nvmlDeviceGetBrand(self.device, &mut brand))?;

            Ok(Brand::try_from(brand)?)
        }
    }

    /**
    Gets bridge chip information for all bridge chips on the board. 
    
    Only applicable to multi-GPU devices.
    
    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `InvalidArg`, if this `Device` is invalid
    * `NotSupported`, if this `Device` does not support this feature
    * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    * `UnexpectedVariant`, for which you can read the docs for
    * `Unknown`, on any unexpected error
    
    # Device Support

    Supports all _fully supported_ devices.
    */
    // Checked against local
    // Tested on machines other than my own
    #[inline]
    pub fn bridge_chip_info(&self) -> Result<BridgeChipHierarchy> {
        unsafe {
            let mut info: nvmlBridgeChipHierarchy_t = mem::zeroed();
            nvml_try(nvmlDeviceGetBridgeChipInfo(self.device, &mut info))?;

            Ok(BridgeChipHierarchy::try_from(info)?)
        }
    }

    /**
    Gets this `Device`'s current clock speed for the given `Clock` type and `ClockId`.
    
    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `InvalidArg`, if this `Device` is invalid or `clock_type` is invalid (shouldn't occur?)
    * `NotSupported`, if this `Device` does not support this feature
    * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    * `Unknown`, on any unexpected error
    
    # Device Support

    Supports Kepler and newer fully supported devices.
    */
    // Checked against local
    // Tested (except for CustomerMaxBoost)
    #[inline]
    pub fn clock(&self, clock_type: Clock, clock_id: ClockId) -> Result<u32> {
        unsafe {
            let mut clock: c_uint = mem::zeroed();

            nvml_try(nvmlDeviceGetClock(
                self.device,
                clock_type.as_c(),
                clock_id.as_c(),
                &mut clock
            ))?;

            Ok(clock)
        }
    }

    /**
    Gets this `Device`'s customer-defined maximum boost clock speed for the
    given `Clock` type.
    
    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `InvalidArg`, if this `Device` is invalid or `clock_type` is invalid (shouldn't occur?)
    * `NotSupported`, if this `Device` or the `clock_type` on this `Device`
    does not support this feature
    * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    * `Unknown`, on any unexpected error
    
    # Device Support

    Supports Pascal and newer fully supported devices.
    */
    // Checked against local
    // Tested on machines other than my own
    #[inline]
    pub fn max_customer_boost_clock(&self, clock_type: Clock) -> Result<u32> {
        unsafe {
            let mut clock: c_uint = mem::zeroed();

            nvml_try(nvmlDeviceGetMaxCustomerBoostClock(
                self.device,
                clock_type.as_c(),
                &mut clock
            ))?;

            Ok(clock)
        }
    }

    /**
    Gets the current compute mode for this `Device`.
    
    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `InvalidArg`, if this `Device` is invalid
    * `NotSupported`, if this `Device` does not support this feature
    * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    * `UnexpectedVariant`, check that error's docs for more info
    * `Unknown`, on any unexpected error
    */
    // Checked against local
    // Tested
    #[inline]
    pub fn compute_mode(&self) -> Result<ComputeMode> {
        unsafe {
            let mut mode: nvmlComputeMode_t = mem::zeroed();
            nvml_try(nvmlDeviceGetComputeMode(self.device, &mut mode))?;

            Ok(ComputeMode::try_from(mode)?)
        }
    }

    /**
    Gets the CUDA compute capability of this `Device`.

    The returned version numbers are the same as those returned by
    `cuDeviceGetAttribute()` from the CUDA API.

    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `InvalidArg`, if this `Device` is invalid
    * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    * `Unknown`, on any unexpected error
    */
    #[inline]
    pub fn cuda_compute_capability(&self) -> Result<CudaComputeCapability> {
        unsafe {
            let mut major: c_int = mem::zeroed();
            let mut minor: c_int = mem::zeroed();

            nvml_try(nvmlDeviceGetCudaComputeCapability(
                self.device,
                &mut major,
                &mut minor
            ))?;

            Ok(CudaComputeCapability {
                major,
                minor
            })
        }
    }

    /**
    Gets this `Device`'s current clock speed for the given `Clock` type.
    
    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `InvalidArg`, if this `Device` is invalid
    * `NotSupported`, if this `Device` cannot report the specified clock
    * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    * `Unknown`, on any unexpected error
    
    # Device Support

    Supports Fermi or newer fully supported devices.
    */
    // Checked against local
    // Tested
    #[inline]
    pub fn clock_info(&self, clock_type: Clock) -> Result<u32> {
        unsafe {
            let mut clock: c_uint = mem::zeroed();

            nvml_try(nvmlDeviceGetClockInfo(
                self.device,
                clock_type.as_c(),
                &mut clock
            ))?;

            Ok(clock)
        }
    }

    /**
    Gets information about processes with a compute context running on this `Device`.
    
    This only returns information about running compute processes (such as a CUDA application
    with an active context). Graphics applications (OpenGL, DirectX) won't be listed by this
    function.
    
    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `InvalidArg`, if this `Device` is invalid
    * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    * `Unknown`, on any unexpected error
    */
    // Tested
    #[inline]
    pub fn running_compute_processes(&self) -> Result<Vec<ProcessInfo>> {
        unsafe {
            let mut count: c_uint = match self.running_compute_processes_count()? {
                0 => return Ok(vec![]),
                value => value,
            };
            let mut processes: Vec<nvmlProcessInfo_t> = vec![mem::zeroed(); count as usize];

            nvml_try(nvmlDeviceGetComputeRunningProcesses(
                self.device,
                &mut count,
                processes.as_mut_ptr()
            ))?;

            Ok(processes.into_iter().map(ProcessInfo::from).collect())
        }
    }

    /**
    Gets the number of processes with a compute context running on this `Device`.
    
    This only returns the count of running compute processes (such as a CUDA application
    with an active context). Graphics applications (OpenGL, DirectX) won't be counted by this
    function.
    
    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `InvalidArg`, if this `Device` is invalid
    * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    * `Unknown`, on any unexpected error
    */
    // Tested as part of `.running_compute_processes()`
    #[inline]
    pub fn running_compute_processes_count(&self) -> Result<u32> {
        unsafe {
            // Indicates that we want the count
            let mut count: c_uint = 0;

            // Passing null doesn't mean we want the count, it's just allowed
            match nvmlDeviceGetComputeRunningProcesses(self.device, &mut count, ptr::null_mut()) {
                nvmlReturn_enum_NVML_ERROR_INSUFFICIENT_SIZE => Ok(count),
                // If success, return 0; otherwise, return error
                other => nvml_try(other).map(|_| 0),
            }
        }
    }

    /**
    Gets a vector of bitmasks with the ideal CPU affinity for this `Device`.
    
    The results are sized to `size`. For example, if processors 0, 1, 32, and 33 are
    ideal for this `Device` and `size` == 2, result[0] = 0x3, result[1] = 0x3.

    64 CPUs per unsigned long on 64-bit machines, 32 on 32-bit machines.
    
    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `InvalidArg`, if this `Device` is invalid
    * `InsufficientSize`, if the passed-in `size` is 0 (must be > 0)
    * `NotSupported`, if this `Device` does not support this feature
    * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    * `Unknown`, on any unexpected error
    
    # Device Support

    Supports Kepler or newer fully supported devices.
    
    # Platform Support

    Only supports Linux.
    */
    // Checked against local
    // Tested
    // TODO: Should we trim zeros here or leave it to the caller?
    #[cfg(target_os = "linux")]
    #[inline]
    pub fn cpu_affinity(&self, size: usize) -> Result<Vec<c_ulong>> {
        unsafe {
            if size == 0 {
                // Return an error containing the minimum size that can be passed.
                bail!(ErrorKind::InsufficientSize(Some(1)));
            }

            let mut affinities: Vec<c_ulong> = vec![mem::zeroed(); size];

            nvml_try(nvmlDeviceGetCpuAffinity(
                self.device,
                size as c_uint,
                affinities.as_mut_ptr()
            ))?;

            Ok(affinities)
        }
    }

    /**
    Gets the current PCIe link generation.
    
    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `InvalidArg`, if this `Device` is invalid
    * `NotSupported`, if PCIe link information is not available
    * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    * `Unknown`, on any unexpected error
    
    # Device Support

    Supports Fermi or newer fully supported devices.
    */
    // Checked against local
    // Tested
    #[inline]
    pub fn current_pcie_link_gen(&self) -> Result<u32> {
        unsafe {
            let mut link_gen: c_uint = mem::zeroed();

            nvml_try(nvmlDeviceGetCurrPcieLinkGeneration(
                self.device,
                &mut link_gen
            ))?;

            Ok(link_gen)
        }
    }

    /**
    Gets the current PCIe link width.
    
    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `InvalidArg`, if this `Device` is invalid
    * `NotSupported`, if PCIe link information is not available
    * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    * `Unknown`, on any unexpected error
    
    # Device Support

    Supports Fermi or newer fully supported devices.
    */
    // Checked against local
    // Tested
    #[inline]
    pub fn current_pcie_link_width(&self) -> Result<u32> {
        unsafe {
            let mut link_width: c_uint = mem::zeroed();
            nvml_try(nvmlDeviceGetCurrPcieLinkWidth(self.device, &mut link_width))?;

            Ok(link_width)
        }
    }

    /**
    Gets the current utilization and sampling size (sampling size in μs) for the Decoder.
    
    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `InvalidArg`, if this `Device` is invalid
    * `NotSupported`, if this `Device` does not support this feature
    * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    * `Unknown`, on any unexpected error
    
    # Device Support

    Supports Kepler or newer fully supported devices.
    */
    // Checked against local
    // Tested
    #[inline]
    pub fn decoder_utilization(&self) -> Result<UtilizationInfo> {
        unsafe {
            let mut utilization: c_uint = mem::zeroed();
            let mut sampling_period: c_uint = mem::zeroed();

            nvml_try(nvmlDeviceGetDecoderUtilization(
                self.device,
                &mut utilization,
                &mut sampling_period
            ))?;

            Ok(UtilizationInfo {
                utilization,
                sampling_period
            })
        }
    }

    /**
    Gets the default applications clock that this `Device` boots with or defaults to after
    `reset_applications_clocks()`.
    
    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `InvalidArg`, if this `Device` is invalid
    * `NotSupported`, if this `Device` does not support this feature
    * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    * `Unknown`, on any unexpected error
    
    # Device Support

    Supports Kepler or newer fully supported devices.
    */
    // Checked against local
    // Tested
    #[inline]
    pub fn default_applications_clock(&self, clock_type: Clock) -> Result<u32> {
        unsafe {
            let mut clock: c_uint = mem::zeroed();

            nvml_try(nvmlDeviceGetDefaultApplicationsClock(
                self.device,
                clock_type.as_c(),
                &mut clock
            ))?;

            Ok(clock)
        }
    }

    /// Not documenting this because it's deprecated. Read NVIDIA's docs if you
    /// must use it.
    #[deprecated(note = "use `Device.memory_error_counter()`")]
    #[inline]
    pub fn detailed_ecc_errors(
        &self,
        error_type: MemoryError,
        counter_type: EccCounter,
    ) -> Result<EccErrorCounts> {
        unsafe {
            let mut counts: nvmlEccErrorCounts_t = mem::zeroed();

            nvml_try(nvmlDeviceGetDetailedEccErrors(
                self.device,
                error_type.as_c(),
                counter_type.as_c(),
                &mut counts
            ))?;

            Ok(counts.into())
        }
    }

    /**
    Gets the display active state for this `Device`. 
    
    This method indicates whether a display is initialized on this `Device`.
    For example, whether or not an X Server is attached to this device and
    has allocated memory for the screen.
    
    A display can be active even when no monitor is physically attached to this `Device`.
    
    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `InvalidArg`, if this `Device` is invalid
    * `NotSupported`, if this `Device` does not support this feature
    * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    * `UnexpectedVariant`, for which you can read the docs for
    * `Unknown`, on any unexpected error
    */
    // Checked against local
    // Tested
    #[inline]
    pub fn is_display_active(&self) -> Result<bool> {
        unsafe {
            let mut state: nvmlEnableState_t = mem::zeroed();
            nvml_try(nvmlDeviceGetDisplayActive(self.device, &mut state))?;

            Ok(bool_from_state(state)?)
        }
    }

    /**
    Gets whether a physical display is currently connected to any of this `Device`'s
    connectors.
    
    This calls the C function `nvmlDeviceGetDisplayMode`.
    
    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `InvalidArg`, if this `Device` is invalid
    * `NotSupported`, if this `Device` does not support this feature
    * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    * `UnexpectedVariant`, for which you can read the docs for
    * `Unknown`, on any unexpected error
    */
    // Checked against local
    // Tested
    #[inline]
    pub fn is_display_connected(&self) -> Result<bool> {
        unsafe {
            let mut state: nvmlEnableState_t = mem::zeroed();
            nvml_try(nvmlDeviceGetDisplayMode(self.device, &mut state))?;

            Ok(bool_from_state(state)?)
        }
    }

    /**
    Gets the current and pending driver model for this `Device`.
    
    On Windows, the device driver can run in either WDDM or WDM (TCC) modes.
    If a display is attached to the device it must run in WDDM mode. TCC mode
    is preferred if a display is not attached.
    
    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `InvalidArg`, if this `Device` is invalid
    * `NotSupported`, if the platform is not Windows
    * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    * `UnexpectedVariant`, for which you can read the docs for
    * `Unknown`, on any unexpected error
    
    # Device Support

    Supports Fermi and newer fully supported devices.
    
    # Platform Support

    Only supports Windows.
    */
    // Checked against local
    // Tested
    #[cfg(target_os = "windows")]
    #[inline]
    pub fn driver_model(&self) -> Result<DriverModelState> {
        unsafe {
            let mut current: nvmlDriverModel_t = mem::zeroed();
            let mut pending: nvmlDriverModel_t = mem::zeroed();

            nvml_try(nvmlDeviceGetDriverModel(
                self.device,
                &mut current,
                &mut pending
            ))?;

            Ok(DriverModelState {
                current: DriverModel::try_from(current)?,
                pending: DriverModel::try_from(pending)?
            })
        }
    }

    /**
    Get the current and pending ECC modes for this `Device`.
    
    Changing ECC modes requires a reboot. The "pending" ECC mode refers to the target
    mode following the next reboot.
    
    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `InvalidArg`, if this `Device` is invalid
    * `NotSupported`, if this `Device` does not support this feature
    * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    * `UnexpectedVariant`, for which you can read the docs for
    * `Unknown`, on any unexpected error
    
    # Device Support

    Supports Fermi and newer fully supported devices. Only applicable to devices with
    ECC. Requires `InfoRom::ECC` version 1.0 or higher.
    */
    // Checked against local
    // Tested on machines other than my own
    #[inline]
    pub fn is_ecc_enabled(&self) -> Result<EccModeState> {
        unsafe {
            let mut current: nvmlEnableState_t = mem::zeroed();
            let mut pending: nvmlEnableState_t = mem::zeroed();

            nvml_try(nvmlDeviceGetEccMode(
                self.device,
                &mut current,
                &mut pending
            ))?;

            Ok(EccModeState {
                currently_enabled: bool_from_state(current)?,
                pending_enabled: bool_from_state(pending)?
            })
        }
    }

    /**
    Gets the current utilization and sampling size (sampling size in μs) for the Encoder.
    
    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `InvalidArg`, if this `Device` is invalid
    * `NotSupported`, if this `Device` does not support this feature
    * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    * `Unknown`, on any unexpected error
    
    # Device Support

    Supports Kepler or newer fully supported devices.
    */
    // Checked against local
    // Tested
    #[inline]
    pub fn encoder_utilization(&self) -> Result<UtilizationInfo> {
        unsafe {
            let mut utilization: c_uint = mem::zeroed();
            let mut sampling_period: c_uint = mem::zeroed();

            nvml_try(nvmlDeviceGetEncoderUtilization(
                self.device,
                &mut utilization,
                &mut sampling_period
            ))?;

            Ok(UtilizationInfo {
                utilization,
                sampling_period
            })
        }
    }

    /**
    Gets the current capacity of this device's encoder in macroblocks per second.

    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `InvalidArg`, if this device is invalid
    * `NotSupported`, if this `Device` does not support the given `for_type`
    * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    * `Unknown`, on any unexpected error

    # Device Support

    Supports Maxwell or newer fully supported devices.
    */
    // Tested
    #[inline]
    pub fn encoder_capacity(&self, for_type: EncoderType) -> Result<u32> {
        unsafe {
            let mut capacity: c_uint = mem::zeroed();

            nvml_try(nvmlDeviceGetEncoderCapacity(
                self.device,
                for_type.as_c(),
                &mut capacity
            ))?;

            Ok(capacity)
        }
    }

    /**
    Gets the current encoder stats for this device.

    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `InvalidArg`, if this device is invalid
    * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    * `Unknown`, on any unexpected error

    # Device Support

    Supports Maxwell or newer fully supported devices.
    */
    // Tested
    #[inline]
    pub fn encoder_stats(&self) -> Result<EncoderStats> {
        unsafe {
            let mut session_count: c_uint = mem::zeroed();
            let mut average_fps: c_uint = mem::zeroed();
            let mut average_latency: c_uint = mem::zeroed();

            nvml_try(nvmlDeviceGetEncoderStats(
                self.device,
                &mut session_count,
                &mut average_fps,
                &mut average_latency
            ))?;

            Ok(EncoderStats {
                session_count,
                average_fps,
                average_latency
            })
        }
    }

    /**
    Gets information about active encoder sessions on this device.

    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    * `UnexpectedVariant`, if an enum variant not defined in this wrapper gets
    returned in a field of an `EncoderSessionInfo` struct
    * `Unknown`, on any unexpected error

    # Device Support

    Supports Maxwell or newer fully supported devices.
    */
    // Tested
    // TODO: Test this with an active session and make sure it works
    #[inline]
    pub fn encoder_sessions(&self) -> Result<Vec<EncoderSessionInfo>> {
        unsafe {
            let mut count = match self.encoder_sessions_count()? {
                0 => return Ok(vec![]),
                value => value
            };
            let mut sessions: Vec<nvmlEncoderSessionInfo_t> =
                vec![mem::zeroed(); count as usize];

            nvml_try(nvmlDeviceGetEncoderSessions(
                self.device,
                &mut count,
                sessions.as_mut_ptr()
            ))?;

            sessions.truncate(count as usize);
            Ok(
                sessions
                    .into_iter()
                    .map(EncoderSessionInfo::try_from)
                    .collect::<Result<_>>()?
            )
        }
    }

    // Helper for the above function. Returns # of sessions that can be queried.
    fn encoder_sessions_count(&self) -> Result<c_uint> {
        unsafe {
            let mut count: c_uint = 0;

            nvml_try(nvmlDeviceGetEncoderSessions(
                self.device,
                &mut count,
                ptr::null_mut()
            ))?;

            Ok(count)
        }
    }

    /**
    Gets the effective power limit in milliwatts that the driver enforces after taking
    into account all limiters.
    
    Note: This can be different from the `.power_management_limit()` if other limits
    are set elswhere. This includes the out-of-band power limit interface.
    
    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `InvalidArg`, if this `Device` is invalid
    * `NotSupported`, if this `Device` does not support this feature
    * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    * `Unknown`, on any unexpected error
    
    # Device Support

    Supports Kepler or newer fully supported devices.
    */
    // Checked against local
    // Tested
    #[inline]
    pub fn enforced_power_limit(&self) -> Result<u32> {
        unsafe {
            let mut limit: c_uint = mem::zeroed();
            nvml_try(nvmlDeviceGetEnforcedPowerLimit(self.device, &mut limit))?;

            Ok(limit)
        }
    }

    /**
    Gets the intended operating speed of this `Device`'s fan as a percentage of the
    maximum fan speed (100%).
    
    Note: The reported speed is the intended fan speed. If the fan is physically blocked
    and unable to spin, the output will not match the actual fan speed.
    
    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `InvalidArg`, if this `Device` is invalid
    * `NotSupported`, if this `Device` does not have a fan
    * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    * `Unknown`, on any unexpected error
    
    # Device Support

    Supports all discrete products with dedicated fans.
    */
    // Checked against local
    // Tested
    #[inline]
    pub fn fan_speed(&self) -> Result<u32> {
        unsafe {
            let mut speed: c_uint = mem::zeroed();
            nvml_try(nvmlDeviceGetFanSpeed(self.device, &mut speed))?;

            Ok(speed)
        }
    }

    /**
    Gets the current GPU operation mode and the pending one (that it will switch to
    after a reboot).
    
    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `InvalidArg`, if this `Device` is invalid
    * `NotSupported`, if this `Device` does not support this feature
    * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    * `UnexpectedVariant`, for which you can read the docs for
    * `Unknown`, on any unexpected error
    
    # Device Support

    Supports GK110 M-class and X-class Tesla products from the Kepler family. Modes `LowDP`
    and `AllOn` are supported on fully supported GeForce products. Not supported
    on Quadro and Tesla C-class products.
    */
    // Checked against local
    // Tested on machines other than my own
    #[inline]
    pub fn gpu_operation_mode(&self) -> Result<OperationModeState> {
        unsafe {
            let mut current: nvmlGpuOperationMode_t = mem::zeroed();
            let mut pending: nvmlGpuOperationMode_t = mem::zeroed();

            nvml_try(nvmlDeviceGetGpuOperationMode(
                self.device,
                &mut current,
                &mut pending
            ))?;

            Ok(OperationModeState {
                current: OperationMode::try_from(current)?,
                pending: OperationMode::try_from(pending)?
            })
        }
    }

    /**
    Gets information about processes with a graphics context running on this `Device`.
    
    This only returns information about graphics based processes (OpenGL, DirectX, etc.).
    
    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `InvalidArg`, if this `Device` is invalid
    * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    * `Unknown`, on any unexpected error
    */
    // Tested
    #[inline]
    pub fn running_graphics_processes(&self) -> Result<Vec<ProcessInfo>> {
        unsafe {
            let mut count: c_uint = match self.running_graphics_processes_count()? {
                0 => return Ok(vec![]),
                value => value,
            };
            let mut processes: Vec<nvmlProcessInfo_t> = vec![mem::zeroed(); count as usize];

            nvml_try(nvmlDeviceGetGraphicsRunningProcesses(
                self.device,
                &mut count,
                processes.as_mut_ptr()
            ))?;
            processes.truncate(count as usize);

            Ok(processes.into_iter().map(ProcessInfo::from).collect())
        }
    }

    /**
    Gets the number of processes with a graphics context running on this `Device`.
    
    This only returns the count of graphics based processes (OpenGL, DirectX).
    
    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `InvalidArg`, if this `Device` is invalid
    * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    * `UnexpectedVariant`, for which you can read the docs for
    * `Unknown`, on any unexpected error
    */
    // Tested as part of `.running_graphics_processes()`
    #[inline]
    pub fn running_graphics_processes_count(&self) -> Result<u32> {
        unsafe {
            // Indicates that we want the count
            let mut count: c_uint = 0;

            // Passing null doesn't indicate that we want the count. It's just allowed.
            match nvmlDeviceGetGraphicsRunningProcesses(self.device, &mut count, ptr::null_mut()) {
                nvmlReturn_enum_NVML_ERROR_INSUFFICIENT_SIZE => Ok(count),
                // If success, return 0; otherwise, return error
                other => nvml_try(other).map(|_| 0),
            }
        }
    }

    /**
    Gets utilization stats for relevant currently running processes.

    Utilization stats are returned for processes that had a non-zero utilization stat
    at some point during the target sample period. Passing `None` as the
    `last_seen_timestamp` will target all samples that the driver has buffered; passing
    a timestamp retrieved from a previous query will target samples taken since that
    timestamp.

    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `InvalidArg`, if this `Device` is invalid
    * `NotSupported`, if this `Device` does not support this feature
    * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    * `Unknown`, on any unexpected error

    # Device Support

    Supports Maxwell or newer fully supported devices.
    */
    #[inline]
    pub fn process_utilization_stats<T>(&self, last_seen_timestamp: T) -> Result<Vec<ProcessUtilizationSample>>
    where
        T: Into<Option<u64>>
    {
        unsafe {
            let last_seen_timestamp = last_seen_timestamp.into().unwrap_or(0);
            let mut count = match self.process_utilization_stats_count()? {
                0 => return Ok(vec![]),
                v => v
            };
            let mut utilization_samples: Vec<nvmlProcessUtilizationSample_t> 
                = vec![mem::zeroed(); count as usize];

            nvml_try(nvmlDeviceGetProcessUtilization(
                self.device,
                utilization_samples.as_mut_ptr(),
                &mut count,
                last_seen_timestamp
            ))?;
            utilization_samples.truncate(count as usize);

            Ok(utilization_samples.into_iter().map(ProcessUtilizationSample::from).collect())
        }
    }

    #[inline]
    fn process_utilization_stats_count(&self) -> Result<c_uint> {
        unsafe {
            let mut count: c_uint = 0;

            match nvmlDeviceGetProcessUtilization(
                self.device,
                ptr::null_mut(),
                &mut count,
                0
            ) {
                // Despite being undocumented, this appears to be the correct behavior
                nvmlReturn_enum_NVML_ERROR_INSUFFICIENT_SIZE => Ok(count),
                other => nvml_try(other).map(|_| 0)
            }
        }
    }

    /**
    Gets the NVML index of this `Device`. 
    
    Keep in mind that the order in which NVML enumerates devices has no guarantees of
    consistency between reboots. Also, the NVML index may not correlate with other APIs,
    such as the CUDA device index.
    
    # Errors 

    * `Uninitialized`, if the library has not been successfully initialized
    * `InvalidArg`, if this `Device` is invalid
    * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    */
    // Checked against local
    // Tested
    #[inline]
    pub fn index(&self) -> Result<u32> {
        unsafe {
            let mut index: c_uint = mem::zeroed();
            nvml_try(nvmlDeviceGetIndex(self.device, &mut index))?;

            Ok(index)
        }
    }

    /**
    Gets the checksum of the config stored in this `Device`'s infoROM.
    
    Can be used to make sure that two GPUs have the exact same configuration.
    The current checksum takes into account configuration stored in PWR and ECC
    infoROM objects. The checksum can change between driver released or when the
    user changes the configuration (e.g. disabling/enabling ECC).
    
    # Errors

    * `CorruptedInfoROM`, if this `Device`'s checksum couldn't be retrieved due to infoROM corruption
    * `Uninitialized`, if the library has not been successfully initialized
    * `NotSupported`, if this `Device` does not support this feature
    * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    * `Unknown`, on any unexpected error
    
    # Device Support

    Supports all devices with an infoROM.
    */
    // Checked against local
    // Tested on machines other than my own
    #[inline]
    pub fn config_checksum(&self) -> Result<u32> {
        unsafe {
            let mut checksum: c_uint = mem::zeroed();

            nvml_try(nvmlDeviceGetInforomConfigurationChecksum(
                self.device,
                &mut checksum
            ))?;

            Ok(checksum)
        }
    }

    /**
    Gets the global infoROM image version.
    
    This image version, just like the VBIOS version, uniquely describes the exact version
    of the infoROM flashed on the board, in contrast to the infoROM object version which
    is only an indicator of supported features.
    
    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `InvalidArg`, if this `Device` is invalid
    * `NotSupported`, if this `Device` does not have an infoROM
    * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    * `Utf8Error`, if the string obtained from the C function is not valid Utf8
    * `Unknown`, on any unexpected error
    
    # Device Support

    Supports all devices with an infoROM.
    */
    // Checked against local
    // Tested on machines other than my own
    #[inline]
    pub fn info_rom_image_version(&self) -> Result<String> {
        unsafe {
            let mut version_vec =
                Vec::with_capacity(NVML_DEVICE_INFOROM_VERSION_BUFFER_SIZE as usize);

            nvml_try(nvmlDeviceGetInforomImageVersion(
                self.device,
                version_vec.as_mut_ptr(),
                NVML_DEVICE_INFOROM_VERSION_BUFFER_SIZE
            ))?;

            let version_raw = CStr::from_ptr(version_vec.as_ptr());
            Ok(version_raw.to_str()?.into())
        }
    }

    /**
    Gets the version information for this `Device`'s infoROM object, for the passed in 
    object type.
    
    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `InvalidArg`, if this `Device` is invalid
    * `NotSupported`, if this `Device` does not have an infoROM
    * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    * `Utf8Error`, if the string obtained from the C function is not valid UTF-8
    * `Unknown`, on any unexpected error
    
    # Device Support

    Supports all devices with an infoROM.
    
    Fermi and higher parts have non-volatile on-board memory for persisting device info,
    such as aggregate ECC counts. The version of the data structures in this memory may
    change from time to time.
    */
    // Checked against local
    // Tested on machines other than my own
    #[inline]
    pub fn info_rom_version(&self, object: InfoRom) -> Result<String> {
        unsafe {
            let mut version_vec =
                Vec::with_capacity(NVML_DEVICE_INFOROM_VERSION_BUFFER_SIZE as usize);

            nvml_try(nvmlDeviceGetInforomVersion(
                self.device,
                object.as_c(),
                version_vec.as_mut_ptr(),
                NVML_DEVICE_INFOROM_VERSION_BUFFER_SIZE
            ))?;

            let version_raw = CStr::from_ptr(version_vec.as_ptr());
            Ok(version_raw.to_str()?.into())
        }
    }

    /**
    Gets the maximum clock speeds for this `Device`.
    
    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `InvalidArg`, if this `Device` is invalid
    * `NotSupported`, if this `Device` cannot report the specified `Clock`
    * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    * `Unknown`, on any unexpected error
    
    # Device Support

    Supports Fermi and newer fully supported devices.
    
    Note: On GPUs from the Fermi family, current P0 (Performance state 0?) clocks 
    (reported by `.clock_info()`) can differ from max clocks by a few MHz.
    */
    // Checked against local
    // Tested
    #[inline]
    pub fn max_clock_info(&self, clock_type: Clock) -> Result<u32> {
        unsafe {
            let mut clock: c_uint = mem::zeroed();

            nvml_try(nvmlDeviceGetMaxClockInfo(
                self.device,
                clock_type.as_c(),
                &mut clock
            ))?;

            Ok(clock)
        }
    }

    /**
    Gets the max PCIe link generation possible with this `Device` and system.
    
    For a gen 2 PCIe device attached to a gen 1 PCIe bus, the max link generation
    this function will report is generation 1.
    
    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `InvalidArg`, if this `Device` is invalid
    * `NotSupported`, if PCIe link information is not available
    * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    * `Unknown`, on any unexpected error
    
    # Device Support

    Supports Fermi and newer fully supported devices.
    */
    // Checked against local
    // Tested
    #[inline]
    pub fn max_pcie_link_gen(&self) -> Result<u32> {
        unsafe {
            let mut max_gen: c_uint = mem::zeroed();

            nvml_try(nvmlDeviceGetMaxPcieLinkGeneration(
                self.device,
                &mut max_gen
            ))?;

            Ok(max_gen)
        }
    }

    /**
    Gets the maximum PCIe link width possible with this `Device` and system.
    
    For a device with a 16x PCie bus width attached to an 8x PCIe system bus,
    this method will report a max link width of 8.
    
    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `InvalidArg`, if this `Device` is invalid
    * `NotSupported`, if PCIe link information is not available
    * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    * `Unknown`, on any unexpected error
    
    # Device Support

    Supports Fermi and newer fully supported devices.
    */
    // Checked against local
    // Tested
    #[inline]
    pub fn max_pcie_link_width(&self) -> Result<u32> {
        unsafe {
            let mut max_width: c_uint = mem::zeroed();
            nvml_try(nvmlDeviceGetMaxPcieLinkWidth(self.device, &mut max_width))?;

            Ok(max_width)
        }
    }

    /**
    Gets the requested memory error counter for this `Device`.
    
    Only applicable to devices with ECC. Requires ECC mode to be enabled.
    
    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `InvalidArg`, if `error_type`, `counter_type`, or `location` is invalid (shouldn't occur?)
    * `NotSupported`, if this `Device` does not support ECC error reporting for the specified
    memory
    * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    * `Unknown`, on any unexpected error
    
    # Device Support

    Supports Fermi and newer fully supported devices. Requires `InfoRom::ECC` version
    2.0 or higher to report aggregate location-based memory error counts. Requires
    `InfoRom::ECC version 1.0 or higher to report all other memory error counts.
    */
    // Checked against local
    // Tested on machines other than my own
    #[inline]
    pub fn memory_error_counter(
        &self,
        error_type: MemoryError,
        counter_type: EccCounter,
        location: MemoryLocation,
    ) -> Result<u64> {
        unsafe {
            let mut count: c_ulonglong = mem::zeroed();

            nvml_try(nvmlDeviceGetMemoryErrorCounter(
                self.device,
                error_type.as_c(),
                counter_type.as_c(),
                location.as_c(),
                &mut count
            ))?;

            Ok(count)
        }
    }

    /**
    Gets the amount of used, free and total memory available on this `Device`, in bytes.
    
    Note that enabling ECC reduces the amount of total available memory due to the
    extra required parity bits.
    
    Also note that on Windows, most device memory is allocated and managed on startup
    by Windows.
    
    Under Linux and Windows TCC (no physical display connected), the reported amount 
    of used memory is equal to the sum of memory allocated by all active channels on 
    this `Device`.
    
    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `InvalidArg`, if this `Device` is invalid
    * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    * `Unknown`, on any unexpected error
    */
    // Checked against local
    // Tested
    #[inline]
    pub fn memory_info(&self) -> Result<MemoryInfo> {
        unsafe {
            let mut info: nvmlMemory_t = mem::zeroed();
            nvml_try(nvmlDeviceGetMemoryInfo(self.device, &mut info))?;

            Ok(info.into())
        }
    }

    /**
    Gets the minor number for this `Device`.
    
    The minor number is such that the NVIDIA device node file for each GPU will
    have the form `/dev/nvidia[minor number]`.
    
    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `InvalidArg`, if this `Device` is invalid
    * `NotSupported`, if this query is not supported by this `Device`
    * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    * `Unknown`, on any unexpected error
    
    # Platform Support

    Only supports Linux.
    */
    // Checked against local
    // Tested
    #[cfg(target_os = "linux")]
    #[inline]
    pub fn minor_number(&self) -> Result<u32> {
        unsafe {
            let mut number: c_uint = mem::zeroed();
            nvml_try(nvmlDeviceGetMinorNumber(self.device, &mut number))?;

            Ok(number)
        }
    }

    /**
    Identifies whether or not this `Device` is on a multi-GPU board.
    
    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `InvalidArg`, if this `Device` is invalid
    * `NotSupported`, if this `Device` does not support this feature
    * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    * `Unknown`, on any unexpected error
    
    # Device Support

    Supports Fermi or newer fully supported devices.
    */
    // Checked against local
    // Tested
    #[inline]
    pub fn is_multi_gpu_board(&self) -> Result<bool> {
        unsafe {
            let mut int_bool: c_uint = mem::zeroed();
            nvml_try(nvmlDeviceGetMultiGpuBoard(self.device, &mut int_bool))?;

            match int_bool {
                0 => Ok(false),
                _ => Ok(true),
            }
        }
    }

    /**
    The name of this `Device`, e.g. "Tesla C2070".
    
    The name is an alphanumeric string that denotes a particular product. 
    
    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `InvalidArg`, if this `Device` is invalid
    * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    * `Utf8Error`, if the string obtained from the C function is not valid Utf8
    * `Unknown`, on any unexpected error
    */
    // Checked against local
    // Tested
    #[inline]
    pub fn name(&self) -> Result<String> {
        unsafe {
            let mut name_vec = Vec::with_capacity(NVML_DEVICE_NAME_BUFFER_SIZE as usize);

            nvml_try(nvmlDeviceGetName(
                self.device,
                name_vec.as_mut_ptr(),
                NVML_DEVICE_NAME_BUFFER_SIZE
            ))?;

            let name_raw = CStr::from_ptr(name_vec.as_ptr());
            Ok(name_raw.to_str()?.into())
        }
    }

    /**
    Gets the PCI attributes of this `Device`.
    
    See `PciInfo` for details about the returned attributes.
    
    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `InvalidArg`, if this `Device` is invalid
    * `GpuLost`, if the GPU has fallen off the bus or is otherwise inaccessible
    * `Utf8Error`, if a string obtained from the C function is not valid Utf8
    * `Unknown`, on any unexpected error
    */
    // Checked against local
    // Tested
    #[inline]
    pub fn pci_info(&self) -> Result<PciInfo> {
        unsafe {
            let mut pci_info: nvmlPciInfo_t = mem::zeroed();
            nvml_try(nvmlDeviceGetPciInfo_v3(self.device, &mut pci_info))?;

            Ok(PciInfo::try_from(pci_info, true, false)?)
        }
    }

    /**
    Gets the PCIe replay counter.
    
    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `InvalidArg`, if this `Device` is invalid
    * `NotSupported`, if this `Device` does not support this feature
    * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    * `Unknown`, on any unexpected error
    
    # Device Support

    Supports Kepler or newer fully supported devices.
    */
    // Checked against local
    // Tested
    #[inline]
    pub fn pcie_replay_counter(&self) -> Result<u32> {
        unsafe {
            let mut value: c_uint = mem::zeroed();
            nvml_try(nvmlDeviceGetPcieReplayCounter(self.device, &mut value))?;

            Ok(value)
        }
    }

    /**
    Gets PCIe utilization information in KB/s.
    
    The function called within this method is querying a byte counter over a 20ms
    interval and thus is the PCIE throughput over that interval.
    
    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `InvalidArg`, if this `Device` is invalid or `counter` is invalid (shouldn't occur?)
    * `NotSupported`, if this `Device` does not support this feature
    * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    * `Unknown`, on any unexpected error
    
    # Device Support

    Supports Maxwell and newer fully supported devices.
    
    # Environment Support

    This method is not supported on virtual machines running vGPUs.
    */
    // Checked against local
    // Tested
    #[inline]
    pub fn pcie_throughput(&self, counter: PcieUtilCounter) -> Result<u32> {
        unsafe {
            let mut throughput: c_uint = mem::zeroed();

            nvml_try(nvmlDeviceGetPcieThroughput(
                self.device,
                counter.as_c(),
                &mut throughput
            ))?;

            Ok(throughput)
        }
    }

    /**
    Gets the current performance state for this `Device`. 0 == max, 15 == min.
    
    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `InvalidArg`, if this `Device` is invalid
    * `NotSupported`, if this `Device` does not support this feature
    * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    * `UnexpectedVariant`, for which you can read the docs for
    * `Unknown`, on any unexpected error
    
    # Device Support

    Supports Fermi or newer fully supported devices.
    */
    // Checked against local
    // Tested
    #[inline]
    pub fn performance_state(&self) -> Result<PerformanceState> {
        unsafe {
            let mut state: nvmlPstates_t = mem::zeroed();
            nvml_try(nvmlDeviceGetPerformanceState(self.device, &mut state))?;

            Ok(PerformanceState::try_from(state)?)
        }
    }

    /**
    Gets whether or not persistent mode is enabled for this `Device`.
    
    When driver persistence mode is enabled the driver software is not torn down
    when the last client disconnects. This feature is disabled by default.
    
    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `InvalidArg`, if this `Device` is invalid
    * `NotSupported`, if this `Device` does not support this feature
    * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    * `UnexpectedVariant`, for which you can read the docs for
    * `Unknown`, on any unexpected error
    
    # Platform Support
    
    Only supports Linux.
    */
    // Checked against local
    // Tested
    #[cfg(target_os = "linux")]
    #[inline]
    pub fn is_in_persistent_mode(&self) -> Result<bool> {
        unsafe {
            let mut state: nvmlEnableState_t = mem::zeroed();
            nvml_try(nvmlDeviceGetPersistenceMode(self.device, &mut state))?;

            Ok(bool_from_state(state)?)
        }
    }

    /**
    Gets the default power management limit for this `Device`, in milliwatts.
    
    This is the limit that this `Device` boots with.
    
    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `InvalidArg`, if this `Device` is invalid
    * `NotSupported`, if this `Device` does not support this feature
    * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    * `Unknown`, on any unexpected error
    
    # Device Support

    Supports Kepler or newer fully supported devices.
    */
    // Checked against local
    // Tested
    #[inline]
    pub fn power_management_limit_default(&self) -> Result<u32> {
        unsafe {
            let mut limit: c_uint = mem::zeroed();
            nvml_try(nvmlDeviceGetPowerManagementDefaultLimit(
                self.device,
                &mut limit
            ))?;

            Ok(limit)
        }
    }

    /**
    Gets the power management limit associated with this `Device`.
    
    The power limit defines the upper boundary for the card's power draw. If the card's
    total power draw reaches this limit, the power management algorithm kicks in.
    
    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `InvalidArg`, if this `Device` is invalid
    * `NotSupported`, if this `Device` does not support this feature
    * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    * `Unknown`, on any unexpected error
    
    # Device Support

    Supports Fermi or newer fully supported devices.
    
    This reading is only supported if power management mode is supported. See
    `.is_power_management_algo_active()`. Yes, it's deprecated, but that's what
    NVIDIA's docs said to see.
    */
    // Checked against local
    // Tested
    #[inline]
    pub fn power_management_limit(&self) -> Result<u32> {
        unsafe {
            let mut limit: c_uint = mem::zeroed();
            nvml_try(nvmlDeviceGetPowerManagementLimit(self.device, &mut limit))?;

            Ok(limit)
        }
    }

    /**
    Gets information about possible power management limit values for this `Device`, in milliwatts.
    
    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `InvalidArg`, if this `Device` is invalid
    * `NotSupported`, if this `Device` does not support this feature
    * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    * `Unknown`, on any unexpected error
    
    # Device Support

    Supports Kepler or newer fully supported devices.
    */
    // Checked against local
    // Tested
    #[inline]
    pub fn power_management_limit_constraints(&self) -> Result<PowerManagementConstraints> {
        unsafe {
            let mut min_limit: c_uint = mem::zeroed();
            let mut max_limit: c_uint = mem::zeroed();

            nvml_try(nvmlDeviceGetPowerManagementLimitConstraints(
                self.device,
                &mut min_limit,
                &mut max_limit
            ))?;

            Ok(PowerManagementConstraints {
                min_limit,
                max_limit
            })
        }
    }

    /// Not documenting this because it's deprecated. Read NVIDIA's docs if you
    /// must use it.
    // Tested
    #[deprecated(note = "NVIDIA states that \"this API has been deprecated.\"")]
    #[inline]
    pub fn is_power_management_algo_active(&self) -> Result<bool> {
        unsafe {
            let mut state: nvmlEnableState_t = mem::zeroed();
            nvml_try(nvmlDeviceGetPowerManagementMode(self.device, &mut state))?;

            Ok(bool_from_state(state)?)
        }
    }

    /// Not documenting this because it's deprecated. Read NVIDIA's docs if you
    /// must use it.
    // Tested
    #[deprecated(note = "use `.performance_state()`.")]
    #[inline]
    pub fn power_state(&self) -> Result<PerformanceState> {
        unsafe {
            let mut state: nvmlPstates_t = mem::zeroed();
            nvml_try(nvmlDeviceGetPowerState(self.device, &mut state))?;

            Ok(PerformanceState::try_from(state)?)
        }
    }

    /**
    Gets the power usage for this GPU and its associated circuitry (memory) in milliwatts.
    
    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `InvalidArg`, if this `Device` is invalid
    * `NotSupported`, if this `Device` does not support power readings
    * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    * `Unknown`, on any unexpected error
    
    # Device Support

    Supports Fermi and newer fully supported devices.
    
    This reading is accurate to within +/- 5% of current power draw on Fermi and Kepler GPUs.
    It is only supported if power management mode is supported. See `.is_power_management_algo_active()`.
    Yes, that is deperecated, but that's what NVIDIA's docs say to see.
    */
    // Checked against local
    // Tested
    #[inline]
    pub fn power_usage(&self) -> Result<u32> {
        unsafe {
            let mut usage: c_uint = mem::zeroed();
            nvml_try(nvmlDeviceGetPowerUsage(self.device, &mut usage))?;

            Ok(usage)
        }
    }

    /**
    Gets this device's total energy consumption in millijoules (mJ) since the last
    driver reload.

    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `InvalidArg`, if this `Device` is invalid
    * `NotSupported`, if this `Device` does not support energy readings
    * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    * `Unknown`, on any unexpected error

    # Device Support

    Supports Pascal and newer fully supported devices.
    */
    #[inline]
    pub fn total_energy_consumption(&self) -> Result<u64> {
        unsafe {
            let mut total: c_ulonglong = mem::zeroed();
            nvml_try(nvmlDeviceGetTotalEnergyConsumption(self.device, &mut total))?;

            Ok(total)
        }
    }

    /**
    Gets the list of retired pages filtered by `cause`, including pages pending retirement.

    **I cannot verify that this method will work because the call within is not supported
    on my dev machine**. Please **verify for yourself** that it works before you use it.
    If you are able to test it on your machine, please let me know if it works; if it
    doesn't, I would love a PR.
    
    The address information provided by this API is the hardware address of the page that was
    retired. Note that this does not match the virtual address used in CUDA, but it will
    match the address information in XID 63.
    
    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `InvalidArg`, if this `Device` is invalid
    * `NotSupported`, if this `Device` doesn't support this feature
    * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    * `Unknown`, on any unexpected error
    
    # Device Support

    Supports Kepler and newer fully supported devices.
    */
    // Checked against local
    // Tested on machines other than my own
    #[inline]
    pub fn retired_pages(&self, cause: RetirementCause) -> Result<Vec<u64>> {
        unsafe {
            let mut count = match self.retired_pages_count(&cause)? {
                0 => return Ok(vec![]),
                value => value,
            };
            let mut causes: Vec<c_ulonglong> = vec![mem::zeroed(); count as usize];

            nvml_try(nvmlDeviceGetRetiredPages(
                self.device,
                cause.as_c(),
                &mut count,
                causes.as_mut_ptr()
            ))?;

            Ok(causes)
        }
    }

    // Helper for the above function. Returns # of samples that can be queried.
    #[inline]
    fn retired_pages_count(&self, cause: &RetirementCause) -> Result<c_uint> {
        unsafe {
            let mut count: c_uint = 0;

            nvml_try(nvmlDeviceGetRetiredPages(
                self.device,
                cause.as_c(),
                &mut count,
                // All NVIDIA says is that this
                // can't be null.
                &mut mem::zeroed()
            ))?;

            Ok(count)
        }
    }

    /**
    Gets whether there are pages pending retirement (they need a reboot to fully retire).
    
    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `InvalidArg`, if this `Device` is invalid
    * `NotSupported`, if this `Device` doesn't support this feature
    * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    * `UnexpectedVariant`, for which you can read the docs for
    * `Unknown`, on any unexpected error
    
    # Device Support

    Supports Kepler and newer fully supported devices.
    */
    // Checked against local
    // Tested on machines other than my own
    #[inline]
    pub fn are_pages_pending_retired(&self) -> Result<bool> {
        unsafe {
            let mut state: nvmlEnableState_t = mem::zeroed();

            nvml_try(nvmlDeviceGetRetiredPagesPendingStatus(
                self.device,
                &mut state
            ))?;

            Ok(bool_from_state(state)?)
        }
    }

    /**
    Gets recent samples for this `Device`.
    
    `last_seen_timestamp` represents the CPU timestamp in μs. Passing in `None`
    will fetch all samples maintained in the underlying buffer; you can
    alternatively pass in a timestamp retrieved from the date of the previous
    query in order to obtain more recent samples.
    
    The advantage of using this method for samples in contrast to polling via
    existing methods is to get higher frequency data at a lower polling cost.
    
    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `InvalidArg`, if this `Device` is invalid
    * `NotSupported`, if this query is not supported by this `Device`
    * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    * `NotFound`, if sample entries are not found
    * `UnexpectedVariant`, check that error's docs for more info
    * `Unknown`, on any unexpected error
    
    # Device Support

    Supports Kepler and newer fully supported devices.

    # Examples

    ```
    # use nvml_wrapper::NVML;
    # use nvml_wrapper::error::*;
    # fn main() -> Result<()> {
    # match test() {
    # Err(Error(ErrorKind::NotFound, _)) => Ok(()),
    # other => other,
    # }
    # }
    # fn test() -> Result<()> {
    # let nvml = NVML::init()?;
    # let device = nvml.device_by_index(0)?;
    use nvml_wrapper::enum_wrappers::device::Sampling;

    // Passing `None` indicates that we want all `Power` samples in the sample buffer
    let power_samples = device.samples(Sampling::Power, None)?;

    // Take the first sample from the vector, if it exists...
    if let Some(sample) = power_samples.get(0) {
        // ...and now we can get all `ProcessorClock` samples that exist with a later
        // timestamp than the `Power` sample.
        let newer_clock_samples = device.samples(Sampling::ProcessorClock, sample.timestamp)?;
    }
    # Ok(())
    # }
    ```
    */
    // Checked against local
    // Tested
    #[inline]
    pub fn samples<T>(&self, sample_type: Sampling, last_seen_timestamp: T) -> Result<Vec<Sample>>
    where
        T: Into<Option<u64>>,
    {
        let timestamp = last_seen_timestamp.into().unwrap_or(0);
        unsafe {
            let mut val_type: nvmlValueType_t = mem::zeroed();
            let mut count = match self.samples_count(&sample_type, timestamp)? {
                0 => return Ok(vec![]),
                value => value,
            };
            let mut samples: Vec<nvmlSample_t> = vec![mem::zeroed(); count as usize];

            nvml_try(nvmlDeviceGetSamples(
                self.device,
                sample_type.as_c(),
                timestamp,
                &mut val_type,
                &mut count,
                samples.as_mut_ptr()
            ))?;

            let val_type_rust = SampleValueType::try_from(val_type)?;
            Ok(
                samples
                    .into_iter()
                    .map(|s| Sample::from_tag_and_struct(&val_type_rust, s))
                    .collect()
            )
        }
    }

    // Helper for the above function. Returns # of samples that can be queried.
    #[inline]
    fn samples_count(&self, sample_type: &Sampling, timestamp: u64) -> Result<c_uint> {
        unsafe {
            let mut val_type: nvmlValueType_t = mem::zeroed();
            let mut count: c_uint = mem::zeroed();

            nvml_try(nvmlDeviceGetSamples(
                self.device,
                sample_type.as_c(),
                timestamp,
                &mut val_type,
                &mut count,
                // Indicates that we want the count
                ptr::null_mut()
            ))?;

            Ok(count)
        }
    }

    /**
    Get values for the given slice of `FieldId`s.

    NVIDIA's docs say that if any of the `FieldId`s are populated by the same driver
    call, the samples for those IDs will be populated by a single call instead of
    a call per ID. It would appear, then, that this is essentially a "batch-request"
    API path for better performance.

    There are too many field ID constants defined in the header to reasonably
    wrap them with an enum in this crate. Instead, I've re-exported the defined
    ID constants at `nvml_wrapper::sys_exports::field_id::*`; stick those
    constants in `FieldId`s for use with this function.

    # Errors

    ## Outer `Result`

    * `InvalidArg`, if `id_slice` has a length of zero

    ## Inner `Result`

    * `UnexpectedVariant`, check that error's docs for more info

    # Device Support

    Device support varies per `FieldId` that you pass in.
    */
    // TODO: Example
    #[inline]
    pub fn field_values_for(&self, id_slice: &[FieldId]) -> Result<Vec<Result<FieldValueSample>>> {
        unsafe {
            let values_count = id_slice.len();
            let mut field_values: Vec<nvmlFieldValue_t> = Vec::with_capacity(values_count);

            for id in id_slice.into_iter() {
                let mut raw: nvmlFieldValue_t = mem::zeroed();
                raw.fieldId = id.0;

                field_values.push(raw);
            }

            nvml_try(nvmlDeviceGetFieldValues(
                self.device,
                values_count as i32,
                field_values.as_mut_ptr()
            ))?;

            Ok(field_values.into_iter().map(|v| FieldValueSample::try_from(v)).collect())
        }
    }

    /**
    Gets the globally unique board serial number associated with this `Device`'s board
    as an alphanumeric string.
    
    This serial number matches the serial number tag that is physically attached to the board.
    
    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `InvalidArg`, if this `Device` is invalid
    * `NotSupported`, if this `Device` doesn't support this feature
    * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    * `Utf8Error`, if the string obtained from the C function is not valid Utf8
    * `Unknown`, on any unexpected error
    
    # Device Support

    Supports all products with an infoROM.
    */
    // Checked against local
    // Tested on machines other than my own
    #[inline]
    pub fn serial(&self) -> Result<String> {
        unsafe {
            let mut serial_vec = Vec::with_capacity(NVML_DEVICE_SERIAL_BUFFER_SIZE as usize);

            nvml_try(nvmlDeviceGetSerial(
                self.device,
                serial_vec.as_mut_ptr(),
                NVML_DEVICE_SERIAL_BUFFER_SIZE
            ))?;

            let serial_raw = CStr::from_ptr(serial_vec.as_ptr());
            Ok(serial_raw.to_str()?.into())
        }
    }

    /**
    Gets the board part number for this `Device`.
    
    The board part number is programmed into the board's infoROM.
    
    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `NotSupported`, if the necessary VBIOS fields have not been filled
    * `GpuLost`, if the target GPU has fellen off the bus or is otherwise inaccessible
    * `Utf8Error`, if the string obtained from the C function is not valid Utf8
    * `Unknown`, on any unexpected error
    */
    // Checked against local
    // Tested on machines other than my own
    #[inline]
    pub fn board_part_number(&self) -> Result<String> {
        unsafe {
            let mut part_num_vec = Vec::with_capacity(NVML_DEVICE_PART_NUMBER_BUFFER_SIZE as usize);

            nvml_try(nvmlDeviceGetBoardPartNumber(
                self.device,
                part_num_vec.as_mut_ptr(),
                NVML_DEVICE_PART_NUMBER_BUFFER_SIZE
            ))?;

            let part_num_raw = CStr::from_ptr(part_num_vec.as_ptr());
            Ok(part_num_raw.to_str()?.into())
        }
    }

    /**
    Gets current throttling reasons.
    
    Note that multiple reasons can be affecting clocks at once.

    The returned bitmask is created via the `ThrottleReasons::from_bits_truncate`
    method, meaning that any bits that don't correspond to flags present in this
    version of the wrapper will be dropped.
    
    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `NotSupported`, if this `Device` does not support this feature
    * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    * `Unknown`, on any unexpected error
    
    # Device Support

    Supports all _fully supported_ devices.
    */
    // Checked against local.
    // Tested
    #[inline]
    pub fn current_throttle_reasons(&self) -> Result<ThrottleReasons> {
        Ok(ThrottleReasons::from_bits_truncate(self.current_throttle_reasons_raw()?))
    }

    /**
    Gets current throttling reasons, erroring if any bits correspond to
    non-present flags.
    
    Note that multiple reasons can be affecting clocks at once.
    
    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `IncorrectBits`, if NVML returns any bits that do not correspond to flags in
    `ThrottleReasons`
    * `NotSupported`, if this `Device` does not support this feature
    * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    * `Unknown`, on any unexpected error
    
    # Device Support

    Supports all _fully supported_ devices.
    */
    // Checked against local.
    // Tested
    #[inline]
    pub fn current_throttle_reasons_strict(&self) -> Result<ThrottleReasons> {
        let reasons = self.current_throttle_reasons_raw()?;

        ThrottleReasons::from_bits(reasons)
            .ok_or_else(|| ErrorKind::IncorrectBits(Bits::U64(reasons)).into())
    }

    // Helper for the above methods.
    #[inline]
    fn current_throttle_reasons_raw(&self) -> Result<c_ulonglong> {
        unsafe {
            let mut reasons: c_ulonglong = mem::zeroed();

            nvml_try(nvmlDeviceGetCurrentClocksThrottleReasons(
                self.device,
                &mut reasons
            ))?;

            Ok(reasons)
        }
    }

    /**
    Gets a bitmask of the supported throttle reasons.
    
    These reasons can be returned by `.current_throttle_reasons()`.

    The returned bitmask is created via the `ThrottleReasons::from_bits_truncate`
    method, meaning that any bits that don't correspond to flags present in this
    version of the wrapper will be dropped.
    
    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    * `Unknown`, on any unexpected error
    
    # Device Support

    Supports all _fully supported_ devices.
    
    # Environment Support

    This method is not supported on virtual machines running vGPUs.
    */
    // Checked against local
    // Tested
    #[inline]
    pub fn supported_throttle_reasons(&self) -> Result<ThrottleReasons> {
        Ok(ThrottleReasons::from_bits_truncate(self.supported_throttle_reasons_raw()?))
    }

    /**
    Gets a bitmask of the supported throttle reasons, erroring if any bits
    correspond to non-present flags.
    
    These reasons can be returned by `.current_throttle_reasons()`.
    
    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `IncorrectBits`, if NVML returns any bits that do not correspond to flags in
    `ThrottleReasons`
    * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    * `Unknown`, on any unexpected error
    
    # Device Support

    Supports all _fully supported_ devices.
    
    # Environment Support

    This method is not supported on virtual machines running vGPUs.
    */
    // Checked against local
    // Tested
    #[inline]
    pub fn supported_throttle_reasons_strict(&self) -> Result<ThrottleReasons> {
        let reasons = self.supported_throttle_reasons_raw()?;

        ThrottleReasons::from_bits(reasons)
            .ok_or_else(|| ErrorKind::IncorrectBits(Bits::U64(reasons)).into())
    }

    // Helper for the above methods.
    #[inline]
    fn supported_throttle_reasons_raw(&self) -> Result<c_ulonglong> {
        unsafe {
            let mut reasons: c_ulonglong = mem::zeroed();

            nvml_try(nvmlDeviceGetSupportedClocksThrottleReasons(
                self.device,
                &mut reasons
            ))?;

            Ok(reasons)
        }
    }

    /**
    Gets a `Vec` of possible graphics clocks that can be used as an arg for
    `set_applications_clocks()`.
    
    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `NotFound`, if the specified `for_mem_clock` is not a supported frequency
    * `InvalidArg`, if this `Device` is invalid
    * `NotSupported`, if this `Device` doesn't support this feature
    * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    * `Unknown`, on any unexpected error
    
    # Device Support

    Supports Kepler and newer fully supported devices.
    */
    // Checked against local
    // Tested
    #[inline]
    pub fn supported_graphics_clocks(&self, for_mem_clock: u32) -> Result<Vec<u32>> {
        match self.supported_graphics_clocks_manual(for_mem_clock, 128) {
            Err(Error(ErrorKind::InsufficientSize(Some(s)), _)) =>
                // `s` is the required size for the call; make the call a second time
                self.supported_graphics_clocks_manual(for_mem_clock, s),
            value => value,
        }
    }

    // Removes code duplication in the above function.
    #[inline]
    fn supported_graphics_clocks_manual(
        &self,
        for_mem_clock: u32,
        size: usize,
    ) -> Result<Vec<u32>> {

        let mut items: Vec<c_uint> = vec![0; size];
        let mut count = size as c_uint;

        unsafe {
            match nvmlDeviceGetSupportedGraphicsClocks(
                self.device,
                for_mem_clock,
                &mut count,
                items.as_mut_ptr()
            ) {
                nvmlReturn_enum_NVML_ERROR_INSUFFICIENT_SIZE =>
                    // `count` is now the size that is required. Return it in the error.
                    bail!(ErrorKind::InsufficientSize(Some(count as usize))),
                value => nvml_try(value)?,
            }
        }

        items.truncate(count as usize);
        Ok(items)
    }

    /**
    Gets a `Vec` of possible memory clocks that can be used as an arg for
    `set_applications_clocks()`.
    
    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `InvalidArg`, if this `Device` is invalid
    * `NotSupported`, if this `Device` doesn't support this feature
    * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    * `Unknown`, on any unexpected error
    
    # Device Support
    
    Supports Kepler and newer fully supported devices.
    */
    // Checked against local
    // Tested
    #[inline]
    pub fn supported_memory_clocks(&self) -> Result<Vec<u32>> {
        match self.supported_memory_clocks_manual(16) {
            Err(Error(ErrorKind::InsufficientSize(Some(s)), _)) => {
                // `s` is the required size for the call; make the call a second time
                self.supported_memory_clocks_manual(s)
            },
            value => value,
        }
    }

    // Removes code duplication in the above function.
    fn supported_memory_clocks_manual(&self, size: usize) -> Result<Vec<u32>> {
        let mut items: Vec<c_uint> = vec![0; size];
        let mut count = size as c_uint;

        unsafe {
            match nvmlDeviceGetSupportedMemoryClocks(
                self.device,
                &mut count,
                items.as_mut_ptr()
            ) {
                nvmlReturn_enum_NVML_ERROR_INSUFFICIENT_SIZE => 
                    // `count` is now the size that is required. Return it in the error.
                    bail!(ErrorKind::InsufficientSize(Some(count as usize))),
                value => nvml_try(value)?,
            }
        }

        items.truncate(count as usize);
        Ok(items)
    }

    /**
    Gets the current temperature readings for the given sensor, in °C.
    
    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `InvalidArg`, if this `Device` is invalid or `sensor` is invalid (shouldn't occur?)
    * `NotSupported`, if this `Device` does not have the specified sensor
    * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    * `Unknown`, on any unexpected error
    */
    // Checked against local
    // Tested
    #[inline]
    pub fn temperature(&self, sensor: TemperatureSensor) -> Result<u32> {
        unsafe {
            let mut temp: c_uint = mem::zeroed();

            nvml_try(nvmlDeviceGetTemperature(
                self.device,
                sensor.as_c(),
                &mut temp
            ))?;

            Ok(temp)
        }
    }

    /**
    Gets the temperature threshold for this `Device` and the specified `threshold_type`, in °C.
    
    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `InvalidArg`, if this `Device` is invalid or `threshold_type` is invalid (shouldn't occur?)
    * `NotSupported`, if this `Device` does not have a temperature sensor or is unsupported
    * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    * `Unknown`, on any unexpected error
    
    # Device Support

    Supports Kepler and newer fully supported devices.
    */
    // Checked against local
    // Tested
    #[inline]
    pub fn temperature_threshold(&self, threshold_type: TemperatureThreshold) -> Result<u32> {
        unsafe {
            let mut temp: c_uint = mem::zeroed();

            nvml_try(nvmlDeviceGetTemperatureThreshold(
                self.device,
                threshold_type.as_c(),
                &mut temp
            ))?;

            Ok(temp)
        }
    }

    /**
    Gets the common ancestor for two devices.
    
    # Errors

    * `InvalidArg`, if either `Device` is invalid
    * `NotSupported`, if this `Device` or the OS does not support this feature
    * `UnexpectedVariant`, for which you can read the docs for
    * `Unknown`, an error has occurred in the underlying topology discovery
    
    # Platform Support

    Only supports Linux.
    */
    // Checked against local
    // Tested
    #[cfg(target_os = "linux")]
    #[inline]
    pub fn topology_common_ancestor(&self, other_device: Device) -> Result<TopologyLevel> {
        unsafe {
            let mut level: nvmlGpuTopologyLevel_t = mem::zeroed();

            nvml_try(nvmlDeviceGetTopologyCommonAncestor(
                self.device,
                other_device.device,
                &mut level
            ))?;

            Ok(TopologyLevel::try_from(level)?)
        }
    }

    /**
    Gets the set of GPUs that are nearest to this `Device` at a specific interconnectivity level.
    
    # Errors

    * `InvalidArg`, if this `Device` is invalid or `level` is invalid (shouldn't occur?)
    * `NotSupported`, if this `Device` or the OS does not support this feature
    * `Unknown`, an error has occurred in the underlying topology discovery
    
    # Platform Support

    Only supports Linux.
    */
    // Checked against local
    // Tested
    #[cfg(target_os = "linux")]
    #[inline]
    pub fn topology_nearest_gpus(&self, level: TopologyLevel) -> Result<Vec<Device<'nvml>>> {
        unsafe {
            let mut count = match self.top_nearest_gpus_count(&level)? {
                0 => return Ok(vec![]),
                value => value,
            };
            let mut gpus: Vec<nvmlDevice_t> = vec![mem::zeroed(); count as usize];

            nvml_try(nvmlDeviceGetTopologyNearestGpus(
                self.device,
                level.as_c(),
                &mut count,
                gpus.as_mut_ptr()
            ))?;

            Ok(gpus.into_iter().map(Device::from).collect())
        }
    }

    // Helper for the above function. Returns # of GPUs in the set.
    #[cfg(target_os = "linux")]
    #[inline]
    fn top_nearest_gpus_count(&self, level: &TopologyLevel) -> Result<c_uint> {
        unsafe {
            let mut count: c_uint = 0;

            nvml_try(nvmlDeviceGetTopologyNearestGpus(
                self.device,
                level.as_c(),
                &mut count,
                // Passing null (I assume?)
                // indicates that we want the
                // GPU count
                ptr::null_mut()
            ))?;

            Ok(count)
        }
    }

    /**
    Gets the total ECC error counts for this `Device`.
    
    Only applicable to devices with ECC. The total error count is the sum of errors across
    each of the separate memory systems, i.e. the total set of errors across the entire device.
    
    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `InvalidArg`, if this `Device` is invalid or either enum is invalid (shouldn't occur?)
    * `NotSupported`, if this `Device` does not support this feature
    * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    * `Unknown`, on any unexpected error
    
    # Device Support

    Supports Fermi and newer fully supported devices. Requires `InfoRom::ECC` version 1.0
    or higher. Requires ECC mode to be enabled.
    */
    // Checked against local
    // Tested on machines other than my own
    #[inline]
    pub fn total_ecc_errors(
        &self,
        error_type: MemoryError,
        counter_type: EccCounter,
    ) -> Result<u64> {
        unsafe {
            let mut count: c_ulonglong = mem::zeroed();

            nvml_try(nvmlDeviceGetTotalEccErrors(
                self.device,
                error_type.as_c(),
                counter_type.as_c(),
                &mut count
            ))?;

            Ok(count)
        }
    }

    /**
    Gets the globally unique immutable UUID associated with this `Device` as a 5 part
    hexadecimal string.
    
    This UUID augments the immutable, board serial identifier. It is a globally unique
    identifier and is the _only_ available identifier for pre-Fermi-architecture products.
    It does NOT correspond to any identifier printed on the board.
    
    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `InvalidArg`, if this `Device` is invalid
    * `NotSupported`, if this `Device` does not support this feature
    * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    * `Utf8Error`, if the string obtained from the C function is not valid Utf8
    * `Unknown`, on any unexpected error

    # Examples

    The UUID can be used to compare two `Device`s and find out if they represent
    the same physical device:

    ```no_run
    # use nvml_wrapper::NVML;
    # use nvml_wrapper::error::*;
    # fn main() -> Result<()> {
    # let nvml = NVML::init()?;
    # let device1 = nvml.device_by_index(0)?;
    # let device2 = nvml.device_by_index(1)?;
    if device1.uuid()? == device2.uuid()? {
        println!("`device1` represents the same physical device that `device2` does.");
    }
    # Ok(())
    # }
    ```
    */
    // Checked against local
    // Tested
    #[inline]
    pub fn uuid(&self) -> Result<String> {
        unsafe {
            let mut uuid_vec = Vec::with_capacity(NVML_DEVICE_UUID_BUFFER_SIZE as usize);

            nvml_try(nvmlDeviceGetUUID(
                self.device,
                uuid_vec.as_mut_ptr(),
                NVML_DEVICE_UUID_BUFFER_SIZE
            ))?;

            let uuid_raw = CStr::from_ptr(uuid_vec.as_ptr());
            Ok(uuid_raw.to_str()?.into())
        }
    }

    /**
    Gets the current utilization rates for this `Device`'s major subsystems.
    
    Note: During driver initialization when ECC is enabled, one can see high GPU
    and memory utilization readings. This is caused by the ECC memory scrubbing
    mechanism that is performed during driver initialization.
    
    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `InvalidArg`, if this `Device` is invalid
    * `NotSupported`, if this `Device` does not support this feature
    * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    * `Unknown`, on any unexpected error
    
    # Device Support

    Supports Fermi and newer fully supported devices.
    */
    // Checked against local
    // Tested
    #[inline]
    pub fn utilization_rates(&self) -> Result<Utilization> {
        unsafe {
            let mut utilization: nvmlUtilization_t = mem::zeroed();
            nvml_try(nvmlDeviceGetUtilizationRates(self.device, &mut utilization))?;

            Ok(utilization.into())
        }
    }

    /**
    Gets the VBIOS version of this `Device`.
    
    The VBIOS version may change from time to time.
    
    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `InvalidArg`, if this `Device` is invalid
    * `NotSupported`, if this `Device` does not support this feature
    * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    * `Utf8Error`, if the string obtained from the C function is not valid UTF-8
    * `Unknown`, on any unexpected error
    */
    // Checked against local
    // Tested
    #[inline]
    pub fn vbios_version(&self) -> Result<String> {
        unsafe {
            let mut version_vec =
                Vec::with_capacity(NVML_DEVICE_VBIOS_VERSION_BUFFER_SIZE as usize);

            nvml_try(nvmlDeviceGetVbiosVersion(
                self.device,
                version_vec.as_mut_ptr(),
                NVML_DEVICE_VBIOS_VERSION_BUFFER_SIZE
            ))?;

            let version_raw = CStr::from_ptr(version_vec.as_ptr());
            Ok(version_raw.to_str()?.into())
        }
    }

    /**
    Gets the duration of time during which this `Device` was throttled (lower than the
    requested clocks) due to power or thermal constraints.
    
    This is important to users who are trying to understand if their GPUs throttle at any
    point while running applications. The difference in violation times at two different
    reference times gives the indication of a GPU throttling event.
    
    Violation for thermal capping is not supported at this time.
    
    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `InvalidArg`, if this `Device` is invalid or `perf_policy` is invalid (shouldn't occur?)
    * `NotSupported`, if this query is not supported by this `Device`
    * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    
    # Device Support

    Supports Kepler or newer fully supported devices.
    */
    // Checked against local
    // Tested
    #[inline]
    pub fn violation_status(&self, perf_policy: PerformancePolicy) -> Result<ViolationTime> {
        unsafe {
            let mut viol_time: nvmlViolationTime_t = mem::zeroed();

            nvml_try(nvmlDeviceGetViolationStatus(
                self.device,
                perf_policy.as_c(),
                &mut viol_time
            ))?;

            Ok(viol_time.into())
        }
    }

    /**
    Checks if this `Device` and the passed-in device are on the same physical board.
    
    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `InvalidArg`, if either `Device` is invalid
    * `NotSupported`, if this check is not supported by this `Device`
    * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    * `Unknown`, on any unexpected error
    */
    // Checked against local
    // Tested
    #[inline]
    pub fn is_on_same_board_as(&self, other_device: &Device) -> Result<bool> {
        unsafe {
            let mut bool_int: c_int = mem::zeroed();

            nvml_try(nvmlDeviceOnSameBoard(
                self.device,
                other_device.unsafe_raw(),
                &mut bool_int
            ))?;

            Ok(match bool_int {
                0 => false,
                _ => true,
            })
        }
    }

    /**
    Resets the application clock to the default value.
    
    This is the applications clock that will be used after a system reboot or a driver
    reload. The default value is a constant, but the current value be changed with
    `.set_applications_clocks()`.
    
    On Pascal and newer hardware, if clocks were previously locked with 
    `.set_applications_clocks()`, this call will unlock clocks. This returns clocks
    to their default behavior of automatically boosting above base clocks as
    thermal limits allow.
    
    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `InvalidArg`, if the `Device` is invalid
    * `NotSupported`, if this `Device` does not support this feature
    * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    * `Unknown`, on any unexpected error
    
    # Device Support

    Supports Fermi and newer non-GeForce fully supported devices and Maxwell or newer
    GeForce devices.
    */
    // Checked against local
    // Tested (no-run)
    #[inline]
    pub fn reset_applications_clocks(&mut self) -> Result<()> {
        unsafe { nvml_try(nvmlDeviceResetApplicationsClocks(self.device)) }
    }

    /**
    Try to set the current state of auto boosted clocks on this `Device`.
    
    Auto boosted clocks are enabled by default on some hardware, allowing the GPU to run
    as fast as thermals will allow it to. Auto boosted clocks should be disabled if fixed
    clock rates are desired.
    
    On Pascal and newer hardware, auto boosted clocks are controlled through application
    clocks. Use `.set_applications_clocks()` and `.reset_applications_clocks()` to control
    auto boost behavior.
    
    Non-root users may use this API by default, but access can be restricted by root using 
    `.set_api_restriction()`.
    
    Note: persistence mode is required to modify the curent auto boost settings and
    therefore must be enabled.
    
    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `InvalidArg`, if the `Device` is invalid
    * `NotSupported`, if this `Device` does not support auto boosted clocks
    * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    * `Unknown`, on any unexpected error
    
    Not sure why nothing is said about `NoPermission`.
    
    # Device Support

    Supports Kepler and newer fully supported devices.
    */
    // Checked against local
    // Tested (no-run)
    #[inline]
    pub fn set_auto_boosted_clocks(&mut self, enabled: bool) -> Result<()> {
        unsafe {
            nvml_try(nvmlDeviceSetAutoBoostedClocksEnabled(
                self.device,
                state_from_bool(enabled)
            ))
        }
    }

    /**
    Sets the ideal affinity for the calling thread and `Device` based on the guidelines given in
    `.cpu_affinity()`.
    
    Currently supports up to 64 processors.
    
    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `InvalidArg`, if the `Device` is invalid
    * `NotSupported`, if this `Device` does not support this feature
    * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    * `Unknown`, on any unexpected error
    
    # Device Support

    Supports Kepler and newer fully supported devices.
    
    # Platform Support

    Only supports Linux.
    */
    // Checked against local
    // Tested (no-run)
    #[cfg(target_os = "linux")]
    #[inline]
    pub fn set_cpu_affinity(&mut self) -> Result<()> {
        unsafe { nvml_try(nvmlDeviceSetCpuAffinity(self.device)) }
    }

    /**
    Try to set the default state of auto boosted clocks on this `Device`.
    
    This is the default state that auto boosted clocks will return to when no compute
    processes (e.g. CUDA application with an active context) are running.
    
    Requires root/admin permissions.
    
    Auto boosted clocks are enabled by default on some hardware, allowing the GPU to run
    as fast as thermals will allow it to. Auto boosted clocks should be disabled if fixed
    clock rates are desired.
    
    On Pascal and newer hardware, auto boosted clocks are controlled through application
    clocks. Use `.set_applications_clocks()` and `.reset_applications_clocks()` to control
    auto boost behavior.
    
    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `NoPermission`, if the calling user does not have permission to change the default state
    * `InvalidArg`, if the `Device` is invalid
    * `NotSupported`, if this `Device` does not support auto boosted clocks
    * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    * `Unknown`, on any unexpected error
    
    # Device Support

    Supports Kepler or newer non-GeForce fully supported devices and Maxwell or newer
    GeForce devices.
    */
    // Checked against local
    // Tested (no-run)
    #[inline]
    pub fn set_auto_boosted_clocks_default(&mut self, enabled: bool) -> Result<()> {
        unsafe {
            // Passing 0 because NVIDIA says flags are not supported yet
            nvml_try(nvmlDeviceSetDefaultAutoBoostedClocksEnabled(
                self.device,
                state_from_bool(enabled),
                0
            ))
        }
    }

    /**
    Reads the infoROM from this `Device`'s flash and verifies the checksum.
    
    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `CorruptedInfoROM`, if this `Device`'s infoROM is corrupted
    * `NotSupported`, if this `Device` does not support this feature
    * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    * `Unknown`, on any unexpected error
    
    Not sure why `InvalidArg` is not mentioned.
    
    # Device Support

    Supports all devices with an infoROM.
    */
    // Checked against local
    // Tested on machines other than my own
    #[inline]
    pub fn validate_info_rom(&self) -> Result<()> {
        unsafe { nvml_try(nvmlDeviceValidateInforom(self.device)) }
    }

    // Wrappers for things from Accounting Statistics now

    /**
    Clears accounting information about all processes that have already terminated.
    
    Requires root/admin permissions.
    
    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `InvalidArg`, if the `Device` is invalid
    * `NotSupported`, if this `Device` does not support this feature
    * `NoPermission`, if the user doesn't have permission to perform this operation
    * `Unknown`, on any unexpected error
    
    # Device Support

    Supports Kepler and newer fully supported devices.
    */
    // Checked against local
    // Tested (no-run)
    #[inline]
    pub fn clear_accounting_pids(&mut self) -> Result<()> {
        unsafe { nvml_try(nvmlDeviceClearAccountingPids(self.device)) }
    }

    /**
    Gets the number of processes that the circular buffer with accounting PIDs can hold
    (in number of elements).
    
    This is the max number of processes that accounting information will be stored for
    before the oldest process information will get overwritten by information
    about new processes.
    
    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `InvalidArg`, if the `Device` is invalid
    * `NotSupported`, if this `Device` does not support this feature or accounting mode
    is disabled
    * `Unknown`, on any unexpected error
    
    # Device Support

    Supports Kepler and newer fully supported devices.
    */
    // Checked against local
    // Tested
    #[inline]
    pub fn accounting_buffer_size(&self) -> Result<u32> {
        unsafe {
            let mut count: c_uint = mem::zeroed();
            nvml_try(nvmlDeviceGetAccountingBufferSize(self.device, &mut count))?;

            Ok(count)
        }
    }

    /**
    Gets whether or not per-process accounting mode is enabled.
    
    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `InvalidArg`, if the `Device` is invalid
    * `NotSupported`, if this `Device` does not support this feature
    * `UnexpectedVariant`, for which you can read the docs for
    * `Unknown`, on any unexpected error
    
    # Device Support

    Supports Kepler and newer fully supported devices.
    */
    // Checked against local
    // Tested
    #[inline]
    pub fn is_accounting_enabled(&self) -> Result<bool> {
        unsafe {
            let mut state: nvmlEnableState_t = mem::zeroed();
            nvml_try(nvmlDeviceGetAccountingMode(self.device, &mut state))?;

            Ok(bool_from_state(state)?)
        }
    }

    /**
    Gets the list of processes that can be queried for accounting stats.
    
    The list of processes returned can be in running or terminated state. Note that
    in the case of a PID collision some processes might not be accessible before
    the circular buffer is full.
    
    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `InvalidArg`, if the `Device` is invalid
    * `NotSupported`, if this `Device` does not support this feature or accounting
    mode is disabled
    * `Unknown`, on any unexpected error
    */
    // Checked against local
    // Tested
    #[inline]
    pub fn accounting_pids(&self) -> Result<Vec<u32>> {
        unsafe {
            let mut count = match self.accounting_pids_count()? {
                0 => return Ok(vec![]),
                value => value,
            };
            let mut pids: Vec<c_uint> = vec![mem::zeroed(); count as usize];

            nvml_try(nvmlDeviceGetAccountingPids(
                self.device,
                &mut count,
                pids.as_mut_ptr()
            ))?;

            Ok(pids)
        }
    }

    // Helper function for the above.
    fn accounting_pids_count(&self) -> Result<c_uint> {
        unsafe {
            // Indicates that we want the count
            let mut count: c_uint = 0;

            // Null also indicates that we want the count
            match nvmlDeviceGetAccountingPids(self.device, &mut count, ptr::null_mut()) {
                // List is empty
                nvmlReturn_enum_NVML_SUCCESS => Ok(0),
                // Count is set to pids count
                nvmlReturn_enum_NVML_ERROR_INSUFFICIENT_SIZE => Ok(count),
                // We know this is an error
                other => nvml_try(other).map(|_| 0),
            }
        }
    }

    /**
    Gets a process's accounting stats.
    
    Accounting stats capture GPU utilization and other statistics across the lifetime
    of a process. Accounting stats can be queried during the lifetime of the process
    and after its termination. The `time` field in `AccountingStats` is reported as
    zero during the lifetime of the process and updated to the actual running time
    after its termination.
    
    Accounting stats are kept in a circular buffer; newly created processes overwrite
    information regarding old processes.
    
    Note:
    * Accounting mode needs to be on. See `.is_accounting_enabled()`.
    * Only compute and graphics applications stats can be queried. Monitoring
    applications can't be queried since they don't contribute to GPU utilization.
    * If a PID collision occurs, the stats of the latest process (the one that
    terminated last) will be reported.
    
    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `InvalidArg`, if the `Device` is invalid
    * `NotFound`, if the process stats were not found
    * `NotSupported`, if this `Device` does not support this feature or accounting
    mode is disabled
    * `Unknown`, on any unexpected error
    
    # Device Support

    Suports Kepler and newer fully supported devices.
    
    # Warning

    On Kepler devices, per-process stats are accurate _only if_ there's one process
    running on this `Device`.
    */
    // Checked against local
    // Tested (for error)
    #[inline]
    pub fn accounting_stats_for(&self, process_id: u32) -> Result<AccountingStats> {
        unsafe {
            let mut stats: nvmlAccountingStats_t = mem::zeroed();

            nvml_try(nvmlDeviceGetAccountingStats(
                self.device,
                process_id,
                &mut stats
            ))?;

            Ok(stats.into())
        }
    }

    /**
    Enables or disables per-process accounting.
    
    Requires root/admin permissions.
    
    Note:
    * This setting is not persistent and will default to disabled after the driver
    unloads. Enable persistence mode to be sure the setting doesn't switch off
    to disabled.
    * Enabling accounting mode has no negative impact on GPU performance.
    * Disabling accounting clears accounting information for all PIDs
    
    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `InvalidArg`, if the `Device` is invalid
    * `NotSupported`, if this `Device` does not support this feature
    * `NoPermission`, if the user doesn't have permission to perform this operation
    * `Unknown`, on any unexpected error
    
    # Device Support

    Supports Kepler and newer fully supported devices.
    */
    // Checked against local
    // Tested (no-run)
    #[inline]
    pub fn set_accounting(&mut self, enabled: bool) -> Result<()> {
        unsafe {
            nvml_try(nvmlDeviceSetAccountingMode(
                self.device,
                state_from_bool(enabled)
            ))
        }
    }

    // Device commands starting here

    /**
    Clears the ECC error and other memory error counts for this `Device`.
    
    Sets all of the specified ECC counters to 0, including both detailed and total counts.
    This operation takes effect immediately.
    
    Requires root/admin permissions and ECC mode to be enabled.
    
    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `InvalidArg`, if the `Device` is invalid or `counter_type` is invalid (shouldn't occur?)
    * `NotSupported`, if this `Device` does not support this feature
    * `NoPermission`, if the user doesn't have permission to perform this operation
    * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    * `Unknown`, on any unexpected error
    
    # Device Support

    Supports Kepler and newer fully supported devices. Only applicable to devices with
    ECC. Requires `InfoRom::ECC` version 2.0 or higher to clear aggregate
    location-based ECC counts. Requires `InfoRom::ECC` version 1.0 or higher to
    clear all other ECC counts.
    */
    // Checked against local
    // Tested (no-run)
    #[inline]
    pub fn clear_ecc_error_counts(&mut self, counter_type: EccCounter) -> Result<()> {
        unsafe {
            nvml_try(nvmlDeviceClearEccErrorCounts(
                self.device,
                counter_type.as_c()
            ))
        }
    }

    /**
    Changes the root/admin restrictions on certain APIs.
    
    This method can be used by a root/admin user to give non root/admin users access
    to certain otherwise-restricted APIs. The new setting lasts for the lifetime of
    the NVIDIA driver; it is not persistent. See `.is_api_restricted()` to query
    current settings.
    
    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `InvalidArg`, if the `Device` is invalid or `api_type` is invalid (shouldn't occur?)
    * `NotSupported`, if this `Device` does not support changing API restrictions or
    this `Device` does not support the feature that API restrictions are being set for
    (e.g. enabling/disabling auto boosted clocks is not supported by this `Device`).
    * `NoPermission`, if the user doesn't have permission to perform this operation
    * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    * `Unknown`, on any unexpected error
    
    # Device Support

    Supports Kepler and newer fully supported devices.
    */
    // Checked against local
    // Tested (no-run)
    #[inline]
    pub fn set_api_restricted(&mut self, api_type: Api, restricted: bool) -> Result<()> {
        unsafe {
            nvml_try(nvmlDeviceSetAPIRestriction(
                self.device,
                api_type.as_c(),
                state_from_bool(restricted)
            ))
        }
    }

    /**
    Sets clocks that applications will lock to.
    
    Sets the clocks that compute and graphics applications will be running at. e.g.
    CUDA driver requests these clocks during context creation which means this
    property defines clocks at which CUDA applications will be running unless some
    overspec event occurs (e.g. over power, over thermal or external HW brake).
    
    Can be used as a setting to request constant performance. Requires root/admin
    permissions.
    
    On Pascal and newer hardware, this will automatically disable automatic boosting
    of clocks. On K80 and newer Kepler and Maxwell GPUs, users desiring fixed performance
    should also call `.set_auto_boosted_clocks(false)` to prevent clocks from automatically
    boosting above the clock value being set here.

    You can determine valid `mem_clock` and `graphics_clock` arg values via
    `.supported_memory_clocks()` and `.supported_graphics_clocks()`.
    
    Note that after a system reboot or driver reload applications clocks go back
    to their default value.
    
    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `InvalidArg`, if the `Device` is invalid or the clocks are not a valid combo
    * `NotSupported`, if this `Device` does not support this feature
    * `NoPermission`, if the user doesn't have permission to perform this operation
    * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    * `Unknown`, on any unexpected error
    
    # Device Support

    Supports Kepler and newer non-GeForce fully supported devices and Maxwell or newer
    GeForce devices.
    */
    // Checked against local
    // Tested (no-run)
    #[inline]
    pub fn set_applications_clocks(&mut self, mem_clock: u32, graphics_clock: u32) -> Result<()> {
        unsafe {
            nvml_try(nvmlDeviceSetApplicationsClocks(
                self.device,
                mem_clock,
                graphics_clock
            ))
        }
    }

    /**
    Sets the compute mode for this `Device`.
    
    The compute mode determines whether a GPU can be used for compute operations
    and whether it can be shared across contexts.
    
    This operation takes effect immediately. Under Linux it is not persistent
    across reboots and always resets to `Default`. Under Windows it is
    persistent.
    
    Under Windows, compute mode may only be set to `Default` when running in WDDM
    (physical display connected).
    
    Requires root/admin permissions.
    
    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `InvalidArg`, if the `Device` is invalid or `mode` is invalid (shouldn't occur?)
    * `NotSupported`, if this `Device` does not support this feature
    * `NoPermission`, if the user doesn't have permission to perform this operation
    * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    * `Unknown`, on any unexpected error
    */
    // Checked against local
    // Tested (no-run)
    #[inline]
    pub fn set_compute_mode(&mut self, mode: ComputeMode) -> Result<()> {
        unsafe { nvml_try(nvmlDeviceSetComputeMode(self.device, mode.as_c())) }
    }

    /**
    Sets the driver model for this `Device`.
    
    This operation takes effect after the next reboot. The model may only be
    set to WDDM when running in DEFAULT compute mode. Changing the model to
    WDDM is not supported when the GPU doesn't support graphics acceleration
    or will not support it after a reboot.
    
    On Windows platforms the device driver can run in either WDDM or WDM (TCC)
    mode. If a physical display is attached to a device it must run in WDDM mode.
    
    It is possible to force the change to WDM (TCC) while the display is still
    attached with a `Behavior` of `FORCE`. This should only be done if the host
    is subsequently powered down and the display is detached from this `Device`
    before the next reboot.
    
    Requires root/admin permissions.
    
    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `InvalidArg`, if the `Device` is invalid or `model` is invalid (shouldn't occur?)
    * `NotSupported`, if this `Device` does not support this feature
    * `NoPermission`, if the user doesn't have permission to perform this operation
    * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    * `Unknown`, on any unexpected error
    
    # Device Support

    Supports Fermi and newer fully supported devices.
    
    # Platform Support

    Only supports Windows.

    # Examples

    ```no_run
    # use nvml_wrapper::NVML;
    # use nvml_wrapper::error::*;
    # fn test() -> Result<()> {
    # let nvml = NVML::init()?;
    # let mut device = nvml.device_by_index(0)?;
    use nvml_wrapper::bitmasks::Behavior;
    use nvml_wrapper::enum_wrappers::device::DriverModel;

    device.set_driver_model(DriverModel::WDM, Behavior::DEFAULT)?;

    // Force the change to WDM (TCC)
    device.set_driver_model(DriverModel::WDM, Behavior::FORCE)?;
    # Ok(())
    # }
    ```
    */
    // Checked against local
    // Tested (no-run)
    #[cfg(target_os = "windows")]
    #[inline]
    pub fn set_driver_model(&mut self, model: DriverModel, flags: Behavior) -> Result<()> {
        unsafe {
            nvml_try(nvmlDeviceSetDriverModel(
                self.device,
                model.as_c(),
                flags.bits()
            ))
        }
    }

    /**
    Set whether or not ECC mode is enabled for this `Device`.
    
    Requires root/admin permissions. Only applicable to devices with ECC.
    
    This operation takes effect after the next reboot.
    
    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `InvalidArg`, if the `Device` is invalid
    * `NotSupported`, if this `Device` does not support this feature
    * `NoPermission`, if the user doesn't have permission to perform this operation
    * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    * `Unknown`, on any unexpected error
    
    # Device Support

    Supports Kepler and newer fully supported devices. Requires `InfoRom::ECC` version
    1.0 or higher.
    */
    // Checked against local
    // Tested (no-run)
    #[inline]
    pub fn set_ecc(&mut self, enabled: bool) -> Result<()> {
        unsafe { nvml_try(nvmlDeviceSetEccMode(self.device, state_from_bool(enabled))) }
    }

    /**
    Sets the GPU operation mode for this `Device`.
    
    Requires root/admin permissions. Changing GOMs requires a reboot, a requirement
    that may be removed in the future.
    
    Compute only GOMs don't support graphics acceleration. Under Windows switching
    to these GOMs when the pending driver model is WDDM (physical display attached)
    is not supported.
    
    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `InvalidArg`, if the `Device` is invalid or `mode` is invalid (shouldn't occur?)
    * `NotSupported`, if this `Device` does not support GOMs or a specific mode
    * `NoPermission`, if the user doesn't have permission to perform this operation
    * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    * `Unknown`, on any unexpected error
    
    # Device Support

    Supports GK110 M-class and X-class Tesla products from the Kepler family. Modes
    `LowDP` and `AllOn` are supported on fully supported GeForce products. Not
    supported on Quadro and Tesla C-class products.
    */
    // Checked against local
    // Tested (no-run)
    #[inline]
    pub fn set_gpu_op_mode(&mut self, mode: OperationMode) -> Result<()> {
        unsafe { nvml_try(nvmlDeviceSetGpuOperationMode(self.device, mode.as_c())) }
    }

    /**
    Sets the persistence mode for this `Device`.
    
    The persistence mode determines whether the GPU driver software is torn down
    after the last client exits.
    
    This operation takes effect immediately and requires root/admin permissions.
    It is not persistent across reboots; after each reboot it will default to
    disabled.
    
    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `InvalidArg`, if the `Device` is invalid
    * `NotSupported`, if this `Device` does not support this feature
    * `NoPermission`, if the user doesn't have permission to perform this operation
    * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    * `Unknown`, on any unexpected error
    
    # Platform Support

    Only supports Linux.
    */
    // Checked against local
    // Tested (no-run)
    #[cfg(target_os = "linux")]
    #[inline]
    pub fn set_persistent(&mut self, enabled: bool) -> Result<()> {
        unsafe {
            nvml_try(nvmlDeviceSetPersistenceMode(
                self.device,
                state_from_bool(enabled)
            ))
        }
    }

    /**
    Sets the power limit for this `Device`, in milliwatts.
    
    This limit is not persistent across reboots or driver unloads. Enable
    persistent mode to prevent the driver from unloading when no application
    is using this `Device`.
    
    Requires root/admin permissions. See `.power_management_limit_constraints()`
    to check the allowed range of values.
    
    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `InvalidArg`, if the `Device` is invalid or `limit` is out of range
    * `NotSupported`, if this `Device` does not support this feature
    * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    * `Unknown`, on any unexpected error
    
    For some reason NVIDIA does not mention `NoPermission`.
    
    # Device Support

    Supports Kepler and newer fully supported devices.
    */
    // Checked against local
    // Tested (no-run)
    #[inline]
    pub fn set_power_management_limit(&mut self, limit: u32) -> Result<()> {
        unsafe { nvml_try(nvmlDeviceSetPowerManagementLimit(self.device, limit)) }
    }

    // Event handling methods

    /**
    Starts recording the given `EventTypes` for this `Device` and adding them
    to the specified `EventSet`.

    Use `.supported_event_types()` to find out which events you can register for
    this `Device`.

    **Unfortunately, due to the way `error-chain` works, there is no way to
    return the set if it is still valid after an error has occured with the
    register call.** The set that you passed in will be freed if any error
    occurs and will not be returned to you. This is not desired behavior
    and I will fix it as soon as it is possible to do so.
    
    All events that occurred before this call was made will not be recorded.
    
    ECC events are only available on `Device`s with ECC enabled. Power capping events
    are only available on `Device`s with power management enabled.
    
    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `InvalidArg`, if `events` is invalid (shouldn't occur?)
    * `NotSupported`, if the platform does not support this feature or some of the
    requested event types.
    * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    * `Unknown`, on any unexpected error. **If this error is returned, the `set` you
    passed in has had its resources freed and will not be returned to you**. NVIDIA's
    docs say that this error means that the set is in an invalid state.
    
    # Device Support

    Supports Fermi and newer fully supported devices.
    
    # Platform Support

    Only supports Linux.

    # Examples

    ```
    # use nvml_wrapper::NVML;
    # use nvml_wrapper::error::*;
    # fn main() -> Result<()> {
    # let nvml = NVML::init()?;
    # let device = nvml.device_by_index(0)?;
    use nvml_wrapper::bitmasks::event::EventTypes;

    let set = nvml.create_event_set()?;

    /*
    Register both `CLOCK_CHANGE` and `PSTATE_CHANGE`.

    `let set = ...` is a quick way to re-bind the set to the same variable, since
    `.register_events()` consumes the set in order to enforce safety and returns it
    if everything went well. It does *not* require `set` to be mutable as nothing
    is being mutated.
    */
    let set = device.register_events(
        EventTypes::CLOCK_CHANGE |
        EventTypes::PSTATE_CHANGE,
        set
    )?;
    # Ok(())
    # }
    ```
    */
    // Checked against local
    // Tested
    // Thanks to Thinkofname for helping resolve lifetime issues
    #[cfg(target_os = "linux")]
    #[inline]
    pub fn register_events(
        &self,
        events: EventTypes,
        set: EventSet<'nvml>,
    ) -> Result<EventSet<'nvml>> {
        unsafe {
            match nvml_try(nvmlDeviceRegisterEvents(
                self.device,
                events.bits(),
                set.unsafe_raw()
            )) {
                Ok(()) => Ok(set),
                Err(Error(ErrorKind::Unknown, _)) => {
                    // NVIDIA says that if an Unknown error is returned, `set` will
                    // be in an undefined state and should be freed.
                    set.release_events().chain_err(|| ErrorKind::SetReleaseFailed)?;
                    bail!(ErrorKind::Unknown)
                },
                Err(e) => {
                    // TODO: So... unfortunately error-chain provides us with no way
                    // to return the set here, even if it's still valid.
                    //
                    // For now we just... get rid of it and force you to create
                    // another one.
                    set.release_events().chain_err(|| ErrorKind::SetReleaseFailed)?;
                    Err(e)
                },
            }
        }
    }

    /**
    Gets the `EventTypes` that this `Device` supports.

    The returned bitmask is created via the `EventTypes::from_bits_truncate`
    method, meaning that any bits that don't correspond to flags present in this
    version of the wrapper will be dropped.
    
    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    * `Unknown`, on any unexpected error
    
    # Device Support

    Supports Fermi and newer fully supported devices.
    
    # Platform Support

    Only supports Linux.

    # Examples

    ```
    # use nvml_wrapper::NVML;
    # use nvml_wrapper::error::*;
    # fn main() -> Result<()> {
    # let nvml = NVML::init()?;
    # let device = nvml.device_by_index(0)?;
    use nvml_wrapper::bitmasks::event::EventTypes;

    let supported = device.supported_event_types()?;

    if supported.contains(EventTypes::CLOCK_CHANGE) {
        println!("The `CLOCK_CHANGE` event is supported.");
    } else if supported.contains(
        EventTypes::SINGLE_BIT_ECC_ERROR |
        EventTypes::DOUBLE_BIT_ECC_ERROR
    ) {
        println!("All ECC error event types are supported.");
    }
    # Ok(())
    # }
    ```
    */
    // Tested
    #[cfg(target_os = "linux")]
    #[inline]
    pub fn supported_event_types(&self) -> Result<EventTypes> {
        Ok(EventTypes::from_bits_truncate(self.supported_event_types_raw()?))
    }

    /**
    Gets the `EventTypes` that this `Device` supports, erroring if any bits
    correspond to non-present flags.
    
    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `IncorrectBits`, if NVML returns any bits that do not correspond to flags in
    `EventTypes`
    * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    * `Unknown`, on any unexpected error
    
    # Device Support

    Supports Fermi and newer fully supported devices.
    
    # Platform Support

    Only supports Linux.
    */
    // Tested
    #[cfg(target_os = "linux")]
    #[inline]
    pub fn supported_event_types_strict(&self) -> Result<EventTypes> {
        let ev_types = self.supported_event_types_raw()?;

        EventTypes::from_bits(ev_types)
            .ok_or_else(|| ErrorKind::IncorrectBits(Bits::U64(ev_types)).into())
    }

    // Helper for the above methods.
    #[cfg(target_os = "linux")]
    #[inline]
    fn supported_event_types_raw(&self) -> Result<c_ulonglong> {
        unsafe {
            let mut ev_types: c_ulonglong = mem::zeroed();
            nvml_try(nvmlDeviceGetSupportedEventTypes(self.device, &mut ev_types))?;

            Ok(ev_types)
        }
    }

    // Drain states

    /**
    Enable or disable drain state for this `Device`.

    If you pass `None` as `pci_info`, `.pci_info()` will be called in order to obtain
    `PciInfo` to be used within this method.
    
    Enabling drain state forces this `Device` to no longer accept new incoming requests.
    Any new NVML processes will no longer see this `Device`.
    
    Must be called as administrator. Persistence mode for this `Device` must be turned
    off before this call is made.
    
    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `NotSupported`, if this `Device` doesn't support this feature
    * `NoPermission`, if the calling process has insufficient permissions to perform
    this operation
    * `InUse`, if this `Device` has persistence mode turned on
    * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    * `Unknown`, on any unexpected error

    In addition, all of the errors returned by:

    * `.pci_info()`
    * `PciInfo.try_into_c()`
    
    # Device Support

    Supports Pascal and newer fully supported devices.
    
    Some Kepler devices are also supported (that's all NVIDIA says, no specifics).
    
    # Platform Support

    Only supports Linux.

    # Examples

    ```no_run
    # use nvml_wrapper::NVML;
    # use nvml_wrapper::error::*;
    # fn test() -> Result<()> {
    # let nvml = NVML::init()?;
    # let mut device = nvml.device_by_index(0)?;
    // Pass `None`, `.set_drain()` call will grab `PciInfo` for us
    device.set_drain(true, None)?;

    let pci_info = device.pci_info()?;

    // Pass in our own `PciInfo`, call will use it instead
    device.set_drain(true, pci_info)?;
    # Ok(())
    # }
    ```
    */
    // Checked against local
    #[cfg(target_os = "linux")]
    #[inline]
    pub fn set_drain<T: Into<Option<PciInfo>>>(
        &mut self,
        enabled: bool,
        pci_info: T,
    ) -> Result<()> {

        let pci_info = if let Some(info) = pci_info.into() {
            info
        } else {
            self.pci_info()?
        };

        unsafe {
            nvml_try(nvmlDeviceModifyDrainState(
                &mut pci_info.try_into_c()?,
                state_from_bool(enabled)
            ))
        }
    }

    /**
    Query the drain state of this `Device`.

    If you pass `None` as `pci_info`, `.pci_info()` will be called in order to obtain
    `PciInfo` to be used within this method.
    
    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `NotSupported`, if this `Device` doesn't support this feature
    * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    * `UnexpectedVariant`, for which you can read the docs for
    * `Unknown`, on any unexpected error

    In addition, all of the errors returned by:

    * `.pci_info()`
    * `PciInfo.try_into_c()`
    
    # Device Support

    Supports Pascal and newer fully supported devices.
    
    Some Kepler devices are also supported (that's all NVIDIA says, no specifics).
    
    # Platform Support

    Only supports Linux.

    # Examples

    ```
    # use nvml_wrapper::NVML;
    # use nvml_wrapper::error::*;
    # fn main() -> Result<()> {
    # let nvml = NVML::init()?;
    # let mut device = nvml.device_by_index(0)?;
    // Pass `None`, `.is_drain_enabled()` call will grab `PciInfo` for us
    device.is_drain_enabled(None)?;

    let pci_info = device.pci_info()?;

    // Pass in our own `PciInfo`, call will use it instead
    device.is_drain_enabled(pci_info)?;
    # Ok(())
    # }
    ```
    */
    // Checked against local
    // Tested
    #[cfg(target_os = "linux")]
    #[inline]
    pub fn is_drain_enabled<T: Into<Option<PciInfo>>>(&self, pci_info: T) -> Result<bool> {
        let pci_info = if let Some(info) = pci_info.into() {
            info
        } else {
            self.pci_info()?
        };

        unsafe {
            let mut state: nvmlEnableState_t = mem::zeroed();

            nvml_try(nvmlDeviceQueryDrainState(
                &mut pci_info.try_into_c()?,
                &mut state
            ))?;

            Ok(bool_from_state(state)?)
        }
    }

    /**
    Removes this `Device` from the view of both NVML and the NVIDIA kernel driver.

    If you pass `None` as `pci_info`, `.pci_info()` will be called in order to obtain
    `PciInfo` to be used within this method.
    
    This call only works if no other processes are attached. If other processes
    are attached when this is called, the `InUse` error will be returned and
    this `Device` will return to its original draining state. The only situation
    where this can occur is if a process was and is still using this `Device`
    before the call to `set_drain()` was made and it was enabled. Note that
    persistence mode counts as an attachment to this `Device` and thus must be
    disabled prior to this call.
    
    For long-running NVML processes, please note that this will change the
    enumeration of current GPUs. As an example, if there are four GPUs present
    and the first is removed, the new enumeration will be 0-2. Device handles
    for the removed GPU will be invalid.
    
    Must be run as administrator.

    # Bad Ergonomics Explanation

    Ideally the `Device` would be returned within the `Error` in the case of an
    error occuring during this call. Unfortunately, `error-chain` / `quick-error`
    do not support generic lifetime parameters, meaning I cannot return the
    `Device` in an `ErrorKind` variant.

    Not being able to recover the `Device` after an error in this call would
    break the functionality, so I worked around this limitation with a
    less-than-satisfactory solution.
    
    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `NotSupported`, if this `Device` doesn't support this feature
    * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    * `InUse`, if this `Device` is still in use and cannot be removed

    In addition, all of the errors returned by:

    * `.pci_info()`
    * `PciInfo.try_into_c()`
    
    # Device Support

    Supports Pascal and newer fully supported devices.
    
    Some Kepler devices are also supported (that's all NVIDIA says, no specifics).
    
    # Platform Support

    Only supports Linux.

    # Examples
    
    How to handle error case:

    ```no_run
    # use nvml_wrapper::NVML;
    # use nvml_wrapper::error::*;
    # fn test() -> Result<()> {
    # let nvml = NVML::init()?;
    # let mut device = nvml.device_by_index(0)?;
    match device.remove(None) {
        (Ok(()), None) => println!("Successful call, `Device` removed"),
        (Err(e), Some(d)) => println!("Unsuccessful call. `Device`: {:?}", d),
        _ => println!("Something else",)
    }
    # Ok(())
    # }
    ```
    Demonstration of the `pci_info` parameter's use:

    ```no_run
    # use nvml_wrapper::NVML;
    # use nvml_wrapper::error::*;
    # fn test() -> Result<()> {
    # let nvml = NVML::init()?;
    # let mut device = nvml.device_by_index(0)?;
    // Pass `None`, `.remove()` call will grab `PciInfo` for us
    device.remove(None).0?;

    # let mut device2 = nvml.device_by_index(0)?;
    // Different `Device` because `.remove()` consumes the `Device`
    let pci_info = device2.pci_info()?;

    // Pass in our own `PciInfo`, call will use it instead
    device2.remove(pci_info).0?;
    # Ok(())
    # }
    ```
    */
    // Checked against local
    // TODO: Fix ergonomics here when possible.
    #[cfg(target_os = "linux")]
    #[inline]
    pub fn remove<T: Into<Option<PciInfo>>>(
        self,
        pci_info: T,
    ) -> (Result<()>, Option<Device<'nvml>>) {

        let pci_info = if let Some(info) = pci_info.into() {
            info
        } else {
            match self.pci_info() {
                Ok(info) => info,
                Err(e) => return (Err(e).chain_err(|| ErrorKind::GetPciInfoFailed), Some(self)),
            }
        };

        let mut raw_pci_info = match pci_info.try_into_c() {
            Ok(info) => info,
            Err(e) => return (Err(e).chain_err(|| ErrorKind::PciInfoToCFailed), Some(self)),
        };

        unsafe {
            match nvml_try(nvmlDeviceRemoveGpu(&mut raw_pci_info)) {
                // `Device` removed; call was successful, no `Device` to return
                Ok(()) => (Ok(()), None),
                // `Device` has not been removed; unsuccessful call, return `Device`
                Err(e) => (Err(e), Some(self)),
            }
        }
    }

    // NvLink

    /**
    Obtain a struct that represents an NvLink.

    NVIDIA does not provide any information as to how to obtain a valid NvLink
    value, so you're on your own there.
    */
    #[inline]
    pub fn link_wrapper_for(&self, link: u32) -> NvLink {
        NvLink {
            device: self,
            link
        }
    }

    /// Consume the struct and obtain the raw device handle that it contains.
    #[inline]
    pub fn into_raw(self) -> nvmlDevice_t {
        self.device
    }

    /// Obtain a reference to the raw device handle contained in the struct.
    #[inline]
    pub fn as_raw(&self) -> &nvmlDevice_t {
        &(self.device)
    }

    /// Obtain a mutable reference to the raw device handle contained in the
    /// struct.
    #[inline]
    pub fn as_mut_raw(&mut self) -> &mut nvmlDevice_t {
        &mut (self.device)
    }

    /// Sometimes necessary for C interop. Use carefully.
    #[inline]
    pub unsafe fn unsafe_raw(&self) -> nvmlDevice_t {
        self.device
    }
}

#[cfg(test)]
#[deny(unused_mut)]
mod test {
    use super::Device;
    #[cfg(target_os = "windows")]
    use bitmasks::Behavior;
    #[cfg(target_os = "linux")]
    use bitmasks::event::*;
    use enum_wrappers::device::*;
    use error::*;
    use test_utils::*;
    use sys_exports::field_id::*;
    use structs::device::FieldId;

    #[test]
    fn device_is_send() {
        assert_send::<Device>()
    }

    #[test]
    fn device_is_sync() {
        assert_sync::<Device>()
    }

    // This modifies device state, so we don't want to actually run the test
    #[allow(dead_code)]
    #[cfg(target_os = "linux")]
    fn clear_cpu_affinity() {
        let nvml = nvml();
        let mut device = device(&nvml);

        device.clear_cpu_affinity().unwrap();
    }

    #[test]
    fn is_api_restricted() {
        let nvml = nvml();
        test_with_device(3, &nvml, |device| {
            device.is_api_restricted(Api::ApplicationClocks)
            // AutoBoostedClocks is not supported on my machine, so not testing
        })
    }

    #[test]
    fn applications_clock() {
        let nvml = nvml();
        test_with_device(3, &nvml, |device| {
            let gfx_clock =
                device.applications_clock(Clock::Graphics).chain_err(|| "graphics clock")?;
            let sm_clock = device.applications_clock(Clock::SM).chain_err(|| "sm clock")?;
            let mem_clock = device.applications_clock(Clock::Memory).chain_err(|| "memory clock")?;
            let vid_clock = device.applications_clock(Clock::Video).chain_err(|| "video clock")?;

            Ok(format!(
                "Graphics Clock: {}, SM Clock: {}, Memory Clock: {}, Video Clock: {}",
                gfx_clock,
                sm_clock,
                mem_clock,
                vid_clock
            ))
        })
    }

    // My machine does not support this call
    #[cfg(not(feature = "test-local"))]
    #[test]
    fn auto_boosted_clocks_enabled() {
        let nvml = nvml();
        test_with_device(3, &nvml, |device| device.auto_boosted_clocks_enabled())
    }

    #[test]
    fn bar1_memory_info() {
        let nvml = nvml();
        test_with_device(3, &nvml, |device| device.bar1_memory_info())
    }

    #[test]
    fn board_id() {
        let nvml = nvml();
        test_with_device(3, &nvml, |device| device.board_id())
    }

    #[test]
    fn brand() {
        let nvml = nvml();
        test_with_device(3, &nvml, |device| device.brand())
    }

    // My machine does not support this call
    #[cfg(not(feature = "test-local"))]
    #[test]
    fn bridge_chip_info() {
        let nvml = nvml();
        test_with_device(3, &nvml, |device| device.bridge_chip_info())
    }

    #[test]
    fn clock() {
        let nvml = nvml();
        test_with_device(3, &nvml, |device| {
            device
                .clock(Clock::Graphics, ClockId::Current)
                .chain_err(|| "graphics + current")?;
            device.clock(Clock::SM, ClockId::TargetAppClock).chain_err(|| "SM + target")?;
            device
                .clock(Clock::Memory, ClockId::DefaultAppClock)
                .chain_err(|| "mem + default")?;
            device
                .clock(Clock::Video, ClockId::TargetAppClock)
                .chain_err(|| "video + target")
            // My machine does not support CustomerMaxBoost
        })
    }

    // My machine does not support this call
    #[cfg(not(feature = "test-local"))]
    #[test]
    fn max_customer_boost_clock() {
        let nvml = nvml();
        test_with_device(3, &nvml, |device| {
            device.max_customer_boost_clock(Clock::Graphics).chain_err(|| "graphics")?;
            device.max_customer_boost_clock(Clock::SM).chain_err(|| "SM")?;
            device.max_customer_boost_clock(Clock::Memory).chain_err(|| "mem")?;
            device.max_customer_boost_clock(Clock::Video).chain_err(|| "video")
        })
    }

    #[test]
    fn compute_mode() {
        let nvml = nvml();
        test_with_device(3, &nvml, |device| device.compute_mode())
    }

    #[test]
    fn clock_info() {
        let nvml = nvml();
        test_with_device(3, &nvml, |device| {
            let gfx_clock = device.clock_info(Clock::Graphics).chain_err(|| "graphics clock")?;
            let sm_clock = device.clock_info(Clock::SM).chain_err(|| "sm clock")?;
            let mem_clock = device.clock_info(Clock::Memory).chain_err(|| "memory clock")?;
            let vid_clock = device.clock_info(Clock::Video).chain_err(|| "video clock")?;

            Ok(format!(
                "Graphics Clock: {}, SM Clock: {}, Memory Clock: {}, Video Clock: {}",
                gfx_clock,
                sm_clock,
                mem_clock,
                vid_clock
            ))
        })
    }

    #[test]
    fn running_compute_processes() {
        let nvml = nvml();
        test_with_device(3, &nvml, |device| device.running_compute_processes())
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn cpu_affinity() {
        let nvml = nvml();
        test_with_device(3, &nvml, |device| device.cpu_affinity(64))
    }

    #[test]
    fn current_pcie_link_gen() {
        let nvml = nvml();
        test_with_device(3, &nvml, |device| device.current_pcie_link_gen())
    }

    #[test]
    fn current_pcie_link_width() {
        let nvml = nvml();
        test_with_device(3, &nvml, |device| device.current_pcie_link_width())
    }

    #[test]
    fn decoder_utilization() {
        let nvml = nvml();
        test_with_device(3, &nvml, |device| device.decoder_utilization())
    }

    #[test]
    fn default_applications_clock() {
        let nvml = nvml();
        test_with_device(3, &nvml, |device| {
            let gfx_clock = device
                .default_applications_clock(Clock::Graphics)
                .chain_err(|| "graphics clock")?;
            let sm_clock = device.default_applications_clock(Clock::SM).chain_err(|| "sm clock")?;
            let mem_clock =
                device.default_applications_clock(Clock::Memory).chain_err(|| "memory clock")?;
            let vid_clock =
                device.default_applications_clock(Clock::Video).chain_err(|| "video clock")?;

            Ok(format!(
                "Graphics Clock: {}, SM Clock: {}, Memory Clock: {}, Video Clock: {}",
                gfx_clock,
                sm_clock,
                mem_clock,
                vid_clock
            ))
        })
    }

    #[test]
    fn is_display_active() {
        let nvml = nvml();
        test_with_device(3, &nvml, |device| device.is_display_active())
    }

    #[test]
    fn is_display_connected() {
        let nvml = nvml();
        test_with_device(3, &nvml, |device| device.is_display_connected())
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn driver_model() {
        let nvml = nvml();
        test_with_device(3, &nvml, |device| device.driver_model())
    }

    // My machine does not support this call
    #[cfg(not(feature = "test-local"))]
    #[test]
    fn is_ecc_enabled() {
        let nvml = nvml();
        test_with_device(3, &nvml, |device| device.is_ecc_enabled())
    }

    #[test]
    fn encoder_utilization() {
        let nvml = nvml();
        test_with_device(3, &nvml, |device| device.encoder_utilization())
    }

    #[test]
    fn encoder_capacity() {
        let nvml = nvml();
        test_with_device(3, &nvml, |device| device.encoder_capacity(
            EncoderType::H264)
        )
    }

    #[test]
    fn encoder_stats() {
        let nvml = nvml();
        test_with_device(3, &nvml, |device| device.encoder_stats())
    }

    #[test]
    fn encoder_sessions() {
        let nvml = nvml();
        test_with_device(3, &nvml, |device| device.encoder_sessions())
    }

    #[test]
    fn enforced_power_limit() {
        let nvml = nvml();
        test_with_device(3, &nvml, |device| device.enforced_power_limit())
    }

    #[test]
    fn fan_speed() {
        let nvml = nvml();
        test_with_device(3, &nvml, |device| device.fan_speed())
    }

    // My machine does not support this call
    #[cfg(not(feature = "test-local"))]
    #[test]
    fn gpu_operation_mode() {
        let nvml = nvml();
        test_with_device(3, &nvml, |device| device.gpu_operation_mode())
    }

    #[test]
    fn running_graphics_processes() {
        let nvml = nvml();
        test_with_device(3, &nvml, |device| device.running_graphics_processes())
    }

    #[test]
    fn process_utilization_stats() {
        let nvml = nvml();
        test_with_device(3, &nvml, |device| device.process_utilization_stats(None))
    }

    #[test]
    fn index() {
        let nvml = nvml();
        test_with_device(3, &nvml, |device| device.index())
    }

    // My machine does not support this call
    #[cfg(not(feature = "test-local"))]
    #[test]
    fn config_checksum() {
        let nvml = nvml();
        test_with_device(3, &nvml, |device| device.config_checksum())
    }

    // My machine does not support this call
    #[cfg(not(feature = "test-local"))]
    #[test]
    fn info_rom_image_version() {
        let nvml = nvml();
        test_with_device(3, &nvml, |device| device.info_rom_image_version())
    }

    // My machine does not support this call
    #[cfg(not(feature = "test-local"))]
    #[test]
    fn info_rom_version() {
        let nvml = nvml();
        test_with_device(3, &nvml, |device| {
            device.info_rom_version(InfoRom::OEM).chain_err(|| "oem")?;
            device.info_rom_version(InfoRom::ECC).chain_err(|| "ecc")?;
            device.info_rom_version(InfoRom::Power).chain_err(|| "power")
        })
    }

    #[test]
    fn max_clock_info() {
        let nvml = nvml();
        test_with_device(3, &nvml, |device| {
            let gfx_clock = device.max_clock_info(Clock::Graphics).chain_err(|| "graphics clock")?;
            let sm_clock = device.max_clock_info(Clock::SM).chain_err(|| "sm clock")?;
            let mem_clock = device.max_clock_info(Clock::Memory).chain_err(|| "memory clock")?;
            let vid_clock = device.max_clock_info(Clock::Video).chain_err(|| "video clock")?;

            Ok(format!(
                "Graphics Clock: {}, SM Clock: {}, Memory Clock: {}, Video Clock: {}",
                gfx_clock,
                sm_clock,
                mem_clock,
                vid_clock
            ))
        })
    }

    #[test]
    fn max_pcie_link_gen() {
        let nvml = nvml();
        test_with_device(3, &nvml, |device| device.max_pcie_link_gen())
    }

    #[test]
    fn max_pcie_link_width() {
        let nvml = nvml();
        test_with_device(3, &nvml, |device| device.max_pcie_link_width())
    }

    // My machine does not support this call
    #[cfg(not(feature = "test-local"))]
    #[test]
    fn memory_error_counter() {
        let nvml = nvml();
        test_with_device(3, &nvml, |device| {
            device.memory_error_counter(
                MemoryError::Corrected,
                EccCounter::Volatile,
                MemoryLocation::Device
            )
        })
    }

    #[test]
    fn memory_info() {
        let nvml = nvml();
        test_with_device(3, &nvml, |device| device.memory_info())
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn minor_number() {
        let nvml = nvml();
        test_with_device(3, &nvml, |device| device.minor_number())
    }

    #[test]
    fn is_multi_gpu_board() {
        let nvml = nvml();
        test_with_device(3, &nvml, |device| device.is_multi_gpu_board())
    }

    #[test]
    fn name() {
        let nvml = nvml();
        test_with_device(3, &nvml, |device| device.name())
    }

    #[test]
    fn pci_info() {
        let nvml = nvml();
        test_with_device(3, &nvml, |device| device.pci_info())
    }

    #[test]
    fn pcie_replay_counter() {
        let nvml = nvml();
        test_with_device(3, &nvml, |device| device.pcie_replay_counter())
    }

    #[test]
    fn pcie_throughput() {
        let nvml = nvml();
        test_with_device(3, &nvml, |device| {
            device.pcie_throughput(PcieUtilCounter::Send).chain_err(|| "send")?;
            device.pcie_throughput(PcieUtilCounter::Receive).chain_err(|| "receive")
        })
    }

    #[test]
    fn performance_state() {
        let nvml = nvml();
        test_with_device(3, &nvml, |device| device.performance_state())
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn is_in_persistent_mode() {
        let nvml = nvml();
        test_with_device(3, &nvml, |device| device.is_in_persistent_mode())
    }

    #[test]
    fn power_management_limit_default() {
        let nvml = nvml();
        test_with_device(3, &nvml, |device| device.power_management_limit_default())
    }

    #[test]
    fn power_management_limit() {
        let nvml = nvml();
        test_with_device(3, &nvml, |device| device.power_management_limit())
    }

    #[test]
    fn power_management_limit_constraints() {
        let nvml = nvml();
        test_with_device(
            3,
            &nvml,
            |device| device.power_management_limit_constraints()
        )
    }

    #[test]
    fn is_power_management_algo_active() {
        let nvml = nvml();

        #[allow(deprecated)]
        test_with_device(3, &nvml, |device| device.is_power_management_algo_active())
    }

    #[test]
    fn power_state() {
        let nvml = nvml();

        #[allow(deprecated)] test_with_device(3, &nvml, |device| device.power_state())
    }

    #[test]
    fn power_usage() {
        let nvml = nvml();
        test_with_device(3, &nvml, |device| device.power_usage())
    }

    // My machine does not support this call
    #[cfg(not(feature = "test-local"))]
    #[test]
    fn retired_pages() {
        let nvml = nvml();
        test_with_device(3, &nvml, |device| {
            device
                .retired_pages(RetirementCause::MultipleSingleBitEccErrors)
                .chain_err(|| "multiplesinglebit")?;
            device
                .retired_pages(RetirementCause::DoubleBitEccError)
                .chain_err(|| "doublebit")
        })
    }

    // My machine does not support this call
    #[cfg(not(feature = "test-local"))]
    #[test]
    fn are_pages_pending_retired() {
        let nvml = nvml();
        test_with_device(3, &nvml, |device| device.are_pages_pending_retired())
    }

    #[test]
    fn samples() {
        let nvml = nvml();
        test_with_device(3, &nvml, |device| {
            device.samples(Sampling::ProcessorClock, None)?;
            Ok(())
        })
    }

    #[test]
    fn field_values_for() {
        let nvml = nvml();
        test_with_device(3, &nvml, |device| {
            device.field_values_for(&[
                FieldId(NVML_FI_DEV_ECC_CURRENT),
                FieldId(NVML_FI_DEV_ECC_PENDING),
                FieldId(NVML_FI_DEV_ECC_SBE_VOL_TOTAL),
                FieldId(NVML_FI_DEV_ECC_DBE_VOL_TOTAL),
                FieldId(NVML_FI_DEV_ECC_SBE_AGG_TOTAL),
                FieldId(NVML_FI_DEV_ECC_DBE_AGG_TOTAL),
                FieldId(NVML_FI_DEV_ECC_SBE_VOL_L1),
                FieldId(NVML_FI_DEV_ECC_DBE_VOL_L1),
                FieldId(NVML_FI_DEV_ECC_SBE_VOL_L2),
                FieldId(NVML_FI_DEV_ECC_DBE_VOL_L2),
                FieldId(NVML_FI_DEV_ECC_SBE_VOL_DEV),
                FieldId(NVML_FI_DEV_ECC_DBE_VOL_DEV),
                FieldId(NVML_FI_DEV_ECC_SBE_VOL_REG),
                FieldId(NVML_FI_DEV_ECC_DBE_VOL_REG),
                FieldId(NVML_FI_DEV_ECC_SBE_VOL_TEX),
                FieldId(NVML_FI_DEV_ECC_DBE_VOL_TEX),
                FieldId(NVML_FI_DEV_ECC_DBE_VOL_CBU),
                FieldId(NVML_FI_DEV_ECC_SBE_AGG_L1),
                FieldId(NVML_FI_DEV_ECC_DBE_AGG_L1),
                FieldId(NVML_FI_DEV_ECC_SBE_AGG_L2),
                FieldId(NVML_FI_DEV_ECC_DBE_AGG_L2),
                FieldId(NVML_FI_DEV_ECC_SBE_AGG_DEV),
                FieldId(NVML_FI_DEV_ECC_DBE_AGG_DEV),
                FieldId(NVML_FI_DEV_ECC_SBE_AGG_REG),
                FieldId(NVML_FI_DEV_ECC_DBE_AGG_REG),
                FieldId(NVML_FI_DEV_ECC_SBE_AGG_TEX),
                FieldId(NVML_FI_DEV_ECC_DBE_AGG_TEX),
                FieldId(NVML_FI_DEV_ECC_DBE_AGG_CBU),

                FieldId(NVML_FI_DEV_PERF_POLICY_POWER),
                FieldId(NVML_FI_DEV_PERF_POLICY_THERMAL),
                FieldId(NVML_FI_DEV_PERF_POLICY_SYNC_BOOST),
                FieldId(NVML_FI_DEV_PERF_POLICY_BOARD_LIMIT),
                FieldId(NVML_FI_DEV_PERF_POLICY_LOW_UTILIZATION),
                FieldId(NVML_FI_DEV_PERF_POLICY_RELIABILITY),
                FieldId(NVML_FI_DEV_PERF_POLICY_TOTAL_APP_CLOCKS),
                FieldId(NVML_FI_DEV_PERF_POLICY_TOTAL_BASE_CLOCKS),

                FieldId(NVML_FI_DEV_MEMORY_TEMP),
                FieldId(NVML_FI_DEV_TOTAL_ENERGY_CONSUMPTION)
            ])
        })
    }

    // Passing an empty slice should return an `InvalidArg` error
    #[should_panic(expected = "InvalidArg")]
    #[test]
    fn field_values_for_empty() {
        let nvml = nvml();
        test_with_device(3, &nvml, |device| {
            device.field_values_for(&[])
        })
    }

    // My machine does not support this call
    #[cfg(not(feature = "test-local"))]
    #[test]
    fn serial() {
        let nvml = nvml();
        test_with_device(3, &nvml, |device| device.serial())
    }

    // My machine does not support this call
    #[cfg(not(feature = "test-local"))]
    #[test]
    fn board_part_number() {
        let nvml = nvml();
        test_with_device(3, &nvml, |device| device.board_part_number())
    }

    #[test]
    fn current_throttle_reasons() {
        let nvml = nvml();
        test_with_device(3, &nvml, |device| device.current_throttle_reasons())
    }

    #[test]
    fn current_throttle_reasons_strict() {
        let nvml = nvml();
        test_with_device(3, &nvml, |device| device.current_throttle_reasons_strict())
    }

    #[test]
    fn supported_throttle_reasons() {
        let nvml = nvml();
        test_with_device(3, &nvml, |device| device.supported_throttle_reasons())
    }

    #[test]
    fn supported_throttle_reasons_strict() {
        let nvml = nvml();
        test_with_device(3, &nvml, |device| device.supported_throttle_reasons_strict())
    }

    #[test]
    fn supported_graphics_clocks() {
        let nvml = nvml();
        #[allow(unused_variables)]
        test_with_device(3, &nvml, |device| {
            let supported = device.supported_graphics_clocks(810)?;

            #[cfg(feature = "test-local")]
            {
                assert_eq!(
                    supported,
                    vec![1531, 1519, 1506, 1493, 1481, 1468, 1455,
                         1443, 1430, 1418, 1405, 1392, 1380, 1367,
                         1354, 1342, 1329, 1316, 1304, 1291, 1278,
                         1266, 1253, 1240, 1228, 1215, 1202, 1190,
                         1177, 1164, 1152, 1139, 1126, 1114, 1101,
                         1088, 1076, 1063, 1050, 1038, 1025, 1013,
                         1000, 988, 975, 963, 950, 938, 925, 913,
                         900, 888, 886, 873, 861, 848, 835, 823,
                         810, 797, 785, 772, 759, 747, 734, 721,
                         709, 696, 683, 671, 658, 645, 633, 620,
                         608, 595, 582, 570, 557, 544, 532, 519,
                         507, 494, 482, 469, 457, 444, 432, 419,
                         407, 405, 324, 270, 202, 162, 135]
                )
            }

            Ok(())
        })
    }

    #[test]
    fn supported_memory_clocks() {
        let nvml = nvml();
        #[allow(unused_variables)]
        test_with_device(3, &nvml, |device| {
            let supported = device.supported_memory_clocks()?;

            #[cfg(feature = "test-local")]
            #[cfg(target_os = "linux")]
            {
                assert_eq!(supported, vec![3505, 3304, 810, 405])
            }

            Ok(())
        })
    }

    #[test]
    fn temperature() {
        let nvml = nvml();
        test_with_device(
            3,
            &nvml,
            |device| device.temperature(TemperatureSensor::Gpu)
        )
    }

    #[test]
    fn temperature_threshold() {
        let nvml = nvml();
        test_with_device(3, &nvml, |device| {
            let slowdown = device
                .temperature_threshold(TemperatureThreshold::Slowdown)
                .chain_err(|| "slowdown")?;
            let shutdown = device
                .temperature_threshold(TemperatureThreshold::Shutdown)
                .chain_err(|| "shutdown")?;

            Ok((slowdown, shutdown))
        })
    }

    // I do not have 2 devices
    #[cfg(not(feature = "test-local"))]
    #[cfg(target_os = "linux")]
    #[test]
    fn topology_common_ancestor() {
        let nvml = nvml();
        let device1 = device(&nvml);
        let device2 = nvml.device_by_index(1).expect("device");

        device1.topology_common_ancestor(device2).expect("TopologyLevel");
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn topology_nearest_gpus() {
        let nvml = nvml();
        let device = device(&nvml);
        test(3, || device.topology_nearest_gpus(TopologyLevel::System))
    }

    // My machine does not support this call
    #[cfg(not(feature = "test-local"))]
    #[test]
    fn total_ecc_errors() {
        let nvml = nvml();
        test_with_device(3, &nvml, |device| {
            device.total_ecc_errors(MemoryError::Corrected, EccCounter::Volatile)
        })
    }

    #[test]
    fn uuid() {
        let nvml = nvml();
        test_with_device(3, &nvml, |device| device.uuid())
    }

    #[test]
    fn utilization_rates() {
        let nvml = nvml();
        test_with_device(3, &nvml, |device| device.utilization_rates())
    }

    #[test]
    fn vbios_version() {
        let nvml = nvml();
        test_with_device(3, &nvml, |device| device.vbios_version())
    }

    #[test]
    fn violation_status() {
        let nvml = nvml();
        test_with_device(3, &nvml, |device| {
            device.violation_status(PerformancePolicy::Power)
        })
    }

    // I do not have 2 devices
    #[cfg(not(feature = "test-local"))]
    #[test]
    fn is_on_same_board_as() {
        let nvml = nvml();
        let device1 = device(&nvml);
        let device2 = nvml.device_by_index(1).expect("device");

        device1.is_on_same_board_as(&device2).expect("bool");
    }

    // This modifies device state, so we don't want to actually run the test
    #[allow(dead_code)]
    fn reset_applications_clocks() {
        let nvml = nvml();
        let mut device = device(&nvml);

        device.reset_applications_clocks().expect("reset clocks")
    }

    // This modifies device state, so we don't want to actually run the test
    #[allow(dead_code)]
    fn set_auto_boosted_clocks() {
        let nvml = nvml();
        let mut device = device(&nvml);

        device.set_auto_boosted_clocks(true).expect("set to true")
    }

    // This modifies device state, so we don't want to actually run the test
    #[allow(dead_code)]
    #[cfg(target_os = "linux")]
    fn set_cpu_affinity() {
        let nvml = nvml();
        let mut device = device(&nvml);

        device.set_cpu_affinity().expect("ideal affinity set")
    }

    // This modifies device state, so we don't want to actually run the test
    #[allow(dead_code)]
    fn set_auto_boosted_clocks_default() {
        let nvml = nvml();
        let mut device = device(&nvml);

        device.set_auto_boosted_clocks_default(true).expect("set to true")
    }

    // My machine does not support this call
    #[cfg(not(feature = "test-local"))]
    #[test]
    fn validate_info_rom() {
        let nvml = nvml();
        test_with_device(3, &nvml, |device| device.validate_info_rom())
    }

    // This modifies device state, so we don't want to actually run the test
    #[allow(dead_code)]
    fn clear_accounting_pids() {
        let nvml = nvml();
        let mut device = device(&nvml);

        device.clear_accounting_pids().expect("cleared")
    }

    #[test]
    fn accounting_buffer_size() {
        let nvml = nvml();
        test_with_device(3, &nvml, |device| device.accounting_buffer_size())
    }

    #[test]
    fn is_accounting_enabled() {
        let nvml = nvml();
        test_with_device(3, &nvml, |device| device.is_accounting_enabled())
    }

    #[test]
    fn accounting_pids() {
        let nvml = nvml();
        test_with_device(3, &nvml, |device| device.accounting_pids())
    }

    #[should_panic(expected = "NotFound")]
    #[test]
    fn accounting_stats_for() {
        let nvml = nvml();
        test_with_device(3, &nvml, |device| {
            let processes = device.running_graphics_processes()?;

            // We never enable accounting mode, so this should return a `NotFound` error
            match device.accounting_stats_for(processes[0].pid) {
                Err(Error(ErrorKind::NotFound, _)) => panic!("NotFound"),
                other => other,
            }
        })
    }

    // This modifies device state, so we don't want to actually run the test
    #[allow(dead_code)]
    fn set_accounting() {
        let nvml = nvml();
        let mut device = device(&nvml);

        device.set_accounting(true).expect("set to true")
    }

    // This modifies device state, so we don't want to actually run the test
    #[allow(dead_code)]
    fn clear_ecc_error_counts() {
        let nvml = nvml();
        let mut device = device(&nvml);

        device.clear_ecc_error_counts(EccCounter::Aggregate).expect("set to true")
    }

    // This modifies device state, so we don't want to actually run the test
    #[allow(dead_code)]
    fn set_api_restricted() {
        let nvml = nvml();
        let mut device = device(&nvml);

        device.set_api_restricted(Api::ApplicationClocks, true).expect("set to true")
    }

    // This modifies device state, so we don't want to actually run the test
    #[allow(dead_code)]
    fn set_applications_clocks() {
        let nvml = nvml();
        let mut device = device(&nvml);

        device.set_applications_clocks(32, 32).expect("set to true")
    }

    // This modifies device state, so we don't want to actually run the test
    #[allow(dead_code)]
    fn set_compute_mode() {
        let nvml = nvml();
        let mut device = device(&nvml);

        device.set_compute_mode(ComputeMode::Default).expect("set to true")
    }

    // This modifies device state, so we don't want to actually run the test
    #[cfg(target_os = "windows")]
    #[allow(dead_code)]
    fn set_driver_model() {
        let nvml = nvml();
        let mut device = device(&nvml);

        device.set_driver_model(DriverModel::WDM, Behavior::DEFAULT).expect("set to wdm")
    }

    // This modifies device state, so we don't want to actually run the test
    #[allow(dead_code)]
    fn set_ecc() {
        let nvml = nvml();
        let mut device = device(&nvml);

        device.set_ecc(true).expect("set to true")
    }

    // This modifies device state, so we don't want to actually run the test
    #[allow(dead_code)]
    fn set_gpu_op_mode() {
        let nvml = nvml();
        let mut device = device(&nvml);

        device.set_gpu_op_mode(OperationMode::AllOn).expect("set to true")
    }

    // This modifies device state, so we don't want to actually run the test
    #[allow(dead_code)]
    #[cfg(target_os = "linux")]
    fn set_persistent() {
        let nvml = nvml();
        let mut device = device(&nvml);

        device.set_persistent(true).expect("set to true")
    }

    // This modifies device state, so we don't want to actually run the test
    #[allow(dead_code)]
    fn set_power_management_limit() {
        let nvml = nvml();
        let mut device = device(&nvml);

        device.set_power_management_limit(250000).expect("set to true")
    }

    #[cfg(target_os = "linux")]
    #[allow(unused_variables)]
    #[test]
    fn register_events() {
        let nvml = nvml();
        test_with_device(3, &nvml, |device| {
            let set = nvml.create_event_set()?;
            let set = device.register_events(
                EventTypes::PSTATE_CHANGE |
                EventTypes::CRITICAL_XID_ERROR |
                EventTypes::CLOCK_CHANGE,
                set
            )?;

            Ok(())
        })
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn supported_event_types() {
        let nvml = nvml();
        test_with_device(3, &nvml, |device| device.supported_event_types())
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn supported_event_types_strict() {
        let nvml = nvml();
        test_with_device(3, &nvml, |device| device.supported_event_types_strict())
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn is_drain_enabled() {
        let nvml = nvml();
        test_with_device(3, &nvml, |device| device.is_drain_enabled(None))
    }
}
