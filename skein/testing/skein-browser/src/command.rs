//! The command and descriptor layout the owner uses to spawn Chromium.

#![expect(clippy::disallowed_types, reason = "the profile argument is assembled once before spawn")]

use alloc::boxed::Box;
use alloc::vec::Vec;

use skein_lib::List;

/// A child pipe requested from the process spawner.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Pipe {
    pub descriptor: u32,
    pub direction: Direction,
}

/// Direction, as seen by the child.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Direction {
    Read,
    Write,
}

/// Spawn configuration; the profile path must name a fresh directory owned
/// by this browser and removed by its owner after child exit.
#[derive(Debug)]
pub struct Command {
    pub program: Box<[u8]>,
    pub args: List<Box<[u8]>>,
    pub env: List<Variable>,
    pub pipes: [Pipe; 3],
}

/// One child environment variable.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Variable {
    pub name: Box<[u8]>,
    pub value: Box<[u8]>,
}

/// Build Chromium's quiet, offline command with a fixed locale and time zone.
/// With `file_access`, a `file://` page can read other files Chromium can read.
#[must_use]
pub fn command(program: &[u8], profile: &[u8], file_access: bool) -> Command {
    assert!(!program.is_empty() && !program.contains(&0), "a program path has no NUL");
    assert!(!profile.is_empty() && !profile.contains(&0), "a profile path has no NUL");
    let mut args = List::with_capacity(16);
    for arg in [
        b"--headless".as_slice(),
        b"--remote-debugging-pipe",
        b"--no-first-run",
        b"--no-default-browser-check",
        b"--disable-background-networking",
        b"--disable-sync",
        b"--disable-extensions",
        b"--disable-component-update",
        b"--disable-default-apps",
        b"--disable-features=MediaRouter",
        b"--metrics-recording-only",
        b"--js-flags=--liftoff-only",
        b"--use-mock-keychain",
    ] {
        args.push(Box::from(arg)).expect("the argument list has room");
    }
    let mut profile_arg = Vec::new();
    profile_arg.extend_from_slice(b"--user-data-dir=");
    profile_arg.extend_from_slice(profile);
    args.push(profile_arg.into_boxed_slice()).expect("the argument list has room");
    if file_access {
        args.push(Box::from(b"--allow-file-access-from-files".as_slice())).expect("the argument list has room");
    }
    args.push(Box::from(b"about:blank".as_slice())).expect("the argument list has room");
    let mut env = List::with_capacity(3);
    env.push(Variable { name: Box::from(b"LANG".as_slice()), value: Box::from(b"C.UTF-8".as_slice()) })
        .expect("room for locale");
    env.push(Variable { name: Box::from(b"LC_ALL".as_slice()), value: Box::from(b"C.UTF-8".as_slice()) })
        .expect("room for locale");
    env.push(Variable { name: Box::from(b"TZ".as_slice()), value: Box::from(b"UTC".as_slice()) })
        .expect("room for time zone");
    Command {
        program: Box::from(program),
        args,
        env,
        pipes: [
            Pipe { descriptor: 3, direction: Direction::Read },
            Pipe { descriptor: 4, direction: Direction::Write },
            Pipe { descriptor: 2, direction: Direction::Write },
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::{Direction, command};

    const CHROMIUM: &[u8] = b"/usr/bin/chromium";
    const PROFILE: &[u8] = b"/tmp/fresh-profile";

    #[test]
    fn chromium_adds_only_the_opt_in_file_access_flag() {
        const FLAG: &[u8] = b"--allow-file-access-from-files";
        let without = command(CHROMIUM, PROFILE, false);
        let with = command(CHROMIUM, PROFILE, true);
        let (page_without, args_without) =
            without.args.as_slice().split_last().expect("initial page without file access");
        let (page_with, args_with) = with.args.as_slice().split_last().expect("initial page with file access");
        let (flag, args_with) = args_with.split_last().expect("file access flag before the initial page");

        assert_eq!(page_without.as_ref(), b"about:blank");
        for arg in args_without {
            assert_ne!(arg.as_ref(), FLAG, "file access is opt-in");
        }
        assert_eq!((args_with, flag.as_ref(), page_with), (args_without, FLAG, page_without));
        assert_eq!(with.env, without.env);
        assert_eq!(with.pipes, without.pipes);
    }

    #[test]
    fn chromium_keeps_wasm_in_the_baseline_tier() {
        const ONE_V8_FLAG_ARGUMENT: usize = 1;
        let command = command(CHROMIUM, PROFILE, false);
        let mut flags = 0;
        for arg in &command.args {
            if arg.starts_with(b"--js-flags=") {
                assert_eq!(arg.as_ref(), b"--js-flags=--liftoff-only");
                flags += 1;
            }
        }
        assert_eq!(flags, ONE_V8_FLAG_ARGUMENT);
    }

    #[test]
    fn chromium_pipe_command() {
        let command = command(CHROMIUM, PROFILE, false);
        assert_eq!(command.program.as_ref(), b"/usr/bin/chromium");
        assert_eq!(command.args.last().expect("initial page").as_ref(), b"about:blank");
        let mut profile = false;
        let mut mock_keychain = false;
        for arg in &command.args {
            if arg.as_ref() == b"--user-data-dir=/tmp/fresh-profile" {
                profile = true;
            }
            if arg.as_ref() == b"--use-mock-keychain" {
                mock_keychain = true;
            }
        }
        assert!(mock_keychain, "profile uses a mock keychain");
        assert!(profile, "the command has a fresh profile");
        assert_eq!(command.pipes.first().expect("command pipe").descriptor, 3);
        assert_eq!(command.pipes.first().expect("command pipe").direction, Direction::Read);
        assert_eq!(command.pipes.get(1).expect("reply pipe").descriptor, 4);
        assert_eq!(command.pipes.get(1).expect("reply pipe").direction, Direction::Write);
        assert_eq!(command.pipes.get(2).expect("stderr pipe").descriptor, 2);
    }
}
