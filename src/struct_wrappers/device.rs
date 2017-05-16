use ffi::bindings::*;
use error::*;
use enum_wrappers::device::*;
use enums::device::*;
use std::os::raw::{c_uint, c_char};
use std::ffi::{CStr, CString};
use std::u32;

/// PCI information about a GPU device.
// Checked against local
// Tested
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PciInfo {
    /// The bus on which the device resides, 0 to 0xff.
    pub bus: u32,
    /// The PCI identifier.
    pub bus_id: String,
    /// The device's ID on the bus, 0 to 31.
    pub device: u32,
    /// The PCI domain on which the device's bus resides, 0 to 0xffff. 
    pub domain: u32,
    /// The combined 16-bit device ID and 16-bit vendor ID.
    pub pci_device_id: u32,
    /// The 32-bit Sub System Device ID.
    pub pci_sub_system_id: u32,
}

impl PciInfo {
    /**
    Waiting for `TryFrom` to be stable. In the meantime, we do this.

    # Errors
    * `Utf8Error`, if the string obtained from the C function is not valid Utf8
    */
    pub fn try_from(struct_: nvmlPciInfo_t) -> Result<Self> {
        unsafe {
            let bus_id_raw = CStr::from_ptr(struct_.busId.as_ptr());
            Ok(PciInfo {
                bus: struct_.bus as u32,
                bus_id: bus_id_raw.to_str()?.into(),
                device: struct_.device as u32,
                domain: struct_.domain as u32,
                pci_device_id: struct_.pciDeviceId as u32,
                pci_sub_system_id: struct_.pciSubSystemId as u32,
            })
        }
    }

    /**
    Waiting for `TryInto` to be stable. In the meantime, we do this.

    # Errors
    * `NulError`, if a nul byte was found in the bus_id (shouldn't occur?)
    * `StringTooLong`, if `bus_id.len()` exceeded the length of
    `NVML_DEVICE_INFOROM_VERSION_BUFFER_SIZE`. This should (?) only be able to
    occur if the user modifies `bus_id` in some fashion. We return an error
    rather than panicking.
    */
    // Tested
    pub fn try_into_c(self) -> Result<nvmlPciInfo_t> {
        use NVML_DEVICE_PCI_BUS_ID_BUFFER_SIZE as _buf_size;

        // This is more readable than spraying `buf_size as usize` everywhere
        fn buf_size() -> usize {
            _buf_size as usize
        }

        // ...but const fn though.
        let mut bus_id_c: [c_char; _buf_size as usize] = [0; _buf_size as usize];
        let mut bus_id = CString::new(self.bus_id)?.into_bytes_with_nul();

        if bus_id.len() > buf_size() {
            bail!(ErrorKind::StringTooLong(buf_size(), bus_id.len()))
        } else if bus_id.len() < buf_size() {
            while bus_id.len() != buf_size() {
                bus_id.push(0);
            }
        };

        bus_id_c.clone_from_slice(&bus_id.iter()
                                         .map(|b| *b as i8)
                                         .collect::<Vec<_>>());

        Ok(nvmlPciInfo_t {
            busId: bus_id_c,
            domain: self.domain as c_uint,
            bus: self.bus as c_uint,
            device: self.device as c_uint,
            pciDeviceId: self.pci_device_id as c_uint,
            pciSubSystemId: self.pci_sub_system_id as c_uint,
            reserved0: u32::MAX as c_uint,
            reserved1: u32::MAX as c_uint,
            reserved2: u32::MAX as c_uint,
            reserved3: u32::MAX as c_uint,
        })
    }
}

/// BAR1 memory allocation information for a device (in bytes)
// Checked against local
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BAR1MemoryInfo {
    /// Unallocated
    pub free: u64,
    /// Total memory 
    pub total: u64,
    /// Allocated
    pub used: u64,
}

impl From<nvmlBAR1Memory_t> for BAR1MemoryInfo {
    fn from(struct_: nvmlBAR1Memory_t) -> Self {
        BAR1MemoryInfo {
            free: struct_.bar1Free as u64,
            total: struct_.bar1Total as u64,
            used: struct_.bar1Used as u64,
        }
    }
}

/// Information about a bridge chip.
// Checked against local
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BridgeChipInfo {
    pub fw_version: FirmwareVersion,
    pub chip_type: BridgeChip,
}

impl From<nvmlBridgeChipInfo_t> for BridgeChipInfo {
    fn from(struct_: nvmlBridgeChipInfo_t) -> Self {
        let fw_version = FirmwareVersion::from(struct_.fwVersion as u32);
        let chip_type = BridgeChip::from(struct_.type_);

        BridgeChipInfo {
            fw_version: fw_version,
            chip_type: chip_type,
        }
    }
}

/**
This struct stores the complete hierarchy of the bridge chip within the board. 

The immediate bridge is stored at index 0 of `chips_hierarchy`. The parent to 
the immediate bridge is at index 1, and so forth.
*/
// Checked against local
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BridgeChipHierarchy {
    /// Hierarchy of bridge chips on the board.
    pub chips_hierarchy: Vec<BridgeChipInfo>,
    /// Number of bridge chips on the board.
    pub chip_count: u8,
}

impl From<nvmlBridgeChipHierarchy_t> for BridgeChipHierarchy {
    fn from(struct_: nvmlBridgeChipHierarchy_t) -> Self {
        let hierarchy = struct_.bridgeChipInfo.iter()
                                              .map(|bci| BridgeChipInfo::from(*bci))
                                              .collect();

        BridgeChipHierarchy {
            chips_hierarchy: hierarchy,
            chip_count: struct_.bridgeCount,
        }
    }
}

/// Information about compute processes running on the GPU.
// Checked against local
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ProcessInfo {
    // Process ID.
    pub pid: u32,
    /// Amount of used GPU memory in bytes.
    pub used_gpu_memory: UsedGpuMemory,
}

impl From<nvmlProcessInfo_t> for ProcessInfo {
    fn from(struct_: nvmlProcessInfo_t) -> Self {
        ProcessInfo {
            pid: struct_.pid,
            used_gpu_memory: UsedGpuMemory::from(struct_.usedGpuMemory),
        }
    }
}

/// Detailed ECC error counts for a device.
// Checked against local
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct EccErrorCounts {
    pub device_memory: u64,
    pub l1_cache: u64,
    pub l2_cache: u64,
    pub register_file: u64,
}

impl From<nvmlEccErrorCounts_t> for EccErrorCounts {
    fn from(struct_: nvmlEccErrorCounts_t) -> Self {
        EccErrorCounts {
            device_memory: struct_.deviceMemory as u64,
            l1_cache: struct_.l1Cache as u64,
            l2_cache: struct_.l2Cache as u64,
            register_file: struct_.registerFile as u64,
        }
    }
}

/// Memory allocation information for a device (in bytes).
// Checked against local
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct MemoryInfo {
    /// Unallocated FB memory.
    pub free: u64,
    /// Total installed FB memory.
    pub total: u64,
    /// Allocated FB memory.
    ///
    /// Note that the driver/GPU always sets aside a small amount of memory for bookkeeping.
    pub used: u64,
}

impl From<nvmlMemory_t> for MemoryInfo {
    fn from(struct_: nvmlMemory_t) -> Self {
        MemoryInfo {
            free: struct_.free as u64,
            total: struct_.total as u64,
            used: struct_.used as u64,
        }
    }
}

/// Utilization information for a device. Each sample period may be between 1 second
/// and 1/6 second, depending on the product being queried.
// Checked against local
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Utilization {
    /// Percent of time over the past sample period during which one or more kernels
    /// was executing on the GPU.
    pub gpu: u32,
    /// Percent of time over the past sample period during which global (device)
    /// memory was being read or written to.
    pub memory: u32,
}

impl From<nvmlUtilization_t> for Utilization {
    fn from(struct_: nvmlUtilization_t) -> Self {
        Utilization {
            gpu: struct_.gpu as u32,
            memory: struct_.memory as u32,
        }
    }
}

/// Performance policy violation status data.
// Checked against local
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ViolationTime {
    /// Represents CPU timestamp in microseconds.
    pub reference_time: u64,
    /// Violation time in nanoseconds.
    pub violation_time: u64,
}

impl From<nvmlViolationTime_t> for ViolationTime {
    fn from(struct_: nvmlViolationTime_t) -> Self {
        ViolationTime {
            reference_time: struct_.referenceTime as u64,
            violation_time: struct_.violationTime as u64,
        }
    }
}

/**
Accounting statistics for a process.

There is a field: `unsigned int reserved[5]` present on the C struct that this wraps
that NVIDIA says is "reserved for future use." If it ever gets used in the future,
an equivalent wrapping field will have to be added to this struct.
*/
// Checked against local
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct AccountingStats {
    /**
    Percent of time over the process's lifetime during which one or more kernels was
    executing on the GPU. This is just like what is returned by
    `Device.utilization_rates()` except it is for the lifetime of a process (not just
    the last sample period). 
    
    It will be `None` if `Device.utilization_rates()` is not supported.
    */
    pub gpu_utilization: Option<u32>,
    /// Whether the process is running.
    pub is_running: bool,
    /// Max total memory in bytes that was ever allocated by the process.
    ///
    /// It will be `None` if `ProcessInfo.used_gpu_memory` is not supported.
    pub max_memory_usage: Option<u64>,
    /**
    Percent of time over the process's lifetime during which global (device) memory
    was being read from or written to.
    
    It will be `None` if `Device.utilization_rates()` is not supported.
    */
    pub memory_utilization: Option<u32>,
    /// CPU timestamp in usec representing the start time for the process.
    pub start_time: u64,
    /// Amount of time in ms during which the compute context was active. This will be
    /// zero if the process is not terminated.
    pub time: u64,
}

impl From<nvmlAccountingStats_t> for AccountingStats {
    fn from(struct_: nvmlAccountingStats_t) -> Self {
        let not_avail_u64 = (NVML_VALUE_NOT_AVAILABLE) as u64;
        let not_avail_u32 = (NVML_VALUE_NOT_AVAILABLE) as u32;

        AccountingStats {
            gpu_utilization: match struct_.gpuUtilization as u32 {
                v if v == not_avail_u32 => None,
                _ => Some(struct_.gpuUtilization as u32),
            },
            is_running: match struct_.isRunning {
                0 => false,
                // NVIDIA only says 1 is for running, but I don't think anything
                // else warrants an error (or a panic), so
                _ => true,
            },
            max_memory_usage: match struct_.maxMemoryUsage as u64 {
                v if v == not_avail_u64 => None,
                _ => Some(struct_.maxMemoryUsage as u64),
            },
            memory_utilization: match struct_.memoryUtilization as u32 {
                v if v == not_avail_u32 => None,
                _ => Some(struct_.memoryUtilization as u32),
            },
            start_time: struct_.startTime as u64,
            time: struct_.time as u64,
        }
    }
}

/// Sample info.
// Checked against local
#[cfg(feature = "nightly")]
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Sample {
    /// CPU timestamp in μs
    pub timestamp: u64,
    pub value: SampleValue,
}

#[cfg(feature = "nightly")]
impl Sample {
    /// Given a tag and an untagged union, returns a Rust enum with the correct union variant.
    pub fn from_tag_and_struct(tag: &SampleValueType, struct_: nvmlSample_t) -> Self {
        Sample {
            timestamp: struct_.timeStamp as u64,
            value: SampleValue::from_tag_and_union(tag, struct_.sampleValue),
        }
    }
}

#[cfg(test)]
#[allow(unused_variables, unused_imports)]
mod tests {
    use test_utils::*;
    use error::*;
    use ffi::bindings::*;
    use std::mem;

    #[test]
    fn pci_info_from_to_c() { 
        let nvml = nvml();
        test_with_device(3, &nvml, |device| {
            let converted = device.pci_info()
                                  .expect("wrapped pci info")
                                  .try_into_c()
                                  .expect("converted c pci info");

            let raw = unsafe {
                let mut pci_info: nvmlPciInfo_t = mem::zeroed();
                nvml_try(nvmlDeviceGetPciInfo_v2(device.unsafe_raw(), &mut pci_info)).expect("raw pci info");
                pci_info
            };

            assert_eq!(converted.busId, raw.busId);
            assert_eq!(converted.domain, raw.domain);
            assert_eq!(converted.bus, raw.bus);
            assert_eq!(converted.device, raw.device);
            assert_eq!(converted.pciDeviceId, raw.pciDeviceId);
            assert_eq!(converted.pciSubSystemId, raw.pciSubSystemId);
            assert_eq!(converted.reserved0, raw.reserved0);
            assert_eq!(converted.reserved1, raw.reserved1);
            assert_eq!(converted.reserved2, raw.reserved2);
            assert_eq!(converted.reserved3, raw.reserved3);

            Ok(())
        })
    }
}
