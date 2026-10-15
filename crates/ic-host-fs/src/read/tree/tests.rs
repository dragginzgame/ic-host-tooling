use super::{TreeVisitor, walk_tree};
use crate::test_support::Fixture;
use rustix::{
    fd::OwnedFd,
    fs::{FileType, Mode, OFlags, open},
};
use std::{
    ffi::OsStr,
    fs::{self, File},
    io::{self, Read as _},
    os::unix::{ffi::OsStrExt as _, fs::symlink},
    path::{Path, PathBuf},
};

#[derive(Default)]
struct RecordingVisitor {
    files: Vec<(PathBuf, Vec<u8>)>,
    entered: Vec<PathBuf>,
    left: Vec<PathBuf>,
}

impl TreeVisitor for RecordingVisitor {
    type Error = io::Error;

    fn file(&mut self, fd: &OwnedFd, relative: &Path, _display: &Path) -> io::Result<()> {
        let mut bytes = Vec::new();
        File::from(fd.try_clone()?).read_to_end(&mut bytes)?;
        self.files.push((relative.to_path_buf(), bytes));
        Ok(())
    }

    fn enter_directory(&mut self, relative: &Path, _display: &Path) -> io::Result<()> {
        self.entered.push(relative.to_path_buf());
        Ok(())
    }

    fn leave_directory(
        &mut self,
        _fd: &OwnedFd,
        relative: &Path,
        _display: &Path,
    ) -> io::Result<()> {
        self.left.push(relative.to_path_buf());
        Ok(())
    }

    fn unsupported(&self, _display: &Path, _kind: FileType) -> io::Error {
        io::Error::new(io::ErrorKind::InvalidData, "unsupported entry")
    }

    fn non_utf8(&self, _display: &Path) -> io::Error {
        io::Error::new(io::ErrorKind::InvalidData, "non-UTF-8 entry")
    }
}

fn opened_root(path: &Path) -> OwnedFd {
    open(
        path,
        OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
        Mode::empty(),
    )
    .unwrap()
}

#[test]
fn visits_opened_files_and_leaves_nested_directories_before_root() {
    let fixture = Fixture::new();
    fs::create_dir(fixture.root.join("nested")).unwrap();
    fs::write(fixture.root.join("nested/bytes"), b"exact").unwrap();
    let mut visitor = RecordingVisitor::default();
    walk_tree(
        &opened_root(&fixture.root),
        Path::new(""),
        &fixture.root,
        &mut visitor,
    )
    .unwrap();
    assert_eq!(
        visitor.files,
        [(PathBuf::from("nested/bytes"), b"exact".to_vec())]
    );
    assert_eq!(visitor.entered, [PathBuf::from("nested")]);
    assert_eq!(visitor.left, [PathBuf::from("nested"), PathBuf::new()]);
}

#[test]
fn rejects_symlink_and_non_utf8_names_without_visiting_targets() {
    let fixture = Fixture::new();
    fs::write(fixture.root.join("target"), b"retained").unwrap();
    symlink("target", fixture.root.join("link")).unwrap();
    let error = walk_tree(
        &opened_root(&fixture.root),
        Path::new(""),
        &fixture.root,
        &mut RecordingVisitor::default(),
    )
    .unwrap_err();
    assert_eq!(error.to_string(), "unsupported entry");
    fs::remove_file(fixture.root.join("link")).unwrap();
    let non_utf8 = OsStr::from_bytes(b"\xff");
    // Observe native name admission independently: some filesystems reject
    // these bytes before the walker can encounter a directory entry.
    let native = fs::write(fixture.root.join(non_utf8), b"retained");
    let mut visitor = RecordingVisitor::default();
    let result = walk_tree(
        &opened_root(&fixture.root),
        Path::new(""),
        &fixture.root,
        &mut visitor,
    );
    match native {
        Ok(()) => {
            assert_eq!(result.unwrap_err().to_string(), "non-UTF-8 entry");
            assert!(
                visitor
                    .files
                    .iter()
                    .all(|(path, _)| path == Path::new("target"))
            );
            assert_eq!(fs::read(fixture.root.join(non_utf8)).unwrap(), b"retained");
        }
        Err(error) => {
            assert_eq!(
                error.raw_os_error(),
                Some(rustix::io::Errno::ILSEQ.raw_os_error())
            );
            result.unwrap();
            assert_eq!(
                visitor.files,
                [(PathBuf::from("target"), b"retained".to_vec())]
            );
            // Enumerate rather than looking up a name the filesystem refused.
            let names: Vec<_> = fs::read_dir(&fixture.root)
                .unwrap()
                .map(|entry| entry.unwrap().file_name())
                .collect();
            assert_eq!(names, [OsStr::new("target")]);
        }
    }
    assert_eq!(fs::read(fixture.root.join("target")).unwrap(), b"retained");
}
