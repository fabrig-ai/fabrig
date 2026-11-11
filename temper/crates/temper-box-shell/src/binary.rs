//! Temper's standalone commands (protocol/binaries.md).
use std::os::unix::ffi::OsStrExt;
use std::{env, process::ExitCode};

#[expect(
    clippy::disallowed_methods,
    reason = "cli C2: arguments select entry modes and the environment resolves commands; temper-box's table will read them"
)]
pub fn main() -> ExitCode {
    if env::args_os().next().as_deref().is_some_and(crate::fake::git::is_git) {
        return ExitCode::from(crate::fake::git::run(&skein_shell::arguments()));
    }
    if let Some(program) = env::args_os().next()
        && crate::claude_code::is_launcher(&program)
    {
        return ExitCode::from(crate::claude_code::run(&program, &skein_shell::arguments()));
    }
    if env::args_os().nth(1).as_deref() == Some(std::ffi::OsStr::new(smith_agent_shell::relay::FLAG)) {
        return smith_agent_shell::relay::main();
    }
    if env::args_os().nth(1).as_deref() == Some(std::ffi::OsStr::new(smith_agent_shell::PROGRAM_CHILD_FLAG)) {
        return smith_agent_shell::program_child::main();
    }
    let arguments = skein_shell::arguments();
    if arguments.as_ref() == [Box::from(b"--help".as_slice())] {
        print!("{}", crate::commands::help());
        return ExitCode::SUCCESS;
    }
    let working = match env::current_dir() {
        Ok(path) => path,
        Err(why) => {
            eprintln!("temper: {why}");
            return ExitCode::FAILURE;
        }
    };
    match crate::configure_with(&arguments, &skein_shell::environment(), working.as_os_str().as_bytes())
        .and_then(crate::run)
    {
        Ok(code) => ExitCode::from(code),
        Err(why) => {
            eprint!("{why}");
            ExitCode::FAILURE
        }
    }
}
