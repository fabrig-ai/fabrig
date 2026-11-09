//! The deployment's agent entry point (temper/docs/design/protocol/README.md, 4.3).
#![forbid(unsafe_code)]

use std::{env, ffi::OsStr, process::ExitCode};
use temper_agent::NAME;

#[expect(
    clippy::disallowed_methods,
    reason = "cli C6: arguments select launcher, relay and agent modes; temper-agent's table will read them"
)]
fn main() -> ExitCode {
    skein_fail_stop::install();
    if let Some(program) = env::args_os().next()
        && temper_launcher::is_launcher(&program)
    {
        return ExitCode::from(temper_launcher::run(NAME, &program, &skein_shell::arguments()));
    }
    if env::args_os().nth(1).as_deref() == Some(OsStr::new(smith_agent_shell::relay::FLAG)) {
        return smith_agent_shell::relay::main();
    }
    if env::args_os().nth(1).as_deref() == Some(OsStr::new(smith_agent_shell::PROGRAM_CHILD_FLAG)) {
        return smith_agent_shell::program_child::main();
    }
    match temper_agent::configure(&skein_shell::arguments()).and_then(temper_agent::run) {
        Ok(code) => ExitCode::from(code),
        Err(why) => {
            eprint!("{why}");
            ExitCode::FAILURE
        }
    }
}
