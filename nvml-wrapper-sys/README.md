# nvml-wrapper-sys

[![Crates.io version](https://img.shields.io/crates/v/nvml-wrapper-sys.svg?style=flat-square)](https://crates.io/crates/nvml-wrapper-sys)
[![Crates.io downloads](https://img.shields.io/crates/d/nvml-wrapper-sys.svg?style=flat-square)](https://crates.io/crates/nvml-wrapper-sys)
[![Docs.rs docs](https://docs.rs/nvml-wrapper-sys/badge.svg)](https://docs.rs/nvml-wrapper-sys)

Rust bindings for the
[NVIDIA Management Library](https://developer.nvidia.com/nvidia-management-library-nvml)
(NVML), a C-based programmatic interface for monitoring and managing various states within
NVIDIA (primarily Tesla) GPUs.

It is intended to be a platform for building 3rd-party applications, and is also the
underlying library for NVIDIA's nvidia-smi tool.

NVML supports the following platforms:

* Windows
  * Windows Server 2008 R2 64-bit
  * Windows Server 2012 R2 64-bit
  * Windows 7 64-bit
  * Windows 8 64-bit
  * Windows 10 64-bit
* Linux
  * 64-bit
  * 32-bit
* Hypervisors
  * Windows Server 2008R2/2012 Hyper-V 64-bit
  * Citrix XenServer 6.2 SP1+
  * VMware ESX 5.1/5.5

And the following products:

* Full Support
  * Tesla products Fermi architecture and up
  * Quadro products Fermi architecture and up
  * GRID products Kepler architecture and up
  * Select GeForce Titan products
* Limited Support
  * All GeForce products Fermi architecture and up

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