// Copyright 2026 <https://github.com/ppetr/>
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#[cfg(feature = "use_libc")]
use libc;
use std::num::NonZeroI32;
use strum;

/// Derived from <https://github.com/abseil/abseil-cpp/blob/master/absl/status/status.h>. See the
/// link for more information.
///
/// (Copyright 2019 The Abseil Authors.)
///
/// Unlike `absl::StatusCode`, this enum only allows representing non-OK values.
#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
    strum::Display,
    strum::EnumString,
    strum::EnumIter,
    strum::FromRepr,
    strum::IntoStaticStr,
    strum::VariantArray,
)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
#[repr(i32)]
#[non_exhaustive]
pub enum ErrorCode {
    /// `Cancelled` (gRPC code "CANCELLED") indicates the operation was cancelled, typically by the
    /// caller.
    Cancelled = 1,

    /// `Unknown` (gRPC code "UNKNOWN") indicates an unknown error occurred. In general, more
    /// specific errors should be raised, if possible. Errors raised by APIs that do not return
    /// enough error information may be converted to this error.
    Unknown = 2,

    /// `InvalidArgument` (gRPC code "INVALID_ARGUMENT") indicates the caller specified an invalid
    /// argument, such as a malformed filename. Note that use of such errors should be narrowly
    /// limited to indicate the invalid nature of the arguments themselves. Errors with validly
    /// formed arguments that may cause errors with the state of the receiving system should be
    /// denoted with `FailedPrecondition` instead.
    InvalidArgument = 3,

    /// `DeadlineExceeded` (gRPC code "DEADLINE_EXCEEDED") indicates a deadline expired before the
    /// operation could complete. For operations that may change state within a system, this error
    /// may be returned even if the operation has completed successfully. For example, a successful
    /// response from a server could have been delayed long enough for the deadline to expire.
    DeadlineExceeded = 4,

    /// `NotFound` (gRPC code "NOT_FOUND") indicates some requested entity (such as a file or
    /// directory) was not found.
    ///
    /// `NotFound` is useful if a request should be denied for an entire class of users, such as
    /// during a gradual feature rollout or undocumented allow list. If a request should be denied
    /// for specific sets of users, such as through user-based access control, use
    /// `PermissionDenied` instead.
    NotFound = 5,

    /// `AlreadyExists` (gRPC code "ALREADY_EXISTS") indicates that the entity a caller attempted to
    /// create (such as a file or directory) is already present.
    AlreadyExists = 6,

    /// `PermissionDenied` (gRPC code "PERMISSION_DENIED") indicates that the caller does not have
    /// permission to execute the specified operation. Note that this error is different than an
    /// error due to an *un*authenticated user. This error code does not imply the request is valid
    /// or the requested entity exists or satisfies any other pre-conditions.
    ///
    /// `PermissionDenied` must not be used for rejections caused by exhausting some resource.
    /// Instead, use `ResourceExhausted` for those errors. `PermissionDenied` must not be used if
    /// the caller cannot be identified. Instead, use `Unauthenticated` for those errors.
    PermissionDenied = 7,

    /// `ResourceExhausted` (gRPC code "RESOURCE_EXHAUSTED") indicates some resource has been
    /// exhausted, perhaps a per-user quota, or perhaps the entire file system is out of space.
    ResourceExhausted = 8,

    /// `FailedPrecondition` (gRPC code "FAILED_PRECONDITION") indicates that the operation was
    /// rejected because the system is not in a state required for the operation's execution. For
    /// example, a directory to be deleted may be non-empty, an "rmdir" operation is applied to a
    /// non-directory, etc.
    ///
    /// Some guidelines that may help a service implementer in deciding between
    /// `FailedPrecondition`, `Aborted`, and `Unavailable`:
    ///
    ///  (a) Use `Unavailable` if the client can retry just the failing call.
    ///  (b) Use `Aborted` if the client should retry at a higher transaction level (such as when a
    ///      client-specified test-and-set fails, indicating the client should restart a
    ///      read-modify-write sequence).
    ///  (c) Use `FailedPrecondition` if the client should not retry until the system state has
    ///      been explicitly fixed. For example, if a "rmdir" fails because the directory is
    ///      non-empty, `FailedPrecondition` should be returned since the client should not retry
    ///      unless the files are deleted from the directory.
    FailedPrecondition = 9,

    /// `Aborted` (gRPC code "ABORTED") indicates the operation was aborted, typically due to a
    /// concurrency issue such as a sequencer check failure or a failed transaction.
    ///
    /// See the guidelines above for deciding between `FailedPrecondition`, `Aborted`, and
    /// `Unavailable`.
    Aborted = 10,

    /// `OutOfRange` (gRPC code "OUT_OF_RANGE") indicates the operation was attempted past the valid
    /// range, such as seeking or reading past an end-of-file.
    ///
    /// Unlike `InvalidArgument`, this error indicates a problem that may be fixed if the system
    /// state changes. For example, a 32-bit file system will generate `InvalidArgument` if asked
    /// to read at an offset that is not in the range [0,2^32-1], but it will generate `OutOfRange`
    /// if asked to read from an offset past the current file size.
    ///
    /// There is a fair bit of overlap between `FailedPrecondition` and `OutOfRange`.  We recommend
    /// using `OutOfRange` (the more specific error) when it applies so that callers who are
    /// iterating through a space can easily look for an `OutOfRange` error to detect when they are
    /// done.
    OutOfRange = 11,

    /// `Unimplemented` (gRPC code "UNIMPLEMENTED") indicates the operation is not implemented or
    /// supported in this service. In this case, the operation should not be re-attempted.
    Unimplemented = 12,

    /// `Internal` (gRPC code "INTERNAL") indicates an internal error has occurred and some
    /// invariants expected by the underlying system have not been satisfied. This error code is
    /// reserved for serious errors.
    Internal = 13,

    /// `Unavailable` (gRPC code "UNAVAILABLE") indicates the service is currently unavailable and
    /// that this is most likely a transient condition. An error such as this can be corrected by
    /// retrying with a backoff scheme. Note that it is not always safe to retry non-idempotent
    /// operations.
    ///
    /// See the guidelines above for deciding between `FailedPrecondition`, `Aborted`, and
    /// `Unavailable`.
    Unavailable = 14,

    /// `DataLoss` (gRPC code "DATA_LOSS") indicates that unrecoverable data loss or corruption has
    /// occurred. As this error is serious, proper alerting should be attached to errors such as
    /// this.
    DataLoss = 15,

    /// `Unauthenticated` (gRPC code "UNAUTHENTICATED") indicates that the request does not have
    /// valid authentication credentials for the operation. Correct the authentication and try
    /// again.
    Unauthenticated = 16,
}

impl ErrorCode {
    /// Adapted from `ErrnoToStatusCode` in
    /// <https://github.com/abseil/abseil-cpp/blob/master/absl/status/status.cc>
    #[cfg(feature = "use_libc")]
    pub fn from_errno(errno: i32) -> Option<ErrorCode> {
        match errno {
            0 => None,
            libc::EINVAL |        // Invalid argument
                libc::ENAMETOOLONG |  // Filename too long
                libc::E2BIG |         // Argument list too long
                libc::EDESTADDRREQ |  // Destination address required
                libc::EDOM |          // Mathematics argument out of domain of function
                libc::EFAULT |        // Bad address
                libc::EILSEQ |        // Illegal byte sequence
                libc::ENOPROTOOPT |   // Protocol not available
                libc::ENOTSOCK |      // Not a socket
                libc::ENOTTY |        // Inappropriate I/O control operation
                libc::EPROTOTYPE |    // Protocol wrong type for socket
                libc::ESPIPE =>       // Invalid seek
                Some(Self::InvalidArgument),
            libc::ETIMEDOUT => // Connection timed out
                Some(Self::DeadlineExceeded),
            libc::ENODEV |  // No such device
                libc::ENOENT |  // No such file or directory
                libc::ENOMEDIUM |  // No medium found
                libc::ENXIO |  // No such device or address
                libc::ESRCH => // No such process
                Some(Self::NotFound),
            libc::EEXIST |         // File exists
                libc::EADDRNOTAVAIL |  // Address not available
                libc::EALREADY |       // Connection already in progress
                libc::ENOTUNIQ => // Name not unique on network
                Some(Self::AlreadyExists),
            libc::EPERM |   // Operation not permitted
                libc::EACCES |  // Permission denied
                libc::ENOKEY |  // Required key not available
                libc::EROFS => // Read only file system
                Some(Self::PermissionDenied),
            libc::ENOTEMPTY |   // Directory not empty
                libc::EISDIR |      // Is a directory
                libc::ENOTDIR |     // Not a directory
                libc::EADDRINUSE |  // Address already in use
                libc::EBADF |       // Invalid file descriptor
                libc::EBADFD |  // File descriptor in bad state
                libc::EBUSY |    // Device or resource busy
                libc::ECHILD |   // No child processes
                libc::EISCONN |  // Socket is connected
                libc::EISNAM |  // Is a named type file
                libc::ENOTBLK |  // Block device required
                libc::ENOTCONN |  // The socket is not connected
                libc::EPIPE |     // Broken pipe
                libc::ESHUTDOWN |  // Cannot send after transport endpoint shutdown
                libc::ETXTBSY |  // Text file busy
                libc::EUNATCH => // Protocol driver not attached
                Some(Self::FailedPrecondition),
            libc::ENOSPC |  // No space left on device
                libc::EDQUOT |  // Disk quota exceeded
                libc::EMFILE |   // Too many open files
                libc::EMLINK |   // Too many links
                libc::ENFILE |   // Too many open files in system
                libc::ENOBUFS |  // No buffer space available
                libc::ENOMEM |   // Not enough space
                libc::EUSERS => // Too many users
                Some(Self::ResourceExhausted),
            libc::ECHRNG |  // Channel number out of range
                libc::EFBIG |      // File too large
                libc::EOVERFLOW |  // Value too large to be stored in data type
                libc::ERANGE =>    // Result too large
                Some(Self::OutOfRange),
            libc::ENOPKG |  // Package not installed
                libc::ENOSYS |        // Function not implemented
                libc::ENOTSUP |       // Operation not supported
                libc::EAFNOSUPPORT |  // Address family not supported
                libc::EPFNOSUPPORT |  // Protocol family not supported
                libc::EPROTONOSUPPORT |  // Protocol not supported
                libc::ESOCKTNOSUPPORT |  // Socket type not supported
                libc::EXDEV => // Improper link
                Some(Self::Unimplemented),
            libc::EAGAIN |  // Resource temporarily unavailable
                libc::ECOMM |  // Communication error on send
                libc::ECONNREFUSED |  // Connection refused
                libc::ECONNABORTED |  // Connection aborted
                libc::ECONNRESET |    // Connection reset
                libc::EINTR |         // Interrupted function call
                libc::EHOSTDOWN |  // Host is down
                libc::EHOSTUNREACH |  // Host is unreachable
                libc::ENETDOWN |      // Network is down
                libc::ENETRESET |     // Connection aborted by network
                libc::ENETUNREACH |   // Network unreachable
                libc::ENOLCK |        // No locks available
                libc::ENOLINK |       // Link has been severed
                libc::ENONET => // Machine is not on the network
                Some(Self::Unavailable),
            libc::EDEADLK |  // Resource deadlock avoided
                libc::ESTALE => // Stale file handle
                Some(Self::Aborted),
            libc::ECANCELED =>  // Operation cancelled
                Some(Self::Cancelled),
            _ => Some(Self::Unknown),
        }
    }

    /// If `err` contains a `raw_os_error()`, return it converted into an `ErrorCode`.
    /// Otherwise returns `None`.
    #[cfg(feature = "use_libc")]
    pub fn from_raw_os_error(err: &std::io::Error) -> Option<ErrorCode> {
        err.raw_os_error().and_then(Self::from_errno)
    }
}

impl From<ErrorCode> for NonZeroI32 {
    fn from(code: ErrorCode) -> Self {
        NonZeroI32::new(code as i32).expect(&format!(
            "The enum value of an ErrorCode must be nonzero, but got '{:?}'",
            code
        ))
    }
}

/// If a value matches one of the defined error codes, returns it as an `ErrorCode`, otherwise
/// returns a `()` error.
///
/// For converting from strings, use the `std::str::FromStr` instance.
impl TryFrom<i32> for ErrorCode {
    type Error = ();

    fn try_from(code: i32) -> Result<Self, ()> {
        Self::from_repr(code).ok_or(())
    }
}
