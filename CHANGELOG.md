# nvml-wrapper Changelog

This file describes the changes / additions / fixes between wrapper releases.

## 0.3.0 (released 2017-07-20)

### Release Summary

The major highlight of this release is the `high_level::event_loop` module, an interface to NVML's event capabilities. Only available on Linux platforms, this module provides you with the boilerplate necessary to quickly and easily watch for events on any number of devices, handling both errors and the events themselves. See the `event_loop` example in the examples folder at the root of the repository for more.

### Rustc Support

This release **requires** and supports **rustc 1.19.0** or higher.

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

### Changes

* The `EventData.event_data` field is now an `Option<XidError>` instead of a `u64`
  * This was done to more strongly represent the field's presence via the type system (it is `None` for most events) and also to statically type the `Unknown` value.

* The `UnexpectedVariant` error now contains the enum value that could not be mapped to an enum variant defined in the wrapper.

* The project is now formatted via rustfmt as much as possible.

* Markdown headers now have two newlines after them, which is (to my knowledge) how they are supposed to be formatted.

## 0.2.0 (released 2017-06-08)

### Release Summary

The major highlight of this release is the `NvLink` struct, an interface to NVML's various NvLink-related functions. This release additionally corrects some issues / oversights in the wrapper and replaces Rust `enum`s with numerical constants for FFI use (see [rust-lang/rust#36927](https://github.com/rust-lang/rust/issues/36927)).

### Rustc Support

This release **requires** and supports **rustc 1.18.0** or higher.

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

* Bitflags: `0.8.x -> 0.9.x`

## 0.1.0 (released 2017-05-17)

### Release Summary

Initial release wrapping the majority of the NVML API surface.

### Rustc Support

This release **requires** and supports **rustc 1.17.0** or higher.
