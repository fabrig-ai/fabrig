//! Configuring chats through ctl and scripted runs in the joined world
//! (temper/docs/design/domain/agent.md, 8.1; temper/docs/design/domain/testing.md, 6.3 and 6.4).
use jig_client_kit::harness::{App, ClientId, Front, World};
use jig_domain_core::{SettingsAnswer, SettingsRead, settings as s};
use jig_domain_tasks as tasks;
use skein_lib::Duration;
use temper_client_domain::{Action, Address, Event, pages::Act};
use temper_client_tools::jig::Printed;
use temper_client_world::{
    app::Temper,
    fixtures,
    front::{HostAnswer, Joint, account_acts},
};
use temper_engine_domain as engine;

type ClientWorld = World<Temper, Joint>;
const REASON: &str = "Give chats more time to wait\ntemper enforces\nCharter 1 waits two seconds.\nyour agents follow\nThe chat's instructions stay as written.\nRunning work stays on its held revision.";
const ACCEPTED: &[u8] = b"Accepted: revision 2 made.";
const PROJECT: u32 = 1;
const WORKFLOW: &[u8] = b"One chat per change, landed once CI passes.";

#[test]
fn the_page_starts_the_asking_chat() {
    for (choose, acts) in [false, true].into_iter().flat_map(|choose| {
        [vec![Act::ChatWords { words: WORKFLOW.into() }, Act::StartChat], vec![Act::Starter { words: WORKFLOW.into() }]]
            .map(|acts| (choose, acts))
    }) {
        let mut front = Joint::with_configuring(fixtures::SEED, fixtures::domain_limits(), false);
        let mut root = temper_client_world::app::Root::Person(Box::new(temper_client_domain::unlinked(
            &fixtures::domain_limits(),
            fixtures::SEED,
        )));
        front.bind(ClientId(0), fixtures::PARTY);
        // The domain-only pump joins the real client and engine without drawing Ask temper.
        account_acts::event(&mut front, &mut root, Temper::start(Address::Ask { project: PROJECT }, Vec::new(), 0));
        let person = root.person();
        if jig_client::shapes::template(person.pages().me(person.client()).expect("me"), PROJECT, 5).is_none() {
            continue;
        }
        if choose {
            account_acts::event(
                &mut front,
                &mut root,
                Event::Act {
                    action: Action::Pages(Act::Choice(temper_client_domain::pages::choice::Act::Model {
                        index: 0,
                        account: None,
                    })),
                }
                .into(),
            );
        }
        for act in acts {
            account_acts::event(&mut front, &mut root, Event::Act { action: Action::Pages(act) }.into());
        }
        let assignment = front.assignments.last().expect("configuring run assigned");
        assert_eq!(assignment.charter, 11);
        assert!(assignment.workspace.repositories.is_empty(), "the configuring chat names no repository");
        let task = front.engine.task(assignment.task).expect("configuring chat");
        assert_eq!(task.spec.words.as_ref(), WORKFLOW);
        assert!(task.llm.is_none(), "the default model's row preserves omitted intent in Create");
        let task = assignment.task;
        let person = root.person();
        assert_eq!(person.pages().address(), &Address::Chat { task });
        assert_eq!(person.pages().chat().expect("chat page").task, task);
        account_acts::event(&mut front, &mut root, Temper::went(Address::Settings { project: PROJECT }));
        account_acts::settle(&mut front, &mut root);
        let person = root.person();
        assert_eq!(
            person.pages().chats().expect("Settings holds chats").open_asking(
                person.client(),
                person.pages().me(person.client()),
                PROJECT,
            ),
            Some(task),
            "The rail finds the Ask temper task in the real engine's delivered rows",
        );
    }
}

fn call(world: &mut ClientWorld, agent: ClientId, words: &[&[u8]]) -> Printed {
    let number = fixtures::give(world, agent, words);
    fixtures::answered(world, agent, number)
}

fn text(world: &mut ClientWorld, agent: ClientId, words: &[&[u8]]) -> String {
    let printed = call(world, agent, words);
    assert_eq!(printed.status, 0, "{printed:?}");
    String::from_utf8(printed.stdout.into_vec()).expect("ctl UTF-8")
}

pub(super) fn propose(member: bool, items: Option<Box<[s::Item]>>) -> (ClientWorld, ClientId, u64, String) {
    propose_in(fixtures::configuring_world(fixtures::SEED, member), items)
}
fn propose_in(mut world: ClientWorld, items: Option<Box<[s::Item]>>) -> (ClientWorld, ClientId, u64, String) {
    let agent = fixtures::configuring_agent(&mut world);
    let started = call(
        &mut world,
        agent,
        &[
            b"chat",
            b"start",
            b"--project",
            b"1",
            b"--template",
            b"3",
            b"Let chats wait two seconds; keep running work on its revision.",
            b"--key",
            b"configure",
        ],
    );
    assert_eq!(started.status, 0, "{started:?}");
    let task = fixtures::done_task(&started);
    let front = world.front();
    let assignment = front.assignments.last().expect("configuring run assigned");
    assert_eq!((assignment.task, assignment.charter), (task, 3));
    assert_ne!(assignment.run.policy.tools.0 & jig_domain_core::ToolFamilies::JIG.settings, 0);
    let inventory = front.host_tool(task, 1, engine::Tool::Settings { read: SettingsRead::Inventory, from: 0 });
    let HostAnswer::Settings(inventory) = inventory else { panic!("settings inventory: {inventory:?}") };
    let SettingsAnswer::Inventory(inventory) = *inventory else { panic!("settings inventory") };
    assert_eq!(inventory.newest, 1);
    let charter = front.host_tool(
        task,
        2,
        engine::Tool::Settings {
            read: SettingsRead::Setting { setting: s::Setting::Charter(1), revision: inventory.newest },
            from: 0,
        },
    );
    let HostAnswer::Settings(charter) = charter else { panic!("settings charter: {charter:?}") };
    let SettingsAnswer::Setting(charter) = *charter else { panic!("whole charter") };
    assert_eq!(charter.revision, 1);
    let Some(s::Value::Charter(mut charter)) = charter.value else { panic!("charter 1") };
    assert_eq!(charter.policy.waiting, Duration::from_secs(1));
    charter.policy.waiting = Duration::from_secs(2);
    let reason = if items.is_some() {
        b"Show drafts, goals, changes and the away switch.".as_slice()
    } else {
        REASON.as_bytes()
    };
    let items = items.unwrap_or_else(|| {
        Box::new([s::Item { setting: s::Setting::Charter(1), edit: s::Edit::Set(s::Value::Charter(charter)) }])
    });
    let kinds = temper_engine_domain_world::kinds::decoded(&items);
    let proposed = front.host_tool(
        task,
        3,
        engine::Tool::ProposeSettings { more: false, against: inventory.newest, items, reason: reason.into(), kinds },
    );
    let HostAnswer::Tool(engine::CallAnswer::Proposed { proposal, holder }) = proposed else {
        panic!("settings proposal: {proposed:?}")
    };
    assert_eq!(holder, tasks::ProposalHolder::Policy { project: 0, kind: tasks::ProposalKind::Settings });
    (world, agent, task, format!("t{task}/{proposal}"))
}

pub(super) fn accept(world: &mut ClientWorld, agent: ClientId, proposal: &str, key: &[u8]) {
    let expected = if let Some(task) = proposal.strip_prefix('t') {
        let (task, _) = task.split_once('/').expect("task proposal");
        format!("done: task {task}, decide\n")
    } else {
        "done: decide\n".to_owned()
    };
    assert_eq!(
        text(
            world,
            agent,
            &[b"proposal", b"decide", proposal.as_bytes(), b"accept", b"--running", b"stays", b"--key", key,]
        ),
        expected
    );
}

#[test]
fn a_configuring_chat_started_through_ctl_proposes_and_an_owner_accepts() {
    let (mut world, agent, task, proposal) = propose(false, None);
    let inbox = text(&mut world, agent, &[b"inbox", b"--kind", b"proposals"]);
    assert!(inbox.contains(&proposal), "{inbox}");
    let review = text(&mut world, agent, &[b"proposal", b"show", proposal.as_bytes()]);
    assert!(review.contains("against revision 1, newest 1"), "{review}");
    assert!(review.contains(REASON), "{review}");
    assert!(review.contains("charter 1 · set · now {"), "{review}");
    assert!(review.contains("\"waiting\":1000"), "{review}");
    assert!(review.contains(" · would be {"), "{review}");
    assert!(review.contains("\"waiting\":2000"), "{review}");
    accept(&mut world, agent, &proposal, b"accept-configuring");
    let front = world.front();
    let run = front.engine.task(task).expect("configuring chat");
    assert!(
        run.inbox.iter().any(|word| matches!(
            word.kind,
            tasks::MessageKind::SettingsDecision { outcome: tasks::SettingsOutcome::Accepted { revision: 2 }, .. }
        )),
        "the run hears accepted: revision 2 made"
    );
    front.configuring_reply(task, ACCEPTED);
    let shown = text(&mut world, agent, &[b"chat", b"show", task.to_string().as_bytes()]);
    assert!(shown.contains(std::str::from_utf8(ACCEPTED).expect("reply UTF-8")), "{shown}");
    let settings = text(&mut world, agent, &[b"settings", b"show"]);
    assert!(settings.starts_with("newest revision 2\n"), "{settings}");
}

#[test]
fn a_configuring_proposal_is_named_and_titled_in_the_inbox() {
    let (mut world, agent, _, _) = propose(false, None);
    let inbox = text(&mut world, agent, &[b"inbox", b"--kind", b"proposals"]);
    let title = REASON.lines().next().expect("proposal title");
    let row = inbox.lines().find(|row| row.contains(title)).expect("inbox carries the proposal title");
    let key = row.split_once(" · ").expect("proposal row").0;
    assert!(key.starts_with('t'), "{inbox}");
    assert_eq!(row, format!("{key} · proposal · {title} · projects: 1"));
    let review = text(&mut world, agent, &[b"proposal", b"show", key.as_bytes()]);
    assert!(review.starts_with(&format!("proposal {key} · ")), "{review}");
    assert!(review.contains(REASON), "{review}");
}

#[test]
fn a_members_configuring_proposal_waits_for_an_owner() {
    let (mut world, agent, _, proposal) = propose(true, None);
    let caller = world.front().caller;
    let roles: Vec<_> = world
        .front()
        .engine
        .store
        .rows
        .values()
        .filter_map(|row| {
            if let engine::Record::Core(jig_domain_core::Record::People(jig_domain_people::Stored::Roles {
                project,
                holdings,
            })) = row
            {
                Some((*project, holdings.clone()))
            } else {
                None
            }
        })
        .collect();
    assert!(
        roles
            .iter()
            .all(|(project, holdings)| *project != 0 || holdings.iter().all(|holding| holding.person != caller)),
        "{roles:?}"
    );
    let shown = call(&mut world, agent, &[b"proposal", b"show", proposal.as_bytes()]);
    assert_eq!(shown.status, 2, "{shown:?}");
    assert_eq!(shown.stderr.as_ref(), b"unknown\n");
    let refused = call(
        &mut world,
        agent,
        &[b"proposal", b"decide", proposal.as_bytes(), b"accept", b"--running", b"stays", b"--key", b"member-accept"],
    );
    assert_eq!(refused.status, 2, "{refused:?}");
    assert!(!refused.stderr.is_empty(), "{refused:?}");
}

#[test]
fn a_stale_configuring_proposal_is_withdrawn() {
    let (mut world, agent, task, proposal) = propose(false, None);
    let charter = text(&mut world, agent, &[b"settings", b"show", b"charter", b"1", b"--json"]);
    let value = charter.trim_end().split_once(",\"value\":").expect("shown charter value").1;
    let value = value.strip_suffix('}').expect("setting envelope");
    assert!(value.contains("\"waiting\":1000"), "{value}");
    let changed = value.replace("\"waiting\":1000", "\"waiting\":3000");
    let file = format!(r#"{{"items":[{{"setting":{{"kind":"charter","number":1}},"set":{changed}}}]}}"#);
    let number = fixtures::give_with_file(
        &mut world,
        agent,
        &[
            b"settings",
            b"propose",
            b"settings.json",
            b"Keep the configured charter",
            b"--against",
            b"1",
            b"--key",
            b"intervening",
        ],
        file.as_bytes(),
    );
    let printed = fixtures::answered(&mut world, agent, number);
    assert_eq!(printed.status, 0, "{printed:?}");
    let proposed = std::str::from_utf8(&printed.stdout).expect("proposal UTF-8");
    let intervening = proposed.strip_prefix("proposed: ").expect("proposal").split_once(", ").expect("recipient").0;
    accept(&mut world, agent, intervening, b"accept-intervening");
    let review = text(&mut world, agent, &[b"proposal", b"show", proposal.as_bytes()]);
    assert!(review.contains("withdrawn"), "{review}");
    assert!(review.contains("against revision 1"), "{review}");
    let run = world.front().engine.task(task).expect("configuring chat");
    assert!(
        run.inbox.iter().any(|word| matches!(
            word.kind,
            tasks::MessageKind::SettingsDecision { outcome: tasks::SettingsOutcome::Stale { newest: 2 }, .. }
        )),
        "the run hears withdrawn as stale at revision 2"
    );
}

#[test]
fn start_over_closes_and_opens_a_fresh_ask() {
    use jig_client::contract as c;
    let mut limits = fixtures::domain_limits();
    // The stock configuring task includes its settings authority and charter words.
    limits.client.domain.text = 4096;
    limits.client.domain.long_text = 4096;
    limits.client.domain.turn_text = 4096;
    let mut front = Joint::with_configuring(fixtures::SEED, limits, false);
    let mut root =
        temper_client_world::app::Root::Person(Box::new(temper_client_domain::unlinked(&limits, fixtures::SEED)));
    front.bind(ClientId(0), fixtures::PARTY);
    account_acts::event(&mut front, &mut root, Temper::start(Address::Ask { project: PROJECT }, Vec::new(), 0));
    let person = root.person();
    if jig_client::shapes::template(person.pages().me(person.client()).expect("me"), PROJECT, 5).is_none() {
        return;
    }
    account_acts::event(
        &mut front,
        &mut root,
        Event::Act { action: Action::Pages(Act::Starter { words: WORKFLOW.into() }) }.into(),
    );
    let assignment = front.assignments.last().expect("configuring run").clone();
    let task = assignment.task;
    let body = b"# How this project works\n\nOne checked change at a time.\n";
    let input = format!(
        "{{\"scope\":{{\"kind\":\"goal\",\"project\":{PROJECT},\"goal\":{task}}},\"description\":\"How this project works\",\"body\":\"# How this project works\\n\\nOne checked change at a time.\\n\"}}"
    );
    let (written, failed) =
        temper_engine_domain_world::host_tool::host_tool(&mut front.engine, &assignment, 1, b"note", input.as_bytes());
    assert!(!failed, "{}", String::from_utf8_lossy(&written));
    let reply = b"I have written how we work.";
    front.configuring_reply(task, reply);
    // Reopening reads the durable document and committed conversation together.
    account_acts::event(&mut front, &mut root, Temper::went(Address::Settings { project: PROJECT }));
    account_acts::event(&mut front, &mut root, Temper::went(Address::Chat { task }));
    let document = root.person().pages().chat().unwrap().draft.clone().expect("written document");
    assert_eq!(document.body.as_ref(), body);
    account_acts::event(&mut front, &mut root, Event::Act { action: Action::Pages(Act::StartOver) }.into());
    assert!(root.person().pages().confirmation().is_some());
    assert!(front.engine.task(task).is_some(), "asking has not closed it");
    account_acts::event(&mut front, &mut root, Event::Act { action: Action::Pages(Act::Confirm) }.into());
    assert_eq!(root.person().pages().address(), &Address::Ask { project: PROJECT });
    assert!(root.person().pages().chat().is_none());
    assert!(front.engine.task(task).is_none(), "the old chat has closed");
    account_acts::event(
        &mut front,
        &mut root,
        Event::Act { action: Action::Pages(Act::Starter { words: b"A fresh way of working".as_slice().into() }) }
            .into(),
    );
    let fresh = front.assignments.last().expect("fresh configuring run");
    assert_ne!(fresh.task, task);
    assert_eq!(fresh.charter, 11);
    let fresh = root.person().pages().chat().expect("fresh chat");
    assert!(fresh.draft.is_none());
    assert!(
        !fresh.thread.lines().iter().any(|line| matches!(line, jig_client::thread::Line::Turn { .. })),
        "a fresh conversation has no old turns"
    );
    account_acts::event(
        &mut front,
        &mut root,
        Temper::went(Address::Chats { filter: temper_client_domain::ChatsFilter::Closed }),
    );
    let closed = root.person().pages().chats().unwrap().page.as_ref().expect("closed chats");
    let c::EndedGroup::Rows { rows, .. } = &closed.ended else { panic!("closed rows") };
    assert!(rows.iter().any(|row| row.chip.task == task), "the old chat stays in Closed");
    account_acts::event(&mut front, &mut root, Temper::went(Address::Chat { task }));
    let person = root.person();
    let old = person.pages().chat().expect("closed chat page");
    assert_eq!(old.draft.as_ref(), Some(&document));
    assert_eq!(old.task(person.client()).unwrap().spec.specification.words.bytes(), WORKFLOW);
    assert!(
        old.thread.lines().iter().any(|line| {
            matches!(line, jig_client::thread::Line::Turn { turn, .. }
            if turn.messages.iter().any(|message| message.blocks.iter().any(|block|
                matches!(block, c::Block::Text(text) if text.bytes() == reply))))
        }),
        "the old conversation stays with its document"
    );
}

fn see(world: &mut ClientWorld, person: ClientId, find: skein_fake_person::Find) {
    world.person(person, vec![skein_fake_person::Step::See { find, within: fixtures::WITHIN }]);
    let outcome = world.run(jig_client_kit::harness::Until::Done);
    assert_eq!(outcome.verdict, skein_world::domain::Verdict::Passed, "{outcome:?}");
}
fn see_proposal(world: &mut ClientWorld, person: ClientId) {
    see(world, person, skein_fake_person::Find::named(skein_ui::Role::Region, b"Proposal"));
}
fn rail(world: &ClientWorld, person: ClientId, project: u32) -> u32 {
    let root = world.root(person).person();
    root.pages().needs().count(root.client(), root.forge().approvals(), project).count
}
fn project_inbox(project: u32) -> Address {
    Address::Inbox {
        filter: temper_client_domain::address::Filter {
            project: Some(project),
            ..temper_client_domain::address::Filter::ALL
        },
    }
}
#[test]
fn a_configuring_proposal_of_a_projects_ways_reaches_its_needs_and_its_rail() {
    use jig_client::contract as c;
    let value = temper_engine_domain_forge::settings::Value::Ways(temper_engine_domain_forge::settings::ProjectWays {
        drafts: true,
        goals: true,
        changes: false,
        away: true,
        way: 0,
    });
    let item = temper_engine_domain_world::kinds::item(7, PROJECT, 1, &value);
    let (mut world, agent, task, proposal) = propose(false, Some(Box::new([item])));
    let inbox = text(&mut world, agent, &[b"inbox", b"--project", b"1", b"--json"]);
    assert!(inbox.contains("\"projects\":[1]"), "{inbox}");
    let review = text(&mut world, agent, &[b"proposal", b"show", proposal.as_bytes(), b"--json"]);
    assert!(review.contains("\"project\":0,\"projects\":[1]"), "{review}");
    let person = world.client(fixtures::PARTY, Address::Home { project: PROJECT });
    see(&mut world, person, skein_fake_person::Find::named(skein_ui::Role::Region, b"Needs you"));
    // The Home inbox reads the same row before opening the flow.
    for _ in 0..32 {
        if world.root(person).person().pages().inbox().is_some_and(|inbox| !inbox.cards.is_empty()) {
            break;
        }
        world.run(jig_client_kit::harness::Until::Steps(2));
    }
    assert_eq!(world.root(person).person().pages().inbox().unwrap().cards.len(), 1);
    assert_eq!(rail(&world, person, PROJECT), 1);
    world.go(person, Address::Chat { task });
    see(&mut world, person, skein_fake_person::Find::text(b"Goals: shown"));
    world.go(person, project_inbox(PROJECT));
    see_proposal(&mut world, person);
    let object = world.root(person).person().pages().inbox().unwrap().cards.get(0).unwrap().object;
    world.event(
        person,
        Event::Act { action: Action::Pages(Act::Decide { object, revision: None, act: c::ActKind::Accept }) }.into(),
    );
    world.event(person, Event::Act { action: Action::Pages(Act::Confirm) }.into());
    for _ in 0..64 {
        if rail(&world, person, PROJECT) == 0 {
            break;
        }
        world.run(jig_client_kit::harness::Until::Steps(2));
    }
    assert_eq!(rail(&world, person, PROJECT), 0);
    assert!(text(&mut world, agent, &[b"settings", b"show"]).starts_with("newest revision 2\n"));
    world.go(person, Address::Chat { task });
    see(
        &mut world,
        person,
        skein_fake_person::Find::named(skein_ui::Role::Paragraph, "✓ Accepted · new work follows it".as_bytes()),
    );
}
#[test]
fn a_deployments_proposal_shows_in_each_project_its_owner_owns() {
    use jig_domain_people as people;
    let mut limits = fixtures::domain_limits();
    limits.client.domain.text = 4096;
    limits.client.domain.long_text = 4096;
    let front = Joint::with_configuring_projects(fixtures::SEED, limits, false, 3);
    let world = World::new(jig_client_kit::harness::Settings { seed: fixtures::SEED, ..Default::default() }, front);
    let (mut world, agent, _, _) = propose_in(world, None);
    let driver = &mut world.front().engine;
    let reply = skein_lib::ReplyTo::new(skein_lib::Token::new(2003));
    driver.send(engine::Event::Party(engine::PartyInput::SignedIn {
        held: None,
        reply_to: skein_lib::ReplyTo::new(skein_lib::Token::new(2003)),
        identity: people::Identity {
            key: temper_engine_domain_forge::user_key(1, 8_u64),
            login: b"other".as_slice().into(),
            name: b"Other owner".as_slice().into(),
        },
    }));
    driver.settle();
    let other = driver
        .delivered
        .iter()
        .find_map(|delivery| match delivery {
            engine::Delivery::WebReply { to, reply: people::Reply::SignedIn { person, .. } } if *to == reply => {
                Some(*person)
            }
            _ => None,
        })
        .expect("other owner signed in");
    let session = driver
        .store
        .rows
        .values()
        .find_map(|row| match row {
            engine::Record::Core(jig_domain_core::Record::People(people::Stored::SignIn {
                number, person, ..
            })) if *person == fixtures::PARTY => Some(*number),
            _ => None,
        })
        .expect("owner's sign-in number");
    driver.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: skein_lib::ReplyTo::new(skein_lib::Token::new(2004)),
        sign_in: session,
        key: [203; 16],
        ask: people::Ask::SetRoles {
            project: 3,
            holdings: Box::new([
                people::Holding { person: other, role: people::Role::Owner },
                people::Holding { person: fixtures::PARTY, role: people::Role::Member },
            ]),
        },
    }));
    driver.settle();
    let person = world.client(fixtures::PARTY, project_inbox(PROJECT));
    for project in [PROJECT, 2] {
        world.go(person, project_inbox(project));
        see_proposal(&mut world, person);
        assert_eq!(rail(&world, person, project), 1);
    }
    world.go(person, project_inbox(3));
    world.run(jig_client_kit::harness::Until::Steps(64));
    let root = world.root(person).person();
    assert!(root.pages().inbox().unwrap().delivered(root.client()));
    assert!(root.pages().inbox().unwrap().cards.is_empty());
    assert_eq!(rail(&world, person, 3), 0);
    let inbox = text(&mut world, agent, &[b"inbox", b"--json"]);
    assert!(inbox.contains("\"projects\":[1,2]"), "{inbox}");
}

fn reject_under_needs(world: &mut ClientWorld, person: ClientId, reason: &[u8]) {
    use skein_fake_person::{Find, Step, Target};
    use skein_ui::Role;
    world.person(
        person,
        vec![
            Step::See { find: Find::named(Role::Region, b"Needs you"), within: fixtures::WITHIN },
            Step::See { find: Find::named(Role::Disclosure, b"Reject with reason"), within: fixtures::WITHIN },
            Step::Type { find: Find::named(Role::TextArea, b"Reason"), words: reason.into() },
            Step::Press { find: Find::exactly_named(Role::Button, b"Reject") },
        ],
    );
    let outcome = world.run(jig_client_kit::harness::Until::Done);
    assert_eq!(outcome.verdict, skein_world::domain::Verdict::Passed, "{outcome:?}");
    assert!(world.root(person).person().pages().confirmation().is_some());
    world.person(
        person,
        vec![Step::Press {
            find: Find::exactly_named(Role::Button, b"Reject").within(Box::new([Target::Role {
                role: Role::Dialog,
                name: b"Reject this proposal?".as_slice().into(),
            }])),
        }],
    );
    let outcome = world.run(jig_client_kit::harness::Until::Done);
    assert_eq!(outcome.verdict, skein_world::domain::Verdict::Passed, "{outcome:?}");
}

#[test]
fn an_owner_rejects_a_chats_pending_proposal_under_needs_you() {
    let (mut world, agent, task, proposal) = propose(false, None);
    let person = world.client(fixtures::PARTY, Address::Home { project: PROJECT });
    let reason = b"Keep chats waiting one second.";
    reject_under_needs(&mut world, person, reason);
    let review = text(&mut world, agent, &[b"proposal", b"show", proposal.as_bytes()]);
    assert!(review.contains("rejected"), "{review}");
    let run = world.front().engine.task(task).expect("configuring chat").clone();
    assert!(run.proposal.is_none(), "the rejected proposal no longer waits");
    assert!(
        run.inbox.iter().any(|word| matches!(
            &word.kind, tasks::MessageKind::SettingsDecision {
                outcome: tasks::SettingsOutcome::Rejected, ..
            } if word.words.as_ref() == reason
        )),
        "the chat hears the owner's rejection reason: {:?}",
        run.inbox
    );
    assert!(text(&mut world, agent, &[b"settings", b"show"]).starts_with("newest revision 1\n"));
}

#[test]
fn change_something_asks_in_the_composer_until_a_message_goes_out() {
    use jig_client_kit::harness::Until;
    use skein_fake_person::{Find, Step};
    use skein_ui::Role;
    use skein_world::domain::Verdict;
    let (mut world, _, task, _) = propose(false, None);
    let person = world.client(fixtures::PARTY, Address::Settings { project: PROJECT });
    world.go(person, Address::Chat { task });
    world.person(
        person,
        vec![
            Step::See { find: Find::named(Role::Button, b"Change something"), within: fixtures::WITHIN },
            Step::Press { find: Find::named(Role::Button, b"Change something") },
        ],
    );
    assert_eq!(world.run(Until::Done).verdict, Verdict::Passed);
    let chat = world.root(person).person().pages().chat().expect("chat page");
    assert!(chat.change_asked.is_some(), "the card asks what should change");
    world.person(
        person,
        vec![
            Step::Type {
                find: Find::named(Role::TextArea, b"Message"),
                words: b"Wait three seconds instead".as_slice().into(),
            },
            Step::Press { find: Find::named(Role::Button, b"Send") },
        ],
    );
    assert_eq!(world.run(Until::Done).verdict, Verdict::Passed);
    assert_eq!(world.root(person).person().pages().chat().expect("chat page").change_asked, None);
}

#[test]
fn a_decision_from_needs_you_reaches_an_open_chats_card() {
    use jig_client_kit::harness::Until;
    use skein_fake_person::{Find, Step};
    use skein_ui::Role;
    use skein_world::domain::Verdict;
    let (mut world, _, task, _) = propose(false, None);
    let chat = world.client(fixtures::PARTY, Address::Chat { task });
    world
        .person(chat, vec![Step::See { find: Find::exactly_named(Role::Button, b"Accept"), within: fixtures::WITHIN }]);
    assert_eq!(world.run(Until::Done).verdict, Verdict::Passed);
    let owner = world.client(fixtures::PARTY, Address::Home { project: PROJECT });
    reject_under_needs(&mut world, owner, b"Keep chats waiting one second.");
    world.person(
        chat,
        vec![
            Step::See {
                find: Find::exactly_named(Role::Paragraph, "Rejected · nothing changes".as_bytes()),
                within: fixtures::WITHIN,
            },
            Step::Gone {
                find: Find::exactly_named(Role::Paragraph, b"Waiting for an owner"),
                within: fixtures::WITHIN,
            },
            Step::Gone { find: Find::exactly_named(Role::Button, b"Accept"), within: fixtures::WITHIN },
        ],
    );
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");
    assert_eq!(world.root(chat).person().pages().address(), &Address::Chat { task });
}

#[test]
fn a_second_proposal_stands_below_the_first_and_a_reply_below_it() {
    use jig_client::{contract as c, thread::Line};
    use skein_ui::Role;
    let (mut world, agent, task, first) = propose(false, None);
    world.front().configuring_reply(task, b"First reply");
    let task_words = task.to_string();
    text(&mut world, agent, &[b"task", b"message", task_words.as_bytes(), b"Tell me more", b"--key", b"more"]);
    world.front().configuring_reply(task, b"Second reply");
    let person = world.client(fixtures::PARTY, Address::Chat { task });
    see(&mut world, person, skein_fake_person::Find::exactly_named(Role::Button, b"Accept"));
    read_chat_start(&mut world, person);
    accept(&mut world, agent, &first, b"accept-first-card");
    let charter = world.front().host_tool(
        task,
        1,
        engine::Tool::Settings {
            read: SettingsRead::Setting { setting: s::Setting::Charter(1), revision: 2 },
            from: 0,
        },
    );
    let HostAnswer::Settings(charter) = charter else { panic!("charter read") };
    let SettingsAnswer::Setting(charter) = *charter else { panic!("charter") };
    let Some(s::Value::Charter(mut charter)) = charter.value else { panic!("charter value") };
    charter.policy.waiting = Duration::from_secs(3);
    let items: Box<[s::Item]> =
        Box::new([s::Item { setting: s::Setting::Charter(1), edit: s::Edit::Set(s::Value::Charter(charter)) }]);
    let kinds = temper_engine_domain_world::kinds::decoded(&items);
    let proposed = world.front().host_tool(
        task,
        2,
        engine::Tool::ProposeSettings { more: false, against: 2, items, reason: REASON.as_bytes().into(), kinds },
    );
    let HostAnswer::Tool(engine::CallAnswer::Proposed { proposal: second, .. }) = proposed else {
        panic!("second proposal: {proposed:?}")
    };
    // Commit the tool's completion before the words of the next run, as a model does.
    world.front().configuring_reply(task, b"");
    text(
        &mut world,
        agent,
        &[b"task", b"message", task_words.as_bytes(), b"Explain the revision", b"--key", b"explain"],
    );
    world.front().configuring_reply(task, b"Third reply");
    see(&mut world, person, skein_fake_person::Find::text(b"Third reply"));
    // Read the older pages as a person does; the first page is deliberately thin.
    read_chat_start(&mut world, person);
    let root = world.root(person).person();
    let chat = root.pages().chat().unwrap();
    let lines = chat.thread.lines().into_boxed();
    let made = lines.iter().position(|line| matches!(line, Line::Activity { line }
        if matches!(line.source.line, c::Activity::Decided { proposal: Some(c::DecidedProposal { number, made: true }), .. } if number == second))).expect("engine making line");
    assert!(
        lines.iter().skip(made + 1).any(|line| matches!(line, Line::Turn { .. })),
        "the engine makes the proposal before its completion commits"
    );
    let view = chat_view(&world, person);
    let mut order = Vec::new();
    for node in view.tree().nodes() {
        if let Some(key) = node.key
            && key.kind == 1
            && key.a == task
        {
            order.push(if key.b == second { "Second card" } else { "First card" });
        }
        for run in &node.text {
            for reply in ["First reply", "Second reply", "Third reply"] {
                if run.text.as_ref() == reply.as_bytes() {
                    order.push(reply);
                }
            }
        }
    }
    assert_eq!(order, ["First reply", "First card", "Second reply", "Second card", "Third reply"]);
}

#[test]
fn change_something_replaces_the_card_below() {
    use jig_client::contract as c;
    use jig_client_kit::harness::Until;
    use skein_fake_person::{Find, Step};
    use skein_ui::Role;
    use skein_world::domain::Verdict;
    let (mut world, agent, task, first) = propose(false, None);
    world.front().configuring_reply(task, b"The proposal waits for your Accept.");
    let person = world.client(fixtures::PARTY, Address::Chat { task });
    see(&mut world, person, Find::exactly_named(Role::Button, b"Change something"));
    let old = world.root(person).person().pages().chat().expect("chat").cards.get(0).expect("first card").object;
    world.person(
        person,
        vec![
            Step::Press { find: Find::exactly_named(Role::Button, b"Change something") },
            Step::Type {
                find: Find::exactly_named(Role::TextArea, b"Message"),
                words: b"Wait three seconds instead".as_slice().into(),
            },
            Step::Press { find: Find::exactly_named(Role::Button, b"Send") },
            Step::See { find: Find::text(b"Wait three seconds instead"), within: fixtures::WITHIN },
        ],
    );
    assert_eq!(world.run(Until::Done).verdict, Verdict::Passed);
    let charter = world.front().host_tool(
        task,
        1,
        engine::Tool::Settings {
            read: SettingsRead::Setting { setting: s::Setting::Charter(1), revision: 1 },
            from: 0,
        },
    );
    let HostAnswer::Settings(charter) = charter else { panic!("charter read") };
    let SettingsAnswer::Setting(charter) = *charter else { panic!("charter") };
    let Some(s::Value::Charter(mut charter)) = charter.value else { panic!("charter value") };
    charter.policy.waiting = Duration::from_secs(3);
    let items: Box<[s::Item]> =
        Box::new([s::Item { setting: s::Setting::Charter(1), edit: s::Edit::Set(s::Value::Charter(charter)) }]);
    let kinds = temper_engine_domain_world::kinds::decoded(&items);
    let proposed = world.front().host_tool(
        task,
        2,
        engine::Tool::ProposeSettings {
            more: false,
            against: 1,
            items,
            reason: b"Wait three seconds instead".as_slice().into(),
            kinds,
        },
    );
    let HostAnswer::Tool(engine::CallAnswer::Proposed { proposal: fresh, .. }) = proposed else {
        panic!("fresh proposal: {proposed:?}")
    };
    world.front().configuring_reply(task, b"");
    let replaced = Find::exactly_named(Role::Paragraph, b"Replaced by the proposal below");
    see(&mut world, person, replaced.clone());
    read_chat_start(&mut world, person);
    let view = chat_view(&world, person);
    let nodes = view.tree().nodes();
    assert_eq!(
        nodes.iter().filter(|node| node.role == Role::Button && node.name.as_deref() == Some(b"Accept")).count(),
        1
    );
    let cards: Vec<_> = nodes
        .iter()
        .filter_map(|node| node.key)
        .filter(|key| key.kind == 1 && key.a == task)
        .map(|key| key.b)
        .collect();
    let first: u64 = first.split_once('/').expect("proposal key").1.parse().expect("first number");
    assert_ne!(fresh, first);
    assert_eq!(cards, [first, fresh], "the fresh card stands below the replaced one");
    world.event(
        person,
        Event::Act { action: Action::Pages(Act::Decide { object: old, revision: None, act: c::ActKind::Accept }) }
            .into(),
    );
    world.run(Until::Steps(2));
    assert!(
        world.root(person).person().pages().confirmation().is_none(),
        "the ended card drops a stale Accept before jig"
    );
    assert!(text(&mut world, agent, &[b"settings", b"show"]).starts_with("newest revision 1\n"));
    see(&mut world, person, replaced);
}

fn read_chat_start(world: &mut ClientWorld, person: ClientId) {
    for _ in 0..32 {
        if world
            .root(person)
            .person()
            .pages()
            .chat()
            .expect("chat page")
            .thread
            .lines
            .edges
            .reaches(jig_client::thread::window::End::After)
        {
            break;
        }
        world.event(person, Event::Act { action: Action::Pages(Act::Older) }.into());
        world.run(jig_client_kit::harness::Until::Steps(2));
    }
}

fn chat_view(world: &ClientWorld, person: ClientId) -> temper_client_view::View {
    let root = world.root(person).person();
    let limits = fixtures::domain_limits();
    let words = temper_client_view::words::Words::english(
        fixtures::FORGE,
        limits.client.domain.text,
        temper_client_view::words::Shell::Browser,
    )
    .expect("browser words fit the world bounds");
    let bounds = fixtures::view_bounds(&limits, &words);
    let mut view = temper_client_view::View::new(&bounds, &limits, words, fixtures::FORGE);
    temper_client_view::build(
        &mut view,
        root,
        fixtures::NOW,
        &mut skein_lib::Queue::with_capacity(bounds.tree.patches),
    );
    view
}

#[test]
fn a_decided_proposal_stands_after_a_reload() {
    use skein_fake_person::Find;
    use skein_ui::Role;
    let (mut world, agent, task, _) = propose(false, None);
    world.front().configuring_reply(task, b"First reply");
    text(
        &mut world,
        agent,
        &[b"task", b"message", task.to_string().as_bytes(), b"Tell me more", b"--key", b"reload-more"],
    );
    world.front().configuring_reply(task, b"Second reply");
    let owner = world.client(fixtures::PARTY, Address::Home { project: PROJECT });
    reject_under_needs(&mut world, owner, b"Keep the settings");
    let reload = world.client(fixtures::PARTY, Address::Chat { task });
    see(&mut world, reload, Find::named(Role::Log, b"Conversation"));
    world.run(jig_client_kit::harness::Until::Steps(16));
    read_chat_start(&mut world, reload);
    see(&mut world, reload, Find::named(Role::Paragraph, "Rejected · nothing changes".as_bytes()));
    let view = chat_view(&world, reload);
    let mut order = Vec::new();
    for node in view.tree().nodes() {
        for run in &node.text {
            let said = run.text.as_ref();
            assert_ne!(said, b"Decision made");
            if said == b"First reply" {
                order.push("first");
            }
            if said.starts_with(b"How temper will run ") {
                order.push("card");
            }
            if said == b"Second reply" {
                order.push("second");
            }
        }
    }
    assert_eq!(order, ["first", "card", "second"]);
}
