//! A deployment at its place restarted with a newer build (testing/process-tier.md).
//! Copies and file identities follow testing/qa/places.md, 4 and qa/deployment.md, 8.
mod session;
use super::newer_build::{Identity, same_bytes};
use crate::support::{self, boxed, parent::Guard};
use serde_json::{Value, json};
use std::{
    fs::{self, File},
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::Command,
};

const PROGRAMS: [(&str, &str); 7] = [
    ("temper-box", "temper-box"),
    ("smith-fake-claude", "fake-claude"),
    ("temper", "temper"),
    ("temper-client-journeys", "temper-client-journeys"),
    ("temper-engine", "temper-engine"),
    ("temper-worker", "temper-worker"),
    ("temper-agent", "temper-agent"),
];

const LIVE: [(&str, &str); 5] = [
    ("temper-client-journeys", "drive"),
    ("temper-engine", "serve"),
    ("temper-worker", "serve"),
    ("temper-box", "forge"),
    ("temper-box", "llm"),
];

struct Place {
    root: support::Scratch,
    scratch: PathBuf,
    bin: PathBuf,
    bundle: PathBuf,
    setups: PathBuf,
    ports: [std::net::SocketAddr; 9],
}
impl Place {
    fn new() -> Self {
        let root = support::Scratch::new("deployment-newer-build");
        let bin = root.path().join("bin");
        let scratch = root.path().join("deployment");
        let bundle = root.path().join("bundle");
        super::bundle(&bundle, super::SOURCE_PAGE);
        super::private(&bin);
        super::private(&scratch);
        let ports = support::free_addresses(&[]);
        let place = Self {
            root,
            scratch,
            bin,
            bundle,
            setups: skein_scratch::paths::tree().join("docs/development/qa/setups"),
            ports,
        };
        place.copy();
        place
    }

    fn copy(&self) {
        for (package, name) in PROGRAMS {
            let source = if name == "temper-box" {
                skein_scratch::paths::binary(name)
            } else {
                skein_scratch::paths::built(package, name)
            };
            let target = self.bin.join(name);
            fs::copy(source, &target).expect("install deployment binary");
            fs::set_permissions(target, fs::Permissions::from_mode(0o700)).expect("private executable");
        }
    }

    fn arguments(&self, drive: bool, restored: bool) -> Vec<String> {
        let mut arguments = if drive {
            vec![
                "drive".into(),
                "--ctl".into(),
                "--deployment".into(),
                self.setups.join("turn-landing/config").display().to_string(),
            ]
        } else {
            vec![
                "recipe".into(),
                "walk".into(),
                "turn-landing".into(),
                "--setups".into(),
                self.setups.display().to_string(),
            ]
        };
        arguments.extend([
            "--bundle".into(),
            self.bundle.display().to_string(),
            "--bin".into(),
            self.bin.display().to_string(),
            "--scratch".into(),
            self.scratch.display().to_string(),
            "--ports".into(),
            self.ports.map(|address| address.port().to_string()).join(","),
            "--strict".into(),
        ]);
        if restored {
            arguments.extend(["--from".into(), "turn-landing".into()]);
        }
        arguments
    }

    fn walk(&self, restored: bool) -> String {
        let seen = session::run(self, self.arguments(false, restored), false, &[]);
        let hash = seen
            .lines
            .last()
            .expect("walk's last line")
            .strip_prefix("walked turn-landing ")
            .expect("walked turn-landing HASH");
        assert_eq!(hash.len(), 64, "chain hash length");
        assert!(hash.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)), "chain hash");
        if !restored {
            assert!(seen.lines.iter().any(|line| line == "turn-landing:7 ok"), "walk restarted engine and worker");
        }
        hash.into()
    }

    fn read(&self, old: &[Identity]) -> (Kept, File) {
        let seen = session::run(self, self.arguments(true, false), true, old);
        assert_eq!(seen.lines.last().map(String::as_str), Some("ended"));
        (seen.kept.expect("ctl and API reads completed"), seen.pin.expect("worker pinned its agent"))
    }

    fn restore(&self) {
        let archive = self.root.path().join("copy.tar");
        super::write(&archive, b"");
        for create in [true, false] {
            let mut command = Command::new("tar");
            command
                .arg(if create { "--create" } else { "--extract" })
                .arg("--file")
                .arg(&archive)
                .arg("--directory")
                .arg(&self.scratch);
            if create {
                command.args(["config", "data"]);
            }
            let output = support::story_command::standalone(&mut command, "the deployment's archive", support::WITHIN);
            no_link(&output.stdout);
            no_link(&output.stderr);
            assert!(output.status.success(), "tar: {}", String::from_utf8_lossy(&output.stderr));
            if create {
                for name in ["config", "data"] {
                    fs::remove_dir_all(self.scratch.join(name)).expect("remove stopped deployment before restore");
                    assert!(!self.scratch.join(name).exists());
                }
            }
        }
    }

    /// Keep every old inode alive while new files are installed at the same paths.
    fn replace(&self) -> Vec<File> {
        let kept: Vec<_> =
            PROGRAMS.iter().map(|(_, name)| File::open(self.bin.join(name)).expect("old binary")).collect();
        for (_, name) in PROGRAMS {
            fs::remove_file(self.bin.join(name)).expect("unlink stopped binary");
        }
        self.copy();
        for ((_, name), file) in PROGRAMS.iter().zip(&kept) {
            assert_ne!(Identity::of(&self.bin.join(name)), Identity::metadata(&file.metadata().expect("old inode")));
        }
        kept
    }

    fn pin(&self) -> PathBuf {
        self.scratch.join("data/worker/workspaces/agent/temper-agent")
    }

    fn processes(&self, old: &[Identity]) -> File {
        assert!(!self.scratch.join("session/link").exists(), "ctl's sign-in file was removed before ready");
        let pin = self.pin();
        let pinned = File::open(&pin).expect("worker's pinned agent");
        assert!(!old.contains(&Identity::metadata(&pinned.metadata().expect("pin identity"))), "no old agent pin");
        assert!(
            same_bytes(
                pinned.try_clone().expect("read pin"),
                File::open(self.bin.join("temper-agent")).expect("new agent")
            ),
            "pin has new agent bytes"
        );
        let guard = Guard::observing_test();
        let root = boxed::find(&guard.pids(), &self.bin.join("temper-client-journeys"), b"drive")
            .expect("installed drive session");
        let mut pids = vec![root];
        let mut index = 0;
        while let Some(&pid) = pids.get(index) {
            pids.extend(guard.children(pid));
            index += 1;
        }
        let identities = PROGRAMS.map(|(_, name)| Identity::of(&self.bin.join(name)));
        let mut alive = Vec::new();
        for pid in pids {
            let Some(executable) = guard.executable(pid).unwrap_or_else(|why| panic!("PID {pid}: executable: {why}"))
            else {
                continue;
            };
            let path = &executable.path;
            assert!(!path.as_os_str().as_encoded_bytes().ends_with(b" (deleted)"), "PID {pid}: deleted executable");
            let name = path.file_name().expect("executable name").to_string_lossy();
            assert!(!name.contains("forgejo") && !name.contains("runner"), "PID {pid}: Forgejo or runner");
            let identity = Identity::metadata(&executable.file.metadata().expect("observed executable identity"));
            assert!(!old.contains(&identity), "PID {pid}: old executable");
            if *path == pin {
                assert_eq!(identity, Identity::of(&pin));
                assert!(
                    same_bytes(executable.file, File::open(self.bin.join("temper-agent")).expect("new agent")),
                    "PID {pid}: agent pin bytes"
                );
            } else {
                assert!(identities.contains(&identity), "PID {pid}: executable outside new bin: {}", path.display());
            }
            let argument = |word: &[u8]| executable.arguments.split(|byte| *byte == 0).any(|arg| arg == word);
            for (program, word) in LIVE {
                if *path == self.bin.join(program) && argument(word.as_bytes()) {
                    alive.push((program, word));
                }
            }
        }
        for required in LIVE {
            assert!(alive.contains(&required), "{} {} must be read alive", required.0, required.1);
        }
        pinned
    }

    fn listeners(&self) {
        let engine = document(&self.scratch.join("config/engine/temper.json"));
        let worker = document(&self.scratch.join("config/worker/temper.json"));
        let agent = document(&self.scratch.join("data/worker/workspaces/agent/agent.json"));
        let model = document(&self.scratch.join("config/model.json"));
        let forge = document(&self.scratch.join("config/forgejo.json"));
        for (socket, port) in [
            (&engine["application"]["clients"]["listener"]["socket"], 0),
            (&engine["engine"]["hosts"]["listener"][0]["socket"], 1),
            (&worker["workers"][0]["engine"]["socket"], 1),
            (&engine["application"]["webhooks"]["listener"]["socket"], 4),
            (&model["listen"]["socket"], 7),
            (&agent["endpoints"][0]["address"], 7),
            (&engine["engine"]["llm"]["endpoints"][0]["address"]["socket"], 7),
            (&forge["listen"]["socket"], 8),
        ] {
            assert_eq!(socket.as_str().expect("rendered listener"), self.ports[port].to_string());
        }
        assert!(worker["workers"][0].get("agent").is_none(), "the engine supplies the worker's agent figures");
        assert_eq!(
            fs::read_link(self.scratch.join("data/bin/git")).expect("fake git link"),
            self.bin.join("temper-box")
        );
    }

    fn api(&self, dispatch: bool) -> Value {
        let forge = document(&self.scratch.join("config/forgejo.json"));
        let address = forge["listen"]["socket"].as_str().expect("fake Forgejo socket");
        let header = self.scratch.join("session/person.header");
        // Only the fake's file is opened; no token enters argv, output, or kept facts.
        let token = fs::read_to_string(forge["admin_token"].as_str().expect("fake admin token file"))
            .expect("fake admin token");
        support::write_private(&header, format!("Authorization: token {}\n", token.trim_end()).as_bytes());
        assert_eq!(fs::metadata(&header).expect("header").permissions().mode() & 0o777, 0o600);
        let get = |route: &str| {
            let (status, value) = support::first_repository::header_request(
                &format!("http://{address}/api/v1"),
                header.to_str().expect("header path"),
                "GET",
                route,
                None,
            );
            let bytes = serde_json::to_vec(&value).expect("API JSON");
            no_link(&bytes);
            assert_eq!(status, 200, "GET {route}: {value}");
            value
        };
        let repository = get("/repos/work/hello");
        assert_eq!(repository["owner"]["login"], "work");
        assert_eq!(repository["name"], "hello");
        let branch = get("/repos/work/hello/branches/main");
        let head = branch["commit"]["id"].as_str().expect("main head");
        if dispatch {
            let output = support::story_command::output(
                Command::new("curl")
                    .args(["--silent", "--show-error", "--fail", "--noproxy", "*", "--header"])
                    .arg(format!("@{}", header.display()))
                    .args([
                        "--request",
                        "POST",
                        "--header",
                        "Content-Type: application/json",
                        "--data",
                        r#"{"ref":"main","inputs":{}}"#,
                    ])
                    .arg(format!("http://{address}/api/v1/repos/work/hello/actions/workflows/gate.yml/dispatches")),
                "the fake gate dispatch",
                support::WITHIN,
                None,
            );
            no_link(&output.stdout);
            no_link(&output.stderr);
            assert!(output.status.success(), "dispatch gate.yml: {}", String::from_utf8_lossy(&output.stderr));
            assert!(output.stdout.is_empty(), "dispatch has no body");
        }
        let route = format!("/repos/work/hello/actions/runs?workflow_id=gate.yml&head_sha={head}");
        let mut patience = support::patience::Patience::arm("the fake gate's successful run", support::WITHIN);
        let run = loop {
            let runs = get(&route);
            let [run] = runs["workflow_runs"].as_array().expect("workflow runs").as_slice() else {
                panic!("one persisted gate run: {runs}")
            };
            if run["status"] == "success" {
                break get(&format!("/repos/work/hello/actions/runs/{}", run["id"].as_u64().expect("run id")));
            }
            assert!(
                dispatch && (run["status"] == "running" || run["status"] == "waiting"),
                "gate.yml did not succeed: {run}"
            );
            if let Some(figures) = patience.failed() {
                panic!("gate.yml did not succeed: {run}; {figures}");
            }
            std::thread::sleep(std::time::Duration::from_millis(1));
        };
        assert_eq!(run["workflow_id"], "gate.yml");
        assert_eq!(run["commit_sha"], head);
        assert_eq!(run["status"], "success");
        let jobs = get(&format!("/repos/work/hello/actions/runs/{}/jobs", run["id"].as_u64().expect("run id")));
        let [job] = jobs.as_array().expect("run jobs").as_slice() else { panic!("one gate job") };
        assert_eq!(job["name"], "gate");
        assert_eq!(job["status"], "success");
        fs::remove_file(header).expect("remove private API header");
        json!({"repository":repository,"run":run,"jobs":jobs})
    }

    fn logs(&self) {
        for name in ["forgejo", "model", "engine", "worker", "box"] {
            let bytes = fs::read(self.scratch.join(format!("session/{name}.log"))).expect("deployment process log");
            no_link(&bytes);
        }
        no_link(&fs::read(self.scratch.join("session/lines")).expect("kept drive lines"));
        assert!(!self.scratch.join("session/link").exists(), "no sign-in link kept");
    }
}

fn document(path: &Path) -> Value {
    serde_json::from_slice(&fs::read(path).expect("rendered deployment document")).expect("deployment JSON")
}
fn no_link(bytes: &[u8]) {
    assert!(
        !bytes.windows(b"/jig/v1/sign-in/link".len()).any(|part| part == b"/jig/v1/sign-in/link"),
        "sign-in link must not be printed, answered or logged"
    );
}

#[derive(Debug, PartialEq, Eq)]
struct Kept {
    project: Value,
    forge: Value,
    repositories: Value,
    landings: Value,
    landing: Value,
    gates: Value,
    api: Value,
}
impl Kept {
    fn read(mut answers: Vec<Value>, api: Value) -> Self {
        let gates = answers.pop().expect("project gates");
        let landing = answers.pop().expect("project landing");
        assert_eq!(landing["setting"], json!({"kind":"project-landing","project":1}));
        assert_eq!(landing["value"]["by"], "turn");
        assert_eq!(landing["value"]["batch"], 2);
        assert_eq!(gates["setting"], json!({"kind":"project-gates","project":1}));
        let [repository] = gates["value"]["repositories"].as_array().expect("gate repositories").as_slice() else {
            panic!("one repository's gates")
        };
        assert_eq!(repository["forge"], 1);
        assert_eq!(repository["repository"], 1);
        let [gate] = repository["gates"].as_array().expect("repository gates").as_slice() else {
            panic!("one blocking gate")
        };
        assert_eq!(gate["name"], "gate");
        assert_eq!(gate["workflow"], "gate.yml");
        assert_eq!(gate["blocking"], true);
        let landings = answers.pop().expect("landing list");
        let repositories = answers.pop().expect("forge repositories");
        let connected = answers.pop().expect("forge list");
        assert_eq!(connected["met"], true);
        let forge = connected["forges"].clone();
        let mut project = answers.pop().expect("project show");
        assert_eq!(answers, Vec::<Value>::new());
        assert_eq!(project["number"], 1);
        assert_eq!(project["cut"], false);
        let [resource] = project["resources"].as_array().expect("resources").as_slice() else {
            panic!("one adopted repository")
        };
        assert_eq!(resource["role"], "owned");
        assert_eq!(resource["home"], true);
        // A read's digest is not stored project state.
        project.as_object_mut().expect("project object").remove("digest");
        let [row] = forge["forges"].as_array().expect("forges").as_slice() else { panic!("one forge") };
        assert_eq!(row["number"], 1);
        assert_eq!(row["state"], "connected");
        assert_eq!(row["mode"], "managed");
        assert_eq!(repositories["more"], false);
        assert!(
            repositories["rows"]
                .as_array()
                .expect("repositories")
                .iter()
                .any(|row| row["owner"]["login"] == "work" && row["name"] == "hello")
        );
        assert_eq!(landings, json!({"project":1,"turn":null,"landings":[]}));
        Self { project, forge, repositories, landings, landing, gates, api }
    }
}

#[test]
fn a_stopped_deployment_restored_at_its_place_uses_new_files() {
    let place = Place::new();
    let hash = place.walk(false);
    place.listeners();
    // A baseline session captures ctl/API facts and the first worker pin before the archive.
    let (before, first_pin) = place.read(&[]);
    place.restore();
    let mut old_files = place.replace();
    old_files.push(first_pin);
    let old = old_files
        .iter()
        .map(|file| Identity::metadata(&file.metadata().expect("old file identity")))
        .collect::<Vec<_>>();
    assert_eq!(place.walk(true), hash, "check-only walk of the same recipe chain");
    place.listeners();
    let (after, new_pin) = place.read(&old);
    assert_eq!(after, before, "ctl and fake Forgejo facts survive the copy and newer build");

    drop(new_pin);
    drop(old_files);
    place.root.remove();
}
