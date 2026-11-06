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
        starts_with: forges::StartsWith::Repository,
        makings: Box::new([]),
        host: b"git.example:3000".as_slice().into(),
        number,
        kind,
        mode,
        connection,
        fetched: None,
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
fn settings_defaults_to_the_first_available_unconnected_card() {
    use connect::Choice as Card;
    let connected = C::Connected { id: 1, login: Box::default() };
    let owned = row(HERE, Kind::Forgejo, Mode::Owned, connected.clone(), Offer::NotOffered);
    let github = row(2, Kind::GitHub, Mode::Connected, connected.clone(), Offer::NotOffered);
    let fake = row(4, Kind::Forgejo, Mode::Fake, connected.clone(), Offer::NotOffered);
    let managed = row(5, Kind::Forgejo, Mode::Managed, connected, Offer::NotOffered);
    for (rows, playground, offered, expected) in [
        (&[][..], None, true, Some(Card::Owned)),
        (&[][..], None, false, Some(Card::GitHub)),
        (core::slice::from_ref(&owned), None, true, Some(Card::GitHub)),
        (core::slice::from_ref(&github), None, true, Some(Card::Owned)),
        (core::slice::from_ref(&github), None, false, None),
        (&[owned.clone(), github.clone()][..], None, true, None),
        (&[fake, managed][..], None, true, Some(Card::Owned)),
        (&[][..], Some(HERE), true, None),
    ] {
        assert_eq!(connect::default_choice(rows, playground, offered), expected);
    }
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
        assert_eq!(connect::step(Some(Choice::Forge(HERE)), &rows(connection), None, true), expected);
    }
    assert_eq!(connect::step(None, &rows(C::NotSetUp), None, true), Shown::Choose);
    assert_eq!(connect::step(Some(Choice::Forge(9)), &rows(C::NotSetUp), None, true), Shown::Choose);
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
        assert_eq!(connect::step(choice, &State::Unknown, None, true), Shown::Details(Choice::Elsewhere));
    }
    let playground = forge::family::playground(&playground::answer(b"title"), &w::CEILINGS).unwrap();
    assert_eq!(
        connect::step(Some(Choice::Forge(playground.forge)), &rows(C::NotSetUp), Some(playground.forge), true),
        Shown::Choose
    );
    let github = State::Rows(Box::new([row(2, Kind::GitHub, Mode::Managed, C::NotConnected, Offer::NotOffered)]));
    assert_eq!(connect::step(Some(Choice::Forge(2)), &github, None, true), Shown::Details(Choice::Forge(2)));
    let fake = State::Rows(Box::new([row(HERE, Kind::Forgejo, Mode::Fake, C::NotSetUp, Offer::Offered)]));
    assert_eq!(connect::step(Some(Choice::Forge(HERE)), &fake, None, true), Shown::Choose);
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
#[test]
fn no_owned_offers_github_and_a_forgejo_elsewhere() {
    for state in [State::Unknown, State::Ended, State::Rows(Box::new([]))] {
        assert_eq!(
            connect::choices(connect::records(&state), None, false).as_ref(),
            &[connect::Choice::GitHub, connect::Choice::Managed]
        );
    }
}
#[test]
fn an_owned_address_with_none_offered_is_choose() {
    for (state, playground, owned) in [
        (State::Unknown, None, false),
        (State::Ended, None, false),
        (State::Rows(Box::new([])), None, false),
        (State::Rows(Box::new([])), Some(1), true),
        (State::Rows(Box::new([row(HERE, Kind::Forgejo, Mode::Owned, C::NotSetUp, Offer::NotOffered)])), None, true),
        (
            State::Rows(Box::new([row(HERE, Kind::Forgejo, Mode::Owned, C::NotSetUp, Offer::NotUnderTheBox)])),
            None,
            true,
        ),
    ] {
        assert_eq!(connect::step(Some(Choice::Owned), &state, playground, owned), Shown::Choose);
        assert_eq!(connect::welcome_step(Some(Choice::Owned), &state, playground, owned), Shown::Choose);
    }
}
#[test]
fn an_offered_owned_address_keeps_its_details() {
    for (state, owned) in [(State::Rows(Box::new([])), true), (rows(C::NotSetUp), false)] {
        assert_eq!(connect::step(Some(Choice::Owned), &state, None, owned), Shown::Details(Choice::Owned));
        assert_eq!(connect::welcome_step(Some(Choice::Owned), &state, None, owned), Shown::Details(Choice::Owned));
    }
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
        if let Request::Unlinked { request: client::UnlinkedRequest::Down { down: Down::Open { watch, subject } } } =
            request
            && subject == forges::watch(&f::limits().forge).unwrap()
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
        if let Request::Unlinked { request: client::UnlinkedRequest::Down { down: Down::Open { watch, subject } } } =
            request
        {
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
        if let Request::Unlinked { request: client::UnlinkedRequest::Down { down: Down::Send { .. } } } = request {
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
        if let Request::Unlinked {
            request:
                client::UnlinkedRequest::Down {
                    down: Down::Send { attempt, key, request: client::Ask::Family { record, .. }, .. },
                },
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
                starts_with: w::StartsWith::Repository,
                makings: List::with_capacity(0),
                host: b"git.example:3000".as_slice().into(),
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
                    defaults: List::with_capacity(0),
                    public: None,
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
        assert_eq!(
            (empty_setup(value.first()).org(), empty_setup(value.first()).repository()),
            (b"work".as_slice(), b"hello".as_slice())
        );
    }
}
#[test]
fn a_set_up_sends_the_fields_the_person_gave() {
    let (mut root, _, _) = opened(true);
    write_setup(&mut root, forges::SetupField::Organization, b"team");
    write_setup(&mut root, forges::SetupField::Repository, b"widget");
    write_setup(&mut root, forges::SetupField::Repository, b"app");
    let (_, _, w::Request::SetUp(value)) = sent(setup(&mut root)) else { panic!("setup") };
    assert_eq!(
        (empty_setup(value.first()).org(), empty_setup(value.first()).repository()),
        (b"team".as_slice(), b"app".as_slice())
    );
    let last = root.forge().set_up().unwrap();
    assert_eq!(
        (empty_setup(&last.first).org(), empty_setup(&last.first).repository()),
        (b"team".as_slice(), b"app".as_slice())
    );
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
        assert_eq!(connect::step(Some(Choice::Forge(HERE)), &rows, None, true), expected);
        assert_eq!(connect::personal(&rows, None).unwrap().number, HERE);
    }
    assert_eq!(
        connect::step(
            Some(Choice::Forge(HERE)),
            &with_first(C::Connecting, forges::FirstState::Made { id: 42 }),
            None,
            true
        ),
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
        assert_eq!(connect::step(Some(Choice::Forge(HERE)), &rows, None, true), Shown::SettingUp(HERE));
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
    event(&mut root, Event::Went { place: Place::Address(Address::Chats { filter: ChatsFilter::All }) });
    assert!(deliver_first(&mut root, watch, connected(), made()).is_empty());
    assert!(deliver_first(&mut root, watch, connected(), made()).is_empty());
}
#[test]
fn a_client_that_did_not_set_up_adopts_nothing() {
    let (mut root, _, watch) = opened(true);
    let (_, _, request) = sent(deliver_first(&mut root, watch, connected(), made()));
    let w::Request::Reachable(read) = request else { panic!("only the sheet reads; nothing adopts") };
    assert_eq!((read.forge(), read.page()), (HERE, 1));
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
    assert_eq!(
        (empty_setup(value.first()).org(), empty_setup(value.first()).repository()),
        (b"playground".as_slice(), b"widget".as_slice())
    );
    assert!(empty_setup(value.first()).playground());
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
                starts_with: w::StartsWith::Repository,
                makings: List::with_capacity(0),
                host: b"git.example:3000".as_slice().into(),
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
    assert_eq!(connect::step(Some(Choice::Forge(2)), &state, None, true), Shown::SettingUp(2));
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
            if let Request::Unlinked { request: client::UnlinkedRequest::Down { down: Down::Send { .. } } } = request {
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
        assert_eq!(connect::welcome_step(Some(Choice::Forge(HERE)), &state, None, true), expected);
        assert_eq!(connect::step(Some(Choice::Forge(HERE)), &state, None, true), Shown::Details(Choice::Forge(HERE)));
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
    assert_eq!(
        (
            empty_setup(value.first()).org(),
            empty_setup(value.first()).repository(),
            empty_setup(value.first()).playground()
        ),
        (b"work".as_slice(), b"hello".as_slice(), false)
    );
    let last = root.forge().set_up().unwrap();
    assert_eq!(
        (empty_setup(&last.first).org(), empty_setup(&last.first).repository(), empty_setup(&last.first).playground()),
        (b"work".as_slice(), b"hello".as_slice(), false)
    );
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
    assert_eq!(
        (empty_setup(request.first()).org(), empty_setup(request.first()).repository()),
        (b"work".as_slice(), b"hello".as_slice())
    );
}
#[test]
fn the_sheets_run_sets_up_owned_with_an_empty_repository_as_written() {
    let (mut root, _, _) = opened(true);
    assert_eq!(root.forge().setup_form().start, forges::Start::Empty);
    write_setup(&mut root, forges::SetupField::Organization, b"mine");
    write_setup(&mut root, forges::SetupField::Repository, b"first");
    let (_, _, request) = sent(f::form_act(&mut root, forge::Act::Forges(forges::Act::RunOwned)));
    let w::Request::SetUp(request) = request else { panic!("set up") };
    let w::Making::New(new) = request.forge() else { panic!("new") };
    assert_eq!(*new.forge(), w::ForgeAsk::Owned);
    assert_eq!(
        (
            empty_setup(request.first()).org(),
            empty_setup(request.first()).repository(),
            empty_setup(request.first()).playground()
        ),
        (b"mine".as_slice(), b"first".as_slice(), false)
    );
    assert!(root.forge().setup_form().copies_refused.is_none());
}
#[test]
fn the_sheets_copies_and_nothing_send_their_intent_and_move_on_after_acceptance() {
    for start in [forges::Start::Copies, forges::Start::Nothing] {
        let (mut root, _, _) = opened(true);
        event(
            &mut root,
            Event::Went {
                place: Place::Address(Address::Welcome { step: Step::Forge { forge: Some(Choice::Owned) } }),
            },
        );
        assert!(f::form_act(&mut root, forge::Act::Forges(forges::Act::Start(start))).is_empty());
        assert!(
            f::form_act(
                &mut root,
                forge::Act::CopiesWrite { text: b"http://source.invalid/team/one.git\n acme/two/".as_slice().into() }
            )
            .is_empty()
        );
        let (attempt, _, request) = sent(f::form_act(&mut root, forge::Act::Forges(forges::Act::RunOwned)));
        let w::Request::SetUp(request) = request else { panic!("set up") };
        let w::Making::New(new) = request.forge() else { panic!("new owned forge") };
        assert_eq!(*new.forge(), w::ForgeAsk::Owned);
        match start {
            forges::Start::Copies => {
                let w::SetUpFirst::Copies(copies) = request.first() else { panic!("copies") };
                assert_eq!(
                    copies.addresses().as_slice(),
                    [
                        Box::from(b"https://source.invalid/team/one".as_slice()),
                        Box::from(b"https://github.com/acme/two".as_slice())
                    ]
                );
            }
            forges::Start::Nothing => assert_eq!(request.first(), &w::SetUpFirst::Nothing),
            forges::Start::Empty => unreachable!(),
        }
        let mut out = reply(&mut root, attempt, w::Outcome::SettingUp);
        assert_eq!(
            out.pop(),
            Some(Request::Show { address: Address::Welcome { step: Step::Model { way: None } }, replace: false })
        );
    }
}

#[test]
fn the_sheet_refuses_blank_names_and_bad_copies_before_sending() {
    use forge::making::CopiesRefused as R;
    for field in [forges::SetupField::Organization, forges::SetupField::Repository] {
        let (mut root, _, _) = opened(true);
        write_setup(&mut root, field, b"   ");
        assert!(f::form_act(&mut root, forge::Act::Forges(forges::Act::RunOwned)).is_empty());
        assert!(root.forge().setup_form().blank);
        assert!(root.forge().set_up().is_none());
        write_setup(&mut root, field, b"filled");
        assert!(!root.forge().setup_form().blank);
    }
    let seventeen = skein_lib::bytes::concat(&[b"a/b\n".as_slice(); 17]);
    for (text, refused) in [
        (b" \n ".as_slice(), R::NoAddress),
        (b" bad ".as_slice(), R::NotAnAddress(b"bad".as_slice().into())),
        (seventeen.as_ref(), R::TooMany),
    ] {
        let (mut root, _, _) = opened(true);
        f::form_act(&mut root, forge::Act::Forges(forges::Act::Start(forges::Start::Copies)));
        f::form_act(&mut root, forge::Act::CopiesWrite { text: text.into() });
        assert!(f::form_act(&mut root, forge::Act::Forges(forges::Act::RunOwned)).is_empty());
        assert_eq!(root.forge().setup_form().copies_refused, Some(refused));
        assert!(root.forge().set_up().is_none());
        f::form_act(&mut root, forge::Act::CopiesWrite { text: b"a/b".as_slice().into() });
        assert!(root.forge().setup_form().copies_refused.is_none());
    }
}

#[test]
fn the_sheets_addresses_are_bounded_as_a_field() {
    let (mut root, _, _) = opened(true);
    let bound = f::limits().forge.field_bytes.checked_add(1).unwrap();
    let mut long = List::with_capacity(bound);
    for _ in 0..bound {
        long.push(b'a').unwrap();
    }
    let long = long.into_boxed();
    assert!(f::form_act(&mut root, forge::Act::CopiesWrite { text: long }).is_empty());
    let draft = root.forge().setup_form();
    assert_eq!((&*draft.copies, draft.copies_written), (b"".as_slice(), 0));
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
fn a_removed_start_over_leaves_for_its_end() {
    for (end, to) in [
        (forges::StartOverEnd::Welcome, b"/welcome?at=forge".as_slice()),
        (forges::StartOverEnd::Forges, b"/forges".as_slice()),
    ] {
        let (mut root, _, watch) = opened(true);
        deliver_first(&mut root, watch, w::Connection::NotSetUp, w::FirstState::Making);
        let row = connect::unfinished(connect::records(root.forges()), None).unwrap();
        assert_eq!(connect::unfinished_acts(row).as_slice(), &[connect::UnfinishedAct::StartOver]);
        let (attempt, _, request) =
            sent(f::form_act(&mut root, forge::Act::Forges(forges::Act::StartOver { forge: HERE, end })));
        let w::Request::RemoveForge(_) = request else { panic!("remove") };
        let out = reply(&mut root, attempt, w::Outcome::Removed);
        let mut leaves = List::with_capacity(1);
        for request in &out {
            if let Request::Leave { to, .. } = request {
                leaves.push(to.as_ref()).expect("one leave after removal");
            }
        }
        assert_eq!(leaves.as_slice(), &[to], "{end:?}");
    }
}

#[test]
fn new_owned_setup_binds_the_made_rows_number_without_adopting() {
    for row_first in [false, true] {
        let (mut root, _, watch) = opened(true);
        let (attempt, _, _) = sent(f::form_act(&mut root, forge::Act::Forges(forges::Act::SetUpOwned)));
        if row_first {
            only_picker_reads(&deliver_first(&mut root, watch, connected(), made()));
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
        only_picker_reads(&out);
    }
}
fn only_picker_reads(out: &Queue<Request>) {
    for request in out {
        if let Request::Unlinked { request: client::UnlinkedRequest::Down { down: Down::Send { request, .. } } } =
            request
        {
            let client::Ask::Family { scope: c::Scope::Deployment, record } = request else {
                panic!("connecting a forge sends no adoption or project request")
            };
            let w::Request::Reachable(read) =
                wire::decode_requesting(&record.bytes, &w::CEILINGS).unwrap().into_parts().request
            else {
                panic!("only the sheet reads; nothing adopts")
            };
            assert_eq!((read.forge(), read.page()), (HERE, 1));
        }
    }
}
#[test]
fn the_welcome_shows_a_forge_not_set_up_as_unfinished() {
    assert_eq!(connect::welcome_step(None, &rows(C::NotSetUp), None, true), Shown::Unfinished(HERE));
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
        assert_eq!(connect::step(Some(Choice::Forge(personal.number)), &state, None, true), Shown::Ready(HERE));
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
                    starts_with: w::StartsWith::Repository,
                    makings: List::with_capacity(0),
                    host: b"git.example:3000".as_slice().into(),
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

#[test]
fn forge_status_keeps_recovery_installation_and_admin_details_separate() {
    for (connection, kind, mode, admin, accepted, expected) in [
        (C::NotSetUp, Kind::Forgejo, Mode::Owned, false, false, false),
        (C::NotConnected, Kind::Forgejo, Mode::Connected, false, false, false),
        (C::Stopped { reason: Box::default() }, Kind::Forgejo, Mode::Owned, false, false, false),
        (C::SetUpFailed { why: Box::default(), at: None }, Kind::Forgejo, Mode::Owned, false, false, false),
        (C::SettingUp { step: None }, Kind::Forgejo, Mode::Owned, false, false, true),
        (C::Connecting, Kind::Forgejo, Mode::Owned, false, false, true),
        (C::Connected { id: 1, login: Box::default() }, Kind::Forgejo, Mode::Owned, false, false, true),
        (C::Connected { id: 1, login: Box::default() }, Kind::Forgejo, Mode::Fake, false, false, false),
        (C::Connecting, Kind::Forgejo, Mode::Managed, true, false, false),
        (C::Connected { id: 1, login: Box::default() }, Kind::Forgejo, Mode::Managed, true, false, true),
        (C::Connecting, Kind::GitHub, Mode::Connected, false, false, false),
        (C::Connected { id: 1, login: Box::default() }, Kind::GitHub, Mode::Connected, false, false, false),
        (C::Connected { id: 1, login: Box::default() }, Kind::GitHub, Mode::Connected, false, true, true),
    ] {
        let mut candidate = row(HERE, kind, mode, connection, Offer::NotOffered);
        candidate.admin = admin;
        if accepted {
            candidate.installations =
                Box::new([forges::Installation { id: 1, account: None, suspended: false, stray: false }]);
        }
        assert_eq!(connect::status_shown(&candidate, None), expected, "{candidate:?}");
        assert!(!connect::status_shown(&candidate, Some(HERE)));
    }
    for (state, shown) in [
        (forges::FirstState::Making, true),
        (forges::FirstState::Made { id: 42 }, true),
        (forges::FirstState::Failed { step: forges::CreateStep::Repository }, false),
    ] {
        let rows = with_first(C::Connected { id: 7, login: Box::default() }, state);
        let candidate = connect::personal(&rows, None).unwrap();
        assert_eq!(connect::status_shown(candidate, None), shown);
    }
    let mut admin =
        row(HERE, Kind::Forgejo, Mode::Managed, C::Connected { id: 1, login: Box::default() }, Offer::NotOffered);
    admin.admin = true;
    admin.ended_stop = Some(forges::AdminStop::NotAdmin);
    assert!(!connect::status_shown(&admin, None));
}

fn empty_setup(first: &w::SetUpFirst) -> &w::FirstEmpty {
    let w::SetUpFirst::Empty(first) = first else { panic!("empty setup fixture") };
    first
}

#[test]
fn set_up_again_keeps_copies_or_nothing_from_the_row_and_last_ask() {
    let mut copies = List::with_capacity(1);
    copies.push(Box::from(b"https://source.example/owner/repo".as_slice())).unwrap();
    let copies = w::CopyAddresses::new(&w::CEILINGS, w::CopyAddressesParts { addresses: copies }).unwrap();
    for (starts_with, expected) in [
        (w::StartsWith::Nothing, w::SetUpFirst::Nothing),
        (w::StartsWith::Copies(copies.clone()), w::SetUpFirst::Copies(copies)),
    ] {
        let (mut root, _, watch) = opened(true);
        let mut parts = super::forges::wire_connection_row(
            HERE,
            w::Kind::Forgejo,
            w::Mode::Owned,
            w::Connection::SetUpFailed(
                w::SetUpFailed::new(&w::CEILINGS, w::SetUpFailedParts { why: b"retry".as_slice().into(), at: None })
                    .unwrap(),
            ),
            w::Offer::Offered,
            None,
        )
        .into_parts();
        parts.starts_with = starts_with;
        let mut forges = List::with_capacity(1);
        forges.push(w::ForgeRow::new(&w::CEILINGS, parts).unwrap()).unwrap();
        let record = client::Family {
            family: wire::FORGE,
            version: wire::CLIENT_VERSION,
            bytes: wire::forges(&w::Forges::new(&w::CEILINGS, w::ForgesParts { forges }).unwrap()),
        };
        up(&mut root, Up::FamilyDelivered { watch, record });
        for _attempt in 0_u32..2 {
            let (attempt, _, w::Request::SetUp(request)) = sent(setup(&mut root)) else { panic!("setup retry") };
            assert_eq!(request.first(), &expected);
            assert_eq!(root.forge().set_up().unwrap().first, expected);
            reply(&mut root, attempt, w::Outcome::SettingUp);
        }
    }
}
