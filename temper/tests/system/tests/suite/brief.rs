//! A stock repair's oversized CI brief reaches smith within the small host bound.
use jig_domain_brief::Core;
use jig_domain_tasks as tasks;
use skein_fake_checkout::Checkout;
use skein_fake_llm_domain::api::{Finish, Line, Script, Turn};
use skein_lib::{ReplyTo, Token};
use smith_agent_world::{Job, Settings, World as Agent};
use smith_domain as smith;
use smith_domain_run as run;
use temper_engine_domain as engine;
use temper_system_world::world;

// The task and a marked CI prefix fit; the repeated CI body must be cut.
const BRIEF_ROOM: u32 = 1_300;

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one stock repair follows its oversized CI brief through smith to its declared change"
)]
fn a_stock_charter_two_repair_starts_with_its_oversized_ci_cut_and_marked() {
    let mut chat = world::chat(b"@report", Job::Reporting);
    chat.run();
    let mut repair = chat.assignment;
    let llm = jig_protocol_config::Llm {
        endpoints: Box::default(),
        providers: Box::default(),

        models: (Box::new([world::charter().policy.model]))
            .into_iter()
            .enumerate()
            .map(|(at, model)| jig_domain_core::settings::CatalogueModel {
                first_read: None,
                number: u32::try_from(at).expect("bounded fixture models") + 1,
                model,
                shown: None,
                description: None,
                picker: true,
                withdrawn: false,
                source: jig_domain_core::settings::ModelSource::Stock,
                estimated: false,
            })
            .collect(),
    };
    let stock =
        temper_engine_service_stock::stock::with_models(llm.models.clone(), chat.root.env.limits.tasks.result_bytes);
    let coder = stock.charters.iter().find(|charter| charter.number == 2).expect("stock coder");
    repair.charter = coder.number;
    repair.run.policy = coder.policy.clone();
    repair.run.ended_by = tasks::EndedBy::Result;
    repair.run.contract = tasks::Contract::Change { connector: 0, kind: 1, words: 1024 };
    let families = jig_domain_core::ToolFamilies::JIG;
    repair.run.authority.tools.0 &= !(families.message.0 | families.control.0);
    repair.run.budget = coder.spend;
    repair.sections = Box::new([
        engine::BriefSection {
            kind: engine::BriefKind::Core(Core::Task),
            body: engine::BriefBody::Text(b"@trim-repair Repair the failed CI".as_slice().into()),
        },
        engine::BriefSection {
            kind: engine::BriefKind::Forge(engine::ForgeBriefKind::Ci),
            body: engine::BriefBody::Text(b"failed CI: the log is long\n".repeat(200).into_boxed_slice()),
        },
    ]);
    let mut disk = Checkout::new();
    disk.mkdir(b"work");
    disk.write(b"work/src/lib.rs", b"pub fn answer() -> u32 { 42 }\n");
    let workspace = run::Workspace {
        directories: Box::new([run::Directory {
            name: b"work".as_slice().into(),
            host: b"/work".as_slice().into(),
            root: Token::new(disk.root(b"work")),
            writable: true,
            git: true,
            conflicts: Box::default(),
        }]),
    };
    let mut settings = Settings::calm(903);
    settings.job = Job::Coding;
    settings.budget = run::Budget { turns: coder.policy.turns, spend: coder.spend, time: coder.policy.time };
    settings.limits.run.budget = settings.budget;
    settings.limits.session.spend = coder.spend;
    settings.limits.session.budget.turns = coder.policy.turns;
    settings.limits.session.budget.time = coder.policy.time;
    settings.limits.run.host_tools = 40;
    settings.limits.run.schema_costs = smith_protocol_llm::schema_costs().expect("production renderer costs");
    let uncut = temper_engine_smith::charter(&repair);
    let non_brief = settings
        .limits
        .run
        .host_part
        .checked_sub(u64::from(
            run::brief_room(&uncut, Some(&workspace), &settings.limits.run).expect("declarations fit"),
        ))
        .expect("declarations within host bound");
    settings.limits.run.host_part = non_brief.checked_add(u64::from(BRIEF_ROOM)).expect("host bound with brief room");
    assert_eq!(run::brief_room(&uncut, Some(&workspace), &settings.limits.run), Some(BRIEF_ROOM));
    assert!(run::brief_bytes(&uncut.brief).expect("uncut brief bytes") > u64::from(BRIEF_ROOM));
    let start = temper_engine_smith::start(
        &repair,
        Some(workspace),
        None,
        ReplyTo::new(Token::new(402)),
        Token::new(repair.task),
        smith::Window {
            turns: 16,
            bytes: smith::max_turn_bytes(&settings.limits)
                .expect("largest turn")
                .checked_mul(16)
                .expect("activation window"),
        },
        &settings.limits.run,
    );
    let smith::Event::Start(smith::Start { charter, .. }) = &start else { panic!("smith start") };
    assert_eq!(charter.brief.sections[0].text, uncut.brief.sections[0].text);
    assert_eq!(charter.brief.sections[1].title.as_ref(), b"CI");
    assert!(charter.brief.sections[1].text.ends_with(b" bytes; whole: read_job/read_checks]\n"));
    assert!(run::brief_bytes(&charter.brief).expect("fitted brief bytes") <= u64::from(BRIEF_ROOM));
    let call = |name: &[u8], arguments: &[u8]| Turn {
        lines: Box::new([Line::Call { name: name.into(), arguments: arguments.into() }]),
        finish: Finish::ToolCalls,
        tokens: 20,
    };
    let script = Script {
        cue: b"@trim-repair".as_slice().into(),
        turns: Box::new([
            call(b"edit", br#"{"path":"src/lib.rs","edits":[{"old":"42","new":"43"}]}"#),
            call(b"finish", br#"{"title":"Repair failed CI","body":"Corrected the answer."}"#),
        ]),
    };
    let mut agent = Agent::with_workspace_scripts_start(settings, disk, Box::new([script]), start);
    agent.run(1000);
    assert!(
        matches!(agent.answer(), run::Answer::Accepted { outcome: run::outcome::Declared::Change(_), .. }),
        "{:?}",
        agent.answer()
    );
    assert!(!agent.prompts().is_empty(), "the repaired brief reaches the model");
    assert_eq!(agent.code(), b"pub fn answer() -> u32 { 43 }\n");
}

#[test]
fn a_large_pull_review_reaches_smith_and_lands() {
    for (seed, bytes) in [(931, 47_148), (932, 41_473), (933, 128)] {
        large_pull_review(seed, bytes);
    }
}

fn large_pull_review(seed: u64, bytes: usize) {
    use temper_system_world::composed::{World, call};
    let review = Script {
        cue: b"Review the change".as_slice().into(),
        turns: Box::new([
            call(b"read_pull", br#"{"repository":"org/repo","number":1}"#),
            call(b"read_file", br#"{"repository":"org/repo","head":"$HEAD","path":"export.csv","max_bytes":512}"#),
            call(b"finish", br#"{"verdict":"1","body":"The change is correct at the reviewed head."}"#),
        ]),
    };
    let mut world = World::large_review_change(seed, bytes, review);
    world.until_review_or_terminal(3);
    let task = world.assignment().task;
    assert_eq!(world.assignment().charter, 4, "{:?}; {:?}", world.reviewed_change(), world.agent().answer());
    assert_eq!(world.pull_diff_bytes(), bytes);
    let uncut = temper_engine_smith::charter(world.assignment());
    eprintln!("pull_diff_bytes={bytes} gathered_charter_owned={:?}", uncut.owned_bytes());
    let head = world.reviewed_change().change.last_head.expect("review head");
    world.until_review_or_terminal(task);
    let agent = world.review_agent(task);
    assert!(!agent.prompts().is_empty(), "large review must reach a model turn: {:?}", agent.answer());
    assert!(matches!(agent.answer(), run::Answer::Accepted { .. }), "{:?}", agent.answer());
    assert_forge_reads(agent);
    let fitted = world.review_charter(task);
    assert_eq!(fitted.instructions, uncut.instructions);
    assert_eq!(fitted.grants, uncut.grants);
    assert_eq!(fitted.outcome, uncut.outcome);
    let pull =
        fitted.brief.sections.iter().find(|section| section.title.as_ref() == b"Pull request").expect("pull retained");
    if bytes > 128 {
        assert!(pull.text.ends_with(b" bytes; whole: read_pull/read_pull_files]\n"));
    } else {
        assert_eq!(fitted, uncut, "an already fitting brief is unchanged");
    }
    assert!(
        matches!(world.reviewed_change().change.state, temper_engine_domain_forge_change::State::Landed { .. }),
        "{:?}",
        world.reviewed_change()
    );
    assert_eq!(world.reviewed_change().change.last_head, Some(head));
    assert_eq!(world.reviewed_change().change.repairs, 0);
    assert_eq!(world.reviewed_change().verdicts.len(), 1, "one approval without invalid-result retries");
    let verdict = &world.reviewed_change().verdicts[0];
    assert_eq!(verdict.head, head);
    assert_eq!(verdict.status, temper_engine_domain_forge_change::Status::Passed);
    assert!(world.reviewed_change().change.gate_failures.is_empty());
    assert_eq!(world.observed.spawns, 3, "chat, producer and one admitted reviewer");
}

fn assert_forge_reads(agent: &smith_agent_world::World) {
    use smith_domain_session::llm::{Block, Returned};
    let results = agent
        .turns()
        .iter()
        .flat_map(|turn| &turn.messages)
        .flat_map(|message| &message.content)
        .filter_map(|block| {
            if let Block::ToolResult { result: Returned::Text { error, .. }, .. } = block { Some(*error) } else { None }
        })
        .collect::<Vec<_>>();
    assert_eq!(results, [false, false, false], "omitted content is read through the forge before approval");
}
