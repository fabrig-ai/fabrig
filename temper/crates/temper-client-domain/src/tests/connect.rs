use super::{fixtures as f, playground};
use crate::ChatsFilter;
use crate::{
    Address, Choice, Event, Place, Request, Step,
    forge::{
        self,
        connect::{self, Shown},
        forges::{self, Connection as C, Kind, Mode, Offer, Row, State},
    },
};
use jig_client::{self as client, Down, Told, Up, contract as c};
use skein_lib::{List, Queue, Time, Token};
use temper_client_protocol_wire::{self as wire, v1 as w};
const HERE: u16 = 3;
fn row(number: u16, kind: Kind, mode: Mode, connection: C, offer: Offer) -> Row {
    Row {
        number,
        kind,
        mode,
        connection,
        offer,
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
fn rows(connection: C) -> State {
    State::Rows(Box::new([row(HERE, Kind::Forgejo, Mode::Owned, connection, Offer::Offered)]))
}
#[test]
fn the_flow_shows_the_step_its_row_is_at() {
    for (connection, expected) in [
        (C::NotSetUp, Shown::Details(Choice::Forge(HERE))),
        (C::SettingUp { step: None }, Shown::SettingUp(HERE)),
        (C::SettingUp { step: Some(forges::Step::Prepare) }, Shown::SettingUp(HERE)),
        (C::SetUpFailed { why: b"fetch failed".as_slice().into(), at: None }, Shown::SettingUp(HERE)),
        (C::Connecting, Shown::SettingUp(HERE)),
        (C::Connected { id: 1, login: b"bot".as_slice().into() }, Shown::Ready(HERE)),
        (C::Stopped { reason: b"stopped".as_slice().into() }, Shown::Choose),
    ] {
        assert_eq!(connect::step(Some(Choice::Forge(HERE)), &rows(connection), None), expected);
    }
    assert_eq!(connect::step(None, &rows(C::NotSetUp), None), Shown::Choose);
    assert_eq!(connect::step(Some(Choice::Forge(9)), &rows(C::NotSetUp), None), Shown::Choose);
    // Leaving the elsewhere card off the welcome does not retire its address.
    for address in [
        Address::Welcome { step: Step::Forge { forge: Some(Choice::Elsewhere) } },
        Address::ConnectForge { forge: Some(Choice::Elsewhere) },
    ] {
        let Place::Address(parsed) = crate::address::parse(&crate::address::print(&address)) else {
            panic!("forge address");
        };
        assert_eq!(parsed, address);
        let (Address::Welcome { step: Step::Forge { forge: choice } } | Address::ConnectForge { forge: choice }) =
            parsed
        else {
            panic!("forge choice");
        };
        assert_eq!(connect::step(choice, &State::Unknown, None), Shown::Details(Choice::Elsewhere));
    }
    let playground = forge::family::playground(&playground::answer(b"title"), &w::CEILINGS).unwrap();
    assert_eq!(
        connect::step(Some(Choice::Forge(playground.forge)), &rows(C::NotSetUp), Some(playground.forge)),
        Shown::Choose
    );
    let github = State::Rows(Box::new([row(2, Kind::GitHub, Mode::Managed, C::NotConnected, Offer::NotOffered)]));
    assert_eq!(connect::step(Some(Choice::Forge(2)), &github, None), Shown::Details(Choice::Forge(2)));
    let fake = State::Rows(Box::new([row(HERE, Kind::Forgejo, Mode::Fake, C::NotSetUp, Offer::Offered)]));
    assert_eq!(connect::step(Some(Choice::Forge(HERE)), &fake, None), Shown::Choose);
}
#[test]
fn a_fresh_deployment_lists_no_forge_and_offers_the_kinds() {
    let state = State::Rows(Box::new([]));
    assert!(forge::listed::rows(&state, None).is_empty());
    assert_eq!(
        connect::choices(connect::records(&state), None, true).as_ref(),
        &[connect::Choice::GitHub, connect::Choice::Owned, connect::Choice::Managed]
    );
    assert_eq!(
        connect::choices(connect::records(&state), None, false).as_ref(),
        &[connect::Choice::GitHub, connect::Choice::Managed]
    );
}
fn event(root: &mut crate::Domain, event: Event) -> Queue<Request> {
    event_at(root, event, Time::ZERO)
}
fn event_at(root: &mut crate::Domain, event: Event, now: Time) -> Queue<Request> {
    let mut out = Queue::uncapped();
    let mut env = f::env(&f::limits());
    env.now = now;
    crate::step(root, &env, event, &mut out);
    out
}
fn up(root: &mut crate::Domain, up: Up) -> Queue<Request> {
    event(root, Event::Client { event: client::Event::Up { up } })
}
fn watch(mut out: Queue<Request>) -> Option<Token> {
    while let Some(request) = out.pop() {
        if let Request::Client {
            request: client::Request::Down { down: Down::Open { watch, subject: client::Watchable::Family { .. } } },
        } = request
        {
            return Some(watch);
        }
    }
    None
}
pub(super) fn opened(owner: bool) -> (crate::Domain, Token, Token) {
    let mut root = crate::unlinked(&f::limits(), f::SEED);
    let (me, _) = f::start_accounts(&mut root, List::with_capacity(0));
    let c::WatchEvent::Delivered(c::Delivery::Me(mut frame)) = f::frame(f::PARTY) else { panic!("me") };
    frame.deployment_role = Some(if owner { c::HeldRole::Owner } else { c::HeldRole::Member });
    let out = up(&mut root, Up::Watched { watch: me, event: c::WatchEvent::Delivered(c::Delivery::Me(frame)) });
    (root, me, watch(out).expect("session forges watch"))
}
#[test]
fn the_forges_watch_opens_once_me_is_delivered_for_every_person() {
    for owner in [false, true] {
        let (mut root, me, _) = opened(owner);
        assert!(watch(up(&mut root, Up::Watched { watch: me, event: f::frame(f::PARTY) })).is_none());
        assert!(
            watch(event(&mut root, Event::Went { place: Place::Address(Address::ConnectForge { forge: None }) }))
                .is_none()
        );
    }
}
#[test]
fn a_refused_forges_watch_is_kept_until_the_next_sign_in() {
    let (mut root, me, token) = opened(false);
    up(&mut root, Up::WatchEnded { watch: token, end: c::WatchEnd::NotAllowed });
    let State::Ended = root.forges() else { panic!("refused watch ended") };
    assert!(watch(up(&mut root, Up::Watched { watch: me, event: f::frame(f::PARTY) })).is_none());
    event(&mut root, Event::SignedOut);
    let mut out = event(&mut root, Event::Client { event: client::Event::SignIn { with: client::With::Transport } });
    let mut served = up(&mut root, Up::Served { served: true, versions: None });
    while let Some(request) = served.pop() {
        out.push(request);
    }
    let mut me = None;
    let mut reopened = None;
    while let Some(request) = out.pop() {
        if let Request::Client { request: client::Request::Down { down: Down::Open { watch, subject } } } = request {
            match subject {
                client::Watchable::Jig(c::Subject::Me) => me = Some(watch),
                client::Watchable::Family { .. } => reopened = Some(watch),
                client::Watchable::Jig(_) => {}
            }
        }
    }
    let delivered = watch(up(&mut root, Up::Watched { watch: me.unwrap(), event: f::frame(f::PARTY) }));
    assert!(reopened.or(delivered).is_some());
}
#[test]
fn signing_out_unwatches_the_forges() {
    let (mut root, _, token) = opened(true);
    up(&mut root, Up::Watched { watch: token, event: c::WatchEvent::Opened });
    event(&mut root, Event::SignedOut);
    assert!(root.client().document(forges::handle()).is_none());
    let State::Unknown = root.forges() else { panic!("forges cleared") };
}
fn setup(root: &mut crate::Domain) -> Queue<Request> {
    f::form_act(root, forge::Act::SetUp { forge: HERE })
}
fn answered(root: &mut crate::Domain, outcome: w::Outcome) -> Queue<Request> {
    let key =
        client::Key { asker: crate::owner::Owner::Forge.asker(), number: root.forge().set_up().unwrap().request.raw() };
    let limits = forge::wire_limits(&f::limits().forge).unwrap();
    let mut out = Queue::uncapped();
    crate::route::told(
        root,
        &f::env(&f::limits()),
        Told::Answered {
            key,
            outcome: client::Outcome::Answered(client::Answered::Family(client::Family {
                family: wire::FORGE,
                version: wire::CLIENT_VERSION,
                bytes: wire::answer(&w::Answer::new(&limits, w::AnswerParts { outcome }).unwrap()),
            })),
        },
        &mut out,
    );
    out
}
#[test]
fn set_up_sends_one_at_a_time() {
    let mut root = f::on_adoption_page();
    let mut out = setup(&mut root);
    let mut sends = 0_u32;
    while let Some(request) = out.pop() {
        if let Request::Client { request: client::Request::Down { down: Down::Send { .. } } } = request {
            sends += 1;
        }
    }
    assert_eq!(sends, 1);
    assert!(setup(&mut root).is_empty());
    assert!(root.forge().form().unwrap().pending.is_none());
}
#[test]
fn a_refused_set_up_keeps_its_refusal() {
    let mut root = f::on_adoption_page();
    setup(&mut root);
    let limits = forge::wire_limits(&f::limits().forge).unwrap();
    answered(
        &mut root,
        w::Outcome::SetUpRefused(
            w::SetUpRefused::new(&limits, w::SetUpRefusedParts { why: w::SetUpRefusal::Busy }).unwrap(),
        ),
    );
    assert_eq!(
        root.forge().set_up().unwrap().answer,
        Some(forges::SetUpAnswer::Outcome(forges::SetUpOutcome::Refused(forges::SetUpRefusal::Busy)))
    );
    assert!(root.forge().set_up().unwrap().progress.is_none());
    assert!(root.forge().form().unwrap().outcome.is_none());
}
#[test]
fn an_accepted_set_up_moves_the_welcome_on_to_model() {
    let mut root = f::on_adoption_page();
    event(
        &mut root,
        Event::Went {
            place: Place::Address(Address::Welcome { step: Step::Forge { forge: Some(Choice::Forge(HERE)) } }),
        },
    );
    setup(&mut root);
    let mut out = answered(&mut root, w::Outcome::SettingUp);
    assert_eq!(
        out.pop(),
        Some(Request::Show { address: Address::Welcome { step: Step::Model { way: None } }, replace: false })
    );
    assert!(out.is_empty());
}

#[test]
fn a_set_up_survives_page_changes_and_parks_out_of_date() {
    let mut root = f::on_adoption_page();
    setup(&mut root);
    let request = root.forge().set_up().unwrap().request;
    event(&mut root, Event::Went { place: Place::Address(Address::ConnectForge { forge: None }) });
    event(&mut root, Event::Forge(forge::Event::Served { served: forge::Served::OutOfDate }));
    assert_eq!(root.forge().set_up().unwrap().request, request);
    assert_eq!(root.forge().set_up().unwrap().progress, Some(skein_lib::RequestProgress::Parked));
    assert!(setup(&mut root).is_empty());
    answered(&mut root, w::Outcome::SettingUp);
    assert!(root.forge().set_up().unwrap().progress.is_none());
    assert_eq!(
        root.forge().set_up().unwrap().answer,
        Some(forges::SetUpAnswer::Outcome(forges::SetUpOutcome::SettingUp))
    );
}

#[test]
fn an_unreadable_set_up_answer_is_unknown_and_allows_another_request() {
    let mut root = f::on_adoption_page();
    setup(&mut root);
    let request = root.forge().set_up().unwrap().request;
    let key = client::Key { asker: crate::owner::Owner::Forge.asker(), number: request.raw() };
    crate::route::told(
        &mut root,
        &f::env(&f::limits()),
        Told::Answered {
            key,
            outcome: client::Outcome::Answered(client::Answered::Family(client::Family {
                family: wire::FORGE,
                version: wire::CLIENT_VERSION,
                bytes: Box::new([255]),
            })),
        },
        &mut Queue::uncapped(),
    );
    assert_eq!(
        root.forge().set_up().unwrap().answer,
        Some(forges::SetUpAnswer::Outcome(forges::SetUpOutcome::Unknown))
    );
    assert!(root.forge().set_up().unwrap().progress.is_none());
    assert!(!setup(&mut root).is_empty());
    assert_ne!(root.forge().set_up().unwrap().request, request);
}

#[test]
fn an_out_of_date_link_unwatches_the_forges() {
    let (mut root, _, token) = opened(true);
    up(&mut root, Up::Watched { watch: token, event: c::WatchEvent::Opened });
    crate::route::told(
        &mut root,
        &f::env(&f::limits()),
        Told::Link { link: client::Link::OutOfDate },
        &mut Queue::uncapped(),
    );
    assert!(root.client().document(forges::handle()).is_none());
    let State::Unknown = root.forges() else { panic!("forges cleared") };
}

#[test]
fn the_playground_forge_is_listed_with_nothing_to_add() {
    let state = State::Rows(Box::new([row(
        1,
        Kind::Forgejo,
        Mode::Fake,
        C::Connected { id: 1, login: Box::default() },
        Offer::NotOffered,
    )]));
    assert_eq!(forge::listed::rows(&state, Some(1)).len(), 1);
    assert_eq!(connect::name(forge::listed::rows(&state, Some(1)).get(0).unwrap(), Some(1)), connect::Name::Playground);
    assert!(connect::choices(connect::records(&state), Some(1), true).is_empty());
    assert!(connect::unfinished(connect::records(&state), Some(1)).is_none());
}
#[test]
fn all_kept_real_records_are_listed() {
    for connection in [C::NotConnected, C::NotSetUp, C::SettingUp { step: None }, C::Stopped { reason: Box::default() }]
    {
        assert_eq!(forge::listed::rows(&rows(connection), None).len(), 1);
    }
}
#[test]
fn the_dot_is_red_for_a_forge_that_needs_the_person_else_amber_while_one_sets_up() {
    use forge::listed::{self, Dot};
    for (connection, expected) in [
        (C::NotSetUp, Dot::None),
        (C::Connected { id: 42, login: Box::default() }, Dot::None),
        (C::Connecting, Dot::Amber),
        (C::SettingUp { step: None }, Dot::Amber),
        (C::SetUpFailed { why: Box::default(), at: None }, Dot::Red),
        (C::Stopped { reason: Box::default() }, Dot::Red),
    ] {
        assert_eq!(listed::dot(&rows(connection), None), expected);
    }
    for connections in
        [[C::Connecting, C::Stopped { reason: Box::default() }], [C::Stopped { reason: Box::default() }, C::Connecting]]
    {
        let state = State::Rows(Box::new([
            row(2, Kind::GitHub, Mode::Managed, connections[0].clone(), Offer::NotOffered),
            row(HERE, Kind::Forgejo, Mode::Owned, connections[1].clone(), Offer::Offered),
        ]));
        assert_eq!(listed::dot(&state, None), Dot::Red);
        assert_eq!(
            connect::underway_forge(&state, None).unwrap().number,
            if connections[0] == C::Connecting { 2 } else { HERE }
        );
    }
    let playground = forge::family::playground(&playground::answer(b"title"), &w::CEILINGS).unwrap();
    let state = State::Rows(Box::new([
        row(playground.forge, Kind::GitHub, Mode::Managed, C::Connecting, Offer::NotOffered),
        row(HERE, Kind::Forgejo, Mode::Fake, C::Stopped { reason: Box::default() }, Offer::NotOffered),
    ]));
    assert_eq!(listed::dot(&state, Some(playground.forge)), Dot::Amber);
    assert!(connect::underway_forge(&state, Some(playground.forge)).is_none());
}

fn sent(mut out: Queue<Request>) -> (Token, skein_lib::RequestKey, w::Request) {
    let mut found = None;
    while let Some(request) = out.pop() {
        if let Request::Client {
            request:
                client::Request::Down { down: Down::Send { attempt, key, request: client::Ask::Family { record, .. }, .. } },
        } = request
        {
            assert!(found.is_none(), "one family request");
            found = Some((
                attempt,
                key,
                wire::decode_requesting(&record.bytes, &w::CEILINGS).unwrap().into_parts().request,
            ));
        }
    }
    found.expect("sent family request")
}
fn reply(root: &mut crate::Domain, attempt: Token, outcome: w::Outcome) -> Queue<Request> {
    reply_at(root, attempt, outcome, Time::ZERO)
}
fn reply_at(root: &mut crate::Domain, attempt: Token, outcome: w::Outcome, now: Time) -> Queue<Request> {
    event_at(
        root,
        Event::Client {
            event: client::Event::Up {
                up: Up::Answer {
                    attempt,
                    envelope: skein_lib::RequestEnvelope::Final,
                    answer: Some(client::Answered::Family(client::Family {
                        family: wire::FORGE,
                        version: wire::CLIENT_VERSION,
                        bytes: wire::answer(&w::Answer::new(&w::CEILINGS, w::AnswerParts { outcome }).unwrap()),
                    })),
                    why: None,
                },
            },
        },
        now,
    )
}
fn first(state: forges::FirstState) -> forges::First {
    forges::First {
        playground: false,
        org: b"work".as_slice().into(),
        repository: b"hello".as_slice().into(),
        state,
        label: Box::from(b"temper-host".as_slice()),
    }
}
fn with_first(connection: C, state: forges::FirstState) -> State {
    let mut row = row(HERE, Kind::Forgejo, Mode::Owned, connection, Offer::Offered);
    row.first = Some(first(state));
    State::Rows(Box::new([row]))
}
fn deliver_first(
    root: &mut crate::Domain,
    watch: Token,
    connection: w::Connection,
    state: w::FirstState,
) -> Queue<Request> {
    let first = w::FirstRepository::new(
        &w::CEILINGS,
        w::FirstRepositoryParts {
            playground: false,
            org: b"work".as_slice().into(),
            repository: b"hello".as_slice().into(),
            state,
            label: Box::from(b"temper-host".as_slice()),
        },
    )
    .unwrap();
    deliver_first_record(root, watch, connection, first)
}
fn deliver_first_record(
    root: &mut crate::Domain,
    watch: Token,
    connection: w::Connection,
    first: w::FirstRepository,
) -> Queue<Request> {
    let mut rows = List::with_capacity(1);
    rows.push(
        w::ForgeRow::new(
            &w::CEILINGS,
            w::ForgeRowParts {
                number: HERE,
                kind: w::Kind::Forgejo,
                mode: w::Mode::Owned,
                connection,
                offer: w::Offer::Offered,
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
    up(
        root,
        Up::FamilyDelivered {
            watch,
            record: client::Family {
                family: wire::FORGE,
                version: wire::CLIENT_VERSION,
                bytes: wire::forges(&w::Forges::new(&w::CEILINGS, w::ForgesParts { forges: rows }).unwrap()),
            },
        },
    )
}
fn made() -> w::FirstState {
    w::FirstState::Made(w::Made::new(&w::CEILINGS, w::MadeParts { id: 42 }).unwrap())
}
fn connected() -> w::Connection {
    w::Connection::Connected(
        w::Connected::new(&w::CEILINGS, w::ConnectedParts { id: 7, login: b"temper".as_slice().into() }).unwrap(),
    )
}
fn write_setup(root: &mut crate::Domain, field: forges::SetupField, text: &[u8]) {
    assert!(f::form_act(root, forge::Act::SetupWrite { field, text: text.into() }).is_empty());
}
#[test]
fn a_set_up_carries_the_repository_name_regardless_of_the_current_project() {
    for project in [None, Some(7_u32)] {
        let (mut root, me, _) = opened(true);
        if let Some(project) = project {
            let c::WatchEvent::Delivered(c::Delivery::Me(mut frame)) = f::frame(f::PARTY) else { panic!("me") };
            frame.projects = List::with_capacity(1);
            frame
                .projects
                .push(c::MeProject {
                    project,
                    name: jig_client::contract::DocumentText::complete(
                        jig_client::contract::Text::new(b"Project 1".as_slice().into(), 64).expect("project name"),
                    ),
                    role: c::HeldRole::Owner,
                    pool: None,
                    templates: List::with_capacity(0),
                    creating: None,
                    models: List::with_capacity(0),
                })
                .unwrap();
            up(&mut root, Up::Watched { watch: me, event: c::WatchEvent::Delivered(c::Delivery::Me(frame)) });
        }
        let (_, _, w::Request::SetUp(value)) = sent(setup(&mut root)) else { panic!("setup") };
        assert_eq!((value.org(), value.repository()), (b"work".as_slice(), b"hello".as_slice()));
    }
}
#[test]
fn a_set_up_sends_the_fields_the_person_gave() {
    let (mut root, _, _) = opened(true);
    write_setup(&mut root, forges::SetupField::Organization, b"team");
    write_setup(&mut root, forges::SetupField::Repository, b"widget");
    write_setup(&mut root, forges::SetupField::Repository, b"app");
    let (_, _, w::Request::SetUp(value)) = sent(setup(&mut root)) else { panic!("setup") };
    assert_eq!((value.org(), value.repository()), (b"team".as_slice(), b"app".as_slice()));
    let last = root.forge().set_up().unwrap();
    assert_eq!((&*last.org, &*last.repository), (b"team".as_slice(), b"app".as_slice()));
}
#[test]
fn an_empty_field_is_required_and_sends_nothing() {
    for field in [forges::SetupField::Organization, forges::SetupField::Repository] {
        let (mut root, _, _) = opened(true);
        write_setup(&mut root, field, b"");
        assert!(setup(&mut root).is_empty());
        assert!(root.forge().setup_form().required(field));
        assert!(root.forge().set_up().is_none());
    }
}
#[test]
fn set_up_again_sends_the_same_fields() {
    let (mut root, _, watch) = opened(true);
    write_setup(&mut root, forges::SetupField::Organization, b"team");
    let (attempt, _, w::Request::SetUp(original)) = sent(setup(&mut root)) else { panic!("setup") };
    reply(&mut root, attempt, w::Outcome::SettingUp);
    deliver_first(
        &mut root,
        watch,
        w::Connection::Connecting,
        w::FirstState::Failed(
            w::CreateFailed::new(&w::CEILINGS, w::CreateFailedParts { step: w::CreateStep::File }).unwrap(),
        ),
    );
    write_setup(&mut root, forges::SetupField::Organization, b"changed");
    let (_, _, w::Request::SetUp(retry)) = sent(setup(&mut root)) else { panic!("setup") };
    assert_eq!(original, retry);
}
#[test]
fn ready_waits_on_the_first_repository_made() {
    for (state, expected) in [
        (forges::FirstState::Making, Shown::SettingUp(HERE)),
        (forges::FirstState::Made { id: 42 }, Shown::Ready(HERE)),
    ] {
        let rows = with_first(C::Connected { id: 7, login: b"temper".as_slice().into() }, state);
        assert_eq!(connect::step(Some(Choice::Forge(HERE)), &rows, None), expected);
        assert_eq!(connect::personal(&rows, None).unwrap().number, HERE);
    }
    assert_eq!(
        connect::step(Some(Choice::Forge(HERE)), &with_first(C::Connecting, forges::FirstState::Made { id: 42 }), None),
        Shown::SettingUp(HERE)
    );
}
#[test]
fn a_first_repository_failed_shows_its_step() {
    for (step, words) in [
        (forges::CreateStep::Admin, b"Set up failed: checking the admin's token".as_slice()),
        (forges::CreateStep::Org, b"Set up failed: making the organization"),
        (forges::CreateStep::Repository, b"Set up failed: making the repository"),
        (forges::CreateStep::File, b"Set up failed: adding the starter workflow"),
    ] {
        let rows = with_first(C::Connected { id: 7, login: Box::default() }, forges::FirstState::Failed { step });
        assert_eq!(connect::step(Some(Choice::Forge(HERE)), &rows, None), Shown::SettingUp(HERE));
        let row = connect::personal(&rows, None).unwrap();
        assert_eq!(forge::words::forge_connection(row).as_ref(), words);
        assert_eq!(forge::listed::dot(&rows, None), forge::listed::Dot::Red);
    }
    let rows = with_first(C::Connected { id: 7, login: Box::default() }, forges::FirstState::Making);
    assert_eq!(forge::listed::dot(&rows, None), forge::listed::Dot::Amber);
}
#[test]
fn the_client_that_set_up_leaves_the_made_repository_unadopted() {
    let (mut root, _, watch) = opened(true);
    let (attempt, _, _) = sent(setup(&mut root));
    reply(&mut root, attempt, w::Outcome::SettingUp);
    event(&mut root, Event::Went { place: Place::Address(Address::Chats { filter: ChatsFilter::All, start: false }) });
    assert!(deliver_first(&mut root, watch, connected(), made()).is_empty());
    assert!(deliver_first(&mut root, watch, connected(), made()).is_empty());
}
#[test]
fn a_client_that_did_not_set_up_adopts_nothing() {
    let (mut root, _, watch) = opened(true);
    assert!(deliver_first(&mut root, watch, connected(), made()).is_empty());
    assert!(root.forge().set_up().is_none());
}
#[test]
fn invalid_refusals_say_their_words() {
    let (mut root, _, _) = opened(true);
    let (attempt, _, _) = sent(setup(&mut root));
    reply(
        &mut root,
        attempt,
        w::Outcome::SetUpRefused(
            w::SetUpRefused::new(&w::CEILINGS, w::SetUpRefusedParts { why: w::SetUpRefusal::Invalid }).unwrap(),
        ),
    );
    let Some(forges::SetUpAnswer::Outcome(forges::SetUpOutcome::Refused(why))) = root.forge().set_up().unwrap().answer
    else {
        panic!("refusal")
    };
    assert_eq!(
        forge::words::set_up_refusal(why),
        b"invalid: the organization, repository or project name is empty or too long"
    );
}

#[test]
fn set_up_again_on_another_client_uses_the_rows_fields() {
    let (mut root, _, watch) = opened(true);
    let value = w::FirstRepository::new(
        &w::CEILINGS,
        w::FirstRepositoryParts {
            playground: true,
            org: b"playground".as_slice().into(),
            repository: b"widget".as_slice().into(),
            state: w::FirstState::Failed(
                w::CreateFailed::new(&w::CEILINGS, w::CreateFailedParts { step: w::CreateStep::File }).unwrap(),
            ),
            label: Box::from(b"temper-host".as_slice()),
        },
    )
    .unwrap();
    deliver_first_record(&mut root, watch, w::Connection::Connecting, value);
    let (_, _, w::Request::SetUp(value)) = sent(setup(&mut root)) else { panic!("setup") };
    assert_eq!((value.org(), value.repository()), (b"playground".as_slice(), b"widget".as_slice()));
    assert!(value.playground());
}
#[test]
fn signing_out_forgets_the_setup_intent() {
    let (mut root, _, _) = opened(true);
    setup(&mut root);
    event(&mut root, Event::SignedOut);
    assert!(root.forge().set_up().is_none());
}

fn github_picker_rows() -> State {
    let mut github = row(
        2,
        Kind::GitHub,
        Mode::Connected,
        C::Connected { id: 9, login: b"app[bot]".as_slice().into() },
        Offer::NotOffered,
    );
    github.installations = Box::new([1, 2].map(|id| forges::Installation {
        id,
        account: Some(forges::Account { id, login: b"acme".as_slice().into() }),
        suspended: false,
        stray: false,
    }));
    State::Rows(Box::new([github]))
}
fn deliver_picker(root: &mut crate::Domain, watch: Token, changed: bool) -> Queue<Request> {
    let mut installations = List::with_capacity(2);
    for id in [1, 2] {
        let login: &[u8] = if changed && id == 2 { b"changed" } else { b"acme" };
        installations
            .push(
                w::Installation::new(
                    &w::CEILINGS,
                    w::InstallationParts {
                        id,
                        account: Some(
                            w::Account::new(&w::CEILINGS, w::AccountParts { id, login: login.into() }).unwrap(),
                        ),
                        suspended: false,
                        stray: false,
                    },
                )
                .unwrap(),
            )
            .unwrap();
    }
    let mut rows = List::with_capacity(1);
    rows.push(
        w::ForgeRow::new(
            &w::CEILINGS,
            w::ForgeRowParts {
                number: 2,
                kind: w::Kind::Github,
                mode: w::Mode::Connected,
                connection: connected(),
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
    up(
        root,
        Up::FamilyDelivered {
            watch,
            record: client::Family {
                family: wire::FORGE,
                version: wire::CLIENT_VERSION,
                bytes: wire::forges(&w::Forges::new(&w::CEILINGS, w::ForgesParts { forges: rows }).unwrap()),
            },
        },
    )
}
#[test]
fn github_without_an_accepted_installation_is_connecting() {
    let mut state = github_picker_rows();
    let State::Rows(rows) = &mut state else { panic!("rows") };
    rows[0].installations = Box::default();
    assert_eq!(connect::step(Some(Choice::Forge(2)), &state, None), Shown::SettingUp(2));
    assert_eq!(connect::unfinished(connect::records(&state), None).unwrap().number, 2);
}
#[test]
fn a_connected_github_advances_only_the_named_welcome_forge() {
    for welcome in [true, false] {
        let (mut root, _, watch) = opened(true);
        let address = if welcome {
            Address::Welcome { step: Step::Forge { forge: Some(Choice::Forge(2)) } }
        } else {
            Address::ConnectForge { forge: Some(Choice::Forge(2)) }
        };
        event(&mut root, Event::Went { place: Place::Address(address) });
        let out = deliver_picker(&mut root, watch, false);
        let mut moves = 0;
        for request in &out {
            if let Request::Show { address, replace } = request {
                assert_eq!(address, &Address::Welcome { step: Step::Model { way: None } });
                assert!(*replace);
                moves += 1;
            }
            if let Request::Client { request: client::Request::Down { down: Down::Send { .. } } } = request {
                panic!("connecting a forge sends no adoption or project request");
            }
        }
        assert_eq!(moves, u32::from(welcome));
    }
}
#[test]
fn welcome_skips_owned_details_only_when_offered_under_the_box() {
    for (offer, expected) in
        [(Offer::Offered, Shown::Choose), (Offer::NotUnderTheBox, Shown::Details(Choice::Forge(HERE)))]
    {
        let state = State::Rows(Box::new([row(HERE, Kind::Forgejo, Mode::Owned, C::NotSetUp, offer)]));
        assert_eq!(connect::welcome_step(Some(Choice::Forge(HERE)), &state, None), expected);
        assert_eq!(connect::step(Some(Choice::Forge(HERE)), &state, None), Shown::Details(Choice::Forge(HERE)));
    }
    let mut candidate = row(HERE, Kind::Forgejo, Mode::Owned, C::NotSetUp, Offer::Offered);
    assert!(connect::at_once(&candidate));
    candidate.admin = true;
    assert!(!connect::at_once(&candidate));
    candidate.admin = false;
    candidate.kind = Kind::GitHub;
    assert!(!connect::at_once(&candidate));
    candidate.kind = Kind::Forgejo;
    candidate.mode = Mode::Managed;
    assert!(!connect::at_once(&candidate));
    candidate.mode = Mode::Owned;
    candidate.connection = C::SettingUp { step: None };
    assert!(!connect::at_once(&candidate));
    candidate.connection = C::NotSetUp;
    candidate.offer = Offer::NotOffered;
    assert!(!connect::at_once(&candidate));
}
#[test]
fn at_once_sends_fixed_names_without_reading_or_marking_the_form() {
    let (mut root, _, _) = opened(true);
    write_setup(&mut root, forges::SetupField::Organization, b"another");
    write_setup(&mut root, forges::SetupField::Repository, b"repository");
    let before = root.forge().setup_form();
    let before = (before.fields.clone(), before.written, before.checked);
    let (_, _, w::Request::SetUp(value)) = sent(f::form_act(&mut root, forge::Act::SetUpAtOnce { forge: HERE })) else {
        panic!("setup")
    };
    let w::Making::Kept(kept) = value.forge() else { panic!("kept forge") };
    assert_eq!(kept.forge(), HERE);
    assert_eq!((value.org(), value.repository(), value.playground()), (b"work".as_slice(), b"hello".as_slice(), false));
    let last = root.forge().set_up().unwrap();
    assert_eq!((&*last.org, &*last.repository, last.playground), (b"work".as_slice(), b"hello".as_slice(), false));
    let after = root.forge().setup_form();
    assert_eq!((after.fields.clone(), after.written, after.checked), before);
    assert!(f::form_act(&mut root, forge::Act::SetUpAtOnce { forge: HERE }).is_empty());
}
#[test]
fn at_once_sends_nothing_without_a_party_or_when_out_of_date() {
    let mut root = crate::unlinked(&f::limits(), f::SEED);
    assert!(f::form_act(&mut root, forge::Act::SetUpAtOnce { forge: HERE }).is_empty());
    assert!(root.forge().set_up().is_none());
    let (mut root, _, _) = opened(true);
    event(&mut root, Event::Forge(forge::Event::Served { served: forge::Served::OutOfDate }));
    assert!(f::form_act(&mut root, forge::Act::SetUpAtOnce { forge: HERE }).is_empty());
    assert!(root.forge().set_up().is_none());
    assert!(!root.forge().setup_form().checked);
}
#[test]
fn accepting_at_once_on_either_welcome_forge_address_moves_to_model() {
    for choice in [None, Some(Choice::Forge(HERE))] {
        let (mut root, _, _) = opened(true);
        event(
            &mut root,
            Event::Went { place: Place::Address(Address::Welcome { step: Step::Forge { forge: choice } }) },
        );
        let (attempt, _, _) = sent(f::form_act(&mut root, forge::Act::SetUpAtOnce { forge: HERE }));
        let out = reply(&mut root, attempt, w::Outcome::SettingUp);
        let mut moved = false;
        for request in &out {
            if let Request::Show { address: Address::Welcome { step: Step::Model { way: None } }, replace: false } =
                request
            {
                moved = true;
            }
        }
        assert!(moved, "accepted setup opens Model");
    }
}

#[test]
fn choosing_github_sends_connect_new_github() {
    let (mut root, _, _) = opened(true);
    let (_, _, request) = sent(f::form_act(
        &mut root,
        forge::Act::Forges(forges::Act::GitHub { back: b"https://temper.example/forges".as_slice().into() }),
    ));
    let w::Request::Connect(request) = request else { panic!("connect") };
    let w::Making::New(new) = request.forge() else { panic!("new") };
    assert_eq!(*new.forge(), w::ForgeAsk::Github);
    let w::How::GithubManifest(_) = request.how() else { panic!("manifest") };
}
#[test]
fn choosing_owned_sends_set_up_new_owned() {
    let (mut root, _, _) = opened(true);
    let (_, _, request) = sent(f::form_act(&mut root, forge::Act::Forges(forges::Act::SetUpOwned)));
    let w::Request::SetUp(request) = request else { panic!("set up") };
    let w::Making::New(new) = request.forge() else { panic!("new") };
    assert_eq!(*new.forge(), w::ForgeAsk::Owned);
    assert_eq!((request.org(), request.repository()), (b"work".as_slice(), b"hello".as_slice()));
}
#[test]
fn an_unfinished_github_with_its_app_offers_go_on_first() {
    let mut candidate = row(4, Kind::GitHub, Mode::Connected, C::Stopped { reason: Box::default() }, Offer::NotOffered);
    candidate.app = Some(forges::AppWay::Manifest);
    let state = State::Rows(Box::new([candidate]));
    let row = connect::unfinished(connect::records(&state), None).unwrap();
    assert_eq!(
        connect::unfinished_acts(row).as_slice(),
        &[connect::UnfinishedAct::GoOn, connect::UnfinishedAct::StartOver]
    );
}
#[test]
fn remove_asks_then_sends_remove_forge() {
    let (mut root, _, watch) = opened(true);
    deliver_first(&mut root, watch, w::Connection::NotSetUp, w::FirstState::Making);
    assert!(f::form_act(&mut root, forge::Act::Forges(forges::Act::Remove { forge: HERE })).is_empty());
    let confirmation = root.forge().setup_form().removal.as_ref().unwrap();
    assert_eq!(
        (confirmation.kind, confirmation.mode, confirmation.effect),
        (Kind::Forgejo, Mode::Owned, forges::RemovalEffect::OwnedDirectory)
    );
    let (_, _, request) = sent(f::form_act(&mut root, forge::Act::Forges(forges::Act::ConfirmRemove)));
    let w::Request::RemoveForge(request) = request else { panic!("remove") };
    assert_eq!(request.forge(), HERE);
    assert!(f::form_act(&mut root, forge::Act::Forges(forges::Act::ConfirmRemove)).is_empty());
}
#[test]
fn an_unfinished_forgejo_offers_start_over_which_removes_then_chooses() {
    let (mut root, _, watch) = opened(true);
    deliver_first(&mut root, watch, w::Connection::NotSetUp, w::FirstState::Making);
    let row = connect::unfinished(connect::records(root.forges()), None).unwrap();
    assert_eq!(connect::unfinished_acts(row).as_slice(), &[connect::UnfinishedAct::StartOver]);
    let (attempt, _, request) =
        sent(f::form_act(&mut root, forge::Act::Forges(forges::Act::StartOver { forge: HERE })));
    let w::Request::RemoveForge(_) = request else { panic!("remove") };
    let out = reply(&mut root, attempt, w::Outcome::Removed);
    let mut chose = false;
    for request in &out {
        if let Request::Leave { to, .. } = request
            && to.as_ref() == b"/welcome?at=forge"
        {
            chose = true;
        }
    }
    assert!(chose);
}

#[test]
fn new_owned_setup_binds_the_made_rows_number_without_adopting() {
    for row_first in [false, true] {
        let (mut root, _, watch) = opened(true);
        let (attempt, _, _) = sent(f::form_act(&mut root, forge::Act::Forges(forges::Act::SetUpOwned)));
        if row_first {
            assert!(deliver_first(&mut root, watch, connected(), made()).is_empty());
        } else {
            reply(&mut root, attempt, w::Outcome::SettingUp);
        }
        let out = if row_first {
            reply(&mut root, attempt, w::Outcome::SettingUp)
        } else {
            deliver_first(&mut root, watch, connected(), made())
        };
        assert_eq!(root.forge().set_up().unwrap().forge, Some(HERE));
        assert!(root.forge().setup_form().making.is_none());
        for request in &out {
            if let Request::Client { request: client::Request::Down { down: Down::Send { .. } } } = request {
                panic!("connecting a forge sends no adoption or project request");
            }
        }
    }
}
#[test]
fn the_welcome_shows_a_forge_not_set_up_as_unfinished() {
    assert_eq!(connect::welcome_step(None, &rows(C::NotSetUp), None), Shown::Unfinished(HERE));
}
#[test]
fn a_connected_forge_without_its_first_repository_continues_instead_of_starting_over() {
    for (kind, mode) in [(Kind::GitHub, Mode::Connected), (Kind::Forgejo, Mode::Owned)] {
        let mut candidate = row(HERE, kind, mode, C::Connected { id: 1, login: Box::default() }, Offer::NotOffered);
        if kind == Kind::GitHub {
            candidate.installations =
                Box::new([forges::Installation { id: 1, account: None, suspended: false, stray: false }]);
        }
        let state = State::Rows(Box::new([candidate]));
        assert!(connect::unfinished(connect::records(&state), None).is_none());
        let personal = connect::personal(&state, None).unwrap();
        assert!(!connect::complete(personal), "the first repository is still needed");
        assert_eq!(connect::step(Some(Choice::Forge(personal.number)), &state, None), Shown::Ready(HERE));
    }
}

#[test]
fn an_adopted_github_without_an_accepted_installation_does_not_offer_start_over() {
    let mut candidate =
        row(HERE, Kind::GitHub, Mode::Connected, C::Connected { id: 1, login: Box::default() }, Offer::NotOffered);
    candidate.repositories =
        Box::new([forges::Adoption { owner: b"acme".as_slice().into(), name: b"app".as_slice().into(), project: 1 }]);
    assert!(connect::complete(&candidate));
    assert!(connect::unfinished(&[candidate], None).is_none());
}

#[test]
fn owned_choices_follow_the_existing_rows_offer() {
    for (offer, owned) in [(Offer::Offered, true), (Offer::NotOffered, false), (Offer::NotUnderTheBox, false)] {
        let state = State::Rows(Box::new([row(HERE, Kind::Forgejo, Mode::Owned, C::NotSetUp, offer)]));
        assert_eq!(
            connect::choices(connect::records(&state), None, !owned).as_ref().contains(&connect::Choice::Owned),
            owned
        );
    }
}
#[test]
fn go_on_uses_a_kept_github_and_its_recorded_app_way() {
    for app in [w::AppWay::Manifest, w::AppWay::Pasted] {
        let (mut root, _, watch) = opened(true);
        let mut rows = List::with_capacity(1);
        rows.push(
            w::ForgeRow::new(
                &w::CEILINGS,
                w::ForgeRowParts {
                    number: 9,
                    kind: w::Kind::Github,
                    mode: w::Mode::Connected,
                    connection: w::Connection::Stopped(
                        w::Stopped::new(&w::CEILINGS, w::StoppedParts { reason: Box::default() }).unwrap(),
                    ),
                    offer: w::Offer::NotOffered,
                    first: None,
                    installations: List::with_capacity(0),
                    ended: None,
                    app: Some(app.clone()),
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
        up(
            &mut root,
            Up::FamilyDelivered {
                watch,
                record: client::Family {
                    family: wire::FORGE,
                    version: wire::CLIENT_VERSION,
                    bytes: wire::forges(&w::Forges::new(&w::CEILINGS, w::ForgesParts { forges: rows }).unwrap()),
                },
            },
        );
        let (_, _, request) = sent(f::form_act(
            &mut root,
            forge::Act::Forges(forges::Act::GoOn {
                forge: 9,
                back: b"https://temper.example/forges".as_slice().into(),
            }),
        ));
        match app {
            w::AppWay::Manifest => {
                let w::Request::Connect(connect) = request else { panic!("connect") };
                let w::Making::Kept(kept) = connect.forge() else { panic!("kept") };
                assert_eq!(kept.forge(), 9);
                let w::How::GithubManifest(_) = connect.how() else { panic!("manifest") };
            }
            w::AppWay::Pasted => {
                let w::Request::Install(install) = request else { panic!("install") };
                assert_eq!(install.forge(), 9);
            }
        }
    }
}

#[test]
fn a_new_github_does_not_rebind_a_retained_owned_setup() {
    let (mut root, _, watch) = opened(true);
    setup(&mut root);
    answered(&mut root, w::Outcome::SettingUp);
    let (attempt, _, _) = sent(f::form_act(
        &mut root,
        forge::Act::Forges(forges::Act::GitHub { back: b"https://temper.example/forges".as_slice().into() }),
    ));
    deliver_picker(&mut root, watch, false);
    assert!(!root.forge().setup_form().making.as_ref().unwrap().accepted);
    reply(&mut root, attempt, w::Outcome::Connecting);
    assert!(root.forge().setup_form().making.is_none());
    assert_eq!(root.forge().set_up().unwrap().forge, Some(HERE));
}

#[test]
fn a_pending_new_github_blocks_a_second_new_forge() {
    let (mut root, _, _) = opened(true);
    sent(f::form_act(
        &mut root,
        forge::Act::Forges(forges::Act::GitHub { back: b"https://temper.example/forges".as_slice().into() }),
    ));
    assert!(f::form_act(&mut root, forge::Act::Forges(forges::Act::SetUpOwned)).is_empty());
    assert!(root.forge().set_up().is_none());
    assert_eq!(root.forge().setup_form().making.as_ref().unwrap().kind, Kind::GitHub);
}

#[test]
fn pasted_tokens_round_trip_each_given_item_in_code_order() {
    let mut rng = skein_lib::Rng::new(f::SEED);
    let api = rng.next_u64().to_be_bytes();
    let git = rng.next_u64().to_be_bytes();
    let secret = rng.next_u64().to_be_bytes();
    for (api, git, secret, codes) in [
        (Some(api.as_slice()), Some(git.as_slice()), None, &[2_u16, 3][..]),
        (Some(api.as_slice()), Some(git.as_slice()), Some(secret.as_slice()), &[2, 3, 5]),
        (None, Some(git.as_slice()), None, &[3][..]),
        (Some(b"".as_slice()), Some(git.as_slice()), Some(b"".as_slice()), &[3][..]),
    ] {
        let bundle = forges::pasted_tokens(api, git, secret).unwrap();
        let decoded = temper_protocol_records::decode_pasted(bundle.expose()).unwrap();
        assert_eq!(decoded.len(), codes.len());
        for ((code, value), expected) in decoded.iter().zip(codes) {
            assert_eq!(code, expected);
            let expected = match code {
                2 => api.unwrap(),
                3 => git.unwrap(),
                5 => secret.unwrap(),
                _ => panic!("tokens item"),
            };
            assert!(value.expose() == expected, "item bytes preserved");
        }
    }
    assert!(forges::pasted_tokens(None, None, None).is_none());
    assert!(forges::pasted_tokens(Some(b""), Some(b""), Some(b"")).is_none());
}
#[test]
fn pasted_tokens_respect_the_aggregate_bundle_bound() {
    let mut rng = skein_lib::Rng::new(f::SEED);
    let mut value = skein_lib::bytes::zeroed(forges::PASTED_PAYLOAD);
    for byte in &mut value {
        *byte = rng.next_u64().to_le_bytes()[0];
    }
    let bundle = forges::pasted_tokens(None, Some(&value), None).unwrap();
    let decoded = temper_protocol_records::decode_pasted(bundle.expose()).unwrap();
    assert_eq!(decoded.len(), 1);
    assert_eq!(decoded[0].0, 3);
    assert!(decoded[0].1.expose() == value.as_ref(), "item bytes preserved");
    assert!(forges::pasted_tokens(Some(&value), Some(&value), None).is_none());
}
#[test]
fn connection_refusals_offer_replace_or_connect_in_words() {
    assert_eq!(
        forges::connect_refusal(&w::ForgeRefusal::Connected),
        (b"connected".as_slice(), b"connected already: forge replace replaces its key or tokens".as_slice())
    );
    assert_eq!(
        forges::connect_refusal(&w::ForgeRefusal::NotConnected),
        (b"not_connected".as_slice(), b"not connected: connect it first".as_slice())
    );
}
