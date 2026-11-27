//! Paths and holding-directory facts for the system's link walk.

use super::{Entry, walk};
use std::cell::RefCell;

const OWNER: u32 = 1000;
const DIRECTORY: libc::mode_t = libc::S_IFDIR | 0o755;
const LINK: libc::mode_t = libc::S_IFLNK | 0o777;

enum Call<'a> {
    Stat(&'a [u8], libc::mode_t),
    Read(&'a [u8], &'a [u8]),
}

fn scripted_walk(path: &[u8], calls: &[Call<'_>]) -> Option<Box<[u8]>> {
    let calls = RefCell::new(calls.iter());
    let answer = walk(
        path,
        OWNER,
        |path| {
            let Some(Call::Stat(expected, mode)) = calls.borrow_mut().next() else {
                panic!("unexpected stat of {path:?}");
            };
            assert_eq!(path, *expected);
            Some(Entry { mode: *mode, owner: OWNER })
        },
        |path| {
            let Some(Call::Read(expected, target)) = calls.borrow_mut().next() else {
                panic!("unexpected readlink of {path:?}");
            };
            assert_eq!(path, *expected);
            Some((*target).into())
        },
    );
    assert_eq!(calls.into_inner().len(), 0, "the walk uses the scripted calls");
    answer
}

#[test]
fn dotdot_stops_before_a_link() {
    assert_eq!(scripted_walk(b"real/../sys/f", &[Call::Stat(b"real", DIRECTORY)]), None, "dotdot ends the walk");
}

#[test]
fn absolute_link_targets_restart_at_root() {
    let answer = scripted_walk(
        b"/system/abs/f",
        &[
            Call::Stat(b"/system", DIRECTORY),
            Call::Stat(b"/system/abs", LINK),
            Call::Stat(b"/system/", DIRECTORY),
            Call::Read(b"/system/abs", b"/real"),
            Call::Stat(b"/real", DIRECTORY),
        ],
    )
    .expect("absolute link passed");
    assert_eq!(&*answer, b"/real/f", "an absolute target replaces the whole prefix");
    assert!(answer.starts_with(b"/"), "the answer starts again at root");
}

#[test]
fn eight_links_are_passed_and_the_ninth_is_left() {
    let names: Vec<_> = (1..=9).map(|number| format!("l{number}")).collect();
    let mut calls = Vec::new();
    for (name, target) in names.iter().zip(names.iter().skip(1)) {
        calls.push(Call::Stat(name.as_bytes(), LINK));
        calls.push(Call::Stat(b".", DIRECTORY));
        calls.push(Call::Read(name.as_bytes(), target.as_bytes()));
    }
    calls.push(Call::Stat(b"l9", LINK));
    let answer = scripted_walk(b"l1/f", &calls).expect("eight links passed");
    assert_eq!(&*answer, b"l9/f", "the ninth link is left as written");
}

#[test]
fn a_sibling_link_names_one_regular_file_without_following_another_link() {
    use std::{fs, os::unix::fs::symlink};

    let scratch = skein_scratch::Scratch::new("sibling-link");
    let root = scratch.path().join("hooks");
    fs::create_dir(&root).expect("hooks directory");
    fs::create_dir(root.join("sub")).expect("subdirectory");
    fs::write(root.join("deliver"), b"regular").expect("regular hook");
    fs::write(root.join("sub/deliver"), b"regular").expect("nested file");
    fs::write(scratch.path().join("pre-pr"), b"fallback").expect("neighboring legacy check");
    for (name, target) in [
        ("finish", "deliver"),
        ("outside", "../pre-pr"),
        ("chain", "finish"),
        ("nested", "sub/deliver"),
        ("directory", "sub"),
        ("dangling", "absent"),
    ] {
        symlink(target, root.join(name)).expect("link in the hooks directory");
    }
    let dir = crate::open_root(&root).expect("open hooks directory");
    let lookup = |name: &[u8]| super::sibling_file(dir.raw(), name);
    assert_eq!(lookup(b"finish").expect("sibling link").as_deref(), Some(b"deliver".as_slice()));
    assert_eq!(lookup(b"deliver"), Ok(None), "direct name needs no rewrite");
    assert_eq!(lookup(b"absent"), Err(libc::ENOENT), "absence is distinct from a bad link");
    for name in [b"outside".as_slice(), b"chain", b"nested", b"directory", b"dangling"] {
        assert_eq!(lookup(name), Err(libc::ELOOP), "refuse {name:?}");
    }
    crate::system::close(dir).expect("close hooks directory");
}
