//! Copies through the shipped clients and fake box (temper/docs/design/domain/forge.md, 4.5).
use super::{Act, Call, Walk, adoption, ctl, path};
use std::path::Path;

fn document(sandbox: &Path) -> temper_fake::forge::Document {
    temper_fake::forge::Document::decode(
        &std::fs::read(sandbox.join("config/forgejo.json")).expect("retained fake Forgejo document"),
    )
    .expect("fake Forgejo document")
}

fn commit(count: u64) -> String {
    String::from_utf8(temper_fake_git::id::id(count).to_vec()).expect("fake commit spelling")
}

fn api(sandbox: &Path, route: &str) -> Call {
    let document = document(sandbox);
    // curl reads the private token file itself; neither argv nor captured output carries it.
    Call::command(
        Path::new("/usr/bin/curl"),
        &[
            "--silent",
            "--show-error",
            "--fail",
            "--noproxy",
            "*",
            "--variable",
            &format!("token@{}", String::from_utf8_lossy(document.admin_token())),
            "--expand-header",
            "Authorization: token {{token:trim}}",
            &format!("http://{}/api/v1/repos/{route}", document.listen()),
        ],
    )
}

#[test]
fn a_copy_is_made_and_adopted_through_ctl_on_the_fake_box() {
    let walk = Walk::new("walk-copy");
    let sandbox = walk.sandbox();
    let connect = sandbox.clone();
    let copy = sandbox.clone();
    let history = sandbox.clone();
    let copied_branch = sandbox.clone();
    let metadata = sandbox.clone();
    let workflow = sandbox.clone();
    let github = sandbox.clone();
    let seen = walk.run(vec![
        Act::After(Box::new(move |_| {
            let document = document(&connect);
            let address = document.listen().to_string();
            Call::new(&[
                "forge",
                "connect",
                "forgejo-admin",
                path(&connect.join("data/forgejo-admin.token")),
                "--api",
                &address,
                "--git",
                &format!("http://{address}"),
                "--first",
                "work/first",
                "--label",
                "copy-runner",
                "--key",
                "copy-forge",
                "--json",
            ])
        })),
        Act::After(Box::new(move |seen| {
            let connected = adoption(seen.last().expect("connected first repository"));
            let project = connected["project"].as_u64().expect("first repository's project").to_string();
            let forge = super::adopted_forge(seen.last().expect("connected forge")).to_string();
            let document = document(&copy);
            let source = document.sources().first().expect("copy source");
            Call::new(&[
                "repository",
                "copy",
                "--project",
                &project,
                "--forge",
                &forge,
                std::str::from_utf8(&source.address).expect("source address"),
                "--as",
                "work/widget",
                "--no-home",
                "--until",
                "adopted",
                "--key",
                "copy-widget",
                "--json",
            ])
        })),
        Act::Now(Call::new(&["forge", "list", "--json"])),
        Act::After(Box::new(move |_| api(&copied_branch, "work/widget/branches/main"))),
        Act::After(Box::new(move |_| api(&metadata, "work/widget"))),
        Act::After(Box::new(move |seen| {
            let document = document(&history);
            let source = document.sources().first().expect("copy source");
            let (forge, _) = document.seed(7);
            let base = forge.domain.branch(&source.repository, &source.landing).expect("source head");
            let base = commit(base);
            let tip = ctl::json(seen.get(3).expect("copied branch"))["commit"]["id"]
                .as_str()
                .expect("copied head")
                .to_owned();
            api(&history, &format!("work/widget/compare/{base}...{tip}?files=false"))
        })),
        Act::After(Box::new(move |seen| {
            api(
                &workflow,
                &format!(
                    "work/widget/raw/.forgejo/workflows/ci.yml?ref={}",
                    ctl::json(seen.get(3).expect("copied branch"))["commit"]["id"].as_str().expect("copied head")
                ),
            )
        })),
        Act::After(Box::new(move |seen| {
            api(
                &github,
                &format!(
                    "work/widget/raw/.github/workflows/ci.yml?ref={}",
                    ctl::json(seen.get(3).expect("copied branch"))["commit"]["id"].as_str().expect("copied head")
                ),
            )
        })),
        Act::After(Box::new(|seen| {
            let project = adoption(&seen[0])["project"].as_u64().expect("first project").to_string();
            Call::new(&["project", "show", &project, "--json"])
        })),
    ]);
    assert_copy(&seen, &sandbox);
    walk.scratch.remove();
}

fn assert_copy(seen: &super::Seen, sandbox: &Path) {
    let [_, copied, forges, branch, metadata, history, workflow, github, project] = seen.answers.as_slice() else {
        panic!("connect, copy, adopted forge and API reads")
    };
    assert_eq!(adoption(copied)["adopted"], true);
    let forge = super::only_forge(forges);
    assert!(forge["makings"].as_array().expect("making rows").is_empty());
    let project = ctl::json(project);
    assert_eq!(project["number"], adoption(&seen.answers[0])["project"]);
    let resources = project["resources"].as_array().expect("adopted project resources");
    assert_eq!(resources.len(), 2, "the first repository and the copy");
    // Project resources name the forge and provider id as big-endian hex segments.
    let provider =
        u32::try_from(ctl::json(metadata)["id"].as_u64().expect("copy's provider id")).expect("provider id fits");
    let resource = serde_json::json!([format!("{:04x}", super::number(&forge)), format!("{provider:08x}")]);
    let repository = resources
        .iter()
        .find(|row| row["part"]["key"]["path"] == resource)
        .expect("the copy's provider id is adopted into the first repository's project");
    assert_eq!(repository["role"], "owned");
    assert_eq!(repository["home"], false);
    assert_eq!(ctl::json(metadata)["default_branch"], "main");
    let history = ctl::json(history);
    assert_eq!(history["total_commits"], 1, "one workflow commit above the source");
    assert_eq!(history["commits"][0]["sha"], ctl::json(branch)["commit"]["id"]);
    let document = document(sandbox);
    let (forge, _) = document.seed(7);
    let source = document.sources().first().expect("source");
    let base = forge.domain.branch(&source.repository, &source.landing).expect("source head");
    assert_eq!(history["commits"][0]["parents"][0]["sha"], commit(base));
    assert_eq!(ctl::json(branch)["name"], "main");
    assert_eq!(
        workflow.stdout.as_slice(),
        temper_forge_setup::starter_workflow(b"copy-runner").expect("starter workflow").as_ref(),
        "the starter is read through the fake API"
    );
    let original = source
        .tree
        .iter()
        .find(|file| file.path.as_ref() == b".github/workflows/ci.yml")
        .expect("source's GitHub workflow");
    assert_eq!(github.stdout.as_slice(), original.content.as_ref(), "the source workflow is preserved");
}

/// Ready's project, then its ordered copy requests in one client pass
/// (temper/docs/design/client/README.md, 3.5; temper/docs/design/draft/repository-copies.md, 7).
#[test]
fn readys_two_copies_are_adopted_in_order_with_the_first_as_home() {
    let walk = Walk::new("walk-ready-copies");
    // Catalogue objects represent another host, so store them apart from Ready's destinations.
    walk.run(Vec::new());
    let catalogue = walk.sandbox().join("config/forgejo.json");
    let mut fixture: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&catalogue).expect("catalogue document")).expect("JSON");
    for source in fixture["sources"].as_array_mut().expect("catalogue sources") {
        source["repository"]["owner"] = "catalogue".into();
    }
    super::super::write(&catalogue, &serde_json::to_vec(&fixture).expect("catalogue JSON"));
    let connect = walk.sandbox();
    walk.run_then(
        vec![
            Act::After(Box::new(move |_| {
                let address = document(&connect).listen().to_string();
                Call::new(&[
                    "forge",
                    "connect",
                    "forgejo-admin",
                    path(&connect.join("data/forgejo-admin.token")),
                    "--api",
                    &address,
                    "--git",
                    &format!("http://{address}"),
                    "--key",
                    "copy-forge",
                    "--json",
                ])
            })),
            Act::Now(Call::new(&["forge", "list", "--until", "connected:1", "--json"])),
        ],
        |config, client| ready_copies(config, client, &walk.sandbox()),
    );
    walk.scratch.remove();
}

use jig_client::{self as client, contract as c};
use jig_client_shell_native::Shell;
use skein_io::kernel::{Complete, Submit};
use skein_lib::{List, Queue, Time, Wall};
use skein_shell::{Clock, Host};
use skein_world::{Referee, real};
use temper_client_domain::forge::{self, forges};
use temper_client_protocol_wire::v1 as w;

const ME: client::Handle = client::Handle { asker: client::Asker::JIG, number: 0 };
const FORGES: client::Handle = client::Handle { asker: client::Asker::JIG, number: 1 };

/// The shipped client transport, with the same request burst as Ready's act.
struct Copies {
    shell: Shell,
    client: client::Client,
    out: Queue<client::Request>,
    limits: temper_client_domain::Limits,
    addresses: Vec<Box<[u8]>>,
    project: Option<u32>,
    creating: bool,
    asked: bool,
    answered: usize,
    snapshots: Vec<Box<[forges::Row]>>,
    closing: bool,
}
impl Copies {
    fn event(&mut self, now: Time, wall: Wall, event: client::Event) {
        self.client.step(now, wall, event, &mut self.out);
        self.drain();
    }
    #[expect(
        clippy::wildcard_enum_match_arm,
        reason = "the walk inspects its answers and forwards other transport outputs"
    )]
    fn drain(&mut self) {
        while let Some(request) = self.out.pop() {
            match request {
                client::Request::Told { told: client::Told::Answered { key, outcome } } => {
                    if key.number == 1 {
                        let client::Outcome::Answered(client::Answered::Jig(c::Answer::Done(c::Outcome::Document(
                            c::DocumentOutcome::ProjectMade { project },
                        )))) = outcome
                        else {
                            panic!("Ready's MakeProject refused: {outcome:?}")
                        };
                        assert_eq!(project, 1);
                        self.project = Some(project);
                    } else {
                        let client::Outcome::Answered(client::Answered::Family(record)) = outcome else {
                            panic!("Ready's copy refused: {outcome:?}")
                        };
                        let limits = forge::wire_limits(&self.limits.forge).expect("forge wire limits");
                        let mut reader = skein_lib::Reader::new(&record.bytes);
                        let answer = w::Answer::decode(&limits, &mut reader).expect("copy answer");
                        assert!(reader.is_empty());
                        assert_eq!(answer.outcome(), &w::Outcome::MakeKept);
                        self.answered += 1;
                    }
                }
                request => self.shell.down(request),
            }
        }
    }
    fn send_copies(&mut self, now: Time, wall: Wall, project: u32) {
        self.asked = true;
        for index in 0..self.addresses.len() {
            let address = &self.addresses[index];
            let (org, name) = forge::making::target(address).expect("catalogue target");
            let into = w::MakeInto::new(&w::CEILINGS, w::MakeIntoParts { project, home: index == 0 }).expect("project");
            let make = forge::making::ask(1, &org, &name, Some(address), Some(into)).expect("Ready's copy");
            let ordinal = u64::try_from(index + 1).expect("two copy keys");
            self.event(
                now,
                wall,
                client::Event::Send {
                    key: client::Key { asker: client::Asker::JIG, number: ordinal + 1 },
                    name: Some(forge::copy_key(ordinal)),
                    request: forge::make_repository(&make, &self.limits.forge).expect("copy request"),
                },
            );
        }
    }
    fn rows(&mut self) {
        let Some(client::Latest::Family(record)) = self.client.document(FORGES).and_then(|doc| doc.latest) else {
            return;
        };
        let rows = forges::rows(record, &self.limits.forge).expect("Home's forge rows");
        if self.snapshots.last() != Some(&rows) {
            eprintln!("Ready copies watch: {rows:?}");
            self.snapshots.push(rows.clone());
        }
        let [row] = rows.as_ref() else { panic!("one connected forge") };
        assert_eq!(row.number, 1);
        assert!(
            row.makings.iter().filter(|making| making.state == w::MakingState::Making).count() <= 1,
            "one writing run per forge"
        );
        for making in &row.makings {
            assert!(!matches!(making.state, w::MakingState::Failed(_)), "copy failed: {making:?}");
            let index = self
                .addresses
                .iter()
                .position(|address| forge::making::is_target(making, address))
                .expect("Home can match each making row to its requested source");
            assert_eq!(making.into.as_ref().expect("copy destination").project(), 1);
            assert_eq!(making.into.as_ref().expect("copy destination").home(), index == 0);
        }
        let all_in = self.addresses.iter().all(|address| {
            let (org, name) = forge::making::target(address).expect("catalogue target");
            row.repositories.iter().any(|repo| repo.owner == org && repo.name == name && repo.project == 1)
        });
        if self.answered == 2 && row.makings.is_empty() && all_in {
            self.closing = true;
            self.shell.close();
        }
    }
}
impl Host for Copies {
    fn iterate(&mut self, now: Time, wall: Wall) {
        self.shell.iterate(now, wall);
        if self.closing {
            while self.shell.events().pop().is_some() {}
            return;
        }
        if let Some((credential, records)) = self.shell.loaded() {
            let mut kept = List::with_capacity(u32::try_from(records.len()).expect("kept requests"));
            for record in records {
                kept.push(record).expect("request room");
            }
            self.event(now, wall, client::Event::Start { kept });
            self.event(
                now,
                wall,
                client::Event::SignIn { with: client::With::Held(credential.expect("signed-in client")) },
            );
            self.event(now, wall, client::Event::Watch { handle: ME, subject: client::Watchable::Jig(c::Subject::Me) });
            self.event(
                now,
                wall,
                client::Event::Watch {
                    handle: FORGES,
                    subject: forges::watch(&self.limits.forge).expect("forges watch"),
                },
            );
        }
        while let Some(event) = self.shell.events().pop() {
            self.event(now, wall, event);
        }
        self.client.fire(now, wall, &mut self.out);
        self.drain();
        self.client.reclaim();
        if !self.creating
            && self.client.link().state == client::Link::Live
            && self.client.document(ME).is_some_and(|doc| doc.state == client::State::Live)
        {
            self.creating = true;
            self.event(
                now,
                wall,
                client::Event::Send {
                    key: client::Key { asker: client::Asker::JIG, number: 1 },
                    name: None,
                    request: client::Ask::Jig(Box::new(c::Ask::MakeProject {
                        name: c::Text::new(b"Widget".as_slice().into(), 64).expect("project name"),
                        policy: 1,
                    })),
                },
            );
        }
        if let Some(project) = self.project
            && !self.asked
        {
            self.send_copies(now, wall, project);
        }
        self.rows();
    }
    fn completions(&mut self) -> &mut Queue<Complete> {
        self.shell.completions()
    }
    fn submissions(&mut self) -> &mut Queue<Submit> {
        self.shell.submissions()
    }
    fn work_pending(&self, now: Time) -> bool {
        self.shell.work_pending(now) || (!self.closing && self.client.next_deadline().is_some_and(|at| at <= now))
    }
    fn next_mechanism_deadline(&self) -> Option<Time> {
        self.shell.next_deadline()
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        if self.closing { None } else { self.client.next_deadline() }
    }
    fn is_empty(&self) -> bool {
        self.closing && self.shell.is_empty()
    }
    fn operations(&self) -> u32 {
        self.shell.operations()
    }
}
struct Judge {
    until: Time,
    done: bool,
}
impl Referee<Copies> for Judge {
    fn act(&mut self, now: Time, clients: &mut [Copies]) {
        self.done = clients.iter().all(Host::is_empty);
        assert!(self.done || now < self.until, "Ready's copy requests did not adopt: {:?}", clients[0].snapshots);
    }
    fn observe(&mut self, _now: Time, clients: &[Copies]) {
        self.done = clients.iter().all(Host::is_empty);
    }
    fn passed(&self) -> bool {
        self.done
    }
    fn next_deadline(&self) -> Option<Time> {
        (!self.done).then_some(self.until)
    }
}
fn ready_copies(config: &Path, client: &Path, sandbox: &Path) {
    let document = document(sandbox);
    let addresses: Vec<_> = document.sources().iter().map(|source| source.address.clone()).collect();
    assert_eq!(addresses.len(), 2);
    let settings: serde_json::Value =
        serde_json::from_slice(&std::fs::read(config.join("temper.json")).expect("configuration")).expect("JSON");
    let engine = settings
        .pointer("/application/clients/listener/socket")
        .and_then(serde_json::Value::as_str)
        .expect("clients listener")
        .parse()
        .expect("socket");
    let root = skein_shell::open_root(client).expect("signed-in client's parent");
    let clock = Clock::new();
    let bounds = temper_client_shell_terminal::limits::native();
    let mut world =
        real::World::new(Judge { until: clock.now().now.saturating_add(crate::support::WITHIN), done: false });
    world.spawn_with_fds(vec![root], || {
        let mut shell = Shell::new(jig_client_shell_native::Settings {
            engine: skein_io::Addr::Ip(engine),
            directory: b"client".as_slice().into(),
            limits: bounds.connections,
            io: bounds.io,
        });
        shell.adopt(root, skein_shell::effective_user(), 716);
        Copies {
            shell,
            client: client::Client::new(&bounds.client.client, 716).expect("client bounds"),
            out: Queue::uncapped(),
            limits: bounds.client,
            addresses,
            project: None,
            creating: false,
            asked: false,
            answered: 0,
            snapshots: Vec::new(),
            closing: false,
        }
    });
    let ended = world.run(&clock, crate::support::WITHIN);
    let copies = &ended.procs[0];
    assert!(copies.is_empty());
    assert!(
        copies.snapshots.iter().any(|rows| {
            let makings = &rows[0].makings;
            makings.iter().any(|making| {
                forge::making::is_target(making, &copies.addresses[0]) && making.state == w::MakingState::Making
            }) && makings.iter().any(|making| {
                forge::making::is_target(making, &copies.addresses[1]) && making.state == w::MakingState::Waiting
            })
        }),
        "the second address waits while the first holds the forge's writing run"
    );
    let project = ctl::alone(client, Call::new(&["project", "show", "1", "--json"]));
    let project = ctl::json(&project);
    let resources = project["resources"].as_array().expect("project repositories");
    assert_eq!(resources.len(), 2, "both copies adopted into Ready's project");
    for (index, source) in document.sources().iter().enumerate() {
        let (org, name) = forge::making::target(&source.address).expect("copy target");
        let metadata = ctl::alone(
            client,
            api(
                sandbox,
                &format!(
                    "{}/{}",
                    std::str::from_utf8(&org).expect("org"),
                    std::str::from_utf8(&name).expect("repository")
                ),
            ),
        );
        let provider =
            u32::try_from(ctl::json(&metadata)["id"].as_u64().expect("copy provider id")).expect("provider id fits");
        let key = serde_json::json!(["0001", format!("{provider:08x}")]);
        let resource = resources.iter().find(|resource| resource["part"]["key"]["path"] == key).expect("copy adopted");
        assert_eq!(resource["role"], "owned");
        assert_eq!(resource["home"], index == 0, "the first address alone is the project's home");
        assert_eq!(
            ctl::json(&metadata)["default_branch"],
            std::str::from_utf8(&source.landing).expect("landing branch")
        );
    }
}
