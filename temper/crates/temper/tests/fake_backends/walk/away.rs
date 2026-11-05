//! Away, a goal's decision and its person's answers on the fake box
//! (temper/docs/design/client/README.md, 3.5, Away; temper/docs/design/domain/agent.md, 8.2).
use super::open_default::{browser_bounds, named};
use super::{Act, Call, Walk, ctl, path};
use crate::support::WITHIN;
use skein_lib::Time;
use skein_shell::{Clock, Host};
use skein_ui::{Act as Input, Role, State};
use skein_world::{Referee, real};
use std::path::Path;
use temper_client_domain::Address;
use temper_client_shell_terminal::client::{NativeClient, Settings, Start};

#[test]
fn a_goal_decides_while_away_then_waits_on_home_after_overturn_and_off() {
    let walk = Walk::new("walk-away");
    let sandbox = walk.sandbox();
    let config = sandbox.join("config");
    let client = walk.scratch.path().to_path_buf();
    let return_config = config.clone();
    let return_client = client.clone();
    let ways = client.join("away-ways.json");
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
                    "away-forge",
                    "--json",
                ])
            })),
            walk.add_key(),
            Act::Now(Call::new(&["account", "list", "--until", "usable", "--json"])),
            Act::Now(Call::new(&["settings", "show", "--json"])),
            Act::After(Box::new(move |seen| {
                let value = serde_json::json!({"items": [{
                    "setting": {"kind": "project-ways", "project": 1},
                    "set": {"drafts": false, "goals": true, "changes": false, "away": true, "way": 0}
                }]});
                crate::support::write_private(&ways, &serde_json::to_vec(&value).expect("project ways JSON"));
                ctl::settings_change(seen.last().expect("settings inventory"), &ways, "away-ways")
            })),
            Act::After(Box::new(move |_| {
                show_home(&config, &client, Visit::Enable);
                Call::new(&[
                    "chat",
                    "start",
                    "--project",
                    "1",
                    "--model",
                    "gpt-6-luna",
                    "--key",
                    "away-chat",
                    "Start a local away goal.",
                    "--json",
                ])
            })),
            Act::Now(Call::new(&["task", "show", "1", "--until", "replied", "--json"])),
            Act::Now(Call::new(&["chat", "show", "1", "--json"])),
            Act::After(Box::new(|seen| {
                assert!(
                    ctl::json(seen.last().expect("chat conversation"))
                        .to_string()
                        .contains("I started the local away goal."),
                    "chat starts its goal"
                );
                Call::new(&["task", "show", "2", "--json"])
            })),
            Act::After(Box::new(move |_| {
                show_home(&return_config, &return_client, Visit::Decisions);
                Call::new(&["task", "show", "2", "--until", "ended", "--json"])
            })),
            Act::Now(Call::new(&["chat", "show", "2", "--json"])),
        ],
        |_, _| (),
    );
    let transcript = ctl::json(seen.answers.last().expect("goal transcript"));
    let calls: Vec<_> = transcript["activity"]["rows"]
        .as_array()
        .expect("goal activity")
        .iter()
        .filter_map(|row| row.pointer("/event/owner/decoded/turn/decoded/messages"))
        .flat_map(|messages| messages.as_array().expect("turn messages"))
        .flat_map(|message| message["blocks"].as_array().expect("message blocks"))
        .filter_map(|block| block.pointer("/call/tool").and_then(serde_json::Value::as_str))
        .collect();
    assert!(calls.starts_with(&["decided", "note"]), "Decided carries on before overturn: {calls:?}");
    walk.scratch.remove();
}

#[derive(Clone, Copy)]
enum Visit {
    Enable,
    Decisions,
}
#[derive(Debug)]
enum Page {
    Starting,
    CarriedOn,
    Home,
    Skipped,
    Limits,
    ByHand,
    Enable,
    Enabled,
    Decided,
    Overturned,
    Off,
    Asked,
    Answered,
    Done,
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
fn press(client: &mut NativeClient, name: &[u8]) -> bool {
    let Some(button) = named(client, Role::Button, name) else { return false };
    if button.states.as_slice().contains(&State::Disabled) {
        return false;
    }
    let id = button.id;
    assert!(client.act(id, Input::Press), "press {}", String::from_utf8_lossy(name));
    true
}
fn toggle_off(client: &mut NativeClient) -> bool {
    let name = client
        .tree()
        .nodes()
        .iter()
        .find(|node| {
            node.role == Role::Button
                && node.name.as_deref().is_some_and(|name| name.starts_with(b"Away"))
                && node.states.as_slice().contains(&State::Pressed)
        })
        .and_then(|node| node.name.clone());
    name.is_some_and(|name| press(client, &name))
}
impl Referee<NativeClient> for Judge {
    #[expect(clippy::too_many_lines, reason = "one referee follows the Away switch, overturn and both Home answers")]
    fn act(&mut self, now: Time, clients: &mut [NativeClient]) {
        let client = &mut clients[0];
        match self.page {
            Page::Starting if client.started() && named(client, Role::Status, b"Owner").is_some() => match self.visit {
                Visit::Enable => {
                    client.go(Address::Home { project: 1 });
                    self.page = Page::Home;
                }
                Visit::Decisions => {
                    client.go(Address::Task { task: 2 });
                    self.page = Page::CarriedOn;
                }
            },
            Page::CarriedOn if words(client, b"I carried on with the small implementation before asking anyone.") => {
                client.go(Address::Home { project: 1 });
                self.page = Page::Home;
            }
            Page::Home => match self.visit {
                Visit::Enable if press(client, b"Skip") => self.page = Page::Skipped,
                Visit::Decisions if words(client, b"Landing: ") => self.page = Page::Decided,
                Visit::Enable | Visit::Decisions => {}
            },
            Page::Skipped if named(client, Role::Region, b"To begin").is_none() => {
                client.go(Address::Settings { project: 1 });
                self.page = Page::Limits;
            }
            Page::Limits => {
                if let Some(option) = named(client, Role::Option, b"I turn it off") {
                    let id = option.id;
                    assert!(client.act(id, Input::Press), "keep Away on until turned off by hand");
                    self.page = Page::ByHand;
                }
            }
            Page::ByHand
                if named(client, Role::Option, b"I turn it off")
                    .is_some_and(|node| node.states.as_slice().contains(&State::Selected))
                    && named(client, Role::Select, b"Away until")
                        .is_some_and(|node| !node.states.as_slice().contains(&State::Disabled)) =>
            {
                client.go(Address::Home { project: 1 });
                self.page = Page::Enable;
            }
            Page::Enable if words(client, b"Landing: ") && press(client, b"Away") => {
                self.page = Page::Enabled;
            }
            Page::Enabled if words(client, b"agents decide within") => {
                client.close();
                self.page = Page::Done;
            }
            Page::Decided if words(client, b"One thing was decided for you.") => {
                assert!(words(client, b"The small implementation fits the design."));
                assert!(words(client, b"Small"));
                assert!(
                    named(client, Role::Button, b"Try the other implementation").is_none(),
                    "Decided carries on without asking"
                );
                if press(client, b"Overturn") {
                    self.page = Page::Overturned;
                }
            }
            Page::Overturned
                if words(client, b"Overturned: it asks you again")
                    && named(client, Role::Button, b"Try the other implementation").is_some() =>
            {
                assert!(words(client, b"Still needs you"));
                if toggle_off(client) {
                    self.page = Page::Off;
                }
            }
            Page::Off if words(client, b"Away is off: questions wait for you again.") => {
                if press(client, b"Try the other implementation") {
                    self.page = Page::Asked;
                }
            }
            Page::Asked
                if named(client, Role::Button, b"Use the small implementation").is_some()
                    && words(client, b"Which local implementation while here?") =>
            {
                assert!(words(client, b"One thing was decided for you."), "off adds no decision");
                if press(client, b"Use the small implementation") {
                    self.page = Page::Answered;
                }
            }
            Page::Answered
                if words(client, b"Use the small implementation")
                    && named(client, Role::Region, b"Goals").is_none()
                    && named(client, Role::Region, b"Chats").is_some() =>
            {
                client.close();
                self.page = Page::Done;
            }
            Page::Starting
            | Page::CarriedOn
            | Page::Skipped
            | Page::ByHand
            | Page::Enable
            | Page::Enabled
            | Page::Decided
            | Page::Overturned
            | Page::Off
            | Page::Asked
            | Page::Answered
            | Page::Done => {}
        }
        assert!(
            self.passed() || now < self.until,
            "away walk stalled at {:?}; tree: {:?}",
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
    // A new host gets a fresh request-key seed, as a shell restart does.
    let seed = match visit {
        Visit::Enable => 96,
        Visit::Decisions => 97,
    };
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
            seed,
            Start::Token,
        )
        .expect("browser-bounded native client")
    });
    let ended = world.run(&clock, WITHIN);
    assert!(ended.procs.iter().all(Host::is_empty), "away walk drained");
}
