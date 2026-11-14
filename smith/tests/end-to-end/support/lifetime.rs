//! Outside observation of an aborted agent's tree (smith/docs/design/protocol/programs.md, 5.1).
//! The keeper owns cleanup; the referee keeps PID/start-time identities, never environment.
use skein_io::kernel::{Exit, Signal};
use skein_lib::{Duration, Time};
use skein_shell::{Clock, Host};
use skein_world::{
    Referee,
    end_to_end::{Binary, Command, Expectation, Mode},
    real,
};
use smith_real_world::Scratch;

#[derive(Debug)]
struct Identity {
    pid: u32,
    parent: u32,
    start: u64,
    arguments: Vec<Vec<u8>>,
}

fn identity(pid: u32) -> Option<Identity> {
    let stat = std::fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
    let fields: Vec<_> = stat.rsplit_once(") ")?.1.split_whitespace().collect();
    // After comm: state (3), ppid (4), ..., starttime (22).
    if fields.first() == Some(&"Z") {
        return None;
    }
    Some(Identity {
        pid,
        parent: fields.get(1)?.parse().ok()?,
        start: fields.get(19)?.parse().ok()?,
        arguments: std::fs::read(format!("/proc/{pid}/cmdline"))
            .ok()?
            .split(|byte| *byte == 0)
            .filter(|argument| !argument.is_empty())
            .map(<[u8]>::to_vec)
            .collect(),
    })
}

fn alive(process: &Identity) -> bool {
    identity(process.pid).is_some_and(|current| current.start == process.start)
}

#[expect(
    clippy::disallowed_types,
    reason = "protocol/programs.md, 5.1: SIGKILL targets the agent alone, without ending its tree"
)]
fn kill_agent(agent: &Identity) -> Result<(), String> {
    if !alive(agent) {
        return Err(String::from("the observed agent identity still holds"));
    }
    let status = std::process::Command::new("/bin/kill")
        .args(["-KILL", &agent.pid.to_string()])
        .status()
        .map_err(|_| String::from("signal the agent"))?;
    if !status.success() {
        return Err(String::from("SIGKILL delivered to the agent"));
    }
    Ok(())
}

struct Judge {
    finding: Option<String>,
    poll: Time,
    deadline: Time,
    killed: Option<Time>,
    found: Option<[Identity; 3]>,
    leftovers: Option<Vec<u32>>,
    cleaning: bool,
}

impl Judge {
    fn record_leftovers(&mut self, now: Time) {
        if let Some(found) = &self.found {
            let live: Vec<_> =
                found.iter().skip(1).filter(|process| alive(process)).map(|process| process.pid).collect();
            if self.leftovers.is_none() && (live.is_empty() || now >= self.deadline) {
                self.leftovers = Some(live);
            }
        }
    }

    fn act_checked(&mut self, now: Time, processes: &mut [Binary]) -> Result<(), String> {
        if now < self.poll {
            return Ok(());
        }
        self.poll = now.saturating_add(Duration::from_millis(10));
        let binary = processes.first_mut().ok_or("product")?;
        if self.killed.is_none() && !self.cleaning {
            let members: Vec<_> = binary.tree_status().live_pids.iter().filter_map(|pid| identity(*pid)).collect();
            let product =
                members.iter().find(|process| process.arguments.first().is_some_and(|name| name.ends_with(b"/smith")));
            let agent = members.iter().find(|process| {
                product.is_some_and(|product| process.parent == product.pid)
                    && process.arguments.first().is_some_and(|name| name.ends_with(b"/smith-agent"))
                    && !process.arguments.iter().any(|argument| argument == b"--mcp-relay")
            });
            if let Some(agent) = agent {
                let print = members.iter().find(|process| {
                    process.parent == agent.pid && process.arguments.iter().any(|argument| argument == b"-p")
                });
                if let Some(print) = print {
                    let relay = members.iter().find(|process| {
                        process.parent == print.pid
                            && process.arguments.iter().any(|argument| argument == b"--mcp-relay")
                    });
                    if let Some(relay) = relay {
                        let found = [
                            identity(agent.pid).ok_or("hanging fixture stays alive")?,
                            identity(print.pid).ok_or("hanging fixture stays alive")?,
                            identity(relay.pid).ok_or("hanging fixture stays alive")?,
                        ];
                        eprintln!(
                            "abort tree (pid, start): {:?}",
                            found.iter().map(|process| (process.pid, process.start)).collect::<Vec<_>>()
                        );
                        kill_agent(agent)?;
                        self.found = Some(found);
                        self.killed = Some(now);
                        self.deadline = now.saturating_add(Duration::from_secs(10));
                    }
                }
            }
        }
        self.record_leftovers(now);
        if now >= self.deadline && !self.cleaning && !binary.is_empty() {
            self.cleaning = true;
            binary.signal_tree(Signal::Kill);
        }
        Ok(())
    }
}

impl Referee<Binary> for Judge {
    fn act(&mut self, now: Time, processes: &mut [Binary]) {
        if self.finding.is_none() {
            self.finding = self.act_checked(now, processes).err();
        }
    }

    fn watch(&mut self, now: Time, processes: &[Binary]) -> Result<(), String> {
        if let Some(why) = &self.finding {
            return Err(why.clone());
        }
        let product = processes.first().ok_or("product")?;
        if product.is_empty() {
            self.record_leftovers(now);
            if self.found.is_none() {
                return Err(format!("agent, print and relay observed: {}", String::from_utf8_lossy(product.stderr())));
            }
            if self.leftovers.as_deref() != Some([].as_slice()) {
                return Err(format!(
                    "the fake print process and its relay must die with the agent\nleft: {:?}\nright: Some([])",
                    self.leftovers.as_deref()
                ));
            }
        }
        Ok(())
    }

    fn next_deadline(&self) -> Option<Time> {
        Some(self.poll)
    }

    fn overdue(&self, _now: Time) -> Option<String> {
        None
    }

    fn passed(&self) -> bool {
        self.finding.is_none() && (self.leftovers.is_some())
    }
}

pub fn abort_agent(scratch: &Scratch) {
    let clock = Clock::new();
    let start = clock.now().now;
    let binary = Binary::start_with_tree(
        Command {
            program: skein_scratch::paths::binary("smith"),
            arguments: vec!["exec".into(), "Hang until the agent ends".into()],
            environment: vec![
                ("HOME".into(), scratch.path().into()),
                ("XDG_CONFIG_HOME".into(), scratch.path().join("config").into()),
                ("XDG_STATE_HOME".into(), scratch.path().join("state").into()),
                ("XDG_RUNTIME_DIR".into(), scratch.path().join("runtime").into()),
                ("PATH".into(), "/usr/bin:/bin".into()),
            ],
            directory: scratch.path().into(),
        },
        Mode::Pipes,
        65_536,
        Expectation::Supervise,
        false,
    )
    .expect("supervised product");
    let mut world = real::World::new(Judge {
        finding: None,
        poll: start,
        deadline: start.saturating_add(Duration::from_secs(10)),
        killed: None,
        found: None,
        leftovers: None,
        cleaning: false,
    });
    world.spawn_with_fds(binary.descriptors(), || binary);
    let outcome = world.run(&clock, Duration::from_secs(25));
    let product = outcome.procs.first().expect("product");
    assert_eq!(product.exit_status(), Some(Exit::Code(1)), "{}", String::from_utf8_lossy(product.stderr()));
    let errors = String::from_utf8_lossy(product.stderr());
    assert!(
        errors.contains("agent") && (errors.contains("ended") || errors.contains("Exited")),
        "agent end named: {errors}"
    );
    let counts = product.counts().expect("settled tree counts");
    eprintln!(
        "abort tree CPU user={:?} system={:?}; wall={:?}",
        counts.user,
        counts.system,
        outcome.end.saturating_since(start)
    );
}
