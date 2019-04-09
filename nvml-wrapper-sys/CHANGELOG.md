# nvml-wrapper-sys Changelog

This file describes the changes / additions / fixes between bindings releases.

## 0.3.1 (released 2019-04-08)

### Release Summary

Improvements were made to the build script:

* An attempt will be made to locate the directory containing `libnvidia-ml.so` and it will be automatically added to the locations that the library is being searched for in. Thanks @SunDoge!
* The script will now display a helpful error message if compilation is attempted on macOS.

## 0.3.0 (released 2017-07-20)

### Release Summary

The `nightly` feature flag has been removed as unions are now available on stable Rust.

### Rust Version Support

This release **requires** and supports **Rust 1.19.0** or higher.

## 0.2.0 (released 2017-06-08)

### Release Summary

Rust `enum`s were removed in favor of numerical constants for C enums. This was done for safety reasons; see [rust-lang/rust#36927](https://github.com/rust-lang/rust/issues/36927) for more information.

### Changes

* Rust `enum`s replaced with numerical constants
* Replaced `::std::os::raw::x` paths with `raw::x` paths for readability
* Removed `Copy` and `Clone` from structs where they did not make sense
  * Forgot about this before

## 0.1.0 (released 2017-05-17)

### Release Summary

Initial release providing bindings for the entirety of the NVML API as well as nightly-only feature usage behind a feature flag.
