//! The fake git core on this process's files, pipes and forge route (testing/fakes.md, 3).
use skein_io::kernel::{Exit, Fd, Spawn};
use skein_shell::Host as _;
use std::{ffi::OsStr, path::Path};
use temper_fake_git::{Git, Route, Start};
use temper_shell::directories::EnvironmentEntry;

/// A link named `git` selects the fake before temper reads any of its own flags.
#[must_use]
pub fn is_git(program: &OsStr) -> bool {
    Path::new(program).file_name() == Some(OsStr::new("git"))
}

/// Run as git with git's argv: every argument after the program name of a
/// link named `git`, or after the `temper fake git` verb.
#[must_use]
pub fn run(arguments: &[Box<[u8]>], program: &[u8], environment: &[EnvironmentEntry], working: &[u8]) -> u8 {
    match start(arguments, program, environment, working) {
        Ok((mut git, mut kernel)) => match skein_shell::drive(&mut kernel, &skein_shell::Clock::new(), &mut git) {
            Exit::Code(code) => code,
            Exit::Signal(_) => 128,
        },
        Err(why) => {
            eprintln!("fatal: {why}");
            128
        }
    }
}

fn start(
    arguments: &[Box<[u8]>],
    program: &[u8],
    environment: &[EnvironmentEntry],
    working: &[u8],
) -> Result<(Git<Route>, skein_shell::Kernel), String> {
    let root = skein_shell::open_root(Path::new("/")).map_err(|why| format!("/: errno {why}"))?;
    let spawn = invocation(arguments, program, environment, working, root);
    let start = Start {
        root,
        pipes: Box::new([(0, Fd::new(0)), (1, Fd::new(1)), (2, Fd::new(2))]),
        owned: Box::new([root, Fd::new(0), Fd::new(1), Fd::new(2)]),
    };
    let git = Git::new(&spawn, start, Route::new(temper_fake::forge::git_limits().git()));
    let kernel = skein_shell::Kernel::open(skein_shell::Config { operations: git.operations() }).map_err(|why| {
        skein_shell::keeper::close_keeper_fd(root);
        why.to_string()
    })?;
    Ok((git, kernel))
}

fn invocation(
    arguments: &[Box<[u8]>],
    program: &[u8],
    environment: &[EnvironmentEntry],
    working: &[u8],
    root: Fd,
) -> Spawn {
    // Kernel file paths are relative to their root. Removing only the leading
    // slash keeps the core's at/name joins beneath /, including local commands.
    let at = working.strip_prefix(b"/").expect("absolute working directory");
    let at = if at.is_empty() { b"." } else { at };
    Spawn {
        program: program.into(),
        args: arguments.into(),
        env: environment.iter().map(|(key, value)| [key.as_ref(), b"=", value.as_ref()].concat().into()).collect(),
        root,
        dir: at.into(),
        pipes: Box::new([]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn invocation_uses_the_given_process_facts() {
        let environment = [(Box::from(b"PATH".as_slice()), Box::from(b"/fixture/bin".as_slice()))];
        let arguments = [Box::from(b"status".as_slice())];
        let spawn = invocation(&arguments, b"/fixture/git", &environment, b"/fixture/repository", Fd::new(7));
        assert_eq!(spawn.program.as_ref(), b"/fixture/git");
        assert_eq!(spawn.args.as_ref(), &arguments);
        assert_eq!(spawn.env.as_ref(), &[Box::<[u8]>::from(b"PATH=/fixture/bin".as_slice())]);
        assert_eq!(spawn.dir.as_ref(), b"fixture/repository");
        assert_eq!(invocation(&[], b"git", &[], b"/", Fd::new(7)).dir.as_ref(), b".");
    }
}
