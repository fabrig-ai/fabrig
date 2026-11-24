//! Scripted command terminals for a checked local change on the fake filesystem.
//! The check sees the workspace that the host will commit (smith/docs/design/domain/run-delivery.md, section 1).

use skein_fake_llm_domain::api;
use skein_fake_machine::{How, Item, Machine, Opened};
use skein_lib::Queue;

pub(super) const CHECK: &[u8] = b".smith-test/check";
pub(super) const CHECK_FILE: &[u8] = b"#!/bin/sh\n";
const COMMAND: &[u8] = b"printf 'ran\n' > command.txt";
const CHECK_INPUTS: [(&[u8], &[u8]); 3] =
    [(b"result.txt", b"new\n"), (b"original.txt", b"edited\n"), (b"command.txt", b"ran\n")];

pub(super) fn files() -> [Item; 2] {
    [Item::directory(b"repo/.smith-test"), Item::file(b"repo/.smith-test/check", CHECK_FILE).mode(0o700)]
}

pub(super) fn scripts(large_write: bool) -> Box<[api::Script]> {
    let mut scripts = super::tool_scripts();
    let [read_original, _search, change @ ..] = scripts[0].turns.as_ref() else {
        unreachable!("the checked change starts by inspecting the original")
    };
    let mut turns = Vec::new();
    if large_write {
        let turn = |name: &[u8], arguments| {
            smith_agent_world::fixture::calls(Box::new([api::Line::Call { name: name.into(), arguments }]), 1)
        };
        turns.extend([
            turn(b"write", smith_agent_process_world::large_coding_write()),
            turn(b"read", br#"{"path":"result.txt"}"#.as_slice().into()),
        ]);
    }
    turns.push(read_original.clone());
    turns.extend_from_slice(change);
    scripts[0].turns = turns.into();
    scripts
}

pub(super) fn command(files: &mut Machine, call: &skein_sim::Call, answers: &mut Queue<skein_sim::Answer>) -> bool {
    let skein_sim::Ask::Spawn { root, program, args, dir, .. } = &call.ask else { return false };
    let root = Opened::new(root.raw());
    let code = match program.as_ref() {
        b"/bin/sh" if args.iter().map(AsRef::as_ref).eq([b"-c".as_slice(), COMMAND]) => {
            let work = files.open(root, dir, How::Directory).expect("command working directory");
            write(files, work, b"command.txt", b"ran\n");
            files.close(work);
            0
        }
        CHECK => {
            assert!(args.is_empty(), "the check runs without arguments");
            let work = files.open(root, dir, How::Directory).expect("check working directory");
            let passed = CHECK_INPUTS.iter().all(|(path, expected)| file_matches(files, work, path, expected));
            if passed {
                write(files, work, b"checks-ran.txt", b"checked\n");
            }
            files.close(work);
            u8::from(!passed)
        }
        _ => return false,
    };
    answers.push(skein_sim::Answer {
        ticket: call.ticket,
        result: Ok(skein_sim::Reply::Program(skein_sim::Program::Exit(code))),
    });
    true
}

fn write(files: &mut Machine, root: Opened, path: &[u8], bytes: &[u8]) {
    let file = files.open(root, path, How::Create { mode: 0o600 }).expect("scripted output file");
    files.write(file, 0, bytes).expect("scripted output bytes");
    files.close(file);
}

fn file_matches(files: &mut Machine, root: Opened, path: &[u8], expected: &[u8]) -> bool {
    let Ok(file) = files.open(root, path, How::ReadNoFollow) else { return false };
    let limit = u32::try_from(expected.len() + 1).expect("the expected check input and one extra byte");
    let bytes = files.read(file, 0, limit).expect("check input bytes");
    files.close(file);
    bytes == expected
}
