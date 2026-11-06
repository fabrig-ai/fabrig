//! Home's stored count and status agent on a fresh fake box
//! (temper/docs/design/client/README.md, 3.5; temper/docs/design/domain/forge.md, 5.6, 5.7).
use super::open_default::{browser_bounds, named};
use super::{Act, Call, Walk, ctl, path};
use crate::support::WITHIN;
use skein_lib::Time;
use skein_shell::{Clock, Host};
use skein_ui::{Act as Input, Role};
use skein_world::{Referee, real};
use std::path::Path;
use temper_client_domain::Address;
use temper_client_shell_terminal::client::{NativeClient, Settings, Start};

const ASK: &str = "Land a local status change.";
const SENTENCE: &[u8] = b"1 change landed on the local forge.";
const COMPUTED: &[u8] = b"One change landed since you last looked. Nothing needs you.";

#[test]
fn home_on_the_fake_box_keeps_a_landing_count_after_box_and_client_restarts() {
    let walk = Walk::new("walk-status");
    let sandbox = walk.sandbox();
    let config = sandbox.join("config/temper.json");
    let client = walk.scratch.path().to_path_buf();
    let first_config = sandbox.join("config");
    walk.run(vec![]);
    let mut document: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&config).expect("public configuration")).expect("JSON");
    // Reopen the prepared box with short polling intervals, leaving room for
    // the real journal to commit between polls instead of filling it with reads.
    let limits = &mut document["application"]["limits"]["overrides"];
    for (name, millis) in [("poll_ms", 100), ("poll_max_ms", 500), ("slow_ms", 1000), ("hinted_ms", 100)] {
        limits[format!("forge.client.{name}")] = millis.into();
    }
    super::super::write(&config, &serde_json::to_vec(&document).expect("configuration JSON"));
    let seen = walk.run_then(
        vec![
            Act::After(Box::new(move |_| {
                let document = temper_fake::forge::Document::decode(
                    &std::fs::read(sandbox.join("config/forgejo.json")).expect("fake forge document"),
                )
                .expect("forge document");
                Call::new(&[
                    "forge",
                    "connect",
                    "forgejo-admin",
                    path(&sandbox.join("data/forgejo-admin.token")),
                    "--api",
                    &document.listen().to_string(),
                    "--git",
                    &format!("http://{}", document.listen()),
                    "--git-program",
                    path(&sandbox.join("data/demo/bin/git")),
                    "--first",
                    "work/widget",
                    "--key",
                    "status-forge",
                    "--json",
                ])
            })),
            Act::Now(Call::new(&[
                "repository",
                "adopt",
                "--project",
                "1",
                "--forge",
                "1",
                "--owner",
                "work",
                "--name",
                "widget",
                "--landing",
                "main",
                "--no-ci",
                "--check",
                "1",
                "--key",
                "status-repository",
                "--json",
            ])),
            walk.add_key(),
            Act::Now(Call::new(&["account", "list", "--until", "usable", "--json"])),
            Act::After(Box::new(move |_| {
                show_home(&first_config, &client, Visit::First);
                Call::new(&[
                    "chat",
                    "start",
                    "--project",
                    "1",
                    "--model",
                    "gpt-6-luna",
                    "--key",
                    "status-chat",
                    ASK,
                    "--json",
                ])
            })),
            Act::Now(Call::new(&["task", "show", "1", "--until", "phase:idle", "--json"])),
            Act::Now(Call::new(&["change", "show", "2", "--until", "landed", "--json"])),
        ],
        show_status,
    );
    assert_eq!(seen.answers.len(), 7, "setup, first look, chat and landed change");
    let landed = ctl::json(seen.answers.last().expect("landing read"));
    assert_eq!(landed["met"], true);
    assert_eq!(landed["change"]["step"], "landed");
    assert_eq!(landed["change"]["merge"].as_str().expect("merge commit").len(), 40);
    walk.run_then(vec![], show_status);
    walk.scratch.remove();
}

#[derive(Debug)]
enum Page {
    Starting,
    Begin,
    FirstLook,
    Computed,
    Done,
}
#[derive(Clone, Copy)]
enum Visit {
    First,
    Return,
    Restart,
}
struct Judge {
    page: Page,
    visit: Visit,
    until: Time,
}

fn words(client: &NativeClient, needle: &[u8]) -> bool {
    client
        .tree()
        .nodes()
        .iter()
        .flat_map(|node| node.text.iter())
        .any(|run| run.text.windows(needle.len()).any(|part| part == needle))
}
impl Referee<NativeClient> for Judge {
    fn act(&mut self, now: Time, clients: &mut [NativeClient]) {
        let client = &mut clients[0];
        match self.page {
            Page::Starting if client.started() && named(client, Role::Status, b"Owner").is_some() => {
                client.go(Address::Home { project: 1 });
                self.page = match self.visit {
                    Visit::First => Page::Begin,
                    Visit::Return | Visit::Restart => Page::Computed,
                };
            }
            Page::Begin => {
                if let Some(skip) = named(client, Role::Button, b"Skip") {
                    let id = skip.id;
                    assert!(client.act(id, Input::Press), "skip To begin");
                    self.page = Page::FirstLook;
                }
            }
            Page::FirstLook if words(client, b"Landing: ") && words(client, b"nothing has landed yet") => {
                assert!(!words(client, SENTENCE), "the first look precedes the landing");
                assert!(!words(client, b"Written by the status agent"), "no brief before a landing");
                client.close();
                self.page = Page::Done;
            }
            Page::Computed if words(client, COMPUTED) => {
                assert!(words(client, b"Landing: "));
                assert!(!words(client, b"Written by the status agent"), "the agent waits for a new sitting");
                assert!(!words(client, b"nothing has landed yet"), "the real landing reaches Home's foot");
                assert!(named(client, Role::Button, b"Skip").is_none(), "the skip survives a client restart");
                client.close();
                self.page = Page::Done;
            }
            Page::Starting | Page::FirstLook | Page::Computed | Page::Done => {}
        }
        assert!(
            self.passed() || now < self.until,
            "status walk stalled at {:?}; tree: {:?}",
            self.page,
            client
                .tree()
                .nodes()
                .iter()
                .map(|node| (
                    node.role,
                    node.name.as_deref().map(String::from_utf8_lossy),
                    node.text.iter().map(|run| String::from_utf8_lossy(&run.text)).collect::<Vec<_>>()
                ))
                .collect::<Vec<_>>()
        );
    }
    fn observe(&mut self, _now: Time, _clients: &[NativeClient]) {}
    fn next_deadline(&self) -> Option<Time> {
        (!self.passed()).then_some(self.until)
    }
    fn passed(&self) -> bool {
        matches!(self.page, Page::Done)
    }
}

fn show_status(config: &Path, client: &Path) {
    // The process tier has a 1080-second wall backstop. The 31-minute sitting is
    // a separate render proof; this walk waits only on the real box's answers.
    let model: serde_json::Value =
        serde_json::from_slice(&std::fs::read(config.join("llm.json")).expect("public model document"))
            .expect("model JSON");
    let script = model["cued"]
        .as_array()
        .expect("system-cued scripts")
        .iter()
        .find(|script| script["cue"] == "You write the few lines at the top of a project's home page")
        .expect("the status charter's script");
    assert_eq!(
        script["turns"][0]["lines"][0]["arguments"]["text"],
        "1 change landed on the local forge.\n\nThe README now records the local status change."
    );
    for visit in [Visit::Return, Visit::Restart] {
        show_home(config, client, visit);
    }
}

fn show_home(config: &Path, client: &Path, visit: Visit) {
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
        real::World::new(Judge { page: Page::Starting, visit, until: clock.now().now.saturating_add(WITHIN) });
    world.spawn_with_fds(vec![root], || {
        NativeClient::new(
            Settings {
                engine: skein_io::Addr::Ip(engine),
                directory: b"client".as_slice().into(),
                limits: browser_bounds(),
            },
            root,
            skein_shell::effective_user(),
            96,
            Start::Token,
        )
        .expect("browser-bounded native client")
    });
    let ended = world.run(&clock, WITHIN);
    assert!(ended.procs.iter().all(Host::is_empty), "status walk drained");
}
