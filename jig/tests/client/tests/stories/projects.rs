//! Project requests through the application's public client, on the scripted engine.
use jig_client::{Asker, Event, Key, contract as c};
use jig_client_scripted_engine::Change;
use jig_client_world::{ClientId, FrontFault, Scripted, Test, Until, World, stories};
use skein_lib::{Duration, Time, Wall};
type ProjectWorld = World<Test, Scripted>;
const PARTY: u64 = 7;
const PROJECT: u32 = 1;
const POLICY: u32 = 1;

fn text(value: &[u8]) -> c::Text {
    c::Text::new(value.into(), 64).expect("project story fixture or delivered fact")
}
fn ready(owner: bool) -> (ProjectWorld, ClientId) {
    let mut world = stories::world(51);
    let mut frame = stories::frame(PARTY);
    frame.deployment_role = if owner { Some(c::HeldRole::Owner) } else { None };
    frame.projects.get_mut(0).expect("project story fixture or delivered fact").role = c::HeldRole::Owner;
    world
        .front()
        .engine
        .act(Time::ZERO, Wall::EPOCH, Change::Me(frame))
        .expect("project story fixture or delivered fact");
    let client = world.client(PARTY, stories::inbox());
    settle(&mut world);
    (world, client)
}
fn settle(world: &mut ProjectWorld) {
    world.run(Until::Time(world.now().saturating_add(Duration::from_millis(80))));
}
fn send(world: &mut ProjectWorld, client: ClientId, number: u64, ask: c::Ask) -> Key {
    let key = Key { asker: Asker::JIG, number };
    world.event(
        client,
        jig_client_test_domain::Event::Client {
            event: Event::Send { name: None, key, request: jig_client::Ask::Jig(Box::new(ask)) },
        },
    );
    key
}
fn make(name: &[u8]) -> c::Ask {
    c::Ask::MakeProject { name: text(name), policy: POLICY }
}
fn answer(world: &mut ProjectWorld) -> c::Answer {
    world
        .front()
        .engine
        .commits()
        .iter()
        .rev()
        .find_map(|commit| commit.answer.clone())
        .expect("project story fixture or delivered fact")
}
fn frame(world: &ProjectWorld, client: ClientId) -> &c::Me {
    let root = world.root(client);
    root.jig().frame(root.client()).document.expect("project story fixture or delivered fact")
}
fn project_made(world: &mut ProjectWorld) -> u32 {
    let c::Answer::Done(c::Outcome::Owner(c::OwnerOutcome::ProjectMade { project })) = answer(world) else {
        panic!("make answered ProjectMade");
    };
    project
}

#[test]
fn a_deployment_owner_makes_a_project_and_finds_it_in_me() {
    let (mut world, client) = ready(true);
    let key = send(&mut world, client, 91, make(b"Tally"));
    settle(&mut world);
    let project = project_made(&mut world);
    assert!(project > PROJECT);
    let made = frame(&world, client)
        .projects
        .iter()
        .find(|p| p.project == project)
        .expect("project story fixture or delivered fact");
    assert_eq!(made.name.bytes(), b"Tally");
    assert_eq!(made.role, c::HeldRole::Owner);
    assert!(world.root(client).client().pending(key).is_none(), "the client received its answer");
    assert!(!world.front().engine.truth().frames[&8].projects.iter().any(|p| p.project == project));
}

#[test]
fn an_owner_renames_a_project_and_me_says_the_new_name() {
    let (mut world, client) = ready(true);
    send(&mut world, client, 91, c::Ask::Rename { project: PROJECT, name: text(b"Tally") });
    settle(&mut world);
    assert_eq!(answer(&mut world), c::Answer::Done(c::Outcome::Owner(c::OwnerOutcome::Renamed { project: PROJECT })));
    assert_eq!(
        frame(&world, client).projects.get(0).expect("project story fixture or delivered fact").name.bytes(),
        b"Tally"
    );
    assert_eq!(
        world.front().engine.truth().frames[&8]
            .projects
            .get(0)
            .expect("project story fixture or delivered fact")
            .name
            .bytes(),
        b"Tally"
    );
}

#[test]
fn a_second_make_of_one_name_is_refused_as_name_taken() {
    let (mut world, client) = ready(true);
    send(&mut world, client, 91, make(b"Tally"));
    settle(&mut world);
    let project = project_made(&mut world);
    send(&mut world, client, 92, make(b"tally"));
    settle(&mut world);
    assert_eq!(answer(&mut world), c::Answer::Refused { refusal: c::Refusal::NameTaken, at: None });
    assert_eq!(frame(&world, client).projects.iter().filter(|p| p.project == project).count(), 1);
}

#[test]
fn a_make_by_a_party_who_owns_no_deployment_is_refused_by_role() {
    let (mut world, client) = ready(false);
    send(&mut world, client, 91, make(b"Tally"));
    settle(&mut world);
    assert_eq!(answer(&mut world), c::Answer::Refused { refusal: c::Refusal::Role { held: None }, at: None });
    assert_eq!(frame(&world, client).projects.len(), 1);
}

#[test]
fn a_name_of_digits_alone_is_refused_naming_its_rule() {
    let (mut world, client) = ready(true);
    send(&mut world, client, 91, make(b"123"));
    settle(&mut world);
    assert_eq!(
        answer(&mut world),
        c::Answer::Refused {
            refusal: c::Refusal::Malformed { at: 0, why: c::Malformed::Name(c::NameRule::Digits) },
            at: None
        }
    );
}

#[test]
fn a_make_retried_under_its_key_after_a_reload_is_answered_once() {
    let (mut world, client) = ready(true);
    world.fault(FrontFault::NoAnswer);
    let key = send(&mut world, client, 91, make(b"Tally"));
    assert!(world.root(client).client().pending(key).is_some());
    world.reload(client);
    settle(&mut world);
    let project = project_made(&mut world);
    assert_eq!(world.front().engine.commits().iter().filter(|commit| commit.key.is_some()).count(), 1);
    assert_eq!(frame(&world, client).projects.iter().filter(|p| p.project == project).count(), 1);
    assert!(world.root(client).client().pending(key).is_none());
}

#[test]
fn an_owner_deletes_a_project_by_its_name() {
    use jig_client_view::address::{Address, ProjectSection};
    use skein_fake_person::{Find, Step};
    use skein_ui::Role;
    let (mut world, client) = ready(true);
    let mut me = frame(&world, client).clone();
    me.projects.get_mut(0).expect("project deletion fixture or served fact").name =
        c::DocumentText::complete(text(b"Tally"));
    world.front().engine.act(Time::ZERO, Wall::EPOCH, Change::Me(me)).expect("project deletion fixture or served fact");
    let doc = c::OwnerProject {
        number: PROJECT,
        name: c::DocumentText::complete(text(b"Tally")),
        acts: list([c::Offered { effect: None, act: c::ActKind::RemoveProject, admission: c::Admission::Allowed }]),
        resources: list([]),
        policy: c::OwnerProjectPolicy {
            templates: list([]),
            ceiling: c::OwnerAuthorityDocument::complete(c::OwnerAuthority {
                models: None,
                tools: 0,
                grants: list([]),
                delegation: c::OwnerDelegation { kinds: list([]), tasks: 1, depth: 1 },
                spend: 100,
                deadline: None,
                notes: 0,
                note_resources: list([]),
            }),
            period_spend: 100,
            escalation_role: None,
            projections: list([]),
            roles: list([]),
            requirements: list([]),
            permissions: None,
            permissions_cut: false,
            cut: false,
        },
        members: list([]),
        current_period: None,
        pools: list([]),
        configured_period: 0,
        configured_pools: list([]),
        configured_pools_cut: false,
        spend: list([]),
        recurring: list([]),
        cut: false,
    };
    world
        .front()
        .engine
        .act(Time::ZERO, Wall::EPOCH, Change::OwnerProject(Box::new(doc)))
        .expect("project deletion fixture or served fact");
    world.event(
        client,
        jig_client_test_domain::Event::Act {
            action: jig_client_test_domain::Action::Jig(jig_client_view::action::Act::Go {
                address: Address::Project { project: PROJECT, section: ProjectSection::Settings },
            }),
        },
    );
    let within = Duration::from_secs(1);
    world.person(client, vec![Step::See { find: Find::named(Role::Region, b"Delete project"), within }]);
    assert_eq!(world.run(Until::Done).verdict, skein_world::domain::Verdict::Passed);
    print_delete_page(&world, client);
    world.person(
        client,
        vec![
            Step::Type {
                find: Find::exactly_named(Role::TextField, b"Type the project's name to delete it"),
                words: Box::from(*b"Tally"),
            },
            Step::Press { find: Find::exactly_named(Role::Button, b"Delete project") },
            Step::See { find: Find::text(b"Project Tally deleted"), within },
        ],
    );
    assert_eq!(world.run(Until::Done).verdict, skein_world::domain::Verdict::Passed);
    assert!(matches!(
        world.root(client).jig().page(),
        jig_client_view::page::Page::Work { address: Address::Inbox { project: None }, .. }
    ));
    assert!(!frame(&world, client).projects.iter().any(|p| p.project == PROJECT));
    assert!(!world.front().engine.truth().owner_projects.contains_key(&PROJECT));
    assert!(world.front().engine.truth().frames.values().all(|me| !me.projects.iter().any(|p| p.project == PROJECT)));
    assert_eq!(
        answer(&mut world),
        c::Answer::Done(c::Outcome::Owner(c::OwnerOutcome::ProjectRemoved { project: PROJECT }))
    );
}
fn list<T>(rows: impl IntoIterator<Item = T>) -> skein_lib::List<T> {
    let rows: Vec<_> = rows.into_iter().collect();
    let mut list =
        skein_lib::List::with_capacity(u32::try_from(rows.len()).expect("project deletion fixture or served fact"));
    for row in rows {
        assert!(list.push(row).is_ok());
    }
    list
}

fn print_delete_page(world: &ProjectWorld, client: ClientId) {
    let (_, view) = jig_client_world::test::new(&jig_client_world::test::limits(), 0);
    let mut printed = skein_ui_text::Print::new(&view.limits().tree).expect("owner page print fits");
    skein_ui_text::render(
        world.tree(client),
        b"jig",
        Some(jig_client_view::regions::JigRegion::Link.region()),
        &mut printed,
    );
    let path = skein_scratch::paths::package().join("tests/golden/project_delete.txt");
    if std::env::var_os("SKEIN_CODEGEN_WRITE").is_some_and(|value| value == "1") {
        std::fs::write(&path, printed.text.as_slice()).expect("refresh owner project page golden");
    }
    assert_eq!(printed.text.as_slice(), std::fs::read(path).expect("owner project page golden"));
    if std::env::var_os("SKEIN_CODEGEN_WRITE").is_none() {
        assert_eq!(printed.text.as_slice(), include_bytes!("../golden/project_delete.txt"));
    }
}
