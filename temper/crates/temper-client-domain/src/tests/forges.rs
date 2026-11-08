#[path = "forge_replace.rs"]
mod forge_replace;
#[path = "forgejo_tokens.rs"]
mod forgejo_tokens;
use super::fixtures;
use crate::forge::{
    self, family,
    forges::{
        Binary, Connection, Fetched, Kind, Mode, Offer, Row, SetUpAnswer, SetUpOutcome, SetUpRefusal, Step, ask, rows,
        watch,
    },
    words,
};
use jig_client::{Answered, Ask, Family, Watchable, contract as c};
use skein_lib::List;
use temper_client_protocol_wire::{self as wire, v1 as w};
fn limits() -> w::Limits {
    forge::wire_limits(&fixtures::limits().forge).unwrap()
}
fn answer(outcome: w::Outcome) -> Family {
    Family {
        family: wire::FORGE,
        version: wire::CLIENT_VERSION,
        bytes: wire::answer(&w::Answer::new(&limits(), w::AnswerParts { outcome }).unwrap()),
    }
}
#[test]
fn set_up_is_a_deployment_request_and_forges_a_deployment_watch() {
    let Ask::Family { scope, record } =
        ask(3, b"work".as_slice().into(), b"hello".as_slice().into(), false, &fixtures::limits().forge).unwrap()
    else {
        panic!("family")
    };
    assert_eq!(scope, c::Scope::Deployment);
    assert_eq!((record.family, record.version), (wire::FORGE, wire::CLIENT_VERSION));
    let w::Request::SetUp(request) = wire::decode_requesting(&record.bytes, &limits()).unwrap().into_parts().request
    else {
        panic!("set up")
    };
    let w::Making::Kept(kept) = request.forge() else { panic!("kept forge") };
    assert_eq!(kept.forge(), 3);
    let Watchable::Family { scope, window, record } = watch(&fixtures::limits().forge).unwrap() else {
        panic!("family")
    };
    assert_eq!(scope, c::Scope::Deployment);
    assert_eq!(window, c::Window { rows: 0, after: None });
    assert_eq!(*wire::decode_watching(&record.bytes, &limits()).unwrap().watch(), w::Watch::Forges);
}
#[test]
fn set_up_accepts_only_its_outcomes_and_the_shared_unknown() {
    for (wire, expected) in [
        (w::Outcome::SettingUp, SetUpOutcome::SettingUp),
        (w::Outcome::Unknown, SetUpOutcome::Unknown),
        (
            w::Outcome::SetUpRefused(
                w::SetUpRefused::new(&limits(), w::SetUpRefusedParts { why: w::SetUpRefusal::NotOffered }).unwrap(),
            ),
            SetUpOutcome::Refused(SetUpRefusal::NotOffered),
        ),
        (
            w::Outcome::SetUpRefused(
                w::SetUpRefused::new(&limits(), w::SetUpRefusedParts { why: w::SetUpRefusal::NotUnderTheBox }).unwrap(),
            ),
            SetUpOutcome::Refused(SetUpRefusal::NotUnderTheBox),
        ),
        (
            w::Outcome::SetUpRefused(
                w::SetUpRefused::new(&limits(), w::SetUpRefusedParts { why: w::SetUpRefusal::Busy }).unwrap(),
            ),
            SetUpOutcome::Refused(SetUpRefusal::Busy),
        ),
    ] {
        let record = answer(wire);
        assert_eq!(family::set_up_answer(&record, &limits()), Some(expected));
        assert_eq!(
            forge::forges::answer(&Answered::Family(record), &fixtures::limits().forge),
            Some(SetUpAnswer::Outcome(expected))
        );
    }
    let adoption = answer(w::Outcome::ForgeRefused(
        w::ForgeRefused::new(&limits(), w::ForgeRefusedParts { reason: w::ForgeReason::NoSuchForge }).unwrap(),
    ));
    assert_eq!(family::set_up_answer(&adoption, &limits()), None);
    let link =
        jig_client_protocol_link::Limits::new(fixtures::limits().client.domain, fixtures::limits().client.exchanges)
            .unwrap();
    for outcome in [
        w::Outcome::SettingUp,
        w::Outcome::SetUpRefused(
            w::SetUpRefused::new(&limits(), w::SetUpRefusedParts { why: w::SetUpRefusal::Busy }).unwrap(),
        ),
    ] {
        assert_eq!(family::answer(&answer(outcome), &limits(), &link), None);
    }
    let mut record = answer(w::Outcome::SettingUp);
    record.bytes = Box::from(b"bad".as_slice());
    assert_eq!(family::set_up_answer(&record, &limits()), None);
}
fn cases() -> Box<[(w::Connection, Connection, &'static [u8])]> {
    Box::new([
        (w::Connection::NotConnected, Connection::NotConnected, b"Not connected"),
        (w::Connection::NotSetUp, Connection::NotSetUp, b"Not set up"),
        (
            w::Connection::SettingUp(
                w::SettingUp::new(&limits(), w::SettingUpParts { step: None, fetched: None }).unwrap(),
            ),
            Connection::SettingUp { step: None },
            b"Setting up",
        ),
        (
            w::Connection::SettingUp(
                w::SettingUp::new(&limits(), w::SettingUpParts { step: Some(w::SetUpStep::Fetch), fetched: None })
                    .unwrap(),
            ),
            Connection::SettingUp { step: Some(Step::Fetch) },
            b"Setting up: fetching",
        ),
        (
            w::Connection::SettingUp(
                w::SettingUp::new(&limits(), w::SettingUpParts { step: Some(w::SetUpStep::Prepare), fetched: None })
                    .unwrap(),
            ),
            Connection::SettingUp { step: Some(Step::Prepare) },
            b"Setting up: preparing",
        ),
        (
            w::Connection::SettingUp(
                w::SettingUp::new(&limits(), w::SettingUpParts { step: Some(w::SetUpStep::Run), fetched: None })
                    .unwrap(),
            ),
            Connection::SettingUp { step: Some(Step::Run) },
            b"Setting up: starting",
        ),
        (
            w::Connection::SetUpFailed(
                w::SetUpFailed::new(
                    &limits(),
                    w::SetUpFailedParts { why: Box::from(b"fetch failed".as_slice()), at: None },
                )
                .unwrap(),
            ),
            Connection::SetUpFailed { why: Box::from(b"fetch failed".as_slice()), at: None },
            b"Set up failed: fetch failed",
        ),
        (w::Connection::Connecting, Connection::Connecting, b"Connecting"),
        (
            w::Connection::Connected(
                w::Connected::new(&limits(), w::ConnectedParts { id: 42, login: Box::from(b"bot".as_slice()) })
                    .unwrap(),
            ),
            Connection::Connected { id: 42, login: Box::from(b"bot".as_slice()) },
            b"Connected as bot (42)",
        ),
        (
            w::Connection::Stopped(
                w::Stopped::new(&limits(), w::StoppedParts { reason: Box::from(b"retired".as_slice()) }).unwrap(),
            ),
            Connection::Stopped { reason: Box::from(b"retired".as_slice()) },
            b"Stopped: retired",
        ),
    ])
}
fn fixture_wire_owned(m: Mode) -> Option<w::OwnedForge> {
    if m == Mode::Owned {
        Some(
            w::OwnedForge::new(
                &limits(),
                w::OwnedForgeParts {
                    forgejo: b"17.0.0".as_slice().into(),
                    runner: Some(b"13.2.0".as_slice().into()),
                    password: b"/fake/forge/person_password".as_slice().into(),
                },
            )
            .unwrap(),
        )
    } else {
        None
    }
}
fn fixture_owned(m: Mode) -> Option<forge::forges::Owned> {
    if m == Mode::Owned {
        Some(forge::forges::Owned {
            forgejo: b"17.0.0".as_slice().into(),
            runner: Some(b"13.2.0".as_slice().into()),
            password: b"/fake/forge/person_password".as_slice().into(),
        })
    } else {
        None
    }
}
pub(super) fn wire_connection_row(
    number: u16,
    kind: w::Kind,
    mode: w::Mode,
    connection: w::Connection,
    offer: w::Offer,
    owned: Option<w::OwnedForge>,
) -> w::ForgeRow {
    let mut repositories = List::with_capacity(1);
    repositories
        .push(
            w::Adoption::new(
                &limits(),
                w::AdoptionParts { owner: b"acme".as_slice().into(), name: b"app".as_slice().into(), project: 7 },
            )
            .unwrap(),
        )
        .unwrap();
    w::ForgeRow::new(
        &limits(),
        w::ForgeRowParts {
            starts_with: w::StartsWith::Repository,
            host: b"git.example:3000".as_slice().into(),
            makings: List::with_capacity(0),
            number,
            kind,
            mode,
            connection,
            offer,
            first: None,
            installations: List::with_capacity(0),
            ended: None,
            app: None,
            web: Some(b"https://forge.example".as_slice().into()),
            repositories,
            owned,
            admin: false,
            read_only: false,
            ended_stop: None,
        },
    )
    .unwrap()
}
#[test]
fn every_connection_decodes_into_the_forges_words() {
    let mut records = List::with_capacity(10);
    let mut expected = List::with_capacity(10);
    for (index, (connection, state, words)) in cases().into_iter().enumerate() {
        let (kind, mode, offer, k, m, o) = match index % 4 {
            0 => (
                w::Kind::Forgejo,
                w::Mode::Connected,
                w::Offer::NotOffered,
                Kind::Forgejo,
                Mode::Connected,
                Offer::NotOffered,
            ),
            1 => (w::Kind::Github, w::Mode::Managed, w::Offer::Offered, Kind::GitHub, Mode::Managed, Offer::Offered),
            2 => (
                w::Kind::Forgejo,
                w::Mode::Owned,
                w::Offer::NotUnderTheBox,
                Kind::Forgejo,
                Mode::Owned,
                Offer::NotUnderTheBox,
            ),
            _ => (w::Kind::Github, w::Mode::Fake, w::Offer::NotOffered, Kind::GitHub, Mode::Fake, Offer::NotOffered),
        };
        let number = u16::try_from(index + 1).unwrap();
        records.push(wire_connection_row(number, kind, mode, connection, offer, fixture_wire_owned(m))).unwrap();
        let row = Row {
            starts_with: forge::forges::StartsWith::Repository,
            makings: Box::new([]),
            host: b"git.example:3000".as_slice().into(),
            number,
            kind: k,
            mode: m,
            connection: state,
            fetched: None,
            offer: o,
            first: None,
            installations: Box::new([]),
            ended: None,
            app: None,
            web: Some(b"https://forge.example".as_slice().into()),
            repositories: Box::new([forge::forges::Adoption {
                owner: b"acme".as_slice().into(),
                name: b"app".as_slice().into(),
                project: 7,
            }]),
            owned: fixture_owned(m),
            admin: false,
            read_only: false,
            ended_stop: None,
        };
        assert_eq!(words::forge_connection(&row).as_ref(), words);
        expected.push(row).unwrap();
    }
    let record = Family {
        family: wire::FORGE,
        version: wire::CLIENT_VERSION,
        bytes: wire::forges(&w::Forges::new(&limits(), w::ForgesParts { forges: records, linked: false }).unwrap()),
    };
    assert_eq!(rows(&record, &fixtures::limits().forge).unwrap().as_ref(), expected.as_slice());
    for bytes in [b"bad".as_slice(), &record.bytes[..record.bytes.len() - 1]] {
        assert!(rows(&Family { bytes: bytes.into(), ..record.clone() }, &fixtures::limits().forge).is_none());
    }
    assert!(rows(&Family { version: wire::CLIENT_VERSION + 1, ..record.clone() }, &fixtures::limits().forge).is_none());
    assert!(rows(&Family { family: wire::FORGE + 1, ..record }, &fixtures::limits().forge).is_none());
}
#[test]
fn the_offer_drives_the_manual_words_and_setup_actions() {
    let mut row = Row {
        starts_with: forge::forges::StartsWith::Repository,
        makings: Box::new([]),
        host: b"git.example:3000".as_slice().into(),
        number: 3,
        kind: Kind::Forgejo,
        mode: Mode::Owned,
        connection: Connection::NotSetUp,
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
    };
    assert_eq!(words::forge_set_up_action(&row), Some(b"Set up a Forgejo here".as_slice()));
    row.offer = Offer::NotUnderTheBox;
    assert_eq!(words::forge_set_up_action(&row), None);
    assert_eq!(words::forge_connection(&row).as_ref(), b"Not set up: start temper with temper box");
    row.connection = Connection::SetUpFailed { why: Box::from(b"why".as_slice()), at: None };
    row.offer = Offer::Offered;
    assert_eq!(words::forge_set_up_action(&row), Some(b"Set up again".as_slice()));
    for (kind, text) in [(Kind::Forgejo, b"Forgejo".as_slice()), (Kind::GitHub, b"GitHub")] {
        assert_eq!(words::forge_kind(kind), text);
    }
    for (mode, text) in [
        (Mode::Connected, b"connected".as_slice()),
        (Mode::Managed, b"managed"),
        (Mode::Owned, b"owned"),
        (Mode::Fake, b"fake"),
    ] {
        assert_eq!(words::forge_mode(mode), text);
    }
    for (outcome, text) in [
        (SetUpOutcome::SettingUp, b"setting up".as_slice()),
        (SetUpOutcome::Refused(SetUpRefusal::NotOffered), b"not offered"),
        (SetUpOutcome::Refused(SetUpRefusal::NotUnderTheBox), b"not under the box"),
        (SetUpOutcome::Refused(SetUpRefusal::Busy), b"busy"),
        (SetUpOutcome::Unknown, b"unknown"),
    ] {
        assert_eq!(words::set_up_outcome(outcome), text);
    }
    row.connection = Connection::Stopped { reason: Box::from([255u8]) };
    assert_eq!(words::forge_connection(&row).as_ref(), "Stopped: �".as_bytes());
}

fn decoded_connection_row(connection: w::Connection) -> Row {
    const FORGE: u16 = 3;
    let mut forges = List::with_capacity(1);
    forges
        .push(wire_connection_row(FORGE, w::Kind::Forgejo, w::Mode::Owned, connection, w::Offer::Offered, None))
        .unwrap();
    let record = w::Forges::new(&limits(), w::ForgesParts { forges, linked: false }).unwrap();
    family::forges(&wire::forges(&record), &limits()).unwrap().into_iter().next().unwrap()
}

fn fetched_connection(step: Option<w::SetUpStep>, fetched: Option<Fetched>) -> w::Connection {
    let fetched = match fetched {
        Some(fetched) => Some(
            w::SetUpFetched::new(
                &limits(),
                w::SetUpFetchedParts {
                    binary: match fetched.binary {
                        Binary::Forgejo => w::SetUpBinary::Forgejo,
                        Binary::Runner => w::SetUpBinary::Runner,
                    },
                    done: fetched.done,
                    total: fetched.total,
                },
            )
            .unwrap(),
        ),
        None => None,
    };
    w::Connection::SettingUp(w::SettingUp::new(&limits(), w::SettingUpParts { step, fetched }).unwrap())
}

#[test]
fn the_row_says_how_far_the_fetch_has_gone() {
    for (fetched, connection, bare, figure, name) in [
        (
            Fetched { binary: Binary::Forgejo, done: 42, total: Some(100) },
            b"Setting up: fetching Forgejo \xc2\xb7 42%".as_slice(),
            b"Setting up: fetching Forgejo".as_slice(),
            b"42%".as_slice(),
            b"Fetching Forgejo".as_slice(),
        ),
        (
            Fetched { binary: Binary::Runner, done: 80, total: Some(100) },
            b"Setting up: fetching its runner \xc2\xb7 80%",
            b"Setting up: fetching its runner",
            b"80%",
            b"Fetching its runner",
        ),
        (
            Fetched { binary: Binary::Forgejo, done: 41_999_999, total: None },
            b"Setting up: fetching Forgejo \xc2\xb7 41 MB",
            b"Setting up: fetching Forgejo",
            b"41 MB",
            b"Fetching Forgejo",
        ),
        (
            Fetched { binary: Binary::Forgejo, done: 419, total: Some(1_000) },
            b"Setting up: fetching Forgejo \xc2\xb7 41%",
            b"Setting up: fetching Forgejo",
            b"41%",
            b"Fetching Forgejo",
        ),
        (
            Fetched { binary: Binary::Forgejo, done: 0, total: Some(1) },
            b"Setting up: fetching Forgejo \xc2\xb7 0%",
            b"Setting up: fetching Forgejo",
            b"0%",
            b"Fetching Forgejo",
        ),
        (
            Fetched { binary: Binary::Runner, done: 1, total: Some(1_000) },
            b"Setting up: fetching its runner \xc2\xb7 0%",
            b"Setting up: fetching its runner",
            b"0%",
            b"Fetching its runner",
        ),
        (
            Fetched { binary: Binary::Forgejo, done: u64::MAX, total: Some(u64::MAX) },
            b"Setting up: fetching Forgejo \xc2\xb7 100%",
            b"Setting up: fetching Forgejo",
            b"100%",
            b"Fetching Forgejo",
        ),
        (
            Fetched { binary: Binary::Runner, done: 999_999, total: None },
            b"Setting up: fetching its runner \xc2\xb7 0 MB",
            b"Setting up: fetching its runner",
            b"0 MB",
            b"Fetching its runner",
        ),
    ] {
        let row = decoded_connection_row(fetched_connection(Some(w::SetUpStep::Fetch), Some(fetched)));
        assert_eq!(row.fetched, Some(fetched));
        assert_eq!(row.connection, Connection::SettingUp { step: Some(Step::Fetch) });
        assert_eq!(words::forge_connection(&row).as_ref(), connection);
        assert_eq!(words::forge_connection_bare(&row).as_ref(), bare);
        assert_eq!(words::fetched_figure(fetched).as_ref(), figure);
        assert_eq!(words::fetching_name(fetched), name);
    }
}

#[test]
fn a_fetched_at_another_step_or_out_of_range_reads_as_none() {
    for (step, done, total, expected) in [
        (Some(w::SetUpStep::Fetch), 0, Some(0), b"Setting up: fetching".as_slice()),
        (Some(w::SetUpStep::Fetch), 1, Some(0), b"Setting up: fetching"),
        (Some(w::SetUpStep::Fetch), 2, Some(1), b"Setting up: fetching"),
        (Some(w::SetUpStep::Prepare), 1, Some(2), b"Setting up: preparing"),
        (Some(w::SetUpStep::Run), 1, Some(2), b"Setting up: starting"),
        (None, 1, Some(2), b"Setting up"),
        (Some(w::SetUpStep::Prepare), 1, None, b"Setting up: preparing"),
        (Some(w::SetUpStep::Run), 1, None, b"Setting up: starting"),
        (None, 1, None, b"Setting up"),
    ] {
        let row =
            decoded_connection_row(fetched_connection(step, Some(Fetched { binary: Binary::Forgejo, done, total })));
        assert_eq!(row.fetched, None);
        assert_eq!(words::forge_connection(&row).as_ref(), expected);
        assert_eq!(words::forge_connection_bare(&row).as_ref(), expected);
    }
}

#[test]
fn no_line_heard_says_the_step_alone() {
    for (wire, connection, expected) in cases() {
        let row = decoded_connection_row(wire);
        assert_eq!(row.fetched, None);
        assert_eq!(row.connection, connection);
        assert_eq!(words::forge_connection(&row).as_ref(), expected);
        assert_eq!(words::forge_connection_bare(&row).as_ref(), expected);
    }
}

#[test]
fn an_out_of_order_document_is_refused() {
    let mut forges = List::with_capacity(2);
    for number in [3, 1] {
        forges
            .push(
                w::ForgeRow::new(
                    &limits(),
                    w::ForgeRowParts {
                        starts_with: w::StartsWith::Repository,
                        makings: List::with_capacity(0),
                        host: b"git.example:3000".as_slice().into(),
                        number,
                        kind: w::Kind::Forgejo,
                        mode: w::Mode::Owned,
                        connection: w::Connection::NotSetUp,
                        offer: w::Offer::Offered,
                        first: None,
                        installations: List::with_capacity(0),
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
    }
    let record = Family {
        family: wire::FORGE,
        version: wire::CLIENT_VERSION,
        bytes: wire::forges(&w::Forges::new(&limits(), w::ForgesParts { forges, linked: false }).unwrap()),
    };
    assert!(rows(&record, &fixtures::limits().forge).is_none());
}

#[test]
fn setup_carries_each_field_without_semantic_form_checks() {
    let Ask::Family { record, .. } =
        ask(3, Box::default(), b"widget".as_slice().into(), false, &fixtures::limits().forge).unwrap()
    else {
        panic!("family")
    };
    let w::Request::SetUp(request) = wire::decode_requesting(&record.bytes, &limits()).unwrap().into_parts().request
    else {
        panic!("setup")
    };
    let w::Making::Kept(kept) = request.forge() else { panic!("kept forge") };
    assert_eq!(kept.forge(), 3);
    assert_eq!(empty_setup(request.first()).org(), b"");
    assert_eq!(empty_setup(request.first()).repository(), b"widget");
    assert!(!empty_setup(request.first()).playground());
}
#[test]
fn first_repository_decodes_each_state_and_creation_step() {
    use forge::forges::{CreateStep, First, FirstState};
    let mut states = List::with_capacity(6);
    states.push((w::FirstState::Making, FirstState::Making)).unwrap();
    states
        .push((
            w::FirstState::Made(w::Made::new(&limits(), w::MadeParts { id: 42 }).unwrap()),
            FirstState::Made { id: 42 },
        ))
        .unwrap();
    for (wire, domain) in [
        (w::CreateStep::Admin, CreateStep::Admin),
        (w::CreateStep::Org, CreateStep::Org),
        (w::CreateStep::Repository, CreateStep::Repository),
        (w::CreateStep::File, CreateStep::File),
    ] {
        states
            .push((
                w::FirstState::Failed(w::CreateFailed::new(&limits(), w::CreateFailedParts { step: wire }).unwrap()),
                FirstState::Failed { step: domain },
            ))
            .unwrap();
    }
    for (wire_state, state) in states.into_boxed() {
        let first = w::FirstRepository::new(
            &limits(),
            w::FirstRepositoryParts {
                playground: false,
                org: b"team".as_slice().into(),
                repository: b"widget".as_slice().into(),
                state: wire_state.clone(),
                label: Box::from(b"docker_1.x-y".as_slice()),
            },
        )
        .unwrap();
        let mut forges = List::with_capacity(1);
        forges
            .push(
                w::ForgeRow::new(
                    &limits(),
                    w::ForgeRowParts {
                        starts_with: w::StartsWith::Repository,
                        makings: List::with_capacity(0),
                        host: b"git.example:3000".as_slice().into(),
                        number: 3,
                        kind: w::Kind::Forgejo,
                        mode: w::Mode::Owned,
                        connection: w::Connection::Connecting,
                        offer: w::Offer::NotOffered,
                        first: Some(first),
                        installations: List::with_capacity(0),
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
        let record = Family {
            family: wire::FORGE,
            version: wire::CLIENT_VERSION,
            bytes: wire::forges(&w::Forges::new(&limits(), w::ForgesParts { forges, linked: false }).unwrap()),
        };
        assert_eq!(
            rows(&record, &fixtures::limits().forge).unwrap()[0].first,
            Some(First {
                playground: false,
                org: b"team".as_slice().into(),
                repository: b"widget".as_slice().into(),
                state,
                label: Box::from(b"docker_1.x-y".as_slice()),
            })
        );
    }
}

#[test]
fn playground_start_outcomes_fit_neither_adopt_nor_setup() {
    let link =
        jig_client_protocol_link::Limits::new(fixtures::limits().client.domain, fixtures::limits().client.exchanges)
            .unwrap();
    for outcome in [
        w::Outcome::Running,
        w::Outcome::StartRefused(
            w::StartRefused::new(&limits(), w::StartRefusedParts { why: w::StartRefusal::NotOffered }).unwrap(),
        ),
        w::Outcome::StartFailed(
            w::StartFailed::new(&limits(), w::StartFailedParts { why: w::StartFailure::Account }).unwrap(),
        ),
    ] {
        let record = answer(outcome);
        assert_eq!(family::set_up_answer(&record, &limits()), None);
        assert_eq!(family::answer(&record, &limits(), &link), None);
    }
}

#[test]
fn installation_accounts_flags_and_connect_ends_survive_client_decode() {
    use forge::forges::{Account, Installation};
    let limits = limits();
    let mut installations = List::with_capacity(2);
    for (id, account, suspended, stray) in [
        (
            11,
            Some(w::Account::new(&limits, w::AccountParts { id: 111, login: Box::from(b"acme".as_slice()) }).unwrap()),
            true,
            false,
        ),
        (22, None, false, true),
    ] {
        installations
            .push(w::Installation::new(&limits, w::InstallationParts { id, account, suspended, stray }).unwrap())
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
                    number: 1,
                    kind: w::Kind::Github,
                    mode: w::Mode::Connected,
                    connection: w::Connection::NotConnected,
                    offer: w::Offer::NotOffered,
                    first: None,
                    installations,
                    ended: Some(
                        w::ConnectStopped::new(
                            &limits,
                            w::ConnectStoppedParts { why: Box::from(b"the pasted value changed".as_slice()) },
                        )
                        .unwrap(),
                    ),
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
    let bytes = wire::forges(&w::Forges::new(&limits, w::ForgesParts { forges, linked: false }).unwrap());
    let rows = family::forges(&bytes, &limits).unwrap();
    assert_eq!(
        rows[0].installations.as_ref(),
        [
            Installation {
                id: 11,
                account: Some(Account { id: 111, login: Box::from(b"acme".as_slice()) }),
                suspended: true,
                stray: false
            },
            Installation { id: 22, account: None, suspended: false, stray: true },
        ]
    );
    assert_eq!(rows[0].ended.as_deref(), Some(b"the pasted value changed".as_slice()));
}

#[test]
fn admin_form_and_typed_stops_survive_client_decode() {
    use forge::forges::{AdminStop, SetupStage};
    let limits = limits();
    for (stop, expected) in [
        (None, None),
        (Some(w::AdminStop::NotAdmin), Some(AdminStop::NotAdmin)),
        (Some(w::AdminStop::Unmarked), Some(AdminStop::Unmarked)),
        (Some(w::AdminStop::Other), Some(AdminStop::Other)),
        (
            Some(w::AdminStop::Refused(
                w::AdminRefused::new(&limits, w::AdminRefusedParts { status: 403, stage: w::SetupStage::Repository })
                    .unwrap(),
            )),
            Some(AdminStop::Refused { status: 403, stage: SetupStage::Repository }),
        ),
    ] {
        for admin in [false, true] {
            let mut forges = List::with_capacity(1);
            forges
                .push(
                    w::ForgeRow::new(
                        &limits,
                        w::ForgeRowParts {
                            starts_with: w::StartsWith::Repository,
                            makings: List::with_capacity(0),
                            host: b"git.example:3000".as_slice().into(),
                            number: 1,
                            kind: w::Kind::Forgejo,
                            mode: w::Mode::Managed,
                            connection: w::Connection::NotConnected,
                            offer: w::Offer::NotOffered,
                            first: None,
                            installations: List::with_capacity(0),
                            ended: None,
                            app: None,
                            web: None,
                            repositories: List::with_capacity(0),
                            owned: None,
                            admin,
                            read_only: false,
                            ended_stop: stop.clone(),
                        },
                    )
                    .unwrap(),
                )
                .unwrap();
            let bytes = wire::forges(&w::Forges::new(&limits, w::ForgesParts { forges, linked: false }).unwrap());
            let rows = family::forges(&bytes, &limits).unwrap();
            assert_eq!(rows[0].admin, admin);
            assert_eq!(rows[0].ended_stop, expected);
        }
    }
}

fn empty_setup(first: &w::SetUpFirst) -> &w::FirstEmpty {
    let w::SetUpFirst::Empty(first) = first else { panic!("empty setup fixture") };
    first
}

#[test]
fn the_first_choice_survives_forge_row_decode() {
    let addresses: Box<[Box<[u8]>]> = Box::new([Box::from(b"https://source.example/owner/repo".as_slice())]);
    let mut copies = List::with_capacity(1);
    copies.push(addresses[0].clone()).unwrap();
    let copies = w::CopyAddresses::new(&limits(), w::CopyAddressesParts { addresses: copies }).unwrap();
    for (starts_with, expected) in [
        (w::StartsWith::Repository, forge::forges::StartsWith::Repository),
        (w::StartsWith::Nothing, forge::forges::StartsWith::Nothing),
        (w::StartsWith::Copies(copies), forge::forges::StartsWith::Copies { addresses }),
    ] {
        let mut parts = wire_connection_row(
            3,
            w::Kind::Forgejo,
            w::Mode::Owned,
            w::Connection::Connecting,
            w::Offer::Offered,
            None,
        )
        .into_parts();
        parts.starts_with = starts_with;
        let mut forges = List::with_capacity(1);
        forges.push(w::ForgeRow::new(&limits(), parts).unwrap()).unwrap();
        let record = w::Forges::new(&limits(), w::ForgesParts { forges, linked: false }).unwrap();
        let decoded = family::forges(&wire::forges(&record), &limits()).unwrap();
        assert_eq!(decoded[0].starts_with, expected);
        assert_eq!(decoded[0].first, None);
    }
}

#[test]
fn connection_steps_replace_settings_with_the_open_row_but_keep_welcome() {
    use crate::{Address, Choice, Event, Place, Request};
    for (kind, connection) in [
        (w::Kind::Forgejo, w::Connection::Connecting),
        (
            w::Kind::Forgejo,
            w::Connection::SettingUp(
                w::SettingUp::new(&limits(), w::SettingUpParts { step: Some(w::SetUpStep::Prepare), fetched: None })
                    .unwrap(),
            ),
        ),
        (
            w::Kind::Forgejo,
            w::Connection::Connected(
                w::Connected::new(&limits(), w::ConnectedParts { id: 7, login: b"temper".as_slice().into() }).unwrap(),
            ),
        ),
        (
            w::Kind::Github,
            w::Connection::Connected(
                w::Connected::new(&limits(), w::ConnectedParts { id: 7, login: b"temper".as_slice().into() }).unwrap(),
            ),
        ),
    ] {
        for welcome in [false, true] {
            let (mut root, _, watch) = super::connect::opened(true);
            let address = if welcome {
                Address::Welcome { step: crate::Step::Forge { forge: Some(Choice::Forge(3)) } }
            } else {
                Address::ConnectForge { forge: Some(Choice::Forge(3)) }
            };
            let mut out = skein_lib::Queue::uncapped();
            crate::step(
                &mut root,
                &fixtures::env(&fixtures::limits()),
                Event::Went { place: Place::Address(address.clone()) },
                &mut out,
            );
            let mut rows = List::with_capacity(1);
            rows.push(wire_connection_row(
                3,
                kind.clone(),
                w::Mode::Owned,
                connection.clone(),
                w::Offer::Offered,
                None,
            ))
            .unwrap();
            let record = w::Forges::new(&limits(), w::ForgesParts { forges: rows, linked: false }).unwrap();
            out.clear();
            crate::step(
                &mut root,
                &fixtures::env(&fixtures::limits()),
                Event::Client {
                    event: jig_client::Event::Up {
                        up: jig_client::Up::FamilyDelivered {
                            watch,
                            record: Family {
                                family: wire::FORGE,
                                version: wire::CLIENT_VERSION,
                                bytes: wire::forges(&record),
                            },
                        },
                    },
                },
                &mut out,
            );
            let mut moved = false;
            for request in &out {
                if let Request::Show { address: Address::Forges, replace } = request {
                    assert!(*replace);
                    moved = true;
                }
            }
            assert_eq!(moved, !welcome);
            if welcome {
                assert_eq!(root.pages().address(), &address);
                assert_eq!(root.pages().forges_opened(), None);
            } else {
                assert_eq!(root.pages().address(), &Address::Forges);
                assert_eq!(root.pages().forges_opened(), Some(3));
                crate::step(
                    &mut root,
                    &fixtures::env(&fixtures::limits()),
                    Event::Went { place: Place::Address(Address::Accounts) },
                    &mut out,
                );
                crate::step(
                    &mut root,
                    &fixtures::env(&fixtures::limits()),
                    Event::Went { place: Place::Address(Address::Forges) },
                    &mut out,
                );
                assert_eq!(root.pages().forges_opened(), None);
            }
        }
    }
}

pub(super) fn forges_document(modes: &[w::Mode], linked: bool) -> Family {
    let mut forges = List::with_capacity(u32::try_from(modes.len()).unwrap());
    for (number, mode) in modes.iter().enumerate() {
        forges
            .push(wire_connection_row(
                u16::try_from(number.checked_add(1).unwrap()).unwrap(),
                w::Kind::Forgejo,
                mode.clone(),
                w::Connection::NotConnected,
                w::Offer::NotOffered,
                None,
            ))
            .unwrap();
    }
    Family {
        family: wire::FORGE,
        version: wire::CLIENT_VERSION,
        bytes: wire::forges(&w::Forges::new(&limits(), w::ForgesParts { forges, linked }).unwrap()),
    }
}

pub(super) fn deliver_forges(root: &mut crate::Domain, watch: skein_lib::Token, record: Family) {
    crate::step(
        root,
        &fixtures::env(&fixtures::limits()),
        crate::Event::Client { event: jig_client::Event::Up { up: jig_client::Up::FamilyDelivered { watch, record } } },
        &mut skein_lib::Queue::uncapped(),
    );
}

#[test]
fn the_forges_document_says_whether_the_printed_link_signed_in() {
    for ending in [jig_client::Link::SignedOut, jig_client::Link::OutOfDate, jig_client::Link::Live] {
        let (mut root, _, watch) = super::connect::opened(true);
        assert_eq!(root.forge().linked(), None, "no document yet");
        for linked in [false, true] {
            let record = forges_document(&[w::Mode::Connected], linked);
            let (decoded, flag) = family::forges_document(&record.bytes, &limits()).unwrap();
            assert_eq!(decoded, family::forges(&record.bytes, &limits()).unwrap());
            assert_eq!(flag, linked);
            deliver_forges(&mut root, watch, record);
            assert_eq!(root.forge().linked(), Some(linked));
        }
        let mut out = skein_lib::Queue::uncapped();
        if ending == jig_client::Link::Live {
            crate::step(
                &mut root,
                &fixtures::env(&fixtures::limits()),
                crate::Event::Client {
                    event: jig_client::Event::Up {
                        up: jig_client::Up::WatchEnded { watch, end: c::WatchEnd::NotAllowed },
                    },
                },
                &mut out,
            );
        } else if ending == jig_client::Link::SignedOut {
            crate::step(&mut root, &fixtures::env(&fixtures::limits()), crate::Event::SignedOut, &mut out);
        } else {
            crate::route::told(
                &mut root,
                &fixtures::env(&fixtures::limits()),
                jig_client::Told::Link { link: ending },
                &mut out,
            );
        }
        assert_eq!(root.forge().linked(), None, "{ending:?}");
    }
}
