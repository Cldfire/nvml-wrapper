use crate::device::Device;
use crate::enum_wrappers::unit::LedColor;
use crate::enums::unit::{LedState, TemperatureReading};
use crate::error::{nvml_try, Result};
use crate::ffi::bindings::*;
use crate::struct_wrappers::unit::{FansInfo, PsuInfo, UnitInfo};
use crate::NVML;
use std::marker::PhantomData;
use std::mem;
use std::os::raw::c_uint;

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

    **I do not have the hardware to test this call. Verify for yourself that it
    works before you use it**. If it works, please let me know; if it doesn't,
    I would love a PR. If NVML is sane this should work, but NVIDIA's docs
    on this call are _anything_ but clear.

    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `InvalidArg`, if the unit is invalid
    * `Unknown`, on any unexpected error

    # Device Support

    For S-class products.
    */
    // Checked against local
    // Tested
    pub fn devices(&self) -> Result<Vec<Device>> {
        unsafe {
            let mut count: c_uint = match self.device_count()? {
                0 => return Ok(vec![]),
                value => value,
            };
            let mut devices: Vec<nvmlDevice_t> = vec![mem::zeroed(); count as usize];

            nvml_try(nvmlUnitGetDevices(
                self.unit,
                &mut count,
                devices.as_mut_ptr(),
            ))?;

            Ok(devices.into_iter().map(Device::from).collect())
        }
    }

    /**
    Gets the count of GPU devices that are attached to this `Unit`.

    **I do not have the hardware to test this call. Verify for yourself that it
    works before you use it**. If it works, please let me know; if it doesn't,
    I would love a PR. If NVML is sane this should work, but NVIDIA's docs
    on this call are _anything_ but clear.

    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `InvalidArg`, if the unit is invalid
    * `Unknown`, on any unexpected error

    # Device Support

    For S-class products.
    */
    // Tested as part of the above
    pub fn device_count(&self) -> Result<u32> {
        unsafe {
            /*
            NVIDIA doesn't even say that `count` will be set to the count if
            `InsufficientSize` is returned. But we can assume sanity, right?

            The idea here is:
            If there are 0 devices, NVML_SUCCESS is returned, `count` is set
              to 0. We return count, all good.
            If there is 1 device, NVML_SUCCESS is returned, `count` is set to
              1. We return count, all good.
            If there are >= 2 devices, NVML_INSUFFICIENT_SIZE is returned.
             `count` is theoretically set to the actual count, and we
              return it.
            */
            let mut count: c_uint = 1;
            let mut devices: [nvmlDevice_t; 1] = [mem::zeroed()];

            match nvmlUnitGetDevices(self.unit, &mut count, devices.as_mut_ptr()) {
                nvmlReturn_enum_NVML_SUCCESS | nvmlReturn_enum_NVML_ERROR_INSUFFICIENT_SIZE => {
                    Ok(count)
                }
                // We know that this will be an error
                other => nvml_try(other).map(|_| 0),
            }
        }
    }

    /**
    Gets fan information for this `Unit` (fan count and state + speed for each).

    # Errors

    * `Uninitialized`, if the library has not been successfully initialized
    * `InvalidArg`, if the unit is invalid
    * `NotSupported`, if this is not an S-class product
    * `UnexpectedVariant`, for which you can read the docs for
    * `Unknown`, on any unexpected error

    # Device Support

    For S-class products.
    */
    // Checked against local
    // Tested
    pub fn fan_info(&self) -> Result<FansInfo> {
        unsafe {
            let mut fans_info: nvmlUnitFanSpeeds_t = mem::zeroed();
            nvml_try(nvmlUnitGetFanSpeedInfo(self.unit, &mut fans_info))?;

            Ok(FansInfo::try_from(fans_info)?)
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
    // Tested
    pub fn led_state(&self) -> Result<LedState> {
        unsafe {
            let mut state: nvmlLedState_t = mem::zeroed();
            nvml_try(nvmlUnitGetLedState(self.unit, &mut state))?;

            Ok(LedState::try_from(state)?)
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
    // Tested
    pub fn psu_info(&self) -> Result<PsuInfo> {
        unsafe {
            let mut info: nvmlPSUInfo_t = mem::zeroed();
            nvml_try(nvmlUnitGetPsuInfo(self.unit, &mut info))?;

            Ok(PsuInfo::try_from(info)?)
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
    // Tested
    pub fn temperature(&self, reading_type: TemperatureReading) -> Result<u32> {
        unsafe {
            let mut temp: c_uint = mem::zeroed();

            nvml_try(nvmlUnitGetTemperature(
                self.unit,
                reading_type as c_uint,
                &mut temp,
            ))?;

            Ok(temp)
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
    // Tested
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
    // Tested (no-run)
    pub fn set_led_color(&mut self, color: LedColor) -> Result<()> {
        unsafe { nvml_try(nvmlUnitSetLedState(self.unit, color.as_c())) }
    }

    /// Get the raw unit handle contained in this struct
    ///
    /// Sometimes necessary for C interop.
    ///
    /// # Safety
    ///
    /// This is unsafe to prevent it from being used without care.
    pub unsafe fn handle(&self) -> nvmlUnit_t {
        self.unit
    }
}

// I do not have access to this hardware and cannot test anything
#[cfg(test)]
#[cfg(not(feature = "test-local"))]
#[deny(unused_mut)]
mod test {
    use crate::enum_wrappers::unit::LedColor;
    use crate::enums::unit::TemperatureReading;
    use crate::test_utils::*;

    #[test]
    fn devices() {
        let nvml = nvml();
        let unit = unit(&nvml);
        unit.devices().expect("devices");
    }

    #[test]
    fn fan_info() {
        let nvml = nvml();
        test_with_unit(3, &nvml, |unit| unit.fan_info())
    }

    #[test]
    fn led_state() {
        let nvml = nvml();
        test_with_unit(3, &nvml, |unit| unit.led_state())
    }

    #[test]
    fn psu_info() {
        let nvml = nvml();
        test_with_unit(3, &nvml, |unit| unit.psu_info())
    }

    #[test]
    fn temperature() {
        let nvml = nvml();
        test_with_unit(3, &nvml, |unit| unit.temperature(TemperatureReading::Board))
    }

    #[test]
    fn info() {
        let nvml = nvml();
        test_with_unit(3, &nvml, |unit| unit.info())
    }

    // This modifies unit state, so we don't want to actually run the test
    #[allow(dead_code)]
    fn set_led_color() {
        let nvml = nvml();
        let mut unit = unit(&nvml);

        unit.set_led_color(LedColor::Amber).expect("set to true")
    }
}
