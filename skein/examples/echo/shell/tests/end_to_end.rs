//! The echo's shipped main against two fake clients, observed only through
//! loopback, stderr and its ring exit (testing-strategy.md, section 2.9).
//! This test belongs to the shell package so Cargo supplies its actual binary.

use skein_echo_client::{self as client, Client, client_limits, plan};
use skein_io::Addr;
use skein_io::kernel::{Complete, Exit, Signal, Submit};
use skein_lib::{Duration, Queue, Time, Wall};
use skein_shell::Clock;
use skein_world::end_to_end::{Binary, Command, Mode, StartError};
use skein_world::{Deadline, Delegate, Expired, Host, Referee, real};

#[derive(Debug)]
#[expect(
    clippy::large_enum_variant,
    reason = "The dispatcher keeps the binary observer inline beside its smaller fake client."
)]
enum Process {
    Echo(Binary),
    Client(ClientHost),
}

#[derive(Debug)]
struct ClientHost {
    client: Box<Client>,
    limits: client::Limits,
}

impl Host for ClientHost {
    fn iterate(&mut self, now: Time, wall: Wall) {
        client::iterate(&mut self.client, now, wall);
    }
    fn completions(&mut self) -> &mut Queue<Complete> {
        self.client.completions()
    }
    fn submissions(&mut self) -> &mut Queue<Submit> {
        self.client.submissions()
    }
    fn work_pending(&self, now: Time) -> bool {
        self.client.work_pending(now)
    }
    fn next_mechanism_deadline(&self) -> Option<Time> {
        self.client.next_deadline()
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        self.client.next_policy_deadline()
    }
    fn is_empty(&self) -> bool {
        self.client.is_empty()
    }
    fn operations(&self) -> u32 {
        client::operations(&self.limits).expect("a small ring")
    }
}

impl Delegate for Process {
    fn host(&self) -> &dyn Host {
        match self {
            Self::Echo(binary) => binary,
            Self::Client(client) => client,
        }
    }

    fn host_mut(&mut self) -> &mut dyn Host {
        match self {
            Self::Echo(binary) => binary,
            Self::Client(client) => client,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Clients {
    Waiting,
    Answered,
    Done,
}

struct Judge {
    listening: Option<Addr>,
    told: bool,
    clients: Clients,
    signalled: bool,
    exit: Option<Exit>,
    now: Time,
    deadline: Time,
}

impl Referee<Process> for Judge {
    fn act(&mut self, _now: Time, procs: &mut [Process]) {
        if !self.told
            && let Some(addr) = self.listening
        {
            for process in &mut *procs {
                match process {
                    Process::Client(host) => host.client.dial(addr),
                    Process::Echo(_) => {}
                }
            }
            self.told = true;
        }
        if self.clients != Clients::Waiting && !self.signalled {
            let Process::Echo(binary) = procs.first_mut().expect("the binary observer") else {
                panic!("the binary observer")
            };
            binary.signal(Signal::Terminate);
            self.signalled = true;
        }
    }

    fn watch(&mut self, now: Time, procs: &[Process]) -> Result<(), String> {
        self.now = now;
        let Process::Echo(binary) = procs.first().expect("the binary observer") else { panic!("the binary observer") };
        for line in binary.stderr().split_inclusive(|byte| *byte == b'\n') {
            if line.ends_with(b"\n")
                && let Some(address) = line.strip_prefix(b"skein-echo: listening at ")
            {
                self.listening = Some(Addr::Ip(
                    std::str::from_utf8(address)
                        .map_err(|_| String::from("printed address"))?
                        .trim()
                        .parse()
                        .map_err(|_| String::from("socket address"))?,
                ));
            }
        }
        if let Some(exit) = binary.exit_status() {
            if exit != Exit::Code(0) {
                return Err(format!(
                    "assertion `left == right` failed: echo startup/shutdown failed: {}\n  left: {:?}\n right: {:?}",
                    String::from_utf8_lossy(binary.stderr()),
                    exit,
                    Exit::Code(0)
                ));
            }
            if !self.signalled {
                return Err(String::from("the echo stays alive until the referee's termination signal"));
            }
            self.exit = Some(exit);
        }
        let mut done = true;
        let mut ready = true;
        for process in procs.get(1..).expect("the two fake clients") {
            let Process::Client(client) = process else { panic!("a fake client") };
            let client = &client.client;
            let seen = client.seen(0);
            if seen.broken != 0 {
                return Err(format!(
                    "assertion `left == right` failed: loopback did not break\n  left: {:?}\n right: {:?}",
                    seen.broken, 0
                ));
            }
            if seen.done.is_some() && !seen.complete {
                return Err(String::from("every line was answered before the fake ended"));
            }
            done &= seen.done.is_some();
            ready &= seen.complete;
        }
        self.clients = if done {
            Clients::Done
        } else if ready {
            Clients::Answered
        } else {
            Clients::Waiting
        };
        Ok(())
    }

    fn next_deadline(&self) -> Option<Time> {
        if (!self.told && self.listening.is_some()) || (self.clients != Clients::Waiting && !self.signalled) {
            Some(self.now)
        } else if self.passed() {
            None
        } else {
            Some(self.deadline)
        }
    }

    fn overdue(&self, now: Time) -> Option<String> {
        (now >= self.deadline && !self.passed()).then(|| {
            format!(
                "echo end to end did not settle: listening={}, clients_done={}, signalled={}, exited={}",
                self.listening.is_some(),
                self.clients == Clients::Done,
                self.signalled,
                self.exit.is_some()
            )
        })
    }

    fn passed(&self) -> bool {
        self.clients == Clients::Done && self.signalled && self.exit.is_some()
    }
}

#[test]
fn sigterm_with_idle_clients_exits_at_once_with_success() {
    let clock = Clock::new();
    let start = clock.now().now;
    let binary = Binary::start(
        Command {
            program: skein_scratch::paths::binary("skein-echo"),
            arguments: vec!["127.0.0.1:0".into()],
            environment: vec![],
            directory: std::env::current_dir().expect("test working directory"),
        },
        Mode::Pipes,
        16 * 1024,
    )
    .unwrap_or_else(|error| match error {
        StartError::Ring(why) => panic!("io_uring is not usable here, so the echo end to end cannot run: {why}"),
        StartError::Directory(errno) => panic!("echo working directory failed: errno {errno}"),
        StartError::Tree(error) => panic!("tree startup failed: {error:?}"),
        StartError::Child(error) => panic!("starting the shipped echo failed: {error:?}"),
    });
    let mut deadline = Deadline::arm("the shipped echo shutdown", Duration::from_secs(2));
    deadline.watch(&binary);
    let mut world = real::World::new(Judge {
        listening: None,
        told: false,
        clients: Clients::Waiting,
        signalled: false,
        exit: None,
        now: start,
        deadline: start.saturating_add(Duration::from_secs(2)),
    });
    world.spawn_with_fds(binary.descriptors(), || Process::Echo(binary));
    for seed in [7, 11] {
        let plans = [skein_echo_client::Plan { then: skein_echo_client::Then::Linger, ..plan(start, seed) }];
        world.spawn(|| {
            let limits = client_limits(1);
            Process::Client(ClientHost { client: Box::new(Client::new(&limits, &plans)), limits })
        });
    }
    let outcome = world.run(&clock, Duration::from_secs(4));
    let Process::Echo(binary) = outcome.procs.first().expect("the binary observer") else {
        panic!("the shipped binary observer")
    };
    assert_eq!(binary.exit_status(), Some(Exit::Code(0)), "the signalled echo exits successfully");
    if deadline.passed() {
        match deadline.expired() {
            Expired::Starved(starved) => {
                drop(outcome);
                starved.pass();
            }
            Expired::Failed(figures) => panic!("shutdown settles well before the shipped idle time: {figures}"),
        }
    }
}
