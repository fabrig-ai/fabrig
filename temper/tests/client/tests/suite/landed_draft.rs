//! A landed draft joins the engine's row to Work and its goal
//! (temper/docs/design/domain/forge/drafts.md, 1-3; client/pages/work.md, 1;
//! client/pages/goal.md, 3, From a landed draft).
use jig_client_kit::harness::{ClientId, Settings, Until, World};
use skein_fake_forge_domain::api as raw;
use skein_fake_person::{Find, Step};
use skein_forge::api;
use skein_lib::{Duration, Reader};
use skein_ui::Role;
use skein_world::domain::Verdict;
use temper_client_domain::{Address, WorkFilter, forge::drafts, pages::drafts::State};
use temper_client_protocol_wire::{self as wire, v1 as w};
use temper_client_world::{app::Temper, fixtures, front::Joint};
use temper_engine_domain_forge::settings;
use temper_engine_domain_world::{forge_stand::ForgeStand, oracle, store::Store, walking};

type Clients = World<Temper, Joint>;
const PROJECT: u32 = 1;
const REPOSITORY: api::Repository = api::Repository { forge: 1, repository: 7 };
const PATH: &[u8] = b"docs/design/draft/catch-up.md";
const TITLE: &[u8] = b"Catch-up barrier";
const START: &[u8] = b"Start a goal: Task by task";

fn stand() -> ForgeStand {
    ForgeStand::with_setup(Box::new([raw::Setup {
        id: REPOSITORY.repository,
        name: b"acme/shop".as_slice().into(),
        default: b"main".as_slice().into(),
        tree: Box::new([raw::File {
            path: PATH.into(),
            content: b"# Catch-up barrier\n\nBuild the catch-up barrier.\n".as_slice().into(),
        }]),
        workflows: Box::new([]),
        labels: Box::new([]),
        protection: None,
        hooked: false,
        checks: raw::Checks {
            cues: Box::new([]),
            contexts: Box::new([]),
            latency_min: Duration::ZERO,
            latency_max: Duration::ZERO,
            silent: 0,
            passes: 1000,
            reruns: 0,
            cue: None,
        },
    }]))
}

fn world() -> (Clients, ClientId) {
    let limits = fixtures::domain_limits();
    let mut bounds = temper_client_world::front::engine_limits();
    // Adoption, ready, create, started and the goal page's automatic Seen.
    bounds.people.requests = 8;
    // The goal's path, title, instructions and provenance fit as one spec.
    bounds.tasks.spec_bytes = 512;
    let mut config = walking::config(fixtures::SEED);
    config.core.connectors = Box::new([fixtures::FORGE]);
    config.core.seed_projects[0].policy.permissions = Box::new([]);
    for template in &mut config.core.defaults.templates {
        if template.tracked.is_some() {
            template.deadline = jig_domain_people::TemplateDeadline::None;
        }
    }
    let mut stock = config.kinds_stock.into_vec();
    stock.push(settings::Stock {
        kind: 7,
        scope: PROJECT,
        number: 1,
        value: settings::SizedValue {
            size: 0,
            value: settings::Value::Ways(settings::ProjectWays { drafts: true, goals: true, changes: true, way: 2 }),
        },
    });
    config.kinds_stock = stock.into_boxed_slice();
    let restart = oracle::config::copy(&config);
    let mut front =
        Joint::with_engine(fixtures::SEED, &limits, Store::with_limits(&bounds), [config, restart], &bounds, 1);
    front.engine.forge = Some(stand());
    fixtures::adopt_change_repository(&mut front);
    let mut world =
        World::new(Settings { seed: fixtures::SEED, words: fixtures::decision_words(), ..Settings::default() }, front);
    let person = world.client(fixtures::PARTY, work());
    run(&mut world, person, vec![see(Find::exactly_named(Role::Heading { level: 1 }, b"Nothing in flight yet."))]);
    (world, person)
}

fn work() -> Address {
    Address::Work { project: PROJECT, filter: WorkFilter::Drafts }
}
fn see(find: Find) -> Step<Address> {
    Step::See { find, within: fixtures::WITHIN }
}
fn gone(find: Find) -> Step<Address> {
    Step::Gone { find, within: fixtures::WITHIN }
}
fn run(world: &mut Clients, person: ClientId, steps: Vec<Step<Address>>) {
    world.person(person, steps);
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");
}
fn set(world: &mut Clients, person: ClientId, key: u8, change: w::DraftChange) -> w::LandedDraft {
    let ask = drafts::ask(PROJECT, None, PATH.into(), change, &fixtures::domain_limits().forge).expect("draft ask");
    let answer = world.front().family_request(person, [key; 16], ask);
    assert_eq!((answer.family, answer.version), (wire::FORGE, wire::CLIENT_VERSION));
    let mut reader = Reader::new(&answer.bytes);
    let answer = w::Answer::decode(&w::CEILINGS, &mut reader).expect("family answer");
    assert!(reader.is_empty());
    let w::Outcome::Drafted(row) = answer.into_parts().outcome else { panic!("drafted") };
    row
}
fn row(world: &Clients, person: ClientId) -> w::LandedDraft {
    let drafts = world.root(person).person().pages().drafts().expect("Work watches drafts");
    let State::Shown(panel) = drafts.state() else { panic!("drafts panel: {drafts:?}") };
    let [row] = panel.drafts().as_slice() else { panic!("one draft row: {panel:?}") };
    row.clone()
}
fn ready() -> (Clients, ClientId, w::LandedDraft) {
    let (mut world, person) = world();
    let api::Answer::Commit(head) = world
        .front()
        .engine
        .forge
        .as_mut()
        .expect("forge")
        .read(REPOSITORY, api::Read::Branch { branch: b"main".as_slice().into() })
    else {
        panic!("main head")
    };
    let ready = w::ReadyDraft::new(&w::CEILINGS, w::ReadyDraftParts { chat: None }).expect("ready change");
    let answer = set(&mut world, person, 120, w::DraftChange::Ready(ready));
    assert_eq!(answer.commit(), temper_engine_domain_forge::commit_text(&head).as_ref());
    assert_eq!(answer.branch(), b"main");
    assert_eq!(answer.path(), PATH);
    assert_eq!(answer.title(), TITLE);
    assert!(answer.ready());
    assert_eq!(*answer.goal(), None);
    run(
        &mut world,
        person,
        vec![
            see(Find::exactly_named(Role::Heading { level: 1 }, b"One draft.")),
            see(Find::named(Role::Paragraph, "catch-up.md · landed in docs/design/draft/ · ready since ".as_bytes())),
            see(Find::text(TITLE)),
            see(Find::exactly_named(Role::Button, START)),
        ],
    );
    assert_eq!(row(&world, person), answer, "Work reads the committed drafts subject");
    (world, person, answer)
}

#[test]
fn a_ready_landed_draft_starts_a_goal_and_names_it_in_its_row() {
    let (mut world, person, ready) = ready();
    run(
        &mut world,
        person,
        vec![
            Step::Press { find: Find::exactly_named(Role::Button, START) },
            see(Find::exactly_named(
                Role::Heading { level: 1 },
                b"Planning from catch-up.md: reading it and the designs it cites.",
            )),
        ],
    );
    let Address::Task { task: goal } = *world.root(person).person().pages().address() else {
        panic!("the goal page opens: {:?}", world.root(person).person().pages().address())
    };
    let task = world.front().engine.task(goal).expect("created goal");
    assert!(task.tracked.is_some());
    let words = std::str::from_utf8(&task.spec.words).expect("goal words");
    assert!(words.contains("Build docs/design/draft/catch-up.md."), "{words}");
    assert!(words.contains("Draft: docs/design/draft/catch-up.md"), "{words}");
    assert!(!words.contains("Chat:"), "a landed file is not a chat snapshot: {words}");
    world.go(person, work());
    run(
        &mut world,
        person,
        vec![
            see(Find::exactly_named(Role::Heading { level: 1 }, b"One in flight.")),
            see(Find::text(b"None.")),
            gone(Find::text(b"catch-up.md")),
            gone(Find::text(TITLE)),
            gone(Find::exactly_named(Role::Button, START)),
        ],
    );
    let mut expected = ready.into_parts();
    expected.goal = Some(goal);
    assert_eq!(row(&world, person).into_parts(), expected, "started keeps the draft and names its goal");
}

#[test]
fn saying_a_ready_draft_is_not_ready_removes_it_from_work() {
    let (mut world, person, ready) = ready();
    let answer = set(&mut world, person, 121, w::DraftChange::NotReady);
    let mut expected = ready.into_parts();
    expected.ready = false;
    assert_eq!(answer.into_parts(), expected, "not_ready keeps the rest of the row");
    run(
        &mut world,
        person,
        vec![
            see(Find::exactly_named(Role::Heading { level: 1 }, b"Nothing in flight yet.")),
            see(Find::text(b"None.")),
            gone(Find::text(b"catch-up.md")),
            gone(Find::text(TITLE)),
            gone(Find::exactly_named(Role::Button, START)),
        ],
    );
    assert_eq!(row(&world, person).into_parts(), expected, "the live drafts subject retains the unlisted row");
}
