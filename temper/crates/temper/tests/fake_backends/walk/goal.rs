//! A chat's goal writes a report, reached from its Goals row on the fake box
//! (temper/docs/design/client/README.md, 5.3, 5.8, 5.13; temper/docs/design/domain/agent.md, 8.3;
//! temper/docs/design/protocol/configuration.md, 12.1).
use super::open_default::{browser_bounds, named};
use super::{Act, Call, Walk, adoption, ctl, path};
use crate::support::WITHIN;
use jig_client::contract as c;
use skein_lib::{List, Time};
use skein_shell::{Clock, Host};
use skein_ui::Role;
use skein_world::{Referee, real};
use std::path::Path;
use temper_client_domain::{Address, GoalsFilter, pages::report};
use temper_client_shell_terminal::client::{NativeClient, Settings, Start};

const ASK: &str = "Start a local goal and write its report.";
const HEADLINE: &[u8] = b"The local goal wrote its report.";

#[test]
fn a_chats_goal_on_the_fake_box_shows_its_report_in_the_goals_list() {
    let walk = Walk::new("walk-goal");
    let sandbox = walk.sandbox();
    let ways = walk.scratch.path().join("goal-ways.json");
    let seen = walk.run_then(
        vec![
            Act::After(Box::new(move |_| connect(&sandbox))),
            walk.add_key(),
            Act::Now(Call::new(&["account", "list", "--until", "usable", "--json"])),
            Act::After(Box::new(|seen| {
                assert_eq!(adoption(&seen[0])["project"], 1, "report's literal project");
                Call::new(&[
                    "chat",
                    "start",
                    "--project",
                    "1",
                    "--model",
                    "gpt-6-luna",
                    "--key",
                    "goal-chat",
                    ASK,
                    "--json",
                ])
            })),
            // The fresh box has no task before chat 1 and its first goal, 2.
            Act::Now(Call::new(&["task", "show", "1", "--until", "replied", "--json"])),
            Act::Now(Call::new(&["chat", "show", "1", "--json"])),
            Act::Now(Call::new(&["task", "show", "2", "--until", "ended", "--json"])),
            Act::Now(Call::new(&["settings", "show", "--json"])),
            Act::After(Box::new(move |seen| {
                let value = serde_json::json!({"items": [{
                    "setting": {"kind": "project-ways", "project": 1},
                    "set": {"drafts": false, "goals": true, "changes": false, "away": false, "way": 0}
                }]});
                crate::support::write_private(&ways, &serde_json::to_vec(&value).expect("project ways JSON"));
                ctl::settings_change(seen.last().expect("settings inventory"), &ways, "goal-ways")
            })),
        ],
        show_goal,
    );
    let conversation = ctl::json(&seen.answers[5]);
    assert!(
        conversation.to_string().contains("I started a goal to write its report."),
        "the chat says it called goal: {conversation}"
    );
    walk.scratch.remove();
}

fn connect(sandbox: &Path) -> Call {
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
        "goal-forge",
        "--json",
    ])
}

#[derive(Debug)]
enum Page {
    Starting,
    Rail,
    Goals,
    Goal,
    Done,
}
struct Judge {
    page: Page,
    until: Time,
}
impl Referee<NativeClient> for Judge {
    fn act(&mut self, now: Time, clients: &mut [NativeClient]) {
        let client = &mut clients[0];
        match self.page {
            Page::Starting if client.started() && named(client, Role::Status, b"Owner").is_some() => {
                client.go(Address::Home { project: 1 });
                self.page = Page::Rail;
            }
            Page::Rail if named(client, Role::Link, b"Goals").is_some() => {
                // Project ways show Goals in the rail from Home (client/README.md, 5.3).
                client.go(Address::Goals { project: 1, filter: GoalsFilter::All });
                self.page = Page::Goals;
            }
            Page::Goals if text(client, Role::Paragraph, HEADLINE) => {
                assert!(
                    text(client, Role::Heading { level: 1 }, b"1 goal, 0 under way."),
                    "the list counts the ended goal"
                );
                let row = client
                    .tree()
                    .nodes()
                    .iter()
                    .find(|node| {
                        node.role == Role::Link
                            && node.name.as_deref().is_some_and(|name| {
                                name.starts_with(b"Write a report for this local demonstration.")
                                    && name.windows(b"landed".len()).any(|part| part == b"landed")
                            })
                    })
                    .expect("the goal row names its landed state")
                    .id;
                assert!(client.act(row, skein_ui::Act::Press), "the goal row opens its page");
                self.page = Page::Goal;
            }
            Page::Goal if text(client, Role::Paragraph, b"I wrote this report on the local fake box.") => {
                assert!(text(client, Role::Heading { level: 1 }, HEADLINE), "the report headlines the goal page");
                assert!(named(client, Role::Region, b"Report").is_some(), "the goal page has its report");
                assert!(text(client, Role::Heading { level: 2 }, b"Landing"), "the goal page shows Landing");
                client.close();
                self.page = Page::Done;
            }
            Page::Starting | Page::Rail | Page::Goals | Page::Goal | Page::Done => {}
        }
        assert!(
            self.passed() || now < self.until,
            "goal walk stalled at {:?}; tree names: {:?}",
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

fn text(client: &NativeClient, role: Role, words: &[u8]) -> bool {
    client.tree().nodes().iter().enumerate().any(|(index, node)| {
        node.role == role
            && client
                .tree()
                .nodes()
                .iter()
                .skip(index)
                .take(usize::try_from(node.size).expect("node count"))
                .flat_map(|child| child.text.iter())
                .flat_map(|run| run.text.iter().copied())
                .collect::<Vec<_>>()
                == words
    })
}

fn show_goal(config: &Path, client: &Path) {
    // Read the generated script's report body, so its Left and Risk fields use the client's parser.
    let model: serde_json::Value =
        serde_json::from_slice(&std::fs::read(config.join("llm.json")).expect("public fake model document"))
            .expect("model JSON");
    let scripts = model["cued"].as_array().expect("system-cued scripts");
    let script = scripts
        .iter()
        .find(|script| script["cue"] == "You are a goal: the work your person asked for")
        .expect("the goal charter's report script");
    let note = &script["turns"][0]["lines"][0]["arguments"];
    let text = |field: &str| {
        c::DocumentText::complete(
            c::Text::new(note[field].as_str().expect("report text").as_bytes().into(), 8192).expect("bounded report"),
        )
    };
    let parsed = report::parse(&c::Note {
        number: 1,
        revision: c::Revision { value: 1 },
        scope: c::NoteScope::Task { project: 1, task: 2 },
        description: text("description"),
        body: text("body"),
        writer: c::NoteAuthor::Task { task: 2, attempt: 1 },
        tasks: List::with_capacity(0),
        changed: None,
        acts: List::with_capacity(0),
        cut: false,
    });
    assert_eq!(parsed.headline.as_ref(), HEADLINE);
    assert_eq!(parsed.left.as_deref(), Some(b"Nothing remains in this demonstration.".as_slice()));
    assert_eq!(parsed.risk.as_deref(), Some(b"This scripted model does not change a repository.".as_slice()));

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
    let mut world = real::World::new(Judge { page: Page::Starting, until: clock.now().now.saturating_add(WITHIN) });
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
    assert!(ended.procs.iter().all(Host::is_empty), "goal walk drained");
}
