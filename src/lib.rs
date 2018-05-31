/*!
A complete, safe, and ergonomic Rust wrapper for the
[NVIDIA Management Library](https://developer.nvidia.com/nvidia-management-library-nvml)
(NVML), a C-based programmatic interface for monitoring and managing various states within
NVIDIA (primarily Tesla) GPUs.

```
# use nvml_wrapper::NVML;
# use nvml_wrapper::error::*;
# fn main() {
# test().unwrap();    
# }
# fn test() -> Result<()> {
let nvml = NVML::init()?;
// Get the first `Device` (GPU) in the system
let device = nvml.device_by_index(0)?;

let brand = device.brand()?; // GeForce on my system
let fan_speed = device.fan_speed()?; // Currently 17% on my system
let power_limit = device.enforced_power_limit()?; // 275k milliwatts on my system
let encoder_util = device.encoder_utilization()?; // Currently 0 on my system; Not encoding anything
let memory_info = device.memory_info()?; // Currently 1.63/6.37 GB used on my system

// ... and there's a whole lot more you can do. Everything in NVML is wrapped and ready to go
# Ok(())
# }
```

NVML is intended to be a platform for building 3rd-party applications, and is also the 
underlying library for NVIDIA's nvidia-smi tool.

It supports the following platforms:

* Windows
  * Windows Server 2008 R2 64-bit
  * Windows Server 2012 R2 64-bit
  * Windows 7 64-bit 
  * Windows 8 64-bit
  * Windows 10 64-bit
* Linux
  * 64-bit
  * 32-bit
* Hypervisors
  * Windows Server 2008R2/2012 Hyper-V 64-bit
  * Citrix XenServer 6.2 SP1+
  * VMware ESX 5.1/5.5

And the following products:

* Full Support
  * Tesla products Fermi architecture and up
  * Quadro products Fermi architecture and up
  * GRID products Kepler architecture and up
  * Select GeForce Titan products
* Limited Support
  * All GeForce products Fermi architecture and up

Although NVIDIA does not explicitly support it, most of the functionality offered
by NVML works on my dev machine (980 Ti). Even if your device is not on the list,
try it out and see what works:

```bash
cargo test
```

## Compilation

The NVML library comes with the NVIDIA drivers and is essentially present on any
system with a functioning NVIDIA graphics card. The compilation steps vary
between Windows and Linux, however.

### Windows

I have been able to successfully compile and run this wrapper's tests using
both the GNU and MSVC toolchains. An import library (`nvml.lib`) is included for
compilation with the MSVC toolchain.

The NVML library dll can be found at `%ProgramW6432%\NVIDIA Corporation\NVSMI\`
(which is `C:\Program Files\NVIDIA Corporation\NVSMI\` on my machine). I had to add
this folder to my `PATH` or place a copy of the dll in the same folder as the executable
in order to have everything work properly at runtime with the GNU toolchain. You may
need to do the same; I'm not sure if the MSVC toolchain needs this step or not.

### Linux

The NVML library can be found at `/usr/lib/nvidia-<driver-version>/libnvidia-ml.so`;
on my system with driver version 375.51 installed, this puts the library at
`/usr/lib/nvidia-375/libnvidia-ml.so`.

The `sys` crates' build script will automatically add the appropriate directory to
the paths searched for the library, so you shouldn't have to do anything manually
in theory.

## NVML Support

This wrapper is being developed against and currently supports NVML version
9.2. Each new version of NVML is guaranteed to be backwards-compatible according
to NVIDIA, so this wrapper should continue to work without issue regardless of
NVML version bumps.

## Rust Version Support

Currently supports Rust 1.26.0 or greater. The target version is the **latest**
stable version; I do not intend to pin to an older one at any time.

## Cargo Features

The `serde` feature can be toggled on in order to `#[derive(Serialize, Deserialize)]`
for every NVML data structure.
*/

#![cfg_attr(feature = "cargo-clippy", allow(doc_markdown))]
#![recursion_limit = "1024"]
#![allow(non_upper_case_globals)]

#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate wrapcenum_derive;
#[cfg(feature = "serde")]
#[macro_use]
extern crate serde;
extern crate nvml_wrapper_sys as ffi;
#[cfg(test)]
#[cfg_attr(test, macro_use)]
extern crate assert_matches;

pub mod device;
pub mod error;
pub mod unit;
pub mod structs;
pub mod struct_wrappers;
pub mod enums;
pub mod enum_wrappers;
pub mod event;
pub mod bitmasks;
pub mod nv_link;
pub mod high_level;
#[cfg(test)]
mod test_utils;

// Re-exports for convenience
pub use device::Device;
pub use event::EventSet;
pub use nv_link::NvLink;
pub use unit::Unit;

/// Re-exports from `nvml-wrapper-sys` that are necessary for use of this wrapper.
pub mod sys_exports {
    /// Use these constants to populate the `structs::device::FieldId` newtype.
    pub mod field_id {
        pub use ffi::bindings::field_id::*;
    }
}

#[cfg(target_os = "linux")]
use std::ptr;
use std::{
    ffi::{
        CStr,
        CString
    },
    io::{
        self,
        Write
    },
    mem,
    os::raw::{
        c_int,
        c_uint
    }
};

#[cfg(target_os = "linux")]
use enum_wrappers::device::TopologyLevel;

use error::{Result, nvml_try};
use ffi::bindings::*;

#[cfg(target_os = "linux")]
use struct_wrappers::device::PciInfo;
use struct_wrappers::unit::HwbcEntry;

use bitmasks::InitFlags;

/**
The main struct that this library revolves around.

According to NVIDIA's documentation, "It is the user's responsibility to call `nvmlInit()`
before calling any other methods, and `nvmlShutdown()` once NVML is no longer being used."
This struct is used to enforce those rules.

Also according to NVIDIA's documentation, "NVML is thread-safe so it is safe to make 
simultaneous NVML calls from multiple threads." In the Rust world, this translates to `NVML`
being `Send` + `Sync`. You can `.clone()` an `Arc` wrapped `NVML` and enjoy using it on any thread.

NOTE: If you care about possible errors returned from `nvmlShutdown()`, use the `.shutdown()`
method on this struct. **The `Drop` implementation ignores errors.**

When reading documentation on this struct and its members, remember that a lot of it, 
especially in regards to errors returned, is copied from NVIDIA's docs. While they can be found
online [here](http://docs.nvidia.com/deploy/nvml-api/index.html), the hosted docs are outdated and
do not accurately reflect the version of NVML that this library is written for; beware. You should
ideally read the doc comments on an up-to-date NVML API header. Such a header can be downloaded
as part of the [CUDA toolkit](https://developer.nvidia.com/cuda-downloads).
*/
#[derive(Debug)]
pub struct NVML;

// Here to clarify that NVML does have these traits. I know they are
// implemented without this.
unsafe impl Send for NVML {}
unsafe impl Sync for NVML {}

impl NVML {
    /**
    Handles NVML initialization and must be called before doing anything else.
    
    This static function can be called multiple times and multiple NVML structs can be
    used at the same time. NVIDIA's docs state that "A reference count of the number of 
    initializations is maintained. Shutdown only occurs when the reference count reaches 
    zero."
    
    In practice, there should be no need to create multiple `NVML` structs; wrap this struct
    in an `Arc` and go that route. 
    
    Note that this will initialize NVML but not any GPUs. This means that NVML can
    communicate with a GPU even when other GPUs in a system are bad or unstable.
    
    # Errors

    * `DriverNotLoaded`, if the NVIDIA driver is not running
    * `NoPermission`, if NVML does not have permission to talk to the driver
    * `Unknown`, on any unexpected error
    */
    // Checked against local
    #[inline]
    pub fn init() -> Result<Self> {
        unsafe {
            nvml_try(nvmlInit_v2())?;
        }

        Ok(NVML)
    }

    /**
    An initialization function that allows you to pass flags to control certain behaviors.

    This is the same as `init()` except for the addition of flags.

    # Errors

    * `DriverNotLoaded`, if the NVIDIA driver is not running
    * `NoPermission`, if NVML does not have permission to talk to the driver
    * `Unknown`, on any unexpected error

    # Examples

    ```
    # use nvml_wrapper::NVML;
    # use nvml_wrapper::error::*;
    use nvml_wrapper::bitmasks::InitFlags;

    # fn main() -> Result<()> {
    // Don't fail if the system doesn't have any NVIDIA GPUs
    NVML::init_with_flags(InitFlags::NO_GPUS)?;
    # Ok(())
    # }
    ```
    */
    // TODO: Example of using multiple flags when multiple flags exist
    #[inline]
    pub fn init_with_flags(flags: InitFlags) -> Result<Self> {
        unsafe {
            nvml_try(nvmlInitWithFlags(flags.bits()))?;
        }

        Ok(NVML)
    }

    /**
    Use this to shutdown NVML and release allocated resources if you care about handling
    potential errors (*the `Drop` implementation ignores errors!*).
    
    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `Unknown`, on any unexpected error
    */
    // Thanks to `sorear` on IRC for suggesting this approach
    // Checked against local
    // Tested
    #[inline]
    pub fn shutdown(self) -> Result<()> {
        unsafe {
            nvml_try(nvmlShutdown())?;
        }

        Ok(mem::forget(self))
    }

    /**
    Get the number of compute devices in the system (compute device == one GPU).
    
    Note that this may return devices you do not have permission to access.
    
    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `Unknown`, on any unexpected error
    */
    // Checked against local
    // Tested
    #[inline]
    pub fn device_count(&self) -> Result<u32> {
        unsafe {
            let mut count: c_uint = mem::zeroed();
            nvml_try(nvmlDeviceGetCount_v2(&mut count))?;

            Ok(count as u32)
        }
    }

    /**
    Gets the version of the system's graphics driver and returns it as an alphanumeric
    string. 
    
    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `Utf8Error`, if the string obtained from the C function is not valid Utf8
    */
    // Checked against local
    // Tested
    #[inline]
    pub fn sys_driver_version(&self) -> Result<String> {
        unsafe {
            let mut version_vec =
                Vec::with_capacity(NVML_SYSTEM_DRIVER_VERSION_BUFFER_SIZE as usize);

            nvml_try(nvmlSystemGetDriverVersion(
                version_vec.as_mut_ptr(),
                NVML_SYSTEM_DRIVER_VERSION_BUFFER_SIZE
            ))?;

            let version_raw = CStr::from_ptr(version_vec.as_ptr());
            Ok(version_raw.to_str()?.into())
        }
    }

    /**
    Gets the version of the system's NVML library and returns it as an alphanumeric
    string. 
    
    # Errors

    * `Utf8Error`, if the string obtained from the C function is not valid Utf8
    */
    // Checked against local
    // Tested
    #[inline]
    pub fn sys_nvml_version(&self) -> Result<String> {
        unsafe {
            let mut version_vec = Vec::with_capacity(NVML_SYSTEM_NVML_VERSION_BUFFER_SIZE as usize);

            nvml_try(nvmlSystemGetNVMLVersion(
                version_vec.as_mut_ptr(),
                NVML_SYSTEM_NVML_VERSION_BUFFER_SIZE
            ))?;

            // Thanks to `Amaranth` on IRC for help with this
            let version_raw = CStr::from_ptr(version_vec.as_ptr());
            Ok(version_raw.to_str()?.into())
        }
    }

    /// Gets the version of the system's CUDA driver.
    /// 
    /// The returned version is the same as what `cuDriverGetVersion()` from the
    /// CUDA API would return.
    #[inline]
    pub fn sys_cuda_driver_version(&self) -> Result<i32> {
        unsafe {
            let mut version: c_int = mem::zeroed();
            nvml_try(nvmlSystemGetCudaDriverVersion(&mut version))?;

            Ok(version)
        }
    }

    /**
    Gets the name of the process for the given process ID, cropped to the provided length.
    
    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `InvalidArg`, if the length is 0 (if this is returned without length being 0, file an issue)
    * `NotFound`, if the process does not exist
    * `NoPermission`, if the user doesn't have permission to perform the operation
    * `Utf8Error`, if the string obtained from the C function is not valid UTF-8. NVIDIA's docs say
    that the string encoding is ANSI, so this may very well happen. 
    * `Unknown`, on any unexpected error
    */
    // TODO: The docs say the string is ANSI-encoded. Not sure if I should try
    // to do anything about that
    // Checked against local
    // Tested
    #[inline]
    pub fn sys_process_name(&self, pid: u32, length: usize) -> Result<String> {
        unsafe {
            let mut name_vec = Vec::with_capacity(length);

            nvml_try(nvmlSystemGetProcessName(
                pid,
                name_vec.as_mut_ptr(),
                length as c_uint
            ))?;

            let name_raw = CStr::from_ptr(name_vec.as_ptr());
            Ok(name_raw.to_str()?.into())
        }
    }

    /**
    Acquire the handle for a particular device based on its index (starts at 0).
    
    Usage of this function causes NVML to initialize the target GPU. Additional
    GPUs may be initialized if the target GPU is an SLI slave. 
    
    You can determine valid indices by using `.device_count()`. This
    function doesn't call that for you, but the actual C function to get
    the device handle will return an error in the case of an invalid index.
    This means that the `InvalidArg` error will be returned if you pass in 
    an invalid index.
    
    NVIDIA's docs state that "The order in which NVML enumerates devices has 
    no guarantees of consistency between reboots. For that reason it is recommended 
    that devices be looked up by their PCI ids or UUID." In this library, that translates
    into usage of `.device_by_uuid()` and `.device_by_pci_bus_id()`.
    
    The NVML index may not correlate with other APIs such as the CUDA device index.
    
    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `InvalidArg`, if index is invalid
    * `InsufficientPower`, if any attached devices have improperly attached external power cables
    * `NoPermission`, if the user doesn't have permission to talk to this device
    * `IrqIssue`, if the NVIDIA kernel detected an interrupt issue with the attached GPUs
    * `GpuLost`, if the target GPU has fallen off the bus or is otherwise inaccessible
    * `Unknown`, on any unexpected error
    */
    // Checked against local
    // Tested
    #[inline]
    pub fn device_by_index(&self, index: u32) -> Result<Device> {
        unsafe {
            let mut device: nvmlDevice_t = mem::zeroed();
            nvml_try(nvmlDeviceGetHandleByIndex_v2(index, &mut device))?;

            Ok(device.into())
        }
    }

    /**
    Acquire the handle for a particular device based on its PCI bus ID.
    
    Usage of this function causes NVML to initialize the target GPU. Additional
    GPUs may be initialized if the target GPU is an SLI slave.
    
    The bus ID corresponds to the `bus_id` returned by `Device.pci_info()`.
    
    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `InvalidArg`, if `pci_bus_id` is invalid
    * `NotFound`, if `pci_bus_id` does not match a valid device on the system
    * `InsufficientPower`, if any attached devices have improperly attached external power cables
    * `NoPermission`, if the user doesn't have permission to talk to this device
    * `IrqIssue`, if the NVIDIA kernel detected an interrupt issue with the attached GPUs
    * `GpuLost`, if the target GPU has fallen off the bus or is otherwise inaccessible
    * `NulError`, for which you can read the docs on `std::ffi::NulError`
    * `Unknown`, on any unexpected error
    */
    // Checked against local
    // Tested
    #[inline]
    pub fn device_by_pci_bus_id<S: AsRef<str>>(&self, pci_bus_id: S) -> Result<Device>
    where
        Vec<u8>: From<S>,
    {
        unsafe {
            let c_string = CString::new(pci_bus_id)?;
            let mut device: nvmlDevice_t = mem::zeroed();

            nvml_try(nvmlDeviceGetHandleByPciBusId_v2(
                c_string.as_ptr(),
                &mut device
            ))?;

            Ok(device.into())
        }
    }

    /// Not documenting this because it's deprecated and does not seem to work
    /// anymore.
    // Tested (for an error)
    #[deprecated(note = "use `.device_by_uuid()`, this errors on dual GPU boards")]
    #[inline]
    pub fn device_by_serial<S: AsRef<str>>(&self, board_serial: S) -> Result<Device>
    where
        Vec<u8>: From<S>,
    {
        unsafe {
            let c_string = CString::new(board_serial)?;
            let mut device: nvmlDevice_t = mem::zeroed();

            nvml_try(nvmlDeviceGetHandleBySerial(c_string.as_ptr(), &mut device))?;

            Ok(device.into())
        }
    }

    /**
    Acquire the handle for a particular device based on its globally unique immutable
    UUID.
    
    Usage of this function causes NVML to initialize the target GPU. Additional
    GPUs may be initialized as the function called within searches for the target GPU.
    
    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `InvalidArg`, if `uuid` is invalid
    * `NotFound`, if `uuid` does not match a valid device on the system
    * `InsufficientPower`, if any attached devices have improperly attached external power cables
    * `IrqIssue`, if the NVIDIA kernel detected an interrupt issue with the attached GPUs
    * `GpuLost`, if the target GPU has fallen off the bus or is otherwise inaccessible
    * `NulError`, for which you can read the docs on `std::ffi::NulError`
    * `Unknown`, on any unexpected error
    
    NVIDIA doesn't mention `NoPermission` for this one. Strange!
    */
    // Checked against local
    // Tested
    #[inline]
    pub fn device_by_uuid<S: AsRef<str>>(&self, uuid: S) -> Result<Device>
    where
        Vec<u8>: From<S>,
    {
        unsafe {
            let c_string = CString::new(uuid)?;
            let mut device: nvmlDevice_t = mem::zeroed();

            nvml_try(nvmlDeviceGetHandleByUUID(c_string.as_ptr(), &mut device))?;

            Ok(device.into())
        }
    }

    /**
    Gets the common ancestor for two devices.
    
    Note: this is the same as `Device.topology_common_ancestor()`.
    
    # Errors

    * `InvalidArg`, if the device is invalid
    * `NotSupported`, if this `Device` or the OS does not support this feature
    * `UnexpectedVariant`, for which you can read the docs for
    * `Unknown`, on any unexpected error
    
    # Platform Support

    Only supports Linux.
    */
    // Checked against local
    // Tested
    #[cfg(target_os = "linux")]
    #[inline]
    pub fn topology_common_ancestor(
        &self,
        device1: &Device,
        device2: &Device,
    ) -> Result<TopologyLevel> {
        unsafe {
            let mut level: nvmlGpuTopologyLevel_t = mem::zeroed();

            nvml_try(nvmlDeviceGetTopologyCommonAncestor(
                device1.unsafe_raw(),
                device2.unsafe_raw(),
                &mut level
            ))?;

            Ok(TopologyLevel::try_from(level)?)
        }
    }

    /**
    Acquire the handle for a particular `Unit` based on its index.
    
    Valid indices are derived from the count returned by `.unit_count()`.
    For example, if `unit_count` is 2 the valid indices are 0 and 1, corresponding
    to UNIT 0 and UNIT 1.
    
    Note that the order in which NVML enumerates units has no guarantees of
    consistency between reboots.
    
    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `InvalidArg`, if `index` is invalid
    * `Unknown`, on any unexpected error
    
    # Device Support

    For S-class products.
    */
    // Checked against local
    // Tested (for an error)
    #[inline]
    pub fn unit_by_index(&self, index: u32) -> Result<Unit> {
        unsafe {
            let mut unit: nvmlUnit_t = mem::zeroed();
            nvml_try(nvmlUnitGetHandleByIndex(index as c_uint, &mut unit))?;

            Ok(unit.into())
        }
    }

    /**
    Checks if the passed-in `Device`s are on the same physical board.
    
    Note: this is the same as `Device.is_on_same_board_as()`.
    
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
    pub fn are_devices_on_same_board(&self, device1: &Device, device2: &Device) -> Result<bool> {
        unsafe {
            let mut bool_int: c_int = mem::zeroed();

            nvml_try(nvmlDeviceOnSameBoard(
                device1.unsafe_raw(),
                device2.unsafe_raw(),
                &mut bool_int
            ))?;

            match bool_int {
                0 => Ok(false),
                _ => Ok(true),
            }
        }
    }

    /**
    Gets the set of GPUs that have a CPU affinity with the given CPU number.
    
    # Errors

    * `InvalidArg`, if `cpu_number` is invalid
    * `NotSupported`, if this `Device` or the OS does not support this feature
    * `Unknown`, an error has occurred in the underlying topology discovery
    
    # Platform Support

    Only supports Linux.
    */
    // Tested
    #[cfg(target_os = "linux")]
    #[inline]
    pub fn topology_gpu_set(&self, cpu_number: u32) -> Result<Vec<Device>> {
        unsafe {
            let mut count = match self.topology_gpu_set_count(cpu_number)? {
                0 => return Ok(vec![]),
                value => value,
            };
            let mut devices: Vec<nvmlDevice_t> = vec![mem::zeroed(); count as usize];

            nvml_try(nvmlSystemGetTopologyGpuSet(
                cpu_number,
                &mut count,
                devices.as_mut_ptr()
            ))?;

            Ok(devices.into_iter().map(Device::from).collect())
        }
    }

    // Helper function for the above.
    #[cfg(target_os = "linux")]
    #[inline]
    fn topology_gpu_set_count(&self, cpu_number: u32) -> Result<c_uint> {
        unsafe {
            // Indicates that we want the count
            let mut count: c_uint = 0;

            // Passing null doesn't indicate that we want the count, just allowed
            nvml_try(nvmlSystemGetTopologyGpuSet(
                cpu_number,
                &mut count,
                ptr::null_mut()
            ))?;

            Ok(count)
        }
    }

    /**
    Gets the IDs and firmware versions for any Host Interface Cards in the system.

    # Errors

    * `Uninitialized`, if the library has not been successfully initialized

    # Device Support

    Supports S-class products.
    */
    // Checked against local
    // Tested
    #[inline]
    pub fn hic_versions(&self) -> Result<Vec<HwbcEntry>> {
        unsafe {
            let mut count: c_uint = match self.hic_count()? {
                0 => return Ok(vec![]),
                value => value,
            };
            let mut hics: Vec<nvmlHwbcEntry_t> = vec![mem::zeroed(); count as usize];

            nvml_try(nvmlSystemGetHicVersion(&mut count, hics.as_mut_ptr()))?;
            
            hics.into_iter().map(HwbcEntry::try_from).collect()
        }
    }

    /**
    Gets the count of Host Interface Cards in the system.

    # Errors

    * `Uninitialized`, if the library has not been successfully initialized

    # Device Support

    Supports S-class products.
    */
    // Tested as part of the above method
    #[inline]
    pub fn hic_count(&self) -> Result<u32> {
        unsafe {
            /*
            NVIDIA doesn't even say that `count` will be set to the count if
            `InsufficientSize` is returned. But we can assume sanity, right?
            
            The idea here is:
            If there are 0 HICs, NVML_SUCCESS is returned, `count` is set
              to 0. We return count, all good.
            If there is 1 HIC, NVML_SUCCESS is returned, `count` is set to
              1. We return count, all good.
            If there are >= 2 HICs, NVML_INSUFFICIENT_SIZE is returned.
             `count` is theoretically set to the actual count, and we
              return it.
            */
            let mut count: c_uint = 1;
            let mut hics: [nvmlHwbcEntry_t; 1] = [mem::zeroed()];

            match nvmlSystemGetHicVersion(&mut count, hics.as_mut_ptr()) {
                nvmlReturn_enum_NVML_SUCCESS |
                nvmlReturn_enum_NVML_ERROR_INSUFFICIENT_SIZE => Ok(count),
                // We know that this will be an error
                other => nvml_try(other).map(|_| 0),
            }
        }
    }

    /**
    Gets the number of units in the system.
    
    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `Unknown`, on any unexpected error
    
    # Device Support

    Supports S-class products.
    */
    // Checked against local
    // Tested
    #[inline]
    pub fn unit_count(&self) -> Result<u32> {
        unsafe {
            let mut count: c_uint = mem::zeroed();
            nvml_try(nvmlUnitGetCount(&mut count))?;

            Ok(count)
        }
    }

    /**
    Create an empty set of events.
    
    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `Unknown`, on any unexpected error
    
    # Device Support

    Supports Fermi and newer fully supported devices.
    */
    // Checked against local
    // Tested
    #[inline]
    pub fn create_event_set(&self) -> Result<EventSet> {
        unsafe {
            let mut set: nvmlEventSet_t = mem::zeroed();
            nvml_try(nvmlEventSetCreate(&mut set))?;

            Ok(set.into())
        }
    }

    /**
    Request the OS and the NVIDIA kernel driver to rediscover a portion of the PCI
    subsystem in search of GPUs that were previously removed.
    
    The portion of the PCI tree can be narrowed by specifying a domain, bus, and
    device in the passed-in `pci_info`. **If all of these fields are zeroes, the
    entire PCI tree will be searched.** Note that for long-running NVML processes,
    the enumeration of devices will change based on how many GPUs are discovered
    and where they are inserted in bus order.
    
    All newly discovered GPUs will be initialized and have their ECC scrubbed which
    may take several seconds per GPU. **All device handles are no longer guaranteed
    to be valid post discovery**. I am not sure if this means **all** device
    handles, literally, or if NVIDIA is referring to handles that had previously
    been obtained to devices that were then removed and have now been
    re-discovered.
    
    Must be run as administrator.
    
    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `OperatingSystem`, if the operating system is denying this feature
    * `NoPermission`, if the calling process has insufficient permissions to
    perform this operation
    * `NulError`, if an issue is encountered when trying to convert a Rust
    `String` into a `CString`.
    * `Unknown`, on any unexpected error
    
    # Device Support

    Supports Pascal and newer fully supported devices.
    
    Some Kepler devices are also supported (that's all NVIDIA says, no specifics).
    
    # Platform Support
    
    Only supports Linux.
    */
    // TODO: constructor for default pci_infos ^
    // Checked against local
    // Tested
    #[cfg(target_os = "linux")]
    #[inline]
    pub fn discover_gpus(&self, pci_info: PciInfo) -> Result<()> {
        unsafe { nvml_try(nvmlDeviceDiscoverGpus(&mut pci_info.try_into_c()?)) }
    }
}

/// This `Drop` implementation ignores errors! Use the `.shutdown()` method on
/// the `NVML` struct
/// if you care about handling them.
impl Drop for NVML {
    fn drop(&mut self) {
        #[allow(unused_must_use)]
        unsafe {
            match nvml_try(nvmlShutdown()) {
                Ok(()) => (),
                Err(e) => {
                    io::stderr().write(
                        format!(
                            "WARNING: Error returned by `nmvlShutdown()` in Drop implementation: \
                             {:?}",
                            e
                        ).as_bytes()
                    );
                },
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use bitmasks::InitFlags;
    use error::{Error, ErrorKind};
    use test_utils::*;

    #[test]
    fn nvml_is_send() {
        assert_send::<NVML>()
    }

    #[test]
    fn nvml_is_sync() {
        assert_sync::<NVML>()
    }

    #[test]
    fn init_with_flags() {
        NVML::init_with_flags(InitFlags::NO_GPUS).unwrap();
    }

    #[test]
    fn shutdown() {
        test(3, || nvml().shutdown())
    }

    #[test]
    fn device_count() {
        test(3, || nvml().device_count())
    }

    #[test]
    fn sys_driver_version() {
        test(3, || nvml().sys_driver_version())
    }

    #[test]
    fn sys_nvml_version() {
        test(3, || nvml().sys_nvml_version())
    }

    #[test]
    fn sys_cuda_driver_version() {
        test(3, || nvml().sys_cuda_driver_version())
    }

    #[test]
    fn sys_process_name() {
        let nvml = nvml();
        test_with_device(3, &nvml, |device| {
            let processes = device.running_graphics_processes()?;
            match nvml.sys_process_name(processes[0].pid, 64) {
                Err(Error(ErrorKind::NoPermission, _)) => Ok("No permission error".into()),
                v => v
            }
        })
    }

    #[test]
    fn device_by_index() {
        let nvml = nvml();
        test(3, || nvml.device_by_index(0))
    }

    #[test]
    fn device_by_pci_bus_id() {
        let nvml = nvml();
        test_with_device(3, &nvml, |device| {
            let id = device.pci_info()?.bus_id;
            nvml.device_by_pci_bus_id(id)
        })
    }

    // Can't get serial on my machine
    #[cfg(not(feature = "test-local"))]
    #[test]
    fn device_by_serial() {
        let nvml = nvml();

        #[allow(deprecated)]
        test_with_device(3, &nvml, |device| {
            let serial = device.serial()?;
            nvml.device_by_serial(serial)
        })
    }

    #[test]
    fn device_by_uuid() {
        let nvml = nvml();
        test_with_device(3, &nvml, |device| {
            let uuid = device.uuid()?;
            nvml.device_by_uuid(uuid)
        })
    }

    // I don't have 2 devices
    #[cfg(not(feature = "test-local"))]
    #[cfg(target_os = "linux")]
    #[test]
    fn topology_common_ancestor() {
        let nvml = nvml();
        let device1 = device(&nvml);
        let device2 = nvml.device_by_index(1).expect("device");

        nvml.topology_common_ancestor(&device1, &device2).expect("TopologyLevel");
    }

    // Errors on my machine
    #[cfg_attr(feature = "test-local", should_panic(expected = "InvalidArg"))]
    #[test]
    fn unit_by_index() {
        let nvml = nvml();
        test(3, || {
            match nvml.unit_by_index(0) {
                // I have no unit to test with
                Err(Error(ErrorKind::InvalidArg, _)) => panic!("InvalidArg"),
                other => other,
            }
        })
    }

    // I don't have 2 devices
    #[cfg(not(feature = "test-local"))]
    #[test]
    fn are_devices_on_same_board() {
        let nvml = nvml();
        let device1 = device(&nvml);
        let device2 = nvml.device_by_index(1).expect("device");

        nvml.are_devices_on_same_board(&device1, &device2).expect("bool");
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn topology_gpu_set() {
        let nvml = nvml();
        test(3, || nvml.topology_gpu_set(0))
    }

    #[test]
    fn hic_version() {
        let nvml = nvml();
        test(3, || nvml.hic_versions())
    }

    #[test]
    fn unit_count() {
        test(3, || nvml().unit_count())
    }

    #[test]
    fn create_event_set() {
        let nvml = nvml();
        test(3, || nvml.create_event_set())
    }

    #[cfg(target_os = "linux")]
    #[should_panic(expected = "NoPermission")]
    #[test]
    fn discover_gpus() {
        let nvml = nvml();
        test_with_device(3, &nvml, |device| {
            let pci_info = device.pci_info()?;

            // We don't test with admin perms and therefore expect an error
            match nvml.discover_gpus(pci_info) {
                Err(Error(ErrorKind::NoPermission, _)) => panic!("NoPermission"),
                other => other,
            }
        })
    }
}
