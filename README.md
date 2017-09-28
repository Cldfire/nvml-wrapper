# nvml-wrapper

[![Docs.rs docs](https://docs.rs/nvml-wrapper/badge.svg)](https://docs.rs/nvml-wrapper)
[![Crates.io version](https://img.shields.io/crates/v/nvml-wrapper.svg?style=flat-square)](https://crates.io/crates/nvml-wrapper)
[![Crates.io downloads](https://img.shields.io/crates/d/nvml-wrapper.svg?style=flat-square)](https://crates.io/crates/nvml-wrapper)
[![Travis build status](https://img.shields.io/travis/Cldfire/nvml-wrapper/master.svg?style=flat-square)](https://travis-ci.org/Cldfire/nvml-wrapper)
[![AppVeyor build status](https://img.shields.io/appveyor/ci/Cldfire/nvml-wrapper/master.svg?style=flat-square)](https://ci.appveyor.com/project/Cldfire/nvml-wrapper)

A complete, safe, and ergonomic Rust wrapper for the
[NVIDIA Management Library](https://developer.nvidia.com/nvidia-management-library-nvml)
(NVML), a C-based programmatic interface for monitoring and managing various states within
NVIDIA (primarily Tesla) GPUs.

```rust
let nvml = NVML::init()?;
// Get the first `Device` (GPU) in the system
let device = nvml.device_by_index(0)?;

let brand = device.brand()?; // GeForce on my system
let fan_speed = device.fan_speed()?; // Currently 17% on my system
let power_limit = device.enforced_power_limit()?; // 275k milliwatts on my system
let encoder_util = device.encoder_utilization()?; // Currently 0 on my system; Not encoding anything
let memory_info = device.memory_info()?; // Currently 1.63/6.37 GB used on my system

// ... and there's a whole lot more you can do. Everything in NVML is wrapped and ready to go
```

NVML is intended to be a platform for building 3rd-party applications, and is
also the underlying library for NVIDIA's nvidia-smi tool.

## Compilation

The NVML library comes with the NVIDIA drivers and is essentially present on any
system with a functioning NVIDIA graphics card. The compilation steps vary
between Windows and Linux, however.

### Windows

The NVML library dll can be found at `%ProgramW6432%\NVIDIA Corporation\NVSMI\`
(which is `C:\Program Files\NVIDIA Corporation\NVSMI\` on my machine). You will need
to add this folder to your `PATH` in order to have everything work properly at
runtime; alternatively, place a copy of the dll in the same folder as your executable.

### Linux

The NVML library can be found at `/usr/lib/nvidia-<driver-version>/libnvidia-ml.so`; on my system with driver version 375.51 installed, this puts the library at
`/usr/lib/nvidia-375/libnvidia-ml.so`. You will need to create a symbolic link:

```bash
sudo ln -s /usr/lib/nvidia-<driver-version>/libnvidia-ml.so /usr/lib
```

## NVML Support

This wrapper has been developed against and is currently supporting NVML version
8. Each new version of NVML is guaranteed to be backwards-compatible according
to NVIDIA, so this wrapper should continue to work without issue regardless of
NVML version bumps.

## Rust Version Support

Currently supports Rust 1.20.0 or greater. The target version is the **latest**
stable version; I do not intend to pin to an older one at any time.

## Cargo Features

The `serde` feature can be toggled on in order to `#[derive(Serialize, Deserialize)]`
for every NVML data structure.

## License

Licensed under either of

* Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
