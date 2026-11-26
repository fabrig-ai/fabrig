//! A goal's design delivery (temper/docs/design/domain/agent/README.md, 5 and 7;
//! domain/forge/branches.md, 3) uses the repository's checks (domain/worker/checkouts.md).
use super::{Key, World, call, engine, tasks};
use skein_fake_forge_domain::Observation;
use skein_fake_llm_domain::api::{Part, Script};
use temper_engine_domain::Record;
use temper_fake_checkout::git::Remote;

const README: &[u8] = b"# app\n";
const EDITED: &[u8] = b"# app\nGoal design\n";
const REPORT: &str = "The design landed in the goal branch.";

fn config(seed: u64) -> engine::Config {
    let mut config = super::agreement::config(seed);
    config.core.defaults.charters[0].policy.modify = true;
    config.core.defaults.templates[0].flags.ended_by = tasks::EndedBy::Result;
    config
}

fn delivery(passes: bool) -> World {
    let turns = vec![
        call(b"read", br#"{"path":"README.md"}"#),
        call(b"edit", br##"{"path":"README.md","edits":[{"old":"# app\n","new":"# app\nGoal design\n"}]}"##),
        call(b"deliver", br#"{"fields":{"title":"Design","body":"Write the goal design."}}"#),
        call(b"finish", &serde_json::to_vec(&serde_json::json!({"report":REPORT})).expect("report JSON")),
    ];
    let script = Script { cue: b"Help the person with their work.".as_slice().into(), turns: turns.into() };
    let mut world = World::configured_with(925, script, true, config);
    world.goal_branch = true;
    world.forge.advance(b"org/repo", b"main", b"README.md", README);
    world.forge.advance(b"org/repo", b"main", b".temper/pre-pr", b"#!checks\nsrc/lib.rs 43\n");
    if passes {
        world.forge.advance(b"org/repo", b"main", b"src/lib.rs", b"pub fn answer() -> u32 { 43 }\n");
    }
    world
}

#[test]
fn a_design_delivery_runs_the_repository_pre_pr_before_pushing() {
    let mut world = delivery(true);
    let main = world.forge.branch(b"org/repo", b"main").expect("landing branch before the goal runs");
    assert_eq!(world.forge.tree(main).get(b"README.md".as_slice()), Some(&README.to_vec()));
    world.run();

    let task = world.assignment().task;
    let Some(Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Ended(ended)))) =
        world.store.rows.get(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Ended(task))))
    else {
        panic!("the goal ended");
    };
    assert_eq!(
        ended.phase,
        tasks::Phase::Ended(tasks::Ending::Done(tasks::TaskResult::Report { words: REPORT.as_bytes().into() }))
    );
    let goal_branch = format!("temper/{}/g{task}", ended.project).into_bytes();
    let run_prefix = format!("temper/{}/r", ended.project).into_bytes();
    // The forge keeps every drained write, including a branch created and later deleted.
    assert!(
        !world.forge.observations().iter().any(|observation| matches!(
            observation, Observation::Moved { repository, branch, .. }
                if repository.as_ref() == b"org/repo" && branch.starts_with(&run_prefix)
        )),
        "no run branch was ever written"
    );
    let repository = &world.assignment().workspace.repositories[0];
    assert_eq!(repository.push, repository.landing, "the assignment delivers design straight to g");
    assert_eq!(repository.push.as_deref(), Some(goal_branch.as_slice()));
    assert_eq!(world.agent().checked(), [true], "the repository's check ran");
    assert_eq!(world.agent().delivery_submissions().len(), 1);
    let head = world.forge.branch(b"org/repo", repository.push.as_ref().expect("g")).expect("delivered branch");
    let [(run, attempt, number, ask)] = world.observed.push_asks.as_slice() else {
        panic!("one design delivery admission: {:?}", world.observed.push_asks);
    };
    assert_eq!(run.raw(), task);
    assert_eq!(attempt.raw(), world.assignment().attempt);
    assert_eq!(ask.tag, repository.tag);
    assert_eq!(ask.branch.as_ref(), goal_branch);
    assert_eq!(ask.expected, Some(temper_worker_checkout_world::translate::commit(main).raw()));
    assert_eq!(ask.commit, temper_worker_checkout_world::translate::commit(head).raw());
    assert_eq!(world.observed.push_answers.as_slice(), [(*run, *attempt, *number)]);
    assert_eq!(world.forge.tree(head).get(b"README.md".as_slice()), Some(&EDITED.to_vec()));
    assert_eq!(world.forge.branch(b"org/repo", b"main"), Some(main), "the landing branch did not move");
    assert_eq!(world.forge.tree(main).get(b"README.md".as_slice()), Some(&README.to_vec()));

    let mut delivered = false;
    for prompt in world.agent().prompts() {
        let parts = || prompt.messages.iter().flat_map(|message| &message.parts);
        for part in parts() {
            if let Part::ToolOutput { id, output, is_error } = part {
                assert!(
                    !is_error && !output.starts_with(b"Error:"),
                    "tool refused: {}",
                    String::from_utf8_lossy(output)
                );
                if parts().any(|part| {
                    matches!(part, Part::ToolCall { id: called, name, .. }
                    if called == id && name.as_ref() == b"deliver")
                }) {
                    assert!(
                        output.starts_with(b"delivered\nreceipt directory="),
                        "delivery answer: {}",
                        String::from_utf8_lossy(output)
                    );
                    assert!(output.windows(goal_branch.len()).any(|bytes| bytes == goal_branch), "the receipt names g");
                    delivered = true;
                }
            }
        }
    }
    assert!(delivered, "the model received its goal branch's delivery receipt");
}

#[test]
fn a_red_repository_pre_pr_refuses_a_design_delivery() {
    let mut world = delivery(false);
    world.run();
    assert_eq!(world.agent().checked(), [false], "the repository's red check ran");
    assert!(world.agent().delivery_submissions().is_empty(), "a red pre-pr prevents the push");
    let repository = &world.assignment().workspace.repositories[0];
    assert_eq!(repository.push, repository.landing, "the assignment delivers design straight to g");
    let head = world.forge.branch(b"org/repo", repository.push.as_ref().expect("g")).expect("goal branch");
    assert_eq!(world.forge.tree(head).get(b"README.md".as_slice()), Some(&README.to_vec()));
    assert!(world.agent().prompts().iter().flat_map(|prompt| &prompt.messages).flat_map(|message| &message.parts).any(|part| {
        matches!(part, Part::ToolOutput { output, is_error: true, .. }
            if output.starts_with(b"checks-failed ") && String::from_utf8_lossy(output).contains("check=\".temper/pre-pr\""))
    }), "the model hears the repository pre-pr failure");
}
