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
#[cfg(feature = "use_any")]
pub(crate) mod any {
    use google_cloud_wkt::Any;

    #[cfg(feature = "use_any")]
    #[derive(Clone, Debug, Default, PartialEq)]
    pub struct Details {
        pub details: Vec<Any>,
    }

    impl Details {
        pub fn is_empty(&self) -> bool {
            return self.details.is_empty();
        }

        pub fn get(&self) -> &[Any] {
            &self.details
        }
    }

    impl std::fmt::Display for Details {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let details = self.get();
            if details.is_empty() {
                Ok(())
            } else {
                write!(f, " {:?}", &details)
            }
        }
    }
}

#[cfg(not(feature = "use_any"))]
pub mod any {
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
    pub struct Details;

    impl std::fmt::Display for Details {
        fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            Ok(())
        }
    }

    impl Details {
        pub fn is_empty(&self) -> bool {
            true
        }
    }
}
