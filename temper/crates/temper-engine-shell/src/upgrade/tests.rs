//! The upgrade command on a deployment copy (protocol/binaries.md, 1, 2).
#![expect(clippy::disallowed_types, reason = "scratch fixtures set filesystem modification times without waiting")]
use super::{Request, run};
use skein_fake_machine::{How, Is, Item, Machine, Opened, Refusal};
use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::os::unix::fs::PermissionsExt;
use std::time::{Duration, SystemTime};
use temper_deployment_fixtures::deployment;

use crate::testing::modified;

#[test]
fn workers_arguments_are_distinct_and_modes_exclude_each_other() {
    let environment = [(Box::from(b"HOME".as_slice()), Box::from(b"/home/test".as_slice()))];
    let words = |words: &[&[u8]]| words.iter().map(|word| Box::from(*word)).collect::<Vec<_>>();
    for mode in ["--check", "--workers"] {
        let (directories, request) =
            super::configure(mode, &words(&[b"--data", b"state", b"backup"]), &environment, b"/work")
                .expect("one directory");
        assert_eq!(directories.data.as_ref(), b"/work/state");
        assert_eq!(
            request,
            if mode == "--check" {
                Request::Check(b"/work/backup".as_slice().into())
            } else {
                Request::Workers(b"/work/backup".as_slice().into())
            }
        );
        for invalid in [
            words(&[]),
            words(&[b""]),
            words(&[b"one", b"two"]),
            words(&[b"--check", b"one"]),
            words(&[b"--workers", b"one"]),
            words(&[b"--back"]),
            words(&[b"one", b"--back"]),
        ] {
            assert!(super::configure(mode, &invalid, &environment, b"/work").is_err(), "{mode}: {invalid:?}");
        }
    }
    assert_eq!(super::configure("--back", &[], &environment, b"/work").unwrap().1, Request::Back);
    for invalid in [words(&[b"one"]), words(&[b"--workers", b"one"]), words(&[b"--check", b"one"])] {
        assert!(super::configure("--back", &invalid, &environment, b"/work").is_err());
    }
    for (mode, arguments, form) in [
        ("--check", &["upgrade", "--check"][..], "[DIRECTORIES] DIRECTORY"),
        ("--workers", &["upgrade", "--workers"], "[DIRECTORIES] DIRECTORY"),
        ("--back", &["upgrade", "--back", "extra"], "[DIRECTORIES]"),
    ] {
        assert_eq!(
            crate::binary::refusal(arguments),
            (
                2,
                format!(
                    "temper-engine upgrade {mode}: invalid arguments\nusage: temper-engine upgrade {mode} {form}\n"
                )
            )
        );
    }
    let arguments = words(&[b"upgrade", b"--workers", b"backup"]);
    let skein_shell::Reached::Command(row, rest) = crate::ENGINE.reach(b"temper-engine", &arguments) else {
        panic!("workers row")
    };
    assert_eq!(row, &crate::commands::UPGRADE_WORKERS);
    assert_eq!(
        super::configure("--workers", rest, &environment, b"/work").unwrap().1,
        Request::Workers(b"/work/backup".as_slice().into())
    );
}

#[test]
fn deployment_copy_commits_both_frame_layouts_with_its_header() {
    use jig_domain_core::{CoreKey, Key};
    use jig_protocol_store::{keys, registry_v1};
    use skein_codec::Record as _;
    use skein_kv::Op;
    use skein_lib::Reader;
    use temper_deployment_fixtures::log;

    const KEY_LIMIT: u32 = 64;
    // The kept core-6 row's commits field follows its layout, tag, ID, and eight counters.
    const COMMITS_OFFSET: usize = 83;
    const FRAME_REGISTRY_WRITES: usize = 3; // Two layouts and the header.

    let (_scratch, directories) = deployment();
    let state = crate::path(&directories.data).join("state");
    let active = std::fs::read(state.join("active")).expect("active generation");
    let bytes = std::fs::read(state.join(log::initial_path(&active))).expect("copied store log");
    let (commits, used) = log::frames(&bytes, 0);
    assert_eq!(used, bytes.len(), "the log ends on a complete commit");
    let [previous, (number, ops)] = commits.last_chunk().expect("at least two commits");
    let put = |key: &[u8], label| {
        ops.iter()
            .find_map(|op| {
                if let Op::Put { key: written, value } = op
                    && written.as_ref() == key
                {
                    Some(value.as_ref())
                } else {
                    None
                }
            })
            .unwrap_or_else(|| panic!("missing {label} put in the final fixture commit"))
    };

    for (family, root, number, id) in [("jig-store-core", "TurnRecord", 9, 6), ("smith-transcript", "Turn", 0, 7)] {
        let key = keys::encode_own(&keys::Own::Layout { family: family.into(), number, id }, KEY_LIMIT)
            .expect("frame layout key");
        let value = put(&key, root);
        let layout =
            registry_v1::Layout::decode(&registry_v1::CEILINGS, &mut Reader::new(value)).expect("frame layout row");
        assert_eq!(layout.root.as_ref(), root.as_bytes());
    }
    let key = keys::encode(&Key::Core(CoreKey::Deployment), KEY_LIMIT).expect("deployment key");
    let header = put(&key, "deployment header");
    let commits = Reader::new(&header[COMMITS_OFFSET..]).u64();
    assert_eq!(commits, Some(*number), "the header names the final commit");
    assert_eq!(*number, previous.0 + 1, "the commit is next in sequence");
    assert_eq!(ops.len(), FRAME_REGISTRY_WRITES);
}

#[test]
fn agent_fixture_round_trip_keeps_the_engine_file_and_installs_figures() {
    use temper_config_shell::{FILE, FORMER, Shape, parse, render};
    let (_scratch, directories) = deployment();
    let config = crate::path(&directories.config);
    let path = config.join(FILE);
    let facts = temper_config_shell::facts(&directories).expect("fixture facts");
    let read = parse(&std::fs::read(&path).expect("engine file"), Shape::EngineAlone, &facts).expect("engine file");
    let canonical = render(&read.file, Shape::EngineAlone);
    let mut formatted = canonical.to_vec();
    formatted.extend_from_slice(b"\n# fixture formatting\n");
    std::fs::write(&path, formatted).expect("formatted engine file");
    let mut agent = temper_agent_fixtures::read(config);
    agent["profile"]["policy"]["max_turns"] = 17.into();
    temper_agent_fixtures::install(config, &agent);
    assert_eq!(std::fs::read(&path).expect("rewritten file"), canonical.as_ref());
    assert!(!config.join(FORMER).exists());
    let installed = temper_agent_fixtures::read(config);
    assert_eq!(installed["profile"]["policy"]["max_turns"], 17);
    assert_eq!(installed["endpoints"], agent["endpoints"]);
    let start = temper_agent_fixtures::start(config);
    assert_eq!(start["workers"], serde_json::json!([]));
    assert_eq!(start["engine"]["hosts"]["listener"][0]["socket"], "127.0.0.1:7103");
}

#[test]
fn an_unremovable_scratch_is_named_and_the_rehearsal_goes_on() {
    let (_scratch, directories) = deployment();
    let stale = crate::path(&directories.cache).join("upgrade-00000000000000aa");
    std::fs::create_dir_all(&stale).expect("stale scratch");
    std::fs::write(stale.join("file"), b"left behind").expect("stale file");
    modified(&stale, SystemTime::now() - Duration::from_secs(7200));
    std::fs::set_permissions(&stale, std::fs::Permissions::from_mode(0o500)).expect("removal denied");
    let lines = super::scratch::sweep("temper-engine", crate::path(&directories.cache));
    let code = run(
        "temper-engine",
        &directories,
        &Request::Workers([directories.data.as_ref(), b"/state"].concat().into()),
        "test-build",
    );
    std::fs::set_permissions(&stale, std::fs::Permissions::from_mode(0o700)).expect("restore cleanup permission");
    assert_eq!(lines, ["temper-engine upgrade: scratch left behind not removed: upgrade-00000000000000aa: errno 13"]);
    assert_eq!(code.expect("removal fault does not stop rehearsal"), 0);
    assert_eq!(std::fs::read(stale.join("file")).expect("unremoved file remains"), b"left behind");
}

const LEFTOVER: &[u8] = b"upgrade-00000000000000aa";
const STAGING: &[u8] = b"upgrade-00000000000000aa.new";
const FILE: &[u8] = b"upgrade-00000000000000aa/file";
const BUSY: &[u8] = b"upgrade-00000000000000bb";
const REGULAR: &[u8] = b"upgrade-00000000000000cc";
const LINK: &[u8] = b"upgrade-00000000000000dd";
const OTHER: [&[u8]; 4] = [b"upgrade-x", b"other", b"upgrade-00000000000000aA", b"upgrade-00000000000000aaa"];
const OUTSIDE: &[u8] = b"outside";
const BYTES: &[u8] = b"checkpoint bytes";
const OLD: Duration = Duration::from_secs(7200);

#[derive(Debug, PartialEq, Eq)]
enum Effect {
    Metadata(Vec<u8>),
    Open(Vec<u8>),
    Lock(Vec<u8>, Result<(), Refusal>),
    Remove(Vec<u8>),
    Release(Opened),
}

struct Cache {
    machine: Machine,
    root: Opened,
    effects: Vec<Effect>,
}

impl Cache {
    fn new(items: &[Item]) -> Self {
        let mut machine = Machine::for_user(1000);
        let root = machine.lay(items);
        Self { machine, root, effects: Vec::new() }
    }

    fn names(&mut self) -> Vec<Box<[u8]>> {
        let listing = self.machine.open(self.root, b".", How::Directory).unwrap();
        let entries = self.machine.list(listing, 16, 512).unwrap();
        self.machine.close(listing);
        entries.into_iter().map(|(_, name)| name).collect()
    }

    fn sweep(&mut self, now: SystemTime, modified: SystemTime) -> Vec<String> {
        let entries = self.names();
        super::scratch::sweep_with(
            "temper-engine",
            entries.into_iter().map(|name| (std::ffi::OsString::from_vec(name.to_vec()), name)),
            now,
            self,
            |cache, name| {
                cache.effects.push(Effect::Metadata(name.to_vec()));
                Ok((cache.kind(name) == Some(Is::Directory)).then_some(modified))
            },
            |cache, name| {
                cache.effects.push(Effect::Open(name.to_vec()));
                let held = cache.open(name, How::Directory).map_err(cache_error)?;
                let result = cache.machine.lock(held);
                cache.effects.push(Effect::Lock(name.to_vec(), result));
                match result {
                    Ok(()) => Ok(Some(held)),
                    Err(why) => {
                        cache.machine.close(held);
                        if why == Refusal::Held { Ok(None) } else { Err(cache_error(why)) }
                    }
                }
            },
            |cache, name, held| {
                cache.effects.push(Effect::Remove(name.to_vec()));
                let result = cache.remove_contents(name, held);
                cache.machine.close(held);
                result
            },
        )
    }

    fn open(&mut self, name: &[u8], how: How) -> Result<Opened, Refusal> {
        self.machine.open(self.root, name, how)
    }

    fn kind(&mut self, name: &[u8]) -> Option<Is> {
        match self.open(name, How::PathNoFollow) {
            Ok(opened) => {
                let kind = self.machine.stat(opened).is;
                self.machine.close(opened);
                Some(kind)
            }
            Err(Refusal::NotFound) => None,
            Err(why) => panic!("unexpected cache refusal: {why:?}"),
        }
    }

    fn remove_contents(&mut self, name: &[u8], held: Opened) -> std::io::Result<()> {
        self.remove_tree(self.root, name, held)
    }

    fn remove_tree(&mut self, parent: Opened, name: &[u8], dir: Opened) -> std::io::Result<()> {
        for (kind, child) in self.machine.list(dir, 16, 512).map_err(cache_error)? {
            if kind == Is::Directory {
                let nested = self.machine.open(dir, &child, How::Directory).map_err(cache_error)?;
                let result = self.remove_tree(dir, &child, nested);
                self.machine.close(nested);
                result?;
            } else {
                self.machine.remove(dir, &child, false).map_err(cache_error)?;
            }
        }
        self.machine.remove(parent, name, true).map_err(cache_error)
    }

    fn own_scratch(&mut self, name: &[u8], held: Opened) -> Option<String> {
        super::scratch::remove_with(
            "temper-engine",
            crate::path(name),
            held,
            self,
            |cache, path| {
                let name = path.as_os_str().as_bytes();
                cache.effects.push(Effect::Remove(name.to_vec()));
                let dir = cache.open(name, How::Directory).map_err(cache_error)?;
                let result = cache.remove_contents(name, dir);
                cache.machine.close(dir);
                result
            },
            |cache, held| {
                cache.effects.push(Effect::Release(held));
                cache.machine.close(held);
            },
        )
    }

    fn bytes(&mut self, name: &[u8]) -> (Vec<u8>, u32) {
        let file = self.open(name, How::Read).expect("kept file");
        let facts = self.machine.stat(file);
        let bytes = self.machine.read(file, 0, u32::try_from(facts.size).expect("fixture bytes")).unwrap();
        self.machine.close(file);
        (bytes, facts.links)
    }
}

fn cache_error(why: Refusal) -> std::io::Error {
    assert_eq!(why, Refusal::Permission, "the fixture scripts permission failures");
    std::io::Error::from_raw_os_error(13)
}

fn scratch_with_outside_link() -> [Item; 5] {
    [
        Item::file(OUTSIDE, BYTES),
        Item::directory(LEFTOVER),
        Item::directory(b"upgrade-00000000000000aa/nested"),
        Item::file(FILE, b"stale"),
        Item::hard_link(b"upgrade-00000000000000aa/nested/linked", OUTSIDE),
    ]
}

#[test]
fn a_rehearsals_sweep_takes_only_stale_unlocked_scratch() {
    const BUSY_FILE: &[u8] = b"upgrade-00000000000000bb/file";
    let mut items = Vec::from(scratch_with_outside_link());
    items.extend([
        Item::directory(BUSY),
        Item::file(BUSY_FILE, b"in use"),
        Item::file(REGULAR, b"regular file"),
        Item::link(LINK, BUSY),
    ]);
    items.extend(OTHER.map(Item::directory));
    let mut cache = Cache::new(&items);
    let mut remaining = cache.names();
    remaining.retain(|name| name.as_ref() != LEFTOVER);
    let held = cache.open(BUSY, How::Directory).expect("busy scratch");
    cache.machine.lock(held).expect("hold busy scratch");
    assert_eq!(cache.bytes(OUTSIDE), (BYTES.to_vec(), 2));
    let now = SystemTime::UNIX_EPOCH + OLD;
    assert_eq!(cache.sweep(now, now - OLD), Vec::<String>::new());
    assert_eq!(
        cache.effects,
        [
            Effect::Metadata(LEFTOVER.to_vec()),
            Effect::Open(LEFTOVER.to_vec()),
            Effect::Lock(LEFTOVER.to_vec(), Ok(())),
            Effect::Remove(LEFTOVER.to_vec()),
            Effect::Metadata(BUSY.to_vec()),
            Effect::Open(BUSY.to_vec()),
            Effect::Lock(BUSY.to_vec(), Err(Refusal::Held)),
            Effect::Metadata(REGULAR.to_vec()),
            Effect::Metadata(LINK.to_vec()),
        ]
    );
    assert_eq!(cache.names(), remaining, "only the stale scratch is gone");
    assert_eq!(cache.bytes(OUTSIDE), (BYTES.to_vec(), 1));
    assert_eq!(cache.bytes(BUSY_FILE).0, b"in use");
    assert_eq!(cache.bytes(REGULAR).0, b"regular file");
    assert_eq!(cache.kind(LINK), Some(Is::Link), "the sweep does not follow the link");
    for name in OTHER {
        assert_eq!(cache.kind(name), Some(Is::Directory));
    }
    cache.machine.close(held);
}

#[test]
fn a_rehearsals_end_removes_its_own_scratch() {
    use std::cell::RefCell;
    const SOURCE: &[u8] = b"/state";
    let report_error = Err("drive: kernel unavailable".to_owned());
    for (open, report, expected, actions) in [
        (Ok(()), Ok(0), Ok(0), &["open", "make", "report", "remove"][..]),
        (Ok(()), Ok(1), Ok(1), &["open", "make", "report", "remove"][..]),
        (Ok(()), report_error.clone(), report_error, &["open", "make", "error", "remove"][..]),
        (Err(2), Ok(0), Err("/state: no readable deployment directory: errno 2".into()), &["open"][..]),
    ] {
        let effects = RefCell::new(Vec::new());
        let result = super::finish_with(
            |made| {
                super::source_with(SOURCE, |path| {
                    assert_eq!(path, crate::path(SOURCE));
                    effects.borrow_mut().push("open");
                    open
                })?;
                *made = Some(LEFTOVER);
                effects.borrow_mut().push("make");
                effects.borrow_mut().push(if report.is_ok() { "report" } else { "error" });
                report
            },
            |scratch| {
                assert_eq!(scratch, LEFTOVER, "remove only this run's scratch");
                effects.borrow_mut().push("remove");
            },
        );
        assert_eq!(result, expected);
        assert_eq!(effects.into_inner(), actions);
    }
}

#[test]
fn own_scratch_removal_releases_all_contents_and_outside_links() {
    let mut cache = Cache::new(&scratch_with_outside_link());
    let held = cache.open(LEFTOVER, How::Directory).expect("own scratch");
    cache.machine.lock(held).expect("own lock");
    assert_eq!(cache.own_scratch(LEFTOVER, held), None);
    assert_eq!(cache.effects, [Effect::Remove(LEFTOVER.to_vec()), Effect::Release(held)]);
    assert_eq!(cache.kind(LEFTOVER), None, "own scratch is removed whole");
    assert_eq!(cache.bytes(OUTSIDE), (BYTES.to_vec(), 1));
}

#[test]
fn an_unlocked_staging_directory_is_never_swept() {
    let mut cache = Cache::new(&[Item::directory(STAGING)]);
    let now = SystemTime::UNIX_EPOCH + OLD;
    assert_eq!(cache.sweep(now, now - OLD), Vec::<String>::new());
    assert_eq!(cache.kind(STAGING), Some(Is::Directory), "the staging name is never swept");
}

#[test]
fn recent_and_future_unlocked_leftovers_are_not_swept() {
    let mut cache = Cache::new(&[Item::directory(LEFTOVER), Item::file(FILE, b"copying")]);
    let now = SystemTime::UNIX_EPOCH + OLD;
    for time in [now, now + OLD] {
        assert_eq!(cache.sweep(now, time), Vec::<String>::new());
        let file = cache.open(FILE, How::Read).expect("recent leftover remains");
        assert_eq!(cache.machine.read(file, 0, 7).unwrap(), b"copying");
        cache.machine.close(file);
    }
    assert_eq!(cache.sweep(now, now - OLD), Vec::<String>::new());
    assert_eq!(cache.kind(LEFTOVER), None, "the same leftover is removed whole once old");
}

#[test]
fn an_unopenable_leftover_is_named_instead_of_silently_skipped() {
    let mut cache = Cache::new(&[Item::directory(LEFTOVER).mode(0o000)]);
    let now = SystemTime::UNIX_EPOCH + OLD;
    let lines = cache.sweep(now, now - OLD);
    assert_eq!(lines, ["temper-engine upgrade: scratch left behind not removed: upgrade-00000000000000aa: errno 13"]);
    assert_eq!(cache.kind(LEFTOVER), Some(Is::Directory), "unopenable leftover remains");
}

#[test]
fn own_scratch_removal_failure_names_the_path_and_cause() {
    let mut cache = Cache::new(&[Item::directory(LEFTOVER).mode(0o500), Item::file(FILE, b"left behind")]);
    let held = cache.open(LEFTOVER, How::Directory).expect("own directory");
    cache.machine.lock(held).expect("own lock");
    let line = cache.own_scratch(LEFTOVER, held);
    assert_eq!(line.as_deref(), Some("temper-engine upgrade: scratch not removed: upgrade-00000000000000aa: errno 13"));
}

#[test]
fn scratch_failures_name_errno_or_error_kind_without_panicking() {
    for (why, expected) in [
        (std::io::Error::from_raw_os_error(13), "scratch: errno 13"),
        (std::io::Error::from(std::io::ErrorKind::Other), "scratch: kind Other"),
    ] {
        assert_eq!(super::scratch::failure(&why), expected);
    }
}
