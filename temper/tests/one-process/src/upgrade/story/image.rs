//! The current build's settled stop (temper/docs/design/protocol/baseline.md, 8).
use super::{Maps, STATE, State, fixtures, golden, mutations, referee, runner::World, stories};
use skein_fake_forge_domain::{Observation, promises::Recorded};
use skein_fake_forge_forgejo::observations;
use skein_lib::{Secret, Wall};
use std::{
    cell::RefCell,
    collections::BTreeMap,
    path::{Path, PathBuf},
    rc::Rc,
};

const README: &str = "# Settled upgrade image

Made by `upgrade::the_settled_image_matches_the_story` in the one-process
world's focused suite: story A, its uncertain forge write, then a drained stop.
This is the current build's fixture, not the once-kept baseline.

Rewrite with `TEMPER_TEST_UPGRADE_IMAGE=1 cargo nextest run -p
temper-one-process-world --test suite -E
'test(=upgrade::the_settled_image_matches_the_story)'` (one shell line).
Run the same test without the variable to check deterministic bytes.
Never edit the image by hand. Rewriting replaces the whole directory.

- `state/`, `worker/`, `client/`, `setup/`: the four machine roots.
- `modes.txt`: every node's kind, mode, owner and machine path, including
  empty directories. Host `dot-git` components stand for machine `.git`.
- `configuration.json`: the story's startup configuration.
- `forge.image`: the fake peer's own lasting domain and service image.
- `outside.json`: issued synthetic tokens and users, stop wall in nanoseconds,
  mid-run task and attempt, and the kept terminal answer's bytes.
- `ledger.jsonl`: the story's mutation projection, in the fake forge's
  observation JSON-lines encoding (no ancestry judgment is stored).

The loader makes a cold referee store and preloads transcript/channel payloads;
source-map reads rebuild the committed prefix, cold parts and layouts before
recovery. No live referee state is kept.
";

#[derive(serde::Serialize, serde::Deserialize)]
struct Outside {
    tokens: Vec<(Vec<u8>, u64)>,
    users: Vec<(u64, Vec<u8>)>,
    wall: u64,
    task: u64,
    attempt: u64,
    kept_answer: Vec<u8>,
}

pub(super) struct Stop {
    disk: skein_fake_machine::Machine,
    configuration: Vec<u8>,
    forge: Box<[u8]>,
    outside: Outside,
    ledger: Vec<Observation>,
}
impl Stop {
    pub(super) fn read(stopped: World) -> Self {
        let (forge, outside) = STATE.with(|s| {
            let mut s = s.borrow_mut();
            let outside = Outside {
                tokens: s.tokens.iter().map(|(secret, user)| (secret.expose().to_vec(), *user)).collect(),
                users: s.users.iter().map(|(id, login)| (*id, login.to_vec())).collect(),
                wall: s.stopped_wall.expect("wall at stop").as_nanos(),
                task: s.task,
                attempt: s.attempt,
                kept_answer: s.kept_answer.clone(),
            };
            (s.forge_image.take().expect("settled peer image"), outside)
        });
        Self {
            disk: stopped.disk,
            configuration: stopped.configuration,
            forge,
            outside,
            ledger: mutations(&stopped.ledger.borrow()),
        }
    }
}

pub(super) fn directory() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("golden/settled")
}

pub(super) fn write(directory: &Path, mut stop: Stop) {
    golden::files::export(&mut stop.disk, directory, &stop.configuration);
    std::fs::write(directory.join("forge.image"), stop.forge).expect("settled forge image");
    std::fs::write(directory.join("outside.json"), serde_json::to_vec_pretty(&stop.outside).expect("outside bytes"))
        .expect("settled outside state");
    let mut ledger = Vec::new();
    for observation in stop.ledger {
        let record = Recorded::Observed { observation, forward: None };
        let size = observations::encoded_len(&record).expect("story observation names are UTF-8");
        let mut line = skein_lib::List::with_capacity(size);
        observations::encode(&record, &mut line).expect("measured observation line");
        ledger.extend_from_slice(line.as_slice());
    }
    std::fs::write(directory.join("ledger.jsonl"), ledger).expect("settled mutation ledger");
    std::fs::write(directory.join("README.md"), README).expect("settled image provenance");
}

pub(super) fn load(directory: &Path) -> World {
    let outside: Outside =
        serde_json::from_slice(&std::fs::read(directory.join("outside.json")).expect("outside file"))
            .expect("settled outside state");
    let ledger = std::fs::read(directory.join("ledger.jsonl"))
        .expect("settled ledger")
        .split(|b| *b == b'\n')
        .filter(|line| !line.is_empty())
        .map(|line| match observations::decode(line).expect("settled observation") {
            Recorded::Observed { observation, .. } => observation,
            Recorded::Lost(_) => panic!("the mutation projection contains no loss checkpoints"),
        })
        .collect::<Vec<_>>();
    STATE.with(|s| {
        *s.borrow_mut() = State {
            tokens: outside.tokens.into_iter().map(|(bytes, user)| (Secret::new(bytes.into()), user)).collect(),
            users: outside.users.into_iter().map(|(id, login)| (id, login.into())).collect(),
            stopped_wall: Some(Wall::from_nanos(outside.wall)),
            task: outside.task,
            attempt: outside.attempt,
            kept_answer: outside.kept_answer,
            issue: ledger
                .iter()
                .any(|o| matches!(o, Observation::Opened {title, ..} if title.as_ref() == b"Upgrade once")),
            forge_image: Some(std::fs::read(directory.join("forge.image")).expect("settled forge bytes").into()),
            ..State::default()
        }
    });
    let mut disk = golden::files::load(directory);
    let mut store = referee::Store::new();
    store.recovering();
    golden::files::transcripts(directory, &mut store);
    // The source log also holds the observer's committed prefix and cold parts.
    // Replay its reads before recovery, without running the service or the story.
    let active = Maps::read(&mut disk).active;
    read_prefix(directory, active, &mut store);
    let start = temper_playground::stories::A.start(fixtures::SEED, skein_sim::Config::calm().wall);
    World {
        disk,
        configuration: std::fs::read(directory.join("configuration.json")).expect("settled configuration"),
        transport: start.fakes.forge,
        store: Rc::new(RefCell::new(store)),
        ledger: Rc::new(RefCell::new(ledger)),
    }
}

fn read_prefix(directory: &Path, active: u64, store: &mut referee::Store) {
    use referee::durable::Observed;
    let relative = format!("generations/{active}/store");
    let root = directory.join("state").join(&relative);
    let mut names = std::fs::read_dir(&root)
        .expect("source map directory")
        .map(|entry| entry.expect("source map entry").file_name())
        .filter(|name| name == "snapshot" || name.as_encoded_bytes().starts_with(b"log-"))
        .collect::<Vec<_>>();
    names.sort_by_key(|name| (name != "snapshot", name.clone()));
    let mut events = Vec::new();
    for name in names {
        let path = format!("{relative}/{}", name.to_str().expect("source map name")).into_bytes();
        events.push(Observed::Bytes {
            path: path.clone(),
            at: 0,
            bytes: std::fs::read(root.join(&name)).expect("source map bytes"),
        });
        events.push(if name == "snapshot" { Observed::Snapshot(path) } else { Observed::Recovered(path) });
    }
    let bounds = temper_protocol_store::bounds(
        &temper_service::limits::profile(temper_service::limits::Profile::Small).forge,
        jig_protocol_config::profile(jig_protocol_config::Profile::Small).core.tasks.spec_bytes,
    )
    .expect("application codec bounds");
    store
        .observe(&mut events, &stories::playground::codec(), fixtures::store_names(), &bounds)
        .expect("settled referee prefix read from its files");
}

fn files(directory: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    let mut files = BTreeMap::new();
    if directory.exists() {
        visit(directory, Path::new(""), &mut files);
    }
    files
}
fn visit(directory: &Path, relative: &Path, files: &mut BTreeMap<PathBuf, Vec<u8>>) {
    for entry in std::fs::read_dir(directory.join(relative)).expect("image directory") {
        let entry = entry.expect("image entry");
        let path = relative.join(entry.file_name());
        let kind = entry.file_type().expect("image node kind");
        if kind.is_dir() {
            visit(directory, &path, files);
        } else {
            assert!(kind.is_file(), "only files and directories in a kept image");
            files.insert(path, std::fs::read(entry.path()).expect("image file"));
        }
    }
}

pub(super) fn compare(made: &Path, kept: &Path) -> Result<(), String> {
    let made = files(made);
    let kept = files(kept);
    for path in made.keys().chain(kept.keys()).collect::<std::collections::BTreeSet<_>>() {
        let difference = match (made.get(path), kept.get(path)) {
            (Some(left), Some(right)) if left != right => Some("differs"),
            (Some(_), None) => Some("missing from kept image"),
            (None, Some(_)) => Some("extra in kept image"),
            _ => None,
        };
        if let Some(why) = difference {
            return Err(format!("{}: {why}", path.display()));
        }
    }
    Ok(())
}

pub(super) fn replace(made: &Path, kept: &Path) {
    let files = files(made);
    if kept.exists() {
        std::fs::remove_dir_all(kept).expect("remove the previous settled image");
    }
    for (path, bytes) in files {
        let target = kept.join(path);
        std::fs::create_dir_all(target.parent().expect("image parent")).expect("kept image directory");
        std::fs::write(target, bytes).expect("replace kept image file");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn comparison_names_changed_missing_and_extra_files() {
        let scratch = skein_scratch::Scratch::new("upgrade-image-comparison");
        let made = scratch.path().join("made");
        let kept = scratch.path().join("kept");
        for directory in [&made, &kept] {
            std::fs::create_dir_all(directory.join("state")).expect("scratch state");
            std::fs::write(directory.join("state/active"), b"1\n").expect("scratch active");
        }
        assert_eq!(compare(&made, &kept), Ok(()));
        std::fs::write(kept.join("state/active"), b"2\n").expect("one changed byte");
        assert_eq!(compare(&made, &kept), Err("state/active: differs".into()));
        std::fs::remove_file(kept.join("state/active")).expect("missing active");
        assert_eq!(compare(&made, &kept), Err("state/active: missing from kept image".into()));
        replace(&made, &kept);
        std::fs::write(kept.join("extra"), b"extra\n").expect("extra file");
        assert_eq!(compare(&made, &kept), Err("extra: extra in kept image".into()));
        replace(&made, &kept);
        assert_eq!(compare(&made, &kept), Ok(()), "rewrite removes stale files");
    }
}
