//! Object stores and borrowing repositories (protocol/git.md, 4.1–4.3, 4.11).
use super::fixtures::{self, Started};
use crate::{
    Request,
    invocation::{self, Command, Format},
};
use alloc::boxed::Box;
use skein_io::kernel::{Done, Error, Fd, Op, OpenHow};
use skein_lib::Token;
use temper_worker_domain_checkout::git;

fn invocation(s: &mut Started, command: &Command, directory: &[u8], header: bool, named: bool, stdout: &[u8]) {
    invocation_program(s, command, directory, header, named, stdout, b"/bin/git");
}
fn invocation_program(
    s: &mut Started,
    command: &Command,
    directory: &[u8],
    header: bool,
    named: bool,
    stdout: &[u8],
    program: &[u8],
) {
    let mut seen = false;
    for request in &s.below.out.io {
        if let skein_io::Request::Spawn { spawn, .. } = request {
            assert_eq!(spawn.args, invocation::arguments(command));
            assert_eq!(spawn.dir.as_ref(), directory);
            assert_eq!(spawn.env.len(), if header { 12 } else { 9 });
            let committer =
                if named { b"GIT_COMMITTER_NAME=Temper Bot".as_slice() } else { b"GIT_COMMITTER_NAME=temper" };
            let mut got_header = false;
            let mut got_committer = false;
            for entry in &spawn.env {
                got_header |= entry.starts_with(b"GIT_CONFIG_COUNT=");
                got_committer |= entry.as_ref() == committer;
                assert!(!entry.starts_with(b"GIT_AUTHOR_"));
            }
            assert_eq!(got_header, header);
            assert!(got_committer);
            seen = true;
        }
    }
    assert!(seen, "one invocation requested");
    let (child, pipes) = s.below.spawned_program(&mut s.component, s.now, program);
    s.stdout(pipes[0], stdout);
    s.end(pipes[0]);
    s.end(pipes[1]);
    s.exited(child, 0);
    s.closed(child, &pipes);
}
fn request(s: &mut Started, op: git::Op, identity: bool) {
    s.down(Request::Io {
        owner: fixtures::OWNER,
        op,
        deadline: fixtures::DEADLINE,
        identity: if identity { Some(fixtures::identity()) } else { None },
    });
}
#[test]
fn clone_reads_the_stores_format_and_writes_whole_relative_alternates_before_origin() {
    for format in [Format::Sha1, Format::Sha256] {
        let mut s = Started::new();
        request(
            &mut s,
            git::Op::Clone { at: fixtures::at(), remote: Box::from(fixtures::REMOTE), store: Token::new(9) },
            false,
        );
        let output = invocation::joined(&[format.name(), b"\n"]);
        invocation(&mut s, &Command::Format, b"stores/9", false, false, &output);
        invocation(
            &mut s,
            &Command::Clone { directory: Box::from(&b"app"[..]), format },
            b"workspaces/7",
            false,
            false,
            b"",
        );
        assert_eq!(
            s.file_operation(),
            Op::Open {
                root: Fd::new(3),
                path: Box::from(&b"workspaces/7/app/.git/objects/info/alternates"[..]),
                how: OpenHow::CreateNoFollow { mode: Some(0o600) }
            }
        );
        s.complete_ok();
        let Op::Write { bytes, from: 0, at: 0, .. } = s.file_operation() else { panic!("whole alternates write") };
        assert_eq!(bytes.as_ref(), b"../../../../../stores/9/objects\n");
        s.complete(Ok(Done::Count(1)));
        let Op::Write { bytes, from: 1, at: 1, .. } = s.file_operation() else { panic!("short write continued") };
        assert_eq!(bytes.as_ref(), b"../../../../../stores/9/objects\n");
        s.complete_ok();
        let Op::Close { .. } = s.file_operation() else { panic!("alternates closed before origin") };
        s.complete_ok();
        invocation(
            &mut s,
            &Command::Remote { remote: Box::from(fixtures::REMOTE) },
            b"workspaces/7/app",
            false,
            false,
            b"",
        );
        assert_eq!(s.done(), Some((fixtures::OWNER, git::Done::Succeeded)));
    }
    for malformed in [b"sha1".as_slice(), b"sha256\n\n", b"sha512\n"] {
        let mut s = Started::new();
        request(
            &mut s,
            git::Op::Clone { at: fixtures::at(), remote: Box::from(fixtures::REMOTE), store: Token::new(9) },
            false,
        );
        invocation(&mut s, &Command::Format, b"stores/9", false, false, malformed);
        let Some((_, git::Done::FailedWithOutput { fault: git::Fault::Broken, .. })) = s.done() else {
            panic!("only exact format lines admitted")
        };
    }
}
#[test]
fn fetch_uses_its_store_then_points_the_repository_without_a_second_header() {
    let mut s = Started::new();
    s.down(fixtures::fetch(fixtures::OWNER, fixtures::DEADLINE));
    invocation(
        &mut s,
        &Command::Fetch {
            remote: Box::from(fixtures::REMOTE),
            source: invocation::Source::Default,
            into: git::Into::Fetched,
            workspace: fixtures::WORKSPACE,
        },
        b"stores/9",
        true,
        true,
        b"* 0000000000000000000000000000000000000000 abababababababababababababababababababab refs/temper/7/fetched\n",
    );
    invocation(
        &mut s,
        &Command::UpdateRef { into: git::Into::Fetched, commit: fixtures::COMMIT },
        b"workspaces/7/app",
        false,
        true,
        b"",
    );
    assert_eq!(s.done(), Some((fixtures::OWNER, git::Done::Fetched { commit: fixtures::COMMIT })));
}
#[test]
fn a_store_clone_installs_from_making_without_replacing_an_existing_store() {
    for raced in [false, true] {
        let mut s = Started::new();
        request(
            &mut s,
            git::Op::Store {
                store: Token::new(9),
                aside: None,
                remote: Box::from(fixtures::REMOTE),
                identity: 1,
                account: None,
                git: None,
            },
            true,
        );
        assert_eq!(s.file_operation(), store_open());
        s.complete_err(Error::NotFound);
        assert_eq!(
            s.file_operation(),
            Op::MakeDirectory { dir: fixtures::MAKING, name: Box::from(&b"1000"[..]), mode: 0o700 }
        );
        s.complete_ok();
        invocation(
            &mut s,
            &Command::Store { directory: Box::from(&b"1000"[..]), remote: Box::from(fixtures::REMOTE) },
            b"making",
            true,
            true,
            b"",
        );
        assert_eq!(
            s.file_operation(),
            Op::Rename {
                from_dir: fixtures::MAKING,
                from: Box::from(&b"1000"[..]),
                to_dir: fixtures::STORES,
                to: Box::from(&b"9"[..]),
                no_replace: true
            }
        );
        if raced {
            s.complete_err(Error::Exists);
            assert_eq!(
                s.file_operation(),
                Op::Rename {
                    from_dir: fixtures::MAKING,
                    from: Box::from(&b"1000"[..]),
                    to_dir: fixtures::TRASH,
                    to: Box::from(&b"1001"[..]),
                    no_replace: true
                }
            );
        }
        s.complete_ok();
        assert_eq!(s.done(), Some((fixtures::OWNER, git::Done::Succeeded)));
    }
}
#[test]
fn make_sets_aside_its_references_in_each_store_and_accepts_missing_ones() {
    let mut s = Started::new();
    request(
        &mut s,
        git::Op::Make {
            workspace: fixtures::WORKSPACE,
            stores: Box::new([Token::new(9), Token::new(10), Token::new(11)]),
        },
        false,
    );
    s.file_operation();
    s.complete_ok();
    s.file_operation();
    s.complete_ok();
    for (path, missing) in [(b"stores/9/refs/temper".as_slice(), false), (b"stores/10/refs/temper", true)] {
        let expected = Box::from(path);
        assert_eq!(s.file_operation(), Op::Open { root: Fd::new(3), path: expected, how: OpenHow::DirectoryNoFollow });
        s.complete(Ok(Done::Fd(Fd::new(8))));
        let Op::Rename { from_dir, from, to_dir, to, no_replace: true } = s.file_operation() else {
            panic!("references moved")
        };
        assert_eq!((from_dir, from.as_ref(), to_dir), (Fd::new(8), b"7".as_slice(), fixtures::TRASH));
        assert_eq!(to.as_ref(), if missing { b"1002" } else { b"1001" });
        if missing {
            s.complete_err(Error::NotFound);
        } else {
            s.complete_ok();
        }
        assert_eq!(s.file_operation(), Op::Close { fd: Fd::new(8) });
        s.complete_ok();
    }
    let Op::Open { path, .. } = s.file_operation() else { panic!("third store") };
    assert_eq!(path.as_ref(), b"stores/11/refs/temper");
    s.complete_err(Error::NotFound);
    assert_eq!(s.done(), Some((fixtures::OWNER, git::Done::Succeeded)));
}

fn store_op(aside: Option<Token>) -> git::Op {
    git::Op::Store {
        store: Token::new(9),
        aside,
        remote: Box::from(fixtures::REMOTE),
        identity: 1,
        account: None,
        git: None,
    }
}
#[test]
fn make_skips_occupied_trash_names_when_retiring_a_kept_stores_references() {
    let mut s = Started::new();
    request(&mut s, git::Op::Make { workspace: fixtures::WORKSPACE, stores: Box::new([Token::new(9)]) }, false);
    s.file_operation();
    s.complete_err(Error::NotFound);
    s.file_operation();
    s.complete_ok();
    s.file_operation();
    s.complete(Ok(Done::Fd(Fd::new(8))));
    for name in [b"1001".as_slice(), b"1002"] {
        assert_eq!(
            s.file_operation(),
            Op::Rename {
                from_dir: Fd::new(8),
                from: Box::from(&b"7"[..]),
                to_dir: fixtures::TRASH,
                to: Box::from(name),
                no_replace: true,
            }
        );
        s.complete_err(Error::Exists);
        assert_eq!(s.done(), None);
    }
    let Op::Rename { to, no_replace: true, .. } = s.file_operation() else { panic!("next unused trash name") };
    assert_eq!(to.as_ref(), b"1003");
    s.complete_ok();
    assert_eq!(s.file_operation(), Op::Close { fd: Fd::new(8) });
    s.complete_ok();
    assert_eq!(s.done(), Some((fixtures::OWNER, git::Done::Succeeded)));
}
fn store_open() -> Op {
    Op::Open { root: fixtures::STORES, path: Box::from(&b"9"[..]), how: OpenHow::DirectoryNoFollow }
}
#[test]
fn a_store_already_there_is_closed_and_succeeds_without_making_or_invoking_git() {
    for (aside, program) in [(None, None), (Some(Token::new(8)), Some(b"/usr/bin/item-git".as_slice()))] {
        let mut s = Started::new();
        let mut op = store_op(aside);
        let git::Op::Store { git, .. } = &mut op else { panic!("Store fixture") };
        *git = match program {
            Some(program) => Some(Box::from(program)),
            None => None,
        };
        request(&mut s, op, true);
        assert!(s.below.out.io.is_empty(), "store lookup precedes even an item's version check");
        if aside.is_some() {
            assert_eq!(
                s.file_operation(),
                Op::Rename {
                    from_dir: fixtures::STORES,
                    from: Box::from(&b"8"[..]),
                    to_dir: fixtures::TRASH,
                    to: Box::from(&b"1000"[..]),
                    no_replace: true,
                }
            );
            s.complete_ok();
        }
        assert_eq!(s.file_operation(), store_open());
        s.complete(Ok(Done::Fd(Fd::new(8))));
        assert_eq!(s.done(), None, "close before terminal");
        assert_eq!(s.file_operation(), Op::Close { fd: Fd::new(8) });
        s.complete_ok();
        assert_eq!(s.done(), Some((fixtures::OWNER, git::Done::Succeeded)));
        assert!(s.below.out.io.is_empty(), "no invocation");
        assert!(s.below.out.kernel.is_empty(), "nothing in making");
    }
}
#[test]
fn a_store_check_refuses_non_directories_and_other_open_errors() {
    for error in [Error::NotADirectory, Error::TooManyLinks, Error::Permission] {
        let mut s = Started::new();
        request(&mut s, store_op(None), true);
        assert_eq!(s.file_operation(), store_open());
        s.complete_err(error);
        assert_eq!(s.done(), Some((fixtures::OWNER, git::Done::Failed { fault: git::Fault::Broken })));
        assert!(s.below.out.io.is_empty());
    }
}
#[test]
fn a_store_check_closes_its_directory_during_shutdown() {
    let mut s = Started::new();
    request(&mut s, store_op(None), true);
    assert_eq!(s.file_operation(), store_open());
    s.down(Request::Shutdown);
    s.complete(Ok(Done::Fd(Fd::new(8))));
    assert_eq!(s.file_operation(), Op::Close { fd: Fd::new(8) });
    s.complete_ok();
    for fd in [fixtures::MAKING, fixtures::STORES, fixtures::WORKSPACES, fixtures::TRASH] {
        assert_eq!(s.file_operation(), Op::Close { fd });
        s.complete_ok();
    }
    assert!(s.component.is_empty());
    assert!(s.below.out.io.is_empty());
}
#[test]
fn a_failed_store_clone_discards_making_and_keeps_its_classified_fault() {
    let mut s = Started::new();
    request(&mut s, store_op(None), true);
    assert_eq!(s.file_operation(), store_open());
    s.complete_err(Error::NotFound);
    assert_eq!(
        s.file_operation(),
        Op::MakeDirectory { dir: fixtures::MAKING, name: Box::from(&b"1000"[..]), mode: 0o700 }
    );
    s.complete_ok();
    let (child, pipes) = s.spawned();
    s.stdout(pipes[1], b"fatal: Authentication failed for 'remote'\n");
    s.end(pipes[0]);
    s.end(pipes[1]);
    s.exited(child, 128);
    s.closed(child, &pipes);
    assert_eq!(
        s.file_operation(),
        Op::Rename {
            from_dir: fixtures::MAKING,
            from: Box::from(&b"1000"[..]),
            to_dir: fixtures::TRASH,
            to: Box::from(&b"1001"[..]),
            no_replace: true,
        }
    );
    assert_eq!(s.done(), None);
    s.complete_ok();
    let Some((fixtures::OWNER, git::Done::FailedWithOutput { fault: git::Fault::Refused, .. })) = s.done() else {
        panic!("clone's authentication fault survives discard")
    };
}
#[test]
fn a_missing_store_checks_its_item_program_before_cloning_and_discards_on_a_bad_version() {
    for (version, usable) in [(b"git version 2.53.0\n".as_slice(), true), (b"git version 2.40.0\n", false)] {
        let mut s = Started::new();
        let mut op = store_op(None);
        let git::Op::Store { git, .. } = &mut op else { panic!("Store fixture") };
        *git = Some(Box::from(&b"/usr/bin/item-git"[..]));
        request(&mut s, op, true);
        assert_eq!(s.file_operation(), store_open());
        s.complete_err(Error::NotFound);
        assert_eq!(
            s.file_operation(),
            Op::MakeDirectory { dir: fixtures::MAKING, name: Box::from(&b"1000"[..]), mode: 0o700 }
        );
        s.complete_ok();
        invocation_program(&mut s, &Command::Version, b"", false, false, version, b"/usr/bin/item-git");
        if usable {
            invocation_program(
                &mut s,
                &Command::Store { directory: Box::from(&b"1000"[..]), remote: Box::from(fixtures::REMOTE) },
                b"making",
                true,
                true,
                b"",
                b"/usr/bin/item-git",
            );
        }
        assert_eq!(
            s.file_operation(),
            Op::Rename {
                from_dir: fixtures::MAKING,
                from: Box::from(&b"1000"[..]),
                to_dir: if usable { fixtures::STORES } else { fixtures::TRASH },
                to: Box::from(if usable { b"9".as_slice() } else { b"1001" }),
                no_replace: true,
            }
        );
        s.complete_ok();
        assert_eq!(
            s.done(),
            Some((
                fixtures::OWNER,
                if usable { git::Done::Succeeded } else { git::Done::Failed { fault: git::Fault::Broken } }
            ))
        );
    }
}
