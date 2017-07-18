use enum_wrappers::device::*;
use enums::device::*;
use error::*;
use ffi::bindings::*;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
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
    /**
    The 32-bit Sub System Device ID.
    
    Will always be `None` if this `PciInfo` was obtained from `NvLink.remote_pci_info()`.
    NVIDIA says that the C field that this corresponds to "is not filled ... and
    is indeterminate" when being returned from that specific call.
    */
    pub pci_sub_system_id: Option<u32>
}

impl PciInfo {
    /**
    Waiting for `TryFrom` to be stable. In the meantime, we do this.

    Passing `false` for `sub_sys_id_present` will set the `pci_sub_system_id` field to
    `None`. See the field docs for more.

    # Errors

    * `Utf8Error`, if the string obtained from the C function is not valid Utf8
    */
    pub fn try_from(struct_: nvmlPciInfo_t, sub_sys_id_present: bool) -> Result<Self> {
        unsafe {
            let bus_id_raw = CStr::from_ptr(struct_.busId.as_ptr());
            Ok(PciInfo {
                bus: struct_.bus,
                bus_id: bus_id_raw.to_str()?.into(),
                device: struct_.device,
                domain: struct_.domain,
                pci_device_id: struct_.pciDeviceId,
                pci_sub_system_id: if sub_sys_id_present {
                    Some(struct_.pciSubSystemId)
                } else {
                    None
                }
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

        bus_id_c
            .clone_from_slice(&bus_id.iter().map(|b| *b as c_char).collect::<Vec<_>>());

        Ok(nvmlPciInfo_t {
            busId: bus_id_c,
            domain: self.domain,
            bus: self.bus,
            device: self.device,
            pciDeviceId: self.pci_device_id,
            pciSubSystemId: if let Some(id) = self.pci_sub_system_id {
                id
            } else {
                // This seems the most correct thing to do? Since this should only
                // be none if obtained from `NvLink.remote_pci_info()`.
                0
            },
            reserved0: u32::MAX,
            reserved1: u32::MAX,
            reserved2: u32::MAX,
            reserved3: u32::MAX
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
    pub used: u64
}

impl From<nvmlBAR1Memory_t> for BAR1MemoryInfo {
    fn from(struct_: nvmlBAR1Memory_t) -> Self {
        BAR1MemoryInfo {
            free: struct_.bar1Free,
            total: struct_.bar1Total,
            used: struct_.bar1Used
        }
    }
}

/// Information about a bridge chip.
// Checked against local
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BridgeChipInfo {
    pub fw_version: FirmwareVersion,
    pub chip_type: BridgeChip
}

impl BridgeChipInfo {
    /**
    Construct `BridgeChipInfo` from the corresponding C struct.

    # Errors

    * `UnexpectedVariant`, for which you can read the docs for
    */
    pub fn try_from(struct_: nvmlBridgeChipInfo_t) -> Result<Self> {
        let fw_version = FirmwareVersion::from(struct_.fwVersion);
        let chip_type = BridgeChip::try_from(struct_.type_)?;

        Ok(BridgeChipInfo {
            fw_version,
            chip_type
        })
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
    pub chip_count: u8
}

impl BridgeChipHierarchy {
    /**
    Construct `BridgeChipHierarchy` from the corresponding C struct.

    # Errors
    
    * `UnexpectedVariant`, for which you can read the docs for
    */
    pub fn try_from(struct_: nvmlBridgeChipHierarchy_t) -> Result<Self> {
        let chips_hierarchy: Result<Vec<BridgeChipInfo>> = struct_
            .bridgeChipInfo
            .iter()
            .map(|bci| BridgeChipInfo::try_from(*bci))
            .collect();

        let chips_hierarchy = chips_hierarchy?;

        Ok(BridgeChipHierarchy {
            chips_hierarchy,
            chip_count: struct_.bridgeCount
        })
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
    pub used_gpu_memory: UsedGpuMemory
}

impl From<nvmlProcessInfo_t> for ProcessInfo {
    fn from(struct_: nvmlProcessInfo_t) -> Self {
        ProcessInfo {
            pid: struct_.pid,
            used_gpu_memory: UsedGpuMemory::from(struct_.usedGpuMemory)
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
    pub register_file: u64
}

impl From<nvmlEccErrorCounts_t> for EccErrorCounts {
    fn from(struct_: nvmlEccErrorCounts_t) -> Self {
        EccErrorCounts {
            device_memory: struct_.deviceMemory,
            l1_cache: struct_.l1Cache,
            l2_cache: struct_.l2Cache,
            register_file: struct_.registerFile
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
    /// Note that the driver/GPU always sets aside a small amount of memory for
    /// bookkeeping.
    pub used: u64
}

impl From<nvmlMemory_t> for MemoryInfo {
    fn from(struct_: nvmlMemory_t) -> Self {
        MemoryInfo {
            free: struct_.free,
            total: struct_.total,
            used: struct_.used
        }
    }
}

/// Utilization information for a device. Each sample period may be between 1
/// second and 1/6 second, depending on the product being queried.
// Checked against local
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Utilization {
    /// Percent of time over the past sample period during which one or more
    /// kernels was executing on the GPU.
    pub gpu: u32,
    /// Percent of time over the past sample period during which global (device)
    /// memory was being read or written to.
    pub memory: u32
}

impl From<nvmlUtilization_t> for Utilization {
    fn from(struct_: nvmlUtilization_t) -> Self {
        Utilization {
            gpu: struct_.gpu,
            memory: struct_.memory
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
    pub violation_time: u64
}

impl From<nvmlViolationTime_t> for ViolationTime {
    fn from(struct_: nvmlViolationTime_t) -> Self {
        ViolationTime {
            reference_time: struct_.referenceTime,
            violation_time: struct_.violationTime
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
    /// Amount of time in ms during which the compute context was active. This
    /// will be zero if the process is not terminated.
    pub time: u64
}

impl From<nvmlAccountingStats_t> for AccountingStats {
    fn from(struct_: nvmlAccountingStats_t) -> Self {
        let not_avail_u64 = (NVML_VALUE_NOT_AVAILABLE) as u64;
        let not_avail_u32 = (NVML_VALUE_NOT_AVAILABLE) as u32;

        AccountingStats {
            gpu_utilization: match struct_.gpuUtilization {
                v if v == not_avail_u32 => None,
                _ => Some(struct_.gpuUtilization),
            },
            is_running: match struct_.isRunning {
                0 => false,
                // NVIDIA only says 1 is for running, but I don't think anything
                // else warrants an error (or a panic), so
                _ => true,
            },
            max_memory_usage: match struct_.maxMemoryUsage {
                v if v == not_avail_u64 => None,
                _ => Some(struct_.maxMemoryUsage),
            },
            memory_utilization: match struct_.memoryUtilization {
                v if v == not_avail_u32 => None,
                _ => Some(struct_.memoryUtilization),
            },
            start_time: struct_.startTime,
            time: struct_.time
        }
    }
}

/// Sample info.
// Checked against local
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Sample {
    /// CPU timestamp in μs
    pub timestamp: u64,
    pub value: SampleValue
}

impl Sample {
    /// Given a tag and an untagged union, returns a Rust enum with the correct
    /// union variant.
    pub fn from_tag_and_struct(tag: &SampleValueType, struct_: nvmlSample_t) -> Self {
        Sample {
            timestamp: struct_.timeStamp,
            value: SampleValue::from_tag_and_union(tag, struct_.sampleValue)
        }
    }
}

#[cfg(test)]
#[allow(unused_variables, unused_imports)]
mod tests {
    use error::*;
    use ffi::bindings::*;
    use std::mem;
    use test_utils::*;

    #[test]
    fn pci_info_from_to_c() {
        let nvml = nvml();
        test_with_device(3, &nvml, |device| {
            let converted = device
                .pci_info()
                .expect("wrapped pci info")
                .try_into_c()
                .expect("converted c pci info");

            let raw = unsafe {
                let mut pci_info: nvmlPciInfo_t = mem::zeroed();
                nvml_try(nvmlDeviceGetPciInfo_v2(device.unsafe_raw(), &mut pci_info))
                    .expect("raw pci info");
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
