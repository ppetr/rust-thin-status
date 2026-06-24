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
mod any_details;
mod builder;
mod status_code;
mod thin_arc_or_int;

// use google_cloud_rpc::model::Status;
use std::error::Error;
use std::num::NonZeroI32;
use thin_arc_or_int::{IsizeInPtr, ThinArcOrInt};
use triomphe::ThinArc;

use crate::any_details::any::Details;

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(not(feature = "use_any"), derive(Copy, Eq, PartialOrd, Ord, Hash))]
struct FullStatus {
    code: NonZeroI32,
    details: Details,
}

impl TryFrom<FullStatus> for IsizeInPtr {
    type Error = FullStatus;

    fn try_from(value: FullStatus) -> Result<IsizeInPtr, FullStatus> {
        value.code.get().try_into().map_err(|_| value)
    }
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(not(feature = "use_any"), derive(Eq, PartialOrd, Ord, Hash))]
pub struct ThinStatus {
    // The wrapped `isize` integer is always non-zero.
    // The `&[u8]` is always in UTF-8.
    thin: ThinArcOrInt<FullStatus, u8>,
}

impl ThinStatus {
    pub fn builder<'a, C: Into<NonZeroI32>>(code: C) -> builder::ThinStatusBuilder<'a> {
        builder::ThinStatusBuilder::new(code)
    }

    pub(crate) fn from_builder(builder: builder::ThinStatusBuilder) -> Self {
        if builder.message.is_empty() && builder.details.is_empty() {
            return Self::from_code(builder.code);
        }
        ThinStatus {
            thin: ThinArcOrInt::from_arc(ThinArc::from_header_and_slice(
                FullStatus {
                    code: builder.code,
                    details: builder.details,
                },
                builder.message.as_bytes(),
            )),
        }
    }

    pub fn from_code(code: NonZeroI32) -> Self {
        ThinStatus {
            thin: ThinArcOrInt::from_convertible(FullStatus {
                code: code,
                details: Default::default(),
            }),
        }
    }

    fn code(&self) -> NonZeroI32 {
        match self.thin.as_isize() {
            Some(code) => unsafe { NonZeroI32::new_unchecked(code as i32) },
            None => {
                let val = &self.thin.as_arc();
                let val = val.expect("ThinArcOrInt contains neither code nor arc");
                val.header.header.code
            }
        }
    }

    fn message(&self) -> &str {
        match self.thin.as_arc() {
            None => "",
            Some(arc) => unsafe { str::from_utf8_unchecked(&arc.slice) },
        }
    }

    #[cfg(feature = "use_any")]
    pub fn details(&self) -> &[google_cloud_wkt::Any] {
        let &arc = &self.thin.as_arc();
        arc.map_or::<&[google_cloud_wkt::Any], _>(&[], |t| &t.header.header.details.get())
    }
}

impl std::fmt::Display for ThinStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // TODO: Use text status code when possible.
        self.code().get().fmt(f)?;
        let msg = self.message();
        if !msg.is_empty() {
            write!(f, ": {}", msg)?;
        }
        #[cfg(feature = "use_any")]
        {
            if let Some(t) = &self.thin.as_arc() {
                t.header.header.details.fmt(f)?;
            }
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
    fn test_from_error_code() {
        // Values exactly at the boundary and within MAX_THIN should not allocate a ThinArc.
        let status: ThinStatus = status_code::ErrorCode::NotFound.into();
        assert_eq!(<NonZeroI32 as Into<i32>>::into(status.code()), 5);
        assert_eq!(status.message(), "");
        #[cfg(feature = "use_any")]
        assert_eq!(status.details(), &[]);
        assert!(status.thin.has_number());
        assert_eq!(format!("{}", status), "5");
    }

    #[test]
    fn test_from_code_within_max_thin() {
        // Values exactly at the boundary and within MAX_THIN should not allocate a ThinArc.
        let status_pos = ThinStatus::from_code(MAX_THIN);
        assert_eq!(status_pos.code(), MAX_THIN);
        assert_eq!(status_pos.message(), "");
        assert!(status_pos.thin.has_number());

        let status_neg = ThinStatus::from_code(-MAX_THIN);
        assert_eq!(status_neg.code(), -MAX_THIN);
        assert_eq!(status_neg.message(), "");
        assert!(status_neg.thin.has_number());

        let status_normal = ThinStatus::from_code(non_zero(42));
        assert_eq!(status_normal.code(), non_zero(42));
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
            assert_eq!(status_overflow.code(), larger);
            assert_eq!(status_overflow.message(), "");

            let status_underflow = ThinStatus::from_code(-larger);
            assert_eq!(status_underflow.code(), -larger);
            assert_eq!(status_underflow.message(), "");
            assert!(status_overflow.thin.has_ref());
            assert!(status_underflow.thin.has_ref());
        }

        // Extreme i32 values (these will surely exceed MAX_THIN if MAX_THIN is smaller).
        let status_max = ThinStatus::from_code(NonZeroI32::MAX);
        assert_eq!(status_max.code(), NonZeroI32::MAX);
        let status_min = ThinStatus::from_code(NonZeroI32::MIN);
        assert_eq!(status_min.code(), NonZeroI32::MIN);
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
        assert_eq!(<NonZeroI32 as Into<i32>>::into(status.code()), 5);
        assert_eq!(status.message(), "message");
        #[cfg(feature = "use_any")]
        assert_eq!(status.details(), &[]);
        assert!(status.thin.has_ref());
        assert_eq!(format!("{}", status), "5: message");
    }

    #[cfg(feature = "use_any")]
    #[test]
    fn test_with_details() {
        let detail =
            google_cloud_wkt::Any::from_msg(&google_cloud_wkt::Duration::clamp(123, 456)).unwrap();
        let status = ThinStatus::builder(status_code::ErrorCode::NotFound)
            .add_detail(detail.clone())
            .build();
        assert_eq!(<NonZeroI32 as Into<i32>>::into(status.code()), 5);
        assert_eq!(status.message(), "");
        assert_eq!(status.details(), vec![detail]);
        assert!(status.thin.has_ref());
        let formatted = format!("{}", status);
        assert!(formatted.starts_with("5 [Any("));
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
        assert_eq!(status1.code(), status2.code());
        assert_eq!(status1.message(), status2.message());

        let _status_different = ThinStatus::from_code(non_zero(13));
        assert_ne!(status1, _status_different);
    }
}
