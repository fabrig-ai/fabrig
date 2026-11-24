//! The shared engine/TUI path and private link (client/tui/directory.md, 1).
use super::{MOST, Publication, forget_with, keep_with, read_with};
use crate::{
    directories::{PrivateDirectories, make_private_with},
    link_file,
    test_disk::{Disk, USER, io_error, name},
};
use skein_fake_machine::{How, Item, Opened, Refusal};
use skein_io::file::Unsafe;
use skein_lib::Secret;
use skein_shell::Unread;
use std::{io, path::Path};

const FIRST: &[u8] = b"http://127.0.0.1:7000/jig/v1/sign-in/link?secret=fixture";

#[derive(Debug, PartialEq, Eq)]
enum Effect {
    Prepare,
    Seed,
    Create(u32),
    Write,
    Sync,
    Close,
    Rename,
    Remove(Result<(), Refusal>),
}

struct Recorded {
    disk: Disk,
    effects: Vec<Effect>,
}

impl Recorded {
    fn new(items: &[Item]) -> Self {
        Self { disk: Disk::new(items), effects: Vec::new() }
    }

    fn contents(&mut self, path: &Path) -> Vec<u8> {
        let file = self.disk.open(path, How::ReadNoFollow).expect("kept link");
        let bytes = self.disk.machine.read(file, 0, MOST).expect("read kept link");
        self.disk.machine.close(file);
        bytes
    }
}

impl Publication for Recorded {
    type File = Opened;

    fn prepare(&mut self, path: &Path, user: u32) -> Result<(), String> {
        self.effects.push(Effect::Prepare);
        make_private_with(path, user, &mut self.disk)
    }
    fn seed(&mut self) -> Result<Secret, i32> {
        self.effects.push(Effect::Seed);
        Ok(Secret::new(Box::new([3; 32])))
    }
    fn create_new(&mut self, path: &Path, mode: u32) -> io::Result<Self::File> {
        self.effects.push(Effect::Create(mode));
        self.disk.open(path, How::Create { mode }).map_err(io_error)
    }
    fn write_all(&mut self, file: &mut Self::File, bytes: &[u8]) -> io::Result<()> {
        self.effects.push(Effect::Write);
        self.disk.machine.append(*file, bytes).map_err(io_error)
    }
    fn sync(&mut self, file: &Self::File) -> io::Result<()> {
        self.effects.push(Effect::Sync);
        self.disk.machine.sync(*file);
        Ok(())
    }
    fn close(&mut self, file: Self::File) {
        self.effects.push(Effect::Close);
        self.disk.machine.close(file);
    }
    fn rename(&mut self, from: &Path, to: &Path) -> io::Result<()> {
        self.effects.push(Effect::Rename);
        assert_eq!(from.parent(), to.parent(), "publication stays in its directory");
        self.disk.at_parent(to, |disk, parent, to| disk.rename(parent, name(from), parent, to, false)).map_err(io_error)
    }
    fn remove(&mut self, path: &Path) -> io::Result<()> {
        let result = self.disk.at_parent(path, |disk, parent, name| disk.remove(parent, name, false));
        self.effects.push(Effect::Remove(result));
        result.map_err(io_error)
    }
}

#[test]
fn keeping_replaces_the_whole_private_file_and_forgetting_is_quiet() {
    let mut link = Recorded::new(&[]);
    let path = Path::new("/missing/parent/printed-link");
    let parent = path.parent().expect("parent");
    let publish = [
        Effect::Prepare,
        Effect::Seed,
        Effect::Create(0o600),
        Effect::Write,
        Effect::Write,
        Effect::Sync,
        Effect::Close,
        Effect::Rename,
    ];
    keep_with(path, FIRST, USER, &mut link).expect("keep");
    assert!(link.contents(path) == [FIRST, b"\n"].concat(), "whole URL and LF");
    assert_eq!(link.disk.symlink_stat(path).expect("file").mode, 0o600);
    for directory in [parent, Path::new("/missing")] {
        assert_eq!(link.disk.symlink_stat(directory).expect("parent").mode, 0o700);
    }
    assert_eq!(link.effects, publish);
    link.effects.clear();

    keep_with(path, b"short", USER, &mut link).expect("replace");
    assert!(link.contents(path) == b"short\n", "no suffix from the previous link");
    let directory = link.disk.open(parent, How::Directory).expect("link directory");
    let entries = link.disk.machine.list(directory, 10, 4096).expect("directory entries");
    link.disk.machine.close(directory);
    assert_eq!(entries.len(), 1, "no temporary left");
    assert_eq!(link.effects, publish);
    link.effects.clear();

    forget_with(path, &mut link);
    assert_eq!(link.disk.open(path, How::PathNoFollow), Err(Refusal::NotFound));
    forget_with(path, &mut link);
    assert_eq!(link.effects, [Effect::Remove(Ok(())), Effect::Remove(Err(Refusal::NotFound))]);
}

#[test]
fn keeping_refuses_an_insecure_or_foreign_parent_without_disclosing_the_url() {
    let path = Path::new("/parent/printed-link");
    for (mode, user) in [(0o770, USER), (0o700, USER + 1)] {
        let mut link = Recorded::new(&[Item::directory(b"parent").mode(mode)]);
        let why = keep_with(path, FIRST, user, &mut link).expect_err("unsafe parent");
        assert!(why.starts_with(&format!("{}: ", path.display())), "error names the target path");
        assert!(!why.as_bytes().windows(FIRST.len()).any(|part| part == FIRST), "error holds no URL");
        assert_eq!(link.disk.open(path, How::PathNoFollow), Err(Refusal::NotFound), "refusal writes nothing");
        assert_eq!(link.effects, [Effect::Prepare]);
    }
}

#[test]
fn reading_refuses_nonprivate_foreign_and_oversize_files_and_strips_one_lf() {
    let path = Path::new("/printed-link");
    let bound = usize::try_from(MOST).expect("link bound");
    for (row, result, expected) in [
        ("absent", Err(Unread::Absent), None),
        ("final LF", Ok([FIRST, b"\n"].concat()), Some(FIRST.to_vec())),
        ("one LF only", Ok(b"two\n\n".to_vec()), Some(b"two\n".to_vec())),
        ("no LF", Ok(b"bare".to_vec()), Some(b"bare".to_vec())),
        ("group-readable", Err(Unread::Unsafe(Unsafe::Mode(0o040))), None),
        ("foreign owner", Err(Unread::Unsafe(Unsafe::Owner { found: USER + 1, expected: USER })), None),
        ("past MOST", Err(Unread::TooLarge { max: MOST }), None),
        ("at MOST", Ok(vec![b'x'; bound]), Some(vec![b'x'; bound])),
    ] {
        let found = read_with(path, USER, |name, max, user| {
            assert_eq!((name, max, user), (path, 4096, USER), "{row}");
            result.map(|bytes| Secret::new(bytes.into_boxed_slice()))
        });
        assert!(found.as_ref().map(Secret::expose) == expected.as_deref(), "{row}");
    }
}

#[test]
fn paths_key_the_deployment_and_refuse_a_missing_absolute_runtime() {
    let mut failures = Vec::new();
    for (data, runtime, linux, expected) in [
        (
            b"/data".as_slice(),
            Some(b"/run/user/test".as_slice()),
            true,
            Some("/run/user/test/temper/printed-link-bd47413b5c03dfc6"),
        ),
        (b"/data", Some(b"relative".as_slice()), true, None),
        (b"/data", None, true, None),
        (b"/data", None, false, Some("/data/printed-link")),
    ] {
        if link_file::path(data, runtime, linux) != expected.map(Into::into) {
            failures.push((runtime, linux));
        }
    }
    assert!(failures.is_empty(), "path rows: {failures:?}");
    assert_ne!(link_file::path(b"/data", Some(b"/runtime"), true), link_file::path(b"/other", Some(b"/runtime"), true));
}

#[test]
fn deployments_resolve_as_the_box_without_flags() {
    let mut failures = Vec::new();
    for (row, entries) in [
        ("absolute TEMPER_DATA", vec![("HOME", "/home/test"), ("TEMPER_DATA", "/deployment")]),
        ("relative TEMPER_DATA", vec![("HOME", "/home/test"), ("TEMPER_DATA", "relative")]),
        ("XDG_DATA_HOME", vec![("HOME", "/home/test"), ("XDG_DATA_HOME", "/xdg")]),
        ("HOME", vec![("HOME", "/home/test")]),
        ("no environment", vec![]),
    ] {
        let environment: Vec<_> =
            entries.iter().map(|(k, v)| (Box::from(k.as_bytes()), Box::from(v.as_bytes()))).collect();
        let expected = crate::directories::resolve(&[], &environment, b"/working").ok().map(|(d, _)| d.data);
        if link_file::deployment(&environment, b"/working") != expected {
            failures.push(row);
        }
    }
    assert!(failures.is_empty(), "deployment rows: {failures:?}");
}
