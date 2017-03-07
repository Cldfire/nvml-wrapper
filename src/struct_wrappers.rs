use super::ffi::*;
use super::errors::*;
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
