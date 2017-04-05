// `error_chain` recursion limit
#![recursion_limit = "1024"]

#[macro_use]
extern crate error_chain;
extern crate nvml_sys as ffi;

use ffi::bindings::nvmlReturn_t;
use ffi::bindings::nvmlReturn_t::*;

error_chain! {
    foreign_links {
        IntoStringError(::std::ffi::IntoStringError);
        Utf8Error(::std::str::Utf8Error);
        NulError(::std::ffi::NulError);
    }
    errors {
        /// Bits that did not correspond to a flag were encountered whilst attempting to
        /// interpret them as bitflags.
        ///
        /// This error is specific to this Rust wrapper.
        IncorrectBits {
            description("Bits that did not correspond to a flag were encountered whilst attempting \
                        to interpret them as bitflags")
        }
        /// An unexpected enum variant was encountered.
        ///
        /// This error is specific to this Rust wrapper. It is used to represent the
        /// possibility that an enum variant that seems to be only used internally by 
        /// the NVML lib gets returned by a function call. While I don't believe it will
        /// ever happen, it's best to be complete.
        UnexpectedVariant {
            description("An unexpected enum variant was encountered (wrapper error).")
        }
        /// NVML was not first initialized with `nvmlInit()`.
        Uninitialized {
            description("NVML was not first initialized with `nvmlInit()`.")
        }
        /// A supplied argument is invalid.
        InvalidArg {
            description("A supplied argument is invalid.")
        }
        /// The requested operation is not available on the target device.
        NotSupported {
            description("The requested operation is not available on the target device.")
        }
        /// The current user does not have permission for the operation.
        NoPermission {
            description("The current user does not have permission for the operation.")
        }
        /// This error is deprecated on the part of the NVML lib itself and should 
        /// not be encountered. Multiple initializations are now allowed through refcounting.
        AlreadyInitialized {
            description("This error is deprecated on the part of the NVML lib itself and should \
                        not be encountered. Multiple initializations are now allowed through refcounting.")
        }
        /// A query to find and object was unsuccessful.
        NotFound {
            description("A query to find and object was unsuccessful.")
        }
        /// An input argument is not large enough.
        InsufficientSize {
            description("An input argument is not large enough.")
        }
        /// A device's external power cables are not properly attached.
        InsufficientPower {
            description("A device's external power cables are not properly attached.")
        }
        /// NVIDIA driver is not loaded.
        DriverNotLoaded {
            description("NVIDIA driver is not loaded.")
        }
        /// User provided timeout passed.
        Timeout {
            description("User provided timeout passed.")
        }
        /// NVIDIA kernel detected an interrupt issue with a GPU.
        IrqIssue {
            description("NVIDIA kernel detected an interrupt issue with a GPU.")
        }
        /// NVML Shared Library couldn't be found or loaded.
        LibraryNotFound {
            description("NVML Shared Library couldn't be found or loaded.")
        }
        /// Local version of NVML doesn't implement this function.
        FunctionNotFound {
            description("Local version of NVML doesn't implement this function.")
        }
        /// infoROM is corrupted.
        CorruptedInfoROM {
            description("infoROM is corrupted.")
        }
        /// The GPU has fallen off the bus or has otherwise become inaccessible.
        GpuLost {
            description("The GPU has fallen off the bus or has otherwise become inaccessible.")
        }
        /// The GPU requires a reset before it can be used again.
        ResetRequired {
            description("The GPU requires a reset before it can be used again.")
        }
        /// The GPU control device has been blocked by the operating system/cgroups.
        OperatingSystem {
            description("The GPU control device has been blocked by the operating system/cgroups.")
        }
        /// RM detects a driver/library version mismatch.
        LibRmVersionMismatch {
            description("RM detects a driver/library version mismatch.")
        }
        /// An operation cannot be performed because the GPU is currently in use.
        InUse {
            description("An operation cannot be performed because the GPU is currently in use.")
        }
        /// No data.
        NoData {
            description("No data.")
        }
        /// An internal driver error occured.
        Unknown {
            description("An internal driver error occured.")
        }
    }
}

/// `?` enabler for nvmlReturn_t types.
#[doc(hidden)]
pub fn nvml_try(code: nvmlReturn_t) -> Result<()> {
    match code {
        NVML_SUCCESS                        => Ok(()),
        NVML_ERROR_UNINITIALIZED            => Err(Error::from_kind(ErrorKind::Uninitialized)),
        NVML_ERROR_INVALID_ARGUMENT         => Err(Error::from_kind(ErrorKind::InvalidArg)),
        NVML_ERROR_NOT_SUPPORTED            => Err(Error::from_kind(ErrorKind::NotSupported)),
        NVML_ERROR_NO_PERMISSION            => Err(Error::from_kind(ErrorKind::NoPermission)),
        NVML_ERROR_ALREADY_INITIALIZED      => Err(Error::from_kind(ErrorKind::AlreadyInitialized)),
        NVML_ERROR_NOT_FOUND                => Err(Error::from_kind(ErrorKind::NotFound)),
        NVML_ERROR_INSUFFICIENT_SIZE        => Err(Error::from_kind(ErrorKind::InsufficientSize)),
        NVML_ERROR_INSUFFICIENT_POWER       => Err(Error::from_kind(ErrorKind::InsufficientPower)),
        NVML_ERROR_DRIVER_NOT_LOADED        => Err(Error::from_kind(ErrorKind::DriverNotLoaded)),
        NVML_ERROR_TIMEOUT                  => Err(Error::from_kind(ErrorKind::Timeout)),
        NVML_ERROR_IRQ_ISSUE                => Err(Error::from_kind(ErrorKind::IrqIssue)),
        NVML_ERROR_LIBRARY_NOT_FOUND        => Err(Error::from_kind(ErrorKind::LibraryNotFound)),
        NVML_ERROR_FUNCTION_NOT_FOUND       => Err(Error::from_kind(ErrorKind::FunctionNotFound)),
        NVML_ERROR_CORRUPTED_INFOROM        => Err(Error::from_kind(ErrorKind::CorruptedInfoROM)),
        NVML_ERROR_GPU_IS_LOST              => Err(Error::from_kind(ErrorKind::GpuLost)),
        NVML_ERROR_RESET_REQUIRED           => Err(Error::from_kind(ErrorKind::ResetRequired)),
        NVML_ERROR_OPERATING_SYSTEM         => Err(Error::from_kind(ErrorKind::OperatingSystem)),
        NVML_ERROR_LIB_RM_VERSION_MISMATCH  => Err(Error::from_kind(ErrorKind::LibRmVersionMismatch)),
        NVML_ERROR_IN_USE                   => Err(Error::from_kind(ErrorKind::InUse)),
        NVML_ERROR_NO_DATA                  => Err(Error::from_kind(ErrorKind::NoData)),
        NVML_ERROR_UNKNOWN                  => Err(Error::from_kind(ErrorKind::Unknown)),
    }
}