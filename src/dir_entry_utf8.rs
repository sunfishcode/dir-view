use crate::{DirViewUtf8, ViewKind};
use cap_std::fs_utf8::{DirEntry, File, FileType, Metadata, OpenOptions};
#[cfg(not(windows))]
use rustix::fs::DirEntryExt;
use std::{fmt, io};

/// Entries returned by the `ReadDir` iterator.
///
/// This corresponds to [`std::fs::DirEntry`].
///
/// Unlike `std::fs::DirEntry`, this API has no `DirEntry::path`, because
/// absolute paths don't interoperate well with the capability model.
///
/// There is a `file_name` function, however there are also `open`,
/// `open_with`, `open_dir`, `remove_file`, and `remove_dir` functions for
/// opening or removing the entry directly, which can be more efficient and
/// convenient.
///
/// There is no `from_std` method, as `std::fs::DirEntry` doesn't provide a
/// way to construct a `DirEntry` without opening directories by ambient paths.
pub struct DirEntryViewUtf8 {
    pub(crate) entry: DirEntry,
    pub(crate) view_kind: ViewKind,
}

impl DirEntryViewUtf8 {
    /// Open the file for reading.
    #[inline]
    pub fn open(&self) -> io::Result<File> {
        self.entry.open()
    }

    /// Open the file with the given options.
    #[inline]
    pub fn open_with(&self, options: &OpenOptions) -> io::Result<File> {
        // Override any flag that allows writing.
        let mut options = options.clone();
        match self.view_kind {
            ViewKind::Full => {}
            ViewKind::Readonly => {
                // Override any flag that allows writing.
                options.append(false);
                options.truncate(false);
                options.write(false);
                options.create(false);
                options.create_new(false);
            }
        }
        self.entry.open_with(&options)
    }

    /// Open the entry as a directory.
    #[inline]
    pub fn open_dir(&self) -> io::Result<DirViewUtf8> {
        Ok(DirViewUtf8 {
            dir: self.entry.open_dir()?,
            view_kind: self.view_kind,
        })
    }

    /// Removes the file from its filesystem.
    #[inline]
    pub fn remove_file(&self) -> io::Result<()> {
        self.check_mutation()?;
        self.entry.remove_file()
    }

    /// Removes the directory from its filesystem.
    #[inline]
    pub fn remove_dir(&self) -> io::Result<()> {
        self.check_mutation()?;
        self.entry.remove_dir()
    }

    /// Returns the metadata for the file that this entry points at.
    ///
    /// This corresponds to [`std::fs::DirEntry::metadata`].
    #[inline]
    pub fn metadata(&self) -> io::Result<Metadata> {
        self.entry.metadata()
    }

    /// Returns the file type for the file that this entry points at.
    ///
    /// This corresponds to [`std::fs::DirEntry::file_type`].
    #[inline]
    pub fn file_type(&self) -> io::Result<FileType> {
        self.entry.file_type()
    }

    /// Returns the bare file name of this directory entry without any other
    /// leading path component.
    ///
    /// This corresponds to [`std::fs::DirEntry::file_name`].
    #[inline]
    pub fn file_name(&self) -> io::Result<String> {
        self.entry.file_name()
    }

    fn check_mutation(&self) -> io::Result<()> {
        match self.view_kind {
            ViewKind::Full => Ok(()),
            ViewKind::Readonly => Err(Self::readonly()),
        }
    }

    fn readonly() -> io::Error {
        io::Error::new(
            io::ErrorKind::PermissionDenied,
            "attempt to modify a directory tree through a read-only `DirViewUtf8`",
        )
    }
}

#[cfg(not(windows))]
impl DirEntryExt for DirEntryViewUtf8 {
    #[inline]
    fn ino(&self) -> u64 {
        self.entry.ino()
    }
}

#[cfg(windows)]
#[doc(hidden)]
impl cap_primitives::fs::_WindowsDirEntryExt for DirEntryViewUtf8 {
    #[inline]
    fn full_metadata(&self) -> io::Result<Metadata> {
        cap_primitives::fs::_WindowsDirEntryExt::full_metadata(&self.entry)
    }
}

impl fmt::Debug for DirEntryViewUtf8 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.entry.fmt(f)
    }
}

#[cfg(feature = "cap-fs-ext")]
impl cap_fs_ext::DirEntryExt for DirEntryViewUtf8 {
    fn full_metadata(&self) -> io::Result<Metadata> {
        cap_fs_ext::DirEntryExt::full_metadata(&self.entry)
    }
}
