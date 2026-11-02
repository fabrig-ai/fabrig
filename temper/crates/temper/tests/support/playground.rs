//! Playground reset across the shipped processes (temper/docs/design/protocol/setup.md, 8.4 and 13.4).
use super::{Process, ctl};
use skein_lib::Time;
use skein_shell::{Clock, Host};
use skein_ui::Role;
use skein_world::{Referee, end_to_end::Binary, real};
use std::path::Path;
use temper_client_domain::{Address, Step};
use temper_client_shell_terminal::client::{NativeClient, Settings, Start};

#[derive(Debug)]
enum Page {
    Waiting,
    Forge,
    Leaving,
    Done,
}
struct Judge {
    page: Page,
    leaving: bool,
    until: Time,
}
fn named(client: &NativeClient, role: Role, name: &[u8]) -> bool {
    client.tree().nodes().iter().any(|node| node.role == role && node.name.as_deref() == Some(name))
}
impl Referee<NativeClient> for Judge {
    fn act(&mut self, now: Time, clients: &mut [NativeClient]) {
        let client = &mut clients[0];
        match self.page {
            Page::Waiting if client.started() && named(client, Role::Status, b"Owner") => {
                client.go(Address::Welcome { step: Step::Forge { forge: None } });
                self.page = if self.leaving { Page::Leaving } else { Page::Forge };
            }
            Page::Forge
                if named(client, Role::Heading { level: 1 }, b"Connect a forge")
                    && named(client, Role::Link, b"Forgejo on this machine") =>
            {
                assert!(!named(client, Role::Button, b"Try the playground"));
                client.close();
                self.page = Page::Done;
            }
            Page::Leaving
                if client.tree().nodes().iter().any(|node| {
                    node.text.iter().flat_map(|run| run.text.iter().copied()).collect::<Vec<_>>()
                        == "Leaving the playground…".as_bytes()
                }) =>
            {
                client.close();
                self.page = Page::Done;
            }
            Page::Waiting | Page::Forge | Page::Leaving | Page::Done => {}
        }
        assert!(
            matches!(self.page, Page::Done) || now < self.until,
            "reset reload did not reach its page: {:?}; {:?}",
            self.page,
            client
                .tree()
                .nodes()
                .iter()
                .filter_map(|node| node.name.as_deref())
                .map(String::from_utf8_lossy)
                .collect::<Vec<_>>()
        );
    }
    fn observe(&mut self, _now: Time, _clients: &[NativeClient]) {}
    fn next_deadline(&self) -> Option<Time> {
        (!matches!(self.page, Page::Done)).then_some(self.until)
    }
    fn passed(&self) -> bool {
        matches!(self.page, Page::Done)
    }
}

pub fn retired_reload(config: &Path, client: &Path) {
    reload(config, client, b"private", false);
}
fn reload(config: &Path, client: &Path, directory: &[u8], leaving: bool) {
    let document: serde_json::Value =
        serde_json::from_slice(&std::fs::read(config.join("temper.json")).expect("configuration")).expect("JSON");
    let engine = document
        .pointer("/application/clients/listener/socket")
        .and_then(serde_json::Value::as_str)
        .expect("clients listener")
        .parse()
        .expect("socket");
    let root = skein_shell::open_root(client).expect("signed-in client's parent root");
    let clock = Clock::new();
    let mut world =
        real::World::new(Judge { page: Page::Waiting, leaving, until: clock.now().now.saturating_add(super::WITHIN) });
    world.spawn_with_fds(vec![root], || {
        NativeClient::new(
            Settings {
                engine: skein_io::Addr::Ip(engine),
                directory: directory.into(),
                limits: temper_client_shell_terminal::limits::native(),
            },
            root,
            skein_shell::effective_user(),
            93,
            Start::Token,
        )
        .expect("reloaded native client")
    });
    let ended = world.run(&clock, super::WITHIN);
    assert!(ended.procs.iter().all(Host::is_empty), "reloaded native client drained");
}
fn call(client: &Path, arguments: &[&str]) -> ctl::Call {
    let directory = client.join("private");
    let arguments: Vec<_> =
        std::iter::once(directory.to_str().expect("client path")).chain(arguments.iter().copied()).collect();
    ctl::Call::bare(&arguments)
}
fn read(client: &Path, arguments: &[&str]) -> serde_json::Value {
    let ended = ctl::alone(client, call(client, arguments));
    assert_eq!(ended.exit, Some(skein_io::kernel::Exit::Code(0)), "{}", String::from_utf8_lossy(&ended.stderr));
    ctl::json(&ended)
}
pub fn end_and_check(config: &Path, data: &Path, client: &Path) {
    let ended = ctl::alone(client, call(client, &["playground", "end"]));
    assert_eq!(ended.exit, Some(skein_io::kernel::Exit::Code(0)), "{}", String::from_utf8_lossy(&ended.stderr));
    assert_eq!(ended.stdout, b"reset\n");
    check_empty(config, data, client);
}
pub fn check_empty(config: &Path, data: &Path, client: &Path) {
    // Recovery can finish without another end call; the page leaves resetting only once the row is gone (6.2).
    retired_reload(config, client);
    let forges = read(client, &["forge", "list", "--json"]);
    assert_eq!(forges["forges"], serde_json::json!([]), "{forges}");
    let accounts = read(client, &["account", "list", "--json"]);
    assert_eq!(accounts["accounts"]["deployment"], serde_json::json!([]), "{accounts}");
    let me = read(client, &["me", "--json"]);
    let projects = me["projects"].as_array().expect("projects list");
    assert!(!projects.iter().any(|row| row["name"]["bytes"] == "Playground"), "{me}");
    for name in ["playground-1", "box/groups/1", "box/started/1"] {
        assert!(!data.join(name).exists(), "reset removes {name}");
    }
}
struct ResetJudge {
    until: Time,
    settled: bool,
}
impl Referee<Process> for ResetJudge {
    fn act(&mut self, _now: Time, _procs: &mut [Process]) {}
    fn observe(&mut self, now: Time, procs: &[Process]) {
        self.settled = procs.iter().all(Host::is_empty);
        assert!(self.settled || now < self.until, "reset did not remove its project before the cut");
    }
    fn next_deadline(&self) -> Option<Time> {
        (!self.settled).then_some(self.until)
    }
    fn passed(&self) -> bool {
        self.settled
    }
}
/// Hold the box's acknowledgement while the engine deletes the project, then abandon the waiting caller.
/// The enclosing box world kills the supervised tree after this check returns.
#[must_use]
pub fn interrupt_reset(binary: &Binary, config: &Path, client: &Path) -> u32 {
    let program = std::fs::canonicalize(skein_scratch::paths::binary("temper")).expect("temper binary");
    let parent = binary
        .tree_status()
        .live_pids
        .iter()
        .find(|pid| {
            std::fs::read_link(format!("/proc/{pid}/exe")).ok().as_deref() == Some(program.as_path())
                && std::fs::read(format!("/proc/{pid}/cmdline"))
                    .is_ok_and(|bytes| bytes.split(|byte| *byte == 0).any(|arg| arg == b"box"))
        })
        .expect("supervised box parent");
    let stopped = std::process::Command::new("/usr/bin/kill")
        .args(["-STOP", &parent.to_string()])
        .status()
        .expect("pause box parent");
    assert!(stopped.success());
    let clock = Clock::new();
    let root = skein_shell::open_root(client).expect("client root");
    let mut world =
        real::World::new(ResetJudge { until: clock.now().now.saturating_add(super::WITHIN), settled: false });
    world.spawn_with_fds(vec![root], || {
        Process::Calls(Box::new(ctl::Calls::new(
            root,
            vec![
                ctl::Step::Start(call(client, &["playground", "end", "--key", "playground-reset"])),
                ctl::Step::UntilMatching(
                    call(client, &["me", "--json"]),
                    temper_real_loop_world::calls::ReadCondition::NoProjectNamed("Playground"),
                ),
                ctl::Step::Cut,
            ],
            None,
            None,
        )))
    });
    let ended = world.run(&clock, super::WITHIN);
    let Process::Calls(calls) = &ended.procs[0] else { panic!("reset calls") };
    let [reset, me] = calls.ends() else { panic!("reset and project read") };
    assert_eq!(reset.exit, Some(skein_io::kernel::Exit::Code(143)), "reset still waits for the box");
    assert!(reset.stdout.is_empty(), "reset has not answered");
    assert!(
        temper_real_loop_world::calls::ReadCondition::NoProjectNamed("Playground").matches(&me.stdout),
        "project removal committed"
    );
    reload(config, client, b"private", true);
    *parent
}
