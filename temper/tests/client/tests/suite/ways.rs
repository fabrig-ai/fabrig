//! The stock ways' acts cross the client and engine's commit boundary
//! (temper/docs/design/client/pages/README.md and temper/docs/design/client/pages/settings.md, 2; temper/docs/design/testing/README.md, 2.1).
use jig_client::contract as c;
use jig_client_kit::harness::{App, ClientId, Settings, Until, World};
use skein_ui::{Node, Role, State as UiState};
use skein_world::domain::Verdict;
use temper_client_domain::{
    Action, Address, Event,
    forge::stock_ways::{State, StockWays, WayAct},
    pages::{Act, home},
};
use temper_client_view::look;
use temper_client_world::{
    app::Temper,
    fixtures,
    front::{
        Joint,
        ways::{self, Reader, Repository},
    },
};
use temper_engine_domain as engine;
use temper_engine_domain_forge::{self as forge, settings};
use temper_engine_domain_world::kinds;

const PROJECT: u32 = 1;
const APPROVE: u32 = 0;
const REVIEW: u32 = 1;
const DRAFT: u32 = 2;
type Clients = World<Temper, Joint>;

fn world(reader: Reader) -> (Clients, ClientId) {
    with_repository(reader, None)
}
fn with_repository(reader: Reader, repository: Option<Repository>) -> (Clients, ClientId) {
    let mut world = World::new(
        Settings { seed: fixtures::SEED, words: fixtures::decision_words(), ..Settings::default() },
        ways::with_repository(fixtures::SEED, &fixtures::domain_limits(), reader, repository),
    );
    world.randomness(0, 0, 0);
    let caller = world.front().caller;
    let person = world.client(caller, Address::Ask { project: PROJECT });
    settle(&mut world);
    (world, person)
}
fn settle(world: &mut Clients) {
    let outcome = world.run(Until::Time(world.now().saturating_add(fixtures::WITHIN)));
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");
}
fn page(world: &Clients, person: ClientId) -> &StockWays {
    world.root(person).person().pages().stock_ways().expect("Ask holds StockWays")
}
fn panel(world: &Clients, person: ClientId) -> &temper_client_protocol_wire::v1::WaysPanel {
    let State::Shown(panel) = page(world, person).state() else { panic!("ways shown: {:?}", page(world, person)) };
    panel
}
fn act(world: &mut Clients, person: ClientId, act: Act) {
    world.event(person, Event::Act { action: Action::Pages(act) }.into());
    settle(world);
}
fn current(world: &Clients, person: ClientId, index: u32, name: &[u8]) {
    assert_eq!(*panel(world, person).current(), Some(index));
    let tabs = page(world, person).tabs();
    let current = tabs.iter().filter(|tab| tab.current).collect::<Vec<_>>();
    assert_eq!(current.len(), 1);
    assert_eq!(current[0].index, Some(index));
    assert_eq!(current[0].name.as_ref(), name);
    let tabs = under(world.tree(person).nodes(), Role::Tabs, b"Ways of working");
    let (at, _) = tabs
        .iter()
        .enumerate()
        .filter(|(_, node)| node.role == Role::Tab)
        .nth(usize::try_from(index).expect("tab index"))
        .expect("current tab drawn");
    assert_eq!(words(subtree(tabs, at)), format!("{}, now", String::from_utf8_lossy(name)));
}

fn subtree(nodes: &[Node], at: usize) -> &[Node] {
    &nodes[at..at + usize::try_from(nodes[at].size).expect("subtree size")]
}
fn under<'a>(nodes: &'a [Node], role: Role, name: &[u8]) -> &'a [Node] {
    let at = nodes
        .iter()
        .position(|node| node.role == role && node.name.as_deref() == Some(name))
        .unwrap_or_else(|| panic!("{role:?} {} drawn", String::from_utf8_lossy(name)));
    subtree(nodes, at)
}
fn words(nodes: &[Node]) -> String {
    nodes
        .iter()
        .flat_map(|node| node.text.as_slice())
        .map(|run| std::str::from_utf8(&run.text).expect("page words"))
        .collect()
}
fn named(nodes: &[Node], role: Role, name: &[u8]) -> bool {
    nodes.iter().any(|node| node.role == role && node.name.as_deref() == Some(name))
}
fn no_acts(nodes: &[Node]) {
    assert!(!named(nodes, Role::Button, b"Use this"), "no Use this");
    assert!(!named(nodes, Role::Button, b"Keep it"), "no Keep it");
}
fn drawn_path(nodes: &[Node]) -> String {
    under(nodes, Role::List, b"The way a change goes")
        .iter()
        .filter(|node| node.role == Role::Item)
        .map(|node| std::str::from_utf8(node.name.as_deref().expect("named stop")).expect("stop words"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn roles_fixture() -> Joint {
    use temper_engine_domain_world::{direct::Driver, oracle, store::Store};
    let limits = fixtures::domain_limits();
    let mut bounds = temper_client_world::front::engine_limits();
    // The stock's kind 9 lines run up to charter 12 beside these six charters.
    bounds.tasks.charters = 16;
    let mut config = kinds::configuration(&bounds);
    let mut charters = config.core.defaults.charters.into_vec();
    for number in [9, 11, 12] {
        let mut charter = charters[0].clone();
        charter.number = number;
        charters.push(charter);
    }
    config.core.defaults.charters = charters.into_boxed_slice();
    let mut templates = config.core.defaults.templates.into_vec();
    for (number, charter) in [(5, 11), (6, 12)] {
        let mut template = templates[0].clone();
        template.number = number;
        template.executor = jig_domain_people::TemplateExecutor::Charter(charter);
        templates.push(template);
    }
    config.core.defaults.templates = templates.into_boxed_slice();
    let bootstrap = oracle::config::copy(&config);
    let mut driver = Driver::cold(Store::with_limits(&bounds), bootstrap, &bounds);
    driver.settle();
    let store = driver.take_store();
    let restart = oracle::config::copy(&config);
    Joint::with_engine(fixtures::SEED, &limits, store, [config, restart], &bounds, 1)
}

// The request's figures, with main's verify
// (temper/docs/design/draft/stock-ways.status.md, Design departs).
const PATHS: [[&str; 3]; 2] = [
    [
        "you ask, a person\nproduce, an agent\ncheck, code\nrepair, an agent, when needed (tests)\ncheck, code\nyou approve, a person\nconflict, code, when needed\nresolve, an agent, when needed\nmerge, code\nverify, code\nmain, a branch",
        "you ask, a person\nproduce, an agent\ncheck, code\nreview, an agent\nrepair, an agent, when needed (tests, review)\ncheck, code\nconflict, code, when needed\nresolve, an agent, when needed\nmerge, code\nverify, code\nmain, a branch",
        "draft, a person\nplan, an agent\nproduce ×n, an agent\nmerge, code (into goal)\ncheck, code\nrepair, an agent, when needed (tests)\ngoal branch, a branch\ncheck, code\nconflict, code, when needed\nresolve, an agent, when needed\nmerge, code\nverify, code\nmain, a branch",
    ],
    [
        "you ask, a person\nproduce, an agent\nyou approve, a person\nconflict, code, when needed\nresolve, an agent, when needed\nmerge, code\nmain, a branch",
        "you ask, a person\nproduce, an agent\nreview, an agent\nrepair, an agent, when needed (review)\nconflict, code, when needed\nresolve, an agent, when needed\nmerge, code\nmain, a branch",
        "draft, a person\nplan, an agent\nproduce ×n, an agent\nmerge, code (into goal)\ngoal branch, a branch\nconflict, code, when needed\nresolve, an agent, when needed\nmerge, code\nmain, a branch",
    ],
];
const NAMES: [&[u8]; 3] = [b"You approve", b"Agents review", b"Draft first"];
const TAB_WORDS: [&str; 3] = ["You approve", "Agents review, now", "Draft first"];

#[test]
fn the_three_ways_draw_ci_and_tiers_or_neither() {
    let mut stock = World::new(
        Settings { seed: fixtures::SEED, words: fixtures::decision_words(), ..Settings::default() },
        roles_fixture(),
    );
    stock.randomness(0, 0, 0);
    let caller = stock.front().caller;
    let person = stock.client(caller, Address::Settings { project: PROJECT });
    settle(&mut stock);
    let nodes = stock.tree(person).nodes();
    assert!(named(nodes, Role::Heading { level: 2 }, b"Roles"));
    let roles = nodes.iter().filter(|node| node.styles.as_slice().contains(&look::HOW_ROLE)).collect::<Vec<_>>();
    assert_eq!(
        roles.iter().map(|node| node.name.as_deref().expect("role name")).collect::<Vec<_>>(),
        [b"Chat".as_slice(), b"Ask temper", b"Drafter", b"Coding agent", b"Reviewer", b"Status agent"]
    );
    assert_eq!(
        roles.last().expect("status agent").description.as_deref(),
        Some(b"When you look at Home after landings: writes the few lines at the top of Home, from what landed and what waits".as_slice())
    );
    for (repository, paths) in [(Repository::CiAndTiers, PATHS[0]), (Repository::Neither, PATHS[1])] {
        let (mut world, person) = with_repository(Reader::Owner, Some(repository));
        assert_eq!(panel(&world, person).own().repository().as_deref(), Some(b"acme/shop".as_slice()));
        current(&world, person, REVIEW, b"Agents review");
        for (index, (name, expected)) in NAMES.into_iter().zip(paths).enumerate() {
            let tab = u32::try_from(index).expect("tab index");
            act(&mut world, person, Act::WayTab { project: PROJECT, tab });
            let nodes = world.tree(person).nodes();
            let shown = under(nodes, Role::Group, name);
            let line = panel(&world, person).ways().get(tab).expect("read's way").line();
            let at = shown
                .iter()
                .position(|node| node.role == Role::Paragraph && node.styles.as_slice().contains(&look::WAYS_SAYS))
                .expect("way's line drawn");
            assert_eq!(words(subtree(shown, at)).as_bytes(), line, "way's line from the read");
            assert_eq!(drawn_path(shown), expected, "{}", String::from_utf8_lossy(name));
            assert_eq!(
                nodes.iter().filter(|node| node.styles.as_slice().contains(&look::WAYS_PANEL)).count(),
                1,
                "only the shown panel"
            );
            let tabs = under(nodes, Role::Tabs, b"Ways of working");
            let selected = tabs
                .iter()
                .enumerate()
                .filter(|(_, node)| node.role == Role::Tab && node.states.as_slice().contains(&UiState::Selected))
                .collect::<Vec<_>>();
            assert_eq!(selected.len(), 1);
            assert_eq!(words(subtree(tabs, selected[0].0)), TAB_WORDS[index]);
        }
        act(&mut world, person, Act::Go { address: Address::Settings { project: PROJECT } });
        for (index, (name, expected)) in NAMES.into_iter().zip(paths).enumerate() {
            let tabs = under(world.tree(person).nodes(), Role::Tabs, b"Ways of working");
            let tab = tabs.iter().filter(|node| node.role == Role::Tab).nth(index).expect("way's tab").id;
            world.act(person, tab, skein_ui::Act::Press);
            settle(&mut world);
            let nodes = world.tree(person).nodes();
            assert!(
                named(nodes, Role::Heading { level: 1 }, b"Agents review every change, and code lands it"),
                "heading says the current way's headline"
            );
            let tab = u32::try_from(index).expect("tab index");
            assert_eq!(page(&world, person).shown(), Some(tab), "pressed tab is shown");
            assert_eq!(drawn_path(under(nodes, Role::Group, name)), expected, "{}", String::from_utf8_lossy(name));
        }
    }
}

#[test]
fn a_custom_rule_draws_yours_without_checks_or_acts() {
    let (world, person) = with_repository(Reader::Owner, Some(Repository::Yours));
    assert_eq!(panel(&world, person).current(), &None);
    assert_eq!(page(&world, person).shown(), Some(3));
    let nodes = world.tree(person).nodes();
    let tabs = under(nodes, Role::Tabs, b"Ways of working");
    assert_eq!(tabs.iter().filter(|node| node.role == Role::Tab).count(), 4);
    let selected = tabs
        .iter()
        .enumerate()
        .filter(|(_, node)| node.role == Role::Tab && node.states.as_slice().contains(&UiState::Selected))
        .collect::<Vec<_>>();
    assert_eq!(selected.len(), 1);
    assert_eq!(words(subtree(tabs, selected[0].0)), "Yours, now");
    let own = under(nodes, Role::Group, b"Yours");
    assert!(!drawn_path(own).lines().any(|line| line == "check, code"));
    no_acts(nodes);
}

#[test]
fn keep_the_current_way_completes_to_begin_and_is_not_offered_again() {
    let (mut world, person) = world(Reader::Owner);
    current(&world, person, REVIEW, b"Agents review");
    assert_eq!(page(&world, person).act(REVIEW), Some(WayAct::Keep));
    act(&mut world, person, Act::WayStop { project: PROJECT, stop: 2 });
    let review = under(world.tree(person).nodes(), Role::Region, b"Review");
    for label in [b"Model".as_slice(), b"Can", b"Asked to"] {
        assert!(named(review, Role::Term, label));
    }
    let open = review
        .iter()
        .find(|node| node.role == Role::Link && node.name.as_deref() == Some("Open the reviewer ›".as_bytes()))
        .expect("reviewer's role link")
        .id;
    world.act(person, open, skein_ui::Act::Press);
    settle(&mut world);
    let nodes = world.tree(person).nodes();
    let at = nodes.iter().position(|node| node.role == Role::Main).expect("role page's main");
    let main = subtree(nodes, at);
    assert!(named(main, Role::Heading { level: 1 }, b"The reviewer"));
    assert!(named(main, Role::Term, b"Asked to"));
    act(&mut world, person, Act::Go { address: Address::Ask { project: PROJECT } });
    assert!(!named(world.tree(person).nodes(), Role::Region, b"Review"));
    assert!(named(under(world.tree(person).nodes(), Role::Group, b"Agents review"), Role::Button, b"Keep it"));
    let before = kinds::revision(&world.front().engine.store.rows);
    act(&mut world, person, Act::KeepWay { project: PROJECT });
    assert_eq!(kinds::revision(&world.front().engine.store.rows), before + 1, "Keep commits its settings");
    assert_eq!(page(&world, person).refused(), None);
    current(&world, person, REVIEW, b"Agents review");
    assert_eq!(page(&world, person).act(REVIEW), None, "Keep goes after the step is done");
    assert!(!named(world.tree(person).nodes(), Role::Button, b"Keep it"));
    let revision = kinds::revision(&world.front().engine.store.rows);
    let toast = world
        .tree(person)
        .nodes()
        .iter()
        .find(|node| node.styles.as_slice().contains(&look::TOAST))
        .expect("way toast");
    let nodes = world.tree(person).nodes();
    let at = nodes.iter().position(|node| node.id == toast.id).expect("toast position");
    assert_eq!(words(subtree(nodes, at)), format!("Agents review is now this project's own, as revision {revision}."));
    world.event(person, Temper::went(Address::Home { project: PROJECT }));
    settle(&mut world);
    let root = world.root(person).person();
    let facts = home::facts(root, PROJECT, skein_lib::Wall::EPOCH);
    assert!(facts.to_begin(), "repository step is still left: {facts:?}");
    assert!(facts.configured, "Tell temper how you work is done: {facts:?}");
    assert!(
        world.tree(person).nodes().iter().any(|node| {
            node.role == skein_ui::Role::Heading { level: 3 }
                && node.name.as_deref() == Some(b"2. Done. Tell temper how you work".as_slice())
        }),
        "Home draws the completed To begin step"
    );
    world.event(person, Temper::went(Address::Ask { project: PROJECT }));
    settle(&mut world);
    current(&world, person, REVIEW, b"Agents review");
    assert_eq!(page(&world, person).act(REVIEW), None, "a fresh document keeps the choice without Keep");
    assert!(!named(world.tree(person).nodes(), Role::Button, b"Keep it"));
}

#[test]
fn use_draft_first_commits_the_goal_way() {
    let (mut world, person) = world(Reader::Owner);
    assert_eq!(page(&world, person).act(DRAFT), Some(WayAct::Use));
    act(&mut world, person, Act::WayTab { project: PROJECT, tab: DRAFT });
    assert!(named(under(world.tree(person).nodes(), Role::Group, b"Draft first"), Role::Button, b"Use this"));
    let before = kinds::revision(&world.front().engine.store.rows);
    act(&mut world, person, Act::UseWay { project: PROJECT, way: DRAFT });
    assert_eq!(page(&world, person).refused(), None);
    assert_eq!(kinds::revision(&world.front().engine.store.rows), before + 1);
    current(&world, person, DRAFT, b"Draft first");
    assert!(!named(under(world.tree(person).nodes(), Role::Group, b"Draft first"), Role::Button, b"Use this"));
    let saved = world
        .front()
        .engine
        .store
        .rows
        .values()
        .find_map(|record| match record {
            engine::Record::Forge { row } => match row.as_ref() {
                forge::Stored::Setting(row)
                    if row.key
                        == (settings::SettingKey {
                            kind: 7,
                            scope: PROJECT,
                            number: 1,
                            slot: settings::SettingSlot::Change,
                        }) =>
                {
                    Some(row.edit.clone())
                }
                _ => None,
            },
            engine::Record::Core(_) => None,
        })
        .expect("committed kind 7");
    let settings::Edit::Set(settings::SizedValue { value: settings::Value::Ways(value), .. }) = saved else {
        panic!("kind 7 value")
    };
    assert!(value.goals);
    assert_eq!(value.way, 4);
}

#[test]
fn use_you_approve_refuses_a_box_owner_unknown_on_the_forge() {
    let (mut world, person) = world(Reader::BoxOwner);
    assert_eq!(panel(&world, person).own().repository().as_deref(), Some(b"acme/shop".as_slice()));
    let before = panel(&world, person).clone();
    let revision = kinds::revision(&world.front().engine.store.rows);
    act(&mut world, person, Act::WayTab { project: PROJECT, tab: APPROVE });
    act(&mut world, person, Act::UseWay { project: PROJECT, way: APPROVE });
    let Some(c::Refusal::Judged { setting, why, .. }) = page(&world, person).refused() else {
        panic!("approval refusal on StockWays: {:?}", page(&world, person));
    };
    assert_eq!(*setting, c::Setting::Application { kind: 2, scope: PROJECT, number: 1 });
    let why = std::str::from_utf8(why.bytes()).expect("refusal words");
    assert!(
        why.contains(
            "project 1 role 0 on forge forge.example requires 1 distinct known forge people; currently 0 eligible"
        ),
        "{why}"
    );
    assert_eq!(panel(&world, person), &before, "a refusal leaves the document unchanged");
    assert_eq!(kinds::revision(&world.front().engine.store.rows), revision, "refused settings do not commit");
    let shown = under(world.tree(person).nodes(), Role::Group, b"You approve");
    let refusal = shown.iter().position(|node| node.role == Role::Alert).expect("refusal drawn");
    let foot = shown.iter().position(|node| node.styles.as_slice().contains(&look::WAYS_FOOT)).expect("panel foot");
    assert!(refusal >= foot + usize::try_from(shown[foot].size).expect("foot size"), "refusal under the foot");
    assert!(words(subtree(shown, refusal)).contains("requires 1 distinct known forge people; currently 0 eligible"));
    assert!(named(shown, Role::Button, b"Use this"));
}

#[test]
fn a_member_reads_the_ways_without_an_act() {
    let (mut world, person) = world(Reader::Member);
    current(&world, person, REVIEW, b"Agents review");
    let panel = panel(&world, person);
    assert!(!panel.may_change());
    assert_eq!(panel.ways().len(), 3);
    for index in [APPROVE, REVIEW, DRAFT] {
        assert_eq!(page(&world, person).act(index), None);
        act(&mut world, person, Act::WayTab { project: PROJECT, tab: index });
        let nodes = world.tree(person).nodes();
        assert_eq!(
            under(nodes, Role::Tabs, b"Ways of working").iter().filter(|node| node.role == Role::Tab).count(),
            3
        );
        let at = usize::try_from(index).expect("tab index");
        assert_eq!(
            drawn_path(under(nodes, Role::Group, NAMES[at])),
            PATHS[1][at].replace("you approve, a person", "an owner approves, a person")
        );
        no_acts(nodes);
    }
}
