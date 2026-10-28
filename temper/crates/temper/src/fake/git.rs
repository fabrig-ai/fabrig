//! The fake git core on this process's files, pipes and forge route (domain/testing.md, 4.3).
use skein_io::kernel::{Exit, Fd, Spawn};
use skein_shell::Host as _;
use std::{env, ffi::OsStr, os::unix::ffi::OsStrExt, path::Path};
use temper_fake_git::{Git, Route, Start};

/// A link named `git` selects the fake before temper reads any of its own flags.
#[must_use]
pub fn is_git(program: &OsStr) -> bool {
    Path::new(program).file_name() == Some(OsStr::new("git"))
}

/// Run as git with git's argv: every argument after the program name of a
/// link named `git`, or after the `temper fake git` verb.
#[must_use]
pub fn run(arguments: &[Box<[u8]>]) -> u8 {
    match start(arguments) {
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

fn start(arguments: &[Box<[u8]>]) -> Result<(Git<Route>, skein_shell::Kernel), String> {
    let working = env::current_dir().map_err(|why| why.to_string())?;
    // Kernel file paths are relative to their root. Removing only the leading
    // slash keeps the core's at/name joins beneath /, including local commands.
    let at = working.as_os_str().as_bytes().strip_prefix(b"/").expect("absolute working directory");
    let at = if at.is_empty() { b"." } else { at };
    let root = skein_shell::open_root(Path::new("/")).map_err(|why| format!("/: errno {why}"))?;
    let spawn = Spawn {
        program: env::args_os().next().map_or_else(|| Box::from(b"git".as_slice()), |name| name.as_bytes().into()),
        args: arguments.into(),
        env: env::vars_os().map(|(key, value)| [key.as_bytes(), b"=", value.as_bytes()].concat().into()).collect(),
        root,
        dir: at.into(),
        pipes: Box::new([]),
    };
    let start = Start {
        root,
        pipes: Box::new([(0, Fd::new(0)), (1, Fd::new(1)), (2, Fd::new(2))]),
        owned: Box::new([root, Fd::new(0), Fd::new(1), Fd::new(2)]),
    };
    let git = Git::new(&spawn, start, Route::new(temper_fake::forge::git_limits()));
    let kernel = skein_shell::Kernel::open(skein_shell::Config { operations: git.operations() }).map_err(|why| {
        skein_shell::close_keeper_fd(root);
        why.to_string()
    })?;
    Ok((git, kernel))
}
