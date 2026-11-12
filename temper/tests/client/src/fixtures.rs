//! Named story values and one client profile, shared by every story.
pub mod b2;
#[cfg(test)]
mod tests;
use crate::{app::Temper, front::Joint};
use jig_client_kit::harness::FrontFault;
use jig_client_kit::harness::{Settings, World};
use jig_domain_tasks as tasks;
use skein_lib::{Duration, RequestLimits, Time, Token};
use std::{
    collections::BTreeMap,
    sync::{Mutex, OnceLock},
};
use temper_client_domain::{Limits, forge};
use temper_client_view as view;
use temper_engine_domain as engine;
use temper_engine_domain_world::direct::Driver;
pub const NOW: skein_lib::Wall = skein_lib::Wall::EPOCH;
pub const FORGE: u16 = temper_client_protocol_wire::number(temper_client_protocol_wire::Owner::Forge);
pub const FORGE_HOST: &[u8] = b"forge.example";
/// The stand speaks Forgejo, with walking's forge number, writer and git host.
#[must_use]
pub fn forge_settings() -> temper_service::forges::Forges {
    use temper_service::settings::{Forge, Kind, Transport};
    let mut base = b"http://".to_vec();
    base.extend_from_slice(FORGE_HOST);
    {
        let mut table = temper_service::forges::Forges::with_capacity(1);
        assert!(table.insert(Forge {
            log_origins: None,
            mode: temper_engine_domain_forge::Mode::Connected,
            number: 1,
            kind: Kind::Forgejo,
            api: jig_protocol_config::Address::Host { name: FORGE_HOST.into(), port: 80 },
            transport: Transport::Plaintext,
            git: base.clone().into(),
            git_program: None,
            web: Some(base.into()),
            app_read_only: false,
            user: 1,
            identity: 1,
        }));
        table
    }
}
pub const REPOSITORIES: &[(&[u8], &[u8], u32)] = &[(b"acme", b"shop", 7), (b"acme", b"second", 8)];
pub const PARTY: u64 = 1;
pub const SEED: u64 = 1101;
pub const WITHIN: Duration = Duration::from_secs(1);

/// Two owned projects, using To begin's second seeded project's shape
/// (temper/docs/design/testing/checks.md, 3; temper/docs/design/client/pages/README.md).
#[must_use]
pub fn across_configuration() -> (engine::Config, engine::Limits) {
    let mut config = temper_engine_domain_world::walking::config(SEED);
    let mut bounds = crate::front::engine_limits();
    bounds.people.initial_owners = 2;
    bounds.authority.projects = 2;
    bounds.people.projects = 3; // The deployment roster and the two projects.
    bounds.people.sign_in_lifetime = Duration::from_secs(2 * 60 * 60);
    bounds.people.requests = 16;
    bounds.tasks.tasks = 4; // Each project's chat and its questioning delegate.
    bounds.tasks.depth = 2; // The chat's delegate keeps its own delegation depth.
    bounds.people.inbox_tasks = 8; // The live and ended inbox scan for those four tasks.
    bounds.fleet.slots = 4; // Both chats and the two delegates can hear their questions.
    bounds.fleet.workstreams = 4;
    bounds.fleet.attempts = 4;
    bounds.call_records = 4; // Each project's delegate call and committed question.
    // Both questions, their answer credits and the overturn notification can coexist.
    bounds.tasks.inbox_bytes = bounds.tasks.message_bytes * 4;
    config.core.defaults.rules.ceiling.delegation.depth = 2;
    let policy = &mut config.core.seed_projects[0].policy.policy;
    policy.ceiling.delegation.depth = 2;
    policy.roles[0].authority.delegation.depth = 2;
    let policy = config.core.seed_projects[0].policy.clone();
    let mut projects = config.core.seed_projects.into_vec();
    projects.push(jig_domain_core::SeedProject { number: 2, name: "Project 2".into(), policy });
    config.core.seed_projects = projects.into_boxed_slice();
    let owner = config.core.owners[0].clone();
    config.core.owners = Box::new([owner.clone(), jig_domain_people::InitialOwner { project: 2, ..owner }]);
    config.core.connectors = Box::new([FORGE]);
    (config, bounds)
}

/// Real roots with room for work and questions on both owned projects.
#[must_use]
pub fn across_people(limits: Limits) -> Joint {
    let (config, bounds) = across_configuration();
    let bootstrap = temper_engine_domain_world::oracle::config::copy(&config);
    let budget = config.core.seed_projects[1].policy.policy.period_spend;
    let mut front = Joint::with_engine(
        SEED,
        &limits,
        temper_engine_domain_world::store::Store::with_limits(&bounds),
        [bootstrap, config],
        &bounds,
        4,
    );
    front.engine.send(engine::Event::Party(engine::PartyInput::Period { project: 2, period: 1, budget, next: None }));
    front.engine.settle();
    front
}
/// Eleven delegates fill one stream's room beside the chat's session watches and
/// Home, including each page's standing-role watch.
pub(crate) const WATCH_ROOM_DELEGATES: u32 = 11;

/// Page watch figures from temper-client-shell-web/src/limits.rs.
#[must_use]
pub fn watch_room_limits() -> Limits {
    let mut limits = domain_limits();
    // The shipped shells hold four projects' documents and the session's own watches.
    limits.client.domain.watches = 26;
    limits.client.domain.handles = 28;
    limits.client.domain.delegates = 80;
    limits.client.domain.reads = 32;
    limits.client.domain.rows = 16;
    limits
}

/// Two shipped page watch sets and a fresh client on the real engine.
#[must_use]
pub fn watch_room_world(seed: u64) -> World<Temper, Joint> {
    world_with(seed, Joint::with_watch_room(seed, &watch_room_limits()))
}

/// A watch-room story with a named deployment stream count.
#[must_use]
pub fn watch_room_world_streams(seed: u64, streams: u32) -> World<Temper, Joint> {
    world_with(seed, Joint::with_watch_room_streams(seed, &watch_room_limits(), streams))
}

/// A running chat with enough live delegates to fill its page's watch set.
pub fn watch_room_chat(front: &mut Joint) -> u64 {
    use temper_engine_domain_world::fixtures::report_delegate;
    let driver = &mut front.engine;
    driver.sign_in();
    driver.settle();
    let jig_domain_people::Ask::Create { project, mut batch } = driver.chat_ask(CHAT_PROJECT, b"A long chat") else {
        panic!("chat creation fixture");
    };
    batch[0].authority.delegation.tasks = WATCH_ROOM_DELEGATES;
    driver.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: skein_lib::ReplyTo::new(CHAT_REPLY),
        sign_in: driver.session(),
        key: CHAT_KEY,
        ask: jig_domain_people::Ask::Create { project, batch },
    }));
    driver.settle();
    let parent = driver.last_assignment();
    let batch = (0..WATCH_ROOM_DELEGATES).map(|_| report_delegate(b"Investigate the change", Box::new([]))).collect();
    driver.call(
        QUESTION_WORKER,
        parent.task,
        parent.attempt,
        DELEGATE_CALL,
        engine::Call {
            completion: DELEGATE_COMPLETION,
            position: DELEGATE_POSITION,
            tool: engine::Tool::Delegate { batch },
        },
    );
    driver.settle();
    let Some(engine::CallAnswer::Delegated(children)) = driver.answered(DELEGATE_CALL) else {
        panic!("watch-room delegates admitted: {:?}", driver.answered(DELEGATE_CALL));
    };
    assert_eq!(children.len(), usize::try_from(WATCH_ROOM_DELEGATES).expect("delegate count"));
    parent.task
}

/// Start a person root at its own page and transport limits.
pub fn watch_room_client(
    world: &mut World<Temper, Joint>,
    address: temper_client_domain::Address,
) -> jig_client_kit::harness::ClientId {
    let limits = watch_room_limits();
    world.front().next_client_limits(limits);
    world.client_with(PARTY, address, limits, watch_room_root)
}

fn watch_room_root(seed: u64) -> (crate::app::Root, crate::app::View) {
    let limits = watch_room_limits();
    let words = phrasebooks(&limits);
    let bounds = view_bounds(&limits, &words);
    (
        crate::app::Root::Person(Box::new(temper_client_domain::unlinked(&limits, seed))),
        crate::app::View::Person(Box::new(view::View::new(&bounds, &limits, words, FORGE))),
    )
}

#[must_use]
pub fn domain_limits() -> Limits {
    {
        let requests = RequestLimits {
            requests: 4,
            bytes: 262_144,
            out: 16,
            first: Duration::from_millis(10),
            most: Duration::from_millis(80),
            margin: Duration::from_secs(1),
        };
        let client = jig_client::Limits {
            domain: jig_client::DomainLimits {
                accounts: jig_client::AccountCounts { accounts: 16, sources: 8, sign_ins: 4 },
                owners: &[jig_client::Spoken {
                    family: temper_client_protocol_wire::FORGE,
                    version: temper_client_protocol_wire::CLIENT_VERSION,
                }],
                requests,
                ask_bytes: requests.bytes / u64::from(requests.requests),
                // Page handles, session settings and Away, project-needs and the goals watch/read, trials and a running trial task.
                handles: 20, // Holds four projects' decided watches and the open project's standing roles.
                // Closing page watches overlap the session watches.
                watches: 19, // Holds four projects' decided watches and the open project's standing roles.
                turn_bytes: 65536,
                backoff: jig_client::Backoff { first: Duration::from_millis(10), most: Duration::from_millis(80) },
                authority: 12,
                heartbeat: Duration::from_millis(100),
                // Project classes add one read beside the story's existing reads.
                reads: 9,
                parts: 8,
                delegates: 2,
                document_bytes: 65536,
                records: 2,
                kept: 4,
                projects: 4,
                // The linked world sends whole starters and cues, as both shipped shells do.
                text: temper_client_protocol_wire::CUE_BYTES,
                turn_text: temper_client_protocol_wire::CUE_BYTES,
                long_text: temper_client_protocol_wire::CUE_BYTES,
                rows: 4,
                fields: 4,
                segments: 9,
            },
            exchanges: 8,
        };

        Limits { forge: forge::Limits { field_bytes: 256, checks: 8 }, client, row_uses: 8, window_bytes: None }
    }
}

type Books = view::words::Words;
static BOOKS: OnceLock<Mutex<BTreeMap<u32, Books>>> = OnceLock::new();
pub fn phrasebooks(limits: &Limits) -> Books {
    let mut books = BOOKS.get_or_init(|| Mutex::new(BTreeMap::new())).lock().expect("fixture books lock");
    books
        .entry(limits.client.domain.text)
        .or_insert_with(|| {
            view::words::Words::english(FORGE, limits.client.domain.text, view::words::Shell::Terminal)
                .expect("temper words fit")
        })
        .clone()
}
#[must_use]
pub fn view_bounds(limits: &Limits, words: &view::words::Words) -> view::Limits {
    view::Limits::of(limits, words).expect("world view bound fits")
}
#[must_use]
pub fn inbox() -> temper_client_domain::Address {
    temper_client_domain::Address::Inbox { filter: temper_client_domain::address::Filter::ALL }
}
#[must_use]
pub fn world(seed: u64) -> World<Temper, Joint> {
    world_with(seed, Joint::new(seed, domain_limits()))
}
#[must_use]
pub fn tui_world(seed: u64) -> World<Temper, Joint> {
    world_with(seed, Joint::with_printed_link(seed, domain_limits()))
}
/// A front's first run offers the printed link beside the handed local link.
#[must_use]
pub fn tui_first_run_world(seed: u64) -> World<Temper, Joint> {
    let (mut front, _) = crate::front::account_acts::tui_first_run_fixture(seed);
    front.offer_printed_link(domain_limits());
    world_with(seed, front)
}
/// The launchpad asks Ask temper through the shipped TUI host.
#[must_use]
pub fn tui_asking_world(seed: u64) -> World<Temper, Joint> {
    world_with(seed, Joint::with_asking_printed_link(seed, domain_limits()))
}
/// A committed question in the second of two projects, with the shipped TUI's printed link.
#[must_use]
pub fn tui_question_world(seed: u64, words: &[u8]) -> (World<Temper, Joint>, u64) {
    let mut world = world_with(seed, across_people(domain_limits()));
    let chat = project_question(world.front(), 2, words);
    world.front().offer_printed_link(temper_client_shell_tui::limits::tui().native.client);
    (world, chat)
}
#[must_use]
pub fn adoption_world(seed: u64) -> World<Temper, Joint> {
    world_with(seed, Joint::adoption(seed, domain_limits()))
}
#[must_use]
pub fn open_world(seed: u64) -> World<Temper, Joint> {
    world_with(seed, Joint::open_chats(seed, domain_limits()))
}
#[must_use]
pub fn refused_hello_world(seed: u64) -> World<Temper, Joint> {
    world_with(seed, Joint::open_chats_with_a_refused_hello(seed, domain_limits()))
}
#[must_use]
pub fn world_with_large_turns(seed: u64) -> World<Temper, Joint> {
    world_with(seed, Joint::with_large_turns(seed, &domain_limits()))
}
#[must_use]
pub fn world_with_long_turns(seed: u64) -> World<Temper, Joint> {
    world_with(seed, Joint::with_long_turns(seed, &domain_limits()))
}

/// The shipped clients' whole-turn allowance, with a small world's row counts.
#[must_use]
pub fn world_with_whole_turns(seed: u64) -> World<Temper, Joint> {
    let limits = whole_turn_limits();
    world_with(seed, Joint::with_whole_turns(seed, &limits))
}

#[must_use]
pub fn whole_turn_limits() -> Limits {
    let mut limits = domain_limits();
    limits.client.domain.long_text = 131_072;
    limits.client.domain.turn_text = 131_072;
    limits.client.domain.document_bytes = 1_048_576;
    limits.client.domain.turn_bytes = 1_048_576;
    limits.client.domain.requests.bytes = 4_194_304;
    limits.client.domain.ask_bytes = 1_048_576;
    limits
}

/// Start a person with the same whole-turn bounds at every client layer.
pub fn whole_turn_client(world: &mut World<Temper, Joint>, task: u64) -> jig_client_kit::harness::ClientId {
    let limits = whole_turn_limits();
    world.front().next_client_limits(limits);
    world.client_with(PARTY, chat_page(task), limits, whole_turn_root)
}
fn whole_turn_root(seed: u64) -> (crate::app::Root, crate::app::View) {
    let limits = whole_turn_limits();
    let words = phrasebooks(&limits);
    let bounds = view_bounds(&limits, &words);
    (
        crate::app::Root::Person(Box::new(temper_client_domain::unlinked(&limits, seed))),
        crate::app::View::Person(Box::new(view::View::new(&bounds, &limits, words, FORGE))),
    )
}

/// Small pages and a byte window that drops whole turns within a short chat.
pub const READING_BYTES: u64 = 2048;
#[must_use]
pub fn reading_limits() -> Limits {
    let mut limits = whole_turn_limits();
    limits.client.domain.text = 256;
    limits.client.domain.long_text = 1024;
    limits.client.domain.turn_text = 1024;
    limits.client.domain.document_bytes = READING_BYTES;
    limits
}
#[must_use]
pub fn reading_world(seed: u64) -> World<Temper, Joint> {
    let limits = reading_limits();
    world_with(seed, Joint::with_reading_pages(seed, &limits))
}
pub fn reading_client(world: &mut World<Temper, Joint>, task: u64) -> jig_client_kit::harness::ClientId {
    let limits = reading_limits();
    world.front().next_client_limits(limits);
    world.client_with(PARTY, chat_page(task), limits, reading_root)
}
fn reading_root(seed: u64) -> (crate::app::Root, crate::app::View) {
    let limits = reading_limits();
    let mut words = phrasebooks(&limits);
    words.shell = view::words::Shell::Browser;
    let bounds = view_bounds(&limits, &words);
    (
        crate::app::Root::Person(Box::new(temper_client_domain::unlinked(&limits, seed))),
        crate::app::View::Person(Box::new(view::View::new(&bounds, &limits, words, FORGE))),
    )
}
/// Commit another whole assistant turn on the running chat's worker link.
pub fn reading_turn(front: &mut Joint, task: u64, words: &[u8]) {
    delegate_turn(front, task, None, words);
}

/// A live prefix for the current run's next turn, without committing it.
pub fn reading_piece(front: &mut Joint, task: u64, bytes: &[u8]) {
    let run = front.engine.task(task).expect("running chat");
    let (attempt, turn) = (run.attempt, run.turn + 1);
    front.engine.send(temper_engine_domain_world::protocol::up(
        QUESTION_WORKER,
        jig_domain_host::LinkUp::Piece {
            run: Token::new(task),
            attempt: Token::new(attempt),
            piece: jig_domain_host::Piece {
                turn,
                retry: 0,
                kind: jig_domain_host::LiveKind::Text,
                offset: 0,
                bytes: bytes.into(),
            },
        },
        engine::Parts::none(),
    ));
    front.engine.settle();
}

/// Jig's referee checks decision text, independently of the application's view
/// phrase shapes. Keep its complete jig vocabulary, replacing only the funding
/// decision that temper renders differently. Held cards use the view's own book.
#[must_use]
pub fn decision_words() -> skein_ui::phrases::Language {
    let mut language = jig_client::words::english();
    let funds = jig_client::words::Word::Decision(jig_client::words::DecisionWord::Funds).id();
    for phrase in view::words::jig_language().phrases {
        if phrase.id == funds {
            for target in &mut language.phrases {
                if target.id == funds {
                    *target = phrase.clone();
                }
            }
        }
    }
    language
}

fn world_with(seed: u64, front: Joint) -> World<Temper, Joint> {
    let mut world =
        World::new(Settings { seed, words: decision_words(), iterations: 4096, ..Settings::default() }, front);
    world.randomness(2, 0, 0);
    world
}
#[must_use]
pub fn codec() -> jig_protocol_transcript::Limits {
    jig_protocol_transcript::Limits::new(
        32768,
        jig_client::contract::TurnLimits { messages: 32, blocks: 32, text: 8192, outcome: 8192 },
    )
}

#[must_use]
pub fn chats_page() -> temper_client_domain::Address {
    temper_client_domain::Address::Chats { filter: temper_client_domain::ChatsFilter::All }
}
#[must_use]
pub fn chats(driver: &Driver) -> u32 {
    u32::try_from(
        driver
            .store
            .rows
            .values()
            .filter(|record| {
                matches!(
                    record,
                    engine::Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row) | tasks::Stored::Ended(row)))
                    if matches!(row.requester, tasks::Party::Person(_)) && row.tracked.is_none()
                )
            })
            .count(),
    )
    .expect("bounded chats")
}
#[must_use]
pub fn faults(seed: u64) -> Vec<(Time, FrontFault)> {
    let choices = [
        FrontFault::Restart { offline: Duration::from_millis(30) },
        FrontFault::NoAnswer,
        FrontFault::DropWatch,
        FrontFault::Busy,
        FrontFault::SignedOut,
    ];
    (0..4)
        .map(|index| {
            (
                Time::from_nanos(Duration::from_millis(20 * (index + 1)).as_nanos()),
                choices[usize::try_from((seed + index) % 5).expect("five faults")].clone(),
            )
        })
        .collect()
}
/// A chat made and held by two failed real worker attempts.
pub fn held_chat(front: &mut Joint) -> u64 {
    let driver = &mut front.engine;
    hello_worker(driver);
    driver.settle();
    driver.sign_in();
    driver.settle();
    driver.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: skein_lib::ReplyTo::new(Token::new(1001)),
        sign_in: driver.session(),
        key: [31; 16],
        ask: temper_engine_domain_world::walking::chat(1, b"Held chat"),
    }));
    driver.settle();
    let first = driver.last_assignment();
    driver.answer(
        Token::new(7),
        first.task,
        first.attempt,
        0,
        tasks::End::Failed(tasks::Class::Run),
        engine::Parts::none(),
    );
    driver.settle();
    driver.env.now = Time::from_nanos(Duration::from_secs(2).as_nanos());
    engine::fire(&mut driver.root, &driver.env);
    engine::release(&mut driver.root, &driver.env, &mut driver.out);
    driver.collect();
    driver.settle();
    let second = driver.last_assignment();
    driver.answer(
        Token::new(7),
        second.task,
        second.attempt,
        0,
        tasks::End::Failed(tasks::Class::Run),
        engine::Parts::none(),
    );
    driver.settle();
    assert!(matches!(driver.task(first.task).expect("held chat").phase, tasks::Phase::Held { .. }));
    first.task
}

/// The stored ended task a client's one-shot task/result read asks for.
#[must_use]
pub fn read(
    store: &temper_engine_domain_world::store::Store,
    range: &jig_domain_core::reads::Range,
) -> jig_domain_core::reads::Loaded {
    use jig_domain_core::{Key, Record, reads};
    let (reads::Scope::Task { task } | reads::Scope::Result { task } | reads::Scope::PersonTask { task }) = range.scope
    else {
        return reads::Loaded::Gone;
    };
    let Some(engine::Record::Core(Record::Tasks(tasks::Stored::Ended(row)))) =
        store.rows.get(&engine::Key::Core(Key::Tasks(tasks::Key::Ended(task))))
    else {
        return reads::Loaded::Gone;
    };
    let value = reads::Row {
        value: Record::Tasks(tasks::Stored::Ended(row.clone())),
        ended_read: None,
        owner: Some(row.clone()),
        ancestors: Box::new([]),
        related: None,
        unjoined: None,
        linked: Box::new([]),
        inbox: None,
    };
    if range.after.is_some() {
        return reads::Loaded::Page { rows: Box::new([]), next: None };
    }
    if range.rows == 0 || value.owned_bytes().expect("finite ended row") > range.bytes {
        return reads::Loaded::Busy;
    }
    reads::Loaded::Page { rows: Box::new([value]), next: None }
}
#[must_use]
pub fn project() -> temper_client_domain::Address {
    temper_client_domain::Address::Home { project: 1 }
}
/// Start a change from the rail's new box (temper/docs/design/client/view.md, 3).
#[must_use]
pub fn start_change(drafts: bool) -> Vec<skein_fake_person::Step<temper_client_domain::Address>> {
    use skein_fake_person::{Find, Step};
    use skein_ui::Role;
    if drafts {
        vec![
            Step::See { find: Find::exactly_named(Role::Menu, b"New draft or change"), within: WITHIN },
            // The terminal exposes menu items in the printed tree.
            Step::See { find: Find::exactly_named(Role::MenuItem, b"Change"), within: WITHIN },
            Step::Press { find: Find::exactly_named(Role::MenuItem, b"Change") },
        ]
    } else {
        vec![
            Step::See { find: Find::exactly_named(Role::Link, b"New change"), within: WITHIN },
            Step::Press { find: Find::exactly_named(Role::Link, b"New change") },
        ]
    }
}
#[must_use]
pub fn adopt(owner: &[u8], name: &[u8]) -> Vec<skein_fake_person::Step<temper_client_domain::Address>> {
    use skein_fake_person::{Find, Step};
    use skein_ui::Role;
    let mut steps = vec![
        Step::See { find: Find::exactly_named(Role::Link, b"How work runs for Project 1"), within: WITHIN },
        Step::Press { find: Find::exactly_named(Role::Link, b"How work runs for Project 1") },
        Step::See { find: Find::exactly_named(Role::Link, b"Repositories"), within: WITHIN },
        Step::Press { find: Find::exactly_named(Role::Link, b"Repositories") },
        Step::See { find: Find::named(Role::Link, b"Add repositories"), within: WITHIN },
        Step::Press { find: Find::named(Role::Link, b"Add repositories") },
        Step::See { find: Find::named(Role::Dialog, b"Add repositories to"), within: WITHIN },
        Step::See { find: Find::exactly_named(Role::Check, b"acme/shop"), within: WITHIN },
        Step::See { find: Find::named(Role::TextField, b"Owner"), within: WITHIN },
        Step::Type { find: Find::named(Role::TextField, b"Owner"), words: owner.into() },
        Step::Type { find: Find::exactly_named(Role::TextField, b"Name"), words: name.into() },
        Step::Type { find: Find::named(Role::TextField, b"Landing branch"), words: b"main".as_slice().into() },
        Step::See {
            find: Find::named(Role::Button, &skein_lib::bytes::concat(&[b"Adopt ", owner, b"/", name])),
            within: WITHIN,
        },
        Step::Press { find: Find::named(Role::Button, b"Adopt") },
    ];
    if !owner.is_empty() && name != b"shop" && name != b"second" {
        steps.insert(
            steps.len() - 2,
            Step::Press { find: Find::exactly_named(Role::Check, b"Make it the project's home") },
        );
        steps.insert(steps.len() - 2, Step::Press { find: Find::exactly_named(Role::Choice, b"Context") });
    }
    steps
}
#[must_use]
pub fn adopt_and_see_outcome(owner: &[u8], name: &[u8]) -> Vec<skein_fake_person::Step<temper_client_domain::Address>> {
    use skein_fake_person::{Find, Step};
    let mut steps = adopt(owner, name);
    steps.extend([Step::See { find: Find::text(b"acme/shop"), within: WITHIN }]);
    steps
}

const QUESTION_CHAT_REPLY: Token = Token::new(1002);
const QUESTION_CHAT_KEY: [u8; 16] = [32; 16];
pub const QUESTION_WORKER: Token = Token::new(7);
pub(crate) const QUESTION_PROJECT: u32 = 1;
const QUESTION_CALL: u64 = 1003;
const QUESTION_DELEGATE: u64 = 1004;
// Each project reserves a delegate call and a question call on the scripted worker.
const QUESTION_CALLS_PER_PROJECT: u64 = 2;
const QUESTION_COMPLETION: u32 = 1;
const QUESTION_POSITION: u32 = 1;
const QUESTION_ASSIGN_AT: Time = Time::from_nanos(2_000_000_000);
/// A question committed by the running chat for its person.
pub fn question(front: &mut Joint, words: &[u8]) -> u64 {
    question_with_text_bound(front, words, None)
}
/// A running chat's question beside a member whose task reads probe the project's policy.
pub fn team_question(front: &mut Joint, words: &[u8]) -> u64 {
    use jig_domain_authority as authority;
    use temper_engine_domain_world::{store::Store, walking};

    let mut limits = crate::front::engine_limits();
    // The owner and member each have a policy role.
    limits.authority.roles = 2;
    let configs = std::array::from_fn(|_| {
        let mut config = walking::config(SEED);
        config.core.connectors = Box::new([FORGE]);
        // Team's chat uses no goals; keep the two clients' reads within the fixed watch table.
        config.core.defaults.templates = config
            .core
            .defaults
            .templates
            .into_vec()
            .into_iter()
            .filter(|template| template.number != jig_client::shapes::GOAL)
            .collect();
        let mut rules = config.core.defaults.rules.clone();
        rules.ceiling.delegation.depth = 2;
        rules.ceiling.delegation.tasks = 2;
        let policy = &mut config.core.seed_projects[0].policy.policy;
        policy.ceiling.delegation.depth = 2;
        policy.ceiling.delegation.tasks = 2;
        policy.roles[0].authority.delegation.depth = 2;
        policy.roles[0].authority.delegation.tasks = 2;
        let mut member = policy.roles[0].clone();
        member.number = jig_domain_people::Role::Member.number();
        // Watch alone (Requests' bit 128); reading a live task also probes the refused takeover.
        member.requests = authority::Requests(128);
        let owner = policy.roles[0].clone();
        policy.roles = Box::new([owner, member]);
        let mut domain = authority::Domain::new(rules, limits.authority).expect("Team authority");
        let mut out = skein_lib::Queue::uncapped();
        authority::step(
            &mut domain,
            authority::Event::Policy { project: QUESTION_PROJECT, policy: policy.clone() },
            &mut out,
        );
        assert_eq!(out.pop(), Some(authority::PolicyFact::Added { project: QUESTION_PROJECT }));
        config.core.seed_projects[0].policy.policy = domain.policy(QUESTION_PROJECT).expect("Team policy").clone();
        config.core.defaults.rules = domain.rules().clone();
        config
    });
    limits.tasks.inbox_bytes = limits.tasks.message_bytes.checked_mul(8).expect("question and answer slots fit");
    limits.fleet.slots = 2;
    limits.fleet.attempts = 4;
    limits.journal.writes = 4096;
    limits.journal.deliveries = 64;
    limits.journal.held = 192;
    *front = Joint::with_engine(SEED, &domain_limits(), Store::with_limits(&limits), configs, &limits, 2);
    commit_question(front, words, false)
}
/// Choose the document text bound before the engine starts, without changing message admission.
pub fn question_with_text_bound(front: &mut Joint, words: &[u8], text: Option<u32>) -> u64 {
    question_with_agreement(front, words, text, false)
}
/// Work agreed first owes the chat a question before its change can start.
pub fn agreed_question(front: &mut Joint, words: &[u8]) -> u64 {
    question_with_agreement(front, words, None, true)
}
fn question_with_agreement(front: &mut Joint, words: &[u8], text: Option<u32>, agree: bool) -> u64 {
    *front = Joint::with_question_text_bound(SEED, &domain_limits(), text, agree);
    commit_question(front, words, agree)
}
/// A reply composer at the shipped message bound, with the real question and engine.
pub fn question_composer(front: &mut Joint, words: &[u8]) -> u64 {
    *front = Joint::with_question_composer(SEED, &whole_turn_limits());
    commit_question(front, words, false)
}
fn commit_question(front: &mut Joint, words: &[u8], agree: bool) -> u64 {
    commit_project_question(front, QUESTION_PROJECT, words, agree)
}
/// Commit the sibling question story on an already configured project, retaining the other project.
pub fn project_question(front: &mut Joint, project: u32, words: &[u8]) -> u64 {
    commit_project_question(front, project, words, false)
}
fn commit_project_question(front: &mut Joint, project: u32, words: &[u8], agree: bool) -> u64 {
    let driver = &mut front.engine;
    let offset = u64::from(project - QUESTION_PROJECT);
    let delegate_call = QUESTION_DELEGATE + offset * QUESTION_CALLS_PER_PROJECT;
    let question_call = QUESTION_CALL + offset * QUESTION_CALLS_PER_PROJECT;
    let mut key = QUESTION_CHAT_KEY;
    key[0] += u8::try_from(offset).expect("fixture project key");
    driver.sign_in();
    driver.settle();
    driver.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: skein_lib::ReplyTo::new(Token::new(QUESTION_CHAT_REPLY.raw() + offset)),
        sign_in: driver.session(),
        key,
        ask: driver.chat_ask(project, b"Question chat"),
    }));
    driver.settle();
    let parent = driver.last_assignment();
    driver.call(
        QUESTION_WORKER,
        parent.task,
        parent.attempt,
        delegate_call,
        engine::Call {
            completion: QUESTION_COMPLETION,
            position: QUESTION_POSITION,
            tool: if agree {
                engine::Tool::Goal { words: b"Ask the person".as_slice().into(), advisor: false, agree: true }
            } else {
                engine::Tool::Delegate {
                    batch: Box::new([temper_engine_domain_world::fixtures::report_delegate(
                        b"Ask the person",
                        Box::new([]),
                    )]),
                }
            },
        },
    );
    driver.settle();
    driver.answer(QUESTION_WORKER, parent.task, parent.attempt, 0, tasks::End::Parked, engine::Parts::none());
    driver.settle();
    driver.env.now = driver.env.now.saturating_add(Duration::from_nanos(QUESTION_ASSIGN_AT.as_nanos()));
    engine::fire(&mut driver.root, &driver.env);
    engine::release(&mut driver.root, &driver.env, &mut driver.out);
    driver.collect();
    driver.settle();
    let run = driver.last_assignment();
    assert_ne!(run.task, parent.task, "delegate assigned: {:?}", driver.answered(delegate_call));
    driver.call(
        QUESTION_WORKER,
        run.task,
        run.attempt,
        question_call,
        engine::Call {
            completion: QUESTION_COMPLETION,
            position: QUESTION_POSITION,
            tool: engine::Tool::Message {
                target: parent.task,
                form: engine::MessageForm::Question,
                words: words.into(),
            },
        },
    );
    driver.settle();
    assert!(
        matches!(driver.answered(question_call), Some(engine::CallAnswer::Sent { .. })),
        "question committed: {:?}",
        driver.answered(question_call)
    );
    parent.task
}
#[test]
fn the_question_fixture_commits_its_numbered_question_before_the_client_starts() {
    let mut front = Joint::new(SEED, domain_limits());
    let task = question(&mut front, b"Which provider first?");
    assert!(
        front
            .engine
            .task(task)
            .expect("chat")
            .inbox
            .iter()
            .any(|message| message.words.as_ref() == b"Which provider first?")
    );
}

const CHAT_REPLY: Token = Token::new(1005);
const CHAT_KEY: [u8; 16] = [35; 16];
const CHAT_PROJECT: u32 = 1;
/// A real running chat, ready for the person's next words.
pub fn chat(front: &mut Joint, words: &[u8]) -> u64 {
    let driver = &mut front.engine;
    driver.sign_in();
    driver.settle();
    driver.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: skein_lib::ReplyTo::new(CHAT_REPLY),
        sign_in: driver.session(),
        key: CHAT_KEY,
        ask: driver.chat_ask(CHAT_PROJECT, words),
    }));
    driver.settle();
    driver.last_assignment().task
}
const FIRST_REPLY_TURN: u32 = 1;
const REPLY_SPEND: u64 = 0;
/// A real committed worker turn puts this chat in the inbox's replies group.
pub fn replied_chat(front: &mut Joint, words: &[u8]) -> u64 {
    let task = chat(front, words);
    let assignment = front.engine.last_assignment();
    let event = temper_engine_domain_world::protocol::turn(
        QUESTION_WORKER,
        task,
        assignment.attempt,
        FIRST_REPLY_TURN,
        REPLY_SPEND,
        None,
        b"reply",
    );
    front.engine.send(event);
    front.engine.settle();
    task
}
/// Finish a replied chat normally, retaining its unread turn beside its Result.
pub fn finish_replied_chat(front: &mut Joint, task: u64) {
    let run = front.engine.task(task).expect("running replied chat");
    front.engine.answer(
        QUESTION_WORKER,
        task,
        run.attempt,
        run.run_spent,
        tasks::End::Finished {
            result: tasks::TaskResult::Report { words: b"The chat is complete".as_slice().into() },
            cancel_delegates: false,
        },
        engine::Parts::none(),
    );
    front.engine.settle();
}
#[must_use]
pub fn chat_page(task: u64) -> temper_client_domain::Address {
    temper_client_domain::Address::Chat { task }
}

const GOAL_REPLY: Token = Token::new(1201);
const GOAL_KEY: [u8; 16] = [41; 16];
/// A goal's brief, its page's heading.
pub const GOAL_BRIEF: &[u8] = b"Fix the adopted repository";
const GOAL_PROJECT: u32 = 1;
const GOAL_PRIORITY: u32 = 1;
const GOAL_LABEL: u32 = 2;
const DELEGATE_CALL: u64 = 1202;
const DELEGATE_COMPLETION: u32 = 1;
const DELEGATE_POSITION: u32 = 1;
const RETRY_AT: Time = Time::from_nanos(2_000_000_000);
const NO_SPEND: u64 = 0;
const DELEGATE_SPEND: u64 = 3;
const DELEGATE_PROGRESS: &[u8] = b"Findings in progress";
const DEPENDENT_DELEGATES: u32 = 2;
pub struct Dependents {
    pub goal: u64,
    pub first: u64,
    pub second: u64,
}
/// A tracked goal with two report delegates; the first has spent, the second waits for it.
pub fn goal_with_dependent_delegates(front: &mut Joint) -> Dependents {
    use temper_engine_domain_world::fixtures::report_delegate;
    let limits = domain_limits();
    *front = Joint::with_delegates(SEED, &limits, DEPENDENT_DELEGATES, limits.client.domain.rows);
    let goal = make_goal_with_delegation(front, false, Some(DEPENDENT_DELEGATES));
    let driver = &mut front.engine;
    let parent = driver.last_assignment();
    assert_eq!(parent.task, goal);
    driver.call(
        QUESTION_WORKER,
        goal,
        parent.attempt,
        DELEGATE_CALL,
        engine::Call {
            completion: DELEGATE_COMPLETION,
            position: DELEGATE_POSITION,
            tool: engine::Tool::Delegate {
                batch: Box::new([
                    report_delegate(b"Investigate the change", Box::new([])),
                    report_delegate(b"Use the findings", Box::new([engine::Dependency::Batch(0)])),
                ]),
            },
        },
    );
    driver.settle();
    let Some(engine::CallAnswer::Delegated(children)) = driver.answered(DELEGATE_CALL) else {
        panic!("dependent delegates admitted: {:?}", driver.answered(DELEGATE_CALL));
    };
    let dependents = Dependents { goal, first: children[0].task, second: children[1].task };
    driver.answer(QUESTION_WORKER, goal, parent.attempt, NO_SPEND, tasks::End::Parked, engine::Parts::none());
    driver.settle();
    let first = driver.last_assignment();
    assert_eq!(first.task, dependents.first, "first delegate runs before its dependant");
    delegate_turn(front, first.task, None, DELEGATE_PROGRESS);
    dependents
}
/// Commit the running delegate's next turn taking all the person's pending words.
/// Its numbered input fence removes those words from the engine's unread inbox.
pub fn read_messages(front: &mut Joint, task: u64) -> u64 {
    let number = front
        .engine
        .task(task)
        .expect("running delegate")
        .inbox
        .iter()
        .rev()
        .find(|message| message.from == tasks::Party::Person(PARTY))
        .expect("pending person message")
        .number;
    delegate_turn(front, task, Some(number), b"I will follow that direction");
    number
}
/// Push the initial turn just beyond the client's retained activity window.
pub fn extend_delegate_conversation(front: &mut Joint, task: u64) {
    let window = domain_limits().client.domain.rows;
    for _ in 1..window {
        delegate_turn(front, task, None, b"The investigation continues");
    }
    delegate_turn(front, task, None, b"The latest findings");
}
fn delegate_turn(front: &mut Joint, task: u64, read: Option<u64>, words: &[u8]) {
    use smith_domain_session::llm;
    let driver = &mut front.engine;
    let run = driver.task(task).expect("running delegate");
    let attempt = run.attempt;
    let number = run.turn + 1;
    // The initial turn spends three; later turns keep that cumulative total.
    let spent = run.run_spent.max(DELEGATE_SPEND);
    let mut turn =
        temper_worker_domain_world::protocol::session_turn(number, spent, words.len(), number == FIRST_REPLY_TURN);
    turn.messages[0].content = Box::new([llm::Block::Text { text: words.into(), replay: None }]);
    let body = smith_protocol_channel::encode_turn(
        &turn,
        &smith_transcript::CEILINGS,
        &temper_worker_domain_world::protocol::endpoints(),
    )
    .expect("delegate turn encodes");
    driver.send(temper_engine_domain_world::protocol::turn(QUESTION_WORKER, task, attempt, number, spent, read, &body));
    driver.settle();
}
/// The same goal after its first delegate's report unblocks the second.
pub fn goal_with_a_finished_dependency(front: &mut Joint) -> Dependents {
    let dependents = goal_with_dependent_delegates(front);
    report_first_delegate(front, dependents.first);
    assert!(
        front.engine.delivered.iter().any(|delivery| matches!(delivery,
            engine::Delivery::Assigned { assignment, .. } if assignment.task == dependents.second
        )),
        "dependant runs after the report"
    );
    dependents
}
/// Finish the running first delegate with its report and settle its spend.
pub fn report_first_delegate(front: &mut Joint, task: u64) {
    report_task(front, task, b"The change is feasible");
}
fn report_task(front: &mut Joint, task: u64, words: &[u8]) {
    let driver = &mut front.engine;
    let run = driver.task(task).expect("reporting run");
    let attempt = run.attempt;
    let spent = run.run_spent;
    driver.answer(
        QUESTION_WORKER,
        task,
        attempt,
        spent,
        tasks::End::Finished { result: tasks::TaskResult::Report { words: words.into() }, cancel_delegates: false },
        engine::Parts::none(),
    );
    driver.settle();
}
const BUDGET_READ: Token = Token::new(1204);
/// A tracked goal whose worker finished with a report retained by the engine.
pub fn finished_goal(front: &mut Joint) -> u64 {
    *front = Joint::with_questions(SEED, &domain_limits());
    let goal = make_goal(front, false);
    let driver = &mut front.engine;
    let run = driver.last_assignment();
    assert_eq!(run.task, goal);
    driver.answer(
        QUESTION_WORKER,
        goal,
        run.attempt,
        NO_SPEND,
        tasks::End::Finished {
            result: tasks::TaskResult::Report { words: b"The repository is ready".as_slice().into() },
            cancel_delegates: false,
        },
        engine::Parts::none(),
    );
    driver.settle();
    goal
}
/// Read the engine's supplied budget, including availability, without computing it in the story.
/// A point read also serves ended tasks, whose watches have ended (client/pages/goal.md, 2).
pub fn task_budget(front: &mut Joint, task: u64) -> jig_domain_core::documents::BudgetLine {
    use jig_domain_core::{documents::Document, reads};
    let driver = &mut front.engine;
    // The joint world drains deliveries between phases; the sign-in's durable
    // identity remains available after its initial reply has been consumed.
    let sign_in = driver
        .store
        .rows
        .values()
        .find_map(|record| {
            if let engine::Record::Core(jig_domain_core::Record::People(jig_domain_people::Stored::SignIn {
                number,
                person: PARTY,
                ..
            })) = record
            {
                Some(*number)
            } else {
                None
            }
        })
        .expect("fixture person's durable sign-in");
    driver.send(engine::Event::Party(engine::PartyInput::Read {
        reply_to: skein_lib::ReplyTo::new(BUDGET_READ),
        sign_in,
        read: reads::Read::Task { task },
    }));
    driver.settle();
    driver
        .delivered
        .iter()
        .rev()
        .find_map(|delivery| {
            let engine::Delivery::Read { reply, answer } = delivery else { return None };
            if *reply != skein_lib::ReplyTo::new(BUDGET_READ) {
                return None;
            }
            match answer {
                reads::Answer::Document(document) => {
                    if let Document::Task(document) = document.as_ref() {
                        Some(document.budget)
                    } else {
                        None
                    }
                }
                reads::Answer::Ended(ended) => {
                    if let reads::EndedValue::Task { document, .. } = &ended.value {
                        Some(document.budget)
                    } else {
                        None
                    }
                }
                reads::Answer::Setting(_)
                | reads::Answer::Revision(_)
                | reads::Answer::Revisions(_)
                | reads::Answer::TaskText(_)
                | reads::Answer::Tasks(_)
                | reads::Answer::Page(_)
                | reads::Answer::None(_) => None,
            }
        })
        .expect("goal budget document")
}
/// A tracked goal with a failed delegate, held by the real engine.
pub fn goal_with_held_delegate(front: &mut Joint) -> (u64, u64) {
    *front = Joint::with_questions(SEED, &domain_limits());
    let goal = make_goal(front, false);
    let driver = &mut front.engine;
    let parent = driver.last_assignment();
    assert_eq!(parent.task, goal);
    driver.call(
        QUESTION_WORKER,
        parent.task,
        parent.attempt,
        DELEGATE_CALL,
        engine::Call {
            completion: DELEGATE_COMPLETION,
            position: DELEGATE_POSITION,
            tool: engine::Tool::Delegate {
                batch: Box::new([temper_engine_domain_world::fixtures::report_delegate(
                    b"Review the change",
                    Box::new([]),
                )]),
            },
        },
    );
    driver.settle();
    let Some(engine::CallAnswer::Delegated(children)) = driver.answered(DELEGATE_CALL) else {
        panic!("delegate admitted: {:?}", driver.answered(DELEGATE_CALL));
    };
    let held = children[0].task;
    driver.answer(QUESTION_WORKER, parent.task, parent.attempt, NO_SPEND, tasks::End::Parked, engine::Parts::none());
    driver.settle();
    fail_delegate(driver, held);
    driver.env.now = RETRY_AT;
    engine::fire(&mut driver.root, &driver.env);
    engine::release(&mut driver.root, &driver.env, &mut driver.out);
    driver.collect();
    driver.settle();
    fail_delegate(driver, held);
    assert!(matches!(driver.task(held).expect("delegate").phase, tasks::Phase::Held { .. }));
    (goal, held)
}
fn fail_delegate(driver: &mut Driver, held: u64) {
    let run = driver.last_assignment();
    assert_eq!(run.task, held, "delegate has the next worker attempt");
    driver.answer(
        QUESTION_WORKER,
        held,
        run.attempt,
        NO_SPEND,
        tasks::End::Failed(tasks::Class::Run),
        engine::Parts::none(),
    );
    driver.settle();
}
/// Two tracked roots reserve only themselves, leaving room for each other.
pub fn amend_tasks(front: &mut Joint) -> (u64, u64) {
    const DEPENDENCY_KEY: [u8; 16] = [42; 16];
    const ROOT_BUDGET: u64 = 1; // walking's minimum run spend, leaving funding for both roots.
    let goal = make_goal_named(front, false, Some(0), GOAL_KEY, Some(ROOT_BUDGET), true);
    let dependency = make_goal_named(front, false, Some(0), DEPENDENCY_KEY, Some(ROOT_BUDGET), true);
    (goal, dependency)
}
/// A listed goal for settings' Work filter, with no spend reserved.
pub fn settings_goal(front: &mut Joint) -> u64 {
    make_goal_named(front, false, None, GOAL_KEY, Some(0), false)
}
fn make_goal(front: &mut Joint, procedure: bool) -> u64 {
    make_goal_with_delegation(front, procedure, None)
}
fn make_goal_with_delegation(front: &mut Joint, procedure: bool, delegates: Option<u32>) -> u64 {
    make_goal_named(front, procedure, delegates, GOAL_KEY, None, true)
}
fn make_goal_named(
    front: &mut Joint,
    procedure: bool,
    delegates: Option<u32>,
    key: [u8; 16],
    budget: Option<u64>,
    sign_in: bool,
) -> u64 {
    let driver = &mut front.engine;
    if sign_in {
        driver.sign_in();
        driver.settle();
    }
    if procedure {
        adopt_goal_repository(driver);
    }
    let jig_domain_people::Ask::Create { mut batch, .. } = driver.chat_ask(GOAL_PROJECT, GOAL_BRIEF) else {
        panic!("create fixture");
    };
    if let Some(budget) = budget {
        batch[0].budget = jig_domain_people::Budget::Named;
        batch[0].authority.spend = budget;
    }
    batch[0].tracked = Some(GOAL_PRIORITY);
    batch[0].label = GOAL_LABEL;
    batch[0].flags.conversation = false;
    batch[0].flags.listed = true;
    if let Some(delegates) = delegates {
        batch[0].authority.delegation.tasks = delegates;
    }
    if procedure {
        batch[0].authority.tools = NO_TOOLS;
        // A malformed role choice keeps this panel fixture without a connector change.
        batch[0].parameters = Box::new([
            jig_domain_people::Parameter::Resource {
                name: REPOSITORY_PARAMETER,
                connector: FORGE_CONNECTOR,
                resource: GOAL_REPOSITORY,
            },
            jig_domain_people::Parameter::Bytes { name: 11, value: b"invalid".as_slice().into() },
        ]);
        batch[0].executor =
            jig_domain_people::TaskExecutor::Procedure { connector: FORGE_CONNECTOR, code: FORGE_PROCEDURE };
        batch[0].contract =
            jig_domain_people::Contract::Change { connector: FORGE_CONNECTOR, kind: FORGE_CHANGE, words: REPORT_BYTES };
    }
    let session = if sign_in {
        driver.session()
    } else {
        driver
            .store
            .rows
            .values()
            .find_map(|record| {
                if let engine::Record::Core(jig_domain_core::Record::People(jig_domain_people::Stored::SignIn {
                    number,
                    person,
                    ..
                })) = record
                {
                    (*person == PARTY).then_some(*number)
                } else {
                    None
                }
            })
            .expect("goal owner's sign-in")
    };
    driver.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: skein_lib::ReplyTo::new(GOAL_REPLY),
        sign_in: session,
        key,
        ask: jig_domain_people::Ask::Create { project: GOAL_PROJECT, batch },
    }));
    driver.settle();
    let mut reply = None;
    for delivery in driver.delivered.iter().rev() {
        if let engine::Delivery::WebReply { reply: value, .. } = delivery {
            reply = Some(value);
            break;
        }
    }
    let Some(jig_domain_people::Reply::Outcome(jig_domain_people::Outcome::Created { first, .. })) = reply else {
        panic!("goal creation expected: {reply:?}");
    };
    *first
}
#[must_use]
pub fn task_page(task: u64) -> temper_client_domain::Address {
    temper_client_domain::Address::Task { task }
}
#[must_use]
pub fn plan_item(task: u64, status: &[u8]) -> skein_fake_person::Find {
    use skein_fake_person::{Find, Target};
    Find::named(skein_ui::Role::Status, status).within(Box::new([Target::ExactRole {
        role: skein_ui::Role::TreeItem,
        name: format!("Task {task}").into_bytes().into_boxed_slice(),
    }]))
}
#[must_use]
pub fn dependency_item(task: u64, find: skein_fake_person::Find) -> skein_fake_person::Find {
    use skein_fake_person::Target;
    find.within(Box::new([
        Target::ExactRole { role: skein_ui::Role::TreeItem, name: format!("Task {task}").into_bytes().into() },
        Target::ExactRole {
            role: skein_ui::Role::List,
            name: format!("Dependencies of task {task}").into_bytes().into(),
        },
    ]))
}
const FORGE_CONNECTOR: u16 = 1;
const NO_TOOLS: u64 = 0;
const FORGE_PROCEDURE: u32 = 2;
const FORGE_CHANGE: u16 = 1;
const REPORT_BYTES: u32 = 32;
/// A tracked procedure root whose chip names its forge-owned panel.
pub fn goal_with_forge_panel(front: &mut Joint) -> u64 {
    *front = Joint::with_procedures(SEED, &domain_limits());
    make_goal(front, true)
}

/// Adopt before pushing under temper's prefix: adoption refuses an unowned branch there.
pub fn adopt_change_repository(front: &mut Joint) {
    front.engine.sign_in();
    front.engine.settle();
    adopt_goal_repository(&mut front.engine);
    // forge_routes.rs's worker also receives the repository's application credential.
    front.engine.send(engine::Event::Credential { number: 1, generation: 1, valid: Duration::from_secs(3600) });
    front.engine.settle();
    assert!(front.engine_adopted(ADOPT_FORGE, GOAL_REPOSITORY_ID));
}

/// A tracked change on the landing joint, handed a branch already pushed on its stand.
/// The repository and sign-in are prepared by `adopt_change_repository`.
/// Leaves the joint's immediate work for the story to drive one step at a time.
pub fn goal_with_pushed_change(front: &mut Joint, branch: &[u8]) -> u64 {
    pushed_change(front, branch, None)
}

/// A change with an explicit descendant allowance, for authority hold stories.
pub fn goal_with_pushed_change_with_tasks(front: &mut Joint, branch: &[u8], tasks: u32) -> u64 {
    pushed_change(front, branch, Some(tasks))
}

pub fn goal_with_pushed_change_without_review_authority(front: &mut Joint, branch: &[u8]) -> u64 {
    constrained_change(front, branch, None, None, false)
}
pub fn goal_with_pushed_change_without_open_grant(front: &mut Joint, branch: &[u8]) -> u64 {
    constrained_change(front, branch, None, Some(3), true)
}
fn pushed_change(front: &mut Joint, branch: &[u8], tasks: Option<u32>) -> u64 {
    constrained_change(front, branch, tasks, None, true)
}
fn constrained_change(
    front: &mut Joint,
    branch: &[u8],
    tasks: Option<u32>,
    omit_kind: Option<u16>,
    review: bool,
) -> u64 {
    let driver = &mut front.engine;
    let jig_domain_people::Ask::Create { mut batch, .. } = driver.chat_ask(GOAL_PROJECT, GOAL_BRIEF) else {
        panic!("create fixture");
    };
    if let Some(tasks) = tasks {
        batch[0].authority.delegation.tasks = tasks;
    }
    if let Some(kind) = omit_kind {
        batch[0].authority.grants =
            batch[0].authority.grants.iter().filter(|grant| grant.kind != kind).cloned().collect();
    }
    // These grants come from this joint's configured forge connector.
    let connector = batch[0].authority.grants[0].connector;
    batch[0].tracked = Some(GOAL_PRIORITY);
    batch[0].label = GOAL_LABEL;
    batch[0].flags.conversation = false;
    batch[0].flags.listed = true;
    batch[0].authority.tools = NO_TOOLS;
    // A normal pushed change can review and then repair under this joint's coding charter.
    batch[0].authority.delegation.kinds = if review {
        Box::new([jig_domain_people::Executor::Charter(1), jig_domain_people::Executor::Charter(4)])
    } else {
        Box::new([])
    };
    batch[0].parameters = Box::new([
        jig_domain_people::Parameter::Resource { name: REPOSITORY_PARAMETER, connector, resource: GOAL_REPOSITORY },
        jig_domain_people::Parameter::Bytes { name: 3, value: branch.into() },
    ]);
    batch[0].executor = jig_domain_people::TaskExecutor::Procedure { connector, code: FORGE_PROCEDURE };
    batch[0].contract = jig_domain_people::Contract::Change { connector, kind: FORGE_CHANGE, words: REPORT_BYTES };
    driver.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: skein_lib::ReplyTo::new(GOAL_REPLY),
        sign_in: driver.session(),
        key: GOAL_KEY,
        ask: jig_domain_people::Ask::Create { project: GOAL_PROJECT, batch },
    }));
    // Commit creation, leaving subsequent procedure work visible to the caller.
    for _ in 0..200 {
        driver.advance(true);
        if let Some(task) = driver.delivered.iter().find_map(|delivery| {
            if let engine::Delivery::WebReply {
                to,
                reply: jig_domain_people::Reply::Outcome(jig_domain_people::Outcome::Created { first, .. }),
            } = delivery
                && *to == skein_lib::ReplyTo::new(GOAL_REPLY)
            {
                Some(*first)
            } else {
                None
            }
        }) {
            return task;
        }
    }
    panic!("pushed change creation: {:?}", driver.delivered);
}

const ADOPT_FORGE: u16 = 1;
const ADOPT_REPLY: Token = Token::new(1203);
const ADOPT_KEY: [u8; 16] = [43; 16];
const ADOPT_DIGEST: [u8; 32] = [43; 32];
const REPOSITORY_PARAMETER: u32 = 1;
const GOAL_REPOSITORY_ID: u32 = 7;
// A task names its repository by the forge and id, packed (domain/forge/adoption.md).
const GOAL_REPOSITORY: u64 = temper_engine_domain_forge::tag(ADOPT_FORGE, GOAL_REPOSITORY_ID);
fn adopt_goal_repository(driver: &mut Driver) {
    driver.connect_forges();
    let mut request = temper_engine_domain_world::fixtures::adoption(ADOPT_FORGE);
    request.owner = b"acme".as_slice().into();
    request.name = b"shop".as_slice().into();
    driver.send(engine::Event::Party(engine::PartyInput::Application {
        scope: jig_domain_people::ApplicationScope::Project(GOAL_PROJECT),
        reply_to: skein_lib::ReplyTo::new(ADOPT_REPLY),
        sign_in: driver.session(),
        key: ADOPT_KEY,
        digest: ADOPT_DIGEST,
        carried: None,
        request: temper_engine_domain::Application::Forge(temper_engine_domain_forge::Ask::Adopt(request)),
    }));
    driver.settle();
}

/// Guard on one-step answer waits, never a client limit.
const ANSWER_STEPS: u32 = 4096;
#[must_use]
pub fn agent(world: &mut World<Temper, Joint>) -> jig_client_kit::harness::ClientId {
    let limits = crate::agent::limits().domain;
    world.front().next_client_limits(limits);
    world.client_with(PARTY, inbox(), limits, crate::agent::new)
}
#[must_use]
pub fn call(words: &[&[u8]]) -> crate::app::Event {
    use std::os::unix::ffi::OsStrExt;
    match temper_client_tools::table::parse(words).unwrap_or_else(|error| panic!("agent call usage: {error:?}")) {
        temper_client_tools::jig::call::Parsed::Call { mut call, json } => {
            // The world supplies the host's already-loaded amendment text.
            if temper_client_tools::table::TOOLS[usize::from(call.tool)].name == b"task amend" {
                for slot in &mut call.values {
                    if let Some(temper_client_tools::jig::call::Values::One(
                        temper_client_tools::jig::call::Value::File(path),
                    )) = slot
                    {
                        let bytes = std::fs::read(std::path::Path::new(std::ffi::OsStr::from_bytes(path)))
                            .expect("world's text file");
                        let stripped = bytes.strip_suffix(b"\n").unwrap_or(&bytes);
                        *slot = Some(temper_client_tools::jig::call::Values::One(
                            temper_client_tools::jig::call::Value::Text(stripped.into()),
                        ));
                    }
                }
            }
            crate::app::Event::Call { number: 0, call, json, value: None }
        }
        temper_client_tools::jig::call::Parsed::File { file, json } => {
            let temper_client_tools::jig::call::Source::Path(path) = &file else { panic!("world call needs a file") };
            let bytes =
                std::fs::read(std::path::Path::new(std::ffi::OsStr::from_bytes(path))).expect("world's call file");
            let call = temper_client_tools::jig::json::call(
                temper_client_tools::table::TOOLS,
                &bytes,
                file,
                &temper_client_tools::limits::tools(),
            )
            .expect("world's JSON call");
            crate::app::Event::Call { number: 0, call, json, value: None }
        }
    }
}
pub fn give(world: &mut World<Temper, Joint>, agent: jig_client_kit::harness::ClientId, words: &[&[u8]]) -> u32 {
    use jig_client_kit::harness::Until;
    let start = world.now();
    for _ in 0..ANSWER_STEPS {
        if world.root(agent).agent().ready() && world.root(agent).agent().kept() == 0 {
            break;
        }
        world.run(Until::Steps(2));
    }
    assert!(world.now() <= start.saturating_add(WITHIN), "agent ready within world time");
    assert!(world.root(agent).agent().ready() && world.root(agent).agent().kept() == 0, "agent ready before call");
    let number = world.root(agent).agent().calls() + 1;
    let crate::app::Event::Call { call, json, .. } = call(words) else { unreachable!() };
    world.event(agent, crate::app::Event::Call { number, call, json, value: None });
    number
}
/// Supply the host's already-loaded FILE bytes without touching the filesystem.
pub fn give_with_file(
    world: &mut World<Temper, Joint>,
    agent: jig_client_kit::harness::ClientId,
    words: &[&[u8]],
    bytes: &[u8],
) -> u32 {
    use jig_client_kit::harness::Until;
    use temper_client_tools::jig::call::{Value, Values};
    let start = world.now();
    for _ in 0..ANSWER_STEPS {
        if world.root(agent).agent().ready() && world.root(agent).agent().kept() == 0 {
            break;
        }
        world.run(Until::Steps(2));
    }
    assert!(world.now() <= start.saturating_add(WITHIN), "agent ready within world time");
    assert!(world.root(agent).agent().ready() && world.root(agent).agent().kept() == 0, "agent ready before call");
    let number = world.root(agent).agent().calls() + 1;
    let crate::app::Event::Call { mut call, json, .. } = call(words) else { unreachable!() };
    let mut files = 0;
    for slot in &mut call.values {
        if matches!(slot, Some(Values::One(Value::File(_)))) {
            *slot = Some(Values::One(Value::Text(bytes.into())));
            files += 1;
        }
    }
    assert_eq!(files, 1, "one FILE slot");
    world.event(agent, crate::app::Event::Call { number, call, json, value: None });
    number
}
pub fn answered(
    world: &mut World<Temper, Joint>,
    agent: jig_client_kit::harness::ClientId,
    number: u32,
) -> temper_client_tools::jig::Printed {
    use jig_client_kit::harness::Until;
    let start = world.now();
    for _ in 0..ANSWER_STEPS {
        let outcome = world.run(Until::Steps(2));
        if let Some(printed) = world.root(agent).agent().printed(number) {
            assert_eq!(outcome.verdict, skein_world::domain::Verdict::Passed, "{outcome:?}");
            assert!(world.now() <= start.saturating_add(WITHIN), "agent answered within world time");
            return temper_client_tools::jig::Printed {
                file: temper_client_tools::jig::FileEnd::Keep,
                status: printed.status,
                stdout: printed.stdout.clone(),
                stderr: printed.stderr.clone(),
            };
        }
    }
    panic!("agent call {number} was not answered");
}
#[must_use]
pub fn done_task(printed: &temper_client_tools::jig::Printed) -> u64 {
    let stdout = std::str::from_utf8(&printed.stdout).expect("printed UTF-8");
    stdout
        .strip_prefix("done: task ")
        .and_then(|value| value.split_once(", "))
        .expect("done task outcome")
        .0
        .parse()
        .expect("task number")
}

/// client/ctl/README.md, 9: two 16 KiB parts within the tools' 32 KiB text bound.
pub const LARGE_TURN_TEXT: usize = 24_576;
#[must_use]
pub fn codec_for(limits: &jig_client::DomainLimits) -> jig_protocol_transcript::Limits {
    let bytes = limits.document_bytes.min(u64::from(jig_client_protocol_wire::CEILINGS.turn_carried_bytes));
    jig_protocol_transcript::Limits::new(
        u32::try_from(bytes).expect("wire bound fits"),
        jig_client::contract::TurnLimits {
            messages: limits.rows,
            blocks: limits.rows,
            text: limits.turn_text,
            outcome: limits.turn_text,
        },
    )
}
/// Commit smith's session turn through the worker's real turn boundary.
pub fn large_turn(front: &mut Joint, task: u64) -> (u64, u32) {
    use jig_domain_core::{CoreRecord, Record, TurnRecord};
    let empty = large_turn_body(0);
    let per_byte = large_turn_body(1).len().checked_sub(empty.len()).expect("opaque padding grows the encoding");
    let mut row = turn_read_row(
        front,
        task,
        Record::Core(CoreRecord::Turn(TurnRecord {
            opens_window: true,
            task,
            attempt: front.engine.last_assignment().attempt,
            turn: FIRST_REPLY_TURN,
            spent: REPLY_SPEND,
            read: None,
            at: NOW,
            transcript: empty,
        })),
    );
    // The first committed turn also adds its retained reply hint to the owner.
    let owner = row.owner.as_mut().expect("turn owner");
    assert!(owner.replies.is_empty(), "the fixture commits its first reply");
    owner.replies = Box::new([tasks::ReplyHint {
        number: owner.last_reply.checked_add(1).expect("reply ordinal"),
        activity: owner.activity.checked_add(1).expect("reply activity"),
        attempt: front.engine.last_assignment().attempt,
        turn: FIRST_REPLY_TURN,
        at: NOW,
    }]);
    // Keep all visible text; only opaque padding fills the remaining whole-row budget.
    let room = front
        .engine
        .env
        .limits
        .loads
        .read_bytes
        .checked_sub(row.owned_bytes().expect("zero-padding row ownership"))
        .expect("visible turn and its owner fit the read");
    let padding = usize::try_from(room).expect("fixture read fits usize") / per_byte;
    large_turn_with_opaque(front, task, padding)
}
/// Commit a turn with a separately sized opaque reasoning item.
pub fn large_turn_with_opaque(front: &mut Joint, task: u64, opaque: usize) -> (u64, u32) {
    let assignment = front.engine.last_assignment();
    let body = large_turn_body(opaque);
    front.engine.send(temper_engine_domain_world::protocol::turn(
        QUESTION_WORKER,
        task,
        assignment.attempt,
        FIRST_REPLY_TURN,
        REPLY_SPEND,
        None,
        &body,
    ));
    front.engine.settle();
    (assignment.attempt, FIRST_REPLY_TURN)
}
/// Three named calls whose results together exceed the display budget.
pub fn turn_with_large_results(front: &mut Joint, task: u64) {
    use smith_domain_session::llm;
    let mut turn = temper_worker_domain_world::protocol::session_turn(FIRST_REPLY_TURN, REPLY_SPEND, 0, true);
    let mut calls = Vec::new();
    let mut results = Vec::new();
    for name in [b"first_call".as_slice(), b"second_call", b"third_call"] {
        calls.push(llm::Block::ToolCall(llm::ToolCall {
            id: name.into(),
            name: name.into(),
            input: Box::from(*b"{}"),
            call: llm::Decoded::Historical,
            replay: None,
        }));
        results.push(llm::Block::ToolResult {
            id: name.into(),
            result: llm::Returned::Text { text: vec![b'r'; 65_536].into_boxed_slice(), error: false, replay: None },
        });
    }
    turn.messages = Box::new([
        llm::Message { role: llm::Role::Assistant, content: calls.into_boxed_slice() },
        llm::Message { role: llm::Role::User, content: results.into_boxed_slice() },
    ]);
    let body = smith_protocol_channel::encode_turn(
        &turn,
        &smith_transcript::CEILINGS,
        &temper_worker_domain_world::protocol::endpoints(),
    )
    .expect("large results encode");
    let assignment = front.engine.last_assignment();
    front.engine.send(temper_engine_domain_world::protocol::turn(
        QUESTION_WORKER,
        task,
        assignment.attempt,
        FIRST_REPLY_TURN,
        REPLY_SPEND,
        None,
        &body,
    ));
    front.engine.settle();
}

fn large_turn_body(opaque: usize) -> Box<[u8]> {
    use smith_domain_session::llm;
    let mut turn =
        temper_worker_domain_world::protocol::session_turn(FIRST_REPLY_TURN, REPLY_SPEND, LARGE_TURN_TEXT, true);
    for message in &mut turn.messages {
        let mut blocks = message.content.to_vec();
        blocks.push(llm::Block::Opaque { bytes: vec![0; opaque].into_boxed_slice() });
        message.content = blocks.into_boxed_slice();
    }
    smith_protocol_channel::encode_turn(
        &turn,
        &smith_transcript::CEILINGS,
        &temper_worker_domain_world::protocol::endpoints(),
    )
    .expect("large turn encodes")
}
/// Complete ownership of a turn alone and joined to its activity row.
#[must_use]
pub fn turn_row_costs(front: &Joint, task: u64) -> (u64, u64) {
    use jig_domain_core as core;
    let turn = front
        .engine
        .store
        .rows
        .values()
        .find_map(|record| {
            if let engine::Record::Core(value @ core::Record::Core(core::CoreRecord::Turn(turn))) = record
                && turn.task == task
            {
                Some(value.clone())
            } else {
                None
            }
        })
        .expect("durable turn");
    let mut row = turn_read_row(front, task, turn);
    let single = row.owned_bytes().expect("single row cost");
    let line = front
        .engine
        .store
        .rows
        .values()
        .find_map(|record| {
            if let engine::Record::Core(core::Record::Tasks(tasks::Stored::Activity(line))) = record
                && line.task == task
                && matches!(line.line, tasks::Line::Turn { .. })
            {
                Some(line.clone())
            } else {
                None
            }
        })
        .expect("durable turn activity");
    row.related = Some(row.value);
    row.value = core::Record::Tasks(tasks::Stored::Activity(line));
    (single, row.owned_bytes().expect("activity row cost"))
}
fn turn_read_row(front: &Joint, task: u64, value: jig_domain_core::Record) -> jig_domain_core::reads::Row {
    jig_domain_core::reads::Row {
        value,
        owner: Some(Box::new(front.engine.task(task).expect("live turn owner").clone())),
        ancestors: Box::new([]),
        related: None,
        unjoined: None,
        ended_read: None,
        linked: Box::new([]),
        inbox: None,
    }
}
/// Follow the printed continuation calls, retaining the turn's text alone.
pub fn read_whole(
    world: &mut World<Temper, Joint>,
    agent: jig_client_kit::harness::ClientId,
    words: &[&[u8]],
) -> (Vec<u8>, u32) {
    let mut words: Vec<Box<[u8]>> = words.iter().map(|word| Box::from(*word)).collect();
    let mut text = Vec::new();
    let mut reads = 0;
    loop {
        let borrowed: Vec<&[u8]> = words.iter().map(AsRef::as_ref).collect();
        let number = give(world, agent, &borrowed);
        let printed = answered(world, agent, number);
        assert_eq!(printed.status, 0, "{printed:?}");
        reads += 1;
        let stdout = std::str::from_utf8(&printed.stdout).expect("turn text UTF-8");
        let mut next = None;
        for line in stdout.lines() {
            if let Some((_, call)) = line.split_once("read on with: ").or_else(|| line.split_once("read with: ")) {
                next = Some(call.split_ascii_whitespace().map(|word| Box::from(word.as_bytes())).collect());
            } else if !line.starts_with("turn ") && !line.starts_with("opaque ") {
                text.extend_from_slice(line.as_bytes());
            }
        }
        let Some(next) = next else { return (text, reads) };
        words = next;
    }
}

/// Copy an executable act from the first acts section of a printed document.
#[must_use]
pub fn printed_act(stdout: &[u8], act: &[u8]) -> Vec<Box<[u8]>> {
    let stdout = std::str::from_utf8(stdout).expect("printed document UTF-8");
    let act = std::str::from_utf8(act).expect("act UTF-8");
    let line = stdout
        .lines()
        .skip_while(|line| *line != "acts:")
        .skip(1)
        .take_while(|line| line.starts_with("  "))
        .find(|line| line.trim_start().strip_prefix(act).is_some_and(|rest| rest.starts_with(' ')))
        .expect("printed act");
    line.split_ascii_whitespace().map(|word| Box::from(word.as_bytes())).collect()
}

/// Cut the answer after application, before the client's first retry.
pub fn cut(world: &mut World<Temper, Joint>, agent: jig_client_kit::harness::ClientId, words: &[&[u8]]) {
    use jig_client_kit::harness::Until;
    world.fault(FrontFault::NoAnswer);
    give(world, agent, words);
    let start = world.now();
    let first_retry = start.saturating_add(temper_client_tools::limits::client().client.domain.requests.first);
    for _ in 0..ANSWER_STEPS {
        world.run(Until::Steps(2));
        if world.front().answers_to_lose() == 0 {
            assert!(start <= world.now() && world.now() < first_retry, "answer cut before first retry");
            return;
        }
    }
    panic!("answer was not cut");
}

#[must_use]
pub fn model_world(seed: u64) -> World<Temper, Joint> {
    world_with(seed, Joint::with_model_connections(seed, domain_limits()))
}

#[must_use]
pub fn accounts_world(seed: u64) -> World<Temper, Joint> {
    world_with(seed, Joint::with_accounts(seed, domain_limits()))
}

/// Finish the chat's assigned run at the spend this funding story scripts.
pub fn finish_chat(front: &mut Joint, task: u64, spent: u64) {
    let attempt = front.hosted_attempt(task).expect("the chat reached its worker");
    front.engine.answer(
        QUESTION_WORKER,
        task,
        attempt,
        spent,
        tasks::End::Finished {
            result: tasks::TaskResult::Report { words: b"Answered by the chat".as_slice().into() },
            cancel_delegates: false,
        },
        engine::Parts::none(),
    );
    front.engine.settle();
}

/// Temper's kinds configured as in the domain world's settings stories.
#[must_use]
pub fn kinds_world(seed: u64) -> World<Temper, Joint> {
    world_with(seed, Joint::with_kinds(seed, domain_limits()))
}

/// The Ask temper stories offer the stock's charter 11 and template 5.
#[must_use]
pub fn asking_world(seed: u64, member: bool) -> World<Temper, Joint> {
    let mut limits = domain_limits();
    // The proposal's whole reason and charter remain visible in its review.
    limits.client.domain.text = 4096;
    limits.client.domain.long_text = 4096;
    world_with(seed, Joint::with_asking(seed, limits, member))
}

pub(crate) fn asking_configuration(seed: u64, member: bool) -> engine::Config {
    use jig_domain_authority as authority;
    let mut config = temper_engine_domain_world::walking::config(seed);
    config.core.kinds =
        temper_service::stock::kinds(&temper_engine_domain_world::walking::limits(), config.core.connectors[0])
            .into_vec()
            .into_iter()
            .filter(|kind| kind.number == 6 || kind.number == 7)
            .collect();
    config.kinds_stock = Box::new([]);
    let llm = jig_protocol_config::Llm {
        endpoints: Box::new([]),
        models: (config.core.settings.models.clone())
            .into_iter()
            .enumerate()
            .map(|(at, model)| jig_domain_core::settings::CatalogueModel {
                first_read: None,
                number: u32::try_from(at).expect("bounded fixture models") + 1,
                model,
                shown: None,
                description: None,
                picker: true,
                withdrawn: false,
                source: jig_domain_core::settings::ModelSource::Stock,
                estimated: false,
            })
            .collect(),

        providers: Box::new([]),
    };
    let stock = temper_service::stock::with_models(
        llm.models.clone(),
        temper_engine_domain_world::fixtures::forge_limits().tasks.result_bytes,
    );
    let asking = stock.charters.iter().find(|charter| charter.number == 11).expect("stock charter 11").clone();
    config.core.defaults.charters = Box::new([config.core.defaults.charters[0].clone(), asking]);
    let template = stock.templates.iter().find(|template| template.number == 5).expect("stock template 5").clone();
    let mut templates = config.core.defaults.templates.to_vec();
    templates.push(template);
    config.core.defaults.templates = templates.into_boxed_slice();
    config.core.defaults.rules.ceiling.tools.0 |=
        jig_domain_core::ToolFamilies::JIG.settings | jig_domain_core::ToolFamilies::JIG.ask.0;
    config.core.defaults.rules.ceiling.notes = authority::Scopes::GOAL;
    config.core.defaults.rules.ceiling.delegation.kinds =
        Box::new([authority::Executor::Charter(1), authority::Executor::Charter(11)]);
    for project in &mut config.core.seed_projects {
        let policy = &mut project.policy.policy;
        policy.ceiling = config.core.defaults.rules.ceiling.clone();
        let owner = &mut policy.roles[0];
        owner.authority.tools.0 |=
            jig_domain_core::ToolFamilies::JIG.settings | jig_domain_core::ToolFamilies::JIG.ask.0;
        owner.authority.notes = authority::Scopes::GOAL;
        owner.authority.delegation.kinds.clone_from(&policy.ceiling.delegation.kinds);
        let mut member = owner.clone();
        member.number = jig_domain_people::Role::Member.number();
        member.decides = authority::Proposals(0);
        policy.roles = Box::new([owner.clone(), member]);
    }
    let mut owner = config.core.owners[0].clone();
    if member {
        owner.identity = temper_engine_domain_forge::user_key(1, 8);
    }
    config.core.owners =
        Box::new([owner.clone(), jig_domain_people::InitialOwner { project: 0, identity: owner.identity }]);
    config
}

/// A second person is the ctl caller, with a member's project role.
pub(crate) fn asking_member(driver: &mut Driver) -> u64 {
    use jig_domain_people as people;
    use skein_lib::ReplyTo;
    driver.sign_in();
    driver.settle();
    let member = driver
        .delivered
        .iter()
        .find_map(|delivery| {
            if let engine::Delivery::WebReply { reply: people::Reply::SignedIn { person, .. }, .. } = delivery {
                Some(*person)
            } else {
                None
            }
        })
        .expect("member signed in");
    let reply = ReplyTo::new(Token::new(2001));
    driver.send(engine::Event::Party(engine::PartyInput::SignedIn {
        held: None,
        reply_to: ReplyTo::new(Token::new(2001)),
        identity: people::Identity {
            key: temper_engine_domain_forge::user_key(1, 8_u64),
            login: b"owner".as_slice().into(),
            name: b"Owner".as_slice().into(),
        },
    }));
    driver.settle();
    let (owner, session) = driver
        .delivered
        .iter()
        .find_map(|delivery| {
            if let engine::Delivery::WebReply { to, reply: people::Reply::SignedIn { person, sign_in, .. } } = delivery
                && *to == reply
            {
                Some((*person, *sign_in))
            } else {
                None
            }
        })
        .expect("second person signed in");
    driver.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(2002)),
        sign_in: session,
        key: [201; 16],
        ask: people::Ask::SetRoles {
            project: 1,
            holdings: Box::new([
                people::Holding { person: member, role: people::Role::Member },
                people::Holding { person: owner, role: people::Role::Owner },
            ]),
        },
    }));
    driver.settle();
    assert!(
        driver.delivered.iter().any(|delivery| matches!(
            delivery,
            engine::Delivery::WebReply {
                reply: people::Reply::Outcome(people::Outcome::RolesSet { project: 1, .. }),
                ..
            }
        )),
        "the ctl caller is now a member"
    );
    member
}

/// The same ctl root as the other stories, under this world's caller.
pub fn asking_agent(world: &mut World<Temper, Joint>) -> jig_client_kit::harness::ClientId {
    let party = world.front().caller;
    let limits = crate::agent::limits().domain;
    world.front().next_client_limits(limits);
    world.client_with(party, inbox(), limits, crate::agent::new)
}

// One goal and two delegated choices, with both executor kinds and one level below the goal.
const PERSON_TASKS: u32 = 3;
const PERSON_KINDS: u32 = 2;
const PERSON_DEPTH: u32 = 2;
// The two walking templates plus reference template 4.
const PERSON_TEMPLATES: u32 = 3;
// Goal and six acts per choice use thirteen retained keys; sixteen leave headroom.
const PERSON_ANSWERS: u32 = 16;
// One whole report reservation plus the joined question fixtures' 512 bytes.
const PERSON_INBOX_BYTES: u32 = PERSON_RESULT_WORDS + 512;
// The person question's report contract (temper/docs/design/domain/agent/agents.md, 2).
const PERSON_RESULT_WORDS: u32 = 4096;
// Three report payloads and the retained task fields fit within 32 KiB.
const PERSON_TRANSCRIPT_BYTES: u32 = 32_768;
/// A ctl root and real engine reproduce template 4 and role 0 delegation.
#[must_use]
pub fn person_task_tools(seed: u64) -> (Joint, crate::app::Root) {
    let mut front = Joint::new(seed, domain_limits());
    let mut limits = front.engine.env.limits;
    limits.authority.executors = PERSON_KINDS;
    limits.tasks.executor_kinds = PERSON_KINDS;
    limits.documents.executors = PERSON_KINDS;
    limits.templates = PERSON_TEMPLATES;
    limits.tasks.contract_choices = 2; // Two choices may admit the same reason.
    limits.tasks.tasks = PERSON_TASKS;
    limits.tasks.project_tasks = PERSON_TASKS;
    limits.tasks.tree_tasks = PERSON_TASKS;
    limits.people.inbox_tasks = PERSON_TASKS * 2;
    limits.people.requests = PERSON_ANSWERS;
    limits.tasks.inbox_bytes = PERSON_INBOX_BYTES;
    limits.tasks.result_bytes = PERSON_RESULT_WORDS;
    limits.journal.result_bytes = limits.journal.result_bytes.max(PERSON_RESULT_WORDS);
    limits.forge.client.op_bytes = limits.forge.client.op_bytes.max(PERSON_RESULT_WORDS);
    limits.journal.transcript_bytes = limits.journal.transcript_bytes.max(PERSON_TRANSCRIPT_BYTES);
    let configs = std::array::from_fn(|_| person_task_configuration(seed));
    let forge = front.engine.forge.take();
    front.engine = Driver::configured(temper_engine_domain_world::store::Store::with_limits(&limits), configs, &limits);
    front.engine.forge = forge;
    hello_worker(&mut front.engine);
    front.engine.settle();
    tools_on(front, seed)
}
/// Start a ctl client on a prepared joined engine fixture.
#[must_use]
pub fn tools_on(mut front: Joint, seed: u64) -> (Joint, crate::app::Root) {
    use jig_client_kit::harness::{App, ClientId, Front};
    front.bind(ClientId(0), PARTY);
    let mut root = crate::agent::new(seed).0;
    crate::front::account_acts::event(&mut front, &mut root, Temper::start(inbox(), Vec::new(), 0));
    (front, root)
}
fn person_task_configuration(seed: u64) -> engine::Config {
    use jig_domain_authority::Executor;
    use jig_domain_people::{TemplateBudget, TemplateDeadline, TemplateExecutor};
    let mut config = temper_engine_domain_world::walking::config(seed);
    config.core.defaults.rules.ceiling.delegation.depth = PERSON_DEPTH;
    config.core.defaults.rules.ceiling.delegation.tasks = PERSON_TASKS;
    let kinds: Box<[Executor]> = Box::new([Executor::Charter(1), Executor::Role(0)]);
    config.core.defaults.rules.ceiling.delegation.kinds.clone_from(&kinds);
    let policy = &mut config.core.seed_projects[0].policy;
    policy.policy.ceiling.delegation.depth = PERSON_DEPTH;
    policy.policy.ceiling.delegation.tasks = PERSON_TASKS;
    policy.policy.roles[0].authority.delegation.depth = PERSON_DEPTH;
    policy.policy.roles[0].authority.delegation.tasks = PERSON_TASKS;
    policy.policy.ceiling.delegation.kinds.clone_from(&kinds);
    policy.policy.roles[0].authority.delegation.kinds = kinds;
    for permission in &mut policy.permissions {
        permission.role = 0;
    }
    config.core.defaults.policies.insert(1, policy.clone()).expect("policy 1");
    let mut templates = config.core.defaults.templates.to_vec();
    let mut goal = templates[1].clone();
    goal.number = 4;
    goal.executor = TemplateExecutor::Charter(1);
    goal.budget = TemplateBudget::AsAllowed { cap: None };
    goal.deadline = TemplateDeadline::None;
    templates.push(goal);
    config.core.defaults.templates = templates.into_boxed_slice();
    config
}

fn hello_worker(driver: &mut Driver) {
    let mut event = temper_engine_domain_world::protocol::hello_slots(QUESTION_WORKER, 1, Box::new([]));
    if let engine::Event::Worker(engine::WorkerInput::Link { link: engine::Link::Hello { hello, .. }, .. }) = &mut event
    {
        hello.bounds.message_bytes = driver.env.limits.tasks.message_bytes;
    }
    driver.send(event);
}

/// Whole browser-sized lines, with the world's two-row activity pages.
#[must_use]
pub fn long_messages_world(seed: u64) -> World<Temper, Joint> {
    use temper_engine_domain_world::{store::Store, walking};
    let limits = whole_turn_limits();
    let small = Joint::with_reading_pages(seed, &limits);
    let mut engine_limits = small.engine.env.limits;
    drop(small);
    engine_limits.people.words = 8192;
    engine_limits.people.requests = 128;
    engine_limits.journal.result_bytes = engine_limits.journal.result_bytes.max(engine_limits.people.words);
    engine_limits.tasks.message_bytes = 10_660;
    engine_limits.tasks.inbox_bytes = 131_072;
    engine_limits.tasks.inbox_messages = 128;
    engine_limits.people.inbox_recipients = 2 * engine_limits.tasks.inbox_messages + engine_limits.authority.roles + 4;
    let front = Joint::with_engine(
        seed,
        &limits,
        Store::with_limits(&engine_limits),
        [walking::config(seed), walking::config(seed)],
        &engine_limits,
        1,
    );
    world_with(seed, front)
}

pub fn long_messages_client(
    world: &mut World<Temper, Joint>,
    address: temper_client_domain::Address,
) -> jig_client_kit::harness::ClientId {
    let limits = whole_turn_limits();
    world.front().next_client_limits(limits);
    world.client_with(PARTY, address, limits, long_messages_root)
}
fn long_messages_root(seed: u64) -> (crate::app::Root, crate::app::View) {
    let limits = whole_turn_limits();
    let mut words = phrasebooks(&limits);
    words.shell = view::words::Shell::Browser;
    let bounds = view_bounds(&limits, &words);
    (
        crate::app::Root::Person(Box::new(temper_client_domain::unlinked(&limits, seed))),
        crate::app::View::Person(Box::new(view::View::new(&bounds, &limits, words, FORGE))),
    )
}

/// A person's message written at the engine, as a ctl caller writes it.
pub fn long_message(front: &mut Joint, task: u64, number: u8, words: &[u8]) {
    use jig_domain_people::{Ask, Outcome, Reply};
    let driver = &mut front.engine;
    let reply_to = skein_lib::ReplyTo::new(Token::new(9000 + u64::from(number)));
    driver.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: skein_lib::ReplyTo::new(Token::new(9000 + u64::from(number))),
        sign_in: driver.session(),
        key: {
            let mut key = [77; 16];
            key[0] = number;
            key
        },
        ask: Ask::Say { project: 1, task, words: words.into() },
    }));
    driver.settle();
    assert!(
        driver.delivered.iter().any(|delivery| matches!(delivery,
            engine::Delivery::WebReply { to, reply: Reply::Outcome(Outcome::Said { task: named, .. }) }
            if *to == reply_to && *named == task
        )),
        "the engine admitted message {number}"
    );
}

pub fn long_messages_task(front: &mut Joint, agent: bool) -> u64 {
    let driver = &mut front.engine;
    driver.sign_in();
    driver.settle();
    let mut ask = driver.chat_ask(1, b"Read long messages");
    if agent {
        let jig_domain_people::Ask::Create { batch, .. } = &mut ask else { panic!("create") };
        batch[0].label = 0;
    }
    driver.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: skein_lib::ReplyTo::new(CHAT_REPLY),
        sign_in: driver.session(),
        key: CHAT_KEY,
        ask,
    }));
    driver.settle();
    driver.last_assignment().task
}
