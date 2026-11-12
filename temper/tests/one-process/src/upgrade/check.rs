//! Rehearsal and rollback on the one-process world's disk, service and configuration.
use super::{FIXTURE_SETS, maps::Maps};
use crate::{fixtures, referee, stories, world};
use jig_protocol_store::{Owner, PatchSets};
use skein_fake_machine::{How, Is, Machine, Opened};
use skein_sim::{Handle, Sim};
use std::{cell::RefCell, collections::BTreeMap, fmt::Write, path::Path, rc::Rc};

type Patches = Vec<(Owner, u32, jig_protocol_store::Counts)>;
#[derive(Clone, Copy, Debug)]
enum Until {
    Check,
    Back,
    Switched,
    Wires,
}

fn configured(document: &[u8]) -> temper_service::Service {
    let mut document: serde_json::Value = serde_json::from_slice(document).expect("configuration");
    let given = temper_playground::documents::given(&document);
    temper_playground::documents::settings::strip_models(&mut document);
    document.as_object_mut().expect("document").remove("world_settings");
    temper_service::startup::restoring_with(
        &serde_json::to_vec(&document).expect("configuration bytes"),
        &jig_protocol_config::Directories::working(),
        &fixtures::files(),
        Some(&temper_playground::documents::agent()),
        &fixtures::seeds(),
        None,
        &given,
    )
    .unwrap_or_else(|why| panic!("{}", String::from_utf8_lossy(&temper_service::startup::lines(&why))))
}

fn drive(disk: &mut Machine, document: &[u8], sets: PatchSets, until: Until) -> (String, u8, Patches) {
    let mut service = configured(document);
    service.patch_sets(sets);
    let mut sim = Sim::new(fixtures::SEED, skein_sim::Config::calm());
    let pid = sim.spawn_process();
    let source = disk.reopen_root(0);
    let source = sim.root(pid, Handle::new(source.raw()));
    match until {
        Until::Check => {
            let scratch = disk.lay(&[]);
            let scratch = sim.root(pid, Handle::new(scratch.raw()));
            service.rehearse_store(source, scratch, 1000).expect("rehearsal adoption");
        }
        Until::Back => service.back_store(source, 1000).expect("back adoption"),
        Until::Switched | Until::Wires => service.adopt_store(source, 1000).expect("start adoption"),
    }
    if let Until::Check | Until::Back = until {
        let owners = service.wait_state().engine.expect("upgrade engine");
        assert!(
            owners.workers.closed && owners.setup.closed && owners.forge.closed,
            "upgrade closes unstarted owners before store I/O: {owners:?}"
        );
    }
    let mut patched = Vec::new();
    for _ in 0..100_000 {
        temper_service::iterate(&mut service, sim.now(), sim.wall());
        while let Some(note) = service.notes().pop() {
            if let temper_service::Note::Engine(temper_engine_service::Note::Patched { of, patch, counts }) = note {
                patched.push((of, patch, counts));
            }
        }
        assert!(service.links().is_empty(), "no secret served before the switch/wires checkpoint");
        let wait = service.wait_state().engine.expect("engine state");
        let reached = match until {
            Until::Check | Until::Back => service.is_empty(),
            Until::Switched => wait.restart == Some(jig_domain_core::start::Phase::Wires) && !wait.mark_durable,
            Until::Wires => wait.mark_durable,
        };
        if reached {
            let report = service.upgrade_report().map_or_else(
                || (String::new(), 0),
                |report| {
                    temper_engine_shell::upgrade::lines(
                        "temper-box",
                        report,
                        &temper_shell::directories::Directories {
                            config: b"/config".as_slice().into(),
                            data: b"/data".as_slice().into(),
                            cache: b"/cache".as_slice().into(),
                            origins: [skein_shell::Origin::Flag; 3],
                        },
                        b"/state",
                    )
                },
            );
            // The world's fault is after Switched, with no wires IO submitted. A killed start closes its descriptors.
            for handle in sim.cut(pid) {
                disk.close(Opened::new(handle.raw()));
            }
            sim.assert_no_open_fds(pid);
            return (report.0, report.1, patched);
        }
        if let Some(end) = service.end() {
            assert!(service.upgrade_report().is_some(), "start refused: {end:?}");
        }
        if let Until::Check | Until::Back = until {
            for submit in service.submissions().iter() {
                if let skein_io::kernel::Op::Listen { .. }
                | skein_io::kernel::Op::Connect { .. }
                | skein_io::kernel::Op::Spawn { .. } = &submit.kind
                {
                    panic!("upgrade tool made an effect: {:?}", submit.kind);
                }
            }
        }
        service.check_wait_boundary(sim.now());
        sim.submit(pid, service.submissions());
        skein_fake_machine::serve(disk, &mut sim);
        if !service.work_pending(sim.now()) && sim.ready(pid) == 0 {
            assert!(sim.advance(), "a pending operation or deadline");
        }
        sim.reap(pid, service.completions());
    }
    panic!(
        "upgrade {until:?} checkpoint did not settle: {:?}; report {:?}",
        service.wait_state(),
        service.upgrade_report()
    );
}

/// Pre-baseline component layouts have no wires and must reach a refusal,
/// including the layout before patch levels were added to version 3.
pub fn prebaseline_rehearsals() {
    let mut unleveled = jig_protocol_store::format::current(1, [7; 16]);
    unleveled.tasks = 9;
    unleveled.people = 14;
    unleveled.settings = 1;
    let before_levels =
        [0, 3, 0, 3, 0, 6, 0, 9, 0, 13, 0, 1, 0, 2, 0, 1, 0, 0, 0, 1, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7];
    for format in [unleveled.encode().expect("unleveled format"), Box::from(before_levels)] {
        source_rehearsal(&format, "A store from before the baseline is not translated");
    }
}

/// A corrupt opening terminal must end the tool even before any root load exists.
pub fn corrupt_open_rehearsal() {
    source_rehearsal(
        &[0],
        "store opening failed: Data(Failure { cause: Decode(Value), row: Some(RowSite { family: Some(Component), key: [0, 0]",
    );
}

fn source_rehearsal(format: &[u8], expected: &str) {
    use skein_fake_machine::Item;
    let key = jig_protocol_store::keys::encode_own(&jig_protocol_store::keys::Own::Format, 64).expect("format key");
    let mut body = vec![1];
    body.extend(u32::try_from(key.len()).expect("key length").to_be_bytes());
    body.extend(key.as_ref());
    body.extend(u32::try_from(format.len()).expect("format length").to_be_bytes());
    body.extend(format);
    let mut frame = b"SKVC".to_vec();
    frame.extend(1_u16.to_be_bytes());
    frame.extend(1_u64.to_be_bytes());
    frame.extend(1_u32.to_be_bytes());
    frame.extend(u32::try_from(body.len()).expect("body length").to_be_bytes());
    frame.extend(body);
    frame.extend(jig_protocol_store::transcripts::frame::crc32c(&frame).to_be_bytes());
    let mut disk = Machine::for_user(1000);
    let root = disk.lay(&[
        Item::file(b"active", b"1\n"),
        Item::directory(b"generations"),
        Item::directory(b"generations/1"),
        Item::directory(b"generations/1/store"),
        Item::directory(b"generations/1/secrets").mode(0o700),
        Item::file(b"generations/1/store/log-00000000000000000001", &frame),
        Item::directory(b"transcripts"),
    ]);
    disk.close(root);
    let before = image(&mut disk);
    let (report, code, patches) =
        drive(&mut disk, &fixtures::document(), temper_engine_service::store::PATCH_SETS, Until::Check);
    eprintln!("{report}");
    assert_eq!(code, 1, "{report}");
    assert!(report.contains(expected), "{report}");
    assert!(report.contains("the copy keeps no wires"), "{report}");
    assert!(patches.is_empty());
    assert_eq!(image(&mut disk), before, "every source path and byte unchanged");
    let scratch = disk.reopen_root(1);
    assert!(disk.list(scratch, 32, 4096).expect("scratch list").is_empty(), "scratch emptied");
    disk.close(scratch);
}

type Image = BTreeMap<Vec<u8>, Option<Vec<u8>>>;
fn image(disk: &mut Machine) -> Image {
    let root = disk.reopen_root(0);
    let mut result = Image::new();
    tree(disk, root, b"", &mut result);
    disk.close(root);
    result
}
fn tree(disk: &mut Machine, root: Opened, prefix: &[u8], result: &mut Image) {
    loop {
        let entries = disk.list(root, 32, 4096).expect("directory entries");
        if entries.is_empty() {
            return;
        }
        for (kind, name) in entries {
            let mut path = prefix.to_vec();
            path.extend_from_slice(&name);
            let how = if kind == Is::Directory { How::DirectoryNoFollow } else { How::ReadNoFollow };
            let child = disk.open(root, &name, how).expect("image child");
            if kind == Is::Directory {
                result.insert(path.clone(), None);
                path.push(b'/');
                tree(disk, child, &path, result);
            } else {
                let size = u32::try_from(disk.stat(child).size).expect("world file length");
                result.insert(path, Some(disk.read(child, 0, size).expect("image bytes")));
            }
            disk.close(child);
        }
    }
}

pub fn rehearsal_then_start_and_back() {
    let scenario = world::Story::ChatReadThenWait { words: b"Hello" };
    let settings = fixtures::calm(fixtures::SEED);
    let (first, mut disk, _) = world::with_patch_sets(temper_engine_service::store::PATCH_SETS, || {
        world::run_phase(
            settings,
            scenario,
            world::Phase::Story,
            temper_playground::machine::fresh(),
            referee::Store::new(),
            Rc::new(RefCell::new(referee::Run::default())),
        )
    });
    assert!(first.drained);
    let document = fixtures::document();
    let original = image(&mut disk);
    let before = Maps::read(&mut disk);
    let (report, code, patches) = drive(&mut disk, &document, FIXTURE_SETS, Until::Check);
    assert_eq!(code, 0, "{report}");
    let patch = jig_protocol_store::levels(&FIXTURE_SETS).application.expect("fixture level");
    assert_eq!(patches, vec![(Owner::Application, patch, jig_protocol_store::Counts { records: 0, commits: 0 })]);
    assert!(report.contains(&format!("patched temper {patch}: 0 records in 0 commits")), "{report}");
    assert!(report.contains("patches and restores pass; what follows the switch is not rehearsed"), "{report}");
    assert!(report.contains("workers may stay: no") && report.contains("requires fresh workers"), "{report}");
    assert_eq!(image(&mut disk), original, "rehearsal leaves every deployment byte unchanged");
    let (_, _, actual) = drive(&mut disk, &document, FIXTURE_SETS, Until::Switched);
    assert_eq!(actual, patches, "start patches as rehearsed");
    assert_eq!(Maps::read(&mut disk).active, before.active + 1, "world fault after Switched");
    assert_eq!(Maps::generation(&mut disk, before.active), before, "previous generation unchanged");
    let (back, code, _) = drive(&mut disk, &document, FIXTURE_SETS, Until::Back);
    assert_eq!(code, 0, "{back}");
    assert_eq!(Maps::read(&mut disk), before, "back opens the exact previous generation");
    drive(&mut disk, &document, temper_engine_service::store::PATCH_SETS, Until::Wires);
    let restored = Maps::read(&mut disk);
    assert_eq!(restored.active, before.active, "old production build opened the previous generation");
    assert_eq!(restored.secrets, before.secrets, "previous secrets restored");
    drive(&mut disk, &document, FIXTURE_SETS, Until::Wires);
    let (back, code, _) = drive(&mut disk, &document, FIXTURE_SETS, Until::Back);
    assert_eq!(code, 1, "{back}");
    assert!(back.contains("past its first commit; going back is restoring a backup"), "{back}");
}

pub fn golden(directory: &Path) {
    stories::set(world::Story::PlaygroundA);
    stories::setup_settings(world::Phase::Story);
    let mut document: serde_json::Value =
        serde_json::from_slice(&std::fs::read(directory.join("configuration.json")).expect("golden configuration"))
            .expect("golden JSON");
    let sections = document.pointer_mut("/engine/limits/overrides/brief.sections").expect("brief sections");
    *sections = sections.as_u64().expect("count").max(u64::from(jig_domain_core::CORE_SECTIONS)).into();
    let temper_service::stock::Given::Own { jig, .. } = temper_playground::documents::given(&document) else {
        panic!("world defaults")
    };
    for (name, count) in [
        ("tasks.charters", jig.charters.len()),
        ("core.templates", jig.templates.len()),
        ("authority.executors", jig.rules.ceiling.delegation.kinds.len()),
    ] {
        let bound = document["engine"]["limits"]["overrides"].get_mut(name).expect("count override");
        *bound = bound.as_u64().expect("count").max(u64::try_from(count).expect("default count")).into();
    }
    let mut disk = crate::golden::files::load(directory);
    let before = image(&mut disk);
    let (report, code, patches) = drive(
        &mut disk,
        &serde_json::to_vec(&document).expect("configuration"),
        temper_engine_service::store::PATCH_SETS,
        Until::Check,
    );
    assert_eq!(code, 0, "{report}");
    let mut actual = String::new();
    for (of, patch, counts) in patches {
        assert!(
            report.contains(&format!(
                "patched {} {patch}: {} records in {} commits",
                temper_engine_shell::upgrade::owner(of),
                counts.records,
                counts.commits
            )),
            "{report}"
        );
        writeln!(actual, "{} {patch} {} {}", temper_engine_shell::upgrade::owner(of), counts.records, counts.commits)
            .expect("count line");
    }
    assert_eq!(actual, std::fs::read_to_string(directory.join("patched.txt")).expect("golden patch counts"));
    assert!(report.contains("patches and restores pass"), "{report}");
    assert_eq!(image(&mut disk), before);
}
