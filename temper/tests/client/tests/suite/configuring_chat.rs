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
fn the_page_starts_the_configuring_chat() {
    for acts in
        [vec![Act::ChatWords { words: WORKFLOW.into() }, Act::StartChat], vec![Act::Starter { words: WORKFLOW.into() }]]
    {
        let mut front = Joint::with_configuring(fixtures::SEED, fixtures::domain_limits(), false);
        let mut root = Temper::new(fixtures::SEED).0;
        front.bind(ClientId(0), fixtures::PARTY);
        // The domain-only pump joins the real client and engine without drawing Configure.
        account_acts::event(
            &mut front,
            &mut root,
            Temper::start(Address::Configure { project: PROJECT }, Vec::new(), 0),
        );
        for act in acts {
            account_acts::event(&mut front, &mut root, Event::Act { action: Action::Pages(act) }.into());
        }
        let assignment = front.assignments.last().expect("configuring run assigned");
        assert_eq!(assignment.charter, 3);
        assert!(assignment.workspace.repositories.is_empty(), "the configuring chat names no repository");
        let task = front.engine.task(assignment.task).expect("configuring chat");
        assert_eq!(task.spec.words.as_ref(), WORKFLOW);
        let task = assignment.task;
        let person = root.person();
        assert_eq!(person.pages().address(), &Address::Chat { task });
        assert_eq!(person.pages().chat().expect("chat page").task, task);
        account_acts::event(&mut front, &mut root, Temper::went(Address::Settings { project: PROJECT }));
        account_acts::settle(&mut front, &mut root);
        let person = root.person();
        assert_eq!(
            person.pages().chats().expect("Settings holds chats").open_configuring(
                person.client(),
                person.pages().me(person.client()),
                PROJECT,
            ),
            Some(task),
            "Settings finds the configuring task in the real engine's delivered rows",
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
    let mut world = fixtures::configuring_world(fixtures::SEED, member);
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
        engine::Tool::ProposeSettings { against: inventory.newest, items, reason: reason.into(), kinds },
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
    assert_eq!(row, format!("{key} · proposal · {title}"));
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
