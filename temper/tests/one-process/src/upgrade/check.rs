//! Rehearsal and rollback on the one-process world's disk, service and configuration.
use super::{CURRENT_SETS, JOBS, first_sets, maps::Maps};
use crate::{fixtures, referee, stories, world};
use jig_protocol_store::{Owner, OwnerSets};
use skein_fake_machine::{How, Is, Machine, Opened};
use skein_sim::{Handle, Sim};
use std::{cell::RefCell, collections::BTreeMap, path::Path, rc::Rc};

pub(crate) mod pre_settings;

type Progress = Vec<temper_engine_service::Note>;
#[derive(Clone, Copy, Debug)]
enum Until {
    Check,
    Workers,
    Back,
    Switched,
    Wires,
}

fn configured(document: &temper_playground::documents::Configuration) -> temper_service::Service {
    let mut document = document.clone();
    let given = temper_playground::documents::given(&document);
    temper_playground::documents::settings::strip_models(&mut document);
    document.as_object_mut().expect("document").remove("world_settings");
    temper_service::startup::restoring_with_limits(
        &serde_json::to_vec(&document).expect("configuration bytes"),
        &jig_protocol_config::Directories::working(),
        &fixtures::files(),
        Some(&temper_playground::documents::agent()),
        &fixtures::seeds(),
        None,
        &given,
        &document.jig,
        &document.limits,
    )
    .unwrap_or_else(|why| panic!("{}", String::from_utf8_lossy(&temper_service::startup::lines(&why))))
}

#[expect(
    clippy::too_many_lines,
    reason = "one driver covers the restart cuts and both rehearsal reaches on the same disk"
)]
fn drive(
    disk: &mut Machine,
    document: &temper_playground::documents::Configuration,
    sets: OwnerSets,
    until: Until,
) -> (String, u8, Progress) {
    let mut service = configured(document);
    service.owner_sets(sets);
    service.prepared_schemas(crate::schemas::binary_schemas());
    let mut sim = Sim::new(fixtures::SEED, skein_sim::Config::calm());
    let pid = sim.spawn_process();
    let source = disk.reopen_root(0);
    let source = sim.root(pid, Handle::new(source.raw()));
    match until {
        Until::Check | Until::Workers => {
            let scratch = disk.lay(&[]);
            let scratch = sim.root(pid, Handle::new(scratch.raw()));
            service
                .rehearse_store(
                    source,
                    scratch,
                    1000,
                    match until {
                        Until::Workers => jig_protocol_store::Reach::Openings,
                        Until::Check => jig_protocol_store::Reach::Whole,
                        Until::Back | Until::Switched | Until::Wires => unreachable!("rehearsal"),
                    },
                )
                .expect("rehearsal adoption");
        }
        Until::Back => service.back_store(source, 1000).expect("back adoption"),
        Until::Switched | Until::Wires => service.adopt_store(source, 1000).expect("start adoption"),
    }
    if let Until::Check | Until::Workers | Until::Back = until {
        let owners = service.wait_state().engine.expect("upgrade engine");
        assert!(
            owners.workers.closed && owners.setup.closed && owners.forge.closed,
            "upgrade closes unstarted owners before store I/O: {owners:?}"
        );
    }
    let mut progress = Vec::new();
    for _ in 0..100_000 {
        temper_service::iterate(&mut service, sim.now(), sim.wall());
        while let Some(note) = service.notes().pop() {
            if let temper_service::Note::Engine(note @ temper_engine_service::Note::Ran { .. }) = note {
                progress.push(note);
            }
        }
        assert!(service.links().is_empty(), "no secret served before the switch/wires checkpoint");
        let wait = service.wait_state().engine.expect("engine state");
        let reached = match until {
            Until::Check | Until::Workers | Until::Back => service.is_empty(),
            Until::Switched => wait.restart == Some(jig_domain_core::start::Phase::Wires) && !wait.mark_durable,
            Until::Wires => wait.mark_durable,
        };
        if reached {
            let report = service.upgrade_report().map_or_else(
                || (String::new(), 0),
                |report| {
                    let render = match until {
                        Until::Workers => temper_engine_shell::upgrade::workers_lines,
                        Until::Check | Until::Back | Until::Switched | Until::Wires => {
                            temper_engine_shell::upgrade::lines
                        }
                    };
                    if let Until::Workers = until {
                        let events = report.events.iter().collect::<Vec<_>>();
                        let [
                            jig_protocol_store::Event::Rehearsing { openings, .. },
                            jig_protocol_store::Event::Closed { removed: true, .. },
                        ] = events.as_slice()
                        else {
                            panic!("{events:?}")
                        };
                        let [jig_protocol_store::KeptOpening::Current(opening)] = openings.as_ref() else {
                            panic!("one current opening: {openings:?}")
                        };
                        assert_eq!(opening.worker.as_ref(), "local");
                        let spoken = report.workers.spoken.as_slice();
                        assert_eq!(opening.wires.len(), spoken.len());
                        for kept in &opening.wires {
                            let wire = spoken.iter().find(|wire| wire.name == kept.name.as_ref()).expect("worker wire");
                            assert_eq!(kept.layout, wire.layout);
                            assert_eq!(kept.text.as_ref(), wire.text.as_bytes());
                        }
                    }
                    render(
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
            return (report.0, report.1, progress);
        }
        if let Some(end) = service.end() {
            assert!(service.upgrade_report().is_some(), "start refused: {end:?}");
        }
        if let Until::Check | Until::Workers | Until::Back = until {
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
/// including the layout before levels were added to version 3.
pub fn prebaseline_rehearsals() {
    let mut unleveled = jig_protocol_store::format::current(1, [7; 16]);
    unleveled.tasks = 9;
    unleveled.people = 14;
    unleveled.settings = 1;
    let before_levels =
        [0, 3, 0, 3, 0, 6, 0, 9, 0, 13, 0, 1, 0, 2, 0, 1, 0, 0, 0, 1, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7];
    for format in [unleveled.encode().expect("unleveled format"), Box::from(before_levels)] {
        source_rehearsal(&format, "No build translates a store from before the baseline");
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
    let frame = first_commit(&[(&key, format)]);
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
    let (report, code, jobs) =
        drive(&mut disk, &fixtures::document(), temper_engine_service::store::OWNER_SETS, Until::Check);
    eprintln!("{report}");
    assert_eq!(code, 1, "{report}");
    assert!(report.contains(expected), "{report}");
    assert!(report.contains("the copy keeps no wires"), "{report}");
    assert_eq!(jobs.len(), 0);
    assert_eq!(image(&mut disk), before, "every source path and byte unchanged");
    let scratch = disk.reopen_root(1);
    assert!(disk.list(scratch, 32, 4096).expect("scratch list").is_empty(), "scratch emptied");
    disk.close(scratch);
}

fn first_commit(rows: &[(&[u8], &[u8])]) -> Vec<u8> {
    let mut body = Vec::new();
    for (key, value) in rows {
        body.push(1);
        body.extend(u32::try_from(key.len()).expect("key length").to_be_bytes());
        body.extend_from_slice(key);
        body.extend(u32::try_from(value.len()).expect("value length").to_be_bytes());
        body.extend_from_slice(value);
    }
    let mut frame = b"SKVC".to_vec();
    frame.extend(1_u16.to_be_bytes());
    frame.extend(1_u64.to_be_bytes());
    frame.extend(u32::try_from(rows.len()).expect("row count").to_be_bytes());
    frame.extend(u32::try_from(body.len()).expect("body length").to_be_bytes());
    frame.extend(body);
    frame.extend(jig_protocol_store::transcripts::frame::crc32c(&frame).to_be_bytes());
    frame
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

fn stopped_chat(sets: OwnerSets) -> Machine {
    let scenario = world::Story::ChatReadThenWait { words: b"Hello" };
    let settings = fixtures::calm(fixtures::SEED);
    let (first, disk, _) = world::with_owner_sets(sets, || {
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
    disk
}

/// A worker's real accepted statement survives the stopped deployment and reaches the tool's verdict.
pub fn opened_worker_rehearsal() {
    let mut disk = stopped_chat(CURRENT_SETS);
    let document = fixtures::document();
    let before = image(&mut disk);
    let (report, code, jobs) = drive(&mut disk, &document, temper_engine_service::store::OWNER_SETS, Until::Check);
    eprintln!("{report}");
    assert_eq!(code, 0, "{report}");
    assert!(jobs.is_empty(), "this equal-level rehearsal runs no jobs");
    // The box defaults to local; the world starts at 2026-01-01 UTC.
    assert!(report.contains("temper-box upgrade: worker local may stay: opened 2026-01-01T00:00:00Z\n"), "{report}");
    assert!(report.ends_with("temper-box upgrade: workers may stay: yes\n"), "{report}");
    assert_eq!(image(&mut disk), before, "rehearsal leaves the stopped deployment unchanged");
    let (workers, code, jobs) = drive(&mut disk, &document, temper_engine_service::store::OWNER_SETS, Until::Workers);
    assert_eq!(code, 0, "{workers}");
    assert!(jobs.is_empty(), "Openings runs no job");
    let verdict = report.split_inclusive('\n').filter(|line| line.contains(" upgrade: worker")).collect::<String>();
    // The stopped-chat fixture copies its two map files and has no sealed files to link.
    let counts = "temper-box upgrade: rehearsal linked 0 files, copied 2\n";
    assert_eq!(workers, format!("{counts}{verdict}"), "file counts precede the check's worker verdict");
    assert_eq!(image(&mut disk), before, "workers leaves every source path and byte unchanged");
}

pub fn rehearsal_then_start_and_back() {
    let mut disk = stopped_chat(first_sets());
    let document = fixtures::document();
    let original = image(&mut disk);
    let before = Maps::read(&mut disk);
    let (report, code, progress) = drive(&mut disk, &document, CURRENT_SETS, Until::Check);
    assert_eq!(code, 0, "{report}");
    assert_eq!(progress.len(), JOBS.len(), "each pending job runs");
    for (note, expected) in progress.iter().zip(JOBS) {
        let temper_engine_service::Note::Ran { of, job, rows, commits, counts } = note else {
            panic!("job progress: {note:?}")
        };
        assert_eq!((*of, job.as_ref()), (super::job_owner(expected), expected));
        assert!(*commits > 0, "the job's end is committed");
        assert!(
            counts.iter().all(|count| count.count == 0),
            "no task rejection, long id or deleted repository in this story"
        );
        let owner = if *of == Owner::Jig { "jig" } else { "temper" };
        assert!(report.contains(&format!("ran {owner} {expected}: {rows} rows in {commits} commits")), "{report}");
    }
    assert!(report.contains("registry commit, jobs and restores pass; opened in "), "{report}");
    assert!(report.ends_with("temper-box upgrade: workers may stay: yes\n"), "{report}");
    assert_eq!(image(&mut disk), original, "rehearsal leaves every deployment byte unchanged");
    let (_, _, actual) = drive(&mut disk, &document, CURRENT_SETS, Until::Switched);
    assert_eq!(actual, progress, "start runs the job as rehearsed");
    let after = Maps::read(&mut disk);
    super::registry_upgrade(&before, &after);
    assert_eq!(after.active, before.active + 1, "world fault after Switched");
    assert_eq!(Maps::generation(&mut disk, before.active), before, "previous generation unchanged");
    let (back, code, _) = drive(&mut disk, &document, CURRENT_SETS, Until::Back);
    assert_eq!(code, 0, "{back}");
    assert_eq!(Maps::read(&mut disk), before, "back opens the exact previous generation");
    drive(&mut disk, &document, first_sets(), Until::Wires);
    let restored = Maps::read(&mut disk);
    assert_eq!(restored.active, before.active, "first build opened the previous generation");
    assert_eq!(restored.secrets, before.secrets, "previous secrets restored");
    drive(&mut disk, &document, CURRENT_SETS, Until::Wires);
    let (back, code, _) = drive(&mut disk, &document, CURRENT_SETS, Until::Back);
    assert_eq!(code, 1, "{back}");
    assert!(back.contains("past its first commit; going back is restoring a backup"), "{back}");
}

pub fn golden(directory: &Path) {
    stories::set(world::Story::PlaygroundA);
    stories::setup_settings(world::Phase::Story);
    let document = crate::golden::read_configuration(directory);
    let mut disk = crate::golden::files::load_kept(directory);
    let before = image(&mut disk);
    let (report, code, jobs) = drive(&mut disk, &document, temper_engine_service::store::OWNER_SETS, Until::Check);
    assert_eq!(code, 0, "{report}");
    assert!(jobs.is_empty(), "floor store needs no jobs");
    assert!(report.contains("registry commit, jobs and restores pass; opened in "), "{report}");
    assert_eq!(image(&mut disk), before);
}
