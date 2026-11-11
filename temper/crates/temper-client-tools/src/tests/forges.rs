use super::fixtures as f;
use crate::{
    forges,
    jig::{self, call::Parsed},
};
use jig_client::{self as client, contract as c};
use skein_lib::{Duration, List};
use temper_client_domain::forge::{
    self,
    forges::{Connection as C, Kind, Mode, Offer, Row, Step},
};
use temper_client_protocol_wire::{self as wire, v1 as w};
pub(super) fn row(connection: C) -> Row {
    Row {
        starts_with: forge::forges::StartsWith::Repository,
        makings: Box::new([]),
        host: b"git.example:3000".as_slice().into(),
        number: 3,
        kind: Kind::Forgejo,
        mode: Mode::Owned,
        connection,
        fetched: None,
        offer: Offer::Offered,
        first: None,
        installations: Box::new([]),
        ended: None,
        app: None,
        web: None,
        repositories: Box::new([]),
        owned: None,
        admin: false,
        read_only: false,
        ended_stop: None,
    }
}
fn record(connection: w::Connection) -> client::Family {
    record_first(connection, None)
}
fn record_first(connection: w::Connection, first: Option<w::FirstRepository>) -> client::Family {
    let limits = forge::wire_limits(&f::limits_root().domain.forge).unwrap();
    let mut forges = List::with_capacity(1);
    forges
        .push(
            w::ForgeRow::new(
                &limits,
                w::ForgeRowParts {
                    starts_with: w::StartsWith::Repository,
                    makings: List::with_capacity(0),
                    host: b"git.example:3000".as_slice().into(),
                    number: 3,
                    kind: w::Kind::Forgejo,
                    mode: w::Mode::Owned,
                    connection,
                    offer: w::Offer::Offered,
                    first,
                    installations: List::with_capacity(0),
                    ended: None,
                    app: None,
                    web: Some(b"https://web.example/shop".as_slice().into()),
                    repositories: List::with_capacity(0),
                    owned: None,
                    admin: false,
                    read_only: false,
                    ended_stop: None,
                },
            )
            .unwrap(),
        )
        .unwrap();
    client::Family {
        family: wire::FORGE,
        version: wire::CLIENT_VERSION,
        bytes: wire::forges(&w::Forges::new(&limits, w::ForgesParts { forges, linked: false }).unwrap()),
    }
}
fn deliver(root: &mut f::Harness, record: client::Family) {
    let subject = forge::forges::watch(&root.limits.domain.forge).unwrap();
    let token = root.watch_token(&subject);
    root.client(client::Event::Up { up: client::Up::FamilyDelivered { watch: token, record } });
}
#[test]
fn forge_list_prints_the_document_host_in_text_and_json() {
    for json in [false, true] {
        let mut root = f::signed_in(f::PARTY);
        let call = root.submit_prepared(f::call(&["forge", "list"]), json, None);
        deliver(&mut root, record(w::Connection::NotConnected));
        let printed = root.answer(call).expect("forge list answer");
        assert_eq!(printed.status, 0);
        let expected =
            if json { br#""host":"git.example:3000""#.as_slice() } else { b"host: git.example:3000".as_slice() };
        assert!(skein_lib::bytes::find(&printed.stdout, expected).is_some(), "document forge host missing");
    }
}

#[test]
fn forge_calls_round_trip_through_argv_json_and_help() {
    for argv in [
        &["forge", "list"][..],
        &["forge", "setup"],
        &["forge", "setup", "3"],
        &["forge", "setup", "3", "--key", "setup"],
        &["forge", "setup", "3", "--org", "team", "--repository", "widget", "--project", "shop"],
        &["forge", "list", "--until", "made:3"],
        &["forge", "setup", "3", "--project", ""],
        &["forge", "list", "--until", "settled"],
        &["forge", "list", "--since", "0123456789abcdef", "--until", "connected:3", "--timeout", "2s"],
    ] {
        let call = f::call(argv);
        assert_eq!(f::printed(&call), Parsed::Call { call: call.clone(), json: false });
        let json = jig::json::write_call(crate::table::TOOLS, &call).unwrap();
        assert_eq!(f::from_json(&json), call);
        let help = f::help(Some(call.tool));
        assert!(skein_lib::bytes::find(&help, argv[0].as_bytes()).is_some());
    }
    for argv in [
        &["forge", "setup", "oops"][..],
        &["forge", "setup", "65536"],
        &["forge", "setup", "0"],
        &["forge", "list", "--until", "busy"],
        &["forge", "list", "--until", "made:x"],
        &["forge", "list", "--until", "made:0"],
        &["forge", "list", "--until", "made:65536"],
        &["forge", "list", "--until", "made:"],
        &["forge", "list", "--until", "connected:x"],
        &["forge", "list", "--until", "connected:65536"],
        &["forge", "list", "--until", "connected:0"],
        &["forge", "list", "--until", "connected:"],
        &["forge", "list", "--until", "connected:18446744073709551616"],
    ] {
        assert!(f::parse(argv).is_err(), "{argv:?}");
    }
}
#[test]
fn every_state_renders_as_text_and_structured_json() {
    let cases: &[(C, &[u8], &[u8])] = &[
        (C::NotConnected, b"Not connected", br#""state":"not_connected""#),
        (C::NotSetUp, b"Not set up", br#""state":"not_set_up""#),
        (C::SettingUp { step: None }, b"Setting up", br#""state":"setting_up""#),
        (C::SettingUp { step: Some(Step::Fetch) }, b"Setting up: fetching", br#""state":"setting_up","step":"fetch""#),
        (
            C::SettingUp { step: Some(Step::Prepare) },
            b"Setting up: preparing",
            br#""state":"setting_up","step":"prepare""#,
        ),
        (C::SettingUp { step: Some(Step::Run) }, b"Setting up: starting", br#""state":"setting_up","step":"run""#),
        (
            C::SetUpFailed { why: b"fetch failed".as_slice().into(), at: None },
            b"Set up failed: fetch failed",
            br#""state":"set_up_failed","why":"fetch failed""#,
        ),
        (C::Connecting, b"Connecting", br#""state":"connecting""#),
        (
            C::Connected { id: 42, login: b"bot".as_slice().into() },
            b"Connected as bot (42)",
            br#""state":"connected","id":42,"login":"bot""#,
        ),
        (
            C::Stopped { reason: b"retired".as_slice().into() },
            b"Stopped: retired",
            br#""state":"stopped","reason":"retired""#,
        ),
    ];
    for (state, words, fields) in cases {
        let rows = [row(state.clone())];
        let plain = forges::print_rows(Some(&rows), false, None, &f::limits_root());
        let json = forges::print_rows(Some(&rows), true, None, &f::limits_root());
        assert_eq!(plain.status, 0);
        assert_eq!(json.status, 0);
        let document = concat(&[
            br#"{"forges":[{"number":3,"host":"git.example:3000","kind":"forgejo","read_only":false,"mode":"owned","offer":"offered","#,
            fields,
            br#","installations":[],"ended":null,"starts_with":"repository","first":null,"makings":[]}]}"#,
        ]);
        let digest = jig::digest::Digest::of(&document).hex();
        let expected = concat(&[
            b"forge 3 \xc2\xb7 host: git.example:3000 \xc2\xb7 Forgejo \xc2\xb7 owned \xc2\xb7 ",
            words,
            b"\ndigest: forges ",
            &digest,
            b"\n",
        ]);
        assert_eq!(&*plain.stdout, expected.as_ref(), "{state:?}");
        let expected = concat(&[
            document.get(..document.len().checked_sub(1).unwrap()).unwrap(),
            b",\"digest\":\"",
            &digest,
            b"\"}",
        ]);
        assert_eq!(&*json.stdout, expected.as_ref(), "{state:?}");
    }
    let mut without = row(C::NotSetUp);
    without.offer = Offer::NotUnderTheBox;
    let plain = forges::print_rows(Some(&[without.clone()]), false, None, &f::limits_root());
    assert!(skein_lib::bytes::find(&plain.stdout, b"Not set up: start temper with temper box").is_some());
    let json = forges::print_rows(Some(&[without]), true, None, &f::limits_root());
    assert!(skein_lib::bytes::find(&json.stdout, b"not_under_the_box").is_some());
}
#[test]
fn settled_excludes_setting_up_and_connecting_and_connected_names_a_forge() {
    for state in [
        C::NotConnected,
        C::NotSetUp,
        C::SetUpFailed { why: Box::default(), at: None },
        C::Stopped { reason: Box::default() },
        C::Connected { id: 1, login: Box::default() },
    ] {
        assert!(forges::holds(&[row(state)], b"settled"));
    }
    for state in [C::SettingUp { step: None }, C::SettingUp { step: Some(Step::Run) }, C::Connecting] {
        assert!(!forges::holds(&[row(C::NotConnected), row(state)], b"settled"));
    }
    let rows = [row(C::Connected { id: 1, login: Box::default() })];
    assert!(forges::holds(&rows, b"connected:3"));
    assert!(!forges::holds(&rows, b"connected:2"));
    assert!(!forges::holds(&[row(C::NotConnected)], b"connected:3"));
    assert!(forges::holds(&[], b"settled"));
}
#[test]
fn the_read_closes_after_its_first_document_and_refuses_invalid_bytes() {
    for (bytes, status) in [
        (record(w::Connection::NotSetUp), 0),
        (client::Family { bytes: b"bad".as_slice().into(), ..record(w::Connection::NotSetUp) }, 1),
    ] {
        let mut root = f::signed_in(f::PARTY);
        let call = root.submit(&["forge", "list"]);
        deliver(&mut root, bytes);
        assert_eq!(root.answer(call).unwrap().status, status);
        assert_eq!(root.closed.len(), 1);
        assert_eq!(root.watches.len(), 1, "only the runner's me remains");
        assert!(crate::root::next_deadline(&root.root).is_none());
    }
}
#[test]
fn a_wait_follows_each_setup_step_and_releases_the_watch_and_timer() {
    let mut root = f::signed_in(f::PARTY);
    let call = root.submit(&["forge", "list", "--until", "settled"]);
    let limits = forge::wire_limits(&root.limits.domain.forge).unwrap();
    for step in [None, Some(w::SetUpStep::Prepare), Some(w::SetUpStep::Run)] {
        deliver(
            &mut root,
            record(w::Connection::SettingUp(
                w::SettingUp::new(&limits, w::SettingUpParts { step, fetched: None }).unwrap(),
            )),
        );
        assert!(root.answer(call).is_none());
    }
    deliver(&mut root, record(w::Connection::Connecting));
    assert!(root.answer(call).is_none());
    deliver(&mut root, record(w::Connection::NotConnected));
    let printed = root.answer(call).unwrap();
    assert_eq!(printed.status, 0);
    assert!(printed.stdout.ends_with(b"met\n"));
    assert_eq!(root.closed.len(), 1);
    assert!(crate::root::next_deadline(&root.root).is_none());
}
#[test]
fn since_uses_the_read_digest_and_plain_wait_requires_a_change() {
    for until in [false, true] {
        let mut root = f::signed_in(f::PARTY);
        let read = root.submit(&["forge", "list", "--json"]);
        deliver(&mut root, record(w::Connection::NotSetUp));
        let printed = root.answer(read).unwrap();
        let digest = f::last_digest(&printed.stdout);
        let flags = if until { &["--until", "settled"][..] } else { &[] };
        let call = root.wait_since(&["forge", "list"], flags, &digest);
        deliver(&mut root, record(w::Connection::NotSetUp));
        assert!(root.answer(call).is_none());
        deliver(&mut root, record(w::Connection::NotConnected));
        assert_eq!(root.answer(call).unwrap().status, 0);
    }
    let mut root = f::signed_in(f::PARTY);
    let call = root.submit(&["forge", "list", "--until", "changed"]);
    deliver(&mut root, record(w::Connection::NotSetUp));
    assert!(root.answer(call).is_none());
    deliver(&mut root, record(w::Connection::NotSetUp));
    assert!(root.answer(call).is_none());
    deliver(&mut root, record(w::Connection::NotConnected));
    assert_eq!(root.answer(call).unwrap().status, 0);
}
#[test]
fn timeout_prints_the_current_document_and_watch_refusals_keep_their_status() {
    let mut root = f::signed_in(f::PARTY);
    let call = root.submit(&["forge", "list", "--until", "connected:3", "--timeout", "1s", "--json"]);
    deliver(&mut root, record(w::Connection::NotSetUp));
    root.now = root.now.checked_add(Duration::from_secs(1)).unwrap();
    root.fire();
    let printed = root.answer(call).unwrap();
    assert_eq!(printed.status, 3);
    assert!(f::json_has(&printed, br#""state":"not_set_up""#));
    assert!(f::json_has(&printed, br#""met":false,"why":"timeout""#));
    assert_eq!(root.closed.len(), 1);
    for (end, status) in [(c::WatchEnd::NotAllowed, 2), (c::WatchEnd::Unknown, 2)] {
        let mut root = f::signed_in(f::PARTY);
        let call = root.submit(&["forge", "list", "--until", "settled"]);
        let token = root.watch_token(&forge::forges::watch(&root.limits.domain.forge).unwrap());
        root.client(client::Event::Up { up: client::Up::WatchEnded { watch: token, end } });
        assert_eq!(root.answer(call).unwrap().status, status);
        assert!(crate::root::next_deadline(&root.root).is_none());
    }
}
#[test]
fn setup_answers_print_their_words_json_and_status() {
    let limits = forge::wire_limits(&f::limits_root().domain.forge).unwrap();
    for (outcome, expected, status) in [
        (w::Outcome::SettingUp, b"setting up\n".as_slice(), 0),
        (w::Outcome::Unknown, b"unknown\n", 2),
        (
            w::Outcome::SetUpRefused(
                w::SetUpRefused::new(&limits, w::SetUpRefusedParts { why: w::SetUpRefusal::Busy }).unwrap(),
            ),
            b"refused: busy\n",
            2,
        ),
        (
            w::Outcome::SetUpRefused(
                w::SetUpRefused::new(&limits, w::SetUpRefusedParts { why: w::SetUpRefusal::NotOffered }).unwrap(),
            ),
            b"refused: not offered\nOne owned forge is offered; read forge list, then project make NAME --key KEY; repository make --project PROJECT --forge FORGE ORG/NAME --until adopted --key KEY\n",
            2,
        ),
        (
            w::Outcome::SetUpRefused(
                w::SetUpRefused::new(&limits, w::SetUpRefusedParts { why: w::SetUpRefusal::NotUnderTheBox }).unwrap(),
            ),
            b"refused: not under the box\nStart temper with `temper box` to set one up here.\n",
            2,
        ),
    ] {
        for json in [false, true] {
            let mut root = f::signed_in(f::PARTY);
            let call = root.submit_prepared(f::call(&["forge", "setup", "3"]), json, None);
            let client::Ask::Family { scope, record } = root.adoption_bytes(call) else { panic!("family") };
            assert_eq!(*scope, c::Scope::Deployment);
            let w::Request::SetUp(request) =
                wire::decode_requesting(&record.bytes, &limits).unwrap().into_parts().request
            else {
                panic!("set up");
            };
            assert_eq!(
                request.forge(),
                &w::Making::Kept(w::Kept::new(&w::CEILINGS, w::KeptParts { forge: 3 }).unwrap())
            );
            root.adoption_answer(call, outcome.clone());
            if status == 0 {
                assert!(root.answer(call).is_none());
                assert!(root.progress.as_slice().last().is_some());
                continue;
            }
            let printed = root.answer(call).unwrap();
            assert_eq!(printed.status, status);
            let bytes = if status == 0 { &printed.stdout } else { &printed.stderr };
            if json {
                assert!(bytes.starts_with(b"{"));
                if let w::Outcome::SetUpRefused(ref refusal) = outcome
                    && *refusal.why() == w::SetUpRefusal::NotUnderTheBox
                {
                    assert_eq!(bytes.as_ref(), b"{\"refused\":{\"why\":\"not_under_the_box\"},\"next\":\"Start temper with `temper box` to set one up here.\"}");
                }
            } else {
                assert_eq!(bytes.as_ref(), expected);
            }
        }
    }
}

#[test]
fn setup_keeps_drawn_keys_and_recovers_the_answer_in_its_own_words() {
    for json in [false, true] {
        let mut root = f::signed_in(f::PARTY);
        let call = root.submit(&["forge", "setup", "3"]);
        assert!(root.saved.contains_key(&call), "a drawn key is durable too");
        let key = root.sent_name(call).unwrap();
        let mut records = f::restored_adoption(f::PARTY);
        let record = records.get_mut(0).unwrap();
        record.key = key;
        record.request.request = root.adoption_bytes(call).clone();
        let mut root = f::started_with(records, f::SEED, json);
        root.deliver(c::Delivery::Me(Box::new(f::me(f::PARTY))));
        root.adoption_answer(0, w::Outcome::SettingUp);
        let expected = if json {
            b"{\"operation\":\"forge-setup\",\"outcome\":{\"setting_up\":{}}}\n".as_slice()
        } else {
            b"kept: forge-setup setting up\n"
        };
        assert_eq!(root.kept_lines.as_slice(), &[Box::<[u8]>::from(expected)]);
    }
}
#[test]
fn setup_names_are_stable_and_unconfirmed_calls_wait_for_me() {
    let mut root = f::started(List::with_capacity(0));
    let call = root.submit(&["forge", "setup", "3", "--key", "start-here"]);
    assert!(root.nothing_sent());
    root.deliver(c::Delivery::Me(Box::new(f::me(f::PARTY))));
    assert_eq!(root.sent_name(call), Some(crate::key::engine(&f::name(b"start-here"))));
    let mut other = f::signed_in(f::PARTY);
    let next = other.submit(&["forge", "setup", "3", "--key", "start-here"]);
    assert_eq!(root.sent_name(call), other.sent_name(next));
    assert_eq!(root.adoption_bytes(call), other.adoption_bytes(next));
}
#[test]
fn a_set_up_rejects_an_adoption_answer_and_preserves_entrance_refusals() {
    let outcome = client::Outcome::Answered(f::family_answer(f::adopted_with_unknown_protection()));
    assert_eq!(forges::setup_print(3, &outcome, false, false, &f::limits_root()).status, 1);
    let outcome = client::Outcome::Answered(client::Answered::Jig(c::Answer::Refused {
        refusal: c::Refusal::Role { held: None },
        at: None,
    }));
    for json in [false, true] {
        assert_eq!(forges::setup_print(3, &outcome, json, false, &f::limits_root()).status, 2);
    }
}

#[test]
fn all_children_share_the_call_budget_and_timeouts_release_it() {
    let mut root = f::started(List::with_capacity(0));
    // Calls wait for me, so no stream-table bound masks the root's call bound.
    let mut calls = List::with_capacity(root.limits.tools.rows);
    for at in 0..root.limits.tools.rows {
        let argv = if at % 2 == 0 {
            &["forge", "list", "--timeout", "1s", "--until", "changed"][..]
        } else {
            &["account", "list", "--timeout", "1s", "--until", "changed"]
        };
        calls.push(root.submit(argv)).unwrap();
    }
    let excess = root.submit(&["forge", "setup", "3", "--json"]);
    let printed = root.answer(excess).unwrap();
    assert_eq!(printed.status, 1);
    assert_eq!(printed.stderr.as_ref(), br#"{"error":"call limit"}"#);
    root.deliver(c::Delivery::Me(Box::new(f::me(f::PARTY))));
    root.now = root.now.checked_add(Duration::from_secs(1)).unwrap();
    root.fire();
    for call in &calls {
        assert!(root.answer(*call).is_some());
    }
    let next = root.submit(&["forge", "setup", "3"]);
    assert!(root.sends.contains_key(&next));
}

fn concat(parts: &[&[u8]]) -> Box<[u8]> {
    let mut size = 0usize;
    for part in parts {
        size = size.checked_add(part.len()).unwrap();
    }
    let mut out = skein_lib::Writer::new(size);
    for part in parts {
        out.put(part).unwrap();
    }
    out.finish()
}

fn first_record(state: w::FirstState) -> w::FirstRepository {
    w::FirstRepository::new(
        &w::CEILINGS,
        w::FirstRepositoryParts {
            playground: false,
            org: b"work".as_slice().into(),
            repository: b"hello".as_slice().into(),
            state,
            label: b"docker".as_slice().into(),
        },
    )
    .unwrap()
}
fn made() -> w::FirstState {
    w::FirstState::Made(w::Made::new(&w::CEILINGS, w::MadeParts { id: 42 }).unwrap())
}
#[test]
fn setup_carries_repository_fields_with_each_project_argument() {
    // Group, verb, forge, flag and value.
    const WORDS_WITH_ONE_FLAG: u32 = 5;
    for (flags, org, repository) in [
        (&[][..], b"work".as_slice(), b"hello".as_slice()),
        (&["--org", "team"][..], b"team", b"hello"),
        (&["--repository", "widget"][..], b"work", b"widget"),
        (&["--project", "7"][..], b"work", b"hello"),
        (&["--project", "shop"][..], b"work", b"hello"),
    ] {
        let mut root = f::signed_in(f::PARTY);
        let mut words = List::with_capacity(WORDS_WITH_ONE_FLAG);
        words.push("forge").unwrap();
        words.push("setup").unwrap();
        words.push("3").unwrap();
        for word in flags {
            words.push(*word).unwrap();
        }
        let call = root.submit(words.as_slice());
        let client::Ask::Family { record, .. } = root.adoption_bytes(call) else { panic!("family") };
        let w::Request::SetUp(value) =
            wire::decode_requesting(&record.bytes, &w::CEILINGS).unwrap().into_parts().request
        else {
            panic!("setup")
        };
        assert_eq!((empty_setup(value.first()).org(), empty_setup(value.first()).repository()), (org, repository));
    }
}

#[test]
fn setup_refuses_an_empty_local_project_before_sending() {
    let mut root = f::signed_in(f::PARTY);
    let call = root.submit(&["forge", "setup", "3", "--project", ""]);
    let printed = root.answer(call).expect("local refusal");
    assert_eq!(printed.status, 2);
    assert_eq!(
        printed.stderr.as_ref(),
        b"refused: invalid: the organization, repository or project name is empty or too long\n"
    );
    assert!(root.nothing_sent());
}
#[test]
fn invalid_setup_refusals_exit_two_with_a_local_project() {
    let mut root = f::signed_in(f::PARTY);
    let call = root.submit(&["forge", "setup", "3", "--project", "Next"]);
    root.adoption_answer(
        call,
        w::Outcome::SetUpRefused(
            w::SetUpRefused::new(&w::CEILINGS, w::SetUpRefusedParts { why: w::SetUpRefusal::Invalid }).unwrap(),
        ),
    );
    let printed = root.answer(call).unwrap();
    assert_eq!(printed.status, 2);
    assert!(
        skein_lib::bytes::find(
            &printed.stderr,
            b"invalid: the organization, repository or project name is empty or too long"
        )
        .is_some()
    );
}
#[test]
fn made_sends_the_home_adoption_with_its_stable_key_and_prints_its_answer() {
    for home_taken in [false, true] {
        let mut root = f::signed_in(f::PARTY);
        let call =
            root.submit(&["forge", "setup", "3", "--org", "team", "--repository", "widget", "--project", "shop"]);
        root.adoption_answer(call, w::Outcome::SettingUp);
        assert!(root.answer(call).is_none());
        assert_eq!(root.progress.as_slice(), &[Box::<[u8]>::from(b"setting up\n".as_slice())]);
        deliver(&mut root, record_first(w::Connection::Connecting, Some(first_record(w::FirstState::Making))));
        assert!(root.answer(call).is_none());
        deliver(&mut root, record_first(w::Connection::Connecting, Some(first_record(made()))));
        assert!(root.answer(call).is_none(), "creation precedes the forge's connection");
        deliver(
            &mut root,
            record_first(
                w::Connection::Connected(
                    w::Connected::new(&w::CEILINGS, w::ConnectedParts { id: 1, login: b"temper".as_slice().into() })
                        .unwrap(),
                ),
                Some(first_record(made())),
            ),
        );
        let adopted = root.sent_adoption(call);
        assert_eq!((adopted.project, adopted.forge, adopted.home, adopted.ci), (f::PROJECT, 3, true, true));
        assert_eq!(
            (adopted.owner.as_ref(), adopted.name.as_ref(), adopted.landing.as_ref()),
            (b"team".as_slice(), b"widget".as_slice(), b"main".as_slice())
        );
        assert_eq!(adopted.role, w::Role::Owned);
        assert_eq!(adopted.prefix.as_ref(), b"temper/");
        assert!(adopted.checks.is_empty());
        assert_eq!(root.sent_name(call), Some(crate::key::engine(&f::name(b"first-repository-3"))));
        assert_eq!(root.sent_name(call), Some(forge::first_repository_key(3, false)));
        assert_eq!(root.closed.len(), 1);
        root.adoption_answer(
            call,
            if home_taken { f::forge_refused(w::ForgeReason::Refused) } else { f::adopted_with_unknown_protection() },
        );
        if home_taken {
            assert!(root.answer(call).is_none());
            assert!(!root.sent_adoption(call).home);
            assert_eq!(root.sent_name(call), Some(crate::key::engine(&f::name(b"first-repository-additional-3"))));
            assert_eq!(root.sent_name(call), Some(forge::first_repository_key(3, true)));
            root.adoption_answer(call, f::adopted_with_unknown_protection());
        }
        let printed = root.answer(call).unwrap();
        assert_eq!(printed.status, 0);
        assert!(printed.stdout.starts_with(b"project: 1 \xc2\xb7 shop\nAdopted "));
        assert!(root.answer(call).is_none(), "one terminal answer");
    }
}
#[test]
fn setup_failure_names_the_creation_or_box_step_and_exits_one() {
    let mut cases = List::with_capacity(5);
    cases
        .push((
            w::Connection::SetUpFailed(
                w::SetUpFailed::new(
                    &w::CEILINGS,
                    w::SetUpFailedParts { why: b"preparing".as_slice().into(), at: None },
                )
                .unwrap(),
            ),
            None,
            b"Set up failed: preparing".as_slice(),
        ))
        .unwrap();
    for (step, words) in [
        (w::CreateStep::Admin, b"checking the admin's token".as_slice()),
        (w::CreateStep::Org, b"making the organization"),
        (w::CreateStep::Repository, b"making the repository"),
        (w::CreateStep::File, b"adding the starter workflow"),
    ] {
        cases
            .push((
                w::Connection::Connecting,
                Some(first_record(w::FirstState::Failed(
                    w::CreateFailed::new(&w::CEILINGS, w::CreateFailedParts { step }).unwrap(),
                ))),
                words,
            ))
            .unwrap();
    }
    for (connection, first, words) in cases.into_boxed() {
        let mut root = f::signed_in(f::PARTY);
        let call = root.submit(&["forge", "setup", "3"]);
        root.adoption_answer(call, w::Outcome::SettingUp);
        deliver(&mut root, record_first(connection, first));
        let printed = root.answer(call).unwrap();
        assert_eq!(printed.status, 1);
        assert!(skein_lib::bytes::find(&printed.stderr, words).is_some());
        assert_eq!(root.closed.len(), 1);
        assert!(crate::root::next_deadline(&root.root).is_none());
    }
}
#[test]
fn made_wait_requires_the_named_forges_first_repository_to_be_made() {
    let mut root = f::signed_in(f::PARTY);
    let call = root.submit(&["forge", "list", "--until", "made:3"]);
    for first in [None, Some(first_record(w::FirstState::Making))] {
        deliver(
            &mut root,
            record_first(
                w::Connection::Connected(
                    w::Connected::new(&w::CEILINGS, w::ConnectedParts { id: 1, login: b"temper".as_slice().into() })
                        .unwrap(),
                ),
                first,
            ),
        );
        assert!(root.answer(call).is_none(), "connected alone is insufficient");
    }
    deliver(&mut root, record_first(w::Connection::Connecting, Some(first_record(made()))));
    assert_eq!(root.answer(call).unwrap().status, 0);
    assert_eq!(root.closed.len(), 1);
    let mut row = row(C::Connecting);
    row.first = Some(forge::forges::First {
        playground: false,
        org: b"work".as_slice().into(),
        repository: b"hello".as_slice().into(),
        state: forge::forges::FirstState::Made { id: 42 },
        label: b"docker".as_slice().into(),
    });
    assert!(!forges::holds(&[row], b"made:4"));
}
#[test]
fn first_repository_text_and_json_name_every_state_and_creation_step() {
    use forge::forges::{CreateStep, First, FirstState};
    for (state, words, field) in [
        (FirstState::Making, b"Making work/hello".as_slice(), br#""state":"making""#.as_slice()),
        (FirstState::Made { id: 42 }, b"work/hello made", br#""state":"made","id":42"#),
        (
            FirstState::Failed { step: CreateStep::Admin },
            b"Making work/hello failed: checking the admin's token",
            br#""state":"failed","step":"admin"#,
        ),
        (
            FirstState::Failed { step: CreateStep::Org },
            b"Making work/hello failed: making the organization",
            br#""state":"failed","step":"org"#,
        ),
        (
            FirstState::Failed { step: CreateStep::Repository },
            b"Making work/hello failed: making the repository",
            br#""state":"failed","step":"repository"#,
        ),
        (
            FirstState::Failed { step: CreateStep::File },
            b"Making work/hello failed: adding the starter workflow",
            br#""state":"failed","step":"file"#,
        ),
    ] {
        let mut row = row(C::Connecting);
        row.first = Some(First {
            playground: false,
            org: b"work".as_slice().into(),
            repository: b"hello".as_slice().into(),
            state,
            label: b"docker".as_slice().into(),
        });
        let plain = forges::print_rows(Some(&[row.clone()]), false, None, &f::limits_root());
        assert!(skein_lib::bytes::find(&plain.stdout, words).is_some());
        let json = forges::print_rows(Some(&[row]), true, None, &f::limits_root());
        assert!(f::json_has(&json, br#""first":{"org":"work","repository":"hello""#));
        assert!(f::json_has(&json, field));
        assert!(skein_lib::bytes::find(&json.stdout, br#""project""#).is_none());
    }
}

#[test]
fn demo_start_answers_print_their_words_json_and_status() {
    let l = &w::CEILINGS;
    for (outcome, expected, json_expected, status) in [
        (w::Outcome::Running, "running\n", r#"{"running":{}}"#, 0),
        (
            w::Outcome::StartRefused(
                w::StartRefused::new(l, w::StartRefusedParts { why: w::StartRefusal::NotOffered }).unwrap(),
            ),
            "refused: not offered\n",
            r#"{"start_refused":{"why":"not_offered"}}"#,
            2,
        ),
        (
            w::Outcome::StartFailed(
                w::StartFailed::new(
                    l,
                    w::StartFailedParts {
                        why: w::StartFailure::Group(
                            w::GroupFailed::new(l, w::GroupFailedParts { why: b"fetch failed".as_slice().into() })
                                .unwrap(),
                        ),
                    },
                )
                .unwrap(),
            ),
            "the fakes did not start (fetch failed)\n",
            r#"{"start_failed":{"why":{"group":{"why":"fetch failed"}}}}"#,
            1,
        ),
        (
            w::Outcome::StartFailed(
                w::StartFailed::new(l, w::StartFailedParts { why: w::StartFailure::Account }).unwrap(),
            ),
            "the account refused\n",
            r#"{"start_failed":{"why":"account"}}"#,
            1,
        ),
        (
            w::Outcome::StartFailed(
                w::StartFailed::new(l, w::StartFailedParts { why: w::StartFailure::Adoption }).unwrap(),
            ),
            "the adoption refused\n",
            r#"{"start_failed":{"why":"adoption"}}"#,
            1,
        ),
        (
            w::Outcome::StartFailed(
                w::StartFailed::new(l, w::StartFailedParts { why: w::StartFailure::NotConnected }).unwrap(),
            ),
            "the forge not connected in time\n",
            r#"{"start_failed":{"why":"not_connected"}}"#,
            1,
        ),
    ] {
        for json in [false, true] {
            let mut root = f::signed_in(f::PARTY);
            let call = root.submit_prepared(f::call(&["playground", "start", "--key", "playground"]), json, None);
            let client::Ask::Family { scope, record } = root.adoption_bytes(call) else { panic!("family") };
            assert_eq!(*scope, c::Scope::Deployment);
            assert_eq!(wire::decode_requesting(&record.bytes, l).unwrap().request(), &w::Request::StartPlayground);
            root.adoption_answer(call, outcome.clone());
            let printed = root.answer(call).unwrap();
            assert_eq!(printed.status, status);
            let bytes = if status == 0 { &printed.stdout } else { &printed.stderr };
            assert_eq!(bytes.as_ref(), if json { json_expected.as_bytes() } else { expected.as_bytes() });
        }
    }
}

#[test]
fn demo_start_keeps_drawn_keys_and_recovers_its_own_answer() {
    for json in [false, true] {
        let mut root = f::signed_in(f::PARTY);
        let call = root.submit(&["playground", "start"]);
        assert!(root.saved.contains_key(&call), "a drawn demo start key is durable");
        let key = root.sent_name(call).unwrap();
        let mut records = f::restored_adoption(f::PARTY);
        let record = records.get_mut(0).unwrap();
        record.key = key;
        record.request.request = root.adoption_bytes(call).clone();
        let mut root = f::started_with(records, f::SEED, json);
        root.deliver(c::Delivery::Me(Box::new(f::me(f::PARTY))));
        root.adoption_answer(0, w::Outcome::Running);
        let expected = if json {
            b"{\"operation\":\"playground-start\",\"outcome\":{\"running\":{}}}\n".as_slice()
        } else {
            b"kept: playground-start running\n"
        };
        assert_eq!(root.kept_lines.as_slice(), &[Box::<[u8]>::from(expected)]);
    }
}

#[test]
fn forges_json_names_installations_and_the_last_connect_end() {
    use forge::forges::{Account, Installation};
    let mut row = row(C::NotConnected);
    row.kind = Kind::GitHub;
    row.installations = Box::new([
        Installation {
            id: 11,
            account: Some(Account { id: 111, login: Box::from(b"acme".as_slice()) }),
            suspended: true,
            stray: false,
        },
        Installation { id: 22, account: None, suspended: false, stray: true },
    ]);
    row.ended = Some(Box::from(b"the pasted value changed".as_slice()));
    let json = forges::print_rows(Some(&[row]), true, None, &f::limits_root());
    assert!(f::json_has(&json, br#""installations":[{"id":11,"account":{"id":111,"login":"acme"},"suspended":true,"stray":false},{"id":22,"account":null,"suspended":false,"stray":true}]"#));
    assert!(f::json_has(&json, br#""ended":"the pasted value changed""#));
}

fn settled_wait_with_installations(count: u32) {
    let mut root = f::signed_in(f::PARTY);
    let call = root.submit(&["forge", "list", "--until", "settled", "--json"]);
    let limits = forge::wire_limits(&root.limits.domain.forge).unwrap();
    let mut installations = List::with_capacity(count);
    for id in 1..=count {
        installations
            .push(
                w::Installation::new(
                    &limits,
                    w::InstallationParts {
                        id: u64::from(id),
                        account: Some(
                            w::Account::new(&limits, w::AccountParts { id: 111, login: b"acme".as_slice().into() })
                                .unwrap(),
                        ),
                        suspended: false,
                        stray: false,
                    },
                )
                .unwrap(),
            )
            .unwrap();
    }
    let mut forges = List::with_capacity(1);
    forges
        .push(
            w::ForgeRow::new(
                &limits,
                w::ForgeRowParts {
                    starts_with: w::StartsWith::Repository,
                    makings: List::with_capacity(0),
                    host: b"git.example:3000".as_slice().into(),
                    number: 3,
                    kind: w::Kind::Github,
                    mode: w::Mode::Connected,
                    connection: w::Connection::Connected(
                        w::Connected::new(&limits, w::ConnectedParts { id: 42, login: b"app".as_slice().into() })
                            .unwrap(),
                    ),
                    offer: w::Offer::NotOffered,
                    first: None,
                    installations,
                    ended: None,
                    app: None,
                    web: None,
                    repositories: List::with_capacity(0),
                    owned: None,
                    admin: false,
                    read_only: false,
                    ended_stop: None,
                },
            )
            .unwrap(),
        )
        .unwrap();
    deliver(
        &mut root,
        client::Family {
            family: wire::FORGE,
            version: wire::CLIENT_VERSION,
            bytes: wire::forges(&w::Forges::new(&limits, w::ForgesParts { forges, linked: false }).unwrap()),
        },
    );
    let printed = root.answer(call).expect("settled wait completes");
    assert_eq!(printed.status, 0);
    assert!(printed.stderr.is_empty());
    assert!(f::json_has(&printed, br#""kind":"github""#));
    assert!(f::json_has(&printed, br#""met":true"#));
    for id in 1..=count {
        let mut measure = jig::render::Text::measure();
        measure.number(u64::from(id));
        let mut number = jig::render::Text::writing_exact(measure);
        number.number(u64::from(id));
        let installation = concat(&[
            br#"{"id":"#,
            &number.finish(),
            br#","account":{"id":111,"login":"acme"},"suspended":false,"stray":false}"#,
        ]);
        assert!(f::json_has(&printed, &installation));
    }
    assert!(printed.stdout.len() <= usize::try_from(root.limits.tools.text).unwrap());
    assert_eq!(root.closed.len(), 1);
    assert!(crate::root::next_deadline(&root.root).is_none());
}

#[test]
fn settled_wait_prints_an_accepted_installation_with_its_account() {
    settled_wait_with_installations(1);
}

#[test]
fn settled_wait_prints_the_wire_maximum_installations_with_their_accounts() {
    settled_wait_with_installations(forge::forges::INSTALLATIONS);
}

#[test]
fn human_forges_names_the_app_accounts_unknown_installations_and_their_states() {
    use forge::forges::{Account, Installation};
    let mut row = row(C::Connected { id: 42, login: Box::from(b"my-app[bot]".as_slice()) });
    row.kind = Kind::GitHub;
    row.installations = Box::new([
        Installation {
            id: 11,
            account: Some(Account { id: 111, login: Box::from(b"acme".as_slice()) }),
            suspended: false,
            stray: false,
        },
        Installation { id: 22, account: None, suspended: true, stray: true },
    ]);
    let plain = forges::print_rows(Some(&[row]), false, None, &f::limits_root());
    assert_eq!(plain.status, 0);
    assert!(
        skein_lib::bytes::find(
            &plain.stdout,
            " · my-app\n  installation 11 · acme\n  installation 22 · suspended · not accepted · kept · forge accept 3 22\n".as_bytes(),
        )
        .is_some()
    );
}

#[test]
fn human_forges_names_the_last_connect_reason_only_when_present() {
    let mut row = row(C::NotConnected);
    for ended in [None, Some(Box::from(b"the pasted value changed".as_slice()))] {
        row.ended = ended;
        let plain = forges::print_rows(Some(&[row.clone()]), false, None, &f::limits_root());
        assert_eq!(plain.status, 0);
        assert_eq!(skein_lib::bytes::find(&plain.stdout, b"The last connect stopped: ").is_some(), row.ended.is_some(),);
        if row.ended.is_some() {
            assert!(
                skein_lib::bytes::find(
                    &plain.stdout,
                    " · The last connect stopped: the pasted value changed\n".as_bytes(),
                )
                .is_some()
            );
        }
    }
}

#[test]
fn human_forges_prints_an_app_installed_nowhere_yet() {
    let mut row = row(C::Connected { id: 42, login: Box::from(b"my-app[bot]".as_slice()) });
    row.kind = Kind::GitHub;
    let plain = forges::print_rows(Some(&[row]), false, None, &f::limits_root());
    assert_eq!(plain.status, 0);
    assert!(skein_lib::bytes::find(&plain.stdout, " · my-app · installed nowhere yet\n".as_bytes()).is_some());
}

#[test]
fn human_forges_bounds_the_wire_maximum_installations_with_longest_logins() {
    let mut limits = f::limits_root();
    let preset = forge::wire_limits(&limits.domain.forge).unwrap();
    // The 80 KiB result fits both wire bounds; 32 KiB still exercises refusal.
    for (bounds, text, status) in [
        (preset, crate::limits::TEXT, 0),
        (w::CEILINGS, crate::limits::TEXT, 0),
        (w::CEILINGS, crate::limits::CLIENT_TEXT, 1),
    ] {
        limits.tools.text = text;
        let mut login = skein_lib::Writer::new(usize::try_from(bounds.account_login).unwrap());
        for _ in 0..bounds.account_login {
            login.put(b"x").unwrap();
        }
        let login = login.finish();
        let mut installations = List::with_capacity(forge::forges::INSTALLATIONS);
        for id in 1..=forge::forges::INSTALLATIONS {
            installations
                .push(
                    w::Installation::new(
                        &bounds,
                        w::InstallationParts {
                            id: u64::from(id),
                            account: Some(
                                w::Account::new(&bounds, w::AccountParts { id: u64::from(id), login: login.clone() })
                                    .unwrap(),
                            ),
                            suspended: true,
                            stray: true,
                        },
                    )
                    .unwrap(),
                )
                .unwrap();
        }
        let mut rows = List::with_capacity(1);
        rows.push(
            w::ForgeRow::new(
                &bounds,
                w::ForgeRowParts {
                    starts_with: w::StartsWith::Repository,
                    makings: List::with_capacity(0),
                    host: b"git.example:3000".as_slice().into(),
                    number: 3,
                    kind: w::Kind::Github,
                    mode: w::Mode::Connected,
                    connection: w::Connection::Connected(
                        w::Connected::new(
                            &bounds,
                            w::ConnectedParts { id: 42, login: b"my-app[bot]".as_slice().into() },
                        )
                        .unwrap(),
                    ),
                    offer: w::Offer::NotOffered,
                    first: None,
                    installations,
                    ended: None,
                    app: None,
                    web: None,
                    repositories: List::with_capacity(0),
                    owned: None,
                    admin: false,
                    read_only: false,
                    ended_stop: None,
                },
            )
            .unwrap(),
        )
        .unwrap();
        let record = client::Family {
            family: wire::FORGE,
            version: wire::CLIENT_VERSION,
            bytes: wire::forges(&w::Forges::new(&bounds, w::ForgesParts { forges: rows, linked: false }).unwrap()),
        };
        let mut domain_limits = limits.domain.forge;
        domain_limits.field_bytes = bounds.account_login;
        let rows = forge::forges::rows(&record, &domain_limits).unwrap();
        let plain = forges::print_rows(Some(&rows), false, None, &limits);
        assert_eq!(plain.status, status);
        if status == 0 {
            for words in [login.as_ref(), b"suspended", b"kept"] {
                let mut count = 0u32;
                for window in plain.stdout.windows(words.len()) {
                    if window == words {
                        count = count.checked_add(1u32).unwrap();
                    }
                }
                assert_eq!(count, forge::forges::INSTALLATIONS);
            }
            assert!(plain.stdout.len() <= usize::try_from(limits.tools.text).unwrap());
        } else {
            // The digest's JSON exceeds tools.text before the human row is rendered.
            assert!(plain.stdout.is_empty());
            assert_eq!(plain.stderr.as_ref(), b"result limit\n");
        }
    }
}

#[test]
fn setup_makes_the_default_or_named_project_then_adopts_the_answered_number() {
    for flags in [&[][..], &["--project", "Next"][..], &["--project", "1"][..]] {
        for json in [false, true] {
            let mut root = f::signed_in(f::PARTY);
            for restored in [false, true] {
                let mut argv = List::with_capacity(7);
                argv.push("forge").unwrap();
                argv.push("setup").unwrap();
                argv.push("3").unwrap();
                for flag in flags {
                    argv.push(*flag).unwrap();
                }
                argv.push("--key").unwrap();
                argv.push("setup-project").unwrap();
                if restored {
                    let mut previous = f::signed_in(f::PARTY);
                    let old = previous.submit(&["forge", "setup", "3", "--project", "999", "--key", "setup-project"]);
                    let mut records = f::restored_adoption(f::PARTY);
                    let record = records.get_mut(0).unwrap();
                    record.key = previous.sent_name(old).unwrap();
                    record.request.request = previous.adoption_bytes(old).clone();
                    root = f::started_with(records, f::SEED, json);
                    root.deliver(c::Delivery::Me(Box::new(f::me(f::PARTY))));
                }
                let call = root.submit_prepared(f::call(argv.as_slice()), json, None);
                root.adoption_answer(if restored { 0 } else { call }, w::Outcome::SettingUp);
                deliver(
                    &mut root,
                    record_first(
                        w::Connection::Connected(
                            w::Connected::new(
                                &w::CEILINGS,
                                w::ConnectedParts { id: 1, login: b"temper".as_slice().into() },
                            )
                            .unwrap(),
                        ),
                        Some(first_record(made())),
                    ),
                );
                let expected = if flags == ["--project", "1"] {
                    1
                } else {
                    let client::Ask::Jig(ask) = root.adoption_bytes(call) else { panic!("MakeProject before Adopt") };
                    let c::Ask::MakeProject { name, policy } = ask.as_ref() else { panic!("MakeProject") };
                    assert_eq!(name.bytes(), if flags.is_empty() { b"hello".as_slice() } else { b"Next" });
                    assert_eq!(*policy, 1);
                    assert_eq!(root.sent_name(call), Some(crate::key::engine(&f::name(b"start-project-3"))));
                    let (attempt, _, _) = root.sends.get(&call).unwrap();
                    root.client(client::Event::Up {
                        up: client::Up::Answer {
                            attempt: *attempt,
                            envelope: skein_lib::RequestEnvelope::Final,
                            answer: Some(client::Answered::Jig(c::Answer::Done(c::Outcome::Document(
                                c::DocumentOutcome::ProjectMade { project: 8 },
                            )))),
                            why: None,
                        },
                    });
                    8
                };
                assert_eq!(root.sent_adoption(call).project, expected);
                assert!(root.sent_adoption(call).home);
                assert_eq!(root.sent_name(call), Some(forge::first_repository_key(3, false)));
                root.adoption_answer(call, f::adopted_with_unknown_protection());
                let printed = root.answer(call).unwrap();
                assert_eq!(printed.status, 0);
                let line = if expected == 1 {
                    "project: 1 · shop\n"
                } else if flags.is_empty() {
                    "project: 8 · hello\n"
                } else {
                    "project: 8 · Next\n"
                };
                if json {
                    let prefix = if expected == 1 {
                        br#"{"project":1,"outcome":{"adopted":"#.as_slice()
                    } else {
                        br#"{"project":8,"outcome":{"adopted":"#.as_slice()
                    };
                    assert!(printed.stdout.starts_with(prefix));
                    f::unique_keys(&printed.stdout).unwrap();
                } else {
                    assert!(printed.stdout.starts_with(line.as_bytes()));
                }
            }
        }
    }
}

#[test]
fn a_refusal_names_the_tool_called() {
    use crate::jig::{
        call::{Value, Values},
        render::outcome,
    };
    const FILE: &[u8] = br#"{"items":[{"setting":{"kind":"rules"},"edit":"reset"}]}"#;
    for (verb, expected) in [
        ("apply", b"settings apply: stale; newest revision 2; read settings show again\n".as_slice()),
        ("propose", b"settings propose: stale; newest revision 2; read settings show again\n".as_slice()),
    ] {
        let mut root = f::signed_in(f::PARTY);
        let mut call = f::call(&["settings", verb, "FILE", "--against", "1", "--key", "refused-settings"]);
        *call.values.get_mut(0).expect("FILE") = Some(Values::One(Value::Text(FILE.into())));
        let number = root.submit_prepared(call, false, None);
        if verb == "apply" {
            let (_, _, request) = root.sends.get(&number).expect("sent settings request");
            let operation = outcome::Operation::of(request);
            let kept = outcome::kept(
                &operation,
                &client::Outcome::Answered(client::Answered::Jig(c::Answer::Done(c::Outcome::SettingsChanged(
                    c::OutcomeSettingsChanged { revision: 2 },
                )))),
                false,
                None,
                &root.limits.tools,
                crate::root::DIAGNOSTIC_PREFIX,
            );
            assert_eq!(kept.as_ref(), b"kept: change-settings, revision 2\n");
        }
        root.answered(
            number,
            c::Answer::Refused { refusal: c::Refusal::Stale(c::RefusalStale { newest: 2 }), at: None },
        );
        let printed = root.answer(number).expect("settings refusal");
        assert_eq!((printed.status, printed.stderr.as_ref()), (2, expected));
    }
    let mut root = f::signed_in(f::PARTY);
    let call = root.submit(&["forge", "setup", "3"]);
    root.adoption_answer(call, w::Outcome::SettingUp);
    deliver(
        &mut root,
        record_first(
            w::Connection::Connected(
                w::Connected::new(&w::CEILINGS, w::ConnectedParts { id: 1, login: b"temper".as_slice().into() })
                    .unwrap(),
            ),
            Some(first_record(made())),
        ),
    );
    let (attempt, _, _) = root.sends.get(&call).unwrap();
    root.client(client::Event::Up {
        up: client::Up::Answer {
            attempt: *attempt,
            envelope: skein_lib::RequestEnvelope::Final,
            answer: Some(client::Answered::Jig(c::Answer::Refused { refusal: c::Refusal::NotOffered, at: None })),
            why: None,
        },
    });
    let printed = root.answer(call).unwrap();
    assert_eq!(printed.status, 2);
    assert_eq!(printed.stderr.as_ref(), b"forge setup: This act is not offered\n");
    let client::Ask::Jig(_) = root.adoption_bytes(call) else { panic!("no adoption after refusal") };
}

#[test]
fn playground_setup_sends_its_copy_and_default_or_explicit_project() {
    // Group, verb, forge, playground switch, project flag and value.
    const PLAYGROUND_WORDS: u32 = 6;
    for flags in [&["--playground"][..], &["--playground", "--project", "copy"][..]] {
        let mut root = f::signed_in(f::PARTY);
        let mut words = List::with_capacity(PLAYGROUND_WORDS);
        words.push("forge").unwrap();
        words.push("setup").unwrap();
        words.push("3").unwrap();
        for word in flags {
            words.push(*word).unwrap();
        }
        let call = root.submit(words.as_slice());
        let client::Ask::Family { record, .. } = root.adoption_bytes(call) else { panic!("family") };
        let w::Request::SetUp(value) =
            wire::decode_requesting(&record.bytes, &w::CEILINGS).unwrap().into_parts().request
        else {
            panic!("setup")
        };
        assert!(empty_setup(value.first()).playground());
        assert_eq!(
            (empty_setup(value.first()).org(), empty_setup(value.first()).repository()),
            (b"playground".as_slice(), b"widget".as_slice())
        );
    }
}
#[test]
fn playground_setup_rejects_org_or_repository_before_sending() {
    for flag in ["--org", "--repository"] {
        let mut root = f::signed_in(f::PARTY);
        let call = root.submit(&["forge", "setup", "3", "--playground", flag, "other"]);
        assert_eq!(root.answer(call).unwrap().status, 1);
        assert!(root.nothing_sent(), "a usage error must send no request");
    }
}

#[test]
fn changed_with_since_waits_for_a_forge_change_after_the_baseline() {
    let mut root = f::signed_in(f::PARTY);
    let read = root.submit(&["forge", "list", "--json"]);
    deliver(&mut root, record(w::Connection::NotConnected));
    let digest = f::last_digest(&root.answer(read).unwrap().stdout);
    let call = root.wait_since(&["forge", "list"], &["--until", "changed"], &digest);
    for connection in [w::Connection::NotSetUp, w::Connection::NotConnected] {
        deliver(&mut root, record(connection));
        assert!(root.answer(call).is_none());
    }
    deliver(&mut root, record(w::Connection::NotSetUp));
    assert_eq!(root.answer(call).unwrap().status, 0);
}

#[test]
fn forge_setup_without_a_number_sends_a_new_owned_forge() {
    for words in [
        &["forge", "setup"][..],
        &["forge", "setup", "--playground"],
        &["forge", "setup", "--org", "team", "--repository", "widget", "--project", "3"],
    ] {
        let mut root = f::signed_in(f::PARTY);
        let call = root.submit(words);
        let client::Ask::Family { scope, record } = root.adoption_bytes(call) else { panic!("family") };
        assert_eq!(*scope, c::Scope::Deployment);
        let w::Request::SetUp(request) =
            wire::decode_requesting(&record.bytes, &w::CEILINGS).unwrap().into_parts().request
        else {
            panic!("setup")
        };
        let w::Making::New(new) = request.forge() else { panic!("new forge") };
        assert_eq!(new.forge(), &w::ForgeAsk::Owned);
        assert_eq!(empty_setup(request.first()).playground(), words.contains(&"--playground"));
        if words.contains(&"--org") {
            assert_eq!(empty_setup(request.first()).org(), b"team");
            assert_eq!(empty_setup(request.first()).repository(), b"widget");
        } else if empty_setup(request.first()).playground() {
            assert_eq!(empty_setup(request.first()).org(), b"playground");
            assert_eq!(empty_setup(request.first()).repository(), b"widget");
        } else {
            assert_eq!(empty_setup(request.first()).org(), b"work");
            assert_eq!(empty_setup(request.first()).repository(), b"hello");
        }
    }
}

#[test]
fn forge_remove_prints_removed_and_its_refusal() {
    lifecycle_answers(
        &["forge", "remove", "3"],
        &[
            (w::Outcome::Removed, b"removed\n", br#"{"removed":{}}"#, 0),
            (
                w::Outcome::RemoveRefused(
                    w::RemoveRefused::new(&w::CEILINGS, w::RemoveRefusedParts { why: w::ForgeRefusal::NoSuchForge })
                        .unwrap(),
                ),
                b"refused: no such forge\n",
                br#"{"remove_refused":{"why":"no_such_forge"}}"#,
                2,
            ),
            (
                w::Outcome::RemoveRefused(
                    w::RemoveRefused::new(&w::CEILINGS, w::RemoveRefusedParts { why: w::ForgeRefusal::Busy }).unwrap(),
                ),
                b"refused: busy\n",
                br#"{"remove_refused":{"why":"busy"}}"#,
                2,
            ),
        ],
    );
}
#[test]
fn playground_end_prints_reset_and_not_offered() {
    lifecycle_answers(
        &["playground", "end"],
        &[
            (w::Outcome::Reset, b"reset\n", br#"{"reset":{}}"#, 0),
            (
                w::Outcome::EndRefused(
                    w::EndRefused::new(&w::CEILINGS, w::EndRefusedParts { why: w::EndRefusal::NotOffered }).unwrap(),
                ),
                b"refused: not offered\n",
                br#"{"end_refused":{"why":"not_offered"}}"#,
                2,
            ),
        ],
    );
}
fn lifecycle_answers(words: &[&str], answers: &[(w::Outcome, &[u8], &[u8], u8)]) {
    for (outcome, text, json_text, status) in answers {
        for json in [false, true] {
            let mut root = f::signed_in(f::PARTY);
            let call = root.submit_prepared(f::call(words), json, None);
            let client::Ask::Family { scope, record } = root.adoption_bytes(call) else { panic!("family") };
            assert_eq!(*scope, c::Scope::Deployment);
            match wire::decode_requesting(&record.bytes, &w::CEILINGS).unwrap().request() {
                w::Request::EndPlayground => assert_eq!(words, &["playground", "end"]),
                w::Request::RemoveForge(value) => {
                    assert_eq!(words, &["forge", "remove", "3"]);
                    assert_eq!(value.forge(), 3);
                }
                other @ (w::Request::Adopt(_)
                | w::Request::SetUp(_)
                | w::Request::Replace(_)
                | w::Request::Connect(_)
                | w::Request::Install(_)
                | w::Request::Reachable(_)
                | w::Request::Accept(_)
                | w::Request::DismissMaking(_)
                | w::Request::MakeRepository(_)
                | w::Request::SetAway(_)
                | w::Request::Overturn(_)
                | w::Request::Looked(_)
                | w::Request::FollowAct(_)
                | w::Request::TryAccount(_)
                | w::Request::Dismiss(_)
                | w::Request::StartPlayground) => panic!("unexpected {other:?}"),
            }
            root.adoption_answer(call, outcome.clone());
            let printed = root.answer(call).unwrap();
            assert_eq!(printed.status, *status);
            let bytes = if *status == 0 { &printed.stdout } else { &printed.stderr };
            assert_eq!(bytes.as_ref(), if json { *json_text } else { *text });
        }
    }
}

#[test]
fn lifecycle_requests_recover_with_their_operation_names() {
    for (words, outcome, text, json_text) in [
        (
            &["forge", "remove", "3"][..],
            w::Outcome::Removed,
            b"kept: forge-remove removed\n".as_slice(),
            b"{\"operation\":\"forge-remove\",\"outcome\":{\"removed\":{}}}\n".as_slice(),
        ),
        (
            &["playground", "end"][..],
            w::Outcome::Reset,
            b"kept: playground-end reset\n".as_slice(),
            b"{\"operation\":\"playground-end\",\"outcome\":{\"reset\":{}}}\n".as_slice(),
        ),
    ] {
        for json in [false, true] {
            let mut root = f::signed_in(f::PARTY);
            let call = root.submit(words);
            assert!(root.saved.contains_key(&call));
            let mut records = f::restored_adoption(f::PARTY);
            let record = records.get_mut(0).unwrap();
            record.key = root.sent_name(call).unwrap();
            record.request.request = root.adoption_bytes(call).clone();
            let mut root = f::started_with(records, f::SEED, json);
            root.deliver(c::Delivery::Me(Box::new(f::me(f::PARTY))));
            root.adoption_answer(0, outcome.clone());
            let expected = if json { json_text } else { text };
            assert_eq!(root.kept_lines.as_slice(), &[Box::<[u8]>::from(expected)]);
        }
    }
}

#[test]
fn new_owned_forge_setup_adopts_on_the_number_in_the_forges_read() {
    let mut root = f::signed_in(f::PARTY);
    let call = root.submit(&["forge", "setup", "--project", "1"]);
    root.adoption_answer(call, w::Outcome::SettingUp);
    assert!(root.answer(call).is_none());
    deliver(
        &mut root,
        record_first(
            w::Connection::Connected(
                w::Connected::new(&w::CEILINGS, w::ConnectedParts { id: 1, login: b"temper".as_slice().into() })
                    .unwrap(),
            ),
            Some(first_record(made())),
        ),
    );
    let adopted = root.sent_adoption(call);
    assert_eq!(adopted.forge, 3);
    assert_eq!(adopted.project, 1);
    assert_eq!(adopted.owner.as_ref(), b"work");
    assert_eq!(adopted.name.as_ref(), b"hello");
}

#[test]
fn human_forges_shows_a_refused_job_log_origin_as_a_diagnostic() {
    let mut row = row(C::Connected { id: 42, login: b"my-app[bot]".as_slice().into() });
    row.kind = Kind::GitHub;
    row.ended = Some(b"Job-log origin refused: https://unrelated.test:443".as_slice().into());
    let plain = forges::print_rows(Some(&[row]), false, None, &f::limits_root());
    assert_eq!(plain.status, 0);
    assert!(skein_lib::bytes::find(&plain.stdout, b"Job-log origin refused: https://unrelated.test:443").is_some());
    assert!(skein_lib::bytes::find(&plain.stdout, b"The last connect stopped").is_none());
}

fn empty_setup(first: &w::SetUpFirst) -> &w::FirstEmpty {
    let w::SetUpFirst::Empty(first) = first else { panic!("empty setup fixture") };
    first
}

#[test]
fn deferred_choices_print_their_intent_without_a_made_repository() {
    use forge::forges::StartsWith;
    for (starts_with, fields) in [
        (StartsWith::Nothing, br#""starts_with":"nothing""#.as_slice()),
        (
            StartsWith::Copies { addresses: Box::new([Box::from(b"https://source.example/owner/repo".as_slice())]) },
            br#""starts_with":"copies","copies":["https://source.example/owner/repo"]"#,
        ),
    ] {
        let mut row = row(C::Connected { id: 1, login: b"temper".as_slice().into() });
        row.starts_with = starts_with;
        let rows = [row];
        let printed = forges::print_rows(Some(&rows), true, None, &f::limits_root());
        assert!(skein_lib::bytes::find(&printed.stdout, fields).is_some());
        assert!(skein_lib::bytes::find(&printed.stdout, br#""first":null"#).is_some());
        let printed = forges::print_rows(Some(&rows), false, None, &f::limits_root());
        assert!(skein_lib::bytes::find(&printed.stdout, b"no repository").is_some());
        assert!(!forges::holds(&rows, b"made:3"));
    }
}

#[test]
fn sixteen_forges_with_sixteen_copy_addresses_fit_the_client_document() {
    // jig's carried family request ceiling is one MiB.
    const FAMILY_REQUEST: u64 = 1_048_576;
    let limits = f::limits_root();
    let bounds = forge::wire_limits(&limits.domain.forge).unwrap();
    let mut addresses = List::with_capacity(wire::FIRST_COPIES);
    for _ in 0..wire::FIRST_COPIES {
        addresses.push(skein_lib::bytes::zeroed(wire::source::ADDRESS_BYTES)).unwrap();
    }
    let copies = w::CopyAddresses::new(&bounds, w::CopyAddressesParts { addresses }).unwrap();
    let one = record(w::Connection::NotSetUp);
    let template =
        wire::decode_forges(&one.bytes, &bounds).unwrap().into_parts().forges.into_boxed()[0].clone().into_parts();
    let mut rows = List::with_capacity(wire::FORGES);
    for number in 1..=wire::FORGES {
        let mut row = template.clone();
        row.number = u16::try_from(number).unwrap();
        row.starts_with = w::StartsWith::Copies(copies.clone());
        rows.push(w::ForgeRow::new(&bounds, row).unwrap()).unwrap();
    }
    let forges = w::Forges::new(&bounds, w::ForgesParts { forges: rows, linked: false }).unwrap();
    let encoded = wire::forges(&forges);
    assert!(w::Forges::decoded_bound(&bounds).is_some());
    let request = w::Requesting::encoded_bound(&bounds).unwrap();
    assert!(u64::try_from(encoded.len()).unwrap() <= limits.domain.client.domain.document_bytes);
    assert_eq!(wire::decode_forges(&encoded, &bounds), Some(forges));
    assert!(request <= FAMILY_REQUEST);
}

#[test]
fn setup_ends_on_its_adoption_without_the_protection_line() {
    for words in
        [&["forge", "setup", "3", "--project", "shop"][..], &["forge", "setup", "3", "--project", "shop", "--json"][..]]
    {
        let mut root = f::signed_in(f::PARTY);
        let call = root.submit(words);
        root.adoption_answer(call, w::Outcome::SettingUp);
        deliver(
            &mut root,
            record_first(
                w::Connection::Connected(
                    w::Connected::new(&w::CEILINGS, w::ConnectedParts { id: 1, login: b"temper".as_slice().into() })
                        .unwrap(),
                ),
                Some(first_record(made())),
            ),
        );
        root.adoption_answer(call, f::adopted_with_unknown_protection());
        let printed = root.answer(call).unwrap();
        assert_eq!(printed.status, 0);
        if words.contains(&"--json") {
            assert!(skein_lib::bytes::find(&printed.stdout, br#""protection":{"unknown":{}}"#).is_some());
        } else {
            assert!(printed.stdout.starts_with("project: 1 · shop\nAdopted ".as_bytes()));
            assert!(skein_lib::bytes::find(&printed.stdout, b"may not read the branch protection").is_none());
        }
    }
}

/// Complete the first owned setup before testing another project's ctl path.
pub(super) fn setup_for_projects() -> f::Harness {
    let mut root = f::signed_in(f::PARTY);
    let call = root.submit(&["forge", "setup", "--project", "1", "--key", "first-setup"]);
    root.adoption_answer(call, w::Outcome::SettingUp);
    project_forge(&mut root);
    assert_eq!(root.sent_adoption(call).project, 1);
    root.adoption_answer(call, f::adopted_with_unknown_protection());
    assert_eq!(root.answer(call).unwrap().status, 0);
    root
}

/// Deliver a made repository adopted into the requested project on forge 3.
pub(super) fn adopted_project_repository(root: &mut f::Harness, project: u32) {
    let limits = forge::wire_limits(&root.limits.domain.forge).unwrap();
    let record = record(w::Connection::Connecting);
    let mut parts = wire::decode_forges(&record.bytes, &limits).unwrap().into_parts();
    let mut row = parts.forges.swap_remove(0).unwrap().into_parts();
    row.makings = List::with_capacity(1);
    row.makings
        .push(
            w::RepositoryMaking::new(
                &limits,
                w::RepositoryMakingParts {
                    org: b"work".as_slice().into(),
                    name: b"second".as_slice().into(),
                    from: w::MakeFrom::Empty,
                    into: Some(w::MakeInto::new(&limits, w::MakeIntoParts { project, home: true }).unwrap()),
                    state: w::MakingState::Adopting(
                        w::MakingMade::new(&limits, w::MakingMadeParts { id: 8, branch: b"main".as_slice().into() })
                            .unwrap(),
                    ),
                },
            )
            .unwrap(),
        )
        .unwrap();
    parts.forges.push(w::ForgeRow::new(&limits, row).unwrap()).unwrap();
    deliver(root, client::Family { bytes: wire::forges(&w::Forges::new(&limits, parts).unwrap()), ..record.clone() });
    deliver(root, record);
}

/// Refresh the owned forge document used by the project creation walk.
pub(super) fn project_forge(root: &mut f::Harness) {
    deliver(
        root,
        record_first(
            w::Connection::Connected(
                w::Connected::new(&w::CEILINGS, w::ConnectedParts { id: 1, login: b"temper".as_slice().into() })
                    .unwrap(),
            ),
            Some(first_record(made())),
        ),
    );
}
