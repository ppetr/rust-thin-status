# thin-status

*Disclaimer: This is a personal project. The views, code, and opinions expressed here are my own and do not represent those of my current or past employers.*

A low-overhead, production-grade error status type for Rust, heavily inspired
by [Abseil's `absl::Status`](https://abseil.io/docs/cpp/guides/status).

It is designed specifically for high-throughput network programming and
RPC frameworks where minimizing memory overhead, optimizing cache
locality, and eliminating happy-path heap allocations are critical
performance requirements.

## Features

-   **Single-Pointer Memory Footprint**: `ThinStatus` occupies exactly
    one pointer word (the size of a `usize`). Niche optimization
    guarantees that `Option<ThinStatus>` and `Result<(), ThinStatus>`
    fit into that same single word without increasing data size.
-   **Zero-Allocation for Inlined Codes**: Standard production error
    codes matching `ErrorCode` that fit within the tagged pointer
    boundaries require absolutely zero heap allocation.
-   **Production RPC and gRPC Alignment**: Built directly around
    canonical RPC status models. Includes out-of-the-box support for
    translating standard POSIX `errno` codes (via `libc`) and converting
    to/from `google-cloud-rpc` models.
-   **Extensible Structured Details**: Supports attaching rich,
    production-tier diagnostic messages and structured arbitrary
    payloads via `google_cloud_wkt::Any` (gated under the `use_any`
    feature) when an error path demands complex telemetry.

## Usage

``` rust
use thin_status::{ThinStatus, ErrorCode};
use std::num::NonZeroI32;

// Zero-allocation status for inline RPC error codes
let status = ThinStatus::from_code(ErrorCode::NotFound);
assert_eq!(std::mem::size_of_val(&status), std::mem::size_of::<usize>());

// Rich errors (with messages) cleanly fallback to thread-safe heap allocation
let rpc_err = ThinStatus::builder(ErrorCode::PermissionDenied)
    .message("Token expired or missing required IAM scope.")
    .build();

// Directly maps to native POSIX subsystem failures
if let Some(err_code) = ErrorCode::from_errno(2) { // ENOENT
    let status = ThinStatus::from_code(err_code);
}
```
