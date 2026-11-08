use super::fixtures as f;
use crate::jig;
use jig_client::{self as client, contract as c};
use skein_lib::{Duration, List};
use temper_client_domain::forge::{self, forges::Connection, making::Making};
use temper_client_protocol_wire::{self as wire, v1 as w};

const MAKE: &[&str] =
    &["repository", "make", "--project", "shop", "--forge", "3", "team/widget", "--key", "make-widget"];
const COPY: &[&str] = &[
    "repository",
    "copy",
    "--project",
    "shop",
    "--forge",
    "3",
    "https://source.invalid/team/widget.git",
    "--key",
    "copy-widget",
];

fn request(root: &f::Harness, call: u32) -> w::MakeRepository {
    let client::Ask::Family { scope, record } = root.adoption_bytes(call) else { panic!("family request") };
    assert_eq!(*scope, c::Scope::Deployment);
    let request = wire::decode_requesting(&record.bytes, &w::CEILINGS).unwrap();
    let w::Request::MakeRepository(make) = request.into_parts().request else { panic!("make repository") };
    make
}
fn extended(words: &[&str], flags: &[&str]) -> jig::call::Call {
    let mut all = List::with_capacity(24);
    for word in words.iter().chain(flags) {
        all.push(word.as_bytes()).unwrap();
    }
    let jig::call::Parsed::Call { call, .. } = crate::table::parse(all.as_slice()).unwrap() else { panic!("call") };
    call
}
fn waiting(root: &mut f::Harness, flags: &[&str], json: bool) -> u32 {
    let call = root.submit_prepared(extended(MAKE, flags), json, None);
    root.adoption_answer(call, w::Outcome::MakeKept);
    assert!(root.answer(call).is_none());
    assert_eq!(
        root.progress.as_slice().last().unwrap().as_ref(),
        if json { b"{\"make_kept\":{}}\n".as_slice() } else { b"kept\n" }
    );
    call
}
fn failed() -> w::MakingState {
    w::MakingState::Failed(
        w::MakingFailed::new(
            &w::CEILINGS,
            w::MakingFailedParts {
                step: w::MakeStep::Repository,
                reason: w::MakeFailure::Taken,
                words: b"name is taken".as_slice().into(),
            },
        )
        .unwrap(),
    )
}
fn making(state: w::MakingState) -> Making {
    Making {
        org: b"team".as_slice().into(),
        name: b"widget".as_slice().into(),
        from: w::MakeFrom::Empty,
        into: Some(w::MakeInto::new(&w::CEILINGS, w::MakeIntoParts { project: 1, home: true }).unwrap()),
        state,
    }
}
fn deliver(root: &mut f::Harness, states: &[w::MakingState], listed: bool) {
    let limits = forge::wire_limits(&root.limits.domain.forge).unwrap();
    let mut makings = List::with_capacity(wire::MAKINGS);
    for state in states {
        makings
            .push(
                w::RepositoryMaking::new(
                    &limits,
                    w::RepositoryMakingParts {
                        org: b"team".as_slice().into(),
                        name: b"widget".as_slice().into(),
                        from: w::MakeFrom::Empty,
                        into: Some(w::MakeInto::new(&limits, w::MakeIntoParts { project: 1, home: true }).unwrap()),
                        state: state.clone(),
                    },
                )
                .unwrap(),
            )
            .unwrap();
    }
    let mut forges = List::with_capacity(1);
    if listed {
        forges
            .push(
                w::ForgeRow::new(
                    &limits,
                    w::ForgeRowParts {
                        starts_with: w::StartsWith::Repository,
                        number: 3,
                        host: b"git.example:3000".as_slice().into(),
                        kind: w::Kind::Forgejo,
                        mode: w::Mode::Owned,
                        connection: w::Connection::Connected(
                            w::Connected::new(&limits, w::ConnectedParts { id: 42, login: b"bot".as_slice().into() })
                                .unwrap(),
                        ),
                        offer: w::Offer::Offered,
                        first: None,
                        installations: List::with_capacity(0),
                        ended: None,
                        app: None,
                        web: None,
                        repositories: List::with_capacity(0),
                        owned: None,
                        admin: true,
                        read_only: false,
                        ended_stop: None,
                        makings,
                    },
                )
                .unwrap(),
            )
            .unwrap();
    }
    let record = client::Family {
        family: wire::FORGE,
        version: wire::CLIENT_VERSION,
        bytes: wire::forges(&w::Forges::new(&limits, w::ForgesParts { forges, linked: false }).unwrap()),
    };
    let subject = forge::forges::watch(&root.limits.domain.forge).unwrap();
    let watch = root.watch_token(&subject);
    root.client(client::Event::Up { up: client::Up::FamilyDelivered { watch, record } });
}
#[test]
fn make_sends_the_request_and_prints_kept() {
    for json in [false, true] {
        let mut root = f::signed_in(f::PARTY);
        let call = root.submit_prepared(f::call(MAKE), json, None);
        let make = request(&root, call);
        assert_eq!((make.forge(), make.org(), make.name()), (3, b"team".as_slice(), b"widget".as_slice()));
        assert_eq!(make.from(), &w::MakeFrom::Empty);
        assert_eq!(w::MakeRepository::into(&make).as_ref().unwrap().project(), 1);
        assert!(w::MakeRepository::into(&make).as_ref().unwrap().home());
        assert_eq!(root.sent_name(call), Some(crate::key::engine(&f::name(b"make-widget"))));
        assert!(root.answer(call).is_none());
        root.adoption_answer(call, w::Outcome::MakeKept);
        let printed = root.answer(call).unwrap();
        assert_eq!(printed.status, 0);
        assert_eq!(printed.stdout.as_ref(), if json { br#"{"make_kept":{}}"#.as_slice() } else { b"kept\n" });
        assert_eq!(root.watches.len(), 1, "a bare act opens no forge watch");
    }
}
#[test]
fn copy_lands_at_its_sources_owner_and_name() {
    let mut root = f::signed_in(f::PARTY);
    let call = root.submit(COPY);
    let make = request(&root, call);
    assert_eq!((make.org(), make.name()), (b"team".as_slice(), b"widget".as_slice()));
    let w::MakeFrom::Copy(copy) = make.from() else { panic!("copy source") };
    assert_eq!(copy.address(), b"https://source.invalid/team/widget");
    root.adoption_answer(call, w::Outcome::MakeKept);
    assert_eq!(root.answer(call).unwrap().stdout.as_ref(), b"kept\n");
}
#[test]
fn as_lands_there() {
    let mut root = f::signed_in(f::PARTY);
    let call = root.submit_call(extended(COPY, &["--as", "other/renamed"]), false);
    let make = request(&root, call);
    assert_eq!((make.org(), make.name()), (b"other".as_slice(), b"renamed".as_slice()));
}
#[test]
fn an_address_refused_names_the_form_and_sends_nothing() {
    for address in [
        "http://source.invalid/team/widget",
        "git@source.invalid:team/widget",
        "source.invalid/team/widget/extra",
        "ssh://source.invalid/team/widget",
    ] {
        let mut root = f::signed_in(f::PARTY);
        let call = root.submit(&["repository", "copy", "--project", "shop", address, "--key", "bad-source"]);
        let printed = root.answer(call).unwrap();
        assert_eq!(printed.status, 1);
        assert!(has(&printed, b"HOST/OWNER/NAME"));
        assert!(root.nothing_sent());
    }
}
#[test]
fn no_home_sends_home_false() {
    for words in [MAKE, COPY] {
        let mut root = f::signed_in(f::PARTY);
        let call = root.submit_call(extended(words, &["--no-home"]), false);
        assert!(!w::MakeRepository::into(&request(&root, call)).as_ref().unwrap().home());
    }
}
#[test]
#[expect(clippy::disallowed_methods, reason = "ordinary Rust parses the printed retry to check its offered call")]
fn a_refused_home_names_no_home() {
    for words in [MAKE, COPY] {
        let mut root = f::signed_in(f::PARTY);
        let call = root.submit(words);
        root.adoption_answer(
            call,
            w::Outcome::MakeRefused(
                w::MakeRefused::new(&w::CEILINGS, w::MakeRefusedParts { why: w::MakeRefusal::HomeHeld }).unwrap(),
            ),
        );
        let printed = root.answer(call).unwrap();
        assert_eq!(printed.status, 2);
        assert!(printed.stdout.is_empty());
        assert!(has(&printed, b"--no-home"));
        assert!(has(&printed, b"new key"));
        let retry = core::str::from_utf8(&printed.stderr)
            .unwrap()
            .lines()
            .find_map(|line| line.strip_prefix("  temper ctl DIR "))
            .unwrap();
        let mut argv = List::with_capacity(24);
        for word in retry.split_whitespace() {
            argv.push(if word == "KEY" { "retry-home" } else { word }).unwrap();
        }
        let retry = root.submit(argv.as_slice());
        assert!(!w::MakeRepository::into(&request(&root, retry)).as_ref().unwrap().home());
        assert_eq!(root.sent_name(retry), Some(crate::key::engine(&f::name(b"retry-home"))));
    }
}
#[test]
fn until_adopted_waits_for_the_row_to_go() {
    for json in [false, true] {
        let mut root = f::signed_in(f::PARTY);
        let call = waiting(&mut root, &["--until", "adopted"], json);
        let made =
            w::MakingMade::new(&w::CEILINGS, w::MakingMadeParts { id: 7, branch: b"main".as_slice().into() }).unwrap();
        for state in [
            w::MakingState::Waiting,
            w::MakingState::Making,
            w::MakingState::Made(made.clone()),
            w::MakingState::Adopting(made),
        ] {
            deliver(&mut root, &[state], true);
            assert!(root.answer(call).is_none());
        }
        deliver(&mut root, &[], false);
        assert!(root.answer(call).is_none(), "a missing forge does not prove adoption");
        deliver(&mut root, &[], true);
        let printed = root.answer(call).unwrap();
        assert_eq!(printed.status, 0);
        if json {
            assert!(has(&printed, br#""adopted":true"#));
            f::unique_keys(&printed.stdout).unwrap();
        } else {
            assert_eq!(printed.stdout.as_ref(), b"team/widget is in 1.\n");
        }
        assert_eq!(root.closed.len(), 1);
        assert!(crate::root::next_deadline(&root.root).is_none());
    }
}
#[test]
fn until_adopted_stops_on_a_failed_row() {
    for json in [false, true] {
        let mut root = f::signed_in(f::PARTY);
        let call = waiting(&mut root, &["--until", "adopted"], json);
        deliver(&mut root, &[failed()], true);
        let printed = root.answer(call).unwrap();
        assert_eq!(printed.status, 1);
        assert!(has(&printed, b"name is taken"));
        assert!(has(&printed, b"repository"));
        assert_eq!(root.closed.len(), 1);
        assert!(crate::root::next_deadline(&root.root).is_none());
    }
}
#[test]
fn until_adopted_past_timeout_is_status_2() {
    for json in [false, true] {
        let mut root = f::signed_in(f::PARTY);
        let call = waiting(&mut root, &["--until", "adopted", "--timeout", "1s"], json);
        deliver(&mut root, &[w::MakingState::Making], true);
        root.now = root.now.saturating_add(Duration::from_secs(1));
        root.fire();
        let printed = root.answer(call).unwrap();
        assert_eq!(printed.status, 2);
        assert!(has(&printed, b"timeout"));
        assert!(has(&printed, if json { b"\"making\"" } else { b"Making team/widget" }));
        assert_eq!(root.closed.len(), 1);
        assert!(crate::root::next_deadline(&root.root).is_none());
    }
}
#[test]
fn timeout_without_until_is_refused() {
    for words in [MAKE, COPY] {
        let mut root = f::signed_in(f::PARTY);
        let call = root.submit_call(extended(words, &["--timeout", "1s"]), false);
        let printed = root.answer(call).unwrap();
        assert_eq!(printed.status, 1);
        assert!(has(&printed, b"--timeout needs --until"));
        assert!(root.nothing_sent());
    }
}
#[test]
fn the_same_key_sent_again_answers_kept_and_waits_again() {
    let mut root = f::signed_in(f::PARTY);
    let first = waiting(&mut root, &["--until", "adopted", "--timeout", "1s"], false);
    let name = root.sent_name(first);
    deliver(&mut root, &[w::MakingState::Making], true);
    root.now = root.now.saturating_add(Duration::from_secs(1));
    root.fire();
    assert_eq!(root.answer(first).unwrap().status, 2);
    let again = waiting(&mut root, &["--until", "adopted"], false);
    assert_eq!(root.sent_name(again), name);
    deliver(
        &mut root,
        &[w::MakingState::Adopting(
            w::MakingMade::new(&w::CEILINGS, w::MakingMadeParts { id: 7, branch: b"main".as_slice().into() }).unwrap(),
        )],
        true,
    );
    assert!(root.answer(again).is_none());
    deliver(&mut root, &[], true);
    assert_eq!(root.answer(again).unwrap().status, 0);
}
#[test]
fn forge_list_shows_the_rows() {
    for json in [false, true] {
        let mut root = f::signed_in(f::PARTY);
        let call = root.submit_prepared(f::call(&["forge", "list"]), json, None);
        deliver(&mut root, &[w::MakingState::Making], true);
        let printed = root.answer(call).unwrap();
        assert_eq!(printed.status, 0);
        assert!(has(
            &printed,
            if json { br#""makings":[{"org":"team","name":"widget""# } else { b"\n  Making team/widget\n" }
        ));
        let mut row = super::forges::row(Connection::NotConnected);
        row.makings = Box::new([making(w::MakingState::Making)]);
        assert!(!crate::forges::holds(&[row.clone()], b"settled"));
        row.makings = Box::new([making(w::MakingState::Failed(
            w::MakingFailed::new(
                &w::CEILINGS,
                w::MakingFailedParts {
                    step: w::MakeStep::Repository,
                    reason: w::MakeFailure::Refused(
                        w::Refused::new(&w::CEILINGS, w::RefusedParts { status: 409 }).unwrap(),
                    ),
                    words: b"name is taken".as_slice().into(),
                },
            )
            .unwrap(),
        ))]);
        assert!(crate::forges::holds(&[row.clone()], b"settled"));
        let printed = crate::forges::print_rows(Some(&[row]), json, None, &root.limits);
        assert!(has(&printed, b"name is taken"));
        if json {
            f::unique_keys(&printed.stdout).unwrap();
        }
    }
}
#[test]
fn an_unknown_project_sends_nothing() {
    for project in ["missing", "999"] {
        let mut root = f::signed_in(f::PARTY);
        let call =
            root.submit(&["repository", "make", "--project", project, "team/widget", "--key", "unknown-project"]);
        let printed = root.answer(call).unwrap();
        assert_eq!(printed.status, 1);
        assert!(has(&printed, b"unknown project"));
        assert!(root.nothing_sent());
    }
}
#[test]
fn restored_requests_print_their_kept_answers() {
    for words in [MAKE, COPY] {
        for json in [false, true] {
            let mut previous = f::signed_in(f::PARTY);
            let call = previous.submit(words);
            let mut records = f::restored_adoption(f::PARTY);
            let record = records.get_mut(0).unwrap();
            record.key = previous.sent_name(call).unwrap();
            record.request.request = previous.adoption_bytes(call).clone();
            let mut root = f::started_with(records, f::SEED, json);
            root.deliver(c::Delivery::Me(Box::new(f::me(f::PARTY))));
            root.adoption_answer(0, w::Outcome::MakeKept);
            let line = root.kept_lines.as_slice().last().unwrap();
            let operation = if words == MAKE { b"repository-make".as_slice() } else { b"repository-copy" };
            assert!(skein_lib::bytes::find(line, operation).is_some());
            assert!(skein_lib::bytes::find(line, b"kept").is_some());
            if json {
                f::unique_keys(line).unwrap();
            }
        }
    }
}

fn has(printed: &jig::Printed, expected: &[u8]) -> bool {
    skein_lib::bytes::find(&printed.stdout, expected).is_some()
        || skein_lib::bytes::find(&printed.stderr, expected).is_some()
}
