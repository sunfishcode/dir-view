# dir-view

<p>
  <a href="https://crates.io/crates/qualified-dir"><img src="https://img.shields.io/crates/v/qualified-dir.svg" alt="crates.io page" /></a>
  <a href="https://docs.rs/qualified-dir"><img src="https://docs.rs/qualified-dir/badge.svg" alt="docs.rs docs" /></a>
</p>

This providfes [`DirView`] a wrapper around [`cap_std::fs::Dir`] and [`cap_std::fs_utf8::Dir`]
which provides limited views of directories.

Currently the only such view is a readonly view.

[`DirView`]: https://docs.rs/dir-view/latest/dir_view/struct.DirView.html
[`cap_std::fs::Dir`]: https://docs.rs/cap-std/latest/cap_std/fs/struct.Dir.html
[`cap_std::fs_utf8::Dir`]: https://docs.rs/cap-std/latest/cap_std/fs_utf8/struct.Dir.html
