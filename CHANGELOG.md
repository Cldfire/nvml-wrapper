# nvml-wrapper Changelog

This file describes the changes / additions / fixes between wrapper releases.

## Unreleased

### Additions

* An import library (`nvml.lib`) has been added that enables compilation using the MSVC toolchain on Windows.
* `bitmasks`
  * Added throttle reasons:
    * `SW_THERMAL_SLOWDOWN`
    * `HW_THERMAL_SLOWDOWN`
    * `HW_POWER_BRAKE_SLOWDOWN`
    * `DISPLAY_CLOCK_SETTING`
  * `InitFlags`
* `enums::device`
  * `SampleValue`
    * Added variant `I64(i64)`
    * The `from_tag_and_union` constructor has been updated to support `i64` values
* `enum_wrappers::device`
  * `MemoryLocation`
    * Added variants:
      * `Cbu`
      * `SRAM`
  * `TemperatureThreshold`
    * Added variants:
      * `MemoryMax`
      * `GpuMax`
  * `PerformancePolicy`
    * Added variants:
      * `BoardLimit`
      * `LowUtilization`
      * `Reliability`
      * `TotalAppClocks`
      * `TotalBaseClocks`
  * `SampleValueType`
    * Added variant `SignedLongLong`
  * `Brand`
    * Added variant `Titan`
  * Added the `EncoderType` enum
  * Added the `DetachGpuState` enum
  * Added the `PcieLinkState` enum
* `Device` struct:
  * Added methods:
    * `cuda_compute_capability`
    * `encoder_capacity`
    * `encoder_stats`
    * `encoder_sessions`
    * `process_utilization_stats`
    * `total_energy_consumption`
    * `field_values_for`
* `error`
  * Added errors:
    * `InsufficientMemory`
    * `VgpuEccNotSupported`
* `NVML` struct:
  * Added methods:
    * `init_with_flags`
    * `sys_cuda_driver_version`
* `structs::device`
  * Added structs:
    * `EncoderStats`
    * `CudaComputeCapability`
    * `FieldId`
* `struct_wrappers::device`
  * Added structs:
    * `EncoderSessionInfo`
    * `ProcessUtilizationSample`
    * `FieldValueSample`

### Removals

* `bitmasks`
  * `ThrottleReasons::Unknown` was removed since its counterpart in the NVML library was removed

### Changes

* `enum_wrappers::device`
  * `TopologyLevel`
    * The `Cpu` variant was replaced by the `Node` variant
* The `UnexpectedVariant` error value is now an `i32` (previously `u32`)
* The `Device.remove()` method now takes additional parameters for more removal options

### Fixes

* Attempting to compile the library on macOS will now result in an informative error
* Methods that allocate `i8` vectors to be passed as cstrings now do so via the `vec!` macro rather than simply using `with_capacity`, meaning the length of the vector gets set appropriately
  * This did not cause a memory leak because we were just working with primitive types that don't have `Drop` impls, but it's nice to have fixed regardless

### Dependencies

* `error-chain`: `0.11.x -> 0.12.x`

## 0.4.1 (released 2019-04-08)

### Release Summary

The version was bumped in order to update the readme with the new information on Linux compilation. See the `sys` crates' changelog for details.

## 0.4.0 (released 2017-09-28)

### Release Summary

This is a small release that updates dependencies and makes a handful of changes for forward-compatibility purposes.

### Rust Version Support

This release **requires** and supports **Rust 1.20.0** or higher.

### Additions

* CI has been set up
  * All it can do is build the crate (no testing of any kind), but at least it's something.

### Changes

* `EventData::try_from()` is replaced by a `From<nvmlEventData_t>` impl as it can no longer error
  * This is because of the `from_bits_truncate()` usage described next
* Methods that deal with bitmasks now use the `from_bits_truncate()` constructor instead of `from_bits()`
  * This allows the wrapper, which is using bindings for NVML 8, to still accept bitmasks from future versions of NVML (such as NVML 9) that may have additional flags
  * `*_strict()` method counterparts are available for most such methods if you need them
* As a result of the `bitflags` update, flags are now associated constants
  * This means that, for instance, `nvml_wrapper::bitmasks::event::CLOCK_CHANGE` is now `nvml_wrapper::bitmasks::event::EventTypes::CLOCK_CHANGE`
* Imports were deglobbed in a number of places
* The `basic_usage` example now uses `pretty-bytes` instead of `number_prefix` to pretty-print bytes

### Dependencies

* `bitflags`: `0.9.x -> 1.0.x`
* `error-chain`: `0.10.x -> 0.11.x`

## 0.3.0 (released 2017-07-20)

### Release Summary

The major highlight of this release is the `high_level::event_loop` module, an interface to NVML's event capabilities. Only available on Linux platforms, this module provides you with the boilerplate necessary to quickly and easily watch for events on any number of devices, handling both errors and the events themselves. See the `event_loop` example in the examples folder at the root of the repository for more.

This release also marks the point at which no nightly features are required for any reason (meaning the removal of the `nightly` feature flag) and the addition of a couple examples demonstrating use of the crate.

### Rust Version Support

This release **requires** and supports **Rust 1.19.0** or higher.

### Additions

* Examples:
  * `basic_usage`
  * `event_loop`

* `enums::event`:
  * New file with the following:
    * `XidError`

* `high_level` module
  * This module will be the home of any high-level abstractions over the NVML API.
  * `high_level::event_loop`:
    * New file with the following:
      * `Event`
      * `EventLoop`
      * `EventLoopState`
      * `EventLoopProvider`, implemented for:
        * `NVML`

### Removals

* The `nightly` feature flag has been removed as nightly features are no longer required (`union` has been stabilized).

### Changes

* The `EventData.event_data` field is now an `Option<XidError>` instead of a `u64`
  * This was done to more strongly represent the field's presence via the type system (it is `None` for most events) and also to statically type the `Unknown` value.

* The `UnexpectedVariant` error now contains the enum value that could not be mapped to an enum variant defined in the wrapper.

* The project is now formatted via rustfmt as much as possible.

* Markdown headers now have two newlines after them, which is (to my knowledge) how they are supposed to be formatted.

## 0.2.0 (released 2017-06-08)

### Release Summary

The major highlight of this release is the `NvLink` struct, an interface to NVML's various NvLink-related functions. This release additionally corrects some issues / oversights in the wrapper and replaces Rust `enum`s with numerical constants for FFI use (see [rust-lang/rust#36927](https://github.com/rust-lang/rust/issues/36927)).

### Rust Version Support

This release **requires** and supports **Rust 1.18.0** or higher.

### Additions

* `NvLink` struct added and fleshed out, wrapping all NvLink-related functions.

  The `NvLink` struct can be obtained from a `Device` via `Device.link_wrapper_for()`. It provides a convenient interface to access the various NvLink-related functions with a common `link` value.
  * Functions wrapped:
    * `nvmlDeviceGetNvLinkState`
    * `nvmlDeviceGetNvLinkVersion`
    * `nvmlDeviceGetNvLinkCapability`
    * `nvmlDeviceGetNvLinkRemotePciInfo`
    * `nvmlDeviceGetNvLinkErrorCounter`
    * `nvmlDeviceResetNvLinkErrorCounters`
    * `nvmlDeviceSetNvLinkUtilizationControl`
    * `nvmlDeviceGetNvLinkUtilizationControl`
    * `nvmlDeviceGetNvLinkUtilizationCounter`
    * `nvmlDeviceFreezeNvLinkUtilizationCounter`
    * `nvmlDeviceResetNvLinkUtilizationCounter`
  * Tests written for all new functionality
    * While I cannot personally run these, they are useful both for static analysis and for those who are able to run the tests.

* `Device` struct:
  * `Device.link_wrapper_for()`

* `enums::nv_link`:
  * New file with the following:
    * `Counter`

* `struct_wrappers::nv_link`:
  * New file with the following:
    * `UtilizationControl`

* `structs::nv_link`:
  * New file with the following:
    * `UtilizationCounter`

* No-run tests were added for all methods that modify state. They are useful to statically verify whatever is statically verifiable.

### Changes

* `InsufficientSize` error now contains an `Option<usize>` instead of a `usize`.
  * This allows for `None` to be returned rather than `0` in the default case, and `Some(size)` to be specified explicitly if possible.
* `PciInfo`'s `pci_sub_system_id` field is now `Option<u32>` instead of `u32`.
  * This change was made in order to accommodate `NvLink.remote_pci_info()`. NVIDIA says that the sub system ID is not set in that function call, and we represent that with `None`.
  * As a direct result of this change, `PciInfo::try_from()` now takes a boolean arg, `sub_sys_id_present`, which is used to specify whether the `pci_sub_system_id` field should be `Some()` or `None`.

### Fixes

* Methods that should have taken `&mut self` now do so, including:
  * `Device.clear_cpu_affinity()`
  * `Device.reset_applications_clocks()`
  * `Device.clear_accounting_pids()`
  * `Device.clear_ecc_error_counts()`

* A method that should have taken `&self` now does:
  * `NVML.are_devices_on_same_board()`

* Numerical constants are now used in place of Rust `enum`s for safety reasons
  * For more info see [rust-lang/rust#36927](https://github.com/rust-lang/rust/issues/36927)
  * Many methods now return an `UnexpectedVariant` error where they did not previously

### Dependencies

* `bitflags`: `0.8.x -> 0.9.x`

## 0.1.0 (released 2017-05-17)

### Release Summary

Initial release wrapping the majority of the NVML API surface.

### Rust Version Support

This release **requires** and supports **Rust 1.17.0** or higher.
