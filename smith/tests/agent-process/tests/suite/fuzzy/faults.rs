//! Program fault sweeps (`smith/docs/design/faults.md`, 3 and 5).

use skein_world::{
    streams::{Stream, Streams},
    sweep::Sweep,
};
use smith_agent_process_world::{
    charter,
    program_world::{self, Case},
    wire,
};
use smith_fake_claude::{Script, Step, Story, Turn};

fn finish() -> Turn {
    Turn::new(vec![Step::Outcome { input: serde_json::json!({"outcome":{"form":"report","text":"done","fields":{}}}) }])
}

fn charter_bytes() -> Box<[u8]> {
    wire::edit_charter(&charter(), |parts| wire::budget(parts, 8, 20))
}

#[test]
fn provider_faults_fall_on_a_program() {
    let mut sweep = Sweep::new("provider_faults_fall_on_a_program");
    sweep.configure(skein_llm_kit::faults::RATE_WINDOW);
    sweep.configure(skein_llm_kit::faults::EXPIRED_SIGN_IN);
    sweep.configure(smith_faults::EDGE_REFUSAL);
    // Not configured until provider failures fixes a_billing_refusal_marks_the_account,
    // an_unserved_model_moves_the_task (temper's system world), and
    // a_silent_run_is_cut_at_its_derived_bound: billing, unserved model and hang.
    let charter = charter_bytes();
    for seed in [0, 1, 7] {
        eprintln!("provider_faults_fall_on_a_program seed {seed}");
        let streams = Streams::new(seed);
        let mut faults = streams.rng(Stream::Faults, "program");
        let (mut turns, fault) = program_world::faults::provider_draw(streams, &mut faults);
        let turn = turns.len();
        turns.push(fault);
        turns.push(finish());
        let story = Story { cued: vec![Script { cue: String::new(), turns }], ..Story::default() };
        eprintln!("fault turn {turn}");
        let mut run =
            program_world::seeded(streams.seed_of(Stream::Scheduling, "program"), Case::Events, &story, &charter);
        run.runs.judge(&mut run.ledger).unwrap_or_else(|why| panic!("seed {seed}: {why}"));
        eprintln!("fell {:?}", run.ledger.fell());
        sweep.ran(&run.ledger);
    }
    sweep.check_fell().expect("every configured program fault fell");
}

#[test]
#[expect(clippy::single_element_loop, reason = "a sweep keeps its seed loop; one seed covers this single fault")]
fn untooled_programs_fall() {
    let mut sweep = Sweep::new("untooled_programs_fall");
    sweep.configure(smith_faults::UNTOOLED_PROGRAM);
    let charter = charter_bytes();
    for seed in [0] {
        eprintln!("untooled_programs_fall seed {seed}");
        let streams = Streams::new(seed);
        let mut faults = streams.rng(Stream::Faults, "program");
        let case = if faults.chance(500) { Case::MissingServer } else { Case::Events };
        let mut workload = streams.rng(Stream::Workload, "runs");
        let mut turns = program_world::faults::read_turns(&mut workload);
        turns.push(finish());
        let story = Story { turns, ..Story::default() };
        let mut run = program_world::seeded(streams.seed_of(Stream::Scheduling, "program"), case, &story, &charter);
        run.runs.judge(&mut run.ledger).unwrap_or_else(|why| panic!("seed {seed}: {why}"));
        eprintln!("fell {:?}", run.ledger.fell());
        sweep.ran(&run.ledger);
    }
    sweep.check_fell().expect("an untooled program fell");
}
