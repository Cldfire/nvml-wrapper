use super::ffi::*;
use super::nvml_errors::*;
use super::enum_wrappers::*;
use super::enums::*;
use super::std::mem;
use std::ffi::CStr;

/// PCI information about a GPU device.
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
    /// Waiting for `TryFrom` to be stable. In the meantime, we do this.
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
}

/// BAR1 memory allocation information for a device (in bytes)
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

/// This struct stores the complete hierarchy of the bridge chip within the board. 
/// 
/// The immediate bridge is stored at index 0 of `chips_hierarchy`. The parent to 
/// the immediate bridge is at index 1, and so forth. 
///
/// If that explanation didn't make anything clear to you, it's not clear to me either,
/// I just copied what NVIDIA's docs said. I suppose it's logical to assume that anyone
/// interested in doing anything with this will already know what a bridge chip 
/// hierarchy is.
pub struct BridgeChipHierarchy {
    /// Hierarchy of bridge chips on the board.
    pub chips_hierarchy: Vec<BridgeChipInfo>,
    /// Number of bridge chips on the board.
    pub chip_count: u8,
}

// TODO: profile this?
// TODO: provide user with explicit option to choose how much mem they want to allocate in advance?
impl From<nvmlBridgeChipHierarchy_t> for BridgeChipHierarchy {
    fn from(struct_: nvmlBridgeChipHierarchy_t) -> Self {
        // Allocate 1/8 possible size in advance
        // [BridgeChipInfo; 128] is currently (3-7-17) 1536 bytes
        // This means we currently allocate 192 bytes
        // TODO: Check that order is correct here (very important that it is!)
        let mut hierarchy: Vec<BridgeChipInfo>
             = Vec::with_capacity(mem::size_of::<[BridgeChipInfo; NVML_MAX_PHYSICAL_BRIDGE as usize]>() / 8);
        hierarchy = struct_.bridgeChipInfo.iter()
                                          .map(|bci| BridgeChipInfo::from(*bci))
                                          .collect();
        // TODO: To shrink or not to shrink? Afaik it does not reallocate, so
        hierarchy.shrink_to_fit();

        BridgeChipHierarchy {
            chips_hierarchy: hierarchy,
            chip_count: struct_.bridgeCount,
        }
    }
}

/// Information about compute processes running on the GPU.
#[derive(Debug)]
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

#[derive(Debug)]
/// Detailed ECC error counts for a device.
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

#[derive(Debug)]
/// Memory allocation information for a device (in bytes).
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

#[derive(Debug)]
/// Utilization information for a device. Each sample period may be between 1 second
/// and 1/6 second, depending on the product being queried.
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

#[derive(Debug)]
/// Performance policy violation status data.
pub struct ViolationTime {
    /// Represents CPU timestamp in microseconds.
    pub reference_time: u64,
    /// in nanoseconds
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

/// Description of an HWBC entry.
#[derive(Debug)]
pub struct HwbcEntry {
    pub id: u32,
    pub firmware_version: String,
}

// In progress
// impl From<nvmlHwbcEntry_t> for HwbcEntry {
//     fn from(struct_: nvmlHwbcEntry_t) {
//         HwbcEntry {

//         }
//     }
// }
