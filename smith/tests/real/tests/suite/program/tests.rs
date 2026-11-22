#![cfg(unix)]

use skein_scratch::Scratch;
use smith_agent_document::program::{Refusal, Unsafe, prepare, private_directory};
use std::{
    fs,
    os::unix::fs::{MetadataExt, PermissionsExt, symlink},
};

#[test]
fn preparing_makes_both_directories_private() {
    let scratch = Scratch::new("program-prepare");
    let programs = scratch.path().join("missing/programs");
    assert!(!private_directory(&programs).expect("absent directory"));
    let configuration = prepare(&programs, "source-1").expect("private directories");
    assert_eq!(configuration, programs.join("source-1"));
    for path in [&programs, &configuration] {
        let metadata = fs::symlink_metadata(path).expect("created directory");
        assert!(metadata.is_dir(), "created a directory");
        assert_eq!(metadata.mode() & 0o777, 0o700);
        assert_eq!(metadata.uid(), skein_shell::effective_user());
    }
}

#[test]
fn preparing_keeps_an_existing_private_directory() {
    let scratch = Scratch::new("program-existing");
    let programs = scratch.path().join("programs");
    let configuration = prepare(&programs, "source").expect("prepare once");
    fs::write(configuration.join("kept"), b"kept").expect("existing source data");
    let inode = fs::symlink_metadata(&configuration).expect("source metadata").ino();
    assert_eq!(prepare(&programs, "source").expect("prepare again"), configuration);
    assert_eq!(fs::symlink_metadata(&configuration).expect("source metadata").ino(), inode);
    assert_eq!(fs::read(configuration.join("kept")).expect("source data"), b"kept");
}

#[test]
fn preparing_refuses_a_directory_open_to_others() {
    let scratch = Scratch::new("program-mode");
    let programs = scratch.path().join("programs");
    fs::create_dir(&programs).expect("programs directory");
    fs::set_permissions(&programs, fs::Permissions::from_mode(0o755)).expect("open mode");
    match prepare(&programs, "source") {
        Err(Refusal::Unsafe { path, found: Unsafe::Mode(0o055) }) => assert_eq!(path, programs),
        other => panic!("expected mode refusal, got {other:?}"),
    }
    assert_eq!(fs::read_dir(&programs).expect("programs contents").count(), 0);
}

#[test]
fn preparing_refuses_a_link() {
    let scratch = Scratch::new("program-link");
    let target = prepare(&scratch.path().join("target"), "source").expect("private target");
    let programs = scratch.path().join("programs");
    symlink(&target, &programs).expect("linked programs");
    for path in [&programs, &programs.join(""), &programs.join(".")] {
        match prepare(path, "source") {
            Err(Refusal::Unsafe { path: refused, found: Unsafe::Link }) => assert_eq!(refused, *path),
            other => panic!("expected link refusal, got {other:?}"),
        }
    }
    assert_eq!(fs::read_dir(target).expect("target contents").count(), 0);
}

#[test]
fn preparing_refuses_a_file() {
    let scratch = Scratch::new("program-file");
    let programs = scratch.path().join("programs");
    fs::write(&programs, b"file").expect("file fixture");
    match prepare(&programs, "source") {
        Err(Refusal::Unsafe { path, found: Unsafe::NotADirectory }) => assert_eq!(path, programs),
        other => panic!("expected file refusal, got {other:?}"),
    }
    assert_eq!(fs::read(programs).expect("untouched file"), b"file");
}

#[test]
fn preparing_checks_the_existing_source_directory() {
    let scratch = Scratch::new("program-source");
    let programs = scratch.path().join("programs");
    let configuration = prepare(&programs, "source").expect("source directory");
    fs::set_permissions(&configuration, fs::Permissions::from_mode(0o755)).expect("open source");
    match prepare(&programs, "source") {
        Err(Refusal::Unsafe { path, found: Unsafe::Mode(0o055) }) => assert_eq!(path, configuration),
        other => panic!("expected source mode refusal, got {other:?}"),
    }
    fs::remove_dir(&configuration).expect("remove open source");
    symlink(scratch.path(), &configuration).expect("source link");
    match prepare(&programs, "source") {
        Err(Refusal::Unsafe { path, found: Unsafe::Link }) => assert_eq!(path, configuration),
        other => panic!("expected source link refusal, got {other:?}"),
    }
}
