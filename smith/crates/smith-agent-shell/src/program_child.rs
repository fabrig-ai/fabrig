//! Child-side exec for programs that die with their agent (smith/docs/design/protocol/programs.md, 5.1).
//! Keeps only argv until exec, inheriting environment, directory and descriptors.
//! `main` enters this mode; `parse` refuses malformed argv before any exec.

use std::{ffi::OsString, io::Write, process::ExitCode};

struct Invocation {
    parent: u32,
    program: OsString,
    arguments: Vec<OsString>,
}

fn parse(arguments: &[OsString]) -> Result<Invocation, &'static str> {
    let parent = arguments.first().ok_or("missing parent")?.to_str().ok_or("invalid parent")?;
    if parent.is_empty() || parent.starts_with('0') || !parent.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err("invalid parent");
    }
    let parent = parent.parse::<u32>().or(Err("invalid parent"))?;
    let program = arguments.get(1).filter(|program| !program.is_empty()).ok_or("missing program")?;
    Ok(Invocation {
        parent,
        program: program.clone(),
        arguments: arguments.get(2..).expect("program follows parent").to_vec(),
    })
}

#[expect(clippy::disallowed_types, reason = "protocol/programs.md, 5.1: the program dies with its agent")]
fn exec(invocation: Invocation) -> std::io::Error {
    let mut command = std::process::Command::new(invocation.program);
    command.args(invocation.arguments);
    skein_shell::exec_dying_with_parent(command, invocation.parent, skein_io::kernel::Signal::Kill)
}

fn run(arguments: &[OsString], errors: &mut dyn Write) -> ExitCode {
    let why = match parse(arguments) {
        Ok(invocation) => exec(invocation).to_string(),
        Err(why) => why.into(),
    };
    let _written = writeln!(errors, "smith-agent: program child: {why}");
    ExitCode::FAILURE
}

/// Exec the program with Kill on its agent's death, or report one line and exit 1.
pub fn main() -> ExitCode {
    run(&std::env::args_os().skip(2).collect::<Vec<_>>(), &mut std::io::stderr())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn malformed_arguments_refuse_without_exec() {
        for (arguments, why) in [
            (Vec::new(), "missing parent"),
            (vec!["-1", "/must-not-exec"], "invalid parent"),
            (vec!["012", "/must-not-exec"], "invalid parent"),
            (vec!["x", "/must-not-exec"], "invalid parent"),
            (vec!["0", "/must-not-exec"], "invalid parent"),
            (vec!["4294967296", "/must-not-exec"], "invalid parent"),
            (vec!["42"], "missing program"),
        ] {
            let arguments: Vec<OsString> = arguments.into_iter().map(Into::into).collect();
            assert_eq!(parse(&arguments).err(), Some(why), "{arguments:?}");
            let mut errors = Vec::new();
            assert_eq!(run(&arguments, &mut errors), ExitCode::from(1), "{arguments:?}");
            assert_eq!(errors, format!("smith-agent: program child: {why}\n").as_bytes(), "{arguments:?}");
        }
    }
}
