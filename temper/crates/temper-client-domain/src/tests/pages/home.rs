//! Home's pure reading and owned watches (temper/docs/design/client/pages/README.md).
use super::welcome::{default_answer, deliver, event, press_default, ready_default_with_forges};
use crate::tests::fixtures as f;
use crate::{
    Action, Address, Domain, Event, Place, Request, forge,
    pages::{
        self, Act,
        home::{self, Facts, Needs, Repository, Skip},
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
const NEED_GROUPS: u32 = 4;
const INBOX_CARDS: u32 = 5;
const INBOX_QUERIES: usize = 2;
// Two chats handles, entries and two inbox queries.
const HOME_HANDLES: u32 = 5;
// The ordered repository fixture holds three adoptions on its lower forge.
const REPOSITORIES: u32 = 3;
// Home has two independently completed steps (client/pages/README.md).
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
    inbox: Token,
    dismissed: Token,
    status: Token,
    decided: Token,
    away: Token,
    approvals: Token,
}
fn fixture() -> Fixture {
    fixture_with_me(f::chats::me())
}
fn fixture_with_me(frame: c::Me) -> Fixture {
    fixture_with_limits(frame, f::limits())
}
fn fixture_with_limits(frame: c::Me, limits: crate::Limits) -> Fixture {
    let mut root = crate::unlinked(&limits, f::SEED);
    let me = f::start(&mut root, List::with_capacity(EMPTY));
    let out = deliver(&mut root, me, c::Delivery::Me(Box::new(frame)));
    let approvals = crate::tests::approvals::opening(&out).unwrap();
    let dismissed = opened(&out, &forge::dismissed::watch(&f::limits().forge).unwrap()).unwrap();
    let forges = opened(&out, &forge::forges::watch(&f::limits().forge).unwrap()).unwrap();
    let settings = opened(&out, &Watchable::Jig(c::Subject::Settings)).unwrap();
    let away = opened(&out, &forge::away::watch(&f::limits().forge).unwrap()).unwrap();
    let standing = opened(&out, &Watchable::Jig(c::Subject::Inbox { window: f::limits().client.domain.rows }));
    let mut out = Queue::uncapped();
    crate::step(
        &mut root,
        &f::env(&limits),
        Event::Went { place: Place::Address(Address::Home { project: f::PROJECT }) },
        &mut out,
    );
    let status = opened(&out, &forge::status::watch(f::PROJECT, &f::limits().forge).unwrap()).unwrap();
    let decided = opened(&out, &forge::decided::watch(f::PROJECT, &f::limits().forge).unwrap()).unwrap();
    let inbox = opened(&out, &Watchable::Jig(c::Subject::Inbox { window: root.pages().inbox().unwrap().window }))
        .or(standing)
        .unwrap();
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
    Fixture {
        root,
        settings,
        live: live.unwrap(),
        closed: closed.unwrap(),
        forges,
        inbox,
        dismissed,
        status,
        decided,
        away,
        approvals,
    }
}
fn settings(fixture: &mut Fixture, follows: c::SettingFollows) -> Queue<Request> {
    setting(fixture, c::Setting::Application { kind: 2, scope: f::PROJECT, number: 1 }, follows)
}
fn setting(fixture: &mut Fixture, setting: c::Setting, follows: c::SettingFollows) -> Queue<Request> {
    let mut lines = List::with_capacity(ONE);
    lines.push(c::SettingLine { setting, name: None, follows, last: None, application: None }).unwrap();
    deliver(
        &mut fixture.root,
        fixture.settings,
        c::Delivery::Settings(Box::new(c::SettingsDocument {
            newest: 0,
            settings: lines,
            held: List::with_capacity(EMPTY),
            pending: List::with_capacity(EMPTY),
        })),
    )
}
fn row(number: u16) -> w::ForgeRowParts {
    w::ForgeRowParts {
        starts_with: w::StartsWith::Repository,
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
                bytes: wire::forges(&w::Forges::new(&w::CEILINGS, w::ForgesParts { forges, linked: false }).unwrap()),
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
fn the_two_steps_count_only_what_is_left() {
    for (configured, repository, left) in [
        (false, Repository::None, TWO_STEPS),
        (true, Repository::None, ONE_STEP),
        (false, Repository::In { owner: b"work".as_slice().into(), name: b"hello".as_slice().into() }, ONE_STEP),
        (true, Repository::In { owner: b"work".as_slice().into(), name: b"hello".as_slice().into() }, NO_STEPS),
        (true, Repository::Unknown, ONE_STEP),
        (true, Repository::Add { forge: FORGE }, ONE_STEP),
        (true, Repository::Adopting(first()), ONE_STEP),
        (true, Repository::Offer { first: first(), refused: None }, ONE_STEP),
        (true, Repository::GitHub { account: None }, ONE_STEP),
    ] {
        assert_eq!(
            Facts {
                decided: None,
                spell: None,
                configured,
                repository,
                needs: Needs::Unknown,
                delivered: true,
                skip: Skip::NotSkipped,
                skip_refused: false,
                status: home::Status::Unknown,
            }
            .left(),
            left
        );
    }
}
#[test]
fn settings_are_configured_only_after_a_change_is_delivered() {
    let mut f = fixture();
    assert!(!home::facts(&f.root, f::PROJECT, Wall::EPOCH).configured, "undelivered");
    settings(&mut f, c::SettingFollows::Default);
    assert!(!home::facts(&f.root, f::PROJECT, Wall::EPOCH).configured, "defaults");
    let before = pages::shown(f.root.pages());
    settings(&mut f, c::SettingFollows::Change);
    assert!(home::facts(&f.root, f::PROJECT, Wall::EPOCH).configured);
    assert!(pages::shown(f.root.pages()) > before, "a settings change redraws Home");
}
#[test]
fn refused_and_ended_settings_are_not_configured() {
    for end in [c::WatchEnd::NotAllowed, c::WatchEnd::SubjectEnded] {
        let mut f = fixture();
        settings(&mut f, c::SettingFollows::Change);
        up(&mut f.root, Up::WatchEnded { watch: f.settings, end });
        assert!(!home::facts(&f.root, f::PROJECT, Wall::EPOCH).configured, "{end:?}");
    }
}
#[test]
fn only_the_projects_own_application_change_marks_the_way_of_working_done() {
    const PROJECT: u32 = 3;
    const OWN: c::Setting = c::Setting::Application { kind: 2, scope: PROJECT, number: 1 };
    use c::SettingFollows::{Change, Default};
    for (item, follows, end, done) in [
        (Some(OWN), Change, None, true),
        (Some(OWN), Default, None, false),
        (Some(c::Setting::Application { kind: 2, scope: 4, number: 1 }), Change, None, false),
        (Some(c::Setting::Rules), Change, None, false),
        (Some(c::Setting::Application { kind: 2, scope: 0, number: 1 }), Change, None, false),
        (Some(c::Setting::Charter(c::SettingNumber { number: 1 })), Change, None, false),
        (Some(c::Setting::Template(c::SettingNumber { number: 1 })), Change, None, false),
        (Some(c::Setting::Policy(c::SettingNumber { number: 1 })), Change, None, false),
        (Some(c::Setting::Model(c::SettingNumber { number: 1 })), Change, None, false),
        (Some(c::Setting::ProjectPolicy { project: PROJECT }), Change, None, false),
        (None, Change, None, false),
        (Some(OWN), Change, Some(c::WatchEnd::SubjectEnded), false),
        (Some(OWN), Change, Some(c::WatchEnd::NotAllowed), false),
    ] {
        let mut f = fixture();
        if let Some(item) = item {
            setting(&mut f, item, follows.clone());
        }
        if let Some(end) = end {
            up(&mut f.root, Up::WatchEnded { watch: f.settings, end });
        }
        assert_eq!(
            home::facts(&f.root, PROJECT, Wall::EPOCH).configured,
            done,
            "setting={item:?}, follows={follows:?}, end={end:?}"
        );
    }
}
#[test]
fn home_shares_settings_and_releases_its_chat_reads() {
    let mut f = fixture();
    up(&mut f.root, Up::Watched { watch: f.settings, event: c::WatchEvent::Opened });
    let settings = f.root.pages().home().unwrap().settings;
    let chats = f.root.pages().chats().unwrap();
    assert_eq!(settings, f.root.forge().settings_handle());
    let handles = [chats.live, chats.closed];
    assert_eq!(chats.filter, crate::ChatsFilter::All);
    let repeat = go(&mut f.root, f::PROJECT);
    assert!(opened(&repeat, &Watchable::Jig(c::Subject::Settings)).is_none());
    let out = event(&mut f.root, Event::Went { place: Place::Address(Address::Settings { project: f::PROJECT }) });
    for handle in handles {
        assert!(f.root.client().document(handle).is_none(), "released {handle:?}");
    }
    assert!(f.root.pages().home().is_none());
    let chats = f.root.pages().chats().expect("Settings holds its own chats");
    assert_ne!(chats.live, handles[0]);
    assert!(f.root.client().document(chats.live).is_some());
    let mut closed = false;
    for request in &out {
        if let Request::Unlinked { request: jig_client::UnlinkedRequest::Down { down: Down::Close { watch } } } =
            request
        {
            closed |= *watch == f.settings;
        }
    }
    assert!(!closed, "the session keeps its settings watch");
    assert!(f.root.client().document(settings).is_some());
}
#[test]
fn unknown_and_ended_forges_never_invent_a_repository() {
    let mut f = fixture();
    assert_eq!(home::facts(&f.root, f::PROJECT, Wall::EPOCH).repository, Repository::Unknown);
    up(&mut f.root, Up::WatchEnded { watch: f.forges, end: c::WatchEnd::NotAllowed });
    assert_eq!(home::facts(&f.root, f::PROJECT, Wall::EPOCH).repository, Repository::None);
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
        home::facts(&f.root, f::PROJECT, Wall::EPOCH).repository,
        Repository::In { owner: b"work".as_slice().into(), name: b"first".as_slice().into() }
    );
    assert_eq!(
        forge::listed::held(f.root.forges(), None, f::PROJECT),
        forge::listed::Held::Several {
            forge: match f.root.forges() {
                forge::forges::State::Rows(rows) => rows.first(),
                forge::forges::State::Unknown | forge::forges::State::Ended => None,
            },
            owner: b"work",
            name: b"first",
            more: 2
        }
    );
}
#[test]
fn a_first_repository_is_offered_until_it_is_adopted_anywhere() {
    let mut f = fixture();
    rows(&mut f.root, f.forges, Box::new([with_first(row(FORGE))]));
    assert_eq!(
        home::facts(&f.root, f::PROJECT, Wall::EPOCH).repository,
        Repository::Offer { first: first(), refused: None }
    );
    rows(&mut f.root, f.forges, Box::new([adopted(with_first(row(FORGE)), OTHER_PROJECT, b"hello")]));
    assert_eq!(home::facts(&f.root, f::PROJECT, Wall::EPOCH).repository, Repository::Add { forge: FORGE });
}
#[test]
fn adoption_out_precedes_an_offer_and_refusal_is_kept() {
    let (mut root, forges) = adopting();
    assert_eq!(home::facts(&root, f::PROJECT, Wall::EPOCH).repository, Repository::Adopting(first()));
    adoption_answer(&mut root, refusal());
    assert_eq!(
        home::facts(&root, f::PROJECT, Wall::EPOCH).repository,
        Repository::Offer { first: first(), refused: Some(forge::Answer::ForgeRefused(forge::Reason::HomeHeld)) }
    );
    assert_eq!(
        home::facts(&root, OTHER_PROJECT, Wall::EPOCH).repository,
        Repository::Offer { first: first(), refused: None }
    );
    rows(&mut root, forges, Box::new([adopted(with_first(row(FORGE)), OTHER_PROJECT, b"hello")]));
    assert_eq!(
        home::facts(&root, f::PROJECT, Wall::EPOCH).repository,
        Repository::Add { forge: FORGE },
        "no stale offer after adoption elsewhere"
    );
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
                home::facts(&root, f::PROJECT, Wall::EPOCH).repository,
                Repository::In { owner: b"work".as_slice().into(), name: b"hello".as_slice().into() },
                "rows override an outstanding adoption"
            );
            adoption_answer(&mut root, answer);
        }
        assert_eq!(
            home::facts(&root, f::PROJECT, Wall::EPOCH).repository,
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
        github.installations = List::with_capacity(NEED_GROUPS);
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
            home::facts(&f.root, f::PROJECT, Wall::EPOCH).repository,
            Repository::GitHub { account: if with_account { Some(b"accepted".as_slice().into()) } else { None } }
        );
    }
}
fn chat_rows() -> List<c::TaskLine> {
    let mut rows = List::with_capacity(ONE);
    rows.push(super::chats::chat_line()).unwrap();
    rows
}
#[test]
fn another_persons_project_cannot_start_an_asking_chat() {
    let mut f = fixture();
    event(&mut f.root, Event::Went { place: Place::Address(Address::Ask { project: OTHER_PROJECT }) });
    event(&mut f.root, Event::Act { action: Action::Pages(Act::ChatWords { words: b"My change".as_slice().into() }) });
    let out = event(&mut f.root, Event::Act { action: Action::Pages(Act::StartChat) });
    assert!(f.root.pages().chats().unwrap().start.is_none());
    let mut sent = false;
    for request in &out {
        if let Request::Unlinked { request: jig_client::UnlinkedRequest::Down { down: Down::Send { .. } } } = request {
            sent = true;
        }
    }
    assert!(!sent, "Ask temper's address never falls back to another project");
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
            forge::listed::Held::One { forge: None, owner: b"playground", name: b"widget" }
        );
    }
}

fn inbox_reads(out: &Queue<Request>) -> Box<[(Token, c::Read)]> {
    let mut reads = List::with_capacity(out.len());
    for request in out {
        if let Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down { down: Down::Read { read, what: jig_client::Readable::Jig(what) } },
        } = request
        {
            reads.push((*read, what.clone())).unwrap();
        }
    }
    reads.into_boxed()
}
fn inbox_page(root: &mut Domain, read: Token, rows: List<c::InboxEntry>, cut: bool) -> Queue<Request> {
    up(
        root,
        Up::ReadDone {
            read,
            result: Readout::Jig(c::ReadResult::Value(c::ReadValue::Inbox(Box::new(c::InboxPage {
                rows,
                cut,
                before: None,
                after: None,
                selection: None,
            })))),
        },
    )
}
fn need_rows() -> List<c::InboxEntry> {
    let mut rows = List::with_capacity(NEED_GROUPS);
    for (object, group) in [
        (f::acts::QUESTION, c::InboxGroup::Questions),
        (c::ObjectKey::Proposal(f::acts::PROPOSAL), c::InboxGroup::Proposals),
        (c::ObjectKey::Escalation(CHAT), c::InboxGroup::Escalations),
        (c::ObjectKey::PersonTask(CHAT), c::InboxGroup::PersonTasks),
    ] {
        rows.push(f::inbox_entry(object, group, f::PROJECT, 0)).unwrap();
    }
    rows
}
#[test]
fn home_reads_its_projects_needs_and_releases_them() {
    for place in [Place::Address(Address::Settings { project: f::PROJECT }), Place::Missing] {
        let mut f = fixture();
        let inbox = f.root.pages().inbox().unwrap();
        assert_eq!(inbox.filter, crate::address::Filter { project: Some(f::PROJECT), kind: None, goal: None });
        let settings = f.root.pages().home().unwrap().settings;
        let chats = f.root.pages().chats().unwrap();
        assert_eq!(chats.live.asker, crate::owner::Owner::Pages.asker());
        assert_eq!(chats.closed.number, chats.live.number + 1);
        assert_eq!(inbox.entries.number, chats.live.number + 2);
        up(&mut f.root, Up::Watched { watch: f.inbox, event: c::WatchEvent::Opened });
        let out = f::inbox_entries(&mut f.root, f.inbox, List::with_capacity(EMPTY));
        let reads = inbox_reads(&out);
        assert_eq!(reads.len(), INBOX_QUERIES);
        for ((_, read), group) in reads.iter().zip([c::InboxReadGroup::Needs, c::InboxReadGroup::Updates]) {
            assert_eq!(*read, c::Read::Inbox { group, project: Some(f::PROJECT), kind: None, goal: None, after: None });
        }
        inbox_page(&mut f.root, reads[0].0, need_rows(), false);
        let inbox = f.root.pages().inbox().unwrap();
        let chats = f.root.pages().chats().unwrap();
        let mut handles = List::with_capacity(inbox.cards.len().checked_add(HOME_HANDLES).unwrap());
        for handle in [chats.live, chats.closed, inbox.entries, inbox.queries[0], inbox.queries[1]] {
            handles.push(handle).unwrap();
        }
        for card in &inbox.cards {
            handles.push(card.handle).unwrap();
        }
        let mut last = 0;
        for handle in &handles {
            last = last.max(handle.number);
        }
        let keeps_chats = place == Place::Address(Address::Settings { project: f::PROJECT });
        let out = event(&mut f.root, Event::Went { place });
        assert!(f.root.pages().inbox().is_none());
        assert_eq!(f.root.pages().chats().is_some(), keeps_chats);
        for handle in &handles {
            assert!(f.root.client().document(*handle).is_none(), "released {handle:?}");
        }
        let mut closed = false;
        for request in &out {
            if let Request::Unlinked { request: jig_client::UnlinkedRequest::Down { down: Down::Close { watch } } } =
                request
            {
                closed |= *watch == f.inbox;
            }
        }
        assert!(!closed, "the standing count keeps the shared inbox watch open");
        assert!(f.root.client().document(pages::needs::handle()).is_some());
        go(&mut f.root, f::PROJECT);
        assert_eq!(f.root.pages().home().unwrap().settings, settings, "settings is the session handle");
        assert!(f.root.pages().chats().unwrap().live.number > last, "no page handle is reused");
    }
}
#[test]
fn needs_are_unknown_until_delivered_then_count_the_needs_alone() {
    for cut in [false, true] {
        let mut f = fixture();
        assert_eq!(home::facts(&f.root, f::PROJECT, Wall::EPOCH).needs, Needs::Unknown);
        let out = f::inbox_entries(&mut f.root, f.inbox, List::with_capacity(EMPTY));
        assert_eq!(home::facts(&f.root, f::PROJECT, Wall::EPOCH).needs, Needs::Unknown, "queries still pending");
        let reads = inbox_reads(&out);
        let needs = inbox_page(&mut f.root, reads[0].0, need_rows(), cut);
        assert_eq!(
            home::facts(&f.root, f::PROJECT, Wall::EPOCH).needs,
            Needs::Unknown,
            "updates and card reads pending"
        );
        let mut updates = List::with_capacity(ONE);
        updates.push(f::inbox_entry(c::ObjectKey::Result(CHAT), c::InboxGroup::Results, f::PROJECT, 0)).unwrap();
        let updates = inbox_page(&mut f.root, reads[1].0, updates, !cut);
        let needs = inbox_reads(&needs);
        let updates = inbox_reads(&updates);
        let cards = needs.iter().chain(updates.iter());
        for (read, _) in cards {
            assert_eq!(home::facts(&f.root, f::PROJECT, Wall::EPOCH).needs, Needs::Unknown, "each card must answer");
            up(&mut f.root, Up::ReadDone { read: *read, result: Readout::Limit });
        }
        assert_eq!(home::facts(&f.root, f::PROJECT, Wall::EPOCH).needs, Needs::Count { count: NEED_GROUPS, cut });
        let inbox = f.root.pages().inbox().unwrap();
        for (card, group) in inbox.needs().zip([
            c::InboxGroup::Questions,
            c::InboxGroup::Proposals,
            c::InboxGroup::Escalations,
            c::InboxGroup::PersonTasks,
        ]) {
            assert_eq!(card.group, group);
        }
        assert_eq!(inbox.cards.len(), INBOX_CARDS, "the update stays in Inbox");
        assert_eq!(home::facts(&f.root, OTHER_PROJECT, Wall::EPOCH).needs, Needs::Unknown);
        let out = f::inbox_entries(&mut f.root, f.inbox, List::with_capacity(EMPTY));
        assert_eq!(
            home::facts(&f.root, f::PROJECT, Wall::EPOCH).needs,
            Needs::Count { count: NEED_GROUPS, cut },
            "known needs stand through the refresh"
        );
        for (read, _) in inbox_reads(&out) {
            inbox_page(&mut f.root, read, List::with_capacity(EMPTY), false);
        }
        assert_eq!(home::facts(&f.root, f::PROJECT, Wall::EPOCH).needs, Needs::Count { count: 0, cut: false });
    }
}
#[test]
fn home_hears_its_chats_and_its_inbox_alike() {
    for chats_first in [false, true] {
        let mut f = fixture();
        let out = f::inbox_entries(&mut f.root, f.inbox, List::with_capacity(EMPTY));
        let reads = inbox_reads(&out);
        for chats in [chats_first, !chats_first] {
            let before = pages::shown(f.root.pages());
            if chats {
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
                for (read, _) in &reads {
                    inbox_page(&mut f.root, *read, List::with_capacity(EMPTY), true);
                }
            }
            assert!(pages::shown(f.root.pages()) > before, "both pages redraw Home");
        }
        assert_eq!(home::facts(&f.root, f::PROJECT, Wall::EPOCH).needs, Needs::Count { count: 0, cut: true });
    }
}

const ASKING: u32 = 5;
const TWO_TEMPLATES: u32 = 2;
const FACTS: u32 = 5;
// Live watch, read page's live rows, read page's ended rows.
const CHAT_SOURCES: u32 = 3;

fn asking_me() -> c::Me {
    let mut me = f::chats::me();
    let project = me.projects.get_mut(0).unwrap();
    let stock = project.templates.get(0).unwrap().clone();
    let mut asking = stock.clone();
    asking.number = ASKING;
    asking.executor = c::TemplateExecutor::Charter(ASKING);
    project.templates = List::with_capacity(TWO_TEMPLATES);
    project.templates.push(stock).unwrap();
    project.templates.push(asking).unwrap();
    me
}
fn chat_page(f: &mut Fixture, rows: List<c::TaskLine>, live: bool, after: Option<c::Cursor>) {
    let (live, ended) = if live {
        (rows, c::EndedGroup::None)
    } else {
        (List::with_capacity(EMPTY), c::EndedGroup::Rows { rows, after: after.clone(), cut: false })
    };
    up(
        &mut f.root,
        Up::ReadDone {
            read: f.closed,
            result: Readout::Jig(c::ReadResult::Value(c::ReadValue::Tasks(Box::new(c::TaskPage {
                live,
                live_after: after,
                ended,
            })))),
        },
    );
}
fn dismissal_rows(f: &mut Fixture, projects: &[u32]) {
    let mut dismissed = List::with_capacity(wire::DISMISSED);
    for project in projects {
        dismissed
            .push(w::PersonFact::ToBegin(w::ToBegin::new(&w::CEILINGS, w::ToBeginParts { project: *project }).unwrap()))
            .unwrap();
    }
    up(
        &mut f.root,
        Up::FamilyDelivered {
            watch: f.dismissed,
            record: jig_client::Family {
                family: wire::FORGE,
                version: wire::CLIENT_VERSION,
                bytes: wire::dismissals(&w::Dismissals::new(&w::CEILINGS, w::DismissalsParts { dismissed }).unwrap()),
            },
        },
    );
}
fn ready(f: &mut Fixture) {
    rows(&mut f.root, f.forges, Box::default());
    settings(f, c::SettingFollows::Default);
    chat_page(f, List::with_capacity(EMPTY), false, None);
    deliver(
        &mut f.root,
        f.live,
        c::Delivery::Tasks(Box::new(c::Tasks { rows: List::with_capacity(EMPTY), more: false })),
    );
    dismissal_rows(f, &[]);
    assert!(home::facts(&f.root, f::PROJECT, Wall::EPOCH).to_begin());
}
fn skip(f: &mut Fixture) -> (jig_client::Key, Queue<Request>) {
    let serial = f.root.pages().serial;
    let out = event(&mut f.root, Event::Act { action: Action::Pages(Act::SkipToBegin) });
    let key = jig_client::Key { asker: crate::owner::Owner::Pages.asker(), number: serial.checked_add(1).unwrap() };
    (key, out)
}
fn skip_answer(f: &mut Fixture, key: jig_client::Key, outcome: jig_client::Outcome) {
    crate::route::told(&mut f.root, &f::env(&f::limits()), Told::Answered { key, outcome }, &mut Queue::uncapped());
}
fn family_answer(outcome: w::Outcome) -> jig_client::Outcome {
    jig_client::Outcome::Answered(jig_client::Answered::Family(jig_client::Family {
        family: wire::FORGE,
        version: wire::CLIENT_VERSION,
        bytes: wire::answer(&w::Answer::new(&w::CEILINGS, w::AnswerParts { outcome }).unwrap()),
    }))
}
#[test]
fn an_asking_chat_marks_the_way_of_working_done() {
    for source in 0..CHAT_SOURCES {
        let mut f = fixture_with_me(asking_me());
        let mut row = super::chats::chat_line();
        row.chip.executor = c::ChipExecutor::Agent { charter: ASKING };
        let mut rows = List::with_capacity(ONE);
        rows.push(row).unwrap();
        if source == 0 {
            deliver(&mut f.root, f.live, c::Delivery::Tasks(Box::new(c::Tasks { rows, more: false })));
        } else {
            chat_page(&mut f, rows, source == 1, None);
        }
        let facts = home::facts(&f.root, f::PROJECT, Wall::EPOCH);
        assert!(facts.configured, "source={source}");
        assert!(!home::facts(&f.root, OTHER_PROJECT, Wall::EPOCH).configured);
    }
}
#[test]
fn a_stock_chat_marks_no_step() {
    for me in [f::chats::me(), asking_me()] {
        for source in 0..CHAT_SOURCES {
            let mut f = fixture_with_me(me.clone());
            if source == 0 {
                deliver(&mut f.root, f.live, c::Delivery::Tasks(Box::new(c::Tasks { rows: chat_rows(), more: false })));
            } else {
                chat_page(&mut f, chat_rows(), source == 1, None);
            }
            let facts = home::facts(&f.root, f::PROJECT, Wall::EPOCH);
            assert!(!facts.configured);
            assert_eq!(facts.left(), TWO_STEPS);
        }
    }
    let mut chosen = asking_me();
    chosen.projects.get_mut(0).unwrap().templates.get_mut(1).unwrap().executor = c::TemplateExecutor::Chosen;
    assert!(!pages::chats::asking(&chosen, f::PROJECT, &c::ChipExecutor::Agent { charter: ASKING }));
    for source in 0..CHAT_SOURCES {
        let mut f = fixture_with_me(asking_me());
        if source == 0 {
            deliver(
                &mut f.root,
                f.live,
                c::Delivery::Tasks(Box::new(c::Tasks { rows: List::with_capacity(EMPTY), more: true })),
            );
        } else {
            chat_page(
                &mut f,
                List::with_capacity(EMPTY),
                source == 1,
                Some(c::Cursor::new(b"next".as_slice().into(), NAME_BYTES).unwrap()),
            );
        }
        let facts = home::facts(&f.root, f::PROJECT, Wall::EPOCH);
        assert!(!facts.configured, "an Ask temper chat past the cut isn't seen");
    }
}
#[test]
fn to_begin_waits_for_every_fact() {
    for withheld in 0..FACTS {
        let mut f = fixture();
        for fact in 0..FACTS {
            if fact != withheld {
                deliver_fact(&mut f, fact);
            }
        }
        let facts = home::facts(&f.root, f::PROJECT, Wall::EPOCH);
        assert!(!facts.to_begin(), "withheld={withheld}");
        assert_eq!(facts.delivered, withheld == 4);
        deliver_fact(&mut f, withheld);
        assert!(home::facts(&f.root, f::PROJECT, Wall::EPOCH).to_begin());
    }
    for ended in [c::WatchEnd::NotAllowed, c::WatchEnd::SubjectEnded] {
        let mut f = fixture();
        for watch in [f.settings, f.live, f.forges] {
            up(&mut f.root, Up::WatchEnded { watch, end: ended });
        }
        up(&mut f.root, Up::ReadDone { read: f.closed, result: Readout::Limit });
        dismissal_rows(&mut f, &[]);
        let facts = home::facts(&f.root, f::PROJECT, Wall::EPOCH);
        assert!(facts.delivered);
        assert_eq!(facts.left(), TWO_STEPS);
        assert!(facts.to_begin());
        up(&mut f.root, Up::WatchEnded { watch: f.dismissed, end: ended });
        assert_eq!(home::facts(&f.root, f::PROJECT, Wall::EPOCH).skip, Skip::Unknown);
        assert!(!home::facts(&f.root, f::PROJECT, Wall::EPOCH).to_begin());
    }
    let mut f = fixture_with_me(asking_me());
    ready(&mut f);
    let mut chat_lines = chat_rows();
    chat_lines.get_mut(0).unwrap().chip.executor = c::ChipExecutor::Agent { charter: ASKING };
    deliver(&mut f.root, f.live, c::Delivery::Tasks(Box::new(c::Tasks { rows: chat_lines, more: false })));
    assert!(home::facts(&f.root, f::PROJECT, Wall::EPOCH).configured);
    up(&mut f.root, Up::WatchEnded { watch: f.live, end: c::WatchEnd::SubjectEnded });
    assert!(!home::facts(&f.root, f::PROJECT, Wall::EPOCH).configured, "ended live read leaves its step");
    let mut facts = home::facts(&f.root, f::PROJECT, Wall::EPOCH);
    facts.configured = true;
    facts.repository = Repository::In { owner: b"work".as_slice().into(), name: b"hello".as_slice().into() };
    assert!(!facts.to_begin(), "nothing left");
    for (address, skipped, changed, repository_in, beginning) in [
        (Address::Home { project: f::PROJECT }, false, false, false, true),
        (Address::Ask { project: f::PROJECT }, false, false, true, true),
        (Address::Ask { project: f::PROJECT }, true, false, true, false),
        (Address::Chats { filter: crate::ChatsFilter::All }, false, false, false, false),
        (Address::Ask { project: f::PROJECT }, false, true, false, true),
        (Address::Ask { project: f::PROJECT }, false, true, true, false),
    ] {
        let mut f = fixture();
        ready(&mut f);
        let mut out = event(&mut f.root, Event::Went { place: Place::Address(address.clone()) });
        while let Some(request) = out.pop() {
            if let Request::Unlinked { request: jig_client::UnlinkedRequest::Down { down } } = request {
                match down {
                    Down::Open { watch, subject: Watchable::Jig(c::Subject::Tasks { .. }) } => f.live = watch,
                    Down::Read { read, what: jig_client::Readable::Jig(c::Read::Tasks { .. }) } => f.closed = read,
                    Down::Close { watch } => {
                        let mut closed = up(&mut f.root, Up::WatchEnded { watch, end: c::WatchEnd::Closed });
                        while let Some(next) = closed.pop() {
                            out.push(next);
                        }
                    }
                    Down::Save { .. }
                    | Down::Erase { .. }
                    | Down::Send { .. }
                    | Down::Open { .. }
                    | Down::Continue { .. }
                    | Down::Read { .. } => {}
                }
            }
        }
        ready(&mut f);
        if skipped {
            dismissal_rows(&mut f, &[f::PROJECT]);
        }
        if repository_in {
            rows(&mut f.root, f.forges, Box::new([adopted(row(FORGE), f::PROJECT, b"hello")]));
        }
        settings(&mut f, if changed { c::SettingFollows::Change } else { c::SettingFollows::Default });
        assert_eq!(home::facts(&f.root, f::PROJECT, Wall::EPOCH).configured, changed, "{address:?}");
        assert_eq!(
            home::beginning(&f.root, f::PROJECT, Wall::EPOCH),
            beginning,
            "{address:?}, skipped={skipped}, changed={changed}"
        );
    }
}
fn deliver_fact(f: &mut Fixture, fact: u32) {
    match fact {
        0 => rows(&mut f.root, f.forges, Box::default()),
        1 => {
            settings(f, c::SettingFollows::Default);
        }
        2 => {
            let out = deliver(
                &mut f.root,
                f.live,
                c::Delivery::Tasks(Box::new(c::Tasks { rows: List::with_capacity(EMPTY), more: false })),
            );
            for request in &out {
                if let Request::Unlinked {
                    request: jig_client::UnlinkedRequest::Down { down: Down::Read { read, .. } },
                } = request
                {
                    f.closed = *read;
                }
            }
        }
        3 => chat_page(f, List::with_capacity(EMPTY), false, None),
        4 => dismissal_rows(f, &[]),
        _ => panic!("five facts"),
    }
}
#[test]
fn a_skipped_project_hides_to_begin_and_a_refused_skip_brings_it_back() {
    let refusals = [
        refusal(),
        family_answer(w::Outcome::CoreRefused(
            w::CoreRefused::new(&w::CEILINGS, w::CoreRefusedParts { refusal: Box::default() }).unwrap(),
        )),
        family_answer(w::Outcome::Removed),
        jig_client::Outcome::Answered(jig_client::Answered::Family(jig_client::Family {
            family: wire::FORGE,
            version: wire::CLIENT_VERSION,
            bytes: Box::default(),
        })),
        jig_client::Outcome::Answered(jig_client::Answered::Jig(c::Answer::Refused {
            refusal: c::Refusal::NotOffered,
            at: None,
        })),
        jig_client::Outcome::Limit,
        jig_client::Outcome::Lost,
        jig_client::Outcome::NotKept,
        jig_client::Outcome::Unconfirmed,
        jig_client::Outcome::Held,
        jig_client::Outcome::Unsent,
        jig_client::Outcome::Full { held: ONE, room_at: Wall::EPOCH },
    ];
    for outcome in refusals {
        let mut f = fixture();
        ready(&mut f);
        let (key, out) = skip(&mut f);
        let mut sends = 0_u32;
        for request in &out {
            if let Request::Unlinked {
                request: jig_client::UnlinkedRequest::Down { down: Down::Send { request, .. } },
            } = request
            {
                if *request == forge::status::ask(f::PROJECT, &f::limits().forge).unwrap() {
                    continue;
                }
                assert_eq!(
                    *request,
                    forge::dismissed::ask(
                        forge::dismissed::PersonFact::ToBegin { project: f::PROJECT },
                        &f::limits().forge
                    )
                    .unwrap()
                );
                sends += 1;
            }
        }
        assert_eq!(sends, 1);
        assert_eq!(home::facts(&f.root, f::PROJECT, Wall::EPOCH).skip, Skip::Skipped);
        assert!(event(&mut f.root, Event::Act { action: Action::Pages(Act::SkipToBegin) }).is_empty());
        assert_eq!(f.root.pages().serial, key.number + 1, "one skip and one look key");
        skip_answer(&mut f, jig_client::Key { number: key.number + 2, ..key }, outcome.clone());
        assert_eq!(home::facts(&f.root, f::PROJECT, Wall::EPOCH).skip, Skip::Skipped, "only our answer");
        skip_answer(&mut f, key, outcome);
        let facts = home::facts(&f.root, f::PROJECT, Wall::EPOCH);
        assert!(facts.skip_refused);
        assert_eq!(facts.skip, Skip::NotSkipped);
        assert!(facts.to_begin());
        let (next, _) = skip(&mut f);
        assert_ne!(next, key);
        assert!(!home::facts(&f.root, f::PROJECT, Wall::EPOCH).skip_refused);
        dismissal_rows(&mut f, &[f::PROJECT]);
        skip_answer(&mut f, next, jig_client::Outcome::Unknown);
        assert_eq!(home::facts(&f.root, f::PROJECT, Wall::EPOCH).skip, Skip::Skipped, "subject answers unknown");
    }
}
#[test]
fn a_skip_answered_before_its_row_arrives_stays_hidden() {
    let mut f = fixture();
    ready(&mut f);
    let (key, _) = skip(&mut f);
    let before = pages::shown(f.root.pages());
    skip_answer(&mut f, key, family_answer(w::Outcome::Dismissed));
    assert!(pages::shown(f.root.pages()) > before);
    dismissal_rows(&mut f, &[]);
    assert_eq!(home::facts(&f.root, f::PROJECT, Wall::EPOCH).skip, Skip::Skipped);
    assert!(!home::facts(&f.root, f::PROJECT, Wall::EPOCH).to_begin());
    event(&mut f.root, Event::Went { place: Place::Address(Address::Settings { project: f::PROJECT }) });
    assert!(event(&mut f.root, Event::Act { action: Action::Pages(Act::SkipToBegin) }).is_empty());
    go(&mut f.root, f::PROJECT);
    assert_eq!(
        home::facts(&f.root, f::PROJECT, Wall::EPOCH).skip,
        Skip::NotSkipped,
        "acknowledgement lasts until leaving Home"
    );
    for outcome in [jig_client::Outcome::Unknown, family_answer(w::Outcome::Unknown)] {
        let mut f = fixture();
        ready(&mut f);
        let (key, _) = skip(&mut f);
        skip_answer(&mut f, key, outcome);
        let facts = home::facts(&f.root, f::PROJECT, Wall::EPOCH);
        assert_eq!(facts.skip, Skip::NotSkipped, "unknown follows the subject");
        assert!(!facts.skip_refused);
        let (next, _) = skip(&mut f);
        skip_answer(&mut f, next, jig_client::Outcome::Unknown);
        assert_ne!(key, next, "unknown ends the request");
    }
}
#[test]
fn another_projects_dismissal_leaves_to_begin() {
    let mut f = fixture();
    ready(&mut f);
    dismissal_rows(&mut f, &[OTHER_PROJECT]);
    assert_eq!(home::facts(&f.root, f::PROJECT, Wall::EPOCH).skip, Skip::NotSkipped);
    assert!(home::facts(&f.root, f::PROJECT, Wall::EPOCH).to_begin());
    dismissal_rows(&mut f, &[f::PROJECT]);
    assert_eq!(home::facts(&f.root, f::PROJECT, Wall::EPOCH).skip, Skip::Skipped);
    assert!(!home::facts(&f.root, f::PROJECT, Wall::EPOCH).to_begin());
}

#[test]
fn skip_respects_its_page_and_request_bounds() {
    let mut f = fixture();
    ready(&mut f);
    let serial = f.root.pages().serial;
    let mut limits = f::limits();
    limits.forge.field_bytes = u32::MAX;
    assert!(
        forge::dismissed::ask(forge::dismissed::PersonFact::ToBegin { project: f::PROJECT }, &limits.forge).is_none()
    );
    let mut out = Queue::uncapped();
    pages::step(
        &mut f.root.pages,
        &f::env(&limits),
        &f.root.client,
        pages::Event::Act { act: Act::SkipToBegin },
        &mut out,
    );
    assert!(out.is_empty());
    assert_eq!(f.root.pages().serial, serial);
    assert_eq!(home::facts(&f.root, f::PROJECT, Wall::EPOCH).skip, Skip::NotSkipped);
    pages::step(
        &mut f.root.pages,
        &f::env(&f::limits()),
        &f.root.client,
        pages::Event::Act { act: Act::SkipToBegin },
        &mut out,
    );
    let key = jig_client::Key { asker: crate::owner::Owner::Pages.asker(), number: serial + 1 };
    assert_eq!(
        out.pop(),
        Some(pages::Request::Client(jig_client::Event::Send {
            key,
            name: None,
            request: forge::dismissed::ask(
                forge::dismissed::PersonFact::ToBegin { project: f::PROJECT },
                &f::limits().forge
            )
            .unwrap(),
        }))
    );
    assert!(out.is_empty());
    assert_eq!(home::facts(&f.root, f::PROJECT, Wall::EPOCH).skip, Skip::Skipped);
    for address in [Address::Settings { project: f::PROJECT }, Address::Chats { filter: crate::ChatsFilter::All }] {
        event(&mut f.root, Event::Went { place: Place::Address(address) });
        let serial = f.root.pages().serial;
        assert!(event(&mut f.root, Event::Act { action: Action::Pages(Act::SkipToBegin) }).is_empty());
        assert_eq!(f.root.pages().serial, serial);
    }
}

fn status_document(f: &mut Fixture, document: &w::ProjectStatus) -> Queue<Request> {
    up(&mut f.root, Up::FamilyDelivered { watch: f.status, record: crate::tests::status::record(document) })
}
fn look_sends(out: &Queue<Request>) -> u32 {
    let expected = forge::status::ask(f::PROJECT, &f::limits().forge).unwrap();
    let mut sends = 0_u32;
    for request in out {
        if let Request::Unlinked { request: jig_client::UnlinkedRequest::Down { down: Down::Send { request, .. } } } =
            request
            && *request == expected
        {
            sends = sends.checked_add(1).unwrap();
        }
    }
    sends
}
fn look_key(f: &Fixture) -> jig_client::Key {
    jig_client::Key { asker: crate::owner::Owner::Pages.asker(), number: f.root.pages().serial }
}
fn looked(f: &mut Fixture, key: jig_client::Key, outcome: jig_client::Outcome) -> Queue<Request> {
    let mut out = Queue::uncapped();
    crate::route::told(&mut f.root, &f::env(&f::limits()), Told::Answered { key, outcome }, &mut out);
    out
}
fn enter_skipped() -> Fixture {
    let mut f = fixture();
    dismissal_rows(&mut f, &[f::PROJECT]);
    rows(&mut f.root, f.forges, Box::default());
    settings(&mut f, c::SettingFollows::Default);
    chat_page(&mut f, List::with_capacity(EMPTY), false, None);
    deliver(
        &mut f.root,
        f.live,
        c::Delivery::Tasks(Box::new(c::Tasks { rows: List::with_capacity(EMPTY), more: false })),
    );
    f
}
#[test]
fn a_watch_opens_with_the_page_and_closes_with_it() {
    let mut f = fixture();
    let settings = f.root.pages().home().unwrap().settings;
    let status = jig_client::Handle {
        number: f.root.pages().chats().unwrap().live.number - 1,
        asker: crate::owner::Owner::Pages.asker(),
    };
    assert!(f.root.client().document(status).is_some(), "Home owns its project status handle");
    for watch in [f.status, f.settings] {
        up(&mut f.root, Up::Watched { watch, event: c::WatchEvent::Opened });
    }
    let out = event(&mut f.root, Event::Went { place: Place::Address(Address::Settings { project: f::PROJECT }) });
    let mut closes = 0_u32;
    for request in &out {
        if let Request::Unlinked { request: jig_client::UnlinkedRequest::Down { down: Down::Close { watch } } } =
            request
            && *watch == f.status
        {
            closes = closes.checked_add(1).unwrap();
        }
    }
    assert_eq!(closes, 1);
    assert!(f.root.client().document(status).is_none());
    assert!(f.root.client().document(settings).is_some(), "session settings survives Home");
    let out = go(&mut f.root, f::PROJECT);
    assert_ne!(opened(&out, &forge::status::watch(f::PROJECT, &f::limits().forge).unwrap()), Some(f.status));
}
#[test]
fn a_look_goes_once_when_the_facts_are_delivered() {
    for last in 0..FACTS {
        let mut f = fixture();
        let serial = f.root.pages().serial;
        for fact in 0..FACTS {
            if fact != last {
                deliver_fact(&mut f, fact);
            }
        }
        if last != 4 {
            dismissal_rows(&mut f, &[f::PROJECT]);
        }
        assert_eq!(f.root.pages().serial, serial, "withheld fact {last}");
        if last == 4 {
            dismissal_rows(&mut f, &[f::PROJECT]);
        } else {
            deliver_fact(&mut f, last);
        }
        let key = look_key(&f);
        assert_eq!(key.number, serial + 1, "last fact {last} sends one look");
        assert!(f.root.client().pending(key).is_some(), "Pages owns the look key");
        let document = crate::tests::status::document(3, 1, None);
        assert_eq!(look_sends(&status_document(&mut f, &document)), 0, "not resent while out");
        assert_eq!(look_sends(&looked(&mut f, key, family_answer(w::Outcome::Looked))), 0);
        assert_eq!(look_sends(&status_document(&mut f, &document)), 0, "not resent after the answer");
        assert_eq!(f.root.pages().serial, key.number);
    }
    let mut f = fixture();
    let serial = f.root.pages().serial;
    for fact in 0..4 {
        deliver_fact(&mut f, fact);
    }
    assert_eq!(f.root.pages().serial, serial);
    let out = up(&mut f.root, Up::WatchEnded { watch: f.dismissed, end: c::WatchEnd::NotAllowed });
    assert_eq!(look_sends(&out), 1, "ended dismissals settle To begin too");
}
#[test]
fn no_look_goes_while_to_begin_shows() {
    let mut f = fixture();
    let serial = f.root.pages().serial;
    ready(&mut f);
    assert!(home::facts(&f.root, f::PROJECT, Wall::EPOCH).to_begin());
    assert_eq!(f.root.pages().serial, serial);
    let document = crate::tests::status::document(3, 1, None);
    assert_eq!(look_sends(&status_document(&mut f, &document)), 0);
    assert_eq!(home::facts(&f.root, f::PROJECT, Wall::EPOCH).status, home::Status::Unknown);
}
#[test]
fn a_skip_sends_the_look() {
    let mut f = fixture();
    ready(&mut f);
    let (skip_key, out) = skip(&mut f);
    assert_eq!(look_sends(&out), 1);
    assert_eq!(look_key(&f).number, skip_key.number + 1);
    assert_eq!(look_key(&f).asker, crate::owner::Owner::Pages.asker());
    assert!(!home::facts(&f.root, f::PROJECT, Wall::EPOCH).to_begin());
    skip_answer(&mut f, skip_key, family_answer(w::Outcome::Dismissed));
    assert_eq!(look_sends(&settings(&mut f, c::SettingFollows::Change)), 0);
}
#[test]
fn a_last_step_sends_the_look() {
    let mut f = fixture();
    let serial = f.root.pages().serial;
    ready(&mut f);
    rows(&mut f.root, f.forges, Box::new([adopted(row(FORGE), f::PROJECT, b"hello")]));
    deliver(&mut f.root, f.live, c::Delivery::Tasks(Box::new(c::Tasks { rows: chat_rows(), more: false })));
    assert!(home::facts(&f.root, f::PROJECT, Wall::EPOCH).to_begin());
    assert_eq!(look_sends(&settings(&mut f, c::SettingFollows::Change)), 1);
    assert_eq!(f.root.pages().serial, serial + 1);
}
#[test]
fn a_second_entry_looks_again() {
    let mut f = enter_skipped();
    let first = look_key(&f);
    assert!(f.root.client().pending(first).is_some());
    event(&mut f.root, Event::Went { place: Place::Address(Address::Settings { project: f::PROJECT }) });
    let out = looked(&mut f, first, family_answer(w::Outcome::Looked));
    assert_eq!(look_sends(&out), 0, "an answer after leaving sends nothing");
    for watch in [f.status, f.live, f.inbox] {
        up(&mut f.root, Up::WatchEnded { watch, end: c::WatchEnd::Closed });
    }
    let out = go(&mut f.root, f::PROJECT);
    assert!(opened(&out, &Watchable::Jig(c::Subject::Settings)).is_none(), "settings stays with the session");
    assert_eq!(look_sends(&out), 0, "the new facts must settle");
    for request in &out {
        if let Request::Unlinked { request: jig_client::UnlinkedRequest::Down { down } } = request {
            match down {
                Down::Open { watch, subject: Watchable::Jig(c::Subject::Tasks { .. }) } => f.live = *watch,
                Down::Read { read, what: jig_client::Readable::Jig(c::Read::Tasks { .. }) } => f.closed = *read,
                Down::Open { .. }
                | Down::Read { .. }
                | Down::Save { .. }
                | Down::Erase { .. }
                | Down::Send { .. }
                | Down::Continue { .. }
                | Down::Close { .. } => {}
            }
        }
    }
    chat_page(&mut f, List::with_capacity(EMPTY), false, None);
    let out = deliver(
        &mut f.root,
        f.live,
        c::Delivery::Tasks(Box::new(c::Tasks { rows: List::with_capacity(EMPTY), more: false })),
    );
    assert_eq!(look_sends(&out), 1, "facts={:?}; output={out:?}", home::facts(&f.root, f::PROJECT, Wall::EPOCH));
    assert_eq!(look_sends(&settings(&mut f, c::SettingFollows::Default)), 0);
    assert_ne!(look_key(&f), first);
}
#[test]
fn a_refused_watch_leaves_status_unknown() {
    for end in [c::WatchEnd::NotAllowed, c::WatchEnd::SubjectEnded] {
        let mut f = enter_skipped();
        let key = look_key(&f);
        looked(&mut f, key, family_answer(w::Outcome::Looked));
        let document = crate::tests::status::document(3, 1, None);
        status_document(&mut f, &document);
        assert_eq!(
            home::facts(&f.root, f::PROJECT, Wall::EPOCH).status,
            home::Status::Shown(Box::new(document.clone()))
        );
        let out = up(&mut f.root, Up::WatchEnded { watch: f.status, end });
        assert_eq!(home::facts(&f.root, f::PROJECT, Wall::EPOCH).status, home::Status::Unknown);
        assert!(opened(&out, &forge::status::watch(f::PROJECT, &f::limits().forge).unwrap()).is_none());
        let out = status_document(&mut f, &document);
        assert_eq!(
            home::facts(&f.root, f::PROJECT, Wall::EPOCH).status,
            home::Status::Unknown,
            "late documents stay ended"
        );
        assert!(opened(&out, &forge::status::watch(f::PROJECT, &f::limits().forge).unwrap()).is_none());
    }
}
#[test]
fn status_waits_for_the_looks_answer() {
    for document_first in [true, false] {
        let mut f = enter_skipped();
        let key = look_key(&f);
        let document = crate::tests::status::document(3, 1, Some(3));
        if document_first {
            status_document(&mut f, &document);
        }
        assert_eq!(home::facts(&f.root, f::PROJECT, Wall::EPOCH).status, home::Status::Unknown);
        looked(&mut f, jig_client::Key { number: key.number + 1, ..key }, family_answer(w::Outcome::Looked));
        assert_eq!(
            home::facts(&f.root, f::PROJECT, Wall::EPOCH).status,
            home::Status::Unknown,
            "only the look's answer"
        );
        looked(&mut f, key, family_answer(w::Outcome::Looked));
        if !document_first {
            assert_eq!(
                home::facts(&f.root, f::PROJECT, Wall::EPOCH).status,
                home::Status::Unknown,
                "the document must arrive too"
            );
            status_document(&mut f, &document);
        }
        assert_eq!(home::facts(&f.root, f::PROJECT, Wall::EPOCH).status, home::Status::Shown(Box::new(document)));
        let next = crate::tests::status::document(4, 1, Some(4));
        status_document(&mut f, &next);
        assert_eq!(home::facts(&f.root, f::PROJECT, Wall::EPOCH).status, home::Status::Shown(Box::new(next)));
        assert_eq!(home::facts(&f.root, OTHER_PROJECT, Wall::EPOCH).status, home::Status::Unknown);
    }
}
#[test]
fn a_refused_look_still_shows_the_document() {
    for answer in
        [refusal(), jig_client::Outcome::Unknown, family_answer(w::Outcome::Unknown), jig_client::Outcome::Lost]
    {
        let mut f = enter_skipped();
        let key = look_key(&f);
        let document = crate::tests::status::document(3, 1, None);
        status_document(&mut f, &document);
        assert_eq!(look_sends(&looked(&mut f, key, answer)), 0);
        assert_eq!(home::facts(&f.root, f::PROJECT, Wall::EPOCH).status, home::Status::Shown(Box::new(document)));
    }
}
#[test]
fn landed_since_saturates() {
    let mut facts = home::facts(&fixture().root, f::PROJECT, Wall::EPOCH);
    assert_eq!(facts.landed_since(), 0);
    for (landed, seen, expected) in [(4, 2, 2), (2, 2, 0), (1, 2, 0), (u64::MAX, 0, u64::MAX)] {
        facts.status = home::Status::Shown(Box::new(crate::tests::status::document(landed, seen, None)));
        assert_eq!(facts.landed_since(), expected);
    }
}
#[test]
fn a_brief_can_be_new_by_landing_or_by_when_it_was_written() {
    let mut facts = home::facts(&fixture().root, f::PROJECT, Wall::EPOCH);
    assert!(facts.brief().is_none());
    for (brief, previous, shown) in [
        (None, Some(50), false),
        (Some(2), None, false),
        (Some(2), Some(99), true),
        (Some(2), Some(100), false),
        (Some(3), None, true),
    ] {
        let mut parts = crate::tests::status::document(4, 2, brief).into_parts();
        parts.previous = previous;
        let document = w::ProjectStatus::new(&w::CEILINGS, parts).unwrap();
        facts.status = home::Status::Shown(Box::new(document.clone()));
        assert_eq!(facts.brief(), if shown { document.brief().as_ref() } else { None });
    }
}

#[test]
fn a_watch_that_cannot_be_made_waits_for_reentry() {
    let mut f = fixture();
    let out = event(&mut f.root, Event::Went { place: Place::Address(Address::Settings { project: f::PROJECT }) });
    for request in &out {
        if let Request::Unlinked { request: jig_client::UnlinkedRequest::Down { down: Down::Open { watch, .. } } } =
            request
        {
            up(&mut f.root, Up::Watched { watch: *watch, event: c::WatchEvent::Opened });
        }
    }
    finish_closes(&mut f, &out);
    let mut limits = f::limits();
    limits.forge.field_bytes = u32::MAX;
    let mut out = Queue::uncapped();
    crate::step(
        &mut f.root,
        &f::env(&limits),
        Event::Went { place: Place::Address(Address::Home { project: f::PROJECT }) },
        &mut out,
    );
    for request in &out {
        if let Request::Unlinked { request: jig_client::UnlinkedRequest::Down { down: Down::Open { watch, .. } } } =
            request
        {
            up(&mut f.root, Up::Watched { watch: *watch, event: c::WatchEvent::Opened });
        }
    }
    finish_closes(&mut f, &out);
    let subject = forge::status::watch(f::PROJECT, &f::limits().forge).unwrap();
    assert!(opened(&out, &subject).is_none());
    assert_eq!(home::facts(&f.root, f::PROJECT, Wall::EPOCH).status, home::Status::Unknown);
    assert!(opened(&out, &Watchable::Jig(c::Subject::Settings)).is_none(), "settings stays with the session");
    assert!(
        opened(&settings(&mut f, c::SettingFollows::Default), &subject).is_none(),
        "a later step cannot retry the watch"
    );
    let out = event(&mut f.root, Event::Went { place: Place::Address(Address::Settings { project: f::PROJECT }) });
    assert!(opened(&out, &subject).is_none());
    for request in &out {
        if let Request::Unlinked { request: jig_client::UnlinkedRequest::Down { down: Down::Open { watch, .. } } } =
            request
        {
            up(&mut f.root, Up::Watched { watch: *watch, event: c::WatchEvent::Opened });
        }
    }
    finish_closes(&mut f, &out);
    let out = go(&mut f.root, f::PROJECT);
    assert!(opened(&out, &subject).is_some(), "the next page entry opens status: {out:?}");
}
#[test]
fn a_status_document_must_name_this_project() {
    let mut f = enter_skipped();
    let key = look_key(&f);
    looked(&mut f, key, family_answer(w::Outcome::Looked));
    let document = crate::tests::status::document(3, 1, None);
    let mut other = document.clone().into_parts();
    other.project = OTHER_PROJECT;
    status_document(&mut f, &w::ProjectStatus::new(&w::CEILINGS, other).unwrap());
    assert_eq!(home::facts(&f.root, f::PROJECT, Wall::EPOCH).status, home::Status::Unknown);
    status_document(&mut f, &document);
    assert_eq!(home::facts(&f.root, f::PROJECT, Wall::EPOCH).status, home::Status::Shown(Box::new(document)));
}
#[test]
fn a_refused_goal_from_changes_no_page() {
    let mut f = fixture();
    ready(&mut f);
    let before = home::facts(&f.root, f::PROJECT, Wall::EPOCH);
    let key = jig_client::Key { asker: crate::owner::Owner::Pages.asker(), number: CHAT };
    let fact = forge::dismissed::PersonFact::GoalFrom { project: f::PROJECT, chat: CHAT, goal: CHAT + 1, at: 0 };
    event(
        &mut f.root,
        Event::Client {
            event: jig_client::Event::Send {
                key,
                name: None,
                request: forge::dismissed::ask(fact, &f::limits().forge).unwrap(),
            },
        },
    );
    for outcome in [
        family_answer(w::Outcome::ForgeRefused(
            w::ForgeRefused::new(&w::CEILINGS, w::ForgeRefusedParts { reason: w::ForgeReason::TooLarge }).unwrap(),
        )),
        family_answer(w::Outcome::Dismissed),
    ] {
        skip_answer(&mut f, key, outcome);
        assert_eq!(home::facts(&f.root, f::PROJECT, Wall::EPOCH), before);
    }
}

#[test]
fn needs_count_falls_when_answered_and_keeps_later_cards() {
    // One inbox question plus one approval, and one omitted approval.
    const BOTH: u32 = 2;
    let mut f = fixture();
    crate::tests::approvals::deliver(
        &mut f.root,
        f.approvals,
        &[crate::tests::approvals::row(crate::tests::approvals::CHANGE, f::PROJECT, None)],
        ONE,
    );
    let out = f::inbox_entries(&mut f.root, f.inbox, List::with_capacity(EMPTY));
    let reads = inbox_reads(&out);
    let mut rows = List::with_capacity(ONE);
    rows.push(f::inbox_entry(f::acts::QUESTION, c::InboxGroup::Questions, f::PROJECT, 0)).unwrap();
    inbox_page(&mut f.root, reads[0].0, rows, false);
    let cards = inbox_page(&mut f.root, reads[1].0, List::with_capacity(EMPTY), false);
    let cards = inbox_reads(&cards);
    up(&mut f.root, Up::ReadDone { read: cards[0].0, result: Readout::Jig(c::ReadResult::Value(f::acts::question())) });
    event(
        &mut f.root,
        Event::Act { action: Action::Pages(Act::Needs(pages::needs::Act::Later { object: f::acts::QUESTION })) },
    );
    assert_eq!(home::facts(&f.root, f::PROJECT, Wall::EPOCH).needs, Needs::Count { count: BOTH, cut: true });
    event(
        &mut f.root,
        Event::Act {
            action: Action::Pages(Act::Needs(pages::needs::Act::Choose {
                object: f::acts::QUESTION,
                words: Box::from(*b"Yes"),
            })),
        },
    );
    assert_eq!(home::facts(&f.root, f::PROJECT, Wall::EPOCH).needs, Needs::Count { count: ONE, cut: true });
    event(
        &mut f.root,
        Event::Act { action: Action::Pages(Act::Needs(pages::needs::Act::Undo { object: f::acts::QUESTION })) },
    );
    assert_eq!(home::facts(&f.root, f::PROJECT, Wall::EPOCH).needs, Needs::Count { count: BOTH, cut: true });
}

const COPY_ADDRESSES: [&[u8]; 2] = [b"https://github.com/acme/widget", b"https://github.com/acme/other"];
fn copy_row() -> w::ForgeRowParts {
    let mut row = row(FORGE);
    row.starts_with = w::StartsWith::Copies(
        w::CopyAddresses::new(
            &w::CEILINGS,
            w::CopyAddressesParts { addresses: List::from_boxed(COPY_ADDRESSES.map(Into::into).into()) },
        )
        .unwrap(),
    );
    row
}
fn copy_making(address: &[u8], state: w::MakingState, project: Option<u32>) -> w::RepositoryMaking {
    let (org, name) = forge::making::target(address).unwrap();
    w::RepositoryMaking::new(
        &w::CEILINGS,
        w::RepositoryMakingParts {
            org,
            name,
            from: w::MakeFrom::Copy(
                w::MakeCopy::new(&w::CEILINGS, w::MakeCopyParts { address: address.into() }).unwrap(),
            ),
            into: match project {
                Some(project) => Some(
                    w::MakeInto::new(&w::CEILINGS, w::MakeIntoParts { project, home: address == COPY_ADDRESSES[0] })
                        .unwrap(),
                ),
                None => None,
            },
            state,
        },
    )
    .unwrap()
}
fn copy_adopted(address: &[u8], project: u32) -> w::Adoption {
    let (owner, name) = forge::making::target(address).unwrap();
    w::Adoption::new(&w::CEILINGS, w::AdoptionParts { owner, name, project }).unwrap()
}
fn copy_failed() -> w::MakingState {
    w::MakingState::Failed(
        w::MakingFailed::new(
            &w::CEILINGS,
            w::MakingFailedParts {
                step: w::MakeStep::Repository,
                reason: w::MakeFailure::Taken,
                words: b"name taken".as_slice().into(),
            },
        )
        .unwrap(),
    )
}
fn copy_line(name: &[u8], state: home::CopyState) -> home::CopyLine {
    home::CopyLine { org: b"acme".as_slice().into(), name: name.into(), state }
}
#[test]
fn copy_states_follow_the_intent_before_repository_and_forge_key_order() {
    use home::CopyState::{Failed, In, Running, Waiting};
    type Case = (&'static str, Box<[(usize, w::MakingState)]>, Box<[usize]>, Repository);
    let cases: [Case; 5] = [
        (
            "one running, one waiting",
            Box::new([(0, w::MakingState::Making), (1, w::MakingState::Waiting)]),
            Box::new([]),
            Repository::Copying {
                lines: Box::new([
                    copy_line(b"widget", Running { host: b"github.com".as_slice().into() }),
                    copy_line(b"other", Waiting),
                ]),
            },
        ),
        (
            "one in, one failed",
            Box::new([(1, copy_failed())]),
            Box::new([0]),
            Repository::Copying {
                lines: Box::new([
                    copy_line(b"widget", In),
                    copy_line(
                        b"other",
                        Failed { words: b"acme/other failed at repository: name taken".as_slice().into() },
                    ),
                ]),
            },
        ),
        (
            "all in",
            Box::new([]),
            Box::new([1, 0]),
            Repository::CopiesIn {
                names: Box::new([b"acme/widget".as_slice().into(), b"acme/other".as_slice().into()]),
            },
        ),
        (
            "one left unasked",
            Box::new([]),
            Box::new([0]),
            Repository::CopyOffer { forge: FORGE, addresses: Box::new([COPY_ADDRESSES[1].into()]) },
        ),
        (
            "none asked",
            Box::new([]),
            Box::new([]),
            Repository::CopyOffer { forge: FORGE, addresses: COPY_ADDRESSES.map(Into::into).into() },
        ),
    ];
    for (label, makings, repositories, expected) in cases {
        let mut f = fixture();
        let mut row = copy_row();
        // The watch's keys sort other before widget; intent order is the reverse.
        row.makings = List::with_capacity(2);
        for (i, state) in makings.iter().rev() {
            row.makings.push(copy_making(COPY_ADDRESSES[*i], state.clone(), Some(f::PROJECT))).unwrap();
        }
        row.repositories = List::with_capacity(2);
        for i in repositories {
            row.repositories.push(copy_adopted(COPY_ADDRESSES[i], f::PROJECT)).unwrap();
        }
        rows(&mut f.root, f.forges, Box::new([row]));
        let facts = home::facts(&f.root, f::PROJECT, Wall::EPOCH);
        assert_eq!(facts.repository, expected, "{label}");
        assert_eq!(facts.left(), if label == "all in" { ONE_STEP } else { TWO_STEPS }, "{label}");
    }
}
#[test]
fn copies_elsewhere_are_asked_but_have_no_line_and_renamed_destinations_are_not_ready_copies() {
    let mut f = fixture();
    let mut row = copy_row();
    row.makings = List::from_boxed(Box::new([
        copy_making(COPY_ADDRESSES[0], w::MakingState::Making, Some(OTHER_PROJECT)),
        copy_making(COPY_ADDRESSES[1], w::MakingState::Waiting, None),
    ]));
    rows(&mut f.root, f.forges, Box::new([row.clone()]));
    assert_eq!(home::facts(&f.root, f::PROJECT, Wall::EPOCH).repository, Repository::Add { forge: FORGE });
    row.makings = List::with_capacity(0);
    row.repositories = List::from_boxed(Box::new([
        copy_adopted(COPY_ADDRESSES[0], OTHER_PROJECT),
        copy_adopted(COPY_ADDRESSES[1], f::PROJECT),
    ]));
    rows(&mut f.root, f.forges, Box::new([row.clone()]));
    assert_eq!(
        home::facts(&f.root, f::PROJECT, Wall::EPOCH).repository,
        Repository::CopiesIn { names: Box::new([b"acme/other".as_slice().into()]) }
    );
    let renamed = row.repositories.get(0).unwrap().clone().into_parts();
    *row.repositories.get_mut(0).unwrap() =
        w::Adoption::new(&w::CEILINGS, w::AdoptionParts { name: b"renamed".as_slice().into(), ..renamed }).unwrap();
    rows(&mut f.root, f.forges, Box::new([row]));
    assert_eq!(
        home::facts(&f.root, f::PROJECT, Wall::EPOCH).repository,
        Repository::CopyOffer { forge: FORGE, addresses: Box::new([COPY_ADDRESSES[0].into()]) }
    );
}
#[test]
fn made_and_adopting_copies_share_the_adopting_line_and_other_rows_follow_intent() {
    let made =
        w::MakingMade::new(&w::CEILINGS, w::MakingMadeParts { id: 1, branch: b"main".as_slice().into() }).unwrap();
    let mut f = fixture();
    let mut row = copy_row();
    row.makings = List::from_boxed(Box::new([
        copy_making(COPY_ADDRESSES[1], w::MakingState::Adopting(made.clone()), Some(f::PROJECT)),
        copy_making(COPY_ADDRESSES[0], w::MakingState::Made(made), Some(f::PROJECT)),
    ]));
    let mut other = self::row(OTHER_FORGE);
    other.makings = List::from_boxed(Box::new([copy_making(
        b"https://code.example/else/last",
        w::MakingState::Making,
        Some(f::PROJECT),
    )]));
    rows(&mut f.root, f.forges, Box::new([row, other]));
    assert_eq!(
        home::facts(&f.root, f::PROJECT, Wall::EPOCH).repository,
        Repository::Copying {
            lines: Box::new([
                copy_line(b"widget", home::CopyState::Adopting),
                copy_line(b"other", home::CopyState::Adopting),
                home::CopyLine {
                    org: b"else".as_slice().into(),
                    name: b"last".as_slice().into(),
                    state: home::CopyState::Running { host: b"code.example".as_slice().into() }
                },
            ])
        }
    );
}
fn copy_act(root: &mut Domain, act: Act) -> Queue<pages::Request> {
    let mut out = Queue::uncapped();
    pages::step(&mut root.pages, &f::env(&f::limits()), &root.client, pages::Event::Act { act }, &mut out);
    out
}
type CopySend = (jig_client::Key, skein_lib::RequestKey, w::MakeRepository);
fn copy_sends(out: Queue<pages::Request>) -> Box<[CopySend]> {
    let mut sends = List::uncapped();
    for request in &out {
        if let pages::Request::Client(jig_client::Event::Send {
            key,
            name,
            request: jig_client::Ask::Family { record, .. },
        }) = request
        {
            let request = wire::decode_requesting(&record.bytes, &w::CEILINGS).unwrap().into_parts().request;
            let w::Request::MakeRepository(ask) = request else { panic!("copy request") };
            sends.push((*key, name.unwrap(), ask)).unwrap();
        }
    }
    sends.into_boxed()
}
#[test]
fn restarting_offers_only_unasked_copies_under_the_ready_keys_once_while_out() {
    let mut f = fixture();
    let mut row = copy_row();
    row.repositories = List::from_boxed(Box::new([copy_adopted(COPY_ADDRESSES[0], f::PROJECT)]));
    rows(&mut f.root, f.forges, Box::new([row]));
    let sends = copy_sends(copy_act(&mut f.root, Act::CopyOffered));
    assert_eq!(sends.len(), 1);
    assert_eq!(sends[0].1, forge::copy_key(2));
    assert_eq!(sends[0].2.org(), b"acme");
    assert_eq!(sends[0].2.name(), b"other");
    assert_eq!(w::MakeRepository::into(&sends[0].2).as_ref().unwrap().project(), f::PROJECT);
    assert!(!w::MakeRepository::into(&sends[0].2).as_ref().unwrap().home());
    assert!(copy_sends(copy_act(&mut f.root, Act::CopyOffered)).is_empty());
    crate::route::told(
        &mut f.root,
        &f::env(&f::limits()),
        Told::Answered { key: sends[0].0, outcome: jig_client::Outcome::Unknown },
        &mut Queue::uncapped(),
    );
    assert_eq!(copy_sends(copy_act(&mut f.root, Act::CopyOffered))[0].1, forge::copy_key(2));
}
#[test]
fn retry_copies_the_failed_rows_source_and_into_under_its_own_key_once_while_out() {
    let mut f = fixture();
    let mut row = copy_row();
    let making = copy_making(COPY_ADDRESSES[0], copy_failed(), Some(f::PROJECT));
    row.makings = List::from_boxed(Box::new([making.clone()]));
    rows(&mut f.root, f.forges, Box::new([row]));
    let act = Act::CopyAgain { org: b"acme".as_slice().into(), name: b"widget".as_slice().into() };
    let sends = copy_sends(copy_act(&mut f.root, act.clone()));
    assert_eq!(sends.len(), 1);
    assert_eq!(sends[0].2.from(), making.from());
    assert_eq!(w::MakeRepository::into(&sends[0].2), w::RepositoryMaking::into(&making));
    let number = skein_lib::Decimal::of(sends[0].0.number);
    let label = skein_lib::bytes::concat(&[b"home-copy-again-", number.as_bytes()]);
    assert_eq!(sends[0].1, crate::key::named(crate::key::TOOLS, &crate::key::Name::new(&label).unwrap()));
    assert!(copy_sends(copy_act(&mut f.root, act.clone())).is_empty());
    crate::route::told(
        &mut f.root,
        &f::env(&f::limits()),
        Told::Answered { key: sends[0].0, outcome: jig_client::Outcome::Unknown },
        &mut Queue::uncapped(),
    );
    let again = copy_sends(copy_act(&mut f.root, act.clone()));
    assert_eq!(again.len(), 1);
    assert_ne!(again[0].1, sends[0].1);
    go(&mut f.root, OTHER_PROJECT);
    assert!(copy_sends(copy_act(&mut f.root, act)).is_empty());
}

fn add_act(root: &mut Domain, act: Act) -> Queue<Request> {
    event(root, Event::Act { action: Action::Pages(act) })
}
fn add_fixture(start: forge::forges::Start) -> Fixture {
    let mut f = fixture();
    let mut forge = row(FORGE);
    forge.starts_with = w::StartsWith::Nothing;
    rows(&mut f.root, f.forges, Box::new([forge]));
    add_act(&mut f.root, Act::AddOpen { start });
    f
}
#[test]
fn add_is_for_the_lowest_connected_owned_or_managed_real_forgejo() {
    for (kind, mode, connected, expected) in [
        (w::Kind::Forgejo, w::Mode::Owned, true, Repository::Add { forge: FORGE }),
        (w::Kind::Forgejo, w::Mode::Managed, true, Repository::Add { forge: FORGE }),
        (w::Kind::Forgejo, w::Mode::Connected, true, Repository::None),
        (w::Kind::Forgejo, w::Mode::Fake, true, Repository::None),
        (w::Kind::Forgejo, w::Mode::Owned, false, Repository::None),
        (w::Kind::Github, w::Mode::Owned, true, Repository::GitHub { account: None }),
    ] {
        let mut f = fixture();
        let mut forge = row(FORGE);
        forge.kind = kind;
        forge.mode = mode;
        if !connected {
            forge.connection = w::Connection::NotConnected;
        }
        rows(&mut f.root, f.forges, Box::new([forge]));
        assert_eq!(home::facts(&f.root, f::PROJECT, Wall::EPOCH).repository, expected);
    }
    let mut f = fixture();
    rows(&mut f.root, f.forges, Box::new([row(FORGE), row(OTHER_FORGE)]));
    assert_eq!(home::facts(&f.root, f::PROJECT, Wall::EPOCH).repository, Repository::Add { forge: FORGE });
    rows(&mut f.root, f.forges, Box::new([row(FORGE), with_first(row(OTHER_FORGE))]));
    assert_eq!(
        home::facts(&f.root, f::PROJECT, Wall::EPOCH).repository,
        Repository::Offer { first: First { forge: OTHER_FORGE, ..first() }, refused: None }
    );
}
fn add_sends(f: &mut Fixture) -> Box<[(jig_client::Key, skein_lib::RequestKey, jig_client::Ask)]> {
    let mut sends = List::uncapped();
    let mut out = Queue::uncapped();
    pages::step(
        &mut f.root.pages,
        &f::env(&f::limits()),
        &f.root.client,
        pages::Event::Act { act: Act::AddSubmit },
        &mut out,
    );
    while let Some(request) = out.pop() {
        if let pages::Request::Client(jig_client::Event::Send { key, name: Some(name), request }) = request {
            sends.push((key, name, request)).unwrap();
        }
    }
    sends.into_boxed()
}
#[test]
fn add_copies_send_together_in_order_with_only_the_first_home() {
    let mut f = add_fixture(forge::forges::Start::Copies);
    add_act(
        &mut f.root,
        Act::AddWrite {
            field: home::AddField::Addresses,
            words: b"acme/widget\nhttps://github.com/acme/other.git".as_slice().into(),
        },
    );
    let serial = f.root.pages().serial;
    let sends = add_sends(&mut f);
    assert_eq!(sends.len(), COPY_ADDRESSES.len());
    for (index, ((key, name, request), address)) in sends.iter().zip(COPY_ADDRESSES).enumerate() {
        let number = serial + u64::try_from(index).unwrap() + 1;
        assert_eq!(key.number, number);
        let number_words = skein_lib::Decimal::of(number);
        let label = skein_lib::bytes::concat(&[b"home-add-", number_words.as_bytes()]);
        assert_eq!(*name, crate::key::named(crate::key::TOOLS, &crate::key::Name::new(&label).unwrap()));
        let (org, target) = forge::making::target(address).unwrap();
        let into = w::MakeInto::new(&w::CEILINGS, w::MakeIntoParts { project: f::PROJECT, home: index == 0 }).unwrap();
        assert_eq!(
            *request,
            forge::make_repository(
                &forge::making::ask(FORGE, &org, &target, Some(address), Some(into)).unwrap(),
                &f::limits().forge
            )
            .unwrap()
        );
    }
    assert!(add_sends(&mut f).is_empty(), "a pending sheet sends once");
    add_act(&mut f.root, Act::AddClose);
    assert!(f.root.pages().home().unwrap().add.is_some(), "a pending sheet stays");
    skip_answer(&mut f, sends[1].0, family_answer(w::Outcome::MakeKept));
    assert!(f.root.pages().home().unwrap().add.as_ref().unwrap().out.is_some());
    skip_answer(&mut f, sends[0].0, family_answer(w::Outcome::MakeKept));
    assert!(f.root.pages().home().unwrap().add.is_none(), "all accepted closes the sheet");
}
#[test]
fn add_empty_checks_fields_and_sends_into_this_project() {
    let mut f = add_fixture(forge::forges::Start::Empty);
    add_act(&mut f.root, Act::AddWrite { field: home::AddField::Org, words: Box::default() });
    assert!(add_sends(&mut f).is_empty());
    assert_eq!(f.root.pages().home().unwrap().add.as_ref().unwrap().refused, Some(home::AddRefused::Blank));
    add_act(&mut f.root, Act::AddWrite { field: home::AddField::Org, words: b" team ".as_slice().into() });
    add_act(&mut f.root, Act::AddWrite { field: home::AddField::Name, words: b" fresh ".as_slice().into() });
    let sends = add_sends(&mut f);
    assert_eq!(sends.len(), 1);
    let into = w::MakeInto::new(&w::CEILINGS, w::MakeIntoParts { project: f::PROJECT, home: true }).unwrap();
    assert_eq!(
        sends[0].2,
        forge::make_repository(
            &forge::making::ask(FORGE, b"team", b"fresh", None, Some(into)).unwrap(),
            &f::limits().forge
        )
        .unwrap()
    );
}
#[test]
fn add_refusals_stay_in_the_sheet_and_leaving_drops_it() {
    for why in [
        w::MakeRefusal::NoSuchForge,
        w::MakeRefusal::NotOffered,
        w::MakeRefusal::NotOwner,
        w::MakeRefusal::Invalid,
        w::MakeRefusal::HomeHeld,
        w::MakeRefusal::Full,
    ] {
        let mut f = add_fixture(forge::forges::Start::Empty);
        let sends = add_sends(&mut f);
        let refused = w::MakeRefused::new(&w::CEILINGS, w::MakeRefusedParts { why: why.clone() }).unwrap();
        skip_answer(&mut f, sends[0].0, family_answer(w::Outcome::MakeRefused(refused)));
        let sheet = f.root.pages().home().unwrap().add.as_ref().unwrap();
        assert_eq!(sheet.refused, Some(home::AddRefused::Make(why)));
        assert!(sheet.out.is_none());
        event(&mut f.root, Event::Went { place: Place::Address(Address::Settings { project: f::PROJECT }) });
        go(&mut f.root, f::PROJECT);
        assert!(f.root.pages().home().unwrap().add.is_none());
    }
}

const DAY: u64 = 86_400_000_000_000;
use crate::tests::decided::decision;
use skein_lib::Wall;
fn decisions(f: &mut Fixture, values: &[w::Decision]) -> Queue<Request> {
    up(&mut f.root, Up::FamilyDelivered { watch: f.decided, record: crate::tests::decided::record(values) })
}
fn away_form(f: &Fixture, wall: u64) -> Option<pages::decided::Shown> {
    home::facts(&f.root, f::PROJECT, Wall::from_nanos(wall)).decided
}
fn task_reads(out: &Queue<Request>) -> Box<[(u64, Token)]> {
    let mut reads = List::uncapped();
    for request in out {
        if let Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down {
                    down: Down::Read { read, what: jig_client::Readable::Jig(c::Read::Task { task }) },
                },
        } = request
        {
            reads.push((*task, *read)).unwrap();
        }
    }
    reads.into_boxed()
}
fn spell(f: &mut Fixture, off: Option<u64>, until: Option<u64>, landed: u64) {
    let mut rows = List::with_capacity(1);
    rows.push(
        w::AwayRow::new(&w::CEILINGS, w::AwayRowParts { project: f::PROJECT, since: 10, until, off, landed }).unwrap(),
    )
    .unwrap();
    up(
        &mut f.root,
        Up::FamilyDelivered {
            watch: f.away,
            record: jig_client::Family {
                family: wire::FORGE,
                version: wire::CLIENT_VERSION,
                bytes: wire::aways(&w::Aways::new(&w::CEILINGS, w::AwaysParts { rows }).unwrap()),
            },
        },
    );
}
#[test]
fn home_watches_decided_while_open() {
    use crate::address::Filter;
    use pages::decided::Document;
    const PROJECTS: [u32; 5] = [f::PROJECT, 91, 92, 93, 94];
    for (address, shown, expected) in [
        (Address::Home { project: f::PROJECT }, None, &[f::PROJECT][..]),
        (Address::Inbox { filter: Filter { project: Some(91), kind: None, goal: None } }, None, &[91][..]),
        (
            Address::Inbox { filter: Filter { project: None, kind: None, goal: None } },
            None,
            &[f::PROJECT, 91, 92, 93][..],
        ),
        (
            Address::Inbox { filter: Filter { project: None, kind: None, goal: None } },
            Some(94),
            &[94, f::PROJECT, 91, 92][..],
        ),
        (Address::Settings { project: f::PROJECT }, None, &[][..]),
    ] {
        let mut me = f::chats::me();
        let template = me.projects.get(0).unwrap().clone();
        me.projects = List::with_capacity(5);
        for project in PROJECTS {
            me.projects.push(c::MeProject { project, ..template.clone() }).unwrap();
        }
        let mut f = fixture_with_me(me);
        let out = event(&mut f.root, Event::Went { place: Place::Address(address.clone()) });
        finish_closes(&mut f, &out);
        if let Some(project) = shown {
            let watch =
                opened(&out, &Watchable::Jig(c::Subject::Inbox { window: f.root.pages().inbox().unwrap().window }))
                    .unwrap_or(f.inbox);
            let mut rows = List::with_capacity(1);
            rows.push(f::inbox_entry(f::acts::QUESTION, c::InboxGroup::Questions, project, 0)).unwrap();
            let out = f::inbox_entries(&mut f.root, watch, rows);
            let reads = inbox_reads(&out);
            up(
                &mut f.root,
                Up::ReadDone { read: reads[0].0, result: Readout::Jig(c::ReadResult::Value(f::acts::question())) },
            );
        }
        for project in PROJECTS {
            let watched = match f.root.pages().decisions(project) {
                Document::Unwatched => false,
                Document::Unknown | Document::Delivered(_) => true,
                Document::Ended => panic!("wanted project {project} was refused"),
            };
            assert_eq!(watched, expected.contains(&project), "{address:?}: project {project}");
        }
    }
    for address in [
        Address::Inbox { filter: Filter { project: Some(f::PROJECT), kind: None, goal: None } },
        Address::Inbox { filter: Filter { project: None, kind: None, goal: None } },
    ] {
        let mut f = fixture();
        decisions(&mut f, &[decision(1, 100)]);
        let reads = task_reads(&decisions(&mut f, &[decision(2, 100)]));
        let out = event(&mut f.root, Event::Went { place: Place::Address(address) });
        assert!(opened(&out, &forge::decided::watch(f::PROJECT, &f::limits().forge).unwrap()).is_none());
        assert!(task_reads(&out).is_empty(), "the shown task read stays");
        for request in &out {
            if let Request::Unlinked { request: jig_client::UnlinkedRequest::Down { down: Down::Close { watch } } } =
                request
            {
                assert_ne!(*watch, f.decided, "wanted project keeps its watch");
            }
        }
        finish_closes(&mut f, &out);
        let mut task = f::chat::task();
        task.chip.task = 2;
        up(
            &mut f.root,
            Up::ReadDone {
                read: reads[0].1,
                result: Readout::Jig(c::ReadResult::Value(c::ReadValue::Task(Box::new(task)))),
            },
        );
        assert_eq!(
            f.root.pages().decided(None, Wall::from_nanos(100)).unwrap().rows[0].state,
            pages::decided::DecisionTask::Live
        );
        let out = event(&mut f.root, Event::Went { place: Place::Address(Address::Settings { project: f::PROJECT }) });
        assert!(f.root.pages().decided(None, Wall::from_nanos(100)).is_none());
        finish_closes(&mut f, &out);
        let out = go(&mut f.root, f::PROJECT);
        let next = opened(&out, &forge::decided::watch(f::PROJECT, &f::limits().forge).unwrap()).unwrap();
        assert_ne!(next, f.decided);
    }
}
#[test]
fn rows_show_for_twenty_four_hours() {
    let mut f = fixture();
    decisions(&mut f, &[decision(1, 100)]);
    assert!(away_form(&f, 99).is_some(), "a clock behind still shows it");
    assert!(away_form(&f, 100).is_some());
    assert!(away_form(&f, DAY + 99).is_some());
    assert!(away_form(&f, DAY + 100).is_none());
    decisions(&mut f, &[decision(2, u64::MAX)]);
    assert!(away_form(&f, u64::MAX).is_some(), "expiry arithmetic cannot wrap");
}
#[test]
#[expect(clippy::too_many_lines, reason = "one ordering test covers project merges and the rows' decoded reaches")]
fn the_seven_newest_are_kept_oldest_first() {
    let mut f = fixture();
    let values = [
        decision(2, 20),
        decision(8, 80),
        decision(1, 10),
        decision(9, 90),
        decision(3, 30),
        decision(7, 70),
        decision(4, 40),
        decision(6, 60),
        decision(5, 50),
    ];
    decisions(&mut f, &values);
    let form = away_form(&f, 100).unwrap();
    assert_eq!(decision_numbers(&form.rows).as_ref(), [3, 4, 5, 6, 7, 8, 9]);
    let row = &form.rows[0];
    assert_eq!(row.task, 3);
    assert_eq!(row.at, Wall::from_nanos(30));
    assert_eq!(row.question.as_ref(), b"Which approach?");
    assert_eq!(row.took.as_ref(), b"The smaller change");
    assert_eq!(row.why.as_ref(), b"It fits the goal.");
    assert!(!row.overturned);
    let mut parts = decision(10, 100).into_parts();
    parts.overturned = true;
    decisions(&mut f, &[w::Decision::new(&w::CEILINGS, parts).unwrap()]);
    assert!(away_form(&f, 100).unwrap().rows[0].overturned);
    for (reason, expected) in [
        (w::ReachReason::Kind, true),
        (w::ReachReason::Set(w::ReachSet::new(&w::CEILINGS, w::ReachSetParts { number: 0 }).unwrap()), false),
    ] {
        let mut parts = decision(1, 100).into_parts();
        let mut reach = parts.reach.clone().into_parts();
        reach.reason = reason;
        parts.reach = w::Reach::new(&w::CEILINGS, reach).unwrap();
        decisions(&mut f, &[w::Decision::new(&w::CEILINGS, parts.clone()).unwrap()]);
        assert_eq!(
            away_form(&f, 100).unwrap().rows[0].reach,
            if expected {
                Some(temper_protocol_reaches::Reach {
                    kind: temper_protocol_reaches::Kind::Wording,
                    class: temper_protocol_reaches::Class::Decide,
                    reason: temper_protocol_reaches::Reason::Kind,
                    under: 0,
                    sets: 0,
                    matched: 0,
                    away: false,
                })
            } else {
                None
            }
        );
        assert_eq!(away_form(&f, 100).unwrap().rows[0].reach.is_some(), expected);
    }
    let mut me = f::chats::me();
    let mut other = me.projects.get(0).unwrap().clone();
    other.project = OTHER_PROJECT;
    me.projects = List::from_boxed(Box::new([me.projects.get(0).unwrap().clone(), other]));
    let mut f = fixture_with_me(me);
    let out = event(
        &mut f.root,
        Event::Went { place: Place::Address(Address::Inbox { filter: crate::address::Filter::ALL }) },
    );
    finish_closes(&mut f, &out);
    let watch = opened(&out, &forge::decided::watch(OTHER_PROJECT, &f::limits().forge).unwrap()).unwrap();
    decisions(&mut f, &values);
    let mut document = w::Decisions::new(
        &w::CEILINGS,
        w::DecisionsParts {
            project: OTHER_PROJECT,
            rows: List::from_boxed(Box::new([decision(1, 95), decision(2, 96)])),
            asked: List::with_capacity(0),
            cut: true,
        },
    )
    .unwrap();
    up(
        &mut f.root,
        Up::FamilyDelivered {
            watch,
            record: jig_client::Family {
                family: wire::FORGE,
                version: wire::CLIENT_VERSION,
                bytes: wire::decisions(&document),
            },
        },
    );
    let form = f.root.pages().decided(None, Wall::from_nanos(100)).unwrap();
    assert_eq!(form.showing, 11);
    assert!(form.more);
    let mut pairs = List::uncapped();
    for row in &form.rows {
        pairs.push((row.project, row.number)).unwrap();
    }
    assert_eq!(
        pairs.as_slice(),
        &[
            (f::PROJECT, 5),
            (f::PROJECT, 6),
            (f::PROJECT, 7),
            (f::PROJECT, 8),
            (f::PROJECT, 9),
            (OTHER_PROJECT, 1),
            (OTHER_PROJECT, 2)
        ]
    );
    let mut parts = document.into_parts();
    parts.cut = false;
    document = w::Decisions::new(&w::CEILINGS, parts).unwrap();
    up(
        &mut f.root,
        Up::FamilyDelivered {
            watch,
            record: jig_client::Family {
                family: wire::FORGE,
                version: wire::CLIENT_VERSION,
                bytes: wire::decisions(&document),
            },
        },
    );
    assert!(!f.root.pages().decided(None, Wall::from_nanos(100)).unwrap().more);
}
#[test]
fn showing_counts_past_seven() {
    let mut f = fixture();
    let values = nine_decisions();
    decisions(&mut f, &values);
    assert_eq!(away_form(&f, 100).unwrap().showing, 9);
    assert_eq!(away_form(&f, DAY + 1).unwrap().showing, 8);
}
#[test]
fn expires_is_when_the_oldest_stops_showing() {
    let mut f = fixture();
    let values = nine_decisions();
    decisions(&mut f, &values);
    assert_eq!(away_form(&f, 100).unwrap().expires, Wall::from_nanos(DAY + 1));
    assert_eq!(away_form(&f, DAY + 1).unwrap().expires, Wall::from_nanos(DAY + 2));
}
#[test]
fn no_form_while_to_begin_shows() {
    let mut f = fixture();
    ready(&mut f);
    spell(&mut f, None, None, 0);
    let out = decisions(&mut f, &[decision(1, 100)]);
    assert!(away_form(&f, 100).is_none());
    assert_eq!(home::facts(&f.root, f::PROJECT, Wall::from_nanos(100)).spell, None);
    assert!(task_reads(&out).is_empty());
    let (_, out) = skip(&mut f);
    assert!(away_form(&f, 100).is_some());
    assert_eq!(task_reads(&out).len(), 1);
}
#[test]
fn spell_does_not_require_a_showing_decision() {
    for (off, until, wall, end, recent) in [
        (None, None, 100, None, true),
        (Some(150), None, 150 + DAY - 1, Some(150), true),
        (Some(150), None, 150 + DAY, Some(150), false),
        (None, Some(200), 200 + DAY - 1, Some(200), true),
        (None, Some(200), 200 + DAY, Some(200), false),
    ] {
        let mut f = fixture();
        status_document(&mut f, &crate::tests::status::document(9, 2, None));
        spell(&mut f, off, until, 3);
        let facts = home::facts(&f.root, f::PROJECT, Wall::from_nanos(wall));
        assert!(facts.decided.is_none());
        assert_eq!(
            facts.spell,
            if recent {
                Some(home::Spell {
                    since: Wall::from_nanos(10),
                    end: match end {
                        Some(end) => Some(Wall::from_nanos(end)),
                        None => None,
                    },
                    landed: Some(6),
                })
            } else {
                None
            }
        );
    }
}
#[test]
fn span_is_since_and_its_end() {
    const HOUR: u64 = 3_600_000_000_000;
    for (delivered, off, until, wall, expected) in [
        (false, None, None, 100, None),
        (true, None, None, 100, Some(None)),
        (true, None, Some(200), 199, Some(None)),
        (true, None, Some(200), 200, Some(Some(200))),
        (true, Some(150), Some(200), 201, Some(Some(150))),
        (true, Some(150), None, 150 + 23 * HOUR, Some(Some(150))),
        (true, Some(150), None, 150 + 25 * HOUR, None),
    ] {
        let mut f = fixture();
        decisions(&mut f, &[decision(1, wall)]);
        if delivered {
            spell(&mut f, off, until, 0);
        }
        let facts = home::facts(&f.root, f::PROJECT, Wall::from_nanos(wall));
        assert!(facts.decided.is_some());
        assert_eq!(
            match facts.spell {
                Some(spell) => Some((spell.since, spell.end)),
                None => None,
            },
            match expected {
                Some(end) => Some((
                    Wall::from_nanos(10),
                    match end {
                        Some(at) => Some(Wall::from_nanos(at)),
                        None => None,
                    }
                )),
                None => None,
            },
            "{delivered} {off:?} {until:?} {wall}"
        );
    }
}
fn spell_landed(f: &Fixture) -> Option<u64> {
    home::facts(&f.root, f::PROJECT, Wall::from_nanos(100)).spell?.landed
}
#[test]
fn landed_is_since_the_rows_count() {
    let mut f = fixture();
    decisions(&mut f, &[decision(1, 100)]);
    assert_eq!(spell_landed(&f), None);
    status_document(&mut f, &crate::tests::status::document(9, 2, None));
    assert_eq!(spell_landed(&f), None);
    spell(&mut f, None, None, 3);
    assert_eq!(spell_landed(&f), Some(6), "uses status even before looked");
    spell(&mut f, None, None, 10);
    assert_eq!(spell_landed(&f), Some(0));
    up(&mut f.root, Up::WatchEnded { watch: f.status, end: c::WatchEnd::NotAllowed });
    assert_eq!(spell_landed(&f), None);
}
#[test]
fn each_row_reads_its_task() {
    let mut limits = f::limits();
    limits.client.domain.reads = 32;
    let mut bounded = fixture_with_limits(f::chats::me(), limits);
    assert_eq!(bounded.root.pages().inbox().unwrap().cards.capacity(), 16);
    let mut out = Queue::uncapped();
    let values = nine_decisions();
    crate::step(
        &mut bounded.root,
        &f::env(&limits),
        Event::Client {
            event: jig_client::Event::Up {
                up: Up::FamilyDelivered { watch: bounded.decided, record: crate::tests::decided::record(&values) },
            },
        },
        &mut out,
    );
    assert_eq!(read_numbers(&task_reads(&out)).as_ref(), [3, 4, 5, 6, 7, 8, 9], "only the seven drawn tasks read");
    let mut f = enter_skipped();
    let values = [decision(1, 10), decision(2, 20)];
    let out = decisions(&mut f, &values);
    let reads = task_reads(&out);
    assert_eq!(read_numbers(&reads).as_ref(), [1, 2]);
    for row in &away_form(&f, 100).unwrap().rows {
        assert_eq!(row.title, None);
        assert_eq!(row.state, pages::decided::DecisionTask::Unknown);
    }
    for (task, read) in reads {
        let mut document = f::chat::task();
        document.chip.task = task;
        document.chip.title =
            c::DocumentText::complete(c::Text::new(b"A useful task title".as_slice().into(), 64).unwrap());
        let title = skein_lib::bytes::copy_of(document.chip.title.bytes());
        up(
            &mut f.root,
            Up::ReadDone { read, result: Readout::Jig(c::ReadResult::Value(c::ReadValue::Task(Box::new(document)))) },
        );
        let form = away_form(&f, 100).unwrap();
        let mut found = false;
        for row in &form.rows {
            if row.task == task {
                assert_eq!(row.title.as_deref(), Some(title.as_ref()));
                assert_eq!(row.state, pages::decided::DecisionTask::Live);
                found = true;
            }
        }
        assert!(found, "the read task has a row");
    }
    assert!(task_reads(&decisions(&mut f, &values)).is_empty(), "same document does not reread");
    assert_eq!(read_numbers(&task_reads(&decisions(&mut f, &[values[0].clone(), decision(3, 30)]))).as_ref(), [3]);
    let stale = task_reads(&decisions(&mut f, &[decision(4, 40)]))[0].1;
    let out = event(&mut f.root, Event::Went { place: Place::Address(Address::Settings { project: f::PROJECT }) });
    finish_closes(&mut f, &out);
    let out = go(&mut f.root, f::PROJECT);
    f.decided = opened(&out, &forge::decided::watch(f::PROJECT, &f::limits().forge).unwrap()).unwrap();
    let read = task_reads(&decisions(&mut f, &[decision(4, 40)]))[0].1;
    assert_ne!(read, stale, "reentry reads again");
    let mut wrong = f::chat::task();
    wrong.chip.task = 999;
    up(
        &mut f.root,
        Up::ReadDone { read, result: Readout::Jig(c::ReadResult::Value(c::ReadValue::Task(Box::new(wrong)))) },
    );
    assert_eq!(away_form(&f, 100).unwrap().rows[0].state, pages::decided::DecisionTask::Unknown);
}
#[test]
fn a_read_ended_or_done_says_so() {
    for (done, retained) in [(false, false), (true, false), (false, true), (true, true)] {
        let mut f = enter_skipped();
        let out = decisions(&mut f, &[decision(1, 10)]);
        let read = task_reads(&out)[0].1;
        let mut task = f::chat::task();
        task.chip.task = 1;
        f::chat::phase(&mut task, c::ChipPhase::Ended);
        task.chip.details.details.ended_at = Some(Wall::from_nanos(20));
        task.chip.details.details.ending = Some(if done {
            c::ChipEnding::Done(c::ResultValue::Words(c::DocumentText::complete(
                c::Text::new(b"Done".as_slice().into(), 64).unwrap(),
            )))
        } else {
            c::ChipEnding::Failed(c::DocumentText::complete(c::Text::new(b"Stopped".as_slice().into(), 64).unwrap()))
        });
        let value = if retained {
            let result = c::Result {
                task: task.chip.clone(),
                ending: c::Ending::Done,
                value: None,
                at: Some(Wall::from_nanos(30)),
                details: None,
                acts: List::with_capacity(0),
            };
            c::ReadValue::Ended(Box::new(c::Ended {
                key: c::ObjectKey::Task(1),
                at: Some(Wall::from_nanos(30)),
                by: None,
                value: c::EndedValue::Task { document: Box::new(task), result: Box::new(result) },
            }))
        } else {
            c::ReadValue::Task(Box::new(task))
        };
        up(&mut f.root, Up::ReadDone { read, result: Readout::Jig(c::ReadResult::Value(value)) });
        let at = Wall::from_nanos(if retained { 30 } else { 20 });
        assert_eq!(
            away_form(&f, 100).unwrap().rows[0].state,
            if done { pages::decided::DecisionTask::Done(at) } else { pages::decided::DecisionTask::Ended(at) }
        );
    }
}
#[test]
fn a_limit_leaves_it_unknown() {
    for page in [
        Readout::Limit,
        Readout::NotServed { family: None, served: None },
        Readout::Jig(c::ReadResult::None(c::ReadNone::Unknown)),
    ] {
        let mut f = enter_skipped();
        let read = task_reads(&decisions(&mut f, &[decision(1, 10)]))[0].1;
        up(&mut f.root, Up::ReadDone { read, result: page });
        let form = away_form(&f, 100).unwrap();
        assert_eq!(form.rows[0].state, pages::decided::DecisionTask::Unknown);
        assert_eq!(form.rows[0].title, None);
        assert!(task_reads(&decisions(&mut f, &[decision(1, 10)])).is_empty());
    }
}

fn finish_closes(f: &mut Fixture, out: &Queue<Request>) {
    for request in out {
        if let Request::Unlinked { request: jig_client::UnlinkedRequest::Down { down: Down::Close { watch } } } =
            request
        {
            up(&mut f.root, Up::WatchEnded { watch: *watch, end: c::WatchEnd::Closed });
        }
    }
}

fn nine_decisions() -> Box<[w::Decision]> {
    let mut values = List::with_capacity(9);
    for n in 1..=9 {
        values.push(decision(n, u64::from(n))).unwrap();
    }
    values.into_boxed()
}
fn decision_numbers(rows: &[pages::decided::DecisionLine]) -> Box<[u32]> {
    let mut values = List::uncapped();
    for row in rows {
        values.push(row.number).unwrap();
    }
    values.into_boxed()
}
fn read_numbers(reads: &[(u64, Token)]) -> Box<[u64]> {
    let mut values = List::uncapped();
    for (task, _) in reads {
        values.push(*task).unwrap();
    }
    values.into_boxed()
}

fn overturn_limits() -> crate::Limits {
    let mut limits = f::limits();
    limits.client.domain.text = 512;
    limits
}
fn overturn_fixture() -> Fixture {
    let mut f = enter_skipped();
    let mut second = decision(2, 0).into_parts();
    second.question = b"Which retry?".as_slice().into();
    second.took = b"Split it".as_slice().into();
    let reads = task_reads(&decisions(&mut f, &[decision(1, 0), w::Decision::new(&w::CEILINGS, second).unwrap()]));
    for (task, read) in reads {
        let mut document = f::chat::task();
        document.chip.task = task;
        up(
            &mut f.root,
            Up::ReadDone { read, result: Readout::Jig(c::ReadResult::Value(c::ReadValue::Task(Box::new(document)))) },
        );
    }
    f
}
fn overturn_act(f: &mut Fixture, number: u32) -> (jig_client::Key, Queue<Request>) {
    let key = jig_client::Key {
        asker: crate::owner::Owner::Pages.asker(),
        number: f.root.pages().serial.checked_add(1).unwrap(),
    };
    let mut out = Queue::uncapped();
    crate::step(
        &mut f.root,
        &f::env(&overturn_limits()),
        Event::Act { action: Action::Pages(Act::Away(pages::away::Act::Overturn { project: f::PROJECT, number })) },
        &mut out,
    );
    (key, out)
}
fn overturn_answer(f: &mut Fixture, key: jig_client::Key, outcome: jig_client::Outcome) -> Queue<Request> {
    let mut out = Queue::uncapped();
    crate::route::told(&mut f.root, &f::env(&overturn_limits()), Told::Answered { key, outcome }, &mut out);
    out
}
fn sent(out: &Queue<Request>) -> Box<[jig_client::Ask]> {
    let mut sends = List::uncapped();
    for request in out {
        if let Request::Unlinked { request: jig_client::UnlinkedRequest::Down { down: Down::Send { request, .. } } } =
            request
        {
            sends.push(request.clone()).unwrap();
        }
    }
    sends.into_boxed()
}
fn overturn_message(f: &mut Fixture, key: jig_client::Key) -> (jig_client::Key, Queue<Request>) {
    let next = jig_client::Key { asker: key.asker, number: f.root.pages().serial.checked_add(1).unwrap() };
    (next, overturn_answer(f, key, family_answer(w::Outcome::Overturned)))
}
#[test]
fn overturn_sends_the_request_then_the_message() {
    for address in [
        Address::Home { project: f::PROJECT },
        Address::Inbox { filter: crate::address::Filter { project: Some(f::PROJECT), ..crate::address::Filter::ALL } },
        Address::Inbox { filter: crate::address::Filter::ALL },
    ] {
        let mut f = overturn_fixture();
        let out = event(&mut f.root, Event::Went { place: Place::Address(address) });
        finish_closes(&mut f, &out);
        let (key, out) = overturn_act(&mut f, 1);
        assert_eq!(sent(&out).as_ref(), [forge::decided::overturn(f::PROJECT, 1, &f::limits().forge).unwrap()]);
        let (message, out) = overturn_message(&mut f, key);
        assert_eq!(sent(&out).len(), 1);
        assert_ne!(key, message);
        assert!(f.root.pages().overturn_busy());
        assert!(f.root.pages().decided(None, Wall::EPOCH).unwrap().rows[0].overturned);
        overturn_answer(
            &mut f,
            message,
            jig_client::Outcome::Answered(jig_client::Answered::Jig(c::Answer::Done(c::Outcome::Document(
                c::DocumentOutcome::Said { task: 1, message: 9 },
            )))),
        );
        assert!(!f.root.pages().overturn_busy());
        assert_eq!(f.root.pages().needs().toast(), Some(&pages::needs::Toasted::Overturned));
    }
}
#[test]
fn the_message_says_question_and_took() {
    let mut f = overturn_fixture();
    let original = decision(1, 0);
    let (key, _) = overturn_act(&mut f, 1);
    decisions(&mut f, &[decision(2, 0)]);
    let (_, out) = overturn_message(&mut f, key);
    assert_eq!(
        sent(&out).as_ref(),
        [jig_client::Ask::Jig(Box::new(c::Ask::Message {
            project: f::PROJECT,
            task: 1,
            words: c::Text::new(
                skein_lib::bytes::concat(&[
                    b"I overturn your decision. ",
                    original.question(),
                    b" You took ",
                    original.took(),
                    b". Ask me again with message, as a question, and wait for my answer.",
                ]),
                512
            )
            .unwrap(),
        }))]
    );
    let question = [b'q'; 400];
    let took = [b't'; 120];
    let words = forge::decided::overturn_words(&question, &took);
    assert_eq!(words.len(), 512);
    c::Text::new(words, 512).expect("maximum overturn fits the deployed message bound");
    let unicode = "é".repeat(200);
    let words = forge::decided::overturn_words(unicode.as_bytes(), &took);
    assert!(words.len() <= 512);
    assert!(skein_lib::bytes::is_utf8(&words));
    assert!(words.ends_with(&skein_lib::bytes::concat(&[
        &took,
        b". Ask me again with message, as a question, and wait for my answer."
    ])));
}
#[test]
fn a_refused_message_leaves_the_row_and_toasts() {
    let mut f = overturn_fixture();
    let (key, _) = overturn_act(&mut f, 1);
    let (message, _) = overturn_message(&mut f, key);
    overturn_answer(&mut f, message, jig_client::Outcome::NotKept);
    assert!(away_form(&f, 0).unwrap().rows[0].overturned);
    assert_eq!(f.root.pages().needs().toast(), Some(&pages::needs::Toasted::OverturnEnded));
    assert!(sent(&overturn_act(&mut f, 1).1).is_empty());
}
#[test]
fn a_refused_overturn_sends_no_message() {
    for outcome in [
        family_answer(w::Outcome::OverturnRefused(
            w::OverturnRefused::new(&w::CEILINGS, w::OverturnRefusedParts { reason: w::OverturnRefusal::Already })
                .unwrap(),
        )),
        family_answer(w::Outcome::OverturnRefused(
            w::OverturnRefused::new(&w::CEILINGS, w::OverturnRefusedParts { reason: w::OverturnRefusal::Unknown })
                .unwrap(),
        )),
        family_answer(w::Outcome::Unknown),
        jig_client::Outcome::Unknown,
    ] {
        let mut f = overturn_fixture();
        let (key, _) = overturn_act(&mut f, 1);
        assert!(sent(&overturn_answer(&mut f, key, outcome)).is_empty());
        assert!(!f.root.pages().overturn_busy());
        assert_eq!(f.root.pages().needs().toast(), Some(&pages::needs::Toasted::OverturnAlready));
    }
}
#[test]
fn a_second_act_waits() {
    let mut f = overturn_fixture();
    let (key, _) = overturn_act(&mut f, 1);
    for number in [1, 2] {
        assert!(sent(&overturn_act(&mut f, number).1).is_empty());
    }
    let (message, _) = overturn_message(&mut f, key);
    assert!(sent(&overturn_act(&mut f, 2).1).is_empty());
    overturn_answer(&mut f, message, jig_client::Outcome::NotKept);
    assert_eq!(sent(&overturn_act(&mut f, 2).1).len(), 1);
}
#[test]
fn leaving_home_still_sends_it() {
    let mut f = overturn_fixture();
    let (key, _) = overturn_act(&mut f, 1);
    event(&mut f.root, Event::Went { place: Place::Address(Address::Settings { project: f::PROJECT }) });
    assert!(f.root.pages().home().is_none());
    let (message, out) = overturn_message(&mut f, key);
    assert_eq!(sent(&out).len(), 1);
    overturn_answer(&mut f, message, jig_client::Outcome::NotKept);
    assert_eq!(f.root.pages().needs().toast(), Some(&pages::needs::Toasted::OverturnEnded));
}
