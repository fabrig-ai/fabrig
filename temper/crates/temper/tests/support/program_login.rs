//! Login fixtures with configuration and public trust only, no deployment secrets.
use std::{fs, path::Path};

/// Copy the box's command, replacing only its documented Model-step placeholder.
/// The shell keeps the printed quoting intact; it execs temper as a separate process.
#[must_use]
pub fn printed(stderr: &[u8], name: &str) -> Option<Box<[u8]>> {
    let _name = smith_host_domain::SourceName::new(name.as_bytes()).expect("portable program source name");
    let stderr = String::from_utf8_lossy(stderr);
    let command = stderr.lines().find_map(|line| {
        line.strip_prefix("box: program login: ")?
            .strip_suffix("NAME (the name the Model step shows)")
            .filter(|line| line.contains(" program-login "))
    })?;
    let command = format!("exec {command}{name}");
    eprintln!("copied program-login command: {command}");
    Some(command.into_bytes().into_boxed_slice())
}

pub fn configuration(directory: &Path, programs: Option<&Path>, executable: &Path) {
    let (agent, deployment) = super::configuration::documents(
        directory,
        skein_scratch::paths::binary("temper").to_str().expect("a UTF-8 path"),
    );
    let mut agent: serde_json::Value = serde_json::from_slice(&agent).expect("agent JSON");
    if let Some(programs) = programs {
        agent["programs"] = serde_json::json!({"claude_code":{"path":executable,"directory":programs}});
    }
    fs::write(directory.join("agent.json"), serde_json::to_vec(&agent).expect("agent JSON")).expect("agent");
    fs::write(directory.join("temper.json"), deployment).expect("deployment configuration");
    fs::write(directory.join("root.der"), skein_tls_world::pki::ROOT).expect("public trust");
}

#[must_use]
pub fn refused(directory: &Path, name: &std::ffi::OsStr) -> String {
    temper::run(temper::Command::ProgramLogin {
        directories: super::configuration::directories(directory),
        name: name.into(),
    })
    .expect_err("login refused")
}

pub fn no_deployment_resources(directory: &Path) {
    for name in [".claude", "state", "worker-root", "worker.secret", "secrets", "accounts", "charters"] {
        assert!(!directory.join(name).exists(), "{name} was neither needed nor created");
    }
}

struct Judge {
    settled: bool,
}
impl skein_world::Referee<super::Process> for Judge {
    fn act(&mut self, _now: skein_lib::Time, _procs: &mut [super::Process]) {}
    fn observe(&mut self, _now: skein_lib::Time, procs: &[super::Process]) {
        self.settled = procs.iter().all(skein_shell::Host::is_empty);
    }
    fn next_deadline(&self) -> Option<skein_lib::Time> {
        None
    }
    fn overdue(&self, _now: skein_lib::Time) -> Option<String> {
        None
    }
    fn passed(&self) -> bool {
        self.settled
    }
}

pub fn run(directory: &Path) -> super::Ended {
    use skein_shell::Host;
    use skein_world::end_to_end::{Binary, Command, Mode, Streams};
    let mut binary = Binary::start(
        Command {
            program: skein_scratch::paths::binary("temper"),
            arguments: std::iter::once("program-login".into())
                .chain(super::configuration::flags(directory))
                .chain(["source".into()])
                .collect(),
            directory: directory.into(),
            environment: vec![
                ("HOME".into(), directory.into()),
                ("PATH".into(), "/bin".into()),
                ("TEMPER_BENIGN".into(), "kept".into()),
                ("CLAUDE_CONFIG_DIR".into(), "/wrong".into()),
                ("ANTHROPIC_API_KEY".into(), "removed".into()),
                ("CLAUDE_CODE_OAUTH_TOKEN".into(), "removed".into()),
                ("CLAUDE_CODE_USE_BEDROCK".into(), "removed".into()),
                ("AWS_BEARER_TOKEN_BEDROCK".into(), "removed".into()),
            ],
        },
        Mode::Pipes,
        65_536,
    )
    .expect("temper program-login starts");
    let streams = binary.take_streams().expect("login pipes");
    let fds = streams.descriptors();
    let Streams::Pipes { input, output } = streams else { panic!("pipes") };
    let mut world = skein_world::real::World::new(Judge { settled: false });
    world.spawn_with_fds(binary.descriptors(), || super::Process::Binary(Box::new(binary)));
    world.spawn_with_fds(fds, || super::Process::Stdout(Box::new(super::StdoutReader::new(input, output))));
    // The process and every inherited pipe must close within five seconds.
    let outcome = world.run(&skein_shell::Clock::new(), skein_lib::Duration::from_secs(10));
    assert!(outcome.procs.iter().all(Host::is_empty), "all login io settles");
    let super::Process::Binary(binary) = &outcome.procs[0] else { panic!("login binary") };
    let super::Process::Stdout(stdout) = &outcome.procs[1] else { panic!("login stdout") };
    super::Ended { exit: binary.exit_status(), stdout: stdout.bytes.clone(), stderr: binary.stderr().to_vec() }
}
