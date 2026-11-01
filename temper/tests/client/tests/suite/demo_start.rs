//! Playground start and reset through the joined engine (draft/playground-reset.md, 6).
use jig_client_world::harness::{App, ClientId, Duty, Front};
use skein_lib::{Duration, Env, Queue, Time, Wall};
use skein_ui::Role;
use temper_client_domain::ChatsFilter;
use temper_client_domain::{self as d, Address, Place, Step, pages::Act};
use temper_client_world::{
    app::{Root, Temper},
    fixtures as f,
    front::{Joint, account_acts as accounts},
};
use temper_engine_domain as engine;
use temper_engine_domain_forge as forge;

fn pump(front: &mut Joint, root: &mut Root, event: d::Event) -> usize {
    pump_at(front, root, Some(event), Time::from_nanos(Duration::from_secs(1).as_nanos()))
}
fn pump_at(front: &mut Joint, root: &mut Root, event: Option<d::Event>, mut now: Time) -> usize {
    let limits = f::domain_limits();
    let mut out = Queue::uncapped();
    let env = Env { now, wall: Wall::EPOCH, limits };
    if let Some(event) = event {
        Temper::step(root, &env, event.into(), &mut out);
    } else {
        Temper::fire(root, &env, &mut out);
    }
    let mut replaced = 0;
    for _ in 0..64 {
        Temper::take(root, &Env { now, wall: Wall::EPOCH, limits }, &mut out);
        while let Some(request) = out.pop() {
            if let d::Request::Show { address, replace } = &request {
                replaced += usize::from(*replace);
                Temper::step(
                    root,
                    &Env { now, wall: Wall::EPOCH, limits },
                    d::Event::Went { place: Place::Address(address.clone()) }.into(),
                    &mut out,
                );
            }
            if let Duty::Wire(request) = Temper::duty(request) {
                front.wire(ClientId(0), now, Wall::EPOCH, request);
            }
        }
        now = now.saturating_add(Duration::from_millis(3));
        let callbacks = front.poll(now, Wall::EPOCH);
        if callbacks.is_empty() {
            return replaced;
        }
        for (_, event) in callbacks {
            front.received(ClientId(0), &event);
            Temper::step(root, &Env { now, wall: Wall::EPOCH, limits }, event, &mut out);
        }
    }
    panic!("demo story did not settle");
}
fn go(front: &mut Joint, root: &mut Root, address: Address) -> usize {
    pump(front, root, d::Event::Went { place: Place::Address(address) })
}
fn view(root: &Root) -> temper_client_view::View {
    let limits = f::domain_limits();
    let words = f::phrasebooks(&limits);
    let bounds = f::view_bounds(&limits, &words);
    let mut view = temper_client_view::View::new(&bounds, &limits, words, f::FORGE);
    temper_client_view::build(&mut view, root.person(), Wall::EPOCH, &mut Queue::with_capacity(bounds.tree.patches));
    view
}
fn has(root: &Root, role: Role, name: &[u8]) -> bool {
    view(root).tree().nodes().iter().any(|node| node.role == role && node.name.as_deref() == Some(name))
}
fn press(front: &mut Joint, root: &mut Root, name: &[u8]) {
    activate(front, root, Role::Button, name);
}
fn activate(front: &mut Joint, root: &mut Root, role: Role, name: &[u8]) {
    let mut view = view(root);
    let id = view
        .tree()
        .nodes()
        .iter()
        .find(|node| node.role == role && node.name.as_deref() == Some(name))
        .expect("button")
        .id;
    let action = temper_client_view::act(&mut view, id, skein_ui::Act::Press).expect("bound start");
    pump(front, root, d::Event::Act { action });
}
#[test]
fn try_the_playground_starts_it_on_the_engine_in_its_project() {
    let (mut front, mut root) = accounts::playground_fixture(f::SEED);
    assert!(has(&root, Role::Heading { level: 1 }, b"Welcome to temper"));
    assert!(has(&root, Role::Button, b"Try the playground"));
    assert!(!has(&root, Role::Region, b"The playground"));
    assert!(front.playground().is_none());
    press(&mut front, &mut root, b"Try the playground");
    assert_eq!(front.playground().expect("running row").state, forge::PlaygroundState::Running);
    assert_eq!(root.person().pages().address(), &Address::Chats { filter: ChatsFilter::All, start: false });
    let row = front.playground().expect("running row");
    assert_eq!(root.person().pages().project(), row.project);
    assert_ne!(row.project, Some(1), "engine makes a playground project");
    assert!(front.engine_adopted(row.forge, f::PLAYGROUND_REPOSITORY));
    assert!(
        front
            .records()
            .iter()
            .any(|record| record.number == row.forge && record.playground && record.mode == forge::Mode::Fake)
    );
    assert!(front.engine.forge.as_ref().expect("fake forge stand").calls > 0);
    assert!(front.engine.store.rows.values().any(|record| matches!(record, engine::Record::Core(jig_domain_core::Record::Accounts(jig_domain_accounts::Stored::Account { number, uses: Some(uses), .. })) if Some(*number) == row.account && uses.as_ref() == [jig_domain_accounts::Use::Project(row.project.expect("project"))])));
    assert!(has(&root, Role::Region, b"The playground"));
    assert!(!has(&root, Role::Region, b"Setup"));
    assert!(has(&root, Role::Status, b"Ready"));
    pump(&mut front, &mut root, d::Event::Act { action: d::Action::Pages(Act::Playground) });
    assert_eq!(front.playground().expect("running row").state, forge::PlaygroundState::Running);
}

#[test]
fn failed_start_survives_navigation_and_try_again_uses_a_new_key() {
    let (mut front, mut root) = accounts::playground_fixture(f::SEED);
    front.playground_key_available = false;
    press(&mut front, &mut root, b"Try the playground");
    assert_eq!(front.playground().expect("start kept its project").account, None);
    assert_eq!(root.person().forge().demo_failure(), Some(&d::forge::demo::StartFailure::Account));
    assert!(has(&root, Role::Alert, b"The playground did not start: the scripted model's account was refused."));
    go(&mut front, &mut root, Address::Accounts);
    go(&mut front, &mut root, Address::Chats { filter: ChatsFilter::All, start: false });
    assert!(has(&root, Role::Button, b"Try again"));
    let before = front.applications_admitted();
    front.playground_key_available = true;
    press(&mut front, &mut root, b"Try again");
    assert!(front.applications_admitted() > before, "retry is admitted with a fresh key");
    assert!(has(&root, Role::Status, b"Ready"));
    assert!(!has(&root, Role::Button, b"Try again"));
}
#[test]
fn running_read_sends_no_start_and_welcome_moves_once() {
    let (mut front, mut root) = accounts::running_playground_fixture(f::SEED);
    let before = front.applications_admitted();
    assert_eq!(root.person().pages().address(), &Address::Chats { filter: ChatsFilter::All, start: false });
    assert!(!has(&root, Role::Status, b"Starting"));
    let replaces = go(&mut front, &mut root, Address::Welcome { step: Step::Welcome });
    assert_eq!(replaces, 1);
    assert_eq!(front.applications_admitted(), before);
    go(&mut front, &mut root, Address::Welcome { step: Step::Ready });
    assert!(!has(&root, Role::Region, b"The playground"));
    assert!(!has(&root, Role::Region, b"Setup"));
}

fn submit_chat(front: &mut Joint, root: &mut Root) -> bool {
    let mut view = view(root);
    let field = view
        .tree()
        .nodes()
        .iter()
        .find(|node| node.role == Role::TextArea && node.name.as_deref() == Some(b"Start a chat"))
        .expect("chat composer")
        .id;
    let Some(action) = temper_client_view::act(&mut view, field, skein_ui::Act::Submit) else { return false };
    pump(front, root, d::Event::Act { action });
    true
}

fn delegate_to_repository(front: &mut Joint, root: &Root) -> temper_engine_domain::CallAnswer {
    use jig_domain_tasks as tasks;
    use temper_engine_domain as engine;
    let Address::Chat { task } = *root.person().pages().address() else { panic!("chat opened") };
    let attempt = front.hosted_attempt(task).expect("chat assigned to the scripted worker");
    let row = front.playground().expect("running playground");
    let mut delegate = temper_engine_domain_world::fixtures::report_delegate(b"Change the title", Box::new([]));
    delegate.authority.tools = tasks::Tools(0);
    delegate.spec.parameters = Box::new([tasks::Parameter::Resource {
        name: 1,
        connector: f::FORGE,
        resource: temper_engine_domain_forge::tag(row.forge, f::PLAYGROUND_REPOSITORY),
    }]);
    front.engine.call(
        skein_lib::Token::new(7),
        task,
        attempt,
        99,
        engine::Call { completion: 1, position: 1, tool: engine::Tool::Delegate { batch: Box::new([delegate]) } },
    );
    // The parent occupies the fixture's worker; an admitted child can wait for
    // placement. Drain immediate work without requiring that deadline to clear.
    front.engine.pump();
    front.engine.answered(99).expect("real delegation answer").clone()
}

#[test]
fn enter_waits_for_adoption_then_the_chat_can_delegate() {
    let (mut front, mut root) = accounts::playground_fixture(f::SEED);
    front.hold_playground_connection = true;
    pump(&mut front, &mut root, d::Event::Act { action: d::Action::Pages(Act::Playground) });
    assert!(root.person().forge().demo_starting());
    assert!(root.person().demo().is_some(), "model is usable before adoption");
    let row = front.playground().expect("start made the row");
    assert!(!front.engine_adopted(row.forge, f::PLAYGROUND_REPOSITORY));
    pump(
        &mut front,
        &mut root,
        d::Event::Act { action: d::Action::Pages(Act::ChatWords { words: b"Change the title".as_slice().into() }) },
    );
    if submit_chat(&mut front, &mut root) {
        assert_eq!(f::chats(&front.engine), 1, "Enter started a chat while adoption was held");
        let answer = delegate_to_repository(&mut front, &root);
        assert!(
            matches!(&answer, temper_engine_domain::CallAnswer::DelegationRefused(problem)
            if problem.why == jig_domain_tasks::Refusal::Holds),
            "missing repository: {answer:?}"
        );
        panic!("Enter started the chat before adoption; its delegation was refused: {answer:?}");
    }
    assert_eq!(f::chats(&front.engine), 0, "Enter waits without creating a chat");
    front.connect_playground();
    accounts::settle(&mut front, &mut root);
    assert!(!root.person().forge().demo_starting());
    assert!(has(&root, Role::Status, b"Ready"));
    assert!(submit_chat(&mut front, &mut root), "Enter now starts the kept draft");
    assert_eq!(f::chats(&front.engine), 1);
    assert!(matches!(delegate_to_repository(&mut front, &root), temper_engine_domain::CallAnswer::Delegated(_)));
}

fn assert_reset(front: &Joint, row: forge::Playground) {
    assert!(front.playground().is_none(), "playground row erased");
    assert!(!front.records().iter().any(|record| record.number == row.forge), "forge row erased");
    let project = row.project.expect("playground project");
    for key in [
        jig_domain_people::Key::Policy(project),
        jig_domain_people::Key::Roles(project),
        jig_domain_people::Key::Deletion(project),
    ] {
        assert!(
            !front.engine.store.rows.contains_key(&engine::Key::Core(jig_domain_core::Key::People(key))),
            "project row erased: {key:?}"
        );
    }
    assert!(
        !front.engine.store.rows.values().any(|record| matches!(record,
        engine::Record::Core(jig_domain_core::Record::Accounts(jig_domain_accounts::Stored::Account { number, .. }))
        if Some(*number) == row.account)),
        "scripted account erased"
    );
}
fn assert_forge_step(root: &Root) {
    assert_eq!(root.person().pages().address(), &Address::Welcome { step: Step::Forge { forge: None } });
    assert_eq!(d::pages::welcome::state(root.person()), Some(d::pages::welcome::State::Empty));
    assert!(has(root, Role::Button, b"Try the playground instead"));
}
#[test]
fn set_up_temper_resets_the_playground_and_opens_the_forge_step() {
    let (mut front, mut root) = accounts::running_playground_fixture(f::SEED);
    let row = front.playground().expect("running playground");
    front.hold_removals = true;
    activate(&mut front, &mut root, Role::Link, b"set up temper");
    assert!(has(&root, Role::Status, "Leaving the playground…".as_bytes()));
    assert!(!has(&root, Role::Link, b"set up temper"));
    assert_eq!(
        front.playground().expect("resetting").state,
        forge::PlaygroundState::Resetting(forge::ResetStep::Forge)
    );
    assert!(!matches!(root.person().pages().address(), Address::Welcome { .. }));
    front.finish_removals();
    accounts::settle(&mut front, &mut root);
    assert_reset(&front, row);
    assert_forge_step(&root);
}
#[test]
fn a_reload_while_resetting_shows_leaving_then_the_forge_step() {
    let (mut front, mut root) = accounts::running_playground_fixture(f::SEED);
    let row = front.playground().expect("running playground");
    front.hold_removals = true;
    activate(&mut front, &mut root, Role::Link, b"set up temper");
    assert_eq!(
        front.playground().expect("reset held at forge").state,
        forge::PlaygroundState::Resetting(forge::ResetStep::Forge)
    );
    accounts::reopen(&mut front, &mut root, f::chats_page(), false);
    assert_eq!(root.person().forge().demo().expect("read resetting").state, d::forge::demo::DemoState::Resetting);
    assert!(has(&root, Role::Status, "Leaving the playground…".as_bytes()));
    front.finish_removals();
    pump_at(&mut front, &mut root, None, Time::from_nanos(Duration::from_secs(4).as_nanos()));
    assert_reset(&front, row);
    assert_forge_step(&root);
}
#[test]
fn a_real_account_before_the_playground_stays_and_shows_ready_at_model() {
    let (mut front, mut root) = accounts::playground_with_real_account_fixture(f::SEED);
    let real: Vec<_> = front
        .engine
        .store
        .rows
        .values()
        .filter_map(|record| {
            if let engine::Record::Core(jig_domain_core::Record::Accounts(jig_domain_accounts::Stored::Account {
                number,
                provider,
                ..
            })) = record
            {
                (*provider == accounts::PROVIDER + 1).then_some(*number)
            } else {
                None
            }
        })
        .collect();
    assert_eq!(real.len(), 1);
    press(&mut front, &mut root, b"Try the playground instead");
    let row = front.playground().expect("running playground");
    activate(&mut front, &mut root, Role::Link, b"set up temper");
    assert_reset(&front, row);
    assert!(front.engine.store.rows.contains_key(&engine::Key::Core(jig_domain_core::Key::Accounts(
        jig_domain_accounts::Key::Account { number: real[0] }
    ))));
    go(&mut front, &mut root, Address::Welcome { step: Step::Model { way: None } });
    assert_eq!(
        d::pages::welcome::model_usable(root.person()),
        Some(true),
        "{:?}",
        d::pages::accounts::document(root.person().client())
    );
    assert!(has(&root, Role::Status, b"done"));
    assert!(has(&root, Role::Link, b"Next"));
}
#[test]
fn forges_and_accounts_offer_nothing_to_add_while_it_runs() {
    let (mut front, mut root) = accounts::running_playground_fixture(f::SEED);
    go(&mut front, &mut root, Address::Forges);
    assert!(has(&root, Role::Heading { level: 2 }, b"The playground"));
    for name in [b"+ Add a forge".as_slice(), b"Connect a forge"] {
        assert!(!has(&root, Role::Link, name));
        assert!(!has(&root, Role::Button, name));
    }
    go(&mut front, &mut root, Address::Accounts);
    assert!(has(&root, Role::Disclosure, "Scripted model · the playground’s · held by the deployment".as_bytes()));
    for name in [b"Connect a model".as_slice(), b"+ Add an account", b"Add an account"] {
        assert!(!has(&root, Role::Link, name));
        assert!(!has(&root, Role::Button, name));
    }
}
#[test]
fn a_begun_real_forge_refuses_the_playground_not_offered() {
    use temper_client_protocol_wire::v1 as w;
    let (mut front, mut root) = accounts::playground_fixture(f::SEED);
    let mut now = Time::from_nanos(Duration::from_secs(1).as_nanos());
    super::connect_forge::run_owned(&mut front, &mut root, &mut now);
    assert_eq!(front.records().len(), 1);
    go(&mut front, &mut root, Address::Welcome { step: Step::Welcome });
    assert!(!has(&root, Role::Button, b"Try the playground"));
    let attempt = skein_lib::Token::new(101);
    front.wire(
        ClientId(0),
        now,
        Wall::EPOCH,
        d::Request::Client {
            request: jig_client::Request::Down {
                down: jig_client::Down::Send {
                    attempt,
                    key: [101; 16],
                    value: None,
                    request: d::forge::demo::ask(&f::domain_limits().forge).expect("start wire request"),
                },
            },
        },
    );
    let outcome = front
        .poll(now, Wall::EPOCH)
        .into_iter()
        .find_map(|(_, event)| {
            if let temper_client_world::app::Event::Root(d::Event::Client {
                event:
                    jig_client::Event::Up {
                        up:
                            jig_client::Up::Answer {
                                attempt: actual,
                                answer: Some(jig_client::Answered::Family(record)),
                                ..
                            },
                    },
            }) = event
                && actual == attempt
            {
                Some(
                    w::Answer::decode(&w::CEILINGS, &mut skein_lib::Reader::new(&record.bytes))
                        .expect("start answer")
                        .into_parts()
                        .outcome,
                )
            } else {
                None
            }
        })
        .expect("engine answered start");
    assert!(matches!(outcome, w::Outcome::StartRefused(refused) if *refused.why() == w::StartRefusal::NotOffered));
    assert!(front.playground().is_none());
    assert_eq!(front.records().len(), 1);
}
