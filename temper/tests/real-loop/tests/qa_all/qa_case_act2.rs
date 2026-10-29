//! Adversarial real-loop runs of the retired first edition,
//! retained in domain/testing.md, 5.3; draft/reference-case.md is the second edition.
use serde_json::Value;
use skein_fake_llm_domain::api::{Line, Script};
use std::net::SocketAddr;
use temper_real_loop_world::{
    case::{
        self, Agent, acts,
        oracle::{self, Miss, Observed, Promise},
    },
    run_cluster_forge,
};

// Each mutation targets one recorded tool call, not serialized script text.
const ONE_EDIT: usize = 1;
const WORKERS: &[&str] = &["ws", "s1", "s2"];

fn edit_call(scripts: &mut [Script], agent: Agent, tool: &[u8], edit: impl Fn(&mut Value)) {
    let script = scripts.iter_mut().find(|s| s.cue.as_ref() == agent.cue()).expect("case agent");
    let mut edited = 0;
    for turn in &mut script.turns {
        for line in &mut turn.lines {
            if let Line::Call { name, arguments } = line
                && name.as_ref() == tool
            {
                let mut call: Value = serde_json::from_slice(arguments).expect("tool JSON");
                println!("editing {}: {call}", String::from_utf8_lossy(tool));
                let original = call.clone();
                edit(&mut call);
                assert_ne!(call, original, "the selected call must actually change");
                *arguments = serde_json::to_vec(&call).expect("edited tool JSON").into();
                edited += 1;
            }
        }
    }
    assert_eq!(edited, ONE_EDIT, "one intended call changed");
}

fn run(
    fixture: temper_real_loop_world::forge::Fixture,
    scripts: fn(SocketAddr) -> Box<[Script]>,
    rules: fn() -> Box<[Box<[skein_fake_llm_domain::api::Rule]>]>,
    steps: fn() -> Vec<temper_real_loop_world::calls::Step>,
    reads: &[(acts::Read, usize)],
) -> Observed {
    let ran = run_cluster_forge(WORKERS, &[], fixture, scripts, rules, steps);
    Observed::from_ran(&ran, reads).with_commands(&steps())
}

fn direction(scripts: fn(SocketAddr) -> Box<[Script]>) -> Observed {
    run(case::fixture_5_4(), scripts, case::scripts::step_5_4_rules, acts::step_5_4, acts::READS_5_4)
}

fn misread(scripts: fn(SocketAddr) -> Box<[Script]>) -> Observed {
    run(case::fixture_5_6(), scripts, case::scripts::step_5_6_rules, acts::step_5_6, acts::READS_5_6)
}

fn only_miss(misses: &[Miss], step: &'static str, promise: Promise) {
    for miss in misses {
        println!("{miss}");
    }
    assert_eq!(misses.iter().map(|miss| (miss.step, miss.promise)).collect::<Vec<_>>(), [(step, promise)]);
}

#[test]
fn missing_deferred_work_is_reported_after_real_cancellation() {
    let observed = direction(|forge| {
        let mut scripts = case::scripts::step_5_4(forge);
        edit_call(&mut scripts, Agent::StoryOwner, b"note", |call| {
            let words = call.get_mut("body").expect("note body");
            *words = "The deferred plan has no task or pull reference.".into();
        });
        scripts
    });
    let misses = oracle::step_5_4(&observed);
    only_miss(&misses, "5.4", Promise::Must("cancel, with work kept under its closed pull"));
}

#[test]
fn a_live_timer_does_not_prevent_fresh_cancellation() {
    // 5.6 Add: timers are the task's, outside its run.
    oracle::assert_met(&oracle::step_5_6(&misread(case::scripts::step_5_6)));
}
