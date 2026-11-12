//! Goal deliveries use the repository's checks (temper/docs/design/domain/worker/checkouts.md).
use super::{World, call, engine, report_script, tasks};
use skein_fake_llm_domain::api::{Part, Script};
use temper_fake_checkout::git::Remote;

fn config(seed: u64) -> engine::Config {
    let mut config = super::agreement::config(seed);
    config.core.defaults.charters[0].policy.modify = true;
    config.core.defaults.templates[0].flags.ended_by = tasks::EndedBy::Result;
    config
}

fn delivery(passes: bool) -> World {
    let mut turns = vec![
        call(b"read", br#"{"path":"README.md"}"#),
        call(b"write", br#"{"path":"README.md","content":"Goal design\n"}"#),
        call(b"deliver", br#"{"fields":{"title":"Design","body":"Write the goal design."}}"#),
    ];
    turns.extend(report_script(false).turns);
    let script = Script { cue: b"Help the person with their work.".as_slice().into(), turns: turns.into() };
    let mut world = World::configured_with(925, script, true, config);
    world.goal_branch = true;
    world.forge.advance(b"org/repo", b"main", b".temper/pre-pr", b"#!checks\nsrc/lib.rs 43\n");
    if passes {
        world.forge.advance(b"org/repo", b"main", b"src/lib.rs", b"pub fn answer() -> u32 { 43 }\n");
    }
    world.run();
    world
}

#[test]
fn a_design_delivery_runs_the_repository_pre_pr_before_pushing() {
    let world = delivery(true);
    let repository = &world.assignment().workspace.repositories[0];
    assert_eq!(repository.push, repository.landing, "the assignment delivers design straight to g");
    assert!(repository.push.is_some());
    assert_eq!(world.agent().checked(), [true], "the repository's check ran");
    assert_eq!(world.agent().delivery_submissions().len(), 1);
    let head = world.forge.branch(b"org/repo", repository.push.as_ref().expect("g")).expect("delivered branch");
    assert_eq!(world.forge.tree(head).get(b"README.md".as_slice()), Some(&b"Goal design\n".to_vec()));
}

#[test]
fn a_red_repository_pre_pr_refuses_a_design_delivery() {
    let world = delivery(false);
    assert_eq!(world.agent().checked(), [false], "the repository's red check ran");
    assert!(world.agent().delivery_submissions().is_empty(), "a red pre-pr prevents the push");
    let repository = &world.assignment().workspace.repositories[0];
    assert_eq!(repository.push, repository.landing, "the assignment delivers design straight to g");
    let head = world.forge.branch(b"org/repo", repository.push.as_ref().expect("g")).expect("goal branch");
    assert_eq!(world.forge.tree(head).get(b"README.md".as_slice()), Some(&b"The answer is 42.\n".to_vec()));
    assert!(world.agent().prompts().iter().flat_map(|prompt| &prompt.messages).flat_map(|message| &message.parts).any(|part| {
        matches!(part, Part::ToolOutput { output, is_error: true, .. }
            if output.starts_with(b"checks-failed ") && String::from_utf8_lossy(output).contains("check=\".temper/pre-pr\""))
    }), "the model hears the repository pre-pr failure");
}
