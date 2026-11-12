//! The agent wrapper preserves both launch contracts (smith/docs/design/protocol/programs.md, 5.1).
#![expect(clippy::disallowed_methods, reason = "ordinary Rust unit fixtures, programming-model.md 10.2")]

use super::{Below, fixtures};
use alloc::boxed::Box;
use skein_io::{self as io, kernel};
use skein_lib::Queue;

fn configured(parent: Option<u32>) -> super::Backend {
    let mut backend = fixtures::resumable();
    backend.settings.parent = parent;
    backend.settings.environment = Box::new([b"KEEP=value".as_slice().into()]);
    backend
}

fn launches(parent: Option<u32>) -> [kernel::Spawn; 2] {
    let mut backend = configured(parent);
    backend.startup = super::startup::Startup::new();
    let request = fixtures::request(&backend);
    let mut below = Queue::uncapped();
    backend.from_domain(&fixtures::env(0), request, &mut Queue::uncapped(), &mut below);
    let status = core::iter::from_fn(|| below.pop())
        .find_map(|request| if let Below::Io(io::Request::Spawn { spawn, .. }) = request { Some(spawn) } else { None })
        .expect("status launch");
    let mut backend = configured(parent);
    fixtures::fresh_window(&mut backend);
    let print = backend.launch_spec().expect("print launch");
    [status, print]
}

#[test]
fn both_launches_wrap_only_program_and_arguments() {
    let direct = launches(None);
    for (parent, program, prefix) in [
        (None, b"/claude".as_slice(), Box::<[&[u8]]>::default()),
        (Some(4242), b"/agent".as_slice(), Box::new([b"--program-child".as_slice(), b"4242", b"/claude"])),
    ] {
        for (launch, direct) in launches(parent).iter().zip(&direct) {
            assert_eq!(launch.program.as_ref(), program, "parent={parent:?}");
            let expected: Box<[_]> = prefix.iter().copied().chain(direct.args.iter().map(AsRef::as_ref)).collect();
            let arguments: Box<[_]> = launch.args.iter().map(AsRef::as_ref).collect();
            assert_eq!(arguments, expected, "parent={parent:?}");
            assert_eq!(launch.env, direct.env, "environment unchanged");
            assert_eq!(
                launch
                    .env
                    .iter()
                    .filter(|entry| entry.starts_with(b"ANTHROPIC_"))
                    .map(AsRef::as_ref)
                    .collect::<Box<[_]>>()
                    .as_ref(),
                &[b"ANTHROPIC_CONFIG_DIR=/programs/source/anthropic".as_slice()]
            );
            assert_eq!(launch.dir, direct.dir, "directory unchanged");
            assert_eq!(launch.root, direct.root, "root unchanged");
            assert_eq!(launch.pipes, direct.pipes, "pipes unchanged");
        }
    }
}
