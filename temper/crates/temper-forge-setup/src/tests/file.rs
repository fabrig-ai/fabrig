//! Group files share setup's entry form, bounded input and resolved paths.
#![expect(clippy::disallowed_macros, clippy::disallowed_types, reason = "ordinary Rust JSON fixture editing")]
use super::fixtures;
use crate::{
    file, limits, problem,
    setup::{Mode, Setup},
};
use jig_protocol_config as config;

fn fixture(forge: u16) -> String {
    let source = include_str!("../../../temper-init/template/temper.json");
    let source = source.split_once("\"setup\": [").unwrap().1;
    let marker = format!("\"forge\": {forge},");
    let member = source.find(&marker).unwrap();
    let start = source[..member].rfind('{').unwrap();
    let mut depth = 0_u32;
    let mut end = start;
    for (offset, byte) in source.as_bytes()[start..].iter().enumerate() {
        match byte {
            b'{' => depth = depth.checked_add(1).unwrap(),
            b'}' => {
                depth = depth.checked_sub(1).unwrap();
                if depth == 0 {
                    end = start.checked_add(offset).unwrap().checked_add(1).unwrap();
                    break;
                }
            }
            _ => {}
        }
    }
    source[start..end]
        .replace("$forge", "127.0.0.1:8369")
        .replace("$owned", "127.0.0.1:3000")
        .replace("$login", "person")
        .replace("$email", "person@localhost")
}
fn read(bytes: &[u8], forge: u16) -> Result<file::GroupFile, problem::Problems> {
    file::read(bytes, forge, &limits::small(), &config::Directories::given(b"/config", b"/data", b"/cache").unwrap())
}
fn group(entry: &str, playground: bool) -> String {
    format!("{{\"setup\":{entry},\"playground\":{playground}}}")
}
fn refusal(bytes: &[u8], forge: u16) -> Box<[u8]> {
    problem::lines(&read(bytes, forge).unwrap_err())
}
#[test]
fn a_group_file_reads_as_its_entry_in_a_document() {
    for (forge, playground) in [(3, false), (1, true), (1, false)] {
        let entry = fixture(forge);
        let list = format!("[{entry}]");
        let setup = Setup::read(
            &fixtures::document(list.as_bytes()),
            0,
            &limits::small(),
            &config::Directories::given(b"/config", b"/data", b"/cache").unwrap(),
            &mut problem::Problems::empty(),
        )
        .unwrap();
        let file = read(group(&entry, playground).as_bytes(), forge).unwrap();
        assert_eq!(file.entry, setup.entries[0]);
        assert_eq!(file.playground, playground);
        match &file.entry.mode {
            Mode::Owned(owned) => {
                assert_eq!(file.entry.directory.as_ref(), b"/data/forge-3");
                assert!(owned.forgejo.starts_with(b"/cache/forgejo/"));
                assert!(owned.runner.as_ref().unwrap().binary.starts_with(b"/cache/forgejo-runner/"));
            }
            Mode::Fake(fake) => {
                assert_eq!(file.entry.directory.as_ref(), b"/data/demo/forge");
                assert_eq!(fake.git.as_ref(), b"/data/demo/bin/git");
            }
            Mode::Managed { .. } => panic!("owned or fake"),
        }
    }
    assert_eq!(file::path(7).as_ref(), b"box/groups/7");
    assert_eq!(file::path(u16::MAX).as_ref(), b"box/groups/65535");
}
#[test]
fn a_group_file_refuses_another_number_an_unasked_owned_entry_and_a_playground_not_fake() {
    for (entry, forge, playground, expected) in [
        (fixture(3), 7, false, b"setup.forge: must match the group number\n".as_slice()),
        (fixture(3).replace("\"start\": \"asked\",", ""), 3, false, b"setup.start: must be asked\n"),
        (fixture(3), 3, true, b"playground: requires a fake setup entry\n"),
        (fixture(1).replace("demo/bin/git", "../git"), 1, true,
            b"setup.mode.fake.git: relative path must be non-empty with no .. component\n"),
        (fixture(3).replace("\"fetched\"", "\"../binary\""), 3, false,
            b"setup.mode.owned.forgejo: relative path must be non-empty with no .. component\nsetup.mode.owned.runner.binary: relative path must be non-empty with no .. component\n"),
    ] {
        assert_eq!(refusal(group(&entry, playground).as_bytes(), forge).as_ref(), expected);
    }
}
#[test]
fn a_group_file_refuses_another_member_and_bytes_past_its_bound() {
    let file = group(&fixture(1), true);
    let extra = file.replacen('{', "{\"other\":null,", 1);
    assert_eq!(refusal(extra.as_bytes(), 1).as_ref(), b"other: unknown member\n");
    let mut bound = file.into_bytes();
    bound.resize(usize::try_from(file::BYTES).unwrap(), b' ');
    assert!(read(&bound, 1).is_ok(), "the bound is inclusive");
    bound.push(b' ');
    read(&bound, 1).unwrap_err();
    for (source, expected) in [
        ("{}", b"setup: missing\nplayground: missing\n".as_slice()),
        ("{\"setup\":null,\"playground\":false}", b"setup: wrong value kind\n"),
    ] {
        assert_eq!(refusal(source.as_bytes(), 1).as_ref(), expected);
    }
    let wrong = group(&fixture(1), true).replace("\"playground\":true", "\"playground\":1");
    assert_eq!(refusal(wrong.as_bytes(), 1).as_ref(), b"playground: wrong value kind\n");
}
