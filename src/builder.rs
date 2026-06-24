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
use std::borrow::Cow;
use std::fmt;
use std::num::NonZeroI32;

use crate::{FullStatus, ThinStatus};

/// Allows convenient construction of `ThinStatus` instances.
pub struct ThinStatusBuilder<'a> {
    pub(crate) full: FullStatus,
    pub(crate) message: Cow<'a, str>,
}

impl<'a> ThinStatusBuilder<'a> {
    /// Constructs a builder from an error code (which can be, and often is `ErrorCode`).
    pub fn new<C: Into<NonZeroI32>>(code: C) -> Self {
        Self {
            full: FullStatus {
                code: code.into(),
                details: Default::default(),
            },
            message: Cow::Borrowed(""),
        }
    }

    /// Sets the error code to `code`.
    pub fn code<C: Into<NonZeroI32>>(mut self, code: C) -> Self {
        self.full.code = code.into();
        self
    }

    /// Sets the message to `message`, discarding any previous one. The builder captures only a
    /// reference to it to avoid copying.
    pub fn message(mut self, message: &'a str) -> Self {
        self.message = Cow::Borrowed(message);
        self
    }

    /// Appends a `google_cloud_wkt::Any` object to the list of details.
    #[cfg(feature = "use_any")]
    pub fn add_detail(mut self, detail: google_cloud_wkt::Any) -> Self {
        self.full.details.details.push(detail);
        self
    }

    /// Sets the internal `Vec<google_cloud_wkt::Any>` to `details`, discarding any previous vector.
    #[cfg(feature = "use_any")]
    pub fn details(mut self, details: Vec<google_cloud_wkt::Any>) -> Self {
        self.full.details.details = details;
        self
    }

    pub fn build(self) -> ThinStatus {
        ThinStatus::from_builder(self)
    }
}

impl From<ThinStatusBuilder<'_>> for ThinStatus {
    fn from(builder: ThinStatusBuilder) -> Self {
        builder.build()
    }
}

/// Allows convenient appending to the text message. If it contains just a reference passed to
/// `message`, the reference is copied into a new `String` that can be appended to.
impl<'a> fmt::Write for ThinStatusBuilder<'a> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.message.to_mut().push_str(s);
        Ok(())
    }
}
