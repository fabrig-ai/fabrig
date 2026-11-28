//! Bind a long-running process to its test (temper/docs/design/testing/fixture-lifetime.md, 1).
use skein_io::kernel::Signal;
use skein_world::end_to_end::Command;
use std::{path::Path, process};

/// Spawn on the thread that keeps the process. On Linux, its exit sends the signal.
pub fn spawn(command: &mut process::Command, signal: Signal) -> std::io::Result<process::Child> {
    skein_shell::spawn_dying_with_parent(command, signal)
}

/// Wrap a skein-io command; the world's loop must spawn and keep it on the same thread.
#[must_use]
pub fn dying_with_test(command: Command, signal: Signal) -> Command {
    if cfg!(target_os = "linux") {
        dying_with_program(command, &skein_scratch::paths::built("temper-box", "temper-box"), signal)
    } else {
        command
    }
}

/// A story starting an installed box supplies its own wrapper executable.
#[must_use]
pub fn dying_with_program(mut command: Command, wrapper: &Path, signal: Signal) -> Command {
    if cfg!(target_os = "linux") {
        let word = match signal {
            Signal::Terminate => "terminate",
            Signal::Kill => "kill",
            Signal::Interrupt | Signal::Stop | Signal::Continue => panic!("not a fixture death signal"),
        };
        let mut arguments = vec![
            "internal".into(),
            "forge-child".into(),
            "--parent".into(),
            process::id().to_string().into(),
            "--signal".into(),
            word.into(),
            "--".into(),
            command.program.into(),
        ];
        arguments.extend(command.arguments);
        command.program = wrapper.into();
        command.arguments = arguments;
    }
    command
}

pub mod tests {
    use super::{Command, Path, Signal, dying_with_program, dying_with_test};
    use std::ffi::OsString;

    fn check(launched: &Command, original: Command, wrapper: &Path, signal: &str) {
        assert_eq!(launched.directory, original.directory);
        assert_eq!(launched.environment, original.environment);
        let mut arguments = Vec::<OsString>::new();
        if cfg!(target_os = "linux") {
            assert_eq!(launched.program, wrapper);
            arguments.extend(
                ["internal", "forge-child", "--parent", &std::process::id().to_string(), "--signal", signal, "--"]
                    .map(OsString::from),
            );
            arguments.push(original.program.into());
        } else {
            assert_eq!(launched.program, original.program);
        }
        arguments.extend(original.arguments);
        assert_eq!(launched.arguments, arguments);
    }

    pub fn an_explicit_program_supplies_its_own_death_signal_wrapper() {
        let installed = Path::new("/installed/temper-box");
        let command = || Command {
            program: installed.into(),
            arguments: ["--config", "/fixture"].map(Into::into).into(),
            directory: "/fixture".into(),
            environment: Vec::new(),
        };
        check(&dying_with_program(command(), installed, Signal::Terminate), command(), installed, "terminate");
    }

    pub fn a_fixture_wrapper_preserves_the_command_and_names_its_owner_and_signal() {
        let command = || Command {
            program: "/built/temper-engine".into(),
            arguments: ["serve", "--config", "/c"].map(Into::into).into(),
            directory: "/c".into(),
            environment: vec![("PATH".into(), "/fixture/bin".into())],
        };
        let wrapper = skein_scratch::paths::built("temper-box", "temper-box");
        for (signal, word) in [(Signal::Terminate, "terminate"), (Signal::Kill, "kill")] {
            check(&dying_with_test(command(), signal), command(), &wrapper, word);
        }
    }

    pub fn long_running_kit_commands_name_their_fixture_death_signal() {
        use crate::{cluster, configuration, serve_command, serve_engine_command, sign_in};
        let root = Path::new("/fixture");
        let wrapper = skein_scratch::paths::built("temper-box", "temper-box");
        let cluster = cluster::Cluster::documents(
            &root.join("cluster"),
            &[
                ("forge", "address", "127.0.0.1:3"),
                ("llm", "address", "127.0.0.1:1"),
                ("engine", "workers", "127.0.0.1:2"),
            ],
        );
        for member in ["engine", "w1", "forge", "llm"] {
            let original = cluster.render_command(member);
            check(&cluster::member_fixture(&original), original, &wrapper, "kill");
        }
        let engine = wrapper.with_file_name("temper-engine");
        let box_flags = configuration::flags(root);
        let serve_flags = |flags: Vec<OsString>| std::iter::once("serve".into()).chain(flags).collect();
        let engine_flags = ["config", "data", "cache"]
            .into_iter()
            .flat_map(|name| [format!("--{name}").into(), root.join(name).into()])
            .collect();
        for (launched, program, arguments, environment, signal) in [
            (
                serve_command(root),
                wrapper.clone(),
                serve_flags(box_flags.clone()),
                vec![("PATH".into(), "/usr/bin:/bin".into())],
                "kill",
            ),
            (serve_engine_command(root, engine.clone()), engine, serve_flags(engine_flags), Vec::new(), "kill"),
            (sign_in::box_command(root), wrapper.clone(), box_flags, Vec::new(), "terminate"),
        ] {
            check(&launched, Command { program, arguments, directory: root.into(), environment }, &wrapper, signal);
        }
    }
}
