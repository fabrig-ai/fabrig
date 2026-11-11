//! A goal's dependent changes land under its branch's rules and keep their reports
//! (temper/docs/design/domain/agent/tools.md, 1; temper/docs/design/domain/forge/change/README.md, 2, and temper/docs/design/domain/forge/change/gates.md, 2, and temper/docs/design/domain/forge/branches.md, 1).
use super::goal_branch::{branch_tip, forge_hold, goal_request};
use super::*;

const CHANNEL: Token = Token::new(7);
const GOAL_REQUEST: u8 = 92;
const CHANGE_CALL: u64 = 93;
const STEPS: u32 = 1000; // The forge world's bound for a full landing and hold release.
const TWO_CHANGES: usize = 2;
const GOAL_AND_A: usize = 2;
const GOAL_AND_BOTH_PRODUCERS: usize = 3;
// Two result reservations leave room for the goal's messages in its 1024-byte inbox.
const RESULT_BYTES: u32 = 384;
const A_REPORT: &[u8] = b"Done: added A.\n## Left\n- none";
const B_REPORT: &[u8] = b"Done: added B.\n## Left\n- none";

fn world() -> World {
    world_with_turns(8)
}

fn world_with_turns(turns: u32) -> World {
    let mut world = World::configured_limits(
        true,
        true,
        true,
        STEPS,
        Gate::default(),
        None,
        StoryLimits { result_bytes: Some(RESULT_BYTES), forge_charter: Some(2), ..StoryLimits::default() },
    );
    world.env.limits.tasks.batch = u32::try_from(TWO_CHANGES).expect("two changes fit u32");
    world = world.with_landing_deployment(goal_landing_rules());
    world.adopt();
    let mut hello = protocol::hello_slots(CHANNEL, 2, Box::new([]));
    let engine::Event::Worker(engine::WorkerInput::Link { link: engine::Link::Hello { hello: bounds, .. }, .. }) =
        &mut hello
    else {
        panic!("worker hello")
    };
    bounds.bounds.turns = turns;
    world.send(hello);
    world
}

fn goal_landing_rules() -> Box<[engine::LandingRule]> {
    let segments = [b"forge".as_slice(), b"forge.example", b"org", b"repo", b"branch"].map(Box::from);
    Box::new([
        engine::LandingRule {
            sets: Box::new([]),
            connector: 0,
            kind: 4,
            enforce: true,
            pattern: authority::Pattern {
                segments: segments.clone().into(),
                last: authority::Last::Exact(b"main".as_slice().into()),
            },
            ci: true,
            up_to_date: true,
            gates: Box::new([]),
            approvals: Box::new([]),
        },
        engine::LandingRule {
            sets: Box::new([]),
            connector: 0,
            kind: 4,
            enforce: true,
            pattern: authority::Pattern {
                segments: segments.into(),
                last: authority::Last::Open(b"temper".as_slice().into()),
            },
            ci: false,
            up_to_date: false,
            gates: Box::new([]),
            approvals: Box::new([]),
        },
    ])
}

fn change_row(world: &World, task: u64) -> &forge_top::ChangeRow {
    let Some(Record::Forge { row }) = world.store.rows.get(&Key::Forge(forge_top::Key::Change(task))) else {
        panic!("registered change {task}")
    };
    let forge_top::Stored::Change(row) = row.as_ref() else { panic!("change row") };
    row
}

fn ended(world: &World, task: u64) -> Option<&tasks::TaskRecord> {
    match world.store.rows.get(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Ended(task)))) {
        Some(Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Ended(row)))) => Some(row),
        _ => None,
    }
}

fn finish(world: &mut World, run: &engine::Assignment, result: tasks::TaskResult) {
    let event = world.protocol.answer(
        CHANNEL,
        run.task,
        run.attempt,
        5,
        tasks::End::Finished { result, cancel_delegates: false },
        engine::Parts::none(),
    );
    world.send(event);
}

fn push(world: &mut World, producer: &engine::Assignment, base: u64, file: &[u8]) -> u64 {
    let branch = producer.workspace.repositories[0].push.clone().expect("producer's destination");
    assert_eq!(
        world.external(2, raw::Op::Git(raw::Git::Create { branch: branch.clone(), commit: base })),
        raw::Answer::Branch(raw::Created::Created)
    );
    let head = fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &branch, file, b"added", 2)
        .expect("producer pushed its change");
    assert_eq!(
        world.external(
            1,
            raw::Op::Write(raw::Write::Status {
                commit: head,
                context: b"build".as_slice().into(),
                state: raw::Check::Pending,
                description: Box::new([]),
                link: Box::new([]),
            })
        ),
        raw::Answer::Done
    );
    head
}

fn producer_result(title: &[u8], report: &[u8]) -> tasks::TaskResult {
    tasks::TaskResult::Change {
        title: tasks::ChangeTitle::new(title.into()),
        connector: 0,
        kind: 2,
        resource: forge_top::tag(forge_world::REPO.forge, forge_world::REPO.repository),
        words: report.into(),
    }
}

fn assert_landed(world: &mut World, goal: u64, change: u64, base: u64, head: u64, report: &[u8]) -> u64 {
    for _ in 0..STEPS {
        world.tick();
        if ended(world, change).is_some() {
            break;
        }
    }
    let row = change_row(world, change);
    assert!(
        matches!(row.change.state, temper_engine_domain_forge_change::State::Landed { .. }),
        "change {change}: {row:?}"
    );
    assert_eq!(row.report.as_ref(), report);
    assert_eq!((row.change.updates, row.change.repairs, row.change.resolutions), (0, 0, 0));
    let number = row.pull.expect("landed pull");
    let branch = row.base.clone();
    let tasks::Phase::Ended(tasks::Ending::Done(tasks::TaskResult::Change { words, .. })) =
        &ended(world, change).expect("change ended").phase
    else {
        panic!("change ended done")
    };
    let words = core::str::from_utf8(words).expect("UTF-8 change result");
    let landing = format!(
        "Landed: pull request #{number} merged into {} as ",
        core::str::from_utf8(&branch).expect("goal branch")
    );
    let lines: Vec<_> = words.lines().collect();
    assert!(lines[2].starts_with(&landing), "{words}");
    assert_eq!(lines[3], "Report:");
    assert_eq!(&lines[4..7], core::str::from_utf8(report).expect("UTF-8 report").lines().collect::<Vec<_>>());
    assert!(
        live_task(world, goal).inbox.iter().any(|message| {
            message.from == tasks::Party::Task(change)
                && matches!(message.kind, tasks::MessageKind::Result(tasks::ResultKind::Change { .. }))
                && message.words.as_ref() == words.as_bytes()
        }),
        "the goal receives its change's report"
    );
    assert!(world.writes.iter().any(|write| matches!(write,
        skein_forge::api::Write::Merge { number: pull, head: merged, how: skein_forge::api::MergeHow::Style }
            if *pull == number && *merged == translate::commit(head))));
    let raw::Answer::Pull(pull) = world.external(1, raw::Op::Read(raw::Read::Pull { number })) else {
        panic!("landed pull")
    };
    assert_eq!(pull.base, branch);
    assert_eq!(pull.commit, head, "the pending head lands without an update");
    assert!(
        pull.statuses.iter().any(|status| status.context.as_ref() == b"build" && status.state == raw::Check::Pending)
    );
    let merge = pull.merged.expect("merged with CI still pending");
    let raw::Answer::Commits { commits } =
        world.external(1, raw::Op::Read(raw::Read::Commits { before: base, after: merge }))
    else {
        panic!("merge ancestry")
    };
    assert_eq!(commits.len(), 1, "skein/docs/design/testing/fakes/forge.md, 2.2: a squash adds one commit");
    let commit = commits.iter().find(|commit| commit.id == merge).expect("merge commit");
    assert_eq!(
        commit.parents.as_ref(),
        &[base],
        "skein/docs/design/testing/fakes/forge.md, 2.2; temper/docs/design/domain/forge/adoption.md, 4: the repository's style squashes onto the base's tip"
    );
    let Ok(raw::Answer::Tree(landed_tree)) =
        world.fake.inspect(&world.fake_env.limits, b"org/repo", &raw::Read::Tree { commit: merge })
    else {
        panic!("landed tree")
    };
    let Ok(raw::Answer::Tree(produced_tree)) =
        world.fake.inspect(&world.fake_env.limits, b"org/repo", &raw::Read::Tree { commit: head })
    else {
        panic!("producer's tree")
    };
    assert_eq!(
        landed_tree, produced_tree,
        "the squash keeps the producer's tree (skein/docs/design/testing/fakes/forge.md, 2.2)"
    );
    assert_eq!(branch_tip(world, &branch), merge);
    assert_eq!(forge_hold(world, &branch).expect("goal still holds its branch").task, goal);
    merge
}

#[test]
fn a_goal_makes_two_changes_into_its_branch_the_second_after_the_first() {
    let mut world = world();
    let people::Ask::Create { project, mut batch } = goal_request(&world, true) else { panic!("goal template") };
    batch[0].authority.tools |= jig_domain_core::ToolFamilies::JIG.control.0;
    let goal = assigned_chat(&mut world, GOAL_REQUEST, people::Ask::Create { project, batch });
    world.until(Until::Ready);
    let branch = format!("temper/{}/g{}", goal.task, goal.task).into_bytes().into_boxed_slice();
    let row = live_task(&world, goal.task);
    assert_eq!(row.tracked, Some(0));
    assert!(matches!(row.phase, tasks::Phase::Active(tasks::Active::Claimed { .. } | tasks::Active::Running { .. })));
    assert_eq!(forge_hold(&world, &branch).expect("G holds g").task, goal.task);
    let main = branch_tip(&mut world, b"main");
    assert_eq!(branch_tip(&mut world, &branch), main, "g is made at main's tip");

    let input = smith_domain_run::HostInput::attested(
        br#"{"changes":[{"name":"a","words":"Add A","landing":"goal"},{"words":"Add B","landing":"goal","after":["a"]}]}"#.as_slice().into(),
    ).expect("bounded change input");
    let temper_engine_smith::Decoded::Call(call) = temper_engine_smith::call(
        smith_domain_run::CallName { activation: goal.attempt, completion: 1, position: 1 },
        b"change",
        &input,
    )
    .expect("change decodes") else {
        panic!("change call")
    };
    let event = world.protocol.call(CHANNEL, goal.task, goal.attempt, CHANGE_CALL, call);
    world.send(event);
    world.until(Until::Ready);
    let Some(engine::CallAnswer::Delegated(made)) = world.answers.last() else {
        panic!("both changes admitted: {:?}; {:?}", world.answers, world.call_errors)
    };
    assert_eq!(made.len(), TWO_CHANGES);
    let (a, b) = (made[0].task, made[1].task);
    assert_eq!(live_task(&world, a).spec.words.as_ref(), b"Add A");
    assert_eq!(live_task(&world, b).spec.words.as_ref(), b"Add B");
    assert_eq!(live_task(&world, b).dependencies.as_ref(), &[a]);
    let producer_a = reviews::next_assignment(&mut world, 1);
    world.until(Until::Ready);
    assert_eq!(world.assigned.len(), GOAL_AND_A, "B waits while a worker slot is free");
    assert_eq!(live_task(&world, producer_a.task).requester, tasks::Party::Task(a));
    assert_eq!(change_row(&world, a).base, branch);
    let a_head = push(&mut world, &producer_a, main, b"A");
    let moved_main = fake::advance(&mut world.fake, &world.fake_env, b"org/repo", b"main", b"other", b"main moved", 1)
        .expect("main advances while G's first change is being produced");
    finish(&mut world, &producer_a, producer_result(b"Add A", A_REPORT));
    let a_merge = assert_landed(&mut world, goal.task, a, main, a_head, A_REPORT);

    let producer_b = reviews::next_assignment(&mut world, 2);
    assert!(ended(&world, a).is_some(), "B starts only after A is done");
    assert_eq!(live_task(&world, producer_b.task).requester, tasks::Party::Task(b));
    assert_eq!(change_row(&world, b).base, branch);
    let b_head = push(&mut world, &producer_b, a_merge, b"B");
    finish(&mut world, &producer_b, producer_result(b"Add B", B_REPORT));
    let b_merge = assert_landed(&mut world, goal.task, b, a_merge, b_head, B_REPORT);
    assert_eq!(world.merges, 2);
    assert_eq!(world.assigned.len(), GOAL_AND_BOTH_PRODUCERS, "neither change asks a review or repair");
    assert!(!world.writes.iter().any(|write| matches!(write, skein_forge::api::Write::Update { .. })));
    assert_eq!(branch_tip(&mut world, b"main"), moved_main);

    finish(&mut world, &goal, tasks::TaskResult::Report { words: b"Done: both changes landed.".as_slice().into() });
    for _ in 0..STEPS {
        world.tick();
        if ended(&world, goal.task).is_some() {
            break;
        }
    }
    assert!(matches!(ended(&world, goal.task).expect("G ended").phase, tasks::Phase::Ended(tasks::Ending::Done(_))));
    world.until(Until::Ready);
    assert!(forge_hold(&world, &branch).is_none(), "G releases its hold as it ends");
    assert_eq!(branch_tip(&mut world, &branch), b_merge, "g survives G's ending");
    assert!(!world.writes.iter().any(|write| matches!(write,
        skein_forge::api::Write::DeleteBranch { branch: deleted } if *deleted == branch)));
    world.close();
}

const DESIGN: &[u8] = b"docs/design/formats.md";
const HOST_REPLY: u32 = 1024;
const PAGE_LINES: usize = 16;
const DESIGN_LINES: usize = 32;
const LINE_BYTES: usize = 64;

fn inspecting_goal() -> (World, engine::Assignment, Box<[u8]>) {
    use jig_domain_core::settings as s;
    let mut world = world_with_turns(32);
    world.fake_env.limits.limits.content_bytes = 4096;
    let mut charter = walking::charter();
    charter.policy.resume = false;
    charter.policy.modify = false;
    charter.policy.shell = false;
    charter.policy.turns = 32;
    world.send(engine::Event::Party(engine::PartyInput::ChangeSettings {
        reply_to: ReplyTo::new(Token::new(300)),
        sign_in: world.signed_in.expect("deployment owner"),
        key: [30; 16],
        digest: [30; 32],
        kinds: Box::new([]),
        change: Box::new(s::Change {
            against: 1,
            items: Box::new([s::Item {
                setting: s::Setting::Charter(1),
                edit: s::Edit::Set(s::Value::Charter(Box::new(charter))),
            }]),
            running: s::Running::Stays,
        }),
    }));
    world.until(Until::Ready);
    let people::Ask::Create { project, mut batch } = goal_request(&world, true) else { panic!("goal request") };
    batch[0].flags.conversation = true;
    let goal = assigned_chat(&mut world, GOAL_REQUEST, people::Ask::Create { project, batch });
    world.until(Until::Ready);
    let branch = format!("temper/{}/g{}", goal.task, goal.task).into_bytes().into();
    (world, goal, branch)
}

fn land_design(
    world: &mut World,
    goal: &engine::Assignment,
    branch: &[u8],
    marker: &[u8],
) -> (engine::Assignment, u64) {
    super::goal_tier::read_messages(world, goal);
    let mut text = [b"x".repeat(LINE_BYTES - 1), b"\n".to_vec()].concat().repeat(DESIGN_LINES);
    text.extend_from_slice(marker);
    text.push(b'\n');
    assert!(text.len() > usize::try_from(HOST_REPLY).expect("host cap"));
    let completion = 100 + u32::try_from(world.answers.len()).expect("bounded calls");
    let answer = super::change_tool::named_read(
        world,
        goal,
        completion,
        b"change",
        br#"{"words":"Land the design","landing":"goal"}"#,
    );
    assert!(
        matches!(answer, engine::CallAnswer::Delegated(_)),
        "designer admitted: {answer:?}; numbers={:?}",
        live_task(world, goal.task).numbers
    );
    let (taken, turns) = super::goal_tier::read_messages(world, goal);
    super::park_after_turn(world, goal, taken, turns);
    let change = temper_engine_domain_world::tier_runs::produce_delegated(world, answer, branch, DESIGN, &text);
    for _ in 0..STEPS {
        world.tick();
        if ended(world, change).is_some() {
            let head = branch_tip(world, branch);
            let file = world.fake.object(head).expect("landed head").tree.get(DESIGN).expect("landed design");
            assert!(
                file.windows(marker.len()).any(|bytes| bytes == marker),
                "change {change} should land {}",
                core::str::from_utf8(marker).expect("marker")
            );
            if let Some(run) = world.assigned.iter().find(|run| run.task == goal.task && run.attempt > goal.attempt) {
                assert!(!run.run.policy.resume);
                return (run.clone(), head);
            }
        }
    }
    panic!("designer's landed change ends");
}

fn read_turn(name: &[u8], input: &[u8]) -> skein_fake_llm_domain::api::Turn {
    smith_agent_world::fixture::calls(
        Box::new([skein_fake_llm_domain::api::Line::Call { name: name.into(), arguments: input.into() }]),
        1,
    )
}

fn design_pages() -> Vec<skein_fake_llm_domain::api::Turn> {
    (1..=DESIGN_LINES + 1)
        .step_by(PAGE_LINES)
        .map(|offset| {
            read_turn(
                b"read",
                format!(
                    r#"{{"path":"docs/design/formats.md","offset":{offset},"window":{}}}"#,
                    PAGE_LINES * LINE_BYTES,
                )
                .as_bytes(),
            )
        })
        .collect()
}

fn design_scripts(input: &[u8]) -> Box<[skein_fake_llm_domain::api::Script]> {
    use skein_fake_llm_domain::api::{Line, Script};
    let mut parent = vec![
        read_turn(b"read_file", input),
        read_turn(b"sub_agent", br#"{"brief":"@survey-design","tools":["inspect"]}"#),
    ];
    parent.extend(design_pages());
    parent.push(read_turn(b"finish", br#"{"report":"Read the design","source":"workspace"}"#));
    let mut child = design_pages();
    child.push(smith_agent_world::fixture::says(
        Box::new([Line::Text { text: b"Survey read the design".as_slice().into() }]),
        1,
    ));
    Box::new([
        Script { cue: b"@goal-design".as_slice().into(), turns: parent.into() },
        Script { cue: b"@survey-design".as_slice().into(), turns: child.into() },
    ])
}

fn inspection_charter(run: &engine::Assignment) -> (smith_agent_world::Settings, smith_domain_run::Charter) {
    let mut settings = smith_agent_world::Settings::calm(251);
    settings.limits.run.host_reply_bytes = HOST_REPLY;
    let mut charter = temper_engine_smith::charter(run);
    assert!(charter.grants.tools.inspect && !charter.grants.tools.modify);
    assert!(charter.grants.deliver.is_none());
    settings.limits.run.host_tools = u32::try_from(charter.grants.host_tools.len()).expect("host tools");
    settings.limits.run.brief_sections = u32::try_from(charter.brief.sections.len()).expect("brief sections");
    charter.llm = smith_agent_world::scripted_charter(&settings).llm;
    charter.models = Box::new([]);
    charter.instructions = b"@goal-design".as_slice().into();
    (settings, charter)
}

fn read_landed_design(world: &mut World, run: &engine::Assignment, head: u64, marker: &[u8]) {
    assert!(
        matches!(&run.workspace.repositories[0].start, engine::ForgeStart::Branch(branch) if branch.ends_with(format!("/g{}", run.task).as_bytes())),
        "goal workspace: {:?}",
        run.workspace
    );
    let (settings, charter) = inspection_charter(run);
    let digits = engine::commit_text(&translate::commit(head));
    let input =
        format!(r#"{{"head":"{}","path":"docs/design/formats.md"}}"#, core::str::from_utf8(&digits).expect("hex"));
    let (disk, workspace) = super::saved_retry::inspect(world, run);
    assert!(
        disk.load(workspace.directories[0].root.raw(), DESIGN, u64::MAX)
            .expect("checkout design")
            .0
            .windows(marker.len())
            .any(|bytes| bytes == marker)
    );
    let mut agent = smith_agent_world::World::with_workspace_scripts_charter(
        settings,
        None,
        Some(workspace),
        disk,
        design_scripts(input.as_bytes()),
        charter,
    );
    agent.enable_parent_host_calls();
    agent.drive(20_000);
    refuse_forge_answer(world, run, &mut agent);
    agent.run(20_000);
    assert!(matches!(agent.answer(), smith_domain_run::Answer::Accepted { .. }));
    assert_design_reads(&agent, marker);
}

fn refuse_forge_answer(world: &mut World, run: &engine::Assignment, agent: &mut smith_agent_world::World) {
    let pending = agent.pending_host_calls();
    let [call] = pending.as_slice() else { panic!("goal's read_file is relayed: {:?}", agent.answer()) };
    let relay = call.relay;
    let completion = 200 + u32::try_from(world.answers.len()).expect("bounded calls");
    let answer = super::change_tool::named_read(world, run, completion, &call.tool, call.input.bytes());
    let rendered = temper_engine_smith::answer(&answer);
    assert!(!rendered.error(), "{answer:?}");
    assert!(rendered.text().len() > usize::try_from(HOST_REPLY).expect("host cap"));
    agent.return_host_reply(relay, smith_domain_run::HostReply::TooLarge).expect("bounded worker reply");
}

fn assert_design_reads(agent: &smith_agent_world::World, marker: &[u8]) {
    use skein_fake_llm_domain::api::Part;
    let last_line = format!("{:?}", [marker, b"\n"].concat());
    let mut readers = std::collections::BTreeSet::new();
    let mut refused = false;
    for query in agent.prompts() {
        for part in query.messages.iter().flat_map(|message| &message.parts) {
            if let Part::ToolOutput { output, is_error, .. } = part {
                if output.windows(last_line.len()).any(|bytes| bytes == last_line.as_bytes()) && !is_error {
                    readers.insert(query.tools.iter().any(|tool| tool.name.as_ref() == b"read_file"));
                }
                refused |= output
                    .windows(b"host-decided answer-too-large answer-not-shown".len())
                    .any(|bytes| bytes == b"host-decided answer-too-large answer-not-shown");
            }
        }
    }
    assert!(refused, "oversized read_file is refused");
    assert_eq!(readers.len(), 2, "goal and inspect-only survey reach the final marker from g");
    assert_eq!(agent.host_submissions().len(), 1, "surveys have workspace tools alone");
    assert!(agent.pushes().is_empty());
}

#[test]
fn a_goal_and_its_survey_read_designs_landed_on_g() {
    let (mut world, goal, branch) = inspecting_goal();
    let main = branch_tip(&mut world, b"main");
    assert_eq!(branch_tip(&mut world, &branch), main, "g starts at main");
    assert_eq!(forge_hold(&world, &branch).expect("confirmed goal hold").task, goal.task);
    let (goal, head) = land_design(&mut world, &goal, &branch, b"first-design-final-line");
    read_landed_design(&mut world, &goal, head, b"first-design-final-line");
    let (goal, head) = land_design(&mut world, &goal, &branch, b"updated-design-final-line");
    read_landed_design(&mut world, &goal, head, b"updated-design-final-line");
    assert_eq!(branch_tip(&mut world, b"main"), main, "main stays unchanged");
    assert!(!world.fake.object(main).expect("main tree").tree.contains_key(DESIGN));
    world.close();
}
