use alloc::boxed::Box;
use jig_domain_host::{self as jig, sources};
use skein_io::kernel::Fd;
use skein_lib::{Queue, Time, Token};

const CANARY: &[u8] = b"canary-7f3a9c";

fn resources() -> crate::sources::Resources {
    resources_at(Some(Fd::new(8)))
}

fn resources_at(parent: Option<Fd>) -> crate::sources::Resources {
    let bytes = b"API_KEY=".len().checked_add(CANARY.len()).expect("entry length fits");
    let mut entry = skein_lib::Writer::new(bytes);
    entry.put(b"API_KEY=").expect("environment key fits");
    entry.put(CANARY).expect("canary fits");
    crate::sources::Resources::new(
        smith_protocol_program::Programs {
            directory: Box::from(b"/private/logins".as_slice()),
            claude: Box::from(b"/private/program".as_slice()),
        },
        Box::new([entry.finish()]),
        1024,
        1000,
        parent,
    )
    .expect("explicit command resources fit")
}

#[test]
fn program_resources_debug_shows_only_counts() {
    let resources = resources();
    let debug = alloc::fmt::format(format_args!("{resources:?}"));
    assert_eq!(debug, "Resources { programs: 1, environment_entries: 1 }");
    assert!(skein_lib::bytes::find(debug.as_bytes(), CANARY).is_none());
    let canary_debug = alloc::fmt::format(format_args!("{CANARY:?}"));
    assert!(!debug.contains(canary_debug.trim_matches(['[', ']'])));
}

#[test]
fn a_configured_component_formats_without_the_environment() {
    let mut component = crate::sources::Component::new(64);
    component.configure(resources()).expect("idle component accepts resources");
    let debug = alloc::fmt::format(format_args!("{component:?}"));
    assert!(skein_lib::bytes::find(debug.as_bytes(), CANARY).is_none());
    let canary_debug = alloc::fmt::format(format_args!("{CANARY:?}"));
    assert!(!debug.contains(canary_debug.trim_matches(['[', ']'])));
    assert!(debug.contains("environment_entries: 1"));
}

#[test]
fn a_stored_name_smith_refuses_answers_unconfigured() {
    let mut component = crate::sources::Component::new(64);
    component.configure(resources()).expect("idle component accepts resources");
    let name = jig::SourceName::from_stored(b"Upper-Case").expect("stored name fits");
    let owner = Token::new(1);
    let io_owner = Token::new(2);
    for (operation, expected) in [
        (
            sources::Operation::Check,
            sources::Answer::Checked(jig::LoginState::Failed {
                reason: jig::SourceFailure::Unconfigured,
                detail: Box::from(b"invalid source name".as_slice()),
            }),
        ),
        (
            sources::Operation::SignOut,
            sources::Answer::SignedOut(jig::SignOutResult::Failed {
                reason: jig::SourceFailure::Unconfigured,
                detail: Box::from(b"invalid source name".as_slice()),
            }),
        ),
    ] {
        let mut below = Queue::uncapped();
        let (answered, answer) = component
            .start(owner, io_owner, name, operation, Fd::new(9), Time::ZERO, &mut below, &mut Queue::uncapped())
            .expect("source answer");
        assert_eq!(answered, owner);
        assert_eq!(answer, expected);
        assert!(below.is_empty());
        assert!(component.settled());
    }
}

#[test]
fn an_unavailable_programs_parent_answers_without_io() {
    let mut component = crate::sources::Component::new(8);
    component.configure(resources_at(None)).expect("startup retains the absent parent");
    let mut below = Queue::uncapped();
    let mut files = Queue::uncapped();
    let (owner, answer) = component
        .start(
            Token::new(1),
            Token::new(2),
            jig::SourceName::new(b"alice").expect("source name"),
            sources::Operation::Check,
            Fd::new(9),
            Time::ZERO,
            &mut below,
            &mut files,
        )
        .expect("unconfigured answer");
    assert_eq!(owner, Token::new(1));
    let sources::Answer::Checked(jig::LoginState::Failed { reason, detail }) = answer else {
        panic!("missing parent is unconfigured");
    };
    assert_eq!(reason, jig::SourceFailure::Unconfigured);
    assert_eq!(detail.as_ref(), b"programs");
    assert!(below.is_empty() && files.is_empty(), "no child and no file creation");
    assert!(component.settled());
}

#[test]
fn an_inspection_deadline_cancels_in_the_workers_namespace() {
    use skein_io::kernel::{Complete, Done, Error, Op};
    let mut component = crate::sources::Component::new(32);
    component.configure(resources()).expect("configured source");
    let prefix = (5_u64 << 60_u32) | (1_u64 << 48_u32);
    let mut stage = crate::sources::FileStage::new(prefix);
    let mut below = Queue::uncapped();
    let mut submitted = Queue::uncapped();
    let deadline = Time::from_nanos(100);
    assert!(
        component
            .start(
                Token::new(1),
                Token::new(2),
                jig::SourceName::new(b"alice").expect("source name"),
                sources::Operation::Check,
                Fd::new(9),
                deadline,
                &mut below,
                stage.requests()
            )
            .is_none()
    );
    assert!(stage.step(Time::ZERO, &mut component, &mut below, &mut submitted).is_none());
    let opening = submitted.pop().expect("inspection open");
    assert!(component.fire(deadline, &mut below).is_none());
    assert!(stage.step(deadline, &mut component, &mut below, &mut submitted).is_none());
    let cancel = submitted.pop().expect("deadline cancel");
    let Op::Cancel { target } = &cancel.kind else { panic!("file deadline cancels") };
    assert_eq!(*target, opening.op, "cancel targets the worker's submitted open");
    stage.complete(Complete { op: cancel.op, kind: cancel.kind, result: Ok(Done::Nothing) }).expect("owned cancel");
    stage.complete(Complete { op: opening.op, kind: opening.kind, result: Err(Error::Cancelled) }).expect("owned open");
    let (owner, answer) = stage.step(deadline, &mut component, &mut below, &mut submitted).expect("settled deadline");
    assert_eq!(owner, Token::new(1));
    let sources::Answer::Checked(jig::LoginState::Failed { reason, .. }) = answer else { panic!("deadline answer") };
    assert_eq!(reason, jig::SourceFailure::Deadline);
    assert!(below.is_empty() && submitted.is_empty());
    assert!(component.settled() && stage.settled());
}
