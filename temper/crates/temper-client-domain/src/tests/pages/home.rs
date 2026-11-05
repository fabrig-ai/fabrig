//! Home's pure reading and owned watches (temper/docs/design/client/README.md, 3.5).
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
    inbox: Token,
    dismissed: Token,
}
fn fixture() -> Fixture {
    fixture_with_me(f::chats::me())
}
fn fixture_with_me(frame: c::Me) -> Fixture {
    let mut root = crate::unlinked(&f::limits(), f::SEED);
    let me = f::start(&mut root, List::with_capacity(EMPTY));
    let out = deliver(&mut root, me, c::Delivery::Me(Box::new(frame)));
    let dismissed = opened(&out, &forge::dismissed::watch(&f::limits().forge).unwrap()).unwrap();
    let forges = opened(&out, &forge::forges::watch(&f::limits().forge).unwrap()).unwrap();
    let settings = opened(&out, &Watchable::Jig(c::Subject::Settings)).unwrap();
    let out = go(&mut root, f::PROJECT);
    let inbox =
        opened(&out, &Watchable::Jig(c::Subject::Inbox { window: root.pages().inbox().unwrap().window })).unwrap();
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
    Fixture { root, settings, live: live.unwrap(), closed: closed.unwrap(), forges, inbox, dismissed }
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
        assert_eq!(
            Facts {
                configured,
                repository,
                chat_asked,
                needs: Needs::Unknown,
                delivered: true,
                skip: Skip::NotSkipped,
                skip_refused: false
            }
            .left(),
            left
        );
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
            home::facts(&f.root, f::PROJECT).repository,
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
fn another_persons_project_cannot_start_a_configuring_chat() {
    let mut f = fixture();
    event(&mut f.root, Event::Went { place: Place::Address(Address::Configure { project: OTHER_PROJECT }) });
    let out = event(
        &mut f.root,
        Event::Act { action: Action::Pages(Act::Starter { words: b"My change".as_slice().into() }) },
    );
    assert!(f.root.pages().chats().unwrap().start.is_none());
    let mut sent = false;
    for request in &out {
        if let Request::Unlinked { request: jig_client::UnlinkedRequest::Down { down: Down::Send { .. } } } = request {
            sent = true;
        }
    }
    assert!(!sent, "Configure's address never falls back to another project");
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
        assert!(closed, "the inbox watch closes");
        go(&mut f.root, f::PROJECT);
        assert_eq!(f.root.pages().home().unwrap().settings, settings, "settings is the session handle");
        assert!(f.root.pages().chats().unwrap().live.number > last, "no page handle is reused");
    }
}
#[test]
fn needs_are_unknown_until_delivered_then_count_the_needs_alone() {
    for cut in [false, true] {
        let mut f = fixture();
        assert_eq!(home::facts(&f.root, f::PROJECT).needs, Needs::Unknown);
        let out = f::inbox_entries(&mut f.root, f.inbox, List::with_capacity(EMPTY));
        assert_eq!(home::facts(&f.root, f::PROJECT).needs, Needs::Unknown, "queries still pending");
        let reads = inbox_reads(&out);
        let needs = inbox_page(&mut f.root, reads[0].0, need_rows(), cut);
        assert_eq!(home::facts(&f.root, f::PROJECT).needs, Needs::Unknown, "updates and card reads pending");
        let mut updates = List::with_capacity(ONE);
        updates.push(f::inbox_entry(c::ObjectKey::Result(CHAT), c::InboxGroup::Results, f::PROJECT, 0)).unwrap();
        let updates = inbox_page(&mut f.root, reads[1].0, updates, !cut);
        let needs = inbox_reads(&needs);
        let updates = inbox_reads(&updates);
        let cards = needs.iter().chain(updates.iter());
        for (read, _) in cards {
            assert_eq!(home::facts(&f.root, f::PROJECT).needs, Needs::Unknown, "each card must answer");
            up(&mut f.root, Up::ReadDone { read: *read, result: Readout::Limit });
        }
        assert_eq!(home::facts(&f.root, f::PROJECT).needs, Needs::Count { count: NEED_GROUPS, cut });
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
        assert_eq!(home::facts(&f.root, OTHER_PROJECT).needs, Needs::Unknown);
        let out = f::inbox_entries(&mut f.root, f.inbox, List::with_capacity(EMPTY));
        assert_eq!(home::facts(&f.root, f::PROJECT).needs, Needs::Unknown, "refresh waits again");
        for (read, _) in inbox_reads(&out) {
            inbox_page(&mut f.root, read, List::with_capacity(EMPTY), false);
        }
        assert_eq!(home::facts(&f.root, f::PROJECT).needs, Needs::Count { count: 0, cut: false });
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
        assert!(home::facts(&f.root, f::PROJECT).chat_asked);
        assert_eq!(home::facts(&f.root, f::PROJECT).needs, Needs::Count { count: 0, cut: true });
    }
}

const CONFIGURING: u32 = 3;
const TWO_TEMPLATES: u32 = 2;
const FACTS: u32 = 5;
// Live watch, read page's live rows, read page's ended rows.
const CHAT_SOURCES: u32 = 3;

fn configuring_me() -> c::Me {
    let mut me = f::chats::me();
    let project = me.projects.get_mut(0).unwrap();
    let stock = project.templates.get(0).unwrap().clone();
    let mut configuring = stock.clone();
    configuring.number = CONFIGURING;
    configuring.executor = c::TemplateExecutor::Charter(CONFIGURING);
    project.templates = List::with_capacity(TWO_TEMPLATES);
    project.templates.push(stock).unwrap();
    project.templates.push(configuring).unwrap();
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
            .push(w::Dismissal::ToBegin(w::ToBegin::new(&w::CEILINGS, w::ToBeginParts { project: *project }).unwrap()))
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
    assert!(home::facts(&f.root, f::PROJECT).to_begin());
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
fn a_configuring_chat_marks_the_first_step_and_not_the_third() {
    for source in 0..CHAT_SOURCES {
        let mut f = fixture_with_me(configuring_me());
        let mut row = super::chats::chat_line();
        row.chip.executor = c::ChipExecutor::Agent { charter: CONFIGURING };
        let mut rows = List::with_capacity(ONE);
        rows.push(row).unwrap();
        if source == 0 {
            deliver(&mut f.root, f.live, c::Delivery::Tasks(Box::new(c::Tasks { rows, more: false })));
        } else {
            chat_page(&mut f, rows, source == 1, None);
        }
        let facts = home::facts(&f.root, f::PROJECT);
        assert!(facts.configured, "source={source}");
        assert!(!facts.chat_asked, "configuring alone source={source}");
        assert!(!home::facts(&f.root, OTHER_PROJECT).configured);
    }
}
#[test]
fn a_stock_chat_marks_the_third_step_alone() {
    for me in [f::chats::me(), configuring_me()] {
        for source in 0..CHAT_SOURCES {
            let mut f = fixture_with_me(me.clone());
            if source == 0 {
                deliver(&mut f.root, f.live, c::Delivery::Tasks(Box::new(c::Tasks { rows: chat_rows(), more: false })));
            } else {
                chat_page(&mut f, chat_rows(), source == 1, None);
            }
            let facts = home::facts(&f.root, f::PROJECT);
            assert!(!facts.configured);
            assert!(facts.chat_asked);
        }
    }
    let mut chosen = configuring_me();
    chosen.projects.get_mut(0).unwrap().templates.get_mut(1).unwrap().executor = c::TemplateExecutor::Chosen;
    assert!(!pages::chats::configuring(&chosen, f::PROJECT, &c::ChipExecutor::Agent { charter: CONFIGURING }));
    for source in 0..CHAT_SOURCES {
        let mut f = fixture_with_me(configuring_me());
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
        let facts = home::facts(&f.root, f::PROJECT);
        assert!(!facts.configured, "a configuring chat past the cut isn't seen");
        assert!(facts.chat_asked, "a cut counts as another chat");
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
        let facts = home::facts(&f.root, f::PROJECT);
        assert!(!facts.to_begin(), "withheld={withheld}");
        assert_eq!(facts.delivered, withheld == 4);
        deliver_fact(&mut f, withheld);
        assert!(home::facts(&f.root, f::PROJECT).to_begin());
    }
    for ended in [c::WatchEnd::NotAllowed, c::WatchEnd::SubjectEnded] {
        let mut f = fixture();
        for watch in [f.settings, f.live, f.forges] {
            up(&mut f.root, Up::WatchEnded { watch, end: ended });
        }
        up(&mut f.root, Up::ReadDone { read: f.closed, result: Readout::Limit });
        dismissal_rows(&mut f, &[]);
        let facts = home::facts(&f.root, f::PROJECT);
        assert!(facts.delivered);
        assert_eq!(facts.left(), THREE_STEPS);
        assert!(facts.to_begin());
        up(&mut f.root, Up::WatchEnded { watch: f.dismissed, end: ended });
        assert_eq!(home::facts(&f.root, f::PROJECT).skip, Skip::Unknown);
        assert!(!home::facts(&f.root, f::PROJECT).to_begin());
    }
    let mut f = fixture_with_me(configuring_me());
    ready(&mut f);
    let mut rows = chat_rows();
    rows.get_mut(0).unwrap().chip.executor = c::ChipExecutor::Agent { charter: CONFIGURING };
    deliver(&mut f.root, f.live, c::Delivery::Tasks(Box::new(c::Tasks { rows, more: false })));
    assert!(home::facts(&f.root, f::PROJECT).configured);
    up(&mut f.root, Up::WatchEnded { watch: f.live, end: c::WatchEnd::SubjectEnded });
    assert!(!home::facts(&f.root, f::PROJECT).configured, "ended live read leaves its step");
    let mut facts = home::facts(&f.root, f::PROJECT);
    facts.configured = true;
    facts.chat_asked = true;
    facts.repository = Repository::In { owner: b"work".as_slice().into(), name: b"hello".as_slice().into() };
    assert!(!facts.to_begin(), "nothing left");
}
fn deliver_fact(f: &mut Fixture, fact: u32) {
    match fact {
        0 => rows(&mut f.root, f.forges, Box::default()),
        1 => settings(f, c::SettingFollows::Default),
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
        jig_client::Outcome::Full { held: ONE, room_at: skein_lib::Wall::EPOCH },
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
                assert_eq!(
                    *request,
                    forge::dismissed::ask(
                        forge::dismissed::Dismissal::ToBegin { project: f::PROJECT },
                        &f::limits().forge
                    )
                    .unwrap()
                );
                sends += 1;
            }
        }
        assert_eq!(sends, 1);
        assert_eq!(home::facts(&f.root, f::PROJECT).skip, Skip::Skipped);
        assert!(event(&mut f.root, Event::Act { action: Action::Pages(Act::SkipToBegin) }).is_empty());
        assert_eq!(f.root.pages().serial, key.number, "one press, one key");
        skip_answer(&mut f, jig_client::Key { number: key.number + 1, ..key }, outcome.clone());
        assert_eq!(home::facts(&f.root, f::PROJECT).skip, Skip::Skipped, "only our answer");
        skip_answer(&mut f, key, outcome);
        let facts = home::facts(&f.root, f::PROJECT);
        assert!(facts.skip_refused);
        assert_eq!(facts.skip, Skip::NotSkipped);
        assert!(facts.to_begin());
        let (next, _) = skip(&mut f);
        assert_ne!(next, key);
        assert!(!home::facts(&f.root, f::PROJECT).skip_refused);
        dismissal_rows(&mut f, &[f::PROJECT]);
        skip_answer(&mut f, next, jig_client::Outcome::Unknown);
        assert_eq!(home::facts(&f.root, f::PROJECT).skip, Skip::Skipped, "subject answers unknown");
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
    assert_eq!(home::facts(&f.root, f::PROJECT).skip, Skip::Skipped);
    assert!(!home::facts(&f.root, f::PROJECT).to_begin());
    event(&mut f.root, Event::Went { place: Place::Address(Address::Settings { project: f::PROJECT }) });
    assert!(event(&mut f.root, Event::Act { action: Action::Pages(Act::SkipToBegin) }).is_empty());
    go(&mut f.root, f::PROJECT);
    assert_eq!(home::facts(&f.root, f::PROJECT).skip, Skip::NotSkipped, "acknowledgement lasts until leaving Home");
    for outcome in [jig_client::Outcome::Unknown, family_answer(w::Outcome::Unknown)] {
        let (key, _) = skip(&mut f);
        skip_answer(&mut f, key, outcome);
        let facts = home::facts(&f.root, f::PROJECT);
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
    assert_eq!(home::facts(&f.root, f::PROJECT).skip, Skip::NotSkipped);
    assert!(home::facts(&f.root, f::PROJECT).to_begin());
    dismissal_rows(&mut f, &[f::PROJECT]);
    assert_eq!(home::facts(&f.root, f::PROJECT).skip, Skip::Skipped);
    assert!(!home::facts(&f.root, f::PROJECT).to_begin());
}

#[test]
fn skip_respects_its_page_and_request_bounds() {
    let mut f = fixture();
    ready(&mut f);
    let serial = f.root.pages().serial;
    let mut limits = f::limits();
    limits.forge.field_bytes = u32::MAX;
    assert!(
        forge::dismissed::ask(forge::dismissed::Dismissal::ToBegin { project: f::PROJECT }, &limits.forge).is_none()
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
    assert_eq!(home::facts(&f.root, f::PROJECT).skip, Skip::NotSkipped);
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
                forge::dismissed::Dismissal::ToBegin { project: f::PROJECT },
                &f::limits().forge
            )
            .unwrap(),
        }))
    );
    assert!(out.is_empty());
    assert_eq!(home::facts(&f.root, f::PROJECT).skip, Skip::Skipped);
    for address in
        [Address::Settings { project: f::PROJECT }, Address::Chats { filter: crate::ChatsFilter::All, start: false }]
    {
        event(&mut f.root, Event::Went { place: Place::Address(address) });
        let serial = f.root.pages().serial;
        assert!(event(&mut f.root, Event::Act { action: Action::Pages(Act::SkipToBegin) }).is_empty());
        assert_eq!(f.root.pages().serial, serial);
    }
}
