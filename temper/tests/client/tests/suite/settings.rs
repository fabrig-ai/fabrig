//! Settings at the client/engine meeting point (temper/docs/design/client/pages/settings.md, 2 and temper/docs/design/client/testing.md).
use jig_client_kit::harness::{ClientId, Settings, Until, World};
use skein_fake_person::{Find, Step};
use skein_ui::{Role, State};
use skein_world::domain::Verdict;
use temper_client_domain::Address;
use temper_client_world::{app::Temper, fixtures, front::Joint};
use temper_engine_domain as engine;
use temper_engine_domain_forge::settings::{ProjectWays, Value};
use temper_engine_domain_world::kinds;

use super::asking_chat::{accept, propose};

type ClientWorld = World<Temper, Joint>;
const PROJECT: u32 = 1;
const WAYS_KIND: u32 = 7;
const SETTING: u32 = 1;
const WORKER: skein_lib::Token = skein_lib::Token::new(7);
const ROSTER: u8 = 202;
const OTHER_OWNER: u64 = 8;
const CHANGE_CALL: u64 = 1;
const SWITCHES: [&[u8]; 3] = [b"Chats can write drafts", b"Goals", b"Changes"];

fn see(find: Find) -> Step<Address> {
    Step::See { find, within: fixtures::WITHIN }
}
fn run(world: &mut ClientWorld, person: ClientId, steps: Vec<Step<Address>>) {
    world.person(person, steps);
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");
}
fn switch(world: &ClientWorld, person: ClientId, label: &[u8], on: bool) {
    let node = world
        .tree(person)
        .nodes()
        .iter()
        .find(|node| node.role == Role::Check && node.name.as_deref() == Some(label))
        .expect("tab's switch");
    assert_eq!(node.states.iter().any(|state| *state == State::Pressed), on, "{label:?}");
    assert!(!node.states.iter().any(|state| *state == State::Disabled), "answered switch is enabled");
}
fn work_filter(world: &mut ClientWorld, person: ClientId, label: &[u8], shown: bool) {
    let find = Find::named(Role::Link, label).within(Box::new([skein_fake_person::Target::ExactRole {
        role: Role::Navigation,
        name: b"Work's filters".as_slice().into(),
    }]));
    let observation = if shown { see(find) } else { Step::Gone { find, within: fixtures::WITHIN } };
    run(
        world,
        person,
        vec![
            Step::Press { find: Find::exactly_named(Role::Link, b"Work") },
            see(Find::exactly_named(Role::Navigation, b"Work's filters")),
            observation,
            Step::Press { find: Find::exactly_named(Role::Link, b"How work runs") },
            see(Find::exactly_named(Role::Check, b"Goals")),
        ],
    );
}
fn accepted() -> (ClientWorld, ClientId) {
    let value = Value::Ways(ProjectWays { drafts: true, goals: true, changes: true, way: 0 });
    let (mut world, agent, _, proposal) =
        propose(false, Some(Box::new([kinds::item(WAYS_KIND, PROJECT, SETTING, &value)])));
    fixtures::settings_goal(world.front());
    let person = world.client(fixtures::PARTY, Address::Settings { project: PROJECT });
    run(&mut world, person, vec![see(Find::exactly_named(Role::Check, b"Changes"))]);
    switch(&world, person, b"Changes", false);
    accept(&mut world, agent, &proposal, b"accept-ways");
    work_filter(&mut world, person, "Goals · ".as_bytes(), true);
    (world, person)
}

#[test]
fn an_accepted_proposal_of_a_projects_ways_shows_in_works_filters() {
    let (world, person) = accepted();
    for label in SWITCHES {
        switch(&world, person, label, true);
    }
    let root = world.root(person).person();
    assert!(temper_client_view::pages::chats::drafts_offered(root, PROJECT));
    assert!(root.pages().me(root.client()).expect("me").projects.iter().any(|project| project.project == PROJECT));
}

fn changes_filter_switch() {
    let mut front = Joint::with_settings_changes(fixtures::SEED, &fixtures::domain_limits());
    front.engine.forge =
        Some(temper_engine_domain_world::forge_stand::ForgeStand::with_checks(fixtures::REPOSITORIES, &[b"build"]));
    fixtures::adopt_change_repository(&mut front);
    front.engine.forge.as_mut().expect("fake forge").push_branch(7, b"temper/settings");
    let change = fixtures::goal_with_pushed_change(&mut front, b"temper/settings");
    let mut world =
        World::new(Settings { seed: fixtures::SEED, words: fixtures::decision_words(), ..Settings::default() }, front);
    let owner = world.client(fixtures::PARTY, fixtures::task_page(change));
    let review = super::change_page::review(&mut world, owner, change);
    super::change_page::finish_review(&mut world, review);
    run(&mut world, owner, vec![see(Find::text("Landed · merge commit".as_bytes()))]);
    world.go(owner, Address::Settings { project: PROJECT });
    run(&mut world, owner, vec![see(Find::exactly_named(Role::Check, b"Changes"))]);
    run(&mut world, owner, vec![Step::Press { find: Find::exactly_named(Role::Check, b"Changes") }]);
    switch(&world, owner, b"Changes", true);
    work_filter(&mut world, owner, "Changes · ".as_bytes(), true);
    run(&mut world, owner, vec![Step::Press { find: Find::exactly_named(Role::Check, b"Changes") }]);
    switch(&world, owner, b"Changes", false);
    work_filter(&mut world, owner, "Changes · ".as_bytes(), false);
}

#[test]
fn a_switch_on_the_tab_changes_works_filters_and_a_member_is_refused() {
    changes_filter_switch();
    let (mut world, owner) = accepted();
    run(&mut world, owner, vec![Step::Press { find: Find::exactly_named(Role::Check, b"Goals") }]);
    switch(&world, owner, b"Goals", false);
    work_filter(&mut world, owner, "Goals · ".as_bytes(), false);
    let driver = &mut world.front().engine;
    let reply = skein_lib::ReplyTo::new(skein_lib::Token::new(u64::from(ROSTER)));
    driver.send(engine::Event::Party(engine::PartyInput::SignedIn {
        held: None,
        reply_to: skein_lib::ReplyTo::new(skein_lib::Token::new(u64::from(ROSTER))),
        identity: jig_domain_people::Identity {
            key: temper_engine_domain_forge::user_key(1, OTHER_OWNER),
            login: b"other-owner".as_slice().into(),
            name: b"Other owner".as_slice().into(),
        },
    }));
    driver.settle();
    let other = driver
        .delivered
        .iter()
        .find_map(|delivery| {
            if let engine::Delivery::WebReply { to, reply: jig_domain_people::Reply::SignedIn { person, .. } } =
                delivery
            {
                (*to == reply).then_some(*person)
            } else {
                None
            }
        })
        .expect("second owner signed in");
    let session = driver
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
                (*person == fixtures::PARTY).then_some(*number)
            } else {
                None
            }
        })
        .expect("owner's secret-free sign-in number");
    driver.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: reply,
        sign_in: session,
        key: [ROSTER; 16],
        ask: jig_domain_people::Ask::SetRoles {
            project: PROJECT,
            holdings: Box::new([
                jig_domain_people::Holding { person: other, role: jig_domain_people::Role::Owner },
                jig_domain_people::Holding { person: fixtures::PARTY, role: jig_domain_people::Role::Member },
            ]),
        },
    }));
    driver.settle();
    assert!(
        driver.delivered.iter().any(|delivery| matches!(
            delivery,
            engine::Delivery::WebReply {
                reply: jig_domain_people::Reply::Outcome(jig_domain_people::Outcome::RolesSet { project: PROJECT, .. }),
                ..
            }
        )),
        "{:?}",
        driver.delivered
    );
    let person = owner;
    run(
        &mut world,
        person,
        vec![
            Step::Press { find: Find::exactly_named(Role::Check, b"Goals") },
            see(Find::text(b"The held role does not permit this action")),
        ],
    );
    switch(&world, person, b"Goals", false);
    work_filter(&mut world, person, "Goals · ".as_bytes(), false);
    assert_eq!(
        world.root(person).person().forge().ways_refused(temper_client_domain::forge::ways::Control::Ways),
        Some(&jig_client::contract::Refusal::Role { held: Some(jig_client::contract::HeldRole::Member) })
    );
}

#[test]
fn a_projects_questions_set_on_the_tab_reach_its_next_change() {
    let mut world = World::new(
        Settings { seed: fixtures::SEED, words: fixtures::decision_words(), ..Settings::default() },
        Joint::with_settings_changes(fixtures::SEED, &fixtures::domain_limits()),
    );
    fixtures::adopt_change_repository(world.front());
    let person = world.client(fixtures::PARTY, Address::Settings { project: PROJECT });
    run(&mut world, person, vec![see(Find::named(Role::Select, b"Questions a goal can answer from its plan"))]);
    let option = world
        .tree(person)
        .nodes()
        .iter()
        .find(|node| node.role == Role::Option && node.name.as_deref() == Some(b"Asked of me too".as_slice()))
        .expect("up option")
        .id;
    world.act(person, option, skein_ui::Act::Press);
    run(&mut world, person, vec![see(Find::named(Role::Option, b"Asked of me too"))]);
    assert_eq!(
        world.root(person).person().forge().questions(PROJECT),
        Some(temper_client_domain::forge::ways::Route::Up)
    );
    let agent = fixtures::agent(&mut world);
    let call = fixtures::give(
        &mut world,
        agent,
        &[
            b"chat",
            b"start",
            b"--project",
            b"1",
            b"--template",
            b"1",
            b"Make the next change",
            b"--key",
            b"next-change",
        ],
    );
    let printed = fixtures::answered(&mut world, agent, call);
    assert_eq!(printed.status, 0, "{printed:?}");
    let chat = fixtures::done_task(&printed);
    let driver = &mut world.front().engine;
    let attempt = driver.task(chat).expect("chat run").attempt;
    driver.call(
        WORKER,
        chat,
        attempt,
        CHANGE_CALL,
        engine::Call {
            completion: 1,
            position: 0,
            tool: engine::Tool::Change {
                changes: Box::new([engine::ChangeCall::words(
                    b"Follow the project's questions".as_slice().into(),
                    None,
                )]),
            },
        },
    );
    // The chat holds the worker; admission is observed with its fleet deadline armed.
    driver.pump();
    let Some(engine::CallAnswer::Delegated(changes)) = driver.answered(CHANGE_CALL) else {
        panic!("change delegated: {:?}", driver.answered(CHANGE_CALL))
    };
    let [change] = changes.as_ref() else { panic!("one change") };
    let row = driver
        .store
        .rows
        .values()
        .find_map(|record| {
            if let engine::Record::Forge { row } = record
                && let temper_engine_domain_forge::Stored::Change(row) = row.as_ref()
                && row.task == change.task
            {
                Some(row.clone())
            } else {
                None
            }
        })
        .expect("stored change");
    assert_eq!(row.questions.route, temper_engine_domain_forge_change::Route::Up);
}
