//! Program budgets and stop order through the shipped composition (smith/docs/design/protocol/programs.md 5.7).
use serde_json::json;
use skein_lib::Duration;
use smith_agent_process_world::{
    charter, program,
    program_world::{self, Case, Run},
};
use smith_channel::{RunFailure, RunResult};
use smith_fake_claude::{Decision, Seen, Step, Story, Turn};

fn selected(change: impl FnOnce(&mut smith_charter::Charter)) -> Box<[u8]> {
    smith_agent_process_world::wire::edit_charter(&charter(), |parts| {
        smith_agent_process_world::wire::budget(parts, 8, 20);
        change(parts);
    })
}

fn run(case: Case, charter: &[u8], turns: Vec<Turn>) -> Run {
    program_world::run(
        case,
        &Story { turns, ..Story::default() },
        Some(b"claude.ai"),
        charter,
        &program(b"source"),
        None,
    )
}

fn turn(steps: Vec<Step>) -> Turn {
    Turn::new(steps)
}

fn finish() -> Step {
    Step::Outcome { input: json!({"outcome":{"form":"report","text":"done","fields":{}}}) }
}

fn winding(parts: &mut smith_charter::Charter) {
    let mut budget = parts.budget.clone();
    budget.turns = 3;
    budget.reserve = smith_charter::Reserve { turns: 2, time: Duration::from_secs(1) };
    parts.budget = budget;
}

fn budget_failure(run: &Run) {
    assert!(
        matches!( &run.answer().expect("answer").result, RunResult::Failed(failed) if matches!( &failed.reason, RunFailure::Budget(_))),
        "answer={:?} stderr={}",
        run.answer(),
        String::from_utf8_lossy(&run.errors)
    );
}

#[test]
fn a_cancel_interrupts_the_turn() {
    let run = run(Case::Ordinary, &selected(|_| {}), vec![turn(vec![Step::Hang])]);
    assert!(
        run.seen.contains(&Seen::Interrupted),
        "seen={:?} calls={:?} answer={:?}",
        run.seen,
        run.invocations,
        run.answer()
    );
    assert!(
        matches!( &run.answer().expect("bounded test fixture").result, RunResult::Failed(failed) if matches!( &failed.reason, RunFailure::Cancelled))
    );
    assert!(!run.seen.contains(&Seen::Terminated));
    assert!(run.exits.contains(&skein_io::kernel::Exit::Code(0)));
    assert_eq!(run.run_directories.len(), 0);
}

#[test]
fn a_program_deaf_to_its_interrupt_is_terminated() {
    let run = run(Case::Ordinary, &selected(|_| {}), vec![turn(vec![Step::Deaf])]);
    assert!(
        run.seen.contains(&Seen::Interrupted),
        "seen={:?} calls={:?} answer={:?}",
        run.seen,
        run.invocations,
        run.answer()
    );
    assert!(run.seen.contains(&Seen::Terminated));
    assert!(run.exits.contains(&skein_io::kernel::Exit::Code(143)));
    assert_eq!(run.run_directories.len(), 0);
}

#[test]
fn the_wind_down_sends_the_note() {
    let run = run(
        Case::Turns,
        &selected(winding),
        vec![turn(vec![Step::Call { tool: "unoffered".into(), input: json!({}) }, Step::Hang]), turn(vec![finish()])],
    );
    assert!(
        matches!(&run.answer().expect("bounded test fixture").result, RunResult::Accepted(_)),
        "{:?}",
        run.answer()
    );
    // The wind-down interrupts the unfinished turn; the final outcome already
    // has its result, so acceptance closes stdin without another interrupt.
    assert_eq!(run.seen.iter().filter(|seen| **seen == Seen::Interrupted).count(), 1);
    let note = "The ordinary budget is spent. Finish now";
    assert!(
        run.received.iter().any(|message| {
            assert_eq!(message["type"], "user");
            assert_eq!(message["message"]["role"], "user");
            message["message"]["content"].as_str().is_some_and(|text| text.contains(note))
        }),
        "received={:?}",
        run.received
    );
}

#[test]
fn wall_time_stops_the_program() {
    let charter = selected(|parts| {
        let mut budget = parts.budget.clone();
        budget.time = Duration::from_secs(1);
        parts.budget = budget;
    });
    let run = run(Case::Turns, &charter, vec![turn(vec![Step::Hang])]);
    budget_failure(&run);
    assert!(
        run.seen.contains(&Seen::Interrupted),
        "seen={:?} calls={:?} answer={:?}",
        run.seen,
        run.invocations,
        run.answer()
    );
}

#[test]
fn a_wind_down_denies_a_built_in() {
    let charter = selected(|parts| {
        winding(parts);
        let mut tools = parts.tools.clone();
        tools.families = smith_charter::Families { inspect: true, modify: false, shell: false, agents: false };
        parts.tools = tools;
    });
    let run = run(
        Case::Tools,
        &charter,
        vec![
            turn(vec![Step::Call { tool: "unoffered".into(), input: json!({}) }, Step::Hang]),
            turn(vec![
                Step::Builtin {
                    tool: "Read".into(),
                    input: json!({"file_path":"/work/repo/file"}),
                    result: "not reached".into(),
                },
                finish(),
            ]),
        ],
    );
    assert!(
        matches!(&run.answer().expect("bounded test fixture").result, RunResult::Accepted(_)),
        "{:?}",
        run.answer()
    );
    assert!(run.seen.iter().any(|seen| matches!(seen, Seen::Prompted { decision: Decision::Deny(message), .. } if message == "not run: allowed calls now: finish ")));
}

#[test]
fn spend_past_the_budget_fails_the_run() {
    let charter = selected(|parts| {
        let mut budget = parts.budget.clone();
        budget.spend = 2;
        parts.budget = budget;
        let mut llm = parts.main.clone();
        let mut prices = llm.prices;
        prices.unit = 1_000_000;
        llm.prices = prices;
        parts.main = llm;
    });
    let run = program_world::run(
        Case::Turns,
        &Story {
            turns: vec![turn(vec![Step::Call { tool: "unoffered".into(), input: json!({}) }, Step::Hang])],
            usage: smith_fake_claude::Usage { input_tokens: 2_000_001, output_tokens: 5 },
            ..Story::default()
        },
        Some(b"claude.ai"),
        &charter,
        &program(b"source"),
        None,
    );
    budget_failure(&run);
    let answer = run.answer().expect("bounded test fixture");
    assert_eq!(answer.spent, 3);
    assert!(
        run.seen.contains(&Seen::Interrupted),
        "seen={:?} calls={:?} answer={:?}",
        run.seen,
        run.invocations,
        run.answer()
    );
}
