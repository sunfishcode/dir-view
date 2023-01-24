use crate::{ReadDirView, ViewKind};
use cap_std::fs::{Dir, DirBuilder, File, Metadata, OpenOptions, Permissions};
use cap_std::io_lifetimes::AsFilelike;
#[cfg(unix)]
use cap_std::os::unix::net::{UnixDatagram, UnixListener, UnixStream};
use cap_std::AmbientAuthority;
#[cfg(target_os = "wasi")]
use rustix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};
use std::{fmt, io};

/// A view of a [`Dir`].
///
/// This provides the same API as `Dir`, but imposes restrictions according
/// to the view kind.
pub struct DirView {
    pub(crate) dir: Dir,
    pub(crate) view_kind: ViewKind,
}

impl DirView {
    /// Constructs a new instance of `Self` from the given [`Dir`] and
    /// [`ViewKind`].
    #[inline]
    pub fn from_dir(dir: Dir, view_kind: ViewKind) -> Self {
        Self { dir, view_kind }
    }

    /// Attempts to open a file in read-only mode.
    ///
    /// This corresponds to [`std::fs::File::open`], but only accesses paths
    /// relative to `self`.
    #[inline]
    pub fn open<P: AsRef<Path>>(&self, path: P) -> io::Result<File> {
        self.dir.open(path)
    }

    /// Opens a file at `path` with the options specified by `options`.
    ///
    /// This corresponds to [`std::fs::OpenOptions::open`].
    ///
    /// Instead of being a method on `OpenOptions`, this is a method on `Dir`,
    /// and it only accesses paths relative to `self`.
    #[inline]
    pub fn open_with<P: AsRef<Path>>(&self, path: P, options: &OpenOptions) -> io::Result<File> {
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
        self.dir.open_with(path, &options)
    }

    /// Attempts to open a directory.
    #[inline]
    pub fn open_dir<P: AsRef<Path>>(&self, path: P) -> io::Result<Self> {
        Ok(Self {
            dir: self.dir.open_dir(path)?,
            view_kind: self.view_kind,
        })
    }

    /// Creates a new, empty directory at the provided path.
    ///
    /// This corresponds to [`std::fs::create_dir`], but only accesses paths
    /// relative to `self`.
    #[inline]
    pub fn create_dir<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        self.check_mutation()?;
        self.dir.create_dir(path)
    }

    /// Recursively create a directory and all of its parent components if they
    /// are missing.
    ///
    /// This corresponds to [`std::fs::create_dir_all`], but only accesses
    /// paths relative to `self`.
    #[inline]
    pub fn create_dir_all<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        self.check_mutation()?;
        self.dir.create_dir_all(path)
    }

    /// Creates the specified directory with the options configured in this
    /// builder.
    ///
    /// This corresponds to [`std::fs::DirBuilder::create`].
    #[cfg(not(target_os = "wasi"))]
    #[inline]
    pub fn create_dir_with<P: AsRef<Path>>(
        &self,
        path: P,
        dir_builder: &DirBuilder,
    ) -> io::Result<()> {
        self.check_mutation()?;
        self.dir.create_dir_with(path, dir_builder)
    }

    /// Opens a file in write-only mode.
    ///
    /// This corresponds to [`std::fs::File::create`], but only accesses paths
    /// relative to `self`.
    #[inline]
    pub fn create<P: AsRef<Path>>(&self, path: P) -> io::Result<File> {
        self.check_mutation()?;
        self.dir.create(path)
    }

    /// Returns the canonical form of a path with all intermediate components
    /// normalized and symbolic links resolved.
    ///
    /// This corresponds to [`std::fs::canonicalize`], but instead of returning
    /// an absolute path, returns a path relative to the directory
    /// represented by `self`.
    #[inline]
    pub fn canonicalize<P: AsRef<Path>>(&self, path: P) -> io::Result<PathBuf> {
        self.dir.canonicalize(path)
    }

    /// Copies the contents of one file to another. This function will also
    /// copy the permission bits of the original file to the destination
    /// file.
    ///
    /// This corresponds to [`std::fs::copy`], but only accesses paths
    /// relative to `self`.
    #[inline]
    pub fn copy<P: AsRef<Path>, Q: AsRef<Path>>(
        &self,
        from: P,
        to_dir: &Self,
        to: Q,
    ) -> io::Result<u64> {
        to_dir.check_mutation()?;
        self.dir.copy(from, &to_dir.dir, to)
    }

    /// Creates a new hard link on a filesystem.
    ///
    /// This corresponds to [`std::fs::hard_link`], but only accesses paths
    /// relative to `self`.
    #[inline]
    pub fn hard_link<P: AsRef<Path>, Q: AsRef<Path>>(
        &self,
        src: P,
        dst_dir: &Self,
        dst: Q,
    ) -> io::Result<()> {
        self.check_mutation()?;
        dst_dir.check_mutation()?;
        self.dir.hard_link(src, &dst_dir.dir, dst)
    }

    /// Given a path, query the file system to get information about a file,
    /// directory, etc.
    ///
    /// This corresponds to [`std::fs::metadata`], but only accesses paths
    /// relative to `self`.
    #[inline]
    pub fn metadata<P: AsRef<Path>>(&self, path: P) -> io::Result<cap_std::fs::Metadata> {
        self.dir.metadata(path)
    }

    /// Queries metadata about the underlying directory.
    ///
    /// This is similar to [`std::fs::File::metadata`], but for `Dir` rather
    /// than for `File`.
    #[inline]
    pub fn dir_metadata(&self) -> io::Result<Metadata> {
        self.dir.dir_metadata()
    }

    /// Returns an iterator over the entries within `self`.
    #[inline]
    pub fn entries(&self) -> io::Result<ReadDirView> {
        Ok(ReadDirView {
            read_dir: self.dir.entries()?,
            view_kind: self.view_kind,
        })
    }

    /// Returns an iterator over the entries within a directory.
    ///
    /// This corresponds to [`std::fs::read_dir`], but only accesses paths
    /// relative to `self`.
    #[inline]
    pub fn read_dir<P: AsRef<Path>>(&self, path: P) -> io::Result<ReadDirView> {
        Ok(ReadDirView {
            read_dir: self.dir.read_dir(path)?,
            view_kind: self.view_kind,
        })
    }

    /// Read the entire contents of a file into a bytes vector.
    ///
    /// This corresponds to [`std::fs::read`], but only accesses paths
    /// relative to `self`.
    #[inline]
    pub fn read<P: AsRef<Path>>(&self, path: P) -> io::Result<Vec<u8>> {
        self.dir.read(path)
    }

    /// Reads a symbolic link, returning the file that the link points to.
    ///
    /// This corresponds to [`std::fs::read_link`], but only accesses paths
    /// relative to `self`.
    #[inline]
    pub fn read_link<P: AsRef<Path>>(&self, path: P) -> io::Result<PathBuf> {
        self.dir.read_link(path)
    }

    /// Read the entire contents of a file into a string.
    ///
    /// This corresponds to [`std::fs::read_to_string`], but only accesses
    /// paths relative to `self`.
    #[inline]
    pub fn read_to_string<P: AsRef<Path>>(&self, path: P) -> io::Result<String> {
        self.dir.read_to_string(path)
    }

    /// Removes an empty directory.
    ///
    /// This corresponds to [`std::fs::remove_dir`], but only accesses paths
    /// relative to `self`.
    #[inline]
    pub fn remove_dir<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        self.check_mutation()?;
        self.dir.remove_dir(path)
    }

    /// Removes a directory at this path, after removing all its contents. Use
    /// carefully!
    ///
    /// This corresponds to [`std::fs::remove_dir_all`], but only accesses
    /// paths relative to `self`.
    #[inline]
    pub fn remove_dir_all<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        self.check_mutation()?;
        self.dir.remove_dir_all(path)
    }

    /// Remove the directory referenced by `self` and consume `self`.
    ///
    /// Even though this implementation works in terms of handles as much as
    /// possible, removal is not guaranteed to be atomic with respect to a
    /// concurrent rename of the directory.
    #[inline]
    pub fn remove_open_dir(self) -> io::Result<()> {
        self.check_mutation()?;
        self.dir.remove_open_dir()
    }

    /// Removes the directory referenced by `self`, after removing all its
    /// contents, and consume `self`. Use carefully!
    ///
    /// Even though this implementation works in terms of handles as much as
    /// possible, removal is not guaranteed to be atomic with respect to a
    /// concurrent rename of the directory.
    #[inline]
    pub fn remove_open_dir_all(self) -> io::Result<()> {
        self.check_mutation()?;
        self.dir.remove_open_dir_all()
    }

    /// Removes a file from a filesystem.
    ///
    /// This corresponds to [`std::fs::remove_file`], but only accesses paths
    /// relative to `self`.
    #[inline]
    pub fn remove_file<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        self.check_mutation()?;
        self.dir.remove_file(path)
    }

    /// Rename a file or directory to a new name, replacing the original file
    /// if to already exists.
    ///
    /// This corresponds to [`std::fs::rename`], but only accesses paths
    /// relative to `self`.
    #[inline]
    pub fn rename<P: AsRef<Path>, Q: AsRef<Path>>(
        &self,
        from: P,
        to_dir: &Self,
        to: Q,
    ) -> io::Result<()> {
        self.check_mutation()?;
        to_dir.check_mutation()?;
        self.dir.rename(from, &to_dir.dir, to)
    }

    /// Changes the permissions found on a file or a directory.
    ///
    /// This corresponds to [`std::fs::set_permissions`], but only accesses
    /// paths relative to `self`. Also, on some platforms, this function
    /// may fail if the file or directory cannot be opened for reading or
    /// writing first.
    #[cfg(not(target_os = "wasi"))]
    #[inline]
    pub fn set_permissions<P: AsRef<Path>>(&self, path: P, perm: Permissions) -> io::Result<()> {
        self.check_mutation()?;
        self.dir.set_permissions(path, perm)
    }

    /// Query the metadata about a file without following symlinks.
    ///
    /// This corresponds to [`std::fs::symlink_metadata`], but only accesses
    /// paths relative to `self`.
    #[inline]
    pub fn symlink_metadata<P: AsRef<Path>>(&self, path: P) -> io::Result<Metadata> {
        self.dir.symlink_metadata(path)
    }

    /// Write a slice as the entire contents of a file.
    ///
    /// This corresponds to [`std::fs::write`], but only accesses paths
    /// relative to `self`.
    #[inline]
    pub fn write<P: AsRef<Path>, C: AsRef<[u8]>>(&self, path: P, contents: C) -> io::Result<()> {
        self.check_mutation()?;
        self.dir.write(path, contents)
    }

    /// Creates a new symbolic link on a filesystem.
    ///
    /// The `original` argument provides the target of the symlink. The `link`
    /// argument provides the name of the created symlink.
    ///
    /// Despite the argument ordering, `original` is not resolved relative to
    /// `self` here. `link` is resolved relative to `self`, and `original` is
    /// not resolved within this function.
    ///
    /// The `link` path is resolved when the symlink is dereferenced, relative
    /// to the directory that contains it.
    ///
    /// This corresponds to [`std::os::unix::fs::symlink`], but only accesses
    /// paths relative to `self`.
    ///
    /// [`std::os::unix::fs::symlink`]: https://doc.rust-lang.org/std/os/unix/fs/fn.symlink.html
    #[cfg(not(windows))]
    #[inline]
    pub fn symlink<P: AsRef<Path>, Q: AsRef<Path>>(&self, original: P, link: Q) -> io::Result<()> {
        self.check_mutation()?;
        self.dir.symlink(original, link)
    }

    /// Creates a new file symbolic link on a filesystem.
    ///
    /// The `original` argument provides the target of the symlink. The `link`
    /// argument provides the name of the created symlink.
    ///
    /// Despite the argument ordering, `original` is not resolved relative to
    /// `self` here. `link` is resolved relative to `self`, and `original` is
    /// not resolved within this function.
    ///
    /// The `link` path is resolved when the symlink is dereferenced, relative
    /// to the directory that contains it.
    ///
    /// This corresponds to [`std::os::windows::fs::symlink_file`], but only
    /// accesses paths relative to `self`.
    ///
    /// [`std::os::windows::fs::symlink_file`]: https://doc.rust-lang.org/std/os/windows/fs/fn.symlink_file.html
    #[cfg(windows)]
    #[inline]
    pub fn symlink_file<P: AsRef<Path>, Q: AsRef<Path>>(
        &self,
        original: P,
        link: Q,
    ) -> io::Result<()> {
        self.check_mutation()?;
        self.dir.symlink_file(original, link)
    }

    /// Creates a new directory symlink on a filesystem.
    ///
    /// The `original` argument provides the target of the symlink. The `link`
    /// argument provides the name of the created symlink.
    ///
    /// Despite the argument ordering, `original` is not resolved relative to
    /// `self` here. `link` is resolved relative to `self`, and `original` is
    /// not resolved within this function.
    ///
    /// The `link` path is resolved when the symlink is dereferenced, relative
    /// to the directory that contains it.
    ///
    /// This corresponds to [`std::os::windows::fs::symlink_dir`], but only
    /// accesses paths relative to `self`.
    ///
    /// [`std::os::windows::fs::symlink_dir`]: https://doc.rust-lang.org/std/os/windows/fs/fn.symlink_dir.html
    #[cfg(windows)]
    #[inline]
    pub fn symlink_dir<P: AsRef<Path>, Q: AsRef<Path>>(
        &self,
        original: P,
        link: Q,
    ) -> io::Result<()> {
        self.check_mutation()?;
        self.dir.symlink_dir(original, link)
    }

    /// Creates a new `UnixListener` bound to the specified socket.
    ///
    /// This corresponds to [`std::os::unix::net::UnixListener::bind`], but
    /// only accesses paths relative to `self`.
    ///
    /// XXX: This function is not yet implemented.
    ///
    /// [`std::os::unix::net::UnixListener::bind`]: https://doc.rust-lang.org/std/os/unix/net/struct.UnixListener.html#method.bind
    #[cfg(unix)]
    #[inline]
    pub fn bind_unix_listener<P: AsRef<Path>>(&self, path: P) -> io::Result<UnixListener> {
        self.dir.bind_unix_listener(path)
    }

    /// Connects to the socket named by path.
    ///
    /// This corresponds to [`std::os::unix::net::UnixStream::connect`], but
    /// only accesses paths relative to `self`.
    ///
    /// XXX: This function is not yet implemented.
    ///
    /// [`std::os::unix::net::UnixStream::connect`]: https://doc.rust-lang.org/std/os/unix/net/struct.UnixStream.html#method.connect
    #[cfg(unix)]
    #[inline]
    pub fn connect_unix_stream<P: AsRef<Path>>(&self, path: P) -> io::Result<UnixStream> {
        self.dir.connect_unix_stream(path)
    }

    /// Creates a Unix datagram socket bound to the given path.
    ///
    /// This corresponds to [`std::os::unix::net::UnixDatagram::bind`], but
    /// only accesses paths relative to `self`.
    ///
    /// XXX: This function is not yet implemented.
    ///
    /// [`std::os::unix::net::UnixDatagram::bind`]: https://doc.rust-lang.org/std/os/unix/net/struct.UnixDatagram.html#method.bind
    #[cfg(unix)]
    #[inline]
    pub fn bind_unix_datagram<P: AsRef<Path>>(&self, path: P) -> io::Result<UnixDatagram> {
        self.dir.bind_unix_datagram(path)
    }

    /// Connects the socket to the specified address.
    ///
    /// This corresponds to [`std::os::unix::net::UnixDatagram::connect`], but
    /// only accesses paths relative to `self`.
    ///
    /// XXX: This function is not yet implemented.
    ///
    /// [`std::os::unix::net::UnixDatagram::connect`]: https://doc.rust-lang.org/std/os/unix/net/struct.UnixDatagram.html#method.connect
    #[cfg(unix)]
    #[inline]
    pub fn connect_unix_datagram<P: AsRef<Path>>(
        &self,
        unix_datagram: &UnixDatagram,
        path: P,
    ) -> io::Result<()> {
        self.dir.connect_unix_datagram(unix_datagram, path)
    }

    /// Sends data on the socket to the specified address.
    ///
    /// This corresponds to [`std::os::unix::net::UnixDatagram::send_to`], but
    /// only accesses paths relative to `self`.
    ///
    /// XXX: This function is not yet implemented.
    ///
    /// [`std::os::unix::net::UnixDatagram::send_to`]: https://doc.rust-lang.org/std/os/unix/net/struct.UnixDatagram.html#method.send_to
    #[cfg(unix)]
    #[inline]
    pub fn send_to_unix_datagram_addr<P: AsRef<Path>>(
        &self,
        unix_datagram: &UnixDatagram,
        buf: &[u8],
        path: P,
    ) -> io::Result<usize> {
        self.dir
            .send_to_unix_datagram_addr(unix_datagram, buf, path)
    }

    /// Creates a new `Dir` instance that shares the same underlying file
    /// handle as the existing `Dir` instance.
    #[inline]
    pub fn try_clone(&self) -> io::Result<Self> {
        Ok(Self {
            dir: self.dir.try_clone()?,
            view_kind: self.view_kind,
        })
    }

    /// Returns `true` if the path points at an existing entity.
    ///
    /// This corresponds to [`std::path::Path::exists`], but only
    /// accesses paths relative to `self`.
    #[inline]
    pub fn exists<P: AsRef<Path>>(&self, path: P) -> bool {
        self.dir.exists(path)
    }

    /// Returns `true` if the path points at an existing entity.
    ///
    /// This corresponds to [`std::fs::try_exists`], but only
    /// accesses paths relative to `self`.
    ///
    /// # API correspondence with `std`
    ///
    /// This API is not yet stable in `std`, but is likely to be. For more
    /// information, see the [tracker issue](https://github.com/rust-lang/rust/issues/83186).
    #[inline]
    pub fn try_exists<P: AsRef<Path>>(&self, path: P) -> io::Result<bool> {
        self.dir.try_exists(path)
    }

    /// Returns `true` if the path exists on disk and is pointing at a regular
    /// file.
    ///
    /// This corresponds to [`std::path::Path::is_file`], but only
    /// accesses paths relative to `self`.
    #[inline]
    pub fn is_file<P: AsRef<Path>>(&self, path: P) -> bool {
        self.dir.is_file(path)
    }

    /// Checks if `path` is a directory.
    ///
    /// This is similar to [`std::path::Path::is_dir`] in that it checks if
    /// `path` relative to `Dir` is a directory. This function will
    /// traverse symbolic links to query information about the destination
    /// file. In case of broken symbolic links, this will return `false`.
    #[inline]
    pub fn is_dir<P: AsRef<Path>>(&self, path: P) -> bool {
        self.dir.is_dir(path)
    }

    /// Constructs a new instance of `Self` by opening the given path as a
    /// directory using the host process' ambient authority.
    ///
    /// # Ambient Authority
    ///
    /// This function is not sandboxed and may access any path that the host
    /// process has access to.
    #[inline]
    pub fn open_ambient_dir<P: AsRef<Path>>(
        path: P,
        view_kind: ViewKind,
        ambient_authority: AmbientAuthority,
    ) -> io::Result<Self> {
        Ok(Self {
            dir: Dir::open_ambient_dir(path, ambient_authority)?,
            view_kind,
        })
    }

    /// Constructs a new instance of `Self` by opening the parent directory
    /// (aka "..") of `self`, using the host process' ambient authority.
    ///
    /// # Ambient Authority
    ///
    /// This function accesses a directory outside of the `self` subtree.
    #[inline]
    pub fn open_parent_dir(
        &self,
        view_kind: ViewKind,
        ambient_authority: AmbientAuthority,
    ) -> io::Result<Self> {
        Ok(Self {
            dir: self.dir.open_parent_dir(ambient_authority)?,
            view_kind,
        })
    }

    /// Construct a new instance of `Self` from existing directory file
    /// descriptor.
    ///
    /// This can be useful when interacting with other libraries and or C/C++
    /// code which has invoked `openat(..., O_DIRECTORY)` external to this
    /// crate.
    pub fn reopen_dir<Filelike: AsFilelike>(
        dir: &Filelike,
        view_kind: ViewKind,
    ) -> io::Result<Self> {
        Ok(Self {
            dir: Dir::reopen_dir(dir)?,
            view_kind,
        })
    }

    fn check_mutation(&self) -> io::Result<()> {
        match self.view_kind {
            ViewKind::Full => Ok(()),
            ViewKind::Readonly => return Err(Self::readonly()),
        }
    }

    fn readonly() -> io::Error {
        io::Error::new(
            io::ErrorKind::PermissionDenied,
            "attempt to modify a directory tree through a read-only `DirView`",
        )
    }
}

impl fmt::Debug for DirView {
    // Like libstd's version, but doesn't print the path.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut b = f.debug_struct("DirView");
        b.field("dir", &self.dir);
        #[cfg(windows)]
        b.field("view_kind", &self.view_kind);
        b.finish()
    }
}
