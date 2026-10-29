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

#[test]
fn a_stock_charter_two_repair_starts_with_its_oversized_ci_cut_and_marked() {
    let mut chat = world::chat(b"@report", Job::Reporting);
    chat.run();
    let mut repair = chat.assignment;
    let llm = jig_protocol_config::Llm {
        endpoints: Box::default(),
        providers: Box::default(),
        descriptions: Box::new([None]),
        models: Box::new([world::charter().policy.model]),
    };
    let stock = temper_service::stock::defaults(&llm);
    let coder = stock.charters.iter().find(|charter| charter.number == 2).expect("stock coder");
    repair.charter = coder.number;
    repair.run.policy = coder.policy.clone();
    repair.run.ended_by = tasks::EndedBy::Result;
    repair.run.contract = tasks::Contract::Change { connector: 0, kind: 1, words: 1024 };
    repair.run.authority.tools.0 &= !jig_domain_core::ToolFamilies::JIG.message.0;
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
    settings.limits.run.host_tools = 40;
    settings.limits.run.host_part = 33_419;
    settings.limits.run.schema_costs = smith_protocol_llm::schema_costs().expect("production renderer costs");
    let uncut = temper_engine_smith::charter(&repair);
    let room = run::brief_room(&uncut, Some(&workspace), &settings.limits.run).expect("declarations fit");
    assert!(run::brief_bytes(&uncut.brief).expect("uncut brief bytes") > u64::from(room));
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
    let smith::Event::Start { charter, .. } = &start else { panic!("smith start") };
    assert_eq!(charter.brief.sections[0].text, uncut.brief.sections[0].text);
    assert_eq!(charter.brief.sections[1].title.as_ref(), b"CI");
    assert!(charter.brief.sections[1].text.ends_with(b" bytes; whole: read_job/read_checks]\n"));
    assert!(run::brief_bytes(&charter.brief).expect("fitted brief bytes") <= u64::from(room));
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
