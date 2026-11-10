//! Stage-one commands: `temper/docs/design/testing/checks.md`, 5.
use std::process::ExitCode;
use temper_stage_one::{check, collect, golden, judge};

#[expect(
    clippy::disallowed_methods,
    reason = "cli C8: the stage-one tool's arguments, until it is a binary of temper/bin/"
)]
fn main() -> ExitCode {
    skein_fail_stop::install();
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some((command, args)) = args.split_first() else {
        return usage();
    };
    match command.as_str() {
        "obligations" => golden::main(args),
        "collect" => collect::main(args),
        "check" => check::main(args),
        "judge" => judge::main(args),
        "verdict" => judge::verdict_main(args),
        _ => usage(),
    }
}
fn usage() -> ExitCode {
    eprintln!(
        "usage: temper-stage-one obligations [--json] | collect ... | check EVIDENCE [--json] | judge EVIDENCE | verdict EVIDENCE [--json]"
    );
    ExitCode::from(2)
}
