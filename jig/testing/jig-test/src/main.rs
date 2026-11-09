//! The reference service and its agent command (protocol/service.md, 7).
use std::{env, process::ExitCode};

#[expect(clippy::disallowed_methods, reason = "jig's round: reads the reference service and agent command arguments")]
fn main() -> ExitCode {
    skein_fail_stop::install();
    if env::args_os().nth(1).is_some_and(|argument| argument == smith_agent_shell::relay::FLAG) {
        return smith_agent_shell::relay::main();
    }
    if env::args_os().nth(1).is_some_and(|argument| argument == smith_agent_shell::PROGRAM_CHILD_FLAG) {
        return smith_agent_shell::program_child::main();
    }
    match jig_test::configure(env::args_os().skip(1)).and_then(jig_test::run) {
        Ok(code) => ExitCode::from(code),
        Err(why) => {
            eprintln!("jig-test: {why}");
            ExitCode::FAILURE
        }
    }
}
