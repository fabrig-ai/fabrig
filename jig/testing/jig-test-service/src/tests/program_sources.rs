use alloc::boxed::Box;
use jig_domain_host::{self as jig, sources};
use skein_io::kernel::Fd;
use skein_lib::{Queue, Time, Token};

const CANARY: &[u8] = b"canary-7f3a9c";

fn resources() -> crate::program_sources::Resources {
    let bytes = b"API_KEY=".len().checked_add(CANARY.len()).expect("entry length fits");
    let mut entry = skein_lib::Writer::new(bytes);
    entry.put(b"API_KEY=").expect("environment key fits");
    entry.put(CANARY).expect("canary fits");
    crate::program_sources::Resources::from_paths(
        Box::from(b"/private/logins".as_slice()),
        Box::from(b"/private/program".as_slice()),
        Box::new([entry.finish()]),
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
    let mut component = crate::program_sources::Component::new(64);
    component.configure(resources()).expect("idle component accepts resources");
    let debug = alloc::fmt::format(format_args!("{component:?}"));
    assert!(skein_lib::bytes::find(debug.as_bytes(), CANARY).is_none());
    let canary_debug = alloc::fmt::format(format_args!("{CANARY:?}"));
    assert!(!debug.contains(canary_debug.trim_matches(['[', ']'])));
    assert!(debug.contains("environment_entries: 1"));
}

#[test]
fn a_stored_name_smith_refuses_answers_unconfigured() {
    let mut component = crate::program_sources::Component::new(64);
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
        let mut above = Queue::uncapped();
        let mut below = Queue::uncapped();
        component.start(owner, io_owner, name, operation, Fd::new(9), Time::ZERO, &mut above, &mut below);
        let jig::Event::SourceAnswered { owner: answered, answer } = above.pop().expect("source answer") else {
            panic!("expected a source answer");
        };
        assert_eq!(answered, owner);
        assert_eq!(answer, expected);
        assert!(above.is_empty());
        assert!(below.is_empty());
        assert!(component.settled());
    }
}
