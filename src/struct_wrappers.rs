use super::ffi::*;
use super::nvml_errors::*;
use super::enum_wrappers::*;
use super::enums::*;
use super::std::mem;
use std::ffi::CStr;

// TODO: Document errors for try_froms

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
        // TODO: Why is this value not used?
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

impl HwbcEntry {
    /// Waiting for `TryFrom` to be stable. In the meantime, we do this.
    pub fn try_from(struct_: nvmlHwbcEntry_t) -> Result<Self> {
        unsafe {
            let version_raw = CStr::from_ptr(struct_.firmwareVersion.as_ptr());
            Ok(HwbcEntry {
                id: struct_.hwbcId as u32,
                firmware_version: version_raw.to_str()?.into()
            })
        }
    }
}

/// Fan information readings for an entire S-class unit.
#[derive(Debug)]
pub struct UnitFansInfo {
    /// Number of fans in the unit.
    pub count: u32,
    /// Fan data for each fan.
    pub fans: Vec<FanInfo>,
}

impl From<nvmlUnitFanSpeeds_t> for UnitFansInfo {
    fn from(struct_: nvmlUnitFanSpeeds_t) -> Self {
        UnitFansInfo {
            count: struct_.count as u32,
            fans: struct_.fans.iter().map(|f| FanInfo::from(*f)).collect(),
        }
    }
}

/// Fan info reading for a single fan in an S-class unit.
#[derive(Debug)]
pub struct FanInfo {
    /// Fan speed (RPM).
    pub speed: u32,
    /// Indicates whether a fan is working properly.
    pub state: FanState,
}

impl From<nvmlUnitFanInfo_t> for FanInfo {
    fn from(struct_: nvmlUnitFanInfo_t) -> Self {
        FanInfo {
            speed: struct_.speed as u32,
            state: struct_.state.into(),
        }
    }
}

/// Power usage information for an S-class unit. The power supply state is a human-readable
/// string that equals "Normal" or contains a combination of "Abnormal" plus one or more of
/// the following (aka good luck matching on it):
///
/// * High voltage
/// * Fan failure
/// * Heatsink temperature
/// * Current limit
/// * Voltage below UV alarm threshold
/// * Low voltage
/// * SI2C remote off command
/// * MOD_DISABLE input
/// * Short pin transition
#[derive(Debug)]
pub struct UnitPsuInfo {
    /// PSU current (in A)
    pub current: u32,
    /// PSU power draw (in W)
    pub power_draw: u32,
    /// Human-readable string describing the PSU state.
    pub state: String,
    /// PSU voltage (in V)
    pub voltage: u32,
}

impl UnitPsuInfo {
    /// Waiting for `TryFrom` to be stable. In the meantime, we do this.
    pub fn try_from(struct_: nvmlPSUInfo_t) -> Result<Self> {
        unsafe {
            let state_raw = CStr::from_ptr(struct_.state.as_ptr());
            Ok(UnitPsuInfo {
                current: struct_.current as u32,
                power_draw: struct_.power as u32,
                state: state_raw.to_str()?.into(),
                voltage: struct_.voltage as u32,
            })
        }
    }
}

/// Static S-class unit info.
#[derive(Debug)]
pub struct UnitInfo {
    pub firmware_version: String,
    /// Product identifier.
    pub id: String,
    pub name: String,
    /// Product serial number.
    pub serial: String,
}

impl UnitInfo {
    /// Waiting for `TryFrom` to be stable. In the meantime, we do this.
    pub fn try_from(struct_: nvmlUnitInfo_t) -> Result<Self> {
        unsafe {
            let version_raw = CStr::from_ptr(struct_.firmwareVersion.as_ptr());
            let id_raw = CStr::from_ptr(struct_.id.as_ptr());
            let name_raw = CStr::from_ptr(struct_.name.as_ptr());
            let serial_raw = CStr::from_ptr(struct_.serial.as_ptr());

            Ok(UnitInfo {
                firmware_version: version_raw.to_str()?.into(),
                id: id_raw.to_str()?.into(),
                name: name_raw.to_str()?.into(),
                serial: serial_raw.to_str()?.into(),
            })
        }
    }
}

/// Accounting statistics for a process.
///
/// There is a field: `unsigned int reserved[5]` present on the C struct that this wraps
/// that NVIDIA says is "reserved for future use." If it ever gets used in the future,
/// an equivalent wrapping field will have to be added to this struct.
#[derive(Debug)]
pub struct AccountingStats {
    /// Percent of time over the process's lifetime during which one or more kernels was
    /// executing on the GPU. This is just like what is returned by
    /// `Device.utilization_rates()` except it is for the lifetime of a process (not just
    /// the last sample period). 
    ///
    /// It will be `None` if `Device.utilization_rates()` is not supported.
    pub gpu_utilization: Option<u32>,
    /// Whether the process is running.
    pub is_running: bool,
    /// Max total memory in bytes that was ever allocated by the process.
    ///
    /// It will be `None` if `nvmlProcessInfo_t->usedGpuMemory` is not supported.
    pub max_memory_usage: Option<u64>,
    /// Percent of time over the process's lifetime during which global (device) memory
    /// was being read from or written to.
    ///
    /// It will be `None` if `Device.utilization_rates()` is not supported.
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
