#![expect(clippy::disallowed_methods, reason = "ordinary Rust fault harness, programming-model.md 10.2")]
use super::super::{BelowEvent, fixtures};
use super::*;

fn claimed_child() -> Backend {
    let mut backend = fixtures::running_child();
    fixtures::claim_completion(fixtures::COMPLETION, &mut backend);
    backend
}

fn ending() -> Backend {
    let mut backend = fixtures::active();
    backend.settings.directory = Some(super::super::Root { file: Token::new(40), descriptor: kernel::Fd::new(41) });
    backend.files.work = true;
    backend.files.instructions = true;
    backend.files.mcp = true;
    backend.files.remove_run = true;
    backend.close(&fixtures::env(0), &mut Queue::uncapped(), &mut Queue::uncapped());
    assert_eq!(backend.startup.phase(), State::Ending);
    backend
}

fn held_snapshot() -> Backend {
    let mut backend = fixtures::active();
    backend.begin_snapshot(
        fixtures::COMPLETION,
        smith_domain::tools::Place { root: Token::new(40), path: b"a".as_slice().into() },
        super::super::native::After::Result(ToDomain::Cancelled { owner: fixtures::COMPLETION }),
    );
    backend
}

#[test]
fn written_source_settings_do_not_restrict_sign_in() {
    let mut backend = fixtures::resumable();
    backend.files.pending = Some(Step::Mcp);
    let mut below = Queue::uncapped();
    backend.file_event(
        &fixtures::env(0),
        file::Event::Stored { owner: backend.token(Owner::Files), digest: skein_io::digest::digest(b"mcp") },
        None,
        &mut Queue::uncapped(),
        &mut below,
    );
    let Some(Below::File { request: file::Request::Store { root, path, bytes, .. }, .. }) = below.pop() else {
        panic!("source settings written after MCP configuration");
    };
    assert_eq!(root, backend.settings.directory.expect("programs root").file);
    assert_eq!(path.as_ref(), b"source/settings.json");
    let settings: serde_json::Value = serde_json::from_slice(&bytes).expect("settings JSON");
    for key in ["forceLoginMethod", "forceLoginOrgUUID"] {
        assert!(settings.get(key).is_none(), "{key} must not restrict sign-in");
    }
}

#[test]
fn an_answer_not_asked_for_fails_its_run_never_the_agent() {
    let files = Token::new(15);
    let native = Token::new(16);
    let starting: fn() -> Backend = fixtures::starting;
    let loaded = || file::Event::Loaded { owner: native, bytes: b"file bytes".as_slice().into() };
    for (name, fixture, owner, event, descriptor, fails) in [
        (
            "starting Stored",
            starting,
            Owner::Files,
            file::Event::Stored { owner: files, digest: skein_io::digest::digest(b"fixture") },
            None,
            true,
        ),
        (
            "active Opened",
            fixtures::active,
            Owner::Files,
            file::Event::Opened { owner: files, file: Token::new(42), len: 0 },
            Some(kernel::Fd::new(43)),
            true,
        ),
        (
            "active Failed",
            fixtures::active,
            Owner::Files,
            file::Event::Failed { owner: files, error: kernel::Error::NotFound, committed: false, residue: None },
            None,
            true,
        ),
        ("child Closed", claimed_child, Owner::Files, file::Event::Closed { owner: files }, None, true),
        ("active Loaded", fixtures::active, Owner::Native, loaded(), None, true),
        ("ending Removed", ending, Owner::Files, file::Event::Removed { owner: files }, None, false),
        ("ending Loaded", ending, Owner::Native, loaded(), None, false),
        ("held Loaded", held_snapshot, Owner::Native, loaded(), None, true),
    ] {
        let mut backend = fixture();
        assert_eq!(event.owner(), backend.token(owner), "{name}");
        assert_eq!(backend.files.pending, None, "{name}");
        let file_state = |backend: &Backend| {
            (
                backend.files.pending,
                backend.files.work,
                backend.files.instructions,
                backend.files.mcp,
                backend.files.remove_run,
                backend.settings.directory.map(|root| (root.file, root.descriptor)),
            )
        };
        let before = file_state(&backend);
        let held = backend.native.is_some();
        let completion = backend.current.owner();
        let mut up = Queue::uncapped();
        let mut below = Queue::uncapped();
        backend.from_io(&fixtures::env(0), BelowEvent::File { event, descriptor }, &mut up, &mut below);
        if fails {
            let Some(ToDomain::Failed { owner, failure, detail, .. }) = up.pop() else {
                panic!("{name}: one failed completion");
            };
            assert_eq!(Some(owner), completion, "{name}");
            assert_eq!(failure, llm::Failure::Program { exit: None }, "{name}");
            assert!(
                core::str::from_utf8(&detail)
                    .expect("failure text")
                    .contains("smith: a private file answer the program did not ask for"),
                "{name}"
            );
        } else {
            assert_eq!(file_state(&backend), before, "{name}: dropped answer preserves private files");
        }
        assert!(up.is_empty(), "{name}: no other domain event");
        assert_eq!(backend.native.is_some(), held, "{name}: held snapshot is retained");
    }
}

fn failed(committed: bool, residue: bool) -> file::Event {
    file::Event::Failed {
        owner: Token::new(15),
        error: kernel::Error::Permission,
        committed,
        residue: residue
            .then(|| file::Residue { name: b".store-test".as_slice().into(), error: kernel::Error::Permission }),
    }
}
fn prepared() -> Backend {
    let mut below = Queue::with_capacity(64);
    let mut backend = fixtures::backend();
    backend.startup.close(&mut backend.server_up, &mut below);
    backend.settings.directory = Some(super::super::Root { file: Token::new(40), descriptor: kernel::Fd::new(41) });
    backend.files.run = Some(Token::new(42));
    backend.files.run_descriptor = Some(kernel::Fd::new(43));
    backend.files.runs = Some(Token::new(44));
    backend.files.runs_descriptor = Some(kernel::Fd::new(45));
    backend.files.remove_run = true;
    backend
}
#[test]
fn a_committed_store_failure_removes_its_target() {
    let mut backend = prepared();
    let fault = failed(true, false);
    assert!(!backend.files.accept(Step::Mcp, fault, None), "the committed fault was injected");
    assert!(backend.files.mcp, "the committed secret is still owned");
    let mut below = Queue::with_capacity(4);
    backend.cleanup(Time::ZERO, &mut below);
    let Some(Below::File { request: file::Request::Remove { name, .. }, .. }) = below.pop() else {
        panic!("cleanup remove");
    };
    assert_eq!(name.as_ref(), b"mcp.json");
    assert!(backend.files.mcp, "submission does not relinquish ownership");
    assert!(backend.files.accept(Step::RemoveMcp, file::Event::Removed { owner: Token::new(15) }, None));
    assert!(!backend.files.mcp);
}
#[test]
fn a_failed_store_residue_is_removed_before_the_target() {
    let mut backend = prepared();
    assert!(!backend.files.accept(Step::Mcp, failed(true, true), None), "the residue fault was injected");
    let mut below = Queue::with_capacity(4);
    backend.cleanup(Time::ZERO, &mut below);
    let Some(Below::File { request: file::Request::Remove { name, .. }, .. }) = below.pop() else {
        panic!("cleanup remove");
    };
    assert_eq!(name.as_ref(), b".store-test");
    assert!(backend.files.mcp);
}
#[test]
fn a_failed_removal_retains_ownership_for_retry() {
    let mut backend = prepared();
    backend.files.mcp = true;
    let mut below = Queue::with_capacity(4);
    backend.cleanup(Time::ZERO, &mut below);
    drop(below.pop());
    let step = backend.files.pending.take().expect("bounded test fixture");
    assert!(!backend.files.accept(step, failed(false, false), None), "the unlink fault was injected");
    assert!(backend.files.mcp);
    backend.cleanup(Time::ZERO, &mut below);
    let Some(Below::File { request: file::Request::Remove { name, .. }, .. }) = below.pop() else {
        panic!("cleanup remove");
    };
    assert_eq!(name.as_ref(), b"mcp.json");
    assert!(backend.files.accept(step, file::Event::Removed { owner: Token::new(15) }, None));
    assert!(!backend.files.mcp);
}
#[test]
fn an_absent_cleanup_target_relinquishes_ownership() {
    let mut files = Files::new();
    files.mcp = true;
    assert!(files.accept(
        Step::RemoveMcp,
        file::Event::Failed { owner: Token::new(15), error: kernel::Error::NotFound, committed: false, residue: None },
        None
    ));
    assert!(!files.mcp);
}
