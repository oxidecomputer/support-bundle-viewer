// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Utilities to inspect support bundles

mod bundle_accessor;
mod dashboard;
mod index;

pub use dashboard::run_dashboard;

pub use bundle_accessor::BoxedFileAccessor;
pub use bundle_accessor::FileAccessor;
pub use bundle_accessor::LocalFileAccess;
pub use bundle_accessor::SupportBundleAccessor;
pub use index::SupportBundleIndex;

// TODO:
// - Remove references to "nexus_client", move that code into omdb
// - Make equivalent for "external client", add that code to CLI
// - Change the input to "run_dashboard". Probably need to accept a
// "SupportBundleAccessor".
