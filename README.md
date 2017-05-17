# nvml-wrapper

[![Crates.io version](https://img.shields.io/crates/v/nvml-wrapper.svg?style=flat-square)](https://crates.io/crates/nvml-wrapper)
[![Crates.io downloads](https://img.shields.io/crates/d/nvml-wrapper.svg?style=flat-square)](https://crates.io/crates/nvml-wrapper)
[![Docs.rs docs](https://docs.rs/nvml-wrapper/badge.svg)](https://docs.rs/nvml-wrapper)

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
// (except for a few (~9) NvLink-related items that I will get to soon)
```

NVML is intended to be a platform for building 3rd-party applications, and is
also the underlying library for NVIDIA's nvidia-smi tool.

## Compilation

This dependency should be a no-effort addition to your `Cargo.toml`. The NVML library
comes with the NVIDIA drivers and is essentially present on any system with a
functioning NVIDIA graphics card.

The `nvml-wrapper-sys` crate should take care of correctly finding and linking to
the NVML library on both Windows and Linux; if it does not, please file an issue.

## Rustc Support

Currently supports rustc 1.17.0 or greater. The target version is the **latest**
stable version; I do not intend to pin to an older one at any time.

A small amount of NVML features involve dealing with untagged unions over FFI; a
rustc nightly-only type is used in order to facilitate this. If you require use
of the nightly-only functionality, compile with the `nightly` feature toggled on
(and of course, with a nightly compiler):

```bash
cargo build --features "nightly"
```

## Cargo Features

The `nightly` feature can be toggled on to enable nightly-only features; read above.

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
