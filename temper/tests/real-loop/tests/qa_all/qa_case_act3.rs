//! Adversarial real-loop runs of the retired first edition,
//! retained in domain/testing.md, 5.3; draft/reference-case.md is the second edition.
use serde_json::Value;
use skein_fake_llm_domain::api::{Line, Script};
use std::net::SocketAddr;
use temper_real_loop_world::{
    case::{
        self, Agent, acts,
        oracle::{self, Observed},
    },
    run_cluster_forge,
};

const WORKERS: &[&str] = &["ws", "s1", "s2"];
// Each selected script call is a single structured mutation.
const ONE_EDIT: usize = 1;

fn edit_call(
    scripts: &mut [Script],
    agent: Agent,
    tool: &[u8],
    select: impl Fn(&Value) -> bool,
    edit: impl Fn(&mut Value),
) {
    let script = scripts.iter_mut().find(|s| s.cue.ends_with(agent.cue())).expect("case agent");
    let mut edited = 0;
    for turn in &mut script.turns {
        for line in &mut turn.lines {
            if let Line::Call { name, arguments } = line
                && name.as_ref() == tool
            {
                let mut call: Value = serde_json::from_slice(arguments).expect("tool JSON");
                if select(&call) {
                    println!("editing {}: {call}", String::from_utf8_lossy(tool));
                    let original = call.clone();
                    edit(&mut call);
                    assert_ne!(call, original, "the selected call must actually change");
                    *arguments = serde_json::to_vec(&call).expect("edited tool JSON").into();
                    edited += 1;
                }
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
fn batch(scripts: fn(SocketAddr) -> Box<[Script]>) -> Observed {
    run(case::fixture_6_1(), scripts, case::scripts::step_6_1_rules, acts::step_6_1, acts::READS_6_1)
}

fn task(document: &Value) -> &Value {
    document.get("task").filter(|value| value.is_object()).unwrap_or(document)
}

#[test]
fn reversing_dependency_order_preserves_the_same_waits() {
    let observed = batch(|forge| {
        let mut scripts = case::scripts::step_6_1(forge);
        edit_call(
            &mut scripts,
            Agent::BatchSubowner,
            b"delegate",
            |_| true,
            |call| {
                let entries = call["batch"].as_array_mut().expect("batch");
                let story = entries.last_mut().expect("story delegate");
                story["dependencies"].as_array_mut().expect("story dependencies").reverse();
            },
        );
        scripts
    });
    // A batch's dependencies name its prerequisites; their order grants no
    // permission to start after only one has ended (jig's domain/tasks.md, 4 and 7).
    for read in [acts::Read::BatchStoryWaiting, acts::Read::BatchStoryAfterOne] {
        let story = task(&observed.documents[&read]);
        println!("{read:?}: dependencies {}, phase {}", story["dependency_facts"], story["chip"]["phase"]);
        assert_eq!(story["chip"]["phase"], "waiting");
        assert!(story["chip"]["details"]["owner"]["details"]["attempt"].is_null());
    }
    let story = task(&observed.documents[&acts::Read::BatchStoryEnded]);
    let dependencies: std::collections::BTreeSet<_> = story["dependency_facts"]
        .as_array()
        .expect("dependencies")
        .iter()
        .map(|dependency| dependency["number"].as_u64().expect("dependency number"))
        .collect();
    assert_eq!(dependencies, [case::BATCH_ONE, case::BATCH_TWO].into());
    assert_eq!(
        task(&observed.documents[&acts::Read::BatchThreeStillWriting])["chip"]["details"]["owner"]["details"]["observation"]
            ["work"],
        "writing"
    );
    oracle::assert_met(&oracle::step_6_1(&observed));
}
