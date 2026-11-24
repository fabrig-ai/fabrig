//! The QA runner's replaying world (temper/docs/design/testing/qa/contracts.md, 5).
mod build_files;
mod builds;
mod cli;
mod events;
mod gaps;
mod home;
mod installation;
mod place;
mod run;
mod run_passes;

use temper_qa_tool::testing::{case, passes};

const HAPPY_PATH: &str = "QA judges the happy path alone: v0.1.0's stories and each part's bar, used as intended (temper/docs/design/draft/v0.1.0.md, 4). Edge cases are off it: malformed, out-of-range or oversized input, limits and bounds, refusals of bad input and their exit codes, errors, races and recovery.";

fn engine_file() -> Box<[u8]> {
    temper_config_shell::render(
        &temper_config_shell::File {
            workers: Some(([127, 0, 0, 1], 7200).into()),
            ..temper_config_shell::File::default()
        },
        temper_config_shell::Shape::EngineAlone,
    )
}
