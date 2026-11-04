//! Home's pure reading and owned watches (temper/docs/design/client/README.md, 3.5).
use super::welcome::{default_answer, deliver, event, press_default, ready_default_with_forges};
use crate::tests::fixtures as f;
use crate::{
    Action, Address, Domain, Event, Place, Request, forge,
    pages::{
        self, Act,
        home::{self, Facts, Repository},
        welcome::First,
    },
};
use jig_client::{Down, Readout, Told, Up, Watchable, contract as c};
use skein_lib::{List, Queue, Token};
use temper_client_protocol_wire::{self as wire, v1 as w};
const OTHER_PROJECT: u32 = 91;
const FORGE: u16 = 3;
const OTHER_FORGE: u16 = 5;
const CHAT: u64 = 9;
const NAME_BYTES: u32 = 64;
const EMPTY: u32 = 0;
const ONE: u32 = 1;
// The ordered repository fixture holds three adoptions on its lower forge.
const REPOSITORIES: u32 = 3;
// Home has three independently completed steps (client/README.md, 3.5).
const THREE_STEPS: u8 = 3;
const TWO_STEPS: u8 = 2;
const ONE_STEP: u8 = 1;
const NO_STEPS: u8 = 0;

fn go(root: &mut Domain, project: u32) -> Queue<Request> {
    event(root, Event::Went { place: Place::Address(Address::Home { project }) })
}
fn up(root: &mut Domain, up: Up) -> Queue<Request> {
    event(root, Event::Client { event: jig_client::Event::Up { up } })
}
fn opened(out: &Queue<Request>, subject: &Watchable) -> Option<Token> {
    for request in out {
        if let Request::Unlinked {
            request: jig_client::UnlinkedRequest::Down { down: Down::Open { watch, subject: found } },
        } = request
            && found == subject
        {
            return Some(*watch);
        }
    }
    None
}
struct Fixture {
    root: Domain,
    settings: Token,
    live: Token,
    closed: Token,
    forges: Token,
}
fn fixture() -> Fixture {
    let mut root = crate::unlinked(&f::limits(), f::SEED);
    let me = f::start(&mut root, List::with_capacity(EMPTY));
    let out = deliver(&mut root, me, c::Delivery::Me(Box::new(f::chats::me())));
    let forges = opened(&out, &forge::forges::watch(&f::limits().forge).unwrap()).unwrap();
    let out = go(&mut root, f::PROJECT);
    let settings = opened(&out, &Watchable::Jig(c::Subject::Settings)).unwrap();
    let mut live = None;
    let mut closed = None;
    for request in &out {
        if let Request::Unlinked { request: jig_client::UnlinkedRequest::Down { down } } = request {
            if let Down::Open { watch, subject: Watchable::Jig(c::Subject::Tasks { .. }) } = down {
                live = Some(*watch);
            }
            if let Down::Read { read, what: jig_client::Readable::Jig(c::Read::Tasks { .. }) } = down {
                closed = Some(*read);
            }
        }
    }
    Fixture { root, settings, live: live.unwrap(), closed: closed.unwrap(), forges }
}
fn settings(fixture: &mut Fixture, follows: c::SettingFollows) {
    let mut lines = List::with_capacity(ONE);
    lines
        .push(c::SettingLine { setting: c::Setting::Rules, name: None, follows, last: None, application: None })
        .unwrap();
    deliver(
        &mut fixture.root,
        fixture.settings,
        c::Delivery::Settings(Box::new(c::SettingsDocument {
            newest: 0,
            settings: lines,
            held: List::with_capacity(EMPTY),
            pending: List::with_capacity(EMPTY),
        })),
    );
}
fn row(number: u16) -> w::ForgeRowParts {
    w::ForgeRowParts {
        host: b"git.example:3000".as_slice().into(),
        number,
        kind: w::Kind::Forgejo,
        mode: w::Mode::Owned,
        connection: w::Connection::Connected(
            w::Connected::new(&w::CEILINGS, w::ConnectedParts { id: 1, login: b"temper".as_slice().into() }).unwrap(),
        ),
        offer: w::Offer::Offered,
        first: None,
        installations: List::with_capacity(EMPTY),
        ended: None,
        app: None,
        web: None,
        repositories: List::with_capacity(EMPTY),
        owned: None,
        admin: false,
        read_only: false,
        ended_stop: None,
        makings: List::with_capacity(EMPTY),
    }
}
fn first() -> First {
    First { forge: FORGE, org: b"work".as_slice().into(), name: b"hello".as_slice().into() }
}
fn with_first(mut row: w::ForgeRowParts) -> w::ForgeRowParts {
    row.first = Some(
        w::FirstRepository::new(
            &w::CEILINGS,
            w::FirstRepositoryParts {
                playground: false,
                org: b"work".as_slice().into(),
                repository: b"hello".as_slice().into(),
                state: w::FirstState::Made(w::Made::new(&w::CEILINGS, w::MadeParts { id: 1 }).unwrap()),
                label: b"main".as_slice().into(),
            },
        )
        .unwrap(),
    );
    row
}
fn adopted(mut row: w::ForgeRowParts, project: u32, name: &[u8]) -> w::ForgeRowParts {
    row.repositories = List::with_capacity(REPOSITORIES);
    row.repositories
        .push(
            w::Adoption::new(
                &w::CEILINGS,
                w::AdoptionParts { owner: b"work".as_slice().into(), name: name.into(), project },
            )
            .unwrap(),
        )
        .unwrap();
    row
}
fn rows(root: &mut Domain, watch: Token, parts: Box<[w::ForgeRowParts]>) {
    let mut forges = List::with_capacity(wire::FORGES);
    for row in parts {
        forges.push(w::ForgeRow::new(&w::CEILINGS, row).unwrap()).unwrap();
    }
    up(
        root,
        Up::FamilyDelivered {
            watch,
            record: jig_client::Family {
                family: wire::FORGE,
                version: wire::CLIENT_VERSION,
                bytes: wire::forges(&w::Forges::new(&w::CEILINGS, w::ForgesParts { forges }).unwrap()),
            },
        },
    );
}
fn adopting() -> (Domain, Token) {
    let (mut root, _, forges) = ready_default_with_forges();
    press_default(&mut root);
    default_answer(
        &mut root,
        jig_client::Outcome::Answered(jig_client::Answered::Jig(c::Answer::Done(c::Outcome::Document(
            c::DocumentOutcome::ProjectMade { project: f::PROJECT },
        )))),
    );
    go(&mut root, f::PROJECT);
    (root, forges)
}
fn adoption_answer(root: &mut Domain, outcome: jig_client::Outcome) {
    let key = root.pages().adopting().unwrap().out.unwrap();
    crate::route::told(root, &f::env(&f::limits()), Told::Answered { key, outcome }, &mut Queue::uncapped());
}
fn refusal() -> jig_client::Outcome {
    jig_client::Outcome::Answered(jig_client::Answered::Family(jig_client::Family {
        family: wire::FORGE,
        version: wire::CLIENT_VERSION,
        bytes: wire::answer(
            &w::Answer::new(
                &w::CEILINGS,
                w::AnswerParts {
                    outcome: w::Outcome::ForgeRefused(
                        w::ForgeRefused::new(&w::CEILINGS, w::ForgeRefusedParts { reason: w::ForgeReason::HomeHeld })
                            .unwrap(),
                    ),
                },
            )
            .unwrap(),
        ),
    }))
}
#[test]
fn the_three_steps_count_only_what_is_left() {
    for (configured, repository, chat_asked, left) in [
        (false, Repository::None, false, THREE_STEPS),
        (true, Repository::None, false, TWO_STEPS),
        (true, Repository::In { owner: b"work".as_slice().into(), name: b"hello".as_slice().into() }, false, ONE_STEP),
        (true, Repository::In { owner: b"work".as_slice().into(), name: b"hello".as_slice().into() }, true, NO_STEPS),
        (true, Repository::Unknown, true, ONE_STEP),
        (true, Repository::Adopting(first()), true, ONE_STEP),
        (true, Repository::Offer { first: first(), refused: None }, true, ONE_STEP),
        (true, Repository::GitHub { account: None }, true, ONE_STEP),
    ] {
        assert_eq!(Facts { configured, repository, chat_asked }.left(), left);
    }
}
#[test]
fn settings_are_configured_only_after_a_change_is_delivered() {
    let mut f = fixture();
    assert!(!home::facts(&f.root, f::PROJECT).configured, "undelivered");
    settings(&mut f, c::SettingFollows::Default);
    assert!(!home::facts(&f.root, f::PROJECT).configured, "defaults");
    let before = pages::shown(f.root.pages());
    settings(&mut f, c::SettingFollows::Change);
    assert!(home::facts(&f.root, f::PROJECT).configured);
    assert!(pages::shown(f.root.pages()) > before, "a settings change redraws Home");
}
#[test]
fn refused_and_ended_settings_are_not_configured() {
    for end in [c::WatchEnd::NotAllowed, c::WatchEnd::SubjectEnded] {
        let mut f = fixture();
        settings(&mut f, c::SettingFollows::Change);
        up(&mut f.root, Up::WatchEnded { watch: f.settings, end });
        assert!(!home::facts(&f.root, f::PROJECT).configured, "{end:?}");
    }
}
#[test]
fn home_owns_and_releases_its_settings_and_chat_reads() {
    let mut f = fixture();
    up(&mut f.root, Up::Watched { watch: f.settings, event: c::WatchEvent::Opened });
    let settings = f.root.pages().home().unwrap().settings;
    let chats = f.root.pages().chats().unwrap();
    let handles = [settings, chats.live, chats.closed];
    assert_eq!(chats.filter, crate::ChatsFilter::All);
    let repeat = go(&mut f.root, f::PROJECT);
    assert!(opened(&repeat, &Watchable::Jig(c::Subject::Settings)).is_none());
    let out = event(&mut f.root, Event::Went { place: Place::Address(Address::Settings { project: f::PROJECT }) });
    for handle in handles {
        assert!(f.root.client().document(handle).is_none(), "released {handle:?}");
    }
    assert!(f.root.pages().home().is_none());
    assert!(f.root.pages().chats().is_none());
    let mut closed = false;
    for request in &out {
        if let Request::Unlinked { request: jig_client::UnlinkedRequest::Down { down: Down::Close { watch } } } =
            request
        {
            closed |= *watch == f.settings;
        }
    }
    assert!(closed, "Home closes its settings watch");
}
#[test]
fn unknown_and_ended_forges_never_invent_a_repository() {
    let mut f = fixture();
    assert_eq!(home::facts(&f.root, f::PROJECT).repository, Repository::Unknown);
    up(&mut f.root, Up::WatchEnded { watch: f.forges, end: c::WatchEnd::NotAllowed });
    assert_eq!(home::facts(&f.root, f::PROJECT).repository, Repository::None);
}
#[test]
fn the_first_project_repository_uses_forge_then_repository_order() {
    let mut f = fixture();
    let mut lower = adopted(row(FORGE), OTHER_PROJECT, b"another-project");
    lower
        .repositories
        .push(
            w::Adoption::new(
                &w::CEILINGS,
                w::AdoptionParts {
                    owner: b"work".as_slice().into(),
                    name: b"first".as_slice().into(),
                    project: f::PROJECT,
                },
            )
            .unwrap(),
        )
        .unwrap();
    // Two repositories on the lower forge precede the higher forge's row.
    lower
        .repositories
        .push(
            w::Adoption::new(
                &w::CEILINGS,
                w::AdoptionParts {
                    owner: b"work".as_slice().into(),
                    name: b"second".as_slice().into(),
                    project: f::PROJECT,
                },
            )
            .unwrap(),
        )
        .unwrap();
    rows(&mut f.root, f.forges, Box::new([lower, adopted(row(OTHER_FORGE), f::PROJECT, b"higher")]));
    assert_eq!(
        home::facts(&f.root, f::PROJECT).repository,
        Repository::In { owner: b"work".as_slice().into(), name: b"first".as_slice().into() }
    );
    assert_eq!(
        forge::listed::held(f.root.forges(), None, f::PROJECT),
        forge::listed::Held::Several { owner: b"work", name: b"first", more: 2 }
    );
}
#[test]
fn a_first_repository_is_offered_until_it_is_adopted_anywhere() {
    let mut f = fixture();
    rows(&mut f.root, f.forges, Box::new([with_first(row(FORGE))]));
    assert_eq!(home::facts(&f.root, f::PROJECT).repository, Repository::Offer { first: first(), refused: None });
    rows(&mut f.root, f.forges, Box::new([adopted(with_first(row(FORGE)), OTHER_PROJECT, b"hello")]));
    assert_eq!(home::facts(&f.root, f::PROJECT).repository, Repository::None);
}
#[test]
fn adoption_out_precedes_an_offer_and_refusal_is_kept() {
    let (mut root, forges) = adopting();
    assert_eq!(home::facts(&root, f::PROJECT).repository, Repository::Adopting(first()));
    adoption_answer(&mut root, refusal());
    assert_eq!(
        home::facts(&root, f::PROJECT).repository,
        Repository::Offer { first: first(), refused: Some(forge::Answer::ForgeRefused(forge::Reason::HomeHeld)) }
    );
    assert_eq!(home::facts(&root, OTHER_PROJECT).repository, Repository::Offer { first: first(), refused: None });
    rows(&mut root, forges, Box::new([adopted(with_first(row(FORGE)), OTHER_PROJECT, b"hello")]));
    assert_eq!(home::facts(&root, f::PROJECT).repository, Repository::None, "no stale offer after adoption elsewhere");
}
#[test]
fn adopted_rows_win_in_either_order_of_rows_and_answers() {
    for (answer_first, answer) in [
        (false, refusal()),
        (true, refusal()),
        (false, jig_client::Outcome::Answered(jig_client::Answered::Family(f::adopted_bytes()))),
        (true, jig_client::Outcome::Answered(jig_client::Answered::Family(f::adopted_bytes()))),
    ] {
        let (mut root, forges) = adopting();
        if answer_first {
            adoption_answer(&mut root, answer.clone());
        }
        rows(&mut root, forges, Box::new([adopted(with_first(row(FORGE)), f::PROJECT, b"hello")]));
        if !answer_first {
            assert_eq!(
                home::facts(&root, f::PROJECT).repository,
                Repository::In { owner: b"work".as_slice().into(), name: b"hello".as_slice().into() },
                "rows override an outstanding adoption"
            );
            adoption_answer(&mut root, answer);
        }
        assert_eq!(
            home::facts(&root, f::PROJECT).repository,
            Repository::In { owner: b"work".as_slice().into(), name: b"hello".as_slice().into() },
            "answer_first={answer_first}"
        );
    }
}
#[test]
fn github_uses_the_first_accepted_usable_installations_account() {
    for with_account in [false, true] {
        let mut f = fixture();
        let mut github = row(FORGE);
        github.kind = w::Kind::Github;
        github.installations = List::with_capacity(4);
        for (id, stray, suspended, login) in [
            (1, true, false, Some(b"stray".as_slice())),
            (2, false, true, Some(b"suspended".as_slice())),
            (3, false, false, if with_account { Some(b"accepted".as_slice()) } else { None }),
            (4, false, false, Some(b"later".as_slice())),
        ] {
            let mut account = None;
            if let Some(login) = login {
                account = Some(w::Account::new(&w::CEILINGS, w::AccountParts { id, login: login.into() }).unwrap());
            }
            github
                .installations
                .push(
                    w::Installation::new(&w::CEILINGS, w::InstallationParts { id, suspended, stray, account }).unwrap(),
                )
                .unwrap();
        }
        rows(&mut f.root, f.forges, Box::new([github]));
        assert_eq!(
            home::facts(&f.root, f::PROJECT).repository,
            Repository::GitHub { account: if with_account { Some(b"accepted".as_slice().into()) } else { None } }
        );
    }
}
fn chat_rows() -> List<c::TaskLine> {
    let limits =
        jig_client_protocol_link::Limits::new(f::limits().client.domain, f::limits().client.exchanges).unwrap();
    let tasks =
        jig_client_protocol_link::translate::documents::tasks(jig_client_test_records::tasks_rows_record(), &limits)
            .unwrap();
    tasks.rows
}
#[test]
fn home_counts_live_and_ended_chats_as_asked() {
    for ended in [false, true] {
        let mut f = fixture();
        assert!(!home::facts(&f.root, f::PROJECT).chat_asked, "undelivered");
        if ended {
            up(
                &mut f.root,
                Up::ReadDone {
                    read: f.closed,
                    result: Readout::Jig(c::ReadResult::Value(c::ReadValue::Tasks(Box::new(c::TaskPage {
                        live: List::with_capacity(EMPTY),
                        live_after: None,
                        ended: c::EndedGroup::Rows { rows: chat_rows(), after: None, cut: false },
                    })))),
                },
            );
        } else {
            deliver(&mut f.root, f.live, c::Delivery::Tasks(Box::new(c::Tasks { rows: chat_rows(), more: false })));
        }
        assert!(home::facts(&f.root, f::PROJECT).chat_asked, "ended={ended}");
        assert!(!home::facts(&f.root, OTHER_PROJECT).chat_asked, "another project's facts");
    }
}
#[test]
fn home_starts_one_first_chat_and_opens_its_answer() {
    let mut f = fixture();
    let mut out = event(&mut f.root, Event::Act { action: Action::Pages(Act::FirstChat) });
    let key = f.root.pages().chats().unwrap().start.unwrap();
    let expected = jig_client::shapes::chat(
        &f::chats::me(),
        f::PROJECT,
        c::Text::new(pages::chats::FIRST_CHAT_WORDS.into(), NAME_BYTES).unwrap(),
    )
    .unwrap();
    let mut sent = List::with_capacity(ONE);
    while let Some(request) = out.pop() {
        if let Request::Unlinked { request: jig_client::UnlinkedRequest::Down { down: Down::Send { request, .. } } } =
            request
        {
            sent.push(request).unwrap();
        }
    }
    assert_eq!(sent.into_boxed().as_ref(), [jig_client::Ask::Jig(Box::new(expected))]);
    assert_eq!(event(&mut f.root, Event::Act { action: Action::Pages(Act::FirstChat) }).len(), 0);
    let mut out = Queue::uncapped();
    crate::route::told(
        &mut f.root,
        &f::env(&f::limits()),
        Told::Answered {
            key,
            outcome: jig_client::Outcome::Answered(jig_client::Answered::Jig(c::Answer::Done(c::Outcome::Document(
                c::DocumentOutcome::Created { first: CHAT, last: CHAT, given: None },
            )))),
        },
        &mut out,
    );
    let mut shown = false;
    for request in &out {
        shown |= *request == Request::Show { address: Address::Chat { task: CHAT }, replace: false };
    }
    assert!(shown, "the first chat's answer opens it");
}
#[test]
fn another_persons_project_cannot_start_a_chat_in_the_open_project() {
    let mut f = fixture();
    go(&mut f.root, OTHER_PROJECT);
    let out = event(&mut f.root, Event::Act { action: Action::Pages(Act::FirstChat) });
    assert!(f.root.pages().chats().unwrap().start.is_none());
    let mut sent = false;
    for request in &out {
        if let Request::Unlinked { request: jig_client::UnlinkedRequest::Down { down: Down::Send { .. } } } = request {
            sent = true;
        }
    }
    assert!(!sent, "Home's address never falls back to another project");
    assert!(!home::facts(&f.root, OTHER_PROJECT).chat_asked);
}

#[test]
fn playground_repository_comes_from_its_own_read() {
    let playground = forge::Playground {
        state: forge::PlaygroundState::Running,
        forge: FORGE,
        project: Some(f::PROJECT),
        provider: 1,
        owner: b"playground".as_slice().into(),
        name: b"widget".as_slice().into(),
        cues: Box::default(),
    };
    for state in [forge::forges::State::Unknown, forge::forges::State::Ended] {
        assert_eq!(
            forge::listed::held(&state, Some(&playground), f::PROJECT),
            forge::listed::Held::One { owner: b"playground", name: b"widget" }
        );
    }
}
