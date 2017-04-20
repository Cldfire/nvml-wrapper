//! Rust wrapper for the NVIDIA Management Library (NVML), a C-based programmatic interface 
//! for monitoring and managing various states within NVIDIA (primarily Tesla) GPUs. 
//!
//! It is intended to be a platform for building 3rd party applications, and is also the 
//! underlying library for NVIDIA's nvidia-smi tool.
//!
//! NVML supports the following platforms:
//!
//! * Windows
//!     * Windows Server 2008 R2 64-bit
//!     * Windows Server 2012 R2 64-bit
//!     * Windows 7 64-bit 
//!     * Windows 8 64-bit
//!     * Windows 10 64-bit
//! * Linux
//!     * 64-bit
//!     * 32-bit
//! * Hypervisors
//!     * Windows Server 2008R2/2012 Hyper-V 64-bit
//!     * Citrix XenServer 6.2 SP1+
//!     * VMware ESX 5.1/5.5
//!
//! And the following products:
//!
//! * Full Support
//!     * Tesla products Fermi architecture and up
//!     * Quadro products Fermi architecture and up
//!     * GRID products Kepler architecture and up
//!     * Select GeForce Titan products
//! * Limited Support
//!     * All GeForce products Fermi architecture and up

// TODO: Finish module docs. Say something about device support.
// TODO: Wrap device in arc as well for arc tests (to avoid sync / send issue)
// TODO: Change into_c to as_c ?

#![recursion_limit = "1024"]

#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate nvml_derive;
extern crate nvml_sys as ffi;

pub mod device;
pub mod error;
pub mod unit;
pub mod structs;
pub mod struct_wrappers;
pub mod enums;
pub mod enum_wrappers;
pub mod event;
pub mod bitmasks;
#[cfg(test)]
mod test_utils;

use error::*;
use ffi::bindings::*;
use device::Device;
use unit::Unit;
use event::EventSet;
use std::os::raw::{c_uint, c_int};
use std::ffi::{CStr, CString};
use std::mem;
use enum_wrappers::device::*;
use struct_wrappers::device::PciInfo;
use std::slice;
use std::io;
use std::io::Write;

/// The main struct that this library revolves around.
///
/// According to NVIDIA's documentation, "It is the user's responsibility to call `nvmlInit()`
/// before calling any other methods, and `nvmlShutdown()` once NVML is no longer being used."
/// This struct is used to enforce those rules.
///
/// Also according to NVIDIA's documentation, "NVML is thread-safe so it is safe to make 
/// simultaneous NVML calls from multiple threads." In the Rust world, this translates to `NVML`
/// being `Send` + `Sync`. You can `.clone()` an `Arc` wrapped `NVML` and enjoy using it on any thread.
/// 
/// NOTE: If you care about possible errors returned from `nvmlShutdown()`, use the `.shutdown()`
/// method on this struct. **The `Drop` implementation ignores errors.**
///
/// When reading documentation on this struct and its members, remember that a lot of it, 
/// especially in regards to errors returned, is copied from NVIDIA's docs. While they can be found
/// online here (http://docs.nvidia.com/deploy/nvml-api/index.html), the hosted docs are outdated and
/// do not accurately reflect the version of NVML that this library is written for. Beware.
pub struct NVML;

// Here to clarify that NVML does have these traits. I know they are implemented without this.
unsafe impl Send for NVML {}
unsafe impl Sync for NVML {}

impl NVML {
    /// Handles NVML initilization and must be called before doing anything else.
    ///
    /// This static function can be called multiple times and multiple NVML structs can be
    /// used at the same time. NVIDIA's docs state that "A reference count of the number of 
    /// initializations is maintained. Shutdown only occurs when the reference count reaches 
    /// zero."
    /// 
    /// Be careful calling this excessively from multiple threads, however; I observed during
    /// testing that calling `.init()` many times in parallel will not return an error from
    /// `.init()` but will cause a subsequent call to a function requiring that the library is
    /// initialized to fail (basically all of the methods on this struct). This is why tests
    /// must be run with `RUST_TEST_THREADS=1`.
    ///
    /// In practice, there should be no need to create multiple `NVML` structs; wrap this struct
    /// in an `Arc` and go that route. 
    ///
    /// Note that this will initialize NVML but not any GPUs. This means that NVML can
    /// communicate with a GPU even when other GPUs in a system are bad or unstable.
    ///
    /// # Errors
    /// * `DriverNotLoaded`, if the NVIDIA driver is not running
    /// * `NoPermission`, if NVML does not have permission to talk to the driver
    /// * `Unknown`, on any unexpected error
    // Checked against local
    #[inline]
    pub fn init() -> Result<Self> {
        unsafe {
            nvml_try(nvmlInit_v2())?;
        }

        Ok(NVML)
    }

    /// Use this to shutdown NVML and release allocated resources if you care about handling
    /// potential errors (*the `Drop` implementation ignores errors!*).
    ///
    /// # Errors
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `Unknown`, on any unexpected error
    // Thanks to `sorear` on IRC for suggesting this approach
    // Checked against local
    #[inline]
    pub fn shutdown(self) -> Result<()> {
        unsafe {
            nvml_try(nvmlShutdown())
        }
    }

    /// Get the number of compute devices in the system (compute device == one GPU).
    ///
    /// Note that this may return devices you do not have permission to access.
    ///
    /// # Errors
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `Unknown`, on any unexpected error
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

    /// Gets the version of the system's graphics driver and returns it as an alphanumeric
    /// string. 
    ///
    /// # Errors
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `Utf8Error`, if the string obtained from the C function is not valid Utf8
    // Checked against local
    // Tested
    #[inline]
    pub fn sys_driver_version(&self) -> Result<String> {
        unsafe {
            let mut version_vec = Vec::with_capacity(NVML_SYSTEM_DRIVER_VERSION_BUFFER_SIZE as usize);
            nvml_try(nvmlSystemGetDriverVersion(version_vec.as_mut_ptr(), NVML_SYSTEM_DRIVER_VERSION_BUFFER_SIZE))?;

            let version_raw = CStr::from_ptr(version_vec.as_ptr());
            Ok(version_raw.to_str()?.into())
        }
    }

    /// Gets the version of the system's NVML library and returns it as an alphanumeric
    /// string. 
    ///
    /// # Errors
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `Utf8Error`, if the string obtained from the C function is not valid Utf8
    // Checked against local
    // Tested
    #[inline]
    pub fn sys_nvml_version(&self) -> Result<String> {
        unsafe {
            let mut version_vec = Vec::with_capacity(NVML_SYSTEM_NVML_VERSION_BUFFER_SIZE as usize);
            nvml_try(nvmlSystemGetNVMLVersion(version_vec.as_mut_ptr(), NVML_SYSTEM_NVML_VERSION_BUFFER_SIZE))?;

            // Thanks to `Amaranth` on IRC for help with this
            let version_raw = CStr::from_ptr(version_vec.as_ptr());
            Ok(version_raw.to_str()?.into())
        }
    }

    /// Gets the name of the process for the given process ID, cropped to the provided length.
    ///
    /// # Errors
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `InvalidArg`, if the length is 0 (if this is returned without length being 0, file an issue)
    /// * `NotFound`, if the process does not exist
    /// * `NoPermission`, if the user doesn't have permission to perform the operation
    /// * `Utf8Error`, if the string obtained from the C function is not valid UTF-8. NVIDIA's docs say
    /// that the string encoding is ANSI, so this may very well happen. 
    /// * `Unknown`, on any unexpected error
    // TODO: The docs say the string is ANSI-encoded. Not sure if I should try to do anything about that
    // Checked against local
    // Tested
    #[inline]
    pub fn sys_process_name(&self, pid: u32, length: usize) -> Result<String> {
        unsafe {
            let mut name_vec = Vec::with_capacity(length);
            nvml_try(nvmlSystemGetProcessName(pid as c_uint, name_vec.as_mut_ptr(), length as c_uint))?;

            let name_raw = CStr::from_ptr(name_vec.as_ptr());
            Ok(name_raw.to_str()?.into())
        }
    }

    /// Acquire the handle for a particular device based on its index (starts at 0).
    ///
    /// Usage of this function causes NVML to initialize the target GPU. Additional
    /// GPUs may be initialized if the target GPU is an SLI slave. 
    ///
    /// You can determine valid indices by using `.get_device_count()`. This
    /// function doesn't call that for you, but the actual C function to get
    /// the device handle will return an error in the case of an invalid index.
    /// This means that the `InvalidArg` error will be returned if you pass in 
    /// an invalid index.
    ///
    /// NVIDIA's docs state that "The order in which NVML enumerates devices has 
    /// no guarantees of consistency between reboots. For that reason it is recommended 
    /// that devices be looked up by their PCI ids or UUID." In this library, that translates
    /// into usage of `.device_by_uuid()` and `.device_by_pci_bus_id()`.
    ///
    /// The NVML index may not correlate with other APIs such as the CUDA device index.
    ///
    /// # Errors
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `InvalidArg`, if index is invalid
    /// * `InsufficientPower`, if any attached devices have improperly attached external power cables
    /// * `NoPermission`, if the user doesn't have permission to talk to this device
    /// * `IrqIssue`, if the NVIDIA kernel detected an interrupt issue with the attached GPUs
    /// * `GpuLost`, if the target GPU has fallen off the bus or is otherwise inaccessible
    /// * `Unknown`, on any unexpected error
    // Checked against local
    // Tested
    #[inline]
    pub fn device_by_index(&self, index: u32) -> Result<Device> {
        unsafe {
            let mut device: nvmlDevice_t = mem::zeroed();
            nvml_try(nvmlDeviceGetHandleByIndex_v2(index as c_uint, &mut device))?;

            Ok(device.into())
        }
    }

    /// Acquire the handle for a particular device based on its PCI bus ID.
    ///
    /// Usage of this function causes NVML to initialize the target GPU. Additional
    /// GPUs may be initialized if the target GPU is an SLI slave.
    ///
    /// The bus ID corresponds to the `bus_id` returned by `Device.pci_info()`.
    ///
    /// # Errors
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `InvalidArg`, if `pci_bus_id` is invalid
    /// * `NotFound`, if `pci_bus_id` does not match a valid device on the system
    /// * `InsufficientPower`, if any attached devices have improperly attached external power cables
    /// * `NoPermission`, if the user doesn't have permission to talk to this device
    /// * `IrqIssue`, if the NVIDIA kernel detected an interrupt issue with the attached GPUs
    /// * `GpuLost`, if the target GPU has fallen off the bus or is otherwise inaccessible
    /// * `NulError`, for which you can read the docs on `std::ffi::NulError`
    /// * `Unknown`, on any unexpected error
    // Checked against local
    // Tested
    #[inline]
    pub fn device_by_pci_bus_id<S: AsRef<str>>(&self, pci_bus_id: S) -> Result<Device>
        where Vec<u8>: From<S> {
        unsafe {
            let c_string = CString::new(pci_bus_id)?;
            let mut device: nvmlDevice_t = mem::zeroed();
            nvml_try(nvmlDeviceGetHandleByPciBusId_v2(c_string.as_ptr(), &mut device))?;

            Ok(device.into())
        }
    }

    /// Not documenting this because it's deprecated and does not seem to work anymore.
    // Tested (for panic)
    #[deprecated(note = "use `.device_by_uuid()`, this errors on dual GPU boards")]
    #[inline]
    pub fn device_by_serial<S: AsRef<str>>(&self, board_serial: S) -> Result<Device>
        where Vec<u8>: From<S> {
        unsafe {
            let c_string = CString::new(board_serial)?;
            let mut device: nvmlDevice_t = mem::zeroed();
            nvml_try(nvmlDeviceGetHandleBySerial(c_string.as_ptr(), &mut device))?;

            Ok(device.into())
        }
    }

    /// Acquire the handle for a particular device based on its globally unique immutable
    /// UUID.
    ///
    /// Usage of this function causes NVML to initialize the target GPU. Additional
    /// GPUs may be initialized as the function called within searches for the target GPU.
    ///
    /// # Errors
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `InvalidArg`, if `uuid` is invalid
    /// * `NotFound`, if `uuid` does not match a valid device on the system
    /// * `InsufficientPower`, if any attached devices have improperly attached external power cables
    /// * `IrqIssue`, if the NVIDIA kernel detected an interrupt issue with the attached GPUs
    /// * `GpuLost`, if the target GPU has fallen off the bus or is otherwise inaccessible
    /// * `NulError`, for which you can read the docs on `std::ffi::NulError`
    /// * `Unknown`, on any unexpected error
    ///
    /// NVIDIA doesn't mention `NoPermission` for this one. Strange!
    // Checked against local
    // Tested
    #[inline]
    pub fn device_by_uuid<S: AsRef<str>>(&self, uuid: S) -> Result<Device> 
        where Vec<u8>: From<S> {
        unsafe {
            let c_string = CString::new(uuid)?;
            let mut device: nvmlDevice_t = mem::zeroed();
            nvml_try(nvmlDeviceGetHandleByUUID(c_string.as_ptr(), &mut device))?;

            Ok(device.into())
        }
    }

    /// Gets the common ancestor for two devices.
    ///
    /// Note: this is the same as `Device.topology_common_ancestor()`.
    ///
    /// # Errors
    /// * `InvalidArg`, if the device is invalid or `threshold_type` is invalid (shouldn't occur?)
    /// * `NotSupported`, if this `Device` or the OS does not support this feature
    /// * `Unknown`, on any unexpected error
    ///
    /// # Platform Support
    /// Only supports Linux.
    // TODO: Investigate this and the method on device more
    // Checked against local
    #[cfg(target_os = "linux")]
    #[inline]
    pub fn topology_common_ancestor(&self, device1: &Device, device2: &Device) -> Result<TopologyLevel> {
        unsafe {
            let mut level: nvmlGpuTopologyLevel_t = mem::zeroed();
            nvml_try(nvmlDeviceGetTopologyCommonAncestor(device1.c_device(), device2.c_device(), &mut level))?;

            Ok(level.into())
        }
    }

    /// Gets the set of GPUs that are nearest to the passed-in `Device` at a specific 
    /// interconnectivity level.
    ///
    /// Note: this is the same as `Device.topology_nearest_gpus()`.
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
    pub fn topology_nearest_gpus(&self, device: &Device, level: TopologyLevel) -> Result<Vec<Device>> {
        unsafe {
            let mut first_item: nvmlDevice_t = mem::zeroed();
            // TODO: Fails if I pass 0? What?
            let mut count: c_uint = 0;
            nvml_try(nvmlDeviceGetTopologyNearestGpus(device.c_device(), 
                                                      level.into_c(), 
                                                      &mut count, 
                                                      &mut first_item))?;
            
            // TODO: Again I believe I'm doing every single one of these wrong. The array has
            // already been malloc'd on the C side according to NVIDIA, meaning I'm probably
            // responsible for freeing the memory or something? Which I'm not doing here?
            // Investigate?
            //
            // Also other topo method below
            Ok(slice::from_raw_parts(&first_item as *const nvmlDevice_t, 
                                     count as usize)
                                     .iter()
                                     .map(|d| Device::from(*d))
                                     .collect())
        }
    }

    /// Acquire the handle for a particular `Unit` based on its index.
    ///
    /// Valid indices are derived from the count returned by `.unit_count()`.
    /// For example, if `unit_count` is 2 the valid indices are 0 and 1, corresponding
    /// to UNIT 0 and UNIT 1.
    ///
    /// Note that the order in which NVML enumerates units has no guarantees of
    /// consistency between reboots.
    ///
    /// # Errors
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `InvalidArg`, if `index` is invalid
    /// * `Unknown`, on any unexpected error
    ///
    /// # Device Support
    /// For S-class products.
    // Checked against local
    // Tested (for a panic)
    #[inline]
    pub fn unit_by_index(&self, index: u32) -> Result<Unit> {
        unsafe {
            let mut unit: nvmlUnit_t = mem::zeroed();
            nvml_try(nvmlUnitGetHandleByIndex(index as c_uint, &mut unit))?;

            Ok(unit.into())
        }
    }

    /// Checks if the passed-in `Device`s are on the same physical board.
    ///
    /// Note: this is the same as `Device.is_on_same_board_as()`.
    ///
    /// # Errors
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `InvalidArg`, if either `Device` is invalid
    /// * `NotSupported`, if this check is not supported by this `Device`
    /// * `GpuLost`, if this `Device` has fallen off the bus or is otherwise inaccessible
    /// * `Unknown`, on any unexpected error
    // Checked against local
    #[inline]
    pub fn is_device_on_same_board_as(device1: &Device, device2: &Device) -> Result<bool> {
        unsafe {
            let mut bool_int: c_int = mem::zeroed();
            nvml_try(nvmlDeviceOnSameBoard(device1.c_device(), device2.c_device(), &mut bool_int))?;

            match bool_int {
                0 => Ok(false),
                _ => Ok(true),
            }
        }
    }

    /// Gets the set of GPUs that have a CPU affinity with the given CPU number.
    ///
    /// # Errors
    /// * `InvalidArg`, if `cpu_number` is invalid
    /// * `NotSupported`, if this `Device` or the OS does not support this feature
    /// * `Unknown`, an error has occured in the underlying topology discovery
    ///
    /// # Platform Support
    /// Only supports Linux.
    #[cfg(target_os = "linux")]
    #[inline]
    pub fn topology_gpu_set(&self, cpu_number: u32) -> Result<Vec<Device>> {
        unsafe {
            let mut first_item: nvmlDevice_t = mem::zeroed();
            let mut count: c_uint = 0;
            nvml_try(nvmlSystemGetTopologyGpuSet(cpu_number as c_uint, &mut count, &mut first_item))?;

            // TODO:
            Ok(slice::from_raw_parts(&first_item as *const nvmlDevice_t, 
                                     count as usize)
                                     .iter()
                                     .map(|d| Device::from(*d))
                                     .collect())
        }
    }

    // TODO: NVIDIA doesn't explain this very well...
    // pub fn hic_version(&self) ->

    /// Gets the number of units in the system.
    ///
    /// # Errors
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `Unknown`, on any unexpected error
    ///
    /// # Device Support
    /// Supports S-class products.
    // Checked against local
    // Tested
    #[inline]
    pub fn unit_count(&self) -> Result<u32> {
        unsafe {
            let mut count: c_uint = mem::zeroed();
            nvml_try(nvmlUnitGetCount(&mut count))?;

            Ok(count as u32)
        }
    }

    /// Create an empty set of events.
    ///
    /// # Errors
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `Unknown`, on any unexpected error
    ///
    /// # Device Support
    /// Supports Fermi and newer fully supported devices.
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

    /// Request the OS and the NVIDIA kernel driver to rediscover a portion of the PCI
    /// subsystem in search of GPUs that were previously removed.
    ///
    /// The portion of the PCI tree can be narrowed by specifying a domain, bus, and
    /// device in the passed-in `pci_info`. **If all of these fields are zeroes, the
    // TODO: constructor for default ^
    /// entire PCI tree will be searched.** Note that for long-running NVML processes,
    /// the enumeration of devices will change based on how many GPUs are discovered
    /// and where they are inserted in bus order.
    ///
    /// All newly discovered GPUs will be initialized and have their ECC scrubbed which
    /// may take several seconds per GPU. **All device handles are no longer guaranteed
    /// to be valid post discovery**. I am not sure if this means **all** device
    /// handles, literally, or if NVIDIA is referring to handles that had previously
    /// been obtained to devices that were then removed and have now been
    /// re-discovered.
    ///
    /// Must be run as administrator.
    ///
    /// # Errors
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `OperatingSystem`, if the operating system is denying this feature
    /// * `NoPermission`, if the calling process has insufficient permissions to
    /// perform this operation
    /// * `NulError`, if an issue is encountered when trying to convert a Rust
    /// `String` into a `CString`.
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
    pub fn discover_gpus(&self, pci_info: PciInfo) -> Result<()> {
        unsafe {
            nvml_try(nvmlDeviceDiscoverGpus(&mut pci_info.try_into_c()?))
        }
    }
}

/// This `Drop` implementation ignores errors! Use the `.shutdown()` method on the `NVML` struct
/// if you care about handling them. 
impl Drop for NVML {
    fn drop(&mut self) {
        unsafe {
            match nvml_try(nvmlShutdown()) {
                Ok(()) => (),
                Err(e) => {
                    io::stderr().write(&format!("WARNING: Error returned by \
                        `nmvlShutdown()` in Drop implementation: {:?}", e).as_bytes())
                        .expect("could not write to stderr");
                }
            }
        }
    }
}

#[cfg(test)]
#[allow(unused_variables, unused_imports)]
mod test {
    use super::*;
    use test_utils::*;
    use std::thread;
    use std::sync::Arc;

    #[test]
    fn device_count() {
        test(3, || {
            nvml().device_count()
        })
    }

    #[test]
    fn sys_driver_version() {
        test(3, || {
            nvml().sys_driver_version()
        })
    }

    #[test]
    fn sys_nvml_version() {
        test(3, || {
            nvml().sys_nvml_version()
        })
    }

    #[test]
    fn sys_process_name() {
        let nvml = nvml();
        test_with_device(3, &nvml, |device| {
            let processes = device.running_graphics_processes(64)?;
            nvml.sys_process_name(processes[0].pid, 64)
        })
    }

    #[test]
    fn device_by_index() {
        let nvml = nvml();
        test(3, || {
            nvml.device_by_index(0)
        })
    }

    #[test]
    fn device_by_pci_bus_id() {
        let nvml = nvml();
        test_with_device(3, &nvml, |device| {
            let id = device.pci_info()?.bus_id;
            nvml.device_by_pci_bus_id(id)
        })
    }

    // Panics on my machine
    #[cfg_attr(feature = "test-local", should_panic)]
    #[test]
    fn device_by_serial() {
        let nvml = nvml();
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

    #[cfg_attr(feature = "test-local", should_panic)]
    #[test]
    fn unit_by_index() {
        let nvml = nvml();
        test(3, || {
            nvml.unit_by_index(0)
        })
    }

    #[test]
    fn unit_count() {
        test(3, || {
            nvml().unit_count()
        })
    }

    #[test]
    fn create_event_set() {
        let nvml = nvml();
        test(3, || {
            nvml.create_event_set()
        })
    }
}
