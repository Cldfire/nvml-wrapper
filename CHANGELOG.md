# nvml-wrapper Changelog

This file describes the changes / additions / fixes between wrapper releases, tracked in a loose version of the [keep a changelog](https://keepachangelog.com/en/1.0.0/) format.

## [Unreleased]

### Added

* `ScopeId`
* `FieldValueRequest`

### Changed

* `Device`
  * Methods
    * `field_values_for()`
* `FieldValueSample`

## [0.10.0] (released 2024-02-10)

Updates for NVML 12.2.

### Added

* `Device`
  * Methods
    * `pcie_link_speed()`
* `DeviceArchitecture`
  * Variants
    * `Ada`
    * `Hopper`

### Changed

* `enums::device`
  * `PcieLinkMaxSpeed`
    * Renamed variants to reflect the fact that they appear to represent per-lane transfers/second, not multi-lane throughput

### Internal

* Bumped MSRV to 1.60.0 for usage of namespaced features
* Bumped crate edition to `2021`
* Removed `rust-hook` from development workflow
* Started building crate in CI on `macos-latest`
* Added a script to find unwrapped function names
* Vendored header files are now excluded from repo stats

### Rust Version Support

The MSRV of this release is 1.60.0. This is for usage of namespaced features.

### Dependencies

* `bitflags`: `1.3` -> `2.4.0`
* `libloading`: `0.7.0` -> `0.8.1`
* `wrapcenum-derive`: `0.4.0` -> `0.4.1`

## [0.9.0] (released 2023-01-20)

### Release Summary

Bug fixes, improvements, and updates for NVML 11.8.

### Added

* Wrapper methods are now annotated with the `#[doc(alias = "...")]` attribute to make them searchable by C function name in rustdoc ([#31](https://github.com/Cldfire/nvml-wrapper/pull/31) - @arpankapoor)
* Some older function versions are now wrapped and available for use behind the `legacy-functions` crate feature
  * `running_compute_processes_v2`
  * `running_graphics_processes_v2`

### Changed

* `enum_wrappers::device`
  * `Brand`
    * Added new variants ([#35](https://github.com/Cldfire/nvml-wrapper/pull/35) - @nemosupremo)

### Fixed

* `Device`
  * `running_compute_processes()`
    * Fixed count handling ([#36](https://github.com/Cldfire/nvml-wrapper/pull/36) - @jjyyxx)
      * This bug would have caused five blank process information structs to be returned in addition to actual process information if there were any running compute processes.

### Internal

* SPDX expressions in `Cargo.toml` files have been updated to avoid using the now-deprecated slash syntax ([#32](https://github.com/Cldfire/nvml-wrapper/pull/32) - @KisaragiEffective)

### Rust Version Support

The MSRV of this release continues to be 1.51.0.

## [0.8.0] (released 2022-05-26)

### Release Summary

Updates for the latest version of NVML (11.6 update 2). More wrapped methods!

### Added

* `Device`:
  * Methods:
    * `set_mem_locked_clocks()` ([#27](https://github.com/Cldfire/nvml-wrapper/pull/27) - @benrod3k)
    * `reset_mem_locked_clocks()`
    * `num_cores()`
    * `num_fans()`
    * `bus_type()`
    * `power_source()`
    * `architecture()`
    * `pcie_link_max_speed()`
    * `memory_bus_width()`
    * `irq_num`

### Changed

* The `NVML` struct has been renamed to `Nvml` ([#22](https://github.com/Cldfire/nvml-wrapper/pull/22) - @TheJltres)
* The `basic_usage` example now prints your device's architecture and number of CUDA cores
* Some methods on `Nvml` have been renamed:
  * `Nvml::blacklist_device_count()` -> `Nvml::excluded_device_count()`
  * `Nvml::blacklist_device_info()` -> `Nvml::excluded_device_info()`
* Some struct wrappers have been modified:
  * `BlacklistDeviceInfo` renamed to `ExcludedDeviceInfo`
  * `ProcessInfo` gained new fields:
    * `gpu_instance_id`
    * `compute_instance_id`
* `Device`:
  * `Device::name()` now creates a buffer sized to the new `NVML_DEVICE_NAME_V2_BUFFER_SIZE` constant
  * `Device::running_compute_processes()` and `Device::running_graphics_processes()` now allocate a bit of headroom in case the process count increases between when they make a call to figure out how much to allocate and when they make a call to get data
  * `Device::set_gpu_locked_clocks()` now takes in `GpuLockedClocksSetting` allowing for both numeric ranges and symbolic boundaries

### Rust Version Support

The MSRV of this release is 1.51.0. This is needed for const generics and the ability to implement `Debug` for arrays of any size.

### Dependencies

* `libloading`: `0.6.6` -> `0.7.0`

### Internal

* Re-organized repo using a workspace
* Tests that can't be run on my machine are now ignored with the `#[ignore]` attribute. The `test-local` feature has been removed as a result
* The `Cargo.lock` file is now committed to the repo

## [0.7.0] (released 2020-12-06)

### Release Summary

Dynamically loading the NVML library at runtime is here! Thanks to a [new bindgen feature](https://github.com/rust-lang/rust-bindgen/pull/1846) that landed recently, `nvml-wrapper-sys` now has regenerated bindings that make use of [the `libloading` crate](https://github.com/nagisa/rust_libloading) and don't require linking to NVML at compile-time.

This means it's now possible to drop NVIDIA-related features in your code at runtime on systems that don't have relevant hardware.

### Rust Version Support

The MSRV of this release is 1.42.0.

### Added

* `NVML` struct:
  * Added methods:
    * `builder`
* `NvmlBuilder` struct
  * A builder struct that provides further flexibility in how NVML is initialized
* `bitmasks`
  * Added events:
    * `POWER_SOURCE_CHANGE`
    * `MIG_CONFIG_CHANGE`
  * Added init flags:
    * `NO_ATTACH`
* `Device` struct:
  * Added methods:
    * `new` (replaces the `From<nvmlDevice_t>` impl)
    * `nvml`
* `EventSet` struct:
  * Added methods:
    * `new` (replaces the `From<nvmlEventSet_t>` impl)
* `EventData` struct:
  * Added methods:
    * `new` (replaces the `From<nvmlEventData_t>` impl)
* `Unit` struct:
  * Added methods:
    * `new` (replaces the `From<nvmlUnit_t>` impl)
    * `nvml`
* `NvmlError`
  * Added variants:
    * `LibloadingError`
    * `FailedToLoadSymbol`

### Changed

* `InitFlags` now implements `Default`
* `NvmlError` and `NvmlErrorWithSource` no longer implement `Clone`, `Eq`, or `PartialEq`
  * They can no longer implement these traits because `libloading::Error` doesn't implement them
  * As a result of this change, `FieldValueSample` no longer implements `Clone` or `PartialEq`
* `NvmlError::UnexpectedVariant` now contains a `u32` instead of an `i32` due to a change in the types of the generated bindings

### Dependencies

* `libloading`: new dependency on `0.6.6`
* `static_assertions`: new dependency on `1.1.x`

## [0.6.0] (released 2020-06-15)

### Release Summary

This release was focused on cleanup and migrating a crate originally written in 2016 to modern Rust conventions. Notably, `error-chain` has been ripped out of the crate entirely and replaced with an error enum that implements `std::error::Error`.

Additionally, the `wrapcenum-derive` dependency (a derive macro used to simplify API generation internally) has been completely re-written and now depends on the `1.0` releases of `syn` and `quote`. There are no user-facing changes as a result of this rewrite, but your crate's dependency tree will likely be very pleased.

### Rust Version Support

The MSRV of this release is 1.42.0.

### Changes

* `nvml-wrapper` updated to Rust 2018 edition
* `nvml-wrapper-sys` updated to Rust 2018 edition
* `wrapcenum-derive` has been re-written and now depends on modern versions of `syn` and `quote`
* Removed `#[inline]` attribute from all functions
* Merged methods to get raw handles from structs into a single method
  * `Device.handle()`, `EventSet.handle()`, `Unit.handle()`
* Ripped `error-chain` out of crate entirely
  * Replaced with `NvmlError` enum that implements `std::error::Error`
* Migrated most `try_from()` methods to implementations of `std::convert::TryFrom`
* Migrated all `try_into_c()` methods to implementations of `std::convert::TryInto`
* Modernized some of the example code

### Dependencies

* `bitflags`: `1.0.x -> 1.2.x`
* `thiserror`: new dependency on `1.0.x`
* `error-chain`: no longer a dependency

## [0.5.0] (released 2019-09-10)

### Release Summary

A long time in the works, 0.5.0 contains the last two years of my extremely sporadic work wrapping some of the new functionality provided in NVML since version 8 alongside a handful of small fixes and improvements.

### Additions

* An import library (`nvml.lib`) has been added that enables compilation using the MSVC toolchain on Windows.
* The `basic_usage` example now prints the system's CUDA driver version.
* `bitmasks`
  * Added throttle reasons:
    * `SW_THERMAL_SLOWDOWN`
    * `HW_THERMAL_SLOWDOWN`
    * `HW_POWER_BRAKE_SLOWDOWN`
    * `DISPLAY_CLOCK_SETTING`
  * Added `device::FbcFlags`
  * Added `InitFlags`
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
  * Added the `FbcSessionType` enum
* `Device` struct:
  * Added methods:
    * `cuda_compute_capability`
    * `encoder_capacity`
    * `encoder_stats`
    * `encoder_sessions`
    * `encoder_sessions_count`
    * `fbc_stats`
    * `fbc_sessions_info`
    * `fbc_session_count`
    * `process_utilization_stats`
    * `total_energy_consumption`
    * `field_values_for`
    * `set_gpu_locked_clocks`
    * `reset_gpu_locked_clocks`
* `error`
  * Added errors:
    * `InsufficientMemory`
    * `VgpuEccNotSupported`
* `NVML` struct:
  * Added methods:
    * `init_with_flags`
    * `sys_cuda_driver_version`
    * `blacklist_device_count`
    * `blacklist_device_info`
* `structs::device`
  * Added structs:
    * `EncoderStats`
    * `CudaComputeCapability`
    * `FieldId`
    * `RetiredPage`
* `struct_wrappers`
  * Added struct:
    * `BlacklistDeviceInfo`
* `struct_wrappers::device`
  * Added structs:
    * `EncoderSessionInfo`
    * `ProcessUtilizationSample`
    * `FieldValueSample`
    * `FbcStats`
    * `FbcSessionInfo`
* `lib.rs`
  * Added functions:
    * `cuda_driver_version_major`
    * `cuda_driver_version_minor`

### Removals

* `bitmasks`
  * `ThrottleReasons::Unknown` was removed since its counterpart in the NVML library was removed

### Changes

* `enum_wrappers::device`
  * `TopologyLevel`
    * The `Cpu` variant was replaced by the `Node` variant
* The `UnexpectedVariant` error value is now an `i32` (previously `u32`)
* The `Device.remove()` method now takes additional parameters for more removal options
* The `Device.fan_speed()` method now takes a fan index to allow reading the speed of different fans
* The `Device.retired_pages()` method now returns the timestamps for each page's retirment along with their addresses
* The `NVML.sys_cuda_driver_version()` method now errors if the CUDA shared library cannot be found

### Fixes

* Methods that allocate `i8` vectors to be passed as cstrings now do so via the `vec!` macro rather than simply using `with_capacity`, meaning the length of the vector gets set appropriately
  * This did not cause a memory leak because we were just working with primitive types that don't have `Drop` impls, but it's nice to have fixed regardless

### Dependencies

* `error-chain`: `0.11.x -> 0.12.x`

## [0.4.1] (released 2019-04-08)

### Release Summary

The version was bumped in order to update the readme with the new information on Linux compilation. See the `sys` crates' changelog for details.

### Fixes

* Attempting to compile the library on macOS will now result in an informative error

## [0.4.0] (released 2017-09-28)

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

## [0.3.0] (released 2017-07-20)

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

## [0.2.0] (released 2017-06-08)

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

## [0.1.0] (released 2017-05-17)

### Release Summary

Initial release wrapping the majority of the NVML API surface.

### Rust Version Support

This release **requires** and supports **Rust 1.17.0** or higher.

[Unreleased]: https://github.com/Cldfire/nvml-wrapper/compare/v0.10.0...HEAD
[0.10.0]: https://github.com/Cldfire/nvml-wrapper/compare/v0.9.0...v0.10.0
[0.9.0]: https://github.com/Cldfire/nvml-wrapper/compare/v0.8.0...v0.9.0
[0.8.0]: https://github.com/Cldfire/nvml-wrapper/compare/v0.7.0...v0.8.0
[0.7.0]: https://github.com/Cldfire/nvml-wrapper/compare/v0.6.0...v0.7.0
[0.6.0]: https://github.com/Cldfire/nvml-wrapper/compare/v0.5.0...v0.6.0
[0.5.0]: https://github.com/Cldfire/nvml-wrapper/compare/v0.4.1...v0.5.0
[0.4.1]: https://github.com/Cldfire/nvml-wrapper/compare/v0.4.0...v0.4.1
[0.4.0]: https://github.com/Cldfire/nvml-wrapper/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/Cldfire/nvml-wrapper/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/Cldfire/nvml-wrapper/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/Cldfire/nvml-wrapper/releases/tag/v0.1.0
