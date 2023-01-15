use cap_tempfile::{TempDir, ambient_authority};
use dir_view::{DirView, ViewKind};

#[test]
fn basic() {
    let temp_dir = TempDir::new(ambient_authority()).unwrap();

    // A full view can do everything.
    let dir = temp_dir.open_dir(".").unwrap();
    let full = DirView::from_dir(dir, ViewKind::Full);
    full.create("hello").unwrap();

    // A readonly view can't change anything.
    let dir = temp_dir.open_dir(".").unwrap();
    let readonly = DirView::from_dir(dir, ViewKind::Readonly);
    assert_eq!(readonly.create("hello").unwrap_err().kind(), std::io::ErrorKind::PermissionDenied);
    assert_eq!(readonly.create("create").unwrap_err().kind(), std::io::ErrorKind::PermissionDenied);
    assert_eq!(readonly.create_dir("create_dir").unwrap_err().kind(), std::io::ErrorKind::PermissionDenied);
    assert_eq!(readonly.rename("hello", &readonly, "to").unwrap_err().kind(), std::io::ErrorKind::PermissionDenied);
    assert_eq!(readonly.remove_file("hello").unwrap_err().kind(), std::io::ErrorKind::PermissionDenied);
    assert_eq!(readonly.remove_dir(".").unwrap_err().kind(), std::io::ErrorKind::PermissionDenied);
    assert_eq!(readonly.symlink("hello", "symlink").unwrap_err().kind(), std::io::ErrorKind::PermissionDenied);
    assert_eq!(readonly.hard_link("hello", &readonly, "hard_link").unwrap_err().kind(), std::io::ErrorKind::PermissionDenied);
}
