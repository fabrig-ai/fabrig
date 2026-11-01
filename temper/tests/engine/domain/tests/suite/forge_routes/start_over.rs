//! An unfinished owned setup starts again (temper/docs/design/draft/playground-reset.md, 6.4;
//! temper/docs/plan/playground-reset-r4/notes/contracts.md, D5).
use super::*;
use temper_engine_domain_world::forge_routes::owned_template;

fn record(world: &World, forge: u16) -> &forge_top::Forge {
    let Some(Record::Forge { row }) = world.store.rows.get(&Key::Forge(forge_top::Key::Forge(forge))) else {
        panic!("kept forge")
    };
    let forge_top::Stored::Forge(record) = row.as_ref() else { panic!("forge record") };
    record
}

fn setup(world: &mut World, key: u8, project: forge_top::SetUpProject) -> u16 {
    world.setup_requests.clear();
    world.forge_ask(
        key,
        forge_top::Ask::SetUp {
            org: b"org".as_slice().into(),
            repository: b"repo".as_slice().into(),
            project,
            playground: false,
        },
    );
    world.until(Until::Ready);
    let (forge, request) = world
        .setup_requests
        .iter()
        .find_map(|request| match request {
            engine::Request::SetUp { forge, request, org, repository, playground } => {
                assert_eq!(org.as_ref(), b"org");
                assert_eq!(repository.as_ref(), b"repo");
                assert!(!playground);
                Some((*forge, *request))
            }
            _ => None,
        })
        .expect("owned group asked");
    world.send(engine::Event::Forge(Box::new(forge_top::Event::SetUpAnswered {
        forge,
        request,
        answer: forge_top::SetUpAnswer::SettingUp,
    })));
    world.until(Until::Ready);
    assert_eq!(record(world, forge).setup, owned_template().setup, "new group uses the machine's template");
    assert!(record(world, forge).first.is_some(), "intent is kept before its repository");
    assert!(!matches!(world.root.forge_connection(forge), forge_top::Connection::Connected { .. }));
    forge
}

fn removed(world: &mut World, forge: u16) {
    world.forge_ask(110, forge_top::Ask::RemoveForge { forge });
    world.until(Until::Ready);
    assert!(record(world, forge).removing);
    world.box_removed(forge, true);
    world.until(Until::Ready);
    assert!(!world.store.rows.contains_key(&Key::Forge(forge_top::Key::Forge(forge))));
    assert!(world.root.forges().rows.is_empty());
}

fn first_connect(world: &mut World, forge: u16, project: u32) {
    assert!(world.root.forges().rows[0].repositories.is_empty());
    // The setup host makes the kept first intent and reports its repository at connect.
    world.send(engine::Event::Forge(Box::new(forge_top::Event::FirstMade { forge, id: 2 })));
    world.send(engine::Event::Forge(Box::new(forge_top::Event::Connection {
        forge,
        state: forge_top::Connection::Connected { id: 1, login: b"temper".as_slice().into() },
    })));
    world.send(engine::Event::Party(engine::PartyInput::Application {
        scope: people::ApplicationScope::Project(project),
        reply_to: ReplyTo::new(Token::new(120)),
        sign_in: world.signed_in.expect("owner session"),
        key: [120; 16],
        digest: [120; 32],
        carried: None,
        request: engine::Application::Forge(forge_top::Ask::Adopt(forge_top::Adoption {
            project,
            home: true,
            forge,
            host: b"forge.example".as_slice().into(),
            owner: b"org".as_slice().into(),
            name: b"repo".as_slice().into(),
            prefix: b"temper/".as_slice().into(),
            role: forge_top::Role::Owned,
            landing: b"main".as_slice().into(),
            ci: true,
            checks: Box::new([]),
        })),
    }));
    world.until(Until::Adopted);
    world.until(Until::Ready);
    assert!(matches!(world.adopted.as_slice(), [forge_top::ApplicationOutcome::Adopted(_)]), "{:?}", world.adopted);
    assert!(record(world, forge).first.is_none());
    let Some(Record::Forge { row }) = world
        .store
        .rows
        .get(&Key::Forge(forge_top::Key::Repository(skein_forge::api::Repository { forge, repository: 2 })))
    else {
        panic!("first repository adopted")
    };
    assert!(matches!(row.as_ref(), forge_top::Stored::Repository(repository) if repository.project == project));
}

#[test]
fn an_unfinished_owned_forge_starts_over_as_a_new_forge() {
    let mut world = World::boxed_setup();
    world.sign_in();
    let old = setup(&mut world, 101, forge_top::SetUpProject::Existing(1));
    let intent = record(&world, old).first.clone();
    world.restart_removal(world.store.fork());
    world.until(Until::Ready);
    assert_eq!(record(&world, old).first, intent, "the unfinished intent survives the cut");
    removed(&mut world, old);
    let new = setup(&mut world, 111, forge_top::SetUpProject::Existing(1));
    assert!(new > old, "removed numbers are never reused");
    first_connect(&mut world, new, 1);
}

fn default_projects(world: &World) -> Vec<u32> {
    let mut projects = Vec::new();
    for record in world.store.rows.values() {
        if let Record::Core(jig_domain_core::Record::People(people::Stored::Policy { project, name, .. })) = record
            && name.as_ref() == "Default"
        {
            projects.push(*project);
        }
    }
    projects
}

#[test]
fn the_default_project_is_reused_by_the_next_set_up() {
    let mut world = World::boxed_setup();
    world.sign_in();
    // The welcome makes Default before asking for its first forge's repository.
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(99)),
        sign_in: world.signed_in.expect("welcome owner"),
        key: [99; 16],
        ask: people::Ask::MakeProject { name: "Default".into(), policy: 1 },
    }));
    world.until(Until::Ready);
    let projects = default_projects(&world);
    assert_eq!(projects.len(), 1, "the welcome made Default");
    let project = projects[0];
    let old = setup(&mut world, 101, forge_top::SetUpProject::Existing(project));
    removed(&mut world, old);
    let new = setup(&mut world, 111, forge_top::SetUpProject::New(b"dEfAuLt".as_slice().into()));
    assert_eq!(
        record(&world, new).first.as_ref().expect("new first intent").project,
        forge_top::SetUpProject::Existing(project)
    );
    first_connect(&mut world, new, project);
    assert_eq!(default_projects(&world), projects, "start over reuses the same project number");
}
