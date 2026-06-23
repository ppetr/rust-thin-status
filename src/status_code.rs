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

/// Derived from https://github.com/abseil/abseil-cpp/blob/master/absl/status/status.h (Copyright
/// 2019 The Abseil Authors). See the link for more information.
///
/// Unlike `absl::StatusCode`, this only allows representing non-OK values.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(i32)]
#[non_exhaustive]
pub enum ErrorCode {
    /// StatusCode::kCancelled
    ///
    /// kCancelled (gRPC code "CANCELLED") indicates the operation was cancelled, typically by the
    /// caller.
    Cancelled = 1,

    /// StatusCode::kUnknown
    ///
    /// kUnknown (gRPC code "UNKNOWN") indicates an unknown error occurred. In general, more
    /// specific errors should be raised, if possible. Errors raised by APIs that do not return
    /// enough error information may be converted to this error.
    Unknown = 2,

    /// StatusCode::kInvalidArgument
    ///
    /// kInvalidArgument (gRPC code "INVALID_ARGUMENT") indicates the caller specified an invalid
    /// argument, such as a malformed filename. Note that use of such errors should be narrowly
    /// limited to indicate the invalid nature of the arguments themselves. Errors with validly
    /// formed arguments that may cause errors with the state of the receiving system should be
    /// denoted with `kFailedPrecondition` instead.
    InvalidArgument = 3,

    /// StatusCode::kDeadlineExceeded
    ///
    /// kDeadlineExceeded (gRPC code "DEADLINE_EXCEEDED") indicates a deadline expired before the
    /// operation could complete. For operations that may change state within a system, this error
    /// may be returned even if the operation has completed successfully. For example, a successful
    /// response from a server could have been delayed long enough for the deadline to expire.
    DeadlineExceeded = 4,

    /// StatusCode::kNotFound
    ///
    /// kNotFound (gRPC code "NOT_FOUND") indicates some requested entity (such as a file or
    /// directory) was not found.
    ///
    /// `kNotFound` is useful if a request should be denied for an entire class of users, such as
    /// during a gradual feature rollout or undocumented allow list. If a request should be denied
    /// for specific sets of users, such as through user-based access control, use
    /// `kPermissionDenied` instead.
    NotFound = 5,

    /// StatusCode::kAlreadyExists
    ///
    /// kAlreadyExists (gRPC code "ALREADY_EXISTS") indicates that the entity a caller attempted to
    /// create (such as a file or directory) is already present.
    AlreadyExists = 6,

    /// StatusCode::kPermissionDenied
    ///
    /// kPermissionDenied (gRPC code "PERMISSION_DENIED") indicates that the caller does not have
    /// permission to execute the specified operation. Note that this error is different than an
    /// error due to an *un*authenticated user. This error code does not imply the request is valid
    /// or the requested entity exists or satisfies any other pre-conditions.
    ///
    /// `kPermissionDenied` must not be used for rejections caused by exhausting some resource.
    /// Instead, use `kResourceExhausted` for those errors. `kPermissionDenied` must not be used if
    /// the caller cannot be identified. Instead, use `kUnauthenticated` for those errors.
    PermissionDenied = 7,

    /// StatusCode::kResourceExhausted
    ///
    /// kResourceExhausted (gRPC code "RESOURCE_EXHAUSTED") indicates some resource has been
    /// exhausted, perhaps a per-user quota, or perhaps the entire file system is out of space.
    ResourceExhausted = 8,

    /// StatusCode::kFailedPrecondition
    ///
    /// kFailedPrecondition (gRPC code "FAILED_PRECONDITION") indicates that the operation was
    /// rejected because the system is not in a state required for the operation's execution. For
    /// example, a directory to be deleted may be non-empty, an "rmdir" operation is applied to a
    /// non-directory, etc.
    ///
    /// Some guidelines that may help a service implementer in deciding between
    /// `kFailedPrecondition`, `kAborted`, and `kUnavailable`:
    ///
    ///  (a) Use `kUnavailable` if the client can retry just the failing call.
    ///  (b) Use `kAborted` if the client should retry at a higher transaction level (such as when a
    ///      client-specified test-and-set fails, indicating the client should restart a
    ///      read-modify-write sequence).
    ///  (c) Use `kFailedPrecondition` if the client should not retry until the system state has
    ///      been explicitly fixed. For example, if a "rmdir" fails because the directory is
    ///      non-empty, `kFailedPrecondition` should be returned since the client should not retry
    ///      unless the files are deleted from the directory.
    FailedPrecondition = 9,

    /// StatusCode::kAborted
    ///
    /// kAborted (gRPC code "ABORTED") indicates the operation was aborted, typically due to a
    /// concurrency issue such as a sequencer check failure or a failed transaction.
    ///
    /// See the guidelines above for deciding between `kFailedPrecondition`, `kAborted`, and
    /// `kUnavailable`.
    Aborted = 10,

    /// StatusCode::kOutOfRange
    ///
    /// kOutOfRange (gRPC code "OUT_OF_RANGE") indicates the operation was attempted past the valid
    /// range, such as seeking or reading past an end-of-file.
    ///
    /// Unlike `kInvalidArgument`, this error indicates a problem that may be fixed if the system
    /// state changes. For example, a 32-bit file system will generate `kInvalidArgument` if asked
    /// to read at an offset that is not in the range [0,2^32-1], but it will generate `kOutOfRange`
    /// if asked to read from an offset past the current file size.
    ///
    /// There is a fair bit of overlap between `kFailedPrecondition` and `kOutOfRange`.  We
    /// recommend using `kOutOfRange` (the more specific error) when it applies so that callers who
    /// are iterating through a space can easily look for an `kOutOfRange` error to detect when they
    /// are done.
    OutOfRange = 11,

    /// StatusCode::kUnimplemented
    ///
    /// kUnimplemented (gRPC code "UNIMPLEMENTED") indicates the operation is not implemented or
    /// supported in this service. In this case, the operation should not be re-attempted.
    Unimplemented = 12,

    /// StatusCode::kInternal
    ///
    /// kInternal (gRPC code "INTERNAL") indicates an internal error has occurred and some
    /// invariants expected by the underlying system have not been satisfied. This error code is
    /// reserved for serious errors.
    Internal = 13,

    /// StatusCode::kUnavailable
    ///
    /// kUnavailable (gRPC code "UNAVAILABLE") indicates the service is currently unavailable and
    /// that this is most likely a transient condition. An error such as this can be corrected by
    /// retrying with a backoff scheme. Note that it is not always safe to retry non-idempotent
    /// operations.
    ///
    /// See the guidelines above for deciding between `kFailedPrecondition`, `kAborted`, and
    /// `kUnavailable`.
    Unavailable = 14,

    /// StatusCode::kDataLoss
    ///
    /// kDataLoss (gRPC code "DATA_LOSS") indicates that unrecoverable data loss or corruption has
    /// occurred. As this error is serious, proper alerting should be attached to errors such as
    /// this.
    DataLoss = 15,

    /// StatusCode::kUnauthenticated
    ///
    /// kUnauthenticated (gRPC code "UNAUTHENTICATED") indicates that the request does not have
    /// valid authentication credentials for the operation. Correct the authentication and try
    /// again.
    Unauthenticated = 16,
}
