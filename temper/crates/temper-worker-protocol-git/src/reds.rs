//! A check's base, contained heads and git paths (domain/forge/reds.md, 5.2).
use crate::{
    invocation::{self, Command},
    sequence::{self, Next},
};
use alloc::boxed::Box;
use skein_lib::{Decimal, List, Writer};
use temper_worker_domain_checkout::git;
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum CheckCommand {
    Pin,
    Merge,
    Base { merging: bool },
    Ancestor { expected: Box<[u8]>, base: git::Commit, width: usize, index: usize, kept: Box<[usize]> },
    Path { kept: Box<[usize]> },
}
pub(crate) fn first(op: &git::Op) -> Next {
    if let git::Op::Reds { action: git::Reds::Clear, .. } = op {
        return path(Box::new([]));
    }
    run(CheckCommand::Pin)
}
pub(crate) fn arguments(command: &CheckCommand) -> Box<[Box<[u8]>]> {
    let mut args = List::with_capacity(8);
    match command {
        CheckCommand::Pin => {
            for word in [b"rev-parse".as_slice(), b"--verify", b"--quiet", b"refs/temper/landing^{commit}"] {
                args.push(Box::from(word)).expect("fixed argv");
            }
        }
        CheckCommand::Merge => {
            for word in [b"rev-parse".as_slice(), b"--verify", b"--quiet", b"MERGE_HEAD"] {
                args.push(Box::from(word)).expect("fixed argv");
            }
        }
        CheckCommand::Base { merging } => {
            for word in [b"merge-base".as_slice(), b"refs/temper/landing", b"HEAD"] {
                args.push(Box::from(word)).expect("fixed argv");
            }
            if *merging {
                args.push(Box::from(b"MERGE_HEAD".as_slice())).expect("fixed argv");
            }
        }
        CheckCommand::Ancestor { expected, base, .. } => {
            for word in [b"merge-base".as_slice(), b"--is-ancestor"] {
                args.push(Box::from(word)).expect("argv");
            }
            args.push(head_hex(expected)).expect("argv");
            args.push(invocation::hex(*base)).expect("argv");
        }
        CheckCommand::Path { .. } => {
            for word in [b"rev-parse".as_slice(), b"--path-format=relative", b"--git-path", b"temper"] {
                args.push(Box::from(word)).expect("fixed argv");
            }
        }
    }
    args.into_boxed()
}
fn run(command: CheckCommand) -> Next {
    Next::Run(Command::Reds(command))
}
fn path(kept: Box<[usize]>) -> Next {
    run(CheckCommand::Path { kept })
}
pub(crate) fn after(op: &git::Op, command: &CheckCommand, code: u8, stdout: &[u8]) -> Next {
    let git::Op::Reds { at, action } = op else { unreachable!("check sequence") };
    match command {
        CheckCommand::Pin => {
            if code != 0 {
                return Next::End(git::Done::Succeeded);
            }
            match action {
                git::Reds::Write { .. } => run(CheckCommand::Merge),
                git::Reds::Prepare { .. } | git::Reds::Clear | git::Reds::Read { .. } => path(Box::new([])),
            }
        }
        CheckCommand::Merge => {
            if code > 1 {
                return sequence::failed(git::Fault::Broken, b"reds merge head", 0);
            }
            run(CheckCommand::Base { merging: code == 0 })
        }
        CheckCommand::Base { .. } => {
            let base = if code == 0 {
                let end = skein_lib::bytes::find(stdout, b"\n").unwrap_or(stdout.len());
                let line = stdout.get(..end).expect("first line");
                let Some(base) = invocation::object(line) else { return path(Box::new([])) };
                Some((base, line.len().checked_div(2).expect("object width")))
            } else {
                None
            };
            let git::Reds::Write { reds } = action else { unreachable!("check base") };
            match base {
                Some((base, width)) => query(reds, base, width, 0, Box::new([])),
                None => path(Box::new([])),
            }
        }
        CheckCommand::Ancestor { base, width, index, kept, .. } => {
            let git::Reds::Write { reds } = action else { unreachable!("check ancestry") };
            let mut accepted = List::with_capacity(u32::try_from(reds.len()).expect("bounded reds"));
            for old in kept {
                accepted.push(*old).expect("red count");
            }
            // A missing object returns 128. Only positive ancestry counts.
            if code == 0 {
                accepted.push(*index).expect("one query per red");
            }
            query(reds, *base, *width, index.checked_add(1).expect("bounded reds"), accepted.into_boxed())
        }
        CheckCommand::Path { kept } => {
            if code != 0 {
                return sequence::failed(git::Fault::Broken, b"reds git path", 0);
            }
            let Some(directory) = stdout.strip_suffix(b"\n") else {
                return sequence::failed(git::Fault::Broken, b"reds git path", 0);
            };
            if directory.is_empty() || directory.contains(&0) {
                return sequence::failed(git::Fault::Broken, b"reds git path", 0);
            }
            let directory = if directory.starts_with(b"/") {
                Box::from(directory)
            } else {
                invocation::joined(&[&invocation::place(at), b"/", directory])
            };
            let (content, marker) = match action {
                git::Reds::Read { .. } => return Next::Results { directory },
                git::Reds::Clear => (None, false),
                git::Reds::Prepare { wait_ms, .. } => (
                    Some(invocation::joined(&[
                        b"temper-reds-wait 1 ",
                        Decimal::of(u64::from(*wait_ms)).as_bytes(),
                        b"\n",
                    ])),
                    true,
                ),
                git::Reds::Write { reds } => (Some(contents(reds, kept)), false),
            };
            let rerun = match action {
                git::Reds::Prepare { rerun_processes, .. } => Some(*rerun_processes),
                git::Reds::Clear | git::Reds::Read { .. } | git::Reds::Write { .. } => None,
            };
            Next::Reds { directory, content, marker, rerun }
        }
    }
}
fn query(reds: &[git::Red], base: git::Commit, width: usize, from: usize, kept: Box<[usize]>) -> Next {
    for (index, red) in reds.iter().enumerate().skip(from) {
        if red.head.len() == width {
            return run(CheckCommand::Ancestor { expected: red.head.clone(), base, width, index, kept });
        }
    }
    path(kept)
}
fn contents(reds: &[git::Red], kept: &[usize]) -> Box<[u8]> {
    let mut keys = skein_lib::Set::with_capacity(u32::try_from(kept.len()).expect("red count"));
    for index in kept {
        keys.insert(reds.get(*index).expect("queried red").key.clone()).expect("red count");
    }
    let mut size = b"temper-reds 1\n".len();
    for key in &keys {
        size = size.checked_add(key.len()).expect("bounded keys").checked_add(3).expect("line separators");
    }
    let mut out = Writer::new(size);
    out.put(b"temper-reds 1\n").expect("measured header");
    for key in &keys {
        for part in [b"- ".as_slice(), key, b"\n"] {
            out.put(part).expect("measured keys");
        }
    }
    out.finish()
}
fn head_hex(raw: &[u8]) -> Box<[u8]> {
    let mut out = Writer::new(raw.len().checked_mul(2).expect("bounded object id"));
    for byte in raw {
        for digit in [byte >> 4_u32, byte & 15] {
            let byte = if digit < 10 {
                b'0'.checked_add(digit)
            } else {
                b'a'.checked_add(digit.checked_sub(10).expect("hex letter"))
            }
            .expect("hex digit");
            out.put(&[byte]).expect("measured object id");
        }
    }
    out.finish()
}
