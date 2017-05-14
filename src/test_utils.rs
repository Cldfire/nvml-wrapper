#![allow(dead_code, unused_variables)]

use NVML;
use unit::Unit;
use event::EventSet;
use error::*;
use device::Device;
use enum_wrappers::device::*;
use struct_wrappers::device::*;
use struct_wrappers::event::*;
use structs::device::*;
use bitmasks::device::*;
use bitmasks::event::*;
use std::fmt::Debug;

pub trait ShouldPrint: Debug {
    fn should_print(&self) -> bool {
        true
    }
}

impl ShouldPrint for () {
    fn should_print(&self) -> bool {
        false
    }
}

impl<'nvml> ShouldPrint for Device<'nvml> {
    fn should_print(&self) -> bool {
        false
    }
}

impl<'nvml> ShouldPrint for Unit<'nvml> {
    fn should_print(&self) -> bool {
        false
    }
}

impl<'nvml> ShouldPrint for EventSet<'nvml> {
    fn should_print(&self) -> bool {
        false
    }
}

impl ShouldPrint for bool {}
impl ShouldPrint for u32 {}
impl ShouldPrint for (u32, u32) {}
impl ShouldPrint for u64 {}
impl ShouldPrint for String {}
impl ShouldPrint for Brand {}
impl ShouldPrint for [i8; 16] {}
impl ShouldPrint for Vec<ProcessInfo> {}
impl<'nvml> ShouldPrint for Vec<Device<'nvml>> {}
impl ShouldPrint for Vec<u32> {}
impl ShouldPrint for Vec<u64> {}
#[cfg(feature = "nightly")]
impl ShouldPrint for Vec<Sample> {}
impl ShouldPrint for Utilization {}
impl ShouldPrint for AutoBoostClocksEnabledInfo {}
impl ShouldPrint for BAR1MemoryInfo {}
impl ShouldPrint for BridgeChipHierarchy {}
impl ShouldPrint for ComputeMode {}
impl ShouldPrint for UtilizationInfo {}
impl ShouldPrint for EccModeInfo {}
impl ShouldPrint for OperationModeInfo {}
impl ShouldPrint for InfoROM {}
impl ShouldPrint for MemoryInfo {}
impl ShouldPrint for PciInfo {}
impl ShouldPrint for PerformanceState {}
impl ShouldPrint for PowerManagementConstraints {}
impl ShouldPrint for ThrottleReasons {}
impl ShouldPrint for ViolationTime {}
impl ShouldPrint for AccountingStats {}
impl ShouldPrint for EventTypes {}
impl<'nvml> ShouldPrint for EventData<'nvml> {}

pub fn nvml() -> NVML {
    NVML::init().expect("initialized library")
}

pub fn device<'nvml>(nvml: &'nvml NVML) -> Device<'nvml> {
    nvml.device_by_index(0).expect("device")
}

pub fn assert_send<T: Send>() {}
pub fn assert_sync<T: Sync>() {}

/// Run all testing methods for the given test.
pub fn test<T, R>(reps: usize, test: T)
    where T: Fn() -> (Result<R>),
          R: ShouldPrint {
    single(|| {
        test()
    });

    multi(reps, || {
        test()
    });
}

pub fn test_with_device<T, R>(reps: usize, nvml: &NVML, test: T)
    where T: Fn(&Device) -> (Result<R>),
          R: ShouldPrint {
    let device = device(nvml);

    single(|| {
        test(&device)
    });

    multi(reps, || {
        test(&device)
    });
}

/// Run the given test once.
pub fn single<T, R>(test: T)
    where T: Fn() -> (Result<R>),
          R: ShouldPrint {
    let res = test().expect("successful single test");

    if res.should_print() {
        print!("{:?} ... ", res);
    }
}

/// Run the given test multiple times.
pub fn multi<T, R>(count: usize, test: T) 
    where T: Fn() -> (Result<R>),
          R: ShouldPrint {
    for i in 0..count {
        test().expect(&format!("successful multi call #{}", i));
    }
}
