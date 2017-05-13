use ffi::bindings::*;
use std::marker::PhantomData;
use std::os::raw::c_uint;
use std::slice;
use std::mem;
use device::Device;
use struct_wrappers::unit::*;
use enum_wrappers::unit::*;
use enums::unit::*;
use error::*;
use NVML;

/**
Struct that represents a unit. 

Obtain a `Unit` with the various methods available to you on the `NVML`
struct.

I don't know what a unit is, but inferring from the docs leads me to believe 
it's some kind of high-end something-or-other that 99% of users won't know 
about either. That being said, I'm wrapping this whole library, so here you go.

Rust's lifetimes will ensure that the NVML instance this `Unit` was created from
is not allowed to be shutdown until this `Unit` is dropped, meaning you shouldn't
have to worry about calls returning `Uninitialized` errors.
*/
// TODO: Use compiletest to ensure lifetime guarantees
#[derive(Debug)]
pub struct Unit<'nvml> {
    unit: nvmlUnit_t,
    _phantom: PhantomData<&'nvml NVML>,
}

// Here to clarify that Unit does have these traits. I know they are implemented without this.
unsafe impl<'nvml> Send for Unit<'nvml> {}
unsafe impl<'nvml> Sync for Unit<'nvml> {}

impl<'nvml> From<nvmlUnit_t> for Unit<'nvml> {
    fn from(unit: nvmlUnit_t) -> Self {
        Unit {
            unit,
            _phantom: PhantomData,
        }
    }
}

impl<'nvml> Unit<'nvml> {
    /**
    Gets the set of GPU devices that are attached to this `Unit`.
    
    # Errors
    * `Uninitialized`, if the library has not been successfully initialized
    * `InsufficientSize`, if `size` is not enough for the array of devices
    * `InvalidArg`, if the unit is invalid
    * `Unknown`, on any unexpected error
    
    # Device Support
    For S-class products.
    */
    // TODO: Validate insufficientsize? ^
    // Checked against local
    #[inline]
    pub fn devices(&self, size: usize) -> Result<Vec<Device>> {
        unsafe {
            let mut first_item: nvmlDevice_t = mem::zeroed();
            let mut count: c_uint = size as c_uint;
            nvml_try(nvmlUnitGetDevices(self.unit, &mut count, &mut first_item))?;

            // TODO: Is this correct, safe, etc.
            Ok(slice::from_raw_parts(first_item as *const nvmlDevice_t,
                                     count as usize)
                                     .iter()
                                     .map(|d| Device::from(*d))
                                     .collect())
        }
    }

    /**
    Gets fan information for this `Unit` (fan count and state + speed for each).
    
    # Errors
    * `Uninitialized`, if the library has not been successfully initialized
    * `InvalidArg`, if the unit is invalid
    * `NotSupported`, if this is not an S-class product
    * `Unknown`, on any unexpected error
    
    # Device Support
    For S-class products.
    */
    // Checked against local
    #[inline]
    pub fn fan_info(&self) -> Result<UnitFansInfo> {
        unsafe {
            let mut fans_info: nvmlUnitFanSpeeds_t = mem::zeroed();
            nvml_try(nvmlUnitGetFanSpeedInfo(self.unit, &mut fans_info))?;

            Ok(fans_info.into())
        }
    }

    /**
    Gets the LED state associated with this `Unit`.
    
    # Errors
    * `Uninitialized`, if the library has not been successfully initialized
    * `InvalidArg`, if the unit is invalid
    * `NotSupported`, if this is not an S-class product
    * `Utf8Error`, if the string obtained from the C function is not valid Utf8
    * `Unknown`, on any unexpected error
    
    # Device Support
    For S-class products.
    */
    // Checked against local
    #[inline]
    pub fn led_state(&self) -> Result<UnitLedState> {
        unsafe {
            let mut state: nvmlLedState_t = mem::zeroed();
            nvml_try(nvmlUnitGetLedState(self.unit, &mut state))?;

            Ok(UnitLedState::try_from(state)?)
        }
    }

    /**
    Gets the PSU stats for this `Unit`.
    
    # Errors
    * `Uninitialized`, if the library has not been successfully initialized
    * `InvalidArg`, if the unit is invalid
    * `NotSupported`, if this is not an S-class product
    * `Utf8Error`, if the string obtained from the C function is not valid Utf8
    * `Unknown`, on any unexpected error
    
    # Device Support
    For S-class products.
    */
    // Checked against local
    #[inline]
    pub fn psu_info(&self) -> Result<UnitPsuInfo> {
        unsafe {
            let mut info: nvmlPSUInfo_t = mem::zeroed();
            nvml_try(nvmlUnitGetPsuInfo(self.unit, &mut info))?;

            Ok(UnitPsuInfo::try_from(info)?)
        }
    }

    /**
    Gets the temperature for the specified `UnitTemperatureReading`, in °C.
    
    Available readings depend on the product.
    
    # Errors
    * `Uninitialized`, if the library has not been successfully initialized
    * `InvalidArg`, if the unit is invalid
    * `NotSupported`, if this is not an S-class product
    * `Unknown`, on any unexpected error
    
    # Device Support
    For S-class products. Available readings depend on the product.
    */
    // Checked against local
    #[inline]
    pub fn temperature(&self, reading_type: UnitTemperatureReading) -> Result<u32> {
        unsafe {
            let mut temp: c_uint = mem::zeroed();
            nvml_try(nvmlUnitGetTemperature(self.unit, reading_type as c_uint, &mut temp))?;

            Ok(temp as u32)
        }
    }

    /**
    Gets the static information associated with this `Unit`.
    
    # Errors
    * `Uninitialized`, if the library has not been successfully initialized
    * `InvalidArg`, if the unit is invalid
    * `Utf8Error`, if the string obtained from the C function is not valid Utf8
    
    # Device Support
    For S-class products.
    */
    // Checked against local
    #[inline]
    pub fn info(&self) -> Result<UnitInfo> {
        unsafe {
            let mut info: nvmlUnitInfo_t = mem::zeroed();
            nvml_try(nvmlUnitGetUnitInfo(self.unit, &mut info))?;

            Ok(UnitInfo::try_from(info)?)
        }
    }

    // Unit commands starting here

    /**
    Sets the LED color for this `Unit`.
    
    Requires root/admin permissions. This operation takes effect immediately.
    
    Note: Current S-class products don't provide unique LEDs for each unit. As such,
    both front and back LEDs will be toggled in unison regardless of which unit is
    specified with this method (aka the `Unit` represented by this struct).
    
    # Errors
    * `Uninitialized`, if the library has not been successfully initialized
    * `InvalidArg`, if the unit is invalid
    * `NotSupported`, if this is not an S-class product
    * `NoPermission`, if the user doesn't have permission to perform this operation
    * `Unknown`, on any unexpected error
    
    # Device Support
    For S-class products.
    */
    // checked against local
    #[inline]
    pub fn set_led_color(&mut self, color: LedColor) -> Result<()> {
        unsafe {
            nvml_try(nvmlUnitSetLedState(self.unit, color.into_c()))
        }
    }

    /// Consume the struct and obtain the raw unit handle that it contains.
    #[inline]
    pub fn into_raw(self) -> nvmlUnit_t {
        self.unit
    }

    /// Obtain a reference to the raw unit handle contained in the struct.
    #[inline]
    pub fn as_raw(&self) -> &nvmlUnit_t {
        &(self.unit)
    }

    /// Obtain a mutable reference to the raw unit handle contained in the struct.
    #[inline]
    pub fn as_mut_raw(&mut self) -> &mut nvmlUnit_t {
        &mut (self.unit)
    }

    /// Sometimes necessary for C interop. Use carefully.
    #[inline]
    pub unsafe fn unsafe_raw(&self) -> nvmlUnit_t {
        self.unit
    }
}
