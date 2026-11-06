use super::*;

#[test]
fn each_ending_answers_as_hosts_md_says() {
    let detail = b"stderr only";
    for ask in [SourceAsk::Check, SourceAsk::SignOut] {
        for (ended, reason) in [
            (Ended::Unspawned { detail }, FailureReason::Missing),
            (Ended::Deadline { detail }, FailureReason::Deadline),
            (
                Ended::Exited { exit: kernel::Exit::Code(2), stdout: Some(b"stdout secret"), detail },
                FailureReason::Exited,
            ),
            (
                Ended::Exited { exit: kernel::Exit::Signal(9), stdout: Some(b"stdout secret"), detail },
                FailureReason::Exited,
            ),
        ] {
            assert_eq!(translate(ask, ended), failure(ask, reason, detail));
        }
    }
    for (code, stdout, expected) in [
        (0, Some(br#"{"loggedIn":true,"authMethod":"claude.ai"}"#.as_slice()), ProgramCheck::Usable),
        (
            0,
            Some(br#"{"loggedIn":true,"authMethod":"api_key"}"#.as_slice()),
            ProgramCheck::SignedInOtherwise(ProgramMethod::new(b"api_key").expect("method")),
        ),
        (0, Some(br#"{"loggedIn":false,"authMethod":"none"}"#.as_slice()), ProgramCheck::NotSignedIn),
        (1, Some(b"whatever".as_slice()), ProgramCheck::NotSignedIn),
        (
            0,
            Some(b"not json".as_slice()),
            ProgramCheck::Failed(ProgramFailure {
                reason: FailureReason::Unreadable,
                detail: detail.as_slice().into(),
            }),
        ),
        (
            0,
            None,
            ProgramCheck::Failed(ProgramFailure {
                reason: FailureReason::Unreadable,
                detail: detail.as_slice().into(),
            }),
        ),
        (
            1,
            None,
            ProgramCheck::Failed(ProgramFailure {
                reason: FailureReason::Unreadable,
                detail: detail.as_slice().into(),
            }),
        ),
    ] {
        assert_eq!(
            translate(SourceAsk::Check, Ended::Exited { exit: kernel::Exit::Code(code), stdout, detail }),
            SourceAnswer::Check(expected)
        );
    }
    for stdout in [Some(b"ignored".as_slice()), None] {
        assert_eq!(
            translate(SourceAsk::SignOut, Ended::Exited { exit: kernel::Exit::Code(0), stdout, detail }),
            SourceAnswer::SignOut(ProgramSignOut::SignedOut)
        );
        assert_eq!(
            translate(SourceAsk::SignOut, Ended::Exited { exit: kernel::Exit::Code(1), stdout, detail }),
            failure(SourceAsk::SignOut, FailureReason::Exited, detail)
        );
    }
}

#[test]
fn cancellation_answers_only_after_resources_settle() {
    use skein_lib::Token;
    let owner = Token::new(1);
    let mut below = Queue::with_capacity(32);
    let mut source = ProgramSource::new(
        owner,
        SourceAsk::Check,
        &Programs { directory: b"/programs".as_slice().into(), claude: b"/claude".as_slice().into() },
        SourceName::new(b"work").unwrap(),
        &[],
        kernel::Fd::new(3),
        b".".as_slice().into(),
        Time::from_nanos(100),
        1024,
        &mut below,
    )
    .unwrap();
    below.pop().unwrap();
    let pipes = [Token::new(7), Token::new(8)];
    source.from_io(IoEvent::Spawned { owner, child: Token::new(9), pipes: Box::new(pipes) }, &mut below);
    while below.pop().is_some() {}
    source.cancel(&mut below);
    source.cancel(&mut below);
    assert_eq!(below.len(), 3);
    let Some(IoRequest::Signal { signal: kernel::Signal::Kill, .. }) = below.pop() else {
        panic!("cancel kills the child")
    };
    for pipe in pipes {
        let Some(IoRequest::Abort { entity }) = below.pop() else { panic!("cancel aborts each output") };
        assert_eq!(entity, pipe);
    }
    assert_eq!(source.answer(), None);
    source.from_io(IoEvent::Closed { owner }, &mut below);
    source.from_io(IoEvent::Closed { owner: pipes[0] }, &mut below);
    assert_eq!(source.answer(), None);
    source.from_io(IoEvent::Closed { owner: pipes[1] }, &mut below);
    assert_eq!(
        source.answer(),
        Some(SourceAnswer::Check(ProgramCheck::Failed(ProgramFailure {
            reason: FailureReason::Deadline,
            detail: Box::new([])
        })))
    );
    assert_eq!(source.answer(), None);
    source.cancel(&mut below);
    assert!(below.is_empty());
}

fn inspection() -> (SourceInspection, Queue<skein_io::file::Request>) {
    let mut requests = Queue::uncapped();
    let inspection = SourceInspection::new(
        Token::new(1),
        kernel::Fd::new(3),
        b"programs".as_slice().into(),
        SourceName::new(b"work").expect("source name"),
        1000,
        Time::from_nanos(100),
        &mut requests,
    );
    let Some(skein_io::file::Request::OpenDirectory { no_follow: true, name, .. }) = requests.pop() else {
        panic!("inspection opens without following links");
    };
    assert_eq!(name.as_ref(), b"programs");
    (inspection, requests)
}

fn inspection_failed(inspection: &mut SourceInspection, reason: FailureReason) {
    let Some(Err(ProgramCheck::Failed(failure))) = inspection.answer() else { panic!("inspection refuses") };
    assert_eq!(failure.reason, reason);
    assert!(!failure.detail.is_empty());
    assert_eq!(inspection.answer(), None, "one inspection answer");
    inspection.fire(Time::from_nanos(100));
    assert_eq!(inspection.answer(), None, "a settled inspection stays settled");
}

#[test]
fn inspection_refuses_unsafe_metadata_and_open_errors_without_spawning() {
    use skein_io::file::{Event, Request};
    let owner = Token::new(1);
    let private = kernel::Stat { kind: kernel::Kind::Directory, size: 0, mode: 0o700, owner: 1000, links: 1 };
    for stat in [
        kernel::Stat { owner: 1001, ..private },
        kernel::Stat { kind: kernel::Kind::File, ..private },
        kernel::Stat { kind: kernel::Kind::Symlink, ..private },
        kernel::Stat { kind: kernel::Kind::Other, ..private },
        kernel::Stat { mode: 0o710, ..private },
        kernel::Stat { mode: 0o701, ..private },
    ] {
        let (mut inspection, mut requests) = inspection();
        inspection.from_file(Event::Opened { owner, file: Token::new(2), len: 0 }, &mut requests);
        let Some(Request::Stat { .. }) = requests.pop() else { panic!("opened directory is stated") };
        inspection.from_file(Event::Stated { owner, stat }, &mut requests);
        assert_eq!(inspection.answer(), None, "refusal waits for close");
        let Some(Request::Close { .. }) = requests.pop() else { panic!("opened directory is closed") };
        inspection.from_file(Event::Closed { owner }, &mut requests);
        inspection_failed(&mut inspection, FailureReason::Unreadable);
        assert!(requests.is_empty(), "unsafe programs directory never opens the source");
    }
    for error in [kernel::Error::NotADirectory, kernel::Error::Permission, kernel::Error::Other(5)] {
        let (mut inspection, mut requests) = inspection();
        inspection.from_file(Event::Failed { owner, error, committed: false, residue: None }, &mut requests);
        inspection_failed(&mut inspection, FailureReason::Unreadable);
        assert!(requests.is_empty());
    }
}

#[test]
fn inspection_deadline_waits_for_each_open_stat_or_close_to_settle() {
    use skein_io::file::{Event, Request};
    let owner = Token::new(1);
    for phase in 0_u8..3 {
        let (mut inspection, mut requests) = inspection();
        if phase > 0 {
            inspection.from_file(Event::Opened { owner, file: Token::new(2), len: 0 }, &mut requests);
            let Some(Request::Stat { .. }) = requests.pop() else { panic!("opened directory is stated") };
        }
        if phase > 1 {
            inspection.from_file(
                Event::Stated {
                    owner,
                    stat: kernel::Stat { kind: kernel::Kind::Directory, size: 0, mode: 0o700, owner: 1000, links: 1 },
                },
                &mut requests,
            );
            let Some(Request::Close { .. }) = requests.pop() else { panic!("opened directory is closed") };
        }
        inspection.fire(Time::from_nanos(99));
        assert_eq!(inspection.next_deadline(), Some(Time::from_nanos(100)));
        inspection.fire(Time::from_nanos(100));
        assert_eq!(inspection.next_deadline(), None);
        assert_eq!(inspection.answer(), None, "deadline waits for the file terminal");
        if phase == 0 {
            inspection.from_file(Event::Opened { owner, file: Token::new(2), len: 0 }, &mut requests);
            let Some(Request::Close { .. }) = requests.pop() else { panic!("opened directory is closed") };
        } else if phase == 1 {
            inspection.from_file(
                Event::Stated {
                    owner,
                    stat: kernel::Stat { kind: kernel::Kind::Directory, size: 0, mode: 0o755, owner: 1000, links: 1 },
                },
                &mut requests,
            );
            let Some(Request::Close { .. }) = requests.pop() else { panic!("opened directory is closed") };
        }
        inspection.from_file(Event::Closed { owner }, &mut requests);
        inspection_failed(&mut inspection, FailureReason::Deadline);
        assert!(requests.is_empty(), "deadline never opens the source");
    }
}
