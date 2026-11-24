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
        let setups = root.path().join("setups");
        let place = Self { root, scratch, bin, bundle, setups, ports };
        place.fixture();
        place.copy();
        place
    }

    fn fixture(&self) {
        let source = skein_scratch::paths::tree().join("docs/development/qa/setups/turn-landing");
        let target = self.setups.join("turn-landing");
        let mut pending = vec![(source, target.clone())];
        while let Some((source, target)) = pending.pop() {
            fs::create_dir_all(&target).expect("fixture directory");
            for entry in fs::read_dir(source).expect("fixture inputs") {
                let entry = entry.expect("fixture entry");
                let destination = target.join(entry.file_name());
                let kind = entry.file_type().expect("fixture kind");
                if kind.is_dir() {
                    pending.push((entry.path(), destination));
                } else {
                    assert!(kind.is_file(), "regular fixture input");
                    fs::copy(entry.path(), destination).expect("fixture copy");
                }
            }
        }
        // The recipe already restarts after landing.json. Commit the model's provider there,
        // so that restart takes the store's fake endpoint rather than its stock default.
        let providers = fs::read_to_string(target.join("config/providers.json")).expect("fixture providers");
        let providers = providers.replace("{place.model}", &self.ports[7].to_string());
        let providers = temper_agent_fixtures::providers::parse(providers.as_bytes());
        let providers = temper_agent_fixtures::providers::flat(&providers);
        let path = target.join("files/landing.json");
        let mut settings = document(&path);
        settings["items"].as_array_mut().expect("landing settings").extend(support::provider_setup::items(&providers));
        support::write_private(&path, &serde_json::to_vec(&settings).expect("fixture settings"));
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

    fn arguments(&self, drive: bool) -> Vec<String> {
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
        arguments
    }

    fn walk(&self) {
        let lines = session::run(self, self.arguments(false), false, &[]);
        assert!(lines.last().is_some_and(|line| line.starts_with("walked turn-landing ")));
        assert!(lines.iter().any(|line| line == "turn-landing:7 ok"), "walk restarted engine and worker");
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

    fn processes(&self, old: &[Identity]) {
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
    }

    fn listeners(&self) {
        let engine = support::start_document(
            &self.scratch.join("config/engine").join(temper_config_shell::FILE),
            temper_config_shell::Shape::EngineAlone,
        );
        let providers = document(&self.scratch.join("config/providers.json"));
        let worker = support::start_document(
            &self.scratch.join("config/worker").join(temper_config_shell::FILE),
            temper_config_shell::Shape::WorkerAlone,
        );
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
            (&providers["providers"][0]["endpoints"][0]["address"]["socket"], 7),
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
    for path in [b"/sign-in?secret=".as_slice(), b"/jig/v1/sign-in/link"] {
        assert!(
            !bytes.windows(path.len()).any(|part| part == path),
            "sign-in link must not be printed, answered or logged"
        );
    }
}

fn kept_project(answer: &Value) {
    assert_eq!(answer["number"], 1);
    assert_eq!(answer["name"]["bytes"], "Hello");
    assert_eq!(
        answer["resources"],
        json!([{
            // The fresh recipe adopted work/hello, repository 1 on forge 1, as Hello's home.
            "part": {"connector": 1, "kind": "resource", "key": {"path": ["0001", "00000001"]}},
            "role": "owned",
            "home": true
        }])
    );
}

#[test]
fn a_stopped_deployment_restored_at_its_place_uses_new_files() {
    let place = Place::new();
    place.walk();
    place.restore();
    let archived_pin = File::open(place.pin()).expect("archived worker pin");
    let mut old_files = place.replace();
    old_files.push(archived_pin);
    let old = old_files
        .iter()
        .map(|file| Identity::metadata(&file.metadata().expect("old file identity")))
        .collect::<Vec<_>>();
    let lines = session::run(&place, place.arguments(true), true, &old);
    assert_eq!(lines.last().map(String::as_str), Some("ended"));
    drop(old_files);
    place.root.remove();
}
