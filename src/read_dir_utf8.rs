use crate::{DirEntryViewUtf8, ViewKind};
use std::{fmt, io};

/// Iterator over the entries in a directory.
///
/// This corresponds to [`std::fs::ReadDir`].
///
/// There is no `from_std` method, as `std::fs::ReadDir` doesn't provide a way
/// to construct a `ReadDir` without opening directories by ambient paths.
pub struct ReadDirViewUtf8 {
    pub(crate) read_dir: cap_std::fs_utf8::ReadDir,
    pub(crate) view_kind: ViewKind,
}

impl Iterator for ReadDirViewUtf8 {
    type Item = io::Result<DirEntryViewUtf8>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.read_dir.next().map(|entry| {
            entry.map(|entry| DirEntryViewUtf8 {
                entry,
                view_kind: self.view_kind,
            })
        })
    }
}

impl fmt::Debug for ReadDirViewUtf8 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.read_dir.fmt(f)
    }
}
