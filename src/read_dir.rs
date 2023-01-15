use crate::{DirEntryView, ViewKind};
use std::{fmt, io};

/// Iterator over the entries in a directory.
///
/// This corresponds to [`std::fs::ReadDir`].
///
/// There is no `from_std` method, as `std::fs::ReadDir` doesn't provide a way
/// to construct a `ReadDir` without opening directories by ambient paths.
pub struct ReadDirView {
    pub(crate) read_dir: cap_std::fs::ReadDir,
    pub(crate) view_kind: ViewKind,
}

impl Iterator for ReadDirView {
    type Item = io::Result<DirEntryView>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.read_dir.next().map(|entry| {
            entry.map(|entry| DirEntryView {
                entry,
                view_kind: self.view_kind,
            })
        })
    }
}

impl fmt::Debug for ReadDirView {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.read_dir.fmt(f)
    }
}
