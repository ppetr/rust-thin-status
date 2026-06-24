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

use crate::any_details::any::Details;
use crate::ThinStatus;

pub struct ThinStatusBuilder<'a> {
    pub(crate) code: NonZeroI32,
    pub(crate) message: Cow<'a, str>,
    pub(crate) details: Details,
}

impl<'a> ThinStatusBuilder<'a> {
    pub fn new<C: Into<NonZeroI32>>(code: C) -> Self {
        Self {
            code: code.into(),
            message: Cow::Borrowed(""),
            details: Default::default(),
        }
    }

    pub fn code<C: Into<NonZeroI32>>(mut self, code: C) -> Self {
        self.code = code.into();
        self
    }

    pub fn message(mut self, message: &'a str) -> Self {
        self.message = Cow::Borrowed(message);
        self
    }

    #[cfg(feature = "use_any")]
    pub fn add_detail(mut self, detail: google_cloud_wkt::Any) -> Self {
        self.details.details.push(detail);
        self
    }

    #[cfg(feature = "use_any")]
    pub fn details(mut self, details: Vec<google_cloud_wkt::Any>) -> Self {
        self.details.details = details;
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

impl<'a> fmt::Write for ThinStatusBuilder<'a> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        // .to_mut() provede alokaci na String pouze při prvním zápisu
        self.message.to_mut().push_str(s);
        Ok(())
    }
}
