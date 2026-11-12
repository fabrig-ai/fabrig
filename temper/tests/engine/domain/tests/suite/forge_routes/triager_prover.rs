//! A red with no goal subscriber meets its triager and prover (domain/areas.md, 4–5;
//! domain/after-landing.md, 7; draft/standing-roles.md, 11).
use super::*;
use forge_top::settings::{self as settings, AreaOwner};
use temper_engine_domain_forge_change::State;
use temper_engine_domain_world::{
    kinds,
    tier_runs::{self, LANDING_TICKS, RUN_TICKS, change, merge, task, tier, until},
};

const CHANNEL: Token = Token::new(7);

fn world() -> World {
    let mut world = World::configured_limits(
        true,
        true,
        false,
        1000,
        Gate::default(),
        None,
        StoryLimits {
            tier: Some(tier_runs::Tier::default()),
            kinds: true,
            standing_roles: true,
            tasks: Some(12),
            spec_bytes: Some(4096),
            notes: Some(authority::Scopes::GOAL),
            ..StoryLimits::default()
        },
    )
    .with_goal_closer();
    world.env.limits.tasks.message_bytes = 1024;
    world.env.limits.tasks.inbox_bytes = 8192;
    world.env.limits.call_records = 8;
    let mut codec = temper_engine_domain_world::store::widest();
    codec.tasks = world.env.limits.tasks;
    codec.authority = world.env.limits.authority;
    codec.call_records = world.env.limits.call_records;
    let mut store = temper_engine_domain_world::store::Store::with_limits(&codec);
    store.seed(world.store.rows.values().cloned().collect(), Box::new([]));
    world.store = store;
    let client = world.env.limits.forge.client;
    world = world.with_forge_client(client);
    world.adopt();
    let mut hello = protocol::hello_slots(CHANNEL, 2, Box::new([]));
    let engine::Event::Worker(engine::WorkerInput::Link { link: engine::Link::Hello { hello: worker, .. }, .. }) =
        &mut hello
    else {
        panic!("worker hello")
    };
    worker.bounds.message_bytes = world.env.limits.tasks.message_bytes;
    world.send(hello);
    let items: Box<[_]> = Box::new([
        kinds::item(
            14,
            1,
            1,
            &settings::Value::ProjectAreas(settings::ProjectAreas {
                areas: Box::new([settings::Area {
                    name: b"tests".as_slice().into(),
                    words: b"the C test suite under test/, its flakes and the tiers' reds".as_slice().into(),
                    owner: AreaOwner::Charter(15),
                }]),
            }),
        ),
        kinds::item(
            15,
            1,
            1,
            &settings::Value::ProjectTriager(settings::ProjectTriager { owner: AreaOwner::Charter(14) }),
        ),
    ]);
    let against = kinds::revision(&world.store.rows);
    world.send(engine::Event::Party(engine::PartyInput::ChangeSettings {
        reply_to: ReplyTo::new(Token::new(300)),
        sign_in: world.signed_in.expect("owner"),
        key: [30; 16],
        digest: [30; 32],
        kinds: kinds::decoded(&items),
        change: Box::new(jig_domain_core::settings::Change {
            against,
            items,
            running: jig_domain_core::settings::Running::Stays,
        }),
    }));
    world.until(Until::Ready);
    assert_eq!(kinds::revision(&world.store.rows), against + 1);
    world
}

fn call(world: &mut World, run: &engine::Assignment, name: u64, tool: &[u8], input: &[u8]) -> engine::CallAnswer {
    world.until(Until::Ready);
    let input = smith_domain_run::HostInput::attested(input.into()).expect("bounded scripted input");
    let temper_engine_smith::Decoded::Call(call) = temper_engine_smith::call_configured(
        smith_domain_run::CallName { activation: run.attempt, completion: 1, position: 1 },
        tool,
        &input,
        &walking::models(),
        &[],
    )
    .expect("scripted call decodes") else {
        panic!("engine call")
    };
    let answer = world.scripted_call(run, name, call.tool);
    let rendered = temper_engine_smith::answer(&answer);
    assert!(
        !rendered.error(),
        "call {name} {}: {}",
        String::from_utf8_lossy(tool),
        String::from_utf8_lossy(rendered.text())
    );
    answer
}

fn park(world: &mut World, run: &engine::Assignment, name: u64, body: &str) {
    let input =
        format!(r#"{{"kind":"search","scopes":[{{"kind":"goal","project":1,"goal":{}}}],"query":"now"}}"#, run.task);
    let answer = call(world, run, name * 10, b"recall", input.as_bytes());
    let engine::CallAnswer::NoteRecalled { entries, more: false } = answer else { panic!("Now recalled") };
    let revision = match entries.as_ref() {
        [] => String::new(),
        [entry] => format!(r#", "name":{}, "recalled":{}"#, entry.name, entry.revision),
        _ => panic!("one Now per role"),
    };
    let input = format!(
        r#"{{"scope":{{"kind":"goal","project":1,"goal":{}}},"description":"now","body":"{body}"{revision}}}"#,
        run.task
    );
    call(world, run, name * 10 + 1, b"note", input.as_bytes());
    world.scripted_read_inbox(run);
    let row = task(world, run.task);
    park_after_turn(world, run, row.inbox.last().map(|word| word.number), row.turn);
    until(world, LANDING_TICKS, "role parked", |world| {
        matches!(task(world, run.task).phase, tasks::Phase::Active(tasks::Active::Idle))
    });
}

fn next_run(world: &mut World, previous: &engine::Assignment) -> engine::Assignment {
    until(world, LANDING_TICKS, "standing role wakes", |world| {
        world.assigned.iter().any(|run| run.task == previous.task && run.attempt != previous.attempt)
    });
    world.assigned.iter().rev().find(|run| run.task == previous.task).expect("role assigned").clone()
}

fn assert_news(world: &World, holder: u64, area: Option<&[u8]>, number: u32, hander: Option<u64>) {
    assert_eq!(engine::finding_holder(&world.root, 1, area), forge_top::Holder::Task(holder));
    let recipients: Vec<_> = world
        .store
        .rows
        .values()
        .filter_map(|record| {
            let Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row) | tasks::Stored::Ended(row))) =
                record
            else {
                return None;
            };
            (!red_findings::news(world, row.number).is_empty()).then_some(row.number)
        })
        .collect();
    assert_eq!(recipients, [holder], "only this step's holder has unread finding news");
    let words = red_findings::news(world, holder);
    let [news] = words.as_slice() else { panic!("one finding news: {words:?}") };
    assert!(matches!(news.kind, tasks::MessageKind::News { class: tasks::NewsClass::Wakes, .. }));
    let text = std::str::from_utf8(&news.words).expect("finding news");
    let mut lines = text.lines();
    assert_eq!(
        lines.next(),
        Some(format!("Finding 1/{number} is yours: Red after landing on main: tier slow").as_str())
    );
    assert_eq!(lines.next(), Some(if area.is_some() { "red, high, area tests" } else { "red, high, no area" }));
    if let Some(hander) = hander {
        assert_eq!(lines.next(), Some(format!("Handed on by task {hander}: a test under test/ fails").as_str()));
    }
}

#[test]
#[expect(clippy::too_many_lines, reason = "one meeting follows the red, both standing roles, its repair and drop")]
fn a_red_meets_the_triager_and_prover_then_clears() {
    let mut world = world();
    let ask = world.standing(1, 14, 100, b"Route the project's findings");
    let triager = assigned_chat(&mut world, 92, ask);
    park(&mut world, &triager, 301, "No findings yet");
    let ask = world.standing(1, 15, 400, b"Own the C tests under test/");
    let prover = assigned_chat(&mut world, 93, ask);
    park(&mut world, &prover, 302, "Waiting for test findings");
    tiers::baseline(&mut world);

    let people::Ask::Create { project, mut batch } = tier_runs::goal(&world) else { panic!("goal fixture") };
    batch[0].flags.ended_by = people::EndedBy::Closer(people::Closer { connector: 0, code: 6 });
    batch[0].symbols = Box::new([]);
    let goal = assigned_chat(&mut world, 94, people::Ask::Create { project, batch });
    let answer = change_tool::change_landing(
        &mut world,
        &goal,
        303,
        None,
        b"Change the C test suite",
        Some(engine::ChangeLanding::Branch(b"main".as_slice().into())),
    );
    let red = tier_runs::produce_delegated(&mut world, answer, b"main", b"failures", b"suite.alpha\n");
    let commit = merge(&world, red);
    assert!(matches!(change(&world, red).change.state, State::Landed { .. }));
    // A closer-ended goal without a branch has no main-landings subscription.
    // Keep it and its uncleared change live, but free the worker for the roles.
    assert!(!world.store.rows.values().any(|record| matches!(record, Record::Forge { row }
        if matches!(row.as_ref(), forge_top::Stored::Subscription(subscription)
            if subscription.task == goal.task && matches!(&subscription.topic,
                forge_top::Topic::Landings { branch, .. } if branch.as_ref() == b"main")))));
    world.scripted_read_inbox(&goal);
    let row = task(&world, goal.task);
    let (last_message, turns) = (row.inbox.last().map(|message| message.number), row.turn);
    park_after_turn(&mut world, &goal, last_message, turns);
    until(&mut world, LANDING_TICKS, "original goal parked", |world| {
        matches!(task(world, goal.task).phase, tasks::Phase::Active(tasks::Active::Idle))
    });
    let assigned = world.assigned.len();
    until(&mut world, RUN_TICKS * 2, "landing judged red", |world| tier(world).record.sequence == 2);
    assert_eq!(world.assigned.len(), assigned, "no agent runs between verdict and finding news");
    let number = {
        let findings = red_findings::findings(&world);
        let [finding] = findings.as_slice() else { panic!("one red finding: {findings:?}") };
        assert_eq!(finding.state, forge_top::FindingState::New);
        assert_eq!(finding.area, None);
        finding.number
    };
    assert_news(&world, triager.task, None, number, None);
    assert_eq!(red_findings::news(&world, prover.task).len(), 0);
    until(&mut world, LANDING_TICKS, "change learns the red verdict", |world| {
        matches!(change(world, red).change.state, State::Red { .. })
    });
    let triager_run = next_run(&mut world, &triager);
    assert!(triager_run.messages.iter().any(|news| news.words.starts_with(b"Finding 1/")));
    let input = format!(r#"{{"act":"hand","number":{number},"area":"tests","why":"a test under test/ fails"}}"#);
    call(&mut world, &triager_run, 304, b"finding", input.as_bytes());
    assert_news(&world, prover.task, Some(b"tests"), number, Some(triager.task));
    park(&mut world, &triager_run, 305, "Handed the red to tests");

    let prover_run = next_run(&mut world, &prover);
    assert!(prover_run.messages.iter().any(|news| news.words.starts_with(b"Finding 1/")));
    let answer = call(&mut world, &prover_run, 3060, b"finding", br#"{"act":"list","state":"new","mine":true}"#);
    let engine::CallAnswer::Finding(answer) = answer else { panic!("prover's findings") };
    let engine::FindingToolAnswer::List(list) = *answer else { panic!("mine list") };
    assert_eq!(list.rows.iter().map(|row| row.number).collect::<Vec<_>>(), [number]);
    let digits = String::from_utf8(forge_top::commit_text(&commit).into_vec()).expect("merge digits");
    let input = format!(r#"{{"words":"Repair the C test under test/","landing":"main","repairs":"{digits}"}}"#);
    let answer = call(&mut world, &prover_run, 306, b"change", input.as_bytes());
    let repair = tier_runs::produce_delegated(&mut world, answer, b"main", b"failures", b"");
    assert_eq!(change(&world, repair).repairs, Some(commit));
    assert!(
        matches!(change(&world, red).change.state, State::Red { .. }),
        "the red waits for the repair's tier verdict"
    );
    park(&mut world, &prover_run, 307, "Repair landed, awaiting its tier verdict");
    until(&mut world, RUN_TICKS * 2, "repair judged", |world| tier(world).record.sequence == 3);
    assert_eq!(tier(&world).record.failing, temper_engine_domain_forge_tiers::Failures::Tests(Box::new([])));
    tiers::cleared(&mut world, red);
    tiers::cleared(&mut world, repair);
    let repaired = forge_top::commit_text(&merge(&world, repair));
    let short = std::str::from_utf8(repaired.get(..12).expect("short commit")).expect("hex digits");
    assert!(
        tiers::result(&world, red)
            .contains(&format!("Cleared by slow run 3, which passed the failures blamed on it; repaired by {short}."))
    );
    assert_eq!(translate::commit(tier_runs::runs(&world).last().expect("repair run").commit), merge(&world, repair));

    let prover_run = next_run(&mut world, &prover_run);
    let reason = format!("repaired by change {repair}");
    let input = format!(r#"{{"act":"drop","number":{number},"reason":"{reason}"}}"#);
    call(&mut world, &prover_run, 308, b"finding", input.as_bytes());
    assert_eq!(engine::finding_holder(&world.root, 1, Some(b"tests")), forge_top::Holder::Task(prover.task));
    tiers::sign_in(&mut world);
    let subject = forge_top::findings_subject(1, None, false);
    let document = world.oracle_document(world.signed_in.expect("owner"), 1, subject).expect("findings subject");
    let forge_top::Document::Findings(document) = document else { panic!("findings document") };
    let [row] = document.rows.as_ref() else { panic!("one finding") };
    assert_eq!(row.number, number);
    assert_eq!(row.state, forge_top::FindingState::Dropped);
    let input = format!(r#"{{"act":"show","number":{number}}}"#);
    let answer = call(&mut world, &prover_run, 3081, b"finding", input.as_bytes());
    let engine::CallAnswer::Finding(answer) = answer else { panic!("dropped finding") };
    let engine::FindingToolAnswer::Show { finding: Some(shown), .. } = *answer else { panic!("finding detail") };
    assert_eq!(shown.reason.as_deref(), Some(reason.as_bytes()));
    let answer = call(&mut world, &prover_run, 309, b"finding", br#"{"act":"list","state":"new","mine":true}"#);
    let engine::CallAnswer::Finding(answer) = answer else { panic!("mine") };
    let engine::FindingToolAnswer::List(list) = *answer else { panic!("mine list") };
    assert!(list.rows.is_empty(), "the repaired finding leaves the prover's new work");
    park(&mut world, &prover_run, 310, "Repair cleared; finding dropped");
    world.close();
}
