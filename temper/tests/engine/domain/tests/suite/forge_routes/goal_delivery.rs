//! A goal writes design straight to g while changes land beside it
//! (temper/docs/design/domain/forge/change/README.md, 1, and temper/docs/design/domain/forge/change/drift.md, 1, and temper/docs/design/domain/forge/change/questions.md, 1, and temper/docs/design/domain/forge/branches.md, 1;
//! temper/docs/design/domain/agent/tools.md;
//! temper/docs/design/protocol/configuration/reference.md).
use super::goal_branch::{branch_tip, forge_hold, goal_request, writer};
use super::goal_landing::{assert_goal_holds, ended, finish_run, land_into_goal, row, status};
use super::*;
use temper_engine_domain_forge_change::State;

const CHANNEL: Token = Token::new(7);
const STEPS: u32 = 1000; // The forge world's bound for a full landing and hold release.
const WORKER_SLOTS: u32 = 3; // Room for the goal, a sibling producer and an early producer.
const RESULT_BYTES: u32 = 224; // Three results, a landing subscription and 128 bytes of core news fit 1024 bytes.
const GOAL_REQUEST: u8 = 92;
const SIBLING_CALL: u64 = 93;
const DELIVERY_CALL: u64 = 94;
const DEPENDENT_CALL: u64 = 95;
const SIBLING_MERGES: u32 = 1;
const WAIT_TICKS: u32 = 20; // Two simulated seconds give the delegate time to claim the busy slot.

fn running_goal() -> (World, engine::Assignment, Box<[u8]>) {
    running_goal_with(false)
}

pub(super) fn running_goal_with(turn: bool) -> (World, engine::Assignment, Box<[u8]>) {
    let mut world = World::configured_limits(
        true,
        true,
        true,
        STEPS,
        Gate::default(),
        None,
        StoryLimits {
            turn,
            slots: Some(WORKER_SLOTS),
            result_bytes: Some(RESULT_BYTES),
            stall: turn.then_some(Duration::from_secs(3600)),
            // Two goal trains, their producer branches and their round branches can coexist.
            forge_branches: turn.then_some(8),
            ..StoryLimits::default()
        },
    )
    .with_landing_deployment(Box::new([engine::LandingRule {
        sets: Box::new([]),
        connector: 0,
        kind: 4,
        enforce: true,
        pattern: authority::Pattern {
            segments: [b"forge".as_slice(), b"forge.example", b"org", b"repo", b"branch"].map(Box::from).into(),
            last: authority::Last::Open(b"temper".as_slice().into()),
        },
        ci: true,
        // Child changes merge beside the goal's design writer.
        up_to_date: false,
        gates: Box::new([]),
        approvals: Box::new([]),
    }]));
    if turn {
        // A held train, its waiter and a second goal train keep their own resources.
        let mut client = world.env.limits.forge.client;
        client.resources = 12;
        client.write_attempts = 2;
        world = world.with_forge_client(client).with_forge_holds(12);
    }
    world.adopt();
    world.send(if turn {
        super::landing::hello_prepare(WORKER_SLOTS)
    } else {
        protocol::hello_slots(CHANNEL, WORKER_SLOTS, Box::new([]))
    });
    let people::Ask::Create { project, mut batch } = goal_request(&world, true) else { panic!("goal template") };
    batch[0].authority.tools |= jig_domain_core::ToolFamilies::JIG.control.0;
    let goal = assigned_chat(&mut world, GOAL_REQUEST, people::Ask::Create { project, batch });
    let branch = format!("temper/{}/g{}", goal.task, goal.task).into_bytes().into_boxed_slice();
    world.until(Until::Ready);
    (world, goal, branch)
}

fn await_end(world: &mut World, task: u64) {
    for _ in 0..STEPS {
        world.tick();
        if ended(world, task).is_some() {
            return;
        }
    }
    panic!("task {task} ended")
}

fn change_keys(world: &World) -> Vec<Key> {
    world.store.rows.keys().filter(|key| matches!(key, Key::Forge(forge_top::Key::ChangeHot(_)))).cloned().collect()
}

fn assert_design_first_recipe() {
    let stock = crate::stock::defaults(RESULT_BYTES);
    let charter = crate::reference_settings::charter(&stock, 5);
    let shared = std::str::from_utf8(&charter.policy.instructions).expect("shared instructions");
    assert!(shared.contains("live changes by task number"), "the recipe names live dependencies");
    let ask = include_str!("../../../../../../crates/temper-engine-service-stock/src/stock/ask.txt");
    assert!(ask.contains("Use after only for live prerequisites"), "Ask temper must propose live dependencies");
    assert!(ask.contains("cite design's repository path and section in words; omit its ended task from after"));
}

fn assert_ended_design_refused(world: &mut World, goal: &engine::Assignment, design: u64, words: &str) {
    let assigned = world.assigned.len();
    let made = live_task(world, goal.task).made;
    let changes = change_keys(world);
    let input = serde_json::json!({"words": words, "landing": "goal", "after": [design]}).to_string();
    let answer = mine(world, goal, DEPENDENT_CALL, input.as_bytes());
    let engine::CallAnswer::DelegationRefused(problem) = answer else { panic!("ended dependency refused: {answer:?}") };
    assert_eq!(problem.why, tasks::Refusal::Dependencies);
    assert_eq!(live_task(world, goal.task).made, made, "refusal creates no task");
    assert_eq!(change_keys(world), changes, "refusal creates no change");
    assert_eq!(world.assigned.len(), assigned, "refusal starts no producer");
}

fn pending_design_delivery(world: &mut World, goal: &engine::Assignment, branch: &[u8]) -> (u64, u64, u64) {
    let base = branch_tip(world, branch);
    let assigned = world.assigned.len();
    let answer = mine(world, goal, DELIVERY_CALL, br#"{"words":"Write the design","landing":"goal"}"#);
    let engine::CallAnswer::Delegated(made) = answer else { panic!("design change: {answer:?}") };
    let design = made[0].task;
    let producer = reviews::next_assignment(world, assigned);
    let head = early_start::push(world, &producer, base, b"docs/design/input.md");
    status(world, head, raw::Check::Pending);
    early_start::finish(world, &producer, b"Write design");
    (base, head, design)
}

#[test]
fn a_design_first_recipe_cites_landed_design_without_an_ended_dependency() {
    let (mut world, goal, branch) = running_goal();
    let (_, design_head, design) = pending_design_delivery(&mut world, &goal, &branch);
    land_delivery(&mut world, design, design_head);
    assert!(matches!(ended(&world, design).expect("design ended").phase, tasks::Phase::Ended(tasks::Ending::Done(_))));
    assert_file(&mut world, &branch, b"docs/design/input.md");
    read_news(&mut world, &goal);

    coding_from_landed_design(world, &goal, &branch, design);
    assert_design_first_recipe();
}

fn coding_from_landed_design(mut world: World, goal: &engine::Assignment, branch: &[u8], design: u64) {
    let words = "Implement docs/design/input.md, section 1, landed into g";
    assert_ended_design_refused(&mut world, goal, design, words);
    let assigned = world.assigned.len();
    let input = serde_json::json!({"words": words, "landing": "goal"}).to_string();
    let answer = mine(&mut world, goal, DEPENDENT_CALL + 1, input.as_bytes());
    let engine::CallAnswer::Delegated(made) = answer else { panic!("citing change admitted: {answer:?}") };
    let coding = made[0].task;
    assert!(live_task(&world, coding).dependencies.is_empty());
    let producer = reviews::next_assignment(&mut world, assigned);
    assert_eq!(live_task(&world, producer.task).requester, tasks::Party::Task(coding));
    assert_eq!(producer.workspace.repositories[0].start, engine::ForgeStart::Base(branch.into()));
    let start = branch_tip(&mut world, branch);
    assert_file(&mut world, branch, b"docs/design/input.md");
    let head = early_start::push(&mut world, &producer, start, b"implementation");
    world.tick();
    status(&mut world, head, raw::Check::Passed);
    finish_run(
        &mut world,
        &producer,
        tasks::TaskResult::Change {
            title: tasks::ChangeTitle::new(b"Implement the landed design".as_slice().into()),
            connector: 0,
            kind: 2,
            resource: forge_top::tag(forge_world::REPO.forge, forge_world::REPO.repository),
            words: b"pushed".as_slice().into(),
        },
    );
    land_delivery(&mut world, coding, head);
    assert_file(&mut world, branch, b"docs/design/input.md");
    assert_file(&mut world, branch, b"implementation");
    early_start::stop(&mut world, goal);
}

fn assert_confirmed_design_recipe() {
    let stock = crate::stock::defaults(RESULT_BYTES);
    let charter = crate::reference_settings::charter(&stock, 5);
    let first = std::str::from_utf8(&charter.parts[0].instructions).expect("first instructions");
    assert!(first.contains("deliver a design before you cut the changes that need it"));
    let ask = include_str!("../../../../../../crates/temper-engine-service-stock/src/stock/ask.txt");
    assert!(ask.contains("confirm all required design landed into g before dispatching coding"));
    assert!(ask.contains("A push, delivery or designer report is insufficient"));
}

fn live_design_allows_early_coding() {
    let (mut world, goal, branch) = running_goal();
    let (base, head, design) = pending_design_delivery(&mut world, &goal, &branch);
    let assigned = world.assigned.len();
    let input = serde_json::json!({"words": "Build on pending design", "landing": "goal", "after": [design]});
    let answer = mine(&mut world, &goal, DEPENDENT_CALL, input.to_string().as_bytes());
    let engine::CallAnswer::Delegated(made) = answer else { panic!("early coding admitted: {answer:?}") };
    let coding = made[0].task;
    let producer = reviews::next_assignment(&mut world, assigned);
    assert_eq!(live_task(&world, producer.task).requester, tasks::Party::Task(coding));
    assert_eq!(
        producer.workspace.repositories[0].start,
        engine::ForgeStart::Merged { branch: branch.clone(), heads: Box::new([translate::commit(head)]) }
    );
    assert_eq!(live_task(&world, coding).started_from.as_ref(), &[design]);
    assert!(ended(&world, design).is_none(), "dispatch and producer start precede design landing");
    assert_eq!(branch_tip(&mut world, &branch), base);
    assert_eq!(world.merges, 0);
    // This reproduces dispatch and run-start order; it makes no coding edit before landing.
    land_delivery(&mut world, design, head);
    early_start::stop(&mut world, &goal);
}

#[test]
fn design_first_waits_for_confirmed_landing_before_coding() {
    live_design_allows_early_coding();
    let (mut world, goal, branch) = running_goal();
    let (base, head, design) = pending_design_delivery(&mut world, &goal, &branch);
    let changes = change_keys(&world);
    let assigned = world.assigned.len();
    world.advance_ticks(WAIT_TICKS);
    assert!(ended(&world, design).is_none(), "a pushed delivery is not a confirmed landing");
    assert_eq!(branch_tip(&mut world, &branch), base);
    assert_eq!(change_keys(&world), changes, "the scripted lead waits without dispatching coding");
    assert_eq!(world.assigned.len(), assigned, "waiting starts no coding producer");
    land_delivery(&mut world, design, head);
    assert_file(&mut world, &branch, b"docs/design/input.md");
    read_news(&mut world, &goal);
    coding_from_landed_design(world, &goal, &branch, design);
    // The world exercises the recipe; these checks pin the model-facing rule, not model compliance.
    assert_confirmed_design_recipe();
}

#[test]
fn a_goals_design_delivery_and_a_sibling_merge_reach_g_without_a_slot() {
    let (mut world, goal, branch) = running_goal();
    let item = &goal.workspace.repositories[0];
    assert_eq!(item.start, engine::ForgeStart::Base(branch.clone()));
    assert_eq!(item.push.as_deref(), Some(branch.as_ref()));
    assert_eq!(item.landing.as_deref(), Some(branch.as_ref()));
    assert_eq!(writer(&world, &branch), None);
    let designers = &forge_hold(&world, &branch).expect("goal hold").design_writers;
    assert_eq!(
        designers.as_ref(),
        &[forge_top::DesignWriter { task: goal.task, attempt: goal.attempt, rereading: false }]
    );
    let home = goal
        .sections
        .iter()
        .find(|section| section.kind == engine::BriefKind::Forge(engine::ForgeBriefKind::Home))
        .expect("design delivery section");
    assert_eq!(home.body, engine::BriefBody::Text(b"deliver pushes the checkout's Markdown straight to the goal branch, merging its moved tip first; code goes through change.".as_slice().into()));
    let base = branch_tip(&mut world, &branch);
    deliver(&mut world, &goal, base, b"design.md");
    let merged = land_into_goal(&mut world, &goal, &branch, SIBLING_CALL, b"sibling");
    assert_eq!(world.merges, SIBLING_MERGES);
    assert_eq!(writer(&world, &branch), None);
    assert_goal_holds(&world, goal.task, &branch);
    let head = deliver(&mut world, &goal, merged, b"after-merge.md");
    super::goal_branch::observe_goal_branch(&mut world, &branch);
    assert_eq!(
        forge_hold(&world, &branch).expect("live goal hold").drift,
        None,
        "a design push after the merge is still owned"
    );
    assert_file(&mut world, &branch, b"design.md");
    assert_file(&mut world, &branch, b"sibling");
    assert_file(&mut world, &branch, b"after-merge.md");
    park(&mut world, &goal);
    for _ in 0..STEPS {
        world.tick();
        if forge_hold(&world, &branch).expect("retained goal hold").design_writers.is_empty() {
            break;
        }
    }
    let hold = forge_hold(&world, &branch).expect("retained goal hold");
    assert!(hold.design_writers.is_empty(), "{hold:?}");
    assert!(hold.drift.is_none());
    let name = hold.name.clone();
    assert!(world.store.rows.values().any(|row| matches!(row,
        Record::Forge { row } if matches!(row.as_ref(), forge_top::Stored::BranchHead(record) if record.name == name && record.commit == translate::commit(head) && record.owned))));
    super::goal_branch::observe_goal_branch(&mut world, &branch);
    assert!(forge_hold(&world, &branch).expect("goal hold after poll").drift.is_none());
    let second = wake(&mut world, &goal);
    assert_eq!(second.workspace.repositories[0].start, engine::ForgeStart::Base(branch.clone()));
    assert_eq!(second.workspace.repositories[0].push.as_deref(), Some(branch.as_ref()));
    early_start::stop(&mut world, &second);
}

#[test]
#[expect(clippy::too_many_lines, reason = "one story follows the delayed delivery read through the later outside push")]
fn a_goals_delivery_merging_its_moved_tip_is_never_drift() {
    let (mut world, goal, branch) = running_goal();
    let name = forge_hold(&world, &branch).expect("goal hold").name.clone();
    let h = branch_tip(&mut world, &branch);
    assert_record_tip(&world, &name, &branch, h);
    let path = b"design.md";
    let draft = world.worker_commit(h, path, b"added");
    let m = land_into_goal(&mut world, &goal, &branch, SIBLING_CALL, b"sibling");
    assert_ne!(h, m);
    assert_eq!(world.merges, SIBLING_MERGES);
    assert_record_tip(&world, &name, &branch, m);

    // As the delivery does on a moved lease: merge in the draft while
    // keeping M as the first parent and push conditionally on M.
    let tree = &world.fake.object(m).expect("merge tree").tree;
    let mut files = skein_lib::List::with_capacity(16);
    for (path, content) in tree {
        files.push(raw::File { path: path.clone(), content: content.clone() }).expect("small fixture tree");
    }
    files.push(raw::File { path: path.as_slice().into(), content: b"added".as_slice().into() }).unwrap();
    let c = fake::commit_tree(
        &mut world.fake,
        &world.fake_env.limits,
        m,
        Some(draft),
        files.into_boxed(),
        b"Merge moved tip",
    )
    .expect("delivery merge");
    // Delay the first keep call for C, not a poll after it has already
    // settled the write. The worker admission itself must not move M.
    world.slow_branch_latency = Duration::from_secs(240);
    world.slow_branch = Some(branch.clone());
    world.run_push(&goal, &branch, c, 2);
    super::goal_branch::hint_goal_branch(&mut world, &branch);
    let mut keep = None;
    for _ in 0..STEPS {
        keep = world.pending.iter().find_map(|(call, op)| match op {
            skein_forge::api::Op::Read(skein_forge::api::Read::Branch { branch: read }) if read == &branch => {
                Some(*call)
            }
            _ => None,
        });
        if keep.is_some() {
            break;
        }
        world.tick();
    }
    world.slow_branch = None;
    let keep = keep.expect("the keep read of g is out");
    assert!(world.pending.contains_key(&keep));
    assert_record_tip(&world, &name, &branch, m);
    world.advance_ticks(2400);
    for _ in 0..STEPS {
        if !world.pending.contains_key(&keep) {
            break;
        }
        world.tick();
    }
    world.until(Until::Ready);
    assert!(!world.pending.contains_key(&keep), "keep answered after four minutes");
    let hold = forge_hold(&world, &branch).expect("goal hold");
    assert!(hold.design_writers.contains(&forge_top::DesignWriter {
        task: goal.task,
        attempt: goal.attempt,
        rereading: false
    }));
    assert!(hold.drift.is_none());
    assert!(!matches!(live_task(&world, goal.task).phase, tasks::Phase::Held { .. }));
    assert_record_tip(&world, &name, &branch, c);
    assert_file(&mut world, &branch, b"sibling");
    assert_file(&mut world, &branch, path);

    park(&mut world, &goal);
    super::goal_branch::observe_goal_branch(&mut world, &branch);
    assert!(forge_hold(&world, &branch).expect("goal hold").design_writers.is_empty());
    assert!(forge_hold(&world, &branch).expect("goal hold").drift.is_none());
    assert_record_tip(&world, &name, &branch, c);

    let other = fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &branch, b"outside", b"moved", 3)
        .expect("outside push onto C");
    world.send(engine::Event::Forge(Box::new(forge_top::Event::Hint {
        hint: skein_forge::api::Hint {
            repository: forge_world::REPO,
            key: None,
            change: skein_forge::api::Change::BranchMoved {
                branch: branch.clone(),
                head: translate::commit(other),
                actor: 3,
            },
        },
    })));
    super::goal_branch::observe_goal_branch(&mut world, &branch);
    let tasks::Phase::Held { why: tasks::Hold::Drift, source: Some(source), .. } = &live_task(&world, goal.task).phase
    else {
        panic!("the outside push holds the goal")
    };
    let tasks::HoldSource::Account { words, .. } = source.as_ref() else { panic!("forge drift account") };
    let c_text = engine::commit_text(&translate::commit(c));
    let other_text = engine::commit_text(&translate::commit(other));
    let c_text = std::str::from_utf8(&c_text).unwrap();
    let other_text = std::str::from_utf8(&other_text).unwrap();
    let words = std::str::from_utf8(words).unwrap();
    assert!(words.contains(&format!("expected {c_text}, found {other_text}")), "{words}");
    assert!(
        words.contains(&format!(
            "temper's last write on it: push of {} by task {} attempt {}, confirmed",
            &c_text[..12],
            goal.task,
            goal.attempt
        )),
        "{words}"
    );
    assert_record_tip(&world, &name, &branch, c);
}

fn assert_record_tip(world: &World, name: &forge_top::Name, branch: &[u8], expected: u64) {
    let commit = Some(translate::commit(expected));
    assert_eq!(writes(world, name).head, commit);
    assert!(
        world.store.rows.values().any(|row| matches!(row, Record::Forge { row }
        if matches!(row.as_ref(), forge_top::Stored::Client(skein_forge_client::Stored::Live(live))
        if live.watch.resource.what == skein_forge_client::What::Branch(branch.into()) && live.pushed == commit))),
        "the client's durable expectation follows the record"
    );
}

fn read_news(world: &mut World, run: &engine::Assignment) {
    world.scripted_read_inbox(run);
}

fn mine(world: &mut World, run: &engine::Assignment, number: u64, input: &[u8]) -> engine::CallAnswer {
    let input = smith_domain_run::HostInput::attested(input.into()).expect("bounded change input");
    let temper_engine_smith::Decoded::Call(call) = temper_engine_smith::call(
        smith_domain_run::CallName { activation: run.attempt, completion: 1, position: 1 },
        b"change",
        &input,
    )
    .expect("change decodes") else {
        panic!("change call")
    };
    world.scripted_call(run, number, call.tool)
}

fn writes<'a>(world: &'a World, name: &forge_top::Name) -> &'a forge_top::Writes {
    match world.store.rows.get(&Key::Forge(forge_top::Key::Writes(name.clone()))) {
        Some(Record::Forge { row }) => match row.as_ref() {
            forge_top::Stored::Writes(writes) => writes,
            other => panic!("branch record: {other:?}"),
        },
        other => panic!("branch record: {other:?}"),
    }
}

#[test]
fn a_design_writers_second_lease_confirms_the_first_and_its_answer_confirms_the_second() {
    use forge_top::{Sender, WriteState, writes::Operation};
    let (mut world, goal, branch) = running_goal();
    let name = forge_hold(&world, &branch).expect("design hold").name.clone();
    let sender = Sender::Attempt { task: goal.task, attempt: goal.attempt };
    let base = branch_tip(&mut world, &branch);
    let first = early_start::push(&mut world, &goal, base, b"first.md");
    let admitted = writes(&world, &name).entries.last().expect("first admission");
    assert_eq!(admitted.state, WriteState::Admitted);
    assert_eq!(admitted.sender, sender);
    let second = early_start::push(&mut world, &goal, first, b"second.md");
    let row = writes(&world, &name);
    let [first_write, second_write] = &row.entries[row.entries.len() - 2..] else { panic!("both admissions") };
    assert_eq!(first_write.state, WriteState::Confirmed, "the next lease read the first push");
    assert_eq!(second_write.state, WriteState::Admitted);
    assert_eq!(second_write.sender, sender);
    assert_eq!(
        second_write.operation,
        Operation::Push { before: Some(translate::commit(first)), after: translate::commit(second) }
    );
    park(&mut world, &goal);
    for _ in 0..STEPS {
        world.tick();
        let row = writes(&world, &name);
        if row.entries.last().expect("second push").state == WriteState::Confirmed {
            assert_eq!(row.head, Some(translate::commit(second)));
            return;
        }
    }
    panic!("the answer's readback confirms the second push");
}

#[test]
fn a_slot_writers_unmade_push_is_refused_at_its_answer() {
    use forge_top::{WriteState, writes::Operation};
    use temper_protocol_worker_wire::v1::Answer;
    let (mut world, goal, goal_branch) = running_goal();
    let base = branch_tip(&mut world, &goal_branch);
    let assigned = world.assigned.len();
    let answer = mine(&mut world, &goal, DELIVERY_CALL, br#"{"words":"Write a file","landing":"goal"}"#);
    assert!(matches!(answer, engine::CallAnswer::Delegated(_)));
    let producer = reviews::next_assignment(&mut world, assigned);
    let item = &producer.workspace.repositories[0];
    let branch = item.push.as_ref().expect("slot branch");
    let kept = early_start::push(&mut world, &producer, base, b"kept");
    let skipped = world.worker_commit(kept, b"not pushed", b"lost the lease");
    // The object is available for ancestry reads, but was never on the slot's branch.
    assert_eq!(
        world.external(
            1,
            raw::Op::Git(raw::Git::PushNew { branch: b"someone-elses-branch".as_slice().into(), commit: skipped })
        ),
        raw::Answer::PushNew { existing: None }
    );
    let attempt = (CHANNEL, producer.task, producer.attempt);
    let Some(Answer::Unadmitted(refusal)) = world.ask_push(attempt, item.tag, &goal_branch, Some(base), skipped) else {
        panic!("a different branch is not this attempt's destination")
    };
    assert!(refusal.words.windows(goal_branch.len()).any(|part| part == goal_branch.as_ref()));
    assert_eq!(world.ask_push(attempt, item.tag, branch, Some(kept), skipped), Some(Answer::Admitted));
    let name = forge_hold(&world, branch).expect("held slot").name.clone();
    let last = writes(&world, &name).entries.last().expect("unmade admission");
    assert_eq!(last.state, WriteState::Admitted);
    let order = last.order;
    early_start::report_checked_push(&mut world, &producer, kept);
    for _ in 0..STEPS {
        world.tick();
        let row = writes(&world, &name);
        if !row.entries.iter().any(|entry| entry.order == order) {
            assert_eq!(row.head, Some(translate::commit(kept)));
            assert!(row.entries.iter().any(|entry| entry.state == WriteState::Confirmed
                && matches!(
                    entry.operation, Operation::Push { after, .. } if after == translate::commit(kept)
                )));
            assert_eq!(
                world.ask_push(attempt, item.tag, branch, Some(kept), skipped),
                None,
                "jig fences the ended attempt"
            );
            return;
        }
    }
    panic!("the answer's readback refuses and erases the unmade push");
}

fn push_branch(run: &engine::Assignment) -> Box<[u8]> {
    run.workspace.repositories[0].push.clone().expect("goal delivery destination")
}

pub(super) fn deliver(world: &mut World, run: &engine::Assignment, base: u64, file: &[u8]) -> u64 {
    let head = early_start::push(world, run, base, file);
    world.tick();
    status(world, head, raw::Check::Pending);
    early_start::observe_push(world, run);
    head
}

fn make_delivery(world: &mut World, run: &engine::Assignment, number: u64) -> u64 {
    let answer = mine(world, run, number, br#"{"words":"Land goal","branch":"mine"}"#);
    let engine::CallAnswer::Delegated(made) = answer else { panic!("delivery admitted: {answer:?}") };
    made[0].task
}

fn land_delivery(world: &mut World, change: u64, head: u64) {
    status(world, head, raw::Check::Passed);
    await_end(world, change);
    assert!(matches!(row(world, change).change.state, State::Landed { .. }), "{:?}", row(world, change));
    assert_eq!(row(world, change).change.last_head, Some(translate::commit(head)));
    world.until(Until::Ready);
}

fn park(world: &mut World, run: &engine::Assignment) {
    // With no pending delegate, a timer beyond the story's horizon keeps the
    // goal waiting; topic subscriptions alone do not.
    if live_task(world, run.task).delegates.is_empty() {
        if let Some(subscription) = live_task(world, run.task).subscriptions.first().map(|item| item.number) {
            world.scripted_call(run, DEPENDENT_CALL, engine::Tool::Unsubscribe { subscription });
        }
        let answer = world.scripted_call(
            run,
            DEPENDENT_CALL + 1,
            engine::Tool::Subscribe {
                kind: tasks::SubscriptionKind::Timer { at: skein_lib::Wall::from_nanos(u64::MAX), period: None },
            },
        );
        assert!(matches!(answer, engine::CallAnswer::Subscribed { .. }), "{answer:?}");
    }
    read_news(world, run);
    let task = live_task(world, run.task);
    super::park_after_turn(world, run, None, task.turn);
    world.until(Until::Ready);
    assert_eq!(live_task(world, run.task).phase, tasks::Phase::Active(tasks::Active::Idle));
}

fn wake(world: &mut World, run: &engine::Assignment) -> engine::Assignment {
    world.signed_in = None;
    world.sign_in();
    assigned_chat(
        world,
        GOAL_REQUEST + 1,
        people::Ask::Say { project: 1, task: run.task, words: b"Continue".as_slice().into() },
    )
}

fn assert_file(world: &mut World, branch: &[u8], file: &[u8]) {
    let tip = branch_tip(world, branch);
    assert_eq!(
        world.fake.object(tip).expect("branch tip").tree.get(file).map(AsRef::as_ref),
        Some(b"added".as_slice())
    );
}

#[test]
fn a_goals_second_run_starts_from_its_direct_design_delivery() {
    let (mut world, goal, branch) = running_goal();
    let base = branch_tip(&mut world, &branch);
    let head = deliver(&mut world, &goal, base, b"first.md");
    park(&mut world, &goal);
    let second = wake(&mut world, &goal);
    let item = &second.workspace.repositories[0];
    assert_eq!(item.start, engine::ForgeStart::Base(branch.clone()));
    assert_eq!(item.push.as_deref(), Some(branch.as_ref()));
    let head = deliver(&mut world, &second, head, b"second.md");
    assert_eq!(branch_tip(&mut world, &branch), head);
    assert_eq!(world.merges, 0);
    assert_file(&mut world, &branch, b"first.md");
    assert_file(&mut world, &branch, b"second.md");
    early_start::stop(&mut world, &second);
}

fn checking(world: &mut World, change: u64, head: u64) {
    for _ in 0..STEPS {
        world.tick();
        if row(world, change).change.state == (State::Checking { head: translate::commit(head) }) {
            return;
        }
    }
    panic!("change checks head {head}: {:?}", row(world, change));
}

#[test]
fn a_delivery_under_its_open_change_is_that_changes_new_head() {
    let (mut world, goal, branch) = running_goal();
    let r = push_branch(&goal);
    let base = branch_tip(&mut world, &branch);
    let h1 = deliver(&mut world, &goal, base, b"first");
    let c1 = make_delivery(&mut world, &goal, DELIVERY_CALL);
    checking(&mut world, c1, h1);
    let h2 = world.worker_commit(h1, b"second", b"added");
    world.worker_push((CHANNEL, goal.task, goal.attempt), &goal.workspace.repositories[0], &r, h2, 1);
    status(&mut world, h2, raw::Check::Pending);
    early_start::observe_push(&mut world, &goal);
    status(&mut world, h1, raw::Check::Passed);
    let mut read_ci = false;
    for _ in 0..STEPS {
        world.tick();
        read_ci |= world.pending.values().any(|op| {
            matches!(op,
            skein_forge::api::Op::Read(skein_forge::api::Read::Checks { commit })
                if *commit == translate::commit(h2))
                || matches!(op, skein_forge::api::Op::Read(skein_forge::api::Read::Pull { number, ci: true })
                    if row(&world, c1).pull == Some(*number))
        });
        assert_eq!(world.merges, 0, "the superseded head never lands");
        assert_eq!(writer(&world, &r), None);
        if read_ci && row(&world, c1).change.state == (State::Checking { head: translate::commit(h2) }) {
            break;
        }
    }
    assert!(read_ci, "CI is read again at the new delivery");
    assert_eq!(row(&world, c1).change.state, State::Checking { head: translate::commit(h2) });
    land_delivery(&mut world, c1, h2);
    let pull = row(&world, c1).pull.expect("delivery pull");
    assert!(!world.writes.iter().any(|write| matches!(write,
        skein_forge::api::Write::Merge { number, head, .. } if *number == pull && *head == translate::commit(h1))));
    assert_file(&mut world, &branch, b"first");
    assert_file(&mut world, &branch, b"second");
    early_start::stop(&mut world, &goal);
}

fn deleted(world: &World, branch: &[u8]) -> bool {
    world.writes.iter().any(|write| {
        matches!(write,
        skein_forge::api::Write::DeleteBranch { branch: gone } if gone.as_ref() == branch)
    })
}

#[test]
fn a_goals_run_branch_is_not_watched_after_claim_or_restart() {
    use super::resource_admission::{branch_reads, kept_branch};
    let (mut world, goal, branch) = running_goal();
    let r = format!("temper/{}/r{}", goal.task, goal.task).into_bytes();
    world.forge_reads = Some(Vec::new());
    // A claim merges existing names with its writes. Older saved names can include r.
    let resources = [&r[..], branch.as_ref()].map(|branch| forge_top::Name {
        forge: 1,
        repository: 2,
        what: forge_top::What::Branch(branch.split(|byte| *byte == b'/').map(Box::from).collect()),
    });
    world.send(engine::Event::Forge(Box::new(forge_top::Event::Names {
        task: goal.task,
        resources: Box::new(resources),
    })));
    world.until(Until::Ready);
    for restart in [false, true] {
        if restart {
            world.restart_same();
            world.until(Until::Ready);
        }
        assert!(forge_hold(&world, &r).is_some(), "the unused run branch remains held");
        assert!(!kept_branch(&world, &r), "a workspace claim must not keep r");
        assert!(kept_branch(&world, &branch), "g remains watched");
        world.advance_ticks(STEPS);
        assert_eq!(branch_reads(&world, &r), 0, "no run-branch reads, restart={restart}");
        assert!(branch_reads(&world, &branch) > 0);
    }
    world.close();
}

#[test]
fn a_goals_close_deletes_no_unused_run_branch() {
    let (mut world, goal, branch) = running_goal();
    let r = format!("temper/{}/r{}", goal.task, goal.task).into_bytes();
    let base = branch_tip(&mut world, &branch);
    deliver(&mut world, &goal, base, b"design.md");
    let report = tasks::TaskResult::Report { words: b"Done".as_slice().into() };
    finish_run(&mut world, &goal, report.clone());
    await_end(&mut world, goal.task);
    world.until(Until::Ready);
    assert_eq!(ended(&world, goal.task).expect("goal ended").phase, tasks::Phase::Ended(tasks::Ending::Done(report)));
    assert!(!deleted(&world, &r));
    assert!(forge_hold(&world, &r).is_none());
    world.close();
}

// temper/docs/design/domain/forge/queues.md: a queued change follows its goal's priority-only amendment.
#[test]
fn a_goals_priority_amendment_moves_its_queued_change() {
    let (mut world, goal, branch) = running_goal();
    let base = branch_tip(&mut world, &branch);
    let head = deliver(&mut world, &goal, base, b"priority");
    let change = make_delivery(&mut world, &goal, DELIVERY_CALL);
    status(&mut world, head, raw::Check::Passed);
    for _ in 0..STEPS {
        world.tick();
        if matches!(row(&world, change).change.state, State::Queued { .. }) {
            break;
        }
    }
    assert!(matches!(row(&world, change).change.state, State::Queued { .. }), "setup: the change is queued");
    let old_priority = live_task(&world, goal.task).tracked.expect("tracked goal");
    assert_eq!(row(&world, change).priority, i32::try_from(old_priority).unwrap());
    let priority = old_priority + 1;
    let amendment = people::Amendment {
        tracked: Some(people::Tracking::Tracked { priority }),
        addressed: None,
        instructions: None,
        spec: None,
        wake: None,
        dependencies: None,
        authority: None,
        llm: None,
        reason: b"Change queue priority".as_slice().into(),
    };
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(96)),
        sign_in: world.signed_in.expect("owner session"),
        key: [96; 16],
        ask: people::Ask::Amend { project: 1, task: goal.task, revision: 0, amendment, release: None },
    }));
    world.until(Until::Ready);
    assert_eq!(live_task(&world, goal.task).tracked, Some(priority), "the priority-only amendment was committed");
    assert_eq!(
        row(&world, change).priority,
        i32::try_from(priority).unwrap(),
        "the goal feed must save its change's new priority"
    );
    land_delivery(&mut world, change, head);
    finish_run(&mut world, &goal, tasks::TaskResult::Report { words: b"Done".as_slice().into() });
    await_end(&mut world, goal.task);
    world.until(Until::Ready);
    world.close();
}
