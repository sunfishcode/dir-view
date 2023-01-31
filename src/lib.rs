#![cfg_attr(doc_cfg, feature(doc_cfg, doc_auto_cfg))]

mod dir;
mod dir_entry;
#[cfg(feature = "fs_utf8")]
mod dir_entry_utf8;
#[cfg(feature = "fs_utf8")]
mod dir_utf8;
mod read_dir;
#[cfg(feature = "fs_utf8")]
mod read_dir_utf8;

pub use cap_std::{ambient_authority, AmbientAuthority};

pub use dir::DirView;
pub use dir_entry::DirEntryView;
#[cfg(feature = "fs_utf8")]
pub use dir_entry_utf8::DirEntryViewUtf8;
#[cfg(feature = "fs_utf8")]
pub use dir_utf8::DirViewUtf8;
pub use read_dir::ReadDirView;
#[cfg(feature = "fs_utf8")]
pub use read_dir_utf8::ReadDirViewUtf8;

/// The kind of a view.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum ViewKind {
    /// Expose everything. The same as `cap_std::fs::Dir` itself.
    Full,

    /// Expose a readonly view. Creating, renaming, or deleting new files or
    /// directories is not permitted, and files can only be opened in readonly
    /// mode.
    Readonly,
}
