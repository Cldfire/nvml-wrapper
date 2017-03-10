use ffi::*;
use std::marker::PhantomData;
use NVML;

/// Struct that represents a unit. 
///
/// Obtain a `Unit` with the various methods available to you on the `NVML`
/// struct.
///
/// I don't know what a unit is, but inferring from the docs leads me to believe 
/// it's some kind of high-end something or other that 99% of users won't know 
/// about either. That being said, I'm wrapping this whole library, so here you go.
///
/// Rust's lifetimes will ensure that the NVML instance this `Unit` was created from
/// is not allowed to be shutdown until this `Unit` is dropped, meaning you shouldn't
/// have to worry about calls returning `Uninitialized` errors.
// TODO: Use compiletest to ensure lifetime guarantees
#[derive(Debug)]
pub struct Unit<'nvml> {
    unit: nvmlUnit_t,
    _phantom: PhantomData<&'nvml NVML>,
}

unsafe impl<'nvml> Send for Unit<'nvml> {}
unsafe impl<'nvml> Sync for Unit<'nvml> {}

impl<'nvml> From<nvmlUnit_t> for Unit<'nvml> {
    fn from(unit: nvmlUnit_t) -> Self {
        Unit {
            unit: unit,
            _phantom: PhantomData,
        }
    }
}

impl<'nvml> Unit<'nvml> {
    
}