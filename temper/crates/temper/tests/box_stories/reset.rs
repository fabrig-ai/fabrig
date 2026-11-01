//! A kept reset survives the box's death (draft/playground-reset.md, 6.2–6.3).
use super::{document, restart_until, rows};
use crate::{
    first_run_directories,
    support::{
        self,
        ctl::{self, Call, Step},
    },
};
use skein_io::kernel::Exit;
use skein_lib::Time;
use skein_shell::{Clock, Host};
use skein_ui::{Act, Role};
use skein_world::{Referee, real};
use std::{collections::BTreeMap, fs, path::Path};
use temper_client_domain::{Address, Step as WelcomeStep};
use temper_client_shell_terminal::client::{NativeClient, Settings, Start};

#[test]
fn a_box_killed_during_the_playgrounds_reset_ends_it_whole_at_its_next_start() {
    reset_at("removed", "Playground: removed");
}

#[test]
fn a_box_killed_as_the_reset_is_asked_ends_it_whole_at_its_next_start() {
    // The pipe's remove and the serve's ask reply are not echoed. Removal
    // suppresses the stopped notice, so its next deterministic mark is removed.
    reset_at("asked", "Playground: removed");
}

fn projects(call: &support::Ended) -> Vec<serde_json::Value> {
    ctl::json(call)["projects"].as_array().expect("person's projects").clone()
}

fn playground_project(call: &support::Ended) -> u64 {
    projects(call).iter().find(|project| project["name"]["bytes"] == "Playground")
        .expect("Playground project")["project"].as_u64().expect("project number")
}

fn running(cut: &support::cut::Cut, config: &serde_json::Value) -> (u64, u64, u64) {
    assert!(cut.calls.len() >= 5, "the running playground was read before its end");
    for call in &cut.calls[..5] {
        assert_eq!(call.exit, Some(Exit::Code(0)), "{}", String::from_utf8_lossy(&call.stderr));
    }
    assert_eq!(cut.calls[1].stdout, b"running\n");
    let forges = rows(&cut.calls[2]);
    assert_eq!(forges.len(), 1);
    assert_eq!(forges[0]["state"], "connected");
    let forge = forges[0]["number"].as_u64().expect("first forge number");
    let project = playground_project(&cut.calls[4]);
    let provider = config["application"]["demo"]["llm"]["provider"].as_u64().expect("scripted provider");
    assert!(
        ctl::json(&cut.calls[3])["accounts"]["deployment"]
            .as_array()
            .expect("accounts")
            .iter()
            .any(|account| account["provider"] == provider)
    );
    (forge, project, provider)
}

fn reset_at(name: &str, marker: &str) {
    let scratch = support::Scratch::new(&format!("playground-reset-cut-{name}"));
    let (paths, flags) = first_run_directories(&scratch);
    let cut = support::cut::at_line(
        scratch.path(),
        flags.clone(),
        scratch.path(),
        vec![
            Step::Run(Call::sign_in()),
            Step::Run(Call::new(&["playground", "start", "--key", "before-cut"])),
            Step::Run(Call::new(&["forge", "list", "--json"])),
            Step::Run(Call::new(&["account", "list", "--json"])),
            Step::Run(Call::new(&["me", "--json"])),
            Step::Run(Call::new(&["playground", "reset", "--key", "cut-reset"])),
        ],
        |line| line.ends_with(marker),
    );
    eprintln!("{name}: killed at {}", cut.line);
    let config = document(&paths[0]);
    let (first, project, provider) = running(&cut, &config);
    let fake = paths[1].join(config["application"]["demo"]["fake"]["directory"].as_str().expect("fake directory"));
    let data = paths[1].clone();
    let config_path = paths[0].clone();
    let client = scratch.path().to_path_buf();
    let before_restart = first;
    let seen = restart_until(
        scratch.path(),
        flags,
        scratch.path(),
        b"\"forges\":[]",
        vec![
            Step::Until(Call::new(&["account", "list", "--json"]), b"\"deployment\":[]"),
            Step::Run(Call::new(&["me", "--json"])),
            Step::Then(Box::new(move |calls| {
                assert!(rows(&calls[1]).is_empty(), "no forge remains after reset");
                let accounts = ctl::json(&calls[2]);
                for holder in ["party", "deployment"] {
                    assert!(
                        !accounts["accounts"][holder]
                            .as_array()
                            .expect("accounts")
                            .iter()
                            .any(|account| account["provider"] == provider),
                        "scripted account removed"
                    );
                }
                assert!(!projects(&calls[3]).iter().any(|row| row["project"] == project), "Playground project removed");
                assert!(!projects(&calls[3]).iter().any(|row| row["name"]["bytes"] == "Playground"));
                let groups = data.join("box/groups");
                if groups.exists() {
                    assert_eq!(fs::read_dir(&groups).expect("group directory").count(), 0, "no group file remains");
                }
                assert!(!fake.exists(), "the fake forge's data directory was removed");
                // The welcome waits for its playground read as well as Me,
                // Accounts and Forges: its offer proves the read is none.
                welcome_at_forge(&config_path, &client);
                Call::new(&["playground", "start", "--key", "after-reset"])
            })),
            Step::Run(Call::new(&["forge", "list", "--json"])),
            Step::Run(Call::new(&["me", "--json"])),
            Step::Then(Box::new(move |calls| {
                let forges = rows(&calls[5]);
                assert_eq!(forges.len(), 1);
                assert!(forges[0]["number"].as_u64().expect("second forge") > before_restart);
                assert_eq!(forges[0]["state"], "connected");
                let next_project = playground_project(&calls[6]);
                assert!(next_project > project, "project numbers are never reused");
                Call::new(&[
                    "chat",
                    "start",
                    "--project",
                    &next_project.to_string(),
                    "--key",
                    "csv",
                    "Add CSV export to widget",
                    "--json",
                ])
            })),
            Step::Run(Call::new(&["task", "show", "1", "--until", "replied", "--timeout", "180s", "--json"])),
            Step::Run(Call::new(&["task", "show", "2", "--until", "ended", "--timeout", "180s", "--json"])),
            Step::Run(Call::new(&["task", "show", "2"])),
        ],
        |_| csv_landed(&paths[0], scratch.path()),
    );
    assert_eq!(seen.calls.len(), 11);
    assert_eq!(seen.calls[4].stdout, b"running\n");
    assert_eq!(
        ctl::json(&seen.calls[7])["outcome"]["done"]["owner"]["created"]["first"],
        1,
        "the reset began no chat, so this is the first task"
    );
    assert_eq!(ctl::json(&seen.calls[8])["met"], true, "the chat replied with its delegate created");
    assert_eq!(ctl::json(&seen.calls[9])["met"], true, "CSV change ended before its wait timed out");
    let _ = ctl::landed_result(std::str::from_utf8(&seen.calls[10].stdout).expect("landed task"));
    scratch.remove();
}

enum Page {
    Waiting,
    Welcome,
    Forge,
    Done,
}
struct Welcome {
    page: Page,
    deadline: support::patience::Patience,
}

fn named(client: &NativeClient, role: Role, name: &[u8]) -> Option<skein_ui::NodeId> {
    client
        .tree()
        .nodes()
        .iter()
        .find(|node| node.role == role && node.name.as_deref() == Some(name))
        .map(|node| node.id)
}

impl Referee<NativeClient> for Welcome {
    fn act(&mut self, _now: Time, clients: &mut [NativeClient]) {
        let client = &mut clients[0];
        match self.page {
            Page::Waiting if client.started() && named(client, Role::Status, b"Owner").is_some() => {
                client.go(Address::Welcome { step: WelcomeStep::Welcome });
                self.page = Page::Welcome;
            }
            Page::Welcome if named(client, Role::Button, b"Try the playground").is_some() => {
                let setup = named(client, Role::Link, b"Set up temper").expect("empty welcome's setup choice");
                assert!(client.act(setup, Act::Press), "Set up temper opens Forge");
                self.page = Page::Forge;
            }
            Page::Forge
                if named(client, Role::Heading { level: 1 }, b"Connect a forge").is_some()
                    && named(client, Role::Button, b"Try the playground instead").is_some() =>
            {
                assert!(named(client, Role::Status, "Leaving the playground…".as_bytes()).is_none());
                client.close();
                self.page = Page::Done;
            }
            Page::Waiting | Page::Welcome | Page::Forge | Page::Done => {}
        }
    }
    fn observe(&mut self, _now: Time, clients: &[NativeClient]) {
        if !matches!(self.page, Page::Done)
            && let Some(figures) = self.deadline.failed()
        {
            panic!(
                "reset welcome did not reach Forge: {:?}; {figures}",
                clients[0]
                    .tree()
                    .nodes()
                    .iter()
                    .filter_map(|node| node.name.as_deref())
                    .map(String::from_utf8_lossy)
                    .collect::<Vec<_>>()
            );
        }
    }
    fn next_deadline(&self) -> Option<Time> {
        if matches!(self.page, Page::Done) { None } else { self.deadline.until() }
    }
    fn passed(&self) -> bool {
        matches!(self.page, Page::Done)
    }
}

fn welcome_at_forge(config: &Path, client: &Path) {
    let config = document(config);
    let engine = config["application"]["clients"]["listener"]["socket"]
        .as_str()
        .expect("client address")
        .parse()
        .expect("socket");
    let root = skein_shell::open_root(client).expect("signed-in client's root");
    let clock = Clock::new();
    let mut world = real::World::new(Welcome {
        page: Page::Waiting,
        deadline: support::patience::Patience::arm("the reset welcome's Forge step", support::boxed::BOX_DEADLINE),
    });
    world.spawn_with_fds(vec![root], || {
        NativeClient::new(
            Settings {
                engine: skein_io::Addr::Ip(engine),
                directory: b"client".as_slice().into(),
                limits: temper_client_shell_terminal::limits::native(),
            },
            root,
            skein_shell::effective_user(),
            93,
            Start::Token,
        )
        .expect("native welcome client")
    });
    let ended = world.run(&clock, support::boxed::BOX_DEADLINE);
    assert!(ended.procs.iter().all(Host::is_empty), "welcome client drained");
}

fn tree(object: &serde_json::Value) -> BTreeMap<Vec<u8>, Vec<u8>> {
    let decode = |value: &serde_json::Value| {
        skein_lib::base64::decode(
            skein_lib::base64::Alphabet::Standard,
            value.as_str().expect("base64 file").as_bytes(),
            8192,
        )
        .expect("file bytes")
        .into_vec()
    };
    object["tree"]
        .as_array()
        .expect("commit tree")
        .iter()
        .map(|file| (decode(&file["path"]), decode(&file["content"])))
        .collect()
}

fn csv_landed(config: &Path, client: &Path) {
    let read = ctl::alone(client, Call::new(&["task", "show", "2"]));
    assert_eq!(read.exit, Some(Exit::Code(0)));
    let task = std::str::from_utf8(&read.stdout).expect("change read");
    let (_, hex) = ctl::landed_result(task);
    let commit = temper_worker_protocol_git::invocation::object(hex.as_bytes()).expect("merge hex");
    let merge = skein_fake_forge_protocol::routes::commit(&skein_forge::forgejo::id::ObjectId {
        format: skein_forge::forgejo::id::ObjectFormat::Sha1,
        bytes: commit.raw(),
    })
    .expect("fake commit number");
    let config = document(config);
    let address = config["application"]["demo"]["fake"]["mode"]["fake"]["listen"]["socket"]
        .as_str()
        .expect("fake listener")
        .parse()
        .expect("socket");
    let landed = support::fake_objects(address, merge);
    let landed = &landed["objects"][0];
    assert_eq!(landed["commit"], merge);
    assert_eq!(landed["parent"], 1, "the restarted fake's seeded head");
    let mut expected: BTreeMap<Vec<u8>, Vec<u8>> =
        temper_fake::forge::FILES.into_iter().map(|(path, bytes)| (path.to_vec(), bytes.to_vec())).collect();
    let seed = support::fake_objects(address, 1);
    assert_eq!(tree(&seed["objects"][0]), expected);
    let script = config["application"]["demo"]["llm"]["cued"]
        .as_array()
        .expect("scripts")
        .iter()
        .find(|script| script["cue"] == "## Task\n\nExport widget's table as CSV")
        .expect("CSV script");
    let edit = &script["turns"][1]["lines"][0]["arguments"]["edits"][0];
    let old = edit["old"].as_str().expect("old main");
    let new = edit["new"].as_str().expect("new main");
    let main = expected.get_mut(b"src/main.rs".as_slice()).expect("main source");
    let source = std::str::from_utf8(main).expect("UTF-8 source");
    assert_eq!(source.matches(old).count(), 1);
    *main = source.replacen(old, new, 1).into_bytes();
    assert_eq!(tree(landed), expected, "CSV script landed on the newly numbered forge");
}
