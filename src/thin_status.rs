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

#[cfg(feature = "use_cloud_rpc")]
use google_cloud_rpc::model as cloud_rpc;
use std::error::Error;
use std::num::NonZeroI32;

use crate::any_details::any::Details;
use crate::builder;
use crate::status_code;
use crate::thin_arc_or_int::{IsizeInPtr, ThinArcOrInt};

/// Holds a status code, and if enabled by feature `use_any`, also a list of `google_cloud_wkt.Any`
/// instances that can provide structured details.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(not(feature = "use_any"), derive(Copy, Eq, PartialOrd, Ord, Hash))]
pub(crate) struct FullStatus {
    pub(crate) code: NonZeroI32,
    pub(crate) details: Details,
}

/// Coverts `FullStatus` to a code embedded inside a pointer as a tagged integer if:
/// - There are no `google_cloud_wkt.Any` details attached.
/// - The integer fits in `IsizeInPtr`.
///
/// Otherwise returns the very same instance.
impl TryFrom<FullStatus> for IsizeInPtr {
    type Error = FullStatus;

    fn try_from(value: FullStatus) -> Result<IsizeInPtr, FullStatus> {
        if value.details.is_empty() {
            value.code.get().try_into().map_err(|_| value)
        } else {
            Err(value)
        }
    }
}

/// Holds a non-OK status code (in most cases `ErrorCode`), a descriptive string message, and
/// optionally (if enabled by feature `use_any`) also `[google_cloud_wkt::Any]`.
///
/// It occupies only a single pointer word, which either wraps a tagged integer code (if there is
/// neither message nor details), or a `ThinArc` pointer that holds a more complex value.
/// Furthermore its representation is never `nullptr`, therefore `Option<ThinStatus>` and
/// `Result<(), ThinStatus>` occupy just a single memory word.
///
/// Note that `Copy`, `Eq`, `PartialOrd` and `Hash` are only provided (derived) when feature
/// `use_any` is disabled, since the `Any` datatype doesn't provide them.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(not(feature = "use_any"), derive(Eq, PartialOrd, Ord, Hash))]
pub struct ThinStatus {
    // The wrapped `isize` integer is always non-zero.
    // The `&[u8]` is always in UTF-8.
    thin: ThinArcOrInt<FullStatus, u8>,
}

impl ThinStatus {
    /// Constructs a builder that allows convenient creation of `ThinStatus` instances.
    pub fn builder<'a, C: Into<NonZeroI32>>(code: C) -> builder::ThinStatusBuilder<'a> {
        builder::ThinStatusBuilder::new(code)
    }

    /// Converts a builder into a new instance.
    ///
    /// This involves at most one heap allocation (if there is a message and/or `google_cloud_wkt.Any`.
    pub(crate) fn from_builder(builder: builder::ThinStatusBuilder) -> Self {
        ThinStatus {
            thin: ThinArcOrInt::from_convertible(builder.full, builder.message.as_bytes()),
        }
    }

    /// A utility function to create a status with just a raw code. If it fits, it'll be stored just
    /// as a tagged integer inside an internal pointer.
    pub fn from_code<C: Into<NonZeroI32>>(code: C) -> Self {
        ThinStatus {
            thin: ThinArcOrInt::from_convertible(
                FullStatus {
                    code: code.into(),
                    details: Default::default(),
                },
                &[],
            ),
        }
    }

    /// Returns the stored error code, or `None` if the raw numerical value doesn't match any of the
    /// `ErrorCode` values.
    pub fn code(&self) -> Option<status_code::ErrorCode> {
        self.code_raw().get().try_into().ok()
    }

    /// Convenience wrapper to `code()` that converts an unknown error to `ErrorCode::Unknown`.
    pub fn code_or_unknown(&self) -> status_code::ErrorCode {
        self.code().unwrap_or(status_code::ErrorCode::Unknown)
    }

    /// Returns the raw numerical error code.
    pub fn code_raw(&self) -> NonZeroI32 {
        match self.thin.as_isize() {
            Some(code) => unsafe { NonZeroI32::new_unchecked(code as i32) },
            None => {
                let val = &self.thin.as_arc();
                let val = val.expect("ThinArcOrInt contains neither code nor arc");
                val.header.header.code
            }
        }
    }

    /// Returns the message attached to the status (if any).
    pub fn message(&self) -> &str {
        match self.thin.as_arc() {
            None => "",
            Some(arc) => unsafe { str::from_utf8_unchecked(&arc.slice) },
        }
    }

    /// Returns the list of `google_cloud_wkt::Any` objects attached to the status.
    #[cfg(feature = "use_any")]
    pub fn details(&self) -> &[google_cloud_wkt::Any] {
        let &arc = &self.thin.as_arc();
        arc.map_or::<&[google_cloud_wkt::Any], _>(&[], |t| &t.header.header.details.get())
    }

    /// Converts a `ThinStatus` into a `google_cloud_rpc::Status`.
    #[cfg(feature = "use_cloud_rpc")]
    pub fn to_cloud_rpc(&self) -> cloud_rpc::Status {
        self.into()
    }

    /// If a given `Status` is non-OK, converts it into `ThinStatus`.
    #[cfg(feature = "use_cloud_rpc")]
    pub fn try_from_cloud_rpc(status: cloud_rpc::Status) -> Result<Self, ()> {
        status.try_into()
    }

    /// If a given `Status` is non-OK, converts it into `ThinStatus`.
    /// Details of type `google_cloud_wkt.Any` are cloned.
    #[cfg(feature = "use_cloud_rpc")]
    pub fn try_from_cloud_rpc_ref(status: &cloud_rpc::Status) -> Result<Self, ()> {
        status.try_into()
    }
}

/// If the contained error code corresponds to an `ErrorCode` (`code()` returns `Some(...)`), it's
/// printed as an upper-case string (such as `NOT_FOUND`); otherwise just as a number.
/// This is followed by a colon, space and the contained message (if there is any).
impl std::fmt::Display for ThinStatus {
    /// If the `alternate` flag is set, always prints the error code as a number.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let code = self.code_raw().get();
        if let Some(error_code) = status_code::ErrorCode::try_from(code).ok() {
            error_code.fmt(f)?;
        } else {
            code.fmt(f)?;
        }
        let msg = self.message();
        if !msg.is_empty() {
            write!(f, ": {}", msg)?;
        }
        Ok(())
    }
}

impl Error for ThinStatus {}

impl From<status_code::ErrorCode> for ThinStatus {
    fn from(code: status_code::ErrorCode) -> Self {
        Self::from_code(NonZeroI32::from(code))
    }
}

/// If a given `Status` is non-OK, converts it into `ThinStatus`.
#[cfg(feature = "use_cloud_rpc")]
impl TryFrom<cloud_rpc::Status> for ThinStatus {
    type Error = ();

    fn try_from(status: cloud_rpc::Status) -> Result<Self, ()> {
        let code: NonZeroI32 = status.code.try_into().map_err(|_| ())?;
        Ok(Self::builder(code)
            .message(&status.message)
            .details(status.details)
            .build())
    }
}

/// If a given `Status` is non-OK, converts it into `ThinStatus`.
/// Details of type `google_cloud_wkt.Any` are cloned.
#[cfg(feature = "use_cloud_rpc")]
impl TryFrom<&cloud_rpc::Status> for ThinStatus {
    type Error = ();

    fn try_from(status: &cloud_rpc::Status) -> Result<Self, ()> {
        let code: NonZeroI32 = status.code.try_into().map_err(|_| ())?;
        Ok(Self::builder(code)
            .message(&status.message)
            .details(status.details.clone())
            .build())
    }
}

/// Converts a `ThinStatus` into a `google_cloud_rpc::Status`.
#[cfg(feature = "use_cloud_rpc")]
impl From<&ThinStatus> for cloud_rpc::Status {
    fn from(status: &ThinStatus) -> Self {
        let mut result = Self::new();
        result.code = status.code_raw().get();
        result.message = status.message().to_string();
        result.details = status.details().to_vec();
        result
    }
}

#[cfg(test)]
mod thin_status_tests {
    use super::*;
    use std::fmt::Write;

    /// Represents the maximal `i32` value that can be stored inside the integer variant of
    /// `ThinArcOrInt`.
    const MAX_THIN: NonZeroI32 = NonZeroI32::new(if IsizeInPtr::MAX as u64 <= i32::MAX as u64 {
        IsizeInPtr::MAX as i32
    } else {
        i32::MAX
    })
    .unwrap();

    const fn non_zero(value: i32) -> NonZeroI32 {
        NonZeroI32::new(value).unwrap()
    }

    #[test]
    fn test_size_optimization() {
        // Ensure that ThinStatus takes up exactly the size of a single pointer.
        assert_eq!(
            std::mem::size_of::<ThinStatus>(),
            std::mem::size_of::<usize>()
        );
    }

    #[test]
    fn test_from_error_code_raw() {
        // Values exactly at the boundary and within MAX_THIN should not allocate a ThinArc.
        let status: ThinStatus = status_code::ErrorCode::NotFound.into();
        assert_eq!(<NonZeroI32 as Into<i32>>::into(status.code_raw()), 5);
        assert_eq!(status.message(), "");
        #[cfg(feature = "use_any")]
        assert_eq!(status.details(), &[]);
        assert!(status.thin.has_number());
        assert_eq!(format!("{}", status), "NOT_FOUND");
    }

    #[test]
    fn test_from_code_within_max_thin() {
        // Values exactly at the boundary and within MAX_THIN should not allocate a ThinArc.
        let status_pos = ThinStatus::from_code(MAX_THIN);
        assert_eq!(status_pos.code_raw(), MAX_THIN);
        assert_eq!(status_pos.message(), "");
        assert!(status_pos.thin.has_number());

        let status_neg = ThinStatus::from_code(-MAX_THIN);
        assert_eq!(status_neg.code_raw(), -MAX_THIN);
        assert_eq!(
            status_neg.code_or_unknown(),
            status_code::ErrorCode::Unknown
        );
        assert_eq!(status_neg.message(), "");
        assert!(status_neg.thin.has_number());

        let status_normal = ThinStatus::from_code(non_zero(42));
        assert_eq!(status_normal.code_raw(), non_zero(42));
        assert_eq!(status_normal.message(), "");
        assert!(status_normal.thin.has_number());
    }

    /// Values outside MAX_THIN must force a ThinArc allocation.
    #[test]
    fn test_from_code_outside_max_thin() {
        // If MAX_THIN equals i32::MAX, testing values beyond the boundary
        // is not meaningful due to i32 overflow, so we check if it is smaller first.
        if MAX_THIN.get() < NonZeroI32::MAX.into() {
            let larger: NonZeroI32 = non_zero(MAX_THIN.get() + 1);
            let status_overflow = ThinStatus::from_code(larger);
            assert_eq!(status_overflow.code_raw(), larger);
            assert_eq!(status_overflow.message(), "");

            let status_underflow = ThinStatus::from_code(-larger);
            assert_eq!(status_underflow.code_raw(), -larger);
            assert_eq!(status_underflow.message(), "");
            assert!(status_overflow.thin.has_ref());
            assert!(status_underflow.thin.has_ref());
        }

        // Extreme i32 values (these will surely exceed MAX_THIN if MAX_THIN is smaller).
        let status_max = ThinStatus::from_code(NonZeroI32::MAX);
        assert_eq!(status_max.code_raw(), NonZeroI32::MAX);
        let status_min = ThinStatus::from_code(NonZeroI32::MIN);
        assert_eq!(status_min.code_raw(), NonZeroI32::MIN);
        if MAX_THIN.get() < NonZeroI32::MAX.into() {
            assert!(status_max.thin.has_ref());
            assert!(status_min.thin.has_ref());
        }
    }

    #[test]
    fn test_from_error_code_and_message() {
        let mut builder = ThinStatus::builder(status_code::ErrorCode::NotFound);
        write!(builder, "message").expect("write! failed");
        let status = builder.build();
        assert_eq!(<NonZeroI32 as Into<i32>>::into(status.code_raw()), 5);
        assert_eq!(status.code(), Some(status_code::ErrorCode::NotFound));
        assert_eq!(status.code_or_unknown(), status_code::ErrorCode::NotFound);
        assert_eq!(status.message(), "message");
        #[cfg(feature = "use_any")]
        assert_eq!(status.details(), &[]);
        assert!(status.thin.has_ref());
        assert_eq!(format!("{}", status), "NOT_FOUND: message");
    }

    #[cfg(feature = "use_any")]
    #[test]
    fn test_with_details() {
        let detail =
            google_cloud_wkt::Any::from_msg(&google_cloud_wkt::Duration::clamp(123, 456)).unwrap();
        let status = ThinStatus::builder(status_code::ErrorCode::NotFound)
            .add_detail(detail.clone())
            .build();
        assert_eq!(<NonZeroI32 as Into<i32>>::into(status.code_raw()), 5);
        assert_eq!(status.message(), "");
        assert_eq!(status.details(), vec![detail]);
        assert!(status.thin.has_ref());
        let formatted = format!("{:#?}", status);
        assert!(formatted.contains("Any"));
        assert!(formatted.contains("type.googleapis.com/google.protobuf.Duration"));
        assert!(formatted.contains("123"));
    }

    #[test]
    fn test_clone_and_equality() {
        let status1 = ThinStatus::builder(non_zero(13))
            .message("Permission Denied")
            .build();
        let status2 = status1.clone();

        assert_eq!(status1, status2);
        assert_eq!(status1.code_raw(), status2.code_raw());
        assert_eq!(status1.message(), status2.message());

        let _status_different = ThinStatus::from_code(non_zero(13));
        assert_ne!(status1, _status_different);
    }

    #[cfg(feature = "use_cloud_rpc")]
    #[test]
    fn test_cloud_rpc_status_conversions() {
        assert!(
            ThinStatus::try_from(cloud_rpc::Status::new()).is_err(),
            "OK status shouldn't be convertible to ThinStatus"
        );

        let detail =
            google_cloud_wkt::Any::from_msg(&google_cloud_wkt::Duration::clamp(123, 456)).unwrap();
        let status = ThinStatus::builder(status_code::ErrorCode::NotFound)
            .message("Be yourself! Everyone else is already taken.")
            .add_detail(detail)
            .build();
        assert_eq!(
            ThinStatus::try_from(cloud_rpc::Status::from(&status)).as_ref(),
            Ok(&status)
        );

        assert_eq!(
            ThinStatus::try_from(&cloud_rpc::Status::from(&status)),
            Ok(status)
        );
    }
}
