// `error_chain` recursion limit
#![recursion_limit = "1024"]

#[macro_use]
extern crate error_chain;
extern crate nvml_sys as ffi;

// TODO: Module docs. Say something about device support.

pub mod device;
pub mod structs;
pub mod struct_wrappers;
pub mod enums;
pub mod enum_wrappers;
pub mod errors;

use errors::*;
use ffi::*;
use device::Device;
use std::os::raw::{c_uint};
use std::ffi::{CStr};
use std::mem;

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
/// method on this struct. _The `Drop` implementation ignores errors._
///
/// When reading documentation on this struct and its members, remember that a lot of it, 
/// especially in regards to errors returned, is copied from NVIDIA's docs. While they can be found
/// online here (http://docs.nvidia.com/deploy/nvml-api/index.html), the hosted docs are outdated and
/// do not accurately reflect the version of NVML that this library is written for. Beware.
pub struct NVML;
// Here to clarify without a doubt that NVML does have these traits
// TODO: Do I even need to do this?
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
    /// Note that this will initialize NVML but not any GPUs.
    ///
    /// # Errors
    /// * `DriverNotLoaded`, if the NVIDIA driver is not running
    /// * `NoPermission`, if NVML does not have permission to talk to the driver
    /// * `Unknown`, on any unexpected error
    pub fn init() -> Result<Self> {
        unsafe {
            // TODO: nvmlInit is #DEFINEd to be nvmlInit_v2, can that be replicated in Rust?
            nvml_try(nvmlInit_v2())?;
        }

        Ok(NVML)
    }

    /// Use this to shutdown NVML and release allocated resources if you care about handling
    /// potential errors (*the `Drop` implementation ignores errors!*).
    ///struct_wrappestruct_wrapperssstruct_wstruct_wrappersappers
    /// # Errors
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `Unknown`, on any unexpected error
    // Thanks to `sorear` on IRC for suggesting this approach
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
    ///
    /// If `InvalidArg` ever gets returned from this function, that is a bug.
    /// Please file an issue with any relevant information. 
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
    ///
    /// If either of `InvalidArg` or `InsufficientSize` ever get returned from this function,
    /// that is a bug. Please file an issue with any relevant information. 
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
    ///
    /// If either of `InvalidArg` or `InsufficientSize` ever get returned from this function,
    /// that is a bug. Please file an issue with any relevant information. 
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
    /// * `Utf8Error`, if the string obtained from the C function is not valid Utf8. NVIDIA's docs say
    /// that the string encoding is ANSI, so this may very well happen. 
    /// * `Unknown`, on any unexpected error
    // TODO: The docs say the string is ANSI-encoded. Not sure if I should try to do anything about that
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
    /// Keep that in mind. 
    ///
    /// # Errors
    /// * `Uninitialized`, if the library has not been successfully initialized
    /// * `InvalidArg`, if index is invalid
    /// * `InsufficientPower`, if any attached devices have improperly attached external power cables
    /// * `NoPermission`, if the user doesn't have permission to talk to this device
    /// * `IrqIssue`, if the NVIDIA kernel detected an interrupt issue with the attached GPUs
    /// * `GpuLost`, if the target GPU has fallen off the bus or is otherwise inaccessible
    /// * `Unknown`, on any unexpected error
    pub fn device_by_index(&self, index: u32) -> Result<Device> {
        unsafe {
            let mut device: nvmlDevice_t = mem::zeroed();
            nvml_try(nvmlDeviceGetHandleByIndex_v2(index as c_uint, &mut device))?;

            Ok(Device::_new(device))
        }
    }

    // pub fn device_by_pci_bus_id(&self, )
}

/// This `Drop` implementation ignores errors! Use the `.shutdown()` method on the `NVML` struct
/// if you care about handling them. 
impl Drop for NVML {
    fn drop(&mut self) {
        unsafe {
            match nvml_try(nvmlShutdown()) {
                Ok(()) => (),
                Err(e) => {
                    println!("WARNING: Error returned by `nmvlShutdown()` in Drop implementation: {:?}", e);
                    panic!("Error returned by `nmvlShutdown()` in Drop implementation: {:?}", e);
                }
            }
        }
    }
}

#[cfg(feature = "test")]
#[allow(unused_variables, unused_imports)]
mod test {
    use super::*;
    use std::thread;
    use std::sync::Arc;

    #[test]
    fn init_drop() {
        let test = NVML::init().expect("init call failed");
    }

    #[test]
    fn init_shutdown() {
        let test = NVML::init().expect("init call failed");
        test.shutdown().expect("shutdown failed");
    }

    #[test]
    fn init_drop_multiple() {
        let test1 = NVML::init().expect("init call1 failed");
        let test2 = NVML::init().expect("init call2 failed");
        let test3 = NVML::init().expect("init call3 failed");
    }

    #[test]
    fn init_shutdown_multiple() {
        let test1 = NVML::init().expect("init call1 failed");
        let test2 = NVML::init().expect("init call2 failed");
        let test3 = NVML::init().expect("init call3 failed");

        test1.shutdown().expect("shutdown1 failed");
        test2.shutdown().expect("shutdown2 failed");
        test3.shutdown().expect("shutdown3 failed");
    }

    #[test]
    fn init_drop_multiple_threads() {
        let handle1 = thread::spawn(|| {
            let test = NVML::init().expect("init call1 failed");
        });

        let handle2 = thread::spawn(|| {
            let test = NVML::init().expect("init call2 failed");
        });

        let handle3 = thread::spawn(|| {
            let test = NVML::init().expect("init call3 failed");
        });
        
        let res1 = handle1.join().expect("handle1 join failed");
        let res2 = handle2.join().expect("handle2 join failed");
        let res3 = handle3.join().expect("handle3 join failed");
    }

    #[test]
    fn init_shutdown_multiple_threads() {
        let handle1 = thread::spawn(|| {
            let test = NVML::init().expect("init call1 failed");
            test.shutdown().expect("shutdown1 failed");
        });

        let handle2 = thread::spawn(|| {
            let test = NVML::init().expect("init call2 failed");
            test.shutdown().expect("shutdown2 failed");
        });

        let handle3 = thread::spawn(|| {
            let test = NVML::init().expect("init call3 failed");
            test.shutdown().expect("shutdown3 failed");
        });
        
        let res1 = handle1.join().expect("handle1 join failed");
        let res2 = handle2.join().expect("handle2 join failed");
        let res3 = handle3.join().expect("handle3 join failed");
    }

    #[test]
    fn device_count() {
        let test = NVML::init().expect("init call failed");
        let count = test.device_count().expect("Could not get device count");

        #[cfg(feature = "test-local")]
        {
            assert_eq!(count, 1);
        }
    }

    #[test]
    fn device_count_multiple() {
        let test1 = NVML::init().expect("init call1 failed");
        let test2 = NVML::init().expect("init call2 failed");
        let test3 = NVML::init().expect("init call3 failed");

        let count1 = test1.device_count().expect("Could not get device count1");
        let count2 = test2.device_count().expect("Could not get device count2");
        let count3 = test3.device_count().expect("Could not get device count3");

        #[cfg(feature = "test-local")]
        {
            assert_eq!(count1, 1);
            assert_eq!(count2, 1);
            assert_eq!(count3, 1);
        }
    }

    #[test]
    fn device_count_multiple_threads() {
        let handle1 = thread::spawn(|| {
            let test = NVML::init().expect("init call1 failed");
            let count = test.device_count().expect("Could not get device count");

            #[cfg(feature = "test-local")]
            {
                assert_eq!(count, 1);
            }
        });

        let handle2 = thread::spawn(|| {
            let test = NVML::init().expect("init call2 failed");
            let count = test.device_count().expect("Could not get device count");

            #[cfg(feature = "test-local")]
            {
                assert_eq!(count, 1);
            }
        });

        let handle3 = thread::spawn(|| {
            let test = NVML::init().expect("init call3 failed");
            let count = test.device_count().expect("Could not get device count");

            #[cfg(feature = "test-local")]
            {
                assert_eq!(count, 1);
            }
        });

        let res1 = handle1.join().expect("handle1 join failed");
        let res2 = handle2.join().expect("handle2 join failed");
        let res3 = handle3.join().expect("handle3 join failed");
    }

    #[test]
    fn device_count_multiple_threads_reference() {
        let test = Arc::new(NVML::init().expect("init call failed"));
        let ref_1 = test.clone();
        let ref_2 = test.clone();
        let ref_3 = test.clone();
        
        let handle1 = thread::spawn(move || {
            let count = ref_1.device_count().expect("Could not get device count1");

            #[cfg(feature = "test-local")]
            {
                assert_eq!(count, 1);
            }
        });

        let handle2 = thread::spawn(move || {
            let count = ref_2.device_count().expect("Could not get device count2");

            #[cfg(feature = "test-local")]
            {
                assert_eq!(count, 1);
            }
        });

        let handle3 = thread::spawn(move || {
            let count = ref_3.device_count().expect("Could not get device count3");

            #[cfg(feature = "test-local")]
            {
                assert_eq!(count, 1);
            }
        });

        let res1 = handle1.join().expect("handle1 join failed");
        let res2 = handle2.join().expect("handle2 join failed");
        let res3 = handle3.join().expect("handle3 join failed");
    }

    // TODO: Gen tests for driver version
    #[test]
    fn driver_version() {
        let test = NVML::init().expect("init call failed");
        let version = test.sys_driver_version().expect("Could not get driver version");
    }

    // TODO: Gen tests for nvml version
    #[test]
    fn nvml_version() {
        let test = NVML::init().expect("init call failed");
        let version = test.sys_nvml_version().expect("Could not get NVML version");
    }

    // TODO: Gen tests for process_name
    #[cfg(feature = "test-local")]
    #[test]
    fn process_name() {
        let test = NVML::init().expect("init call failed");
        // TODO: This is stupid
        let name = test.sys_process_name(25121, 80).expect("Could not get name for PID");
    }

    // TODO: This test and others below are specific to a machine with a GPU
    // TODO: Why is this cfg thing not working?!?!?!??!?
    #[test]
    #[cfg_attr(not(feature = "test-local"), should_panic)]
    fn device_by_index() {
        let test = NVML::init().expect("init call failed");
        let device = test.device_by_index(0).expect("Could not get a device by index 0");
    }

    #[test]
    #[cfg_attr(not(feature = "test-local"), should_panic)]
    fn device_by_index_multiple() {
        let test1 = NVML::init().expect("init call1 failed");
        let test2 = NVML::init().expect("init call2 failed");
        let test3 = NVML::init().expect("init call3 failed");

        let device1 = test1.device_by_index(0).expect("Could not get device1 by index 0");
        let device2 = test2.device_by_index(0).expect("Could not get device2 by index 0");
        let device3 = test3.device_by_index(0).expect("Could not get device3 by index 0");
    }

    #[cfg_attr(not(feature = "test-local"), should_panic)]
    #[test]
    fn device_by_index_multiple_threads() {
        let handle1 = thread::spawn(|| {
            let test = NVML::init().expect("init call1 failed");
            let device = test.device_by_index(0).expect("Could not get device1 by index 0");
        });

        let handle2 = thread::spawn(|| {
            let test = NVML::init().expect("init call2 failed");
            let device = test.device_by_index(0).expect("Could not get device2 by index 0");
        });

        let handle3 = thread::spawn(|| {
            let test = NVML::init().expect("init call3 failed");
            let device = test.device_by_index(0).expect("Could not get device3 by index 0");
        });

        let res1 = handle1.join().expect("handle1 join failed");
        let res2 = handle2.join().expect("handle2 join failed");
        let res3 = handle3.join().expect("handle3 join failed");
    }

    #[cfg_attr(not(feature = "test-local"), should_panic)]
    #[test]
    fn device_by_index_multiple_threads_reference() {
        let test = Arc::new(NVML::init().expect("init call failed"));
        let ref_1 = test.clone();
        let ref_2 = test.clone();
        let ref_3 = test.clone();
        
        let handle1 = thread::spawn(move || {
            let device = ref_1.device_by_index(0).expect("Could not get device1 by index 0");
        });

        let handle2 = thread::spawn(move || {
            let device = ref_2.device_by_index(0).expect("Could not get device2 by index 0");
        });

        let handle3 = thread::spawn(move || {
            let device = ref_3.device_by_index(0).expect("Could not get device3 by index 0");
        });

        let res1 = handle1.join().expect("handle1 join failed");
        let res2 = handle2.join().expect("handle2 join failed");
        let res3 = handle3.join().expect("handle3 join failed");
    }
}
