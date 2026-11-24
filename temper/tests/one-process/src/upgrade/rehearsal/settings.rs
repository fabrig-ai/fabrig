//! Stopped settings and people records through the shell's store-only opening (temper/docs/design/protocol/configuration/limit-settings.md, 3).
use super::live::{OWNER, accounts, settle};
use crate::{
    fixtures,
    golden::files,
    testing::{Disk, USER},
};
use jig_domain_core::{CoreRecord, Deployment, Record, settings as s};
use jig_protocol_store::{BackRefusal, Event, Generations, OpenRefusal, Request, Write};
use skein_fake_machine::{How, Item, Machine};
use skein_lib::{List, Wall};
use temper_engine_service::{self as service, startup, store::Store, upgrade::Report};
use temper_engine_shell::{
    serve::Opening,
    testing::{parse_held, rehearse_opening},
};
use temper_shell::directories::Directories;

const DATA: u32 = 0;
const SCRATCH: u32 = 1;
const STATE: &[u8] = b"state";
const SOURCE: &[u8] = b"/data/state";
const EMPTY_ROOT_ENTRIES: usize = 1; // The inventory includes the root itself.
const BUILD: &str = "test-build";
const HOLD_FILE: &[u8] = b"limits-held";
// The floor fixture's first person and the held figure from the startup boundary.
const PERSON: u64 = 1;
const IDENTITY_BOUND: &[u8] = b"people.identity_bytes 7\n";
const IDENTITY_BYTES: u64 = 8; // The subject and login below, one past the held bound.
const STOCK_VERSIONS: u32 = 16; // jig-protocol-config's standard profile.
const HELD_VERSIONS: u32 = STOCK_VERSIONS + 1;
const RAISED_VERSIONS: u64 = 64; // The binary witness's copy uses the same raise.
const TEMPLATE_HOLE: u32 = 9;
const CHARTER_HOLE: u32 = 17;
// The two numbering ranges are 1–32 and 33–64.
const FIRST_OWN: u32 = 33;
const PAST_OWN: u32 = 65;
const FIRST_CHARTER: u32 = 1;

pub struct Rehearsal {
    document: Box<[u8]>,
    directories: Directories,
    given: service::stock::Given,
    jig: jig_protocol_config::Directories,
    files: service::settings::Files,
    seeds: service::settings::Seeds,
    engine: jig_protocol_config::Engine,
}

impl Rehearsal {
    #[must_use]
    pub fn new(document: Box<[u8]>, directories: Directories, given: service::stock::Given) -> Self {
        let jig = temper_service_shell::jig_directories(&directories);
        let engine = temper_engine_shell::serve::settings(&document, &directories, &given);
        Self {
            document,
            directories,
            given,
            jig,
            files: service::settings::Files { entries: List::with_capacity(0) },
            seeds: fixtures::seeds(),
            engine,
        }
    }

    /// Neither tool creates a deployment in an existing empty directory.
    pub fn empty(&self) {
        let mut disk = disk(None);
        let refusal = service::End::Stopped(service::Stop::StoreOpen {
            why: OpenRefusal::Generations { why: Generations::Unreadable },
        });
        self.check(&mut disk, |report, _| assert_eq!(report.end, Some(refusal)));
        self.back(&mut disk, BackRefusal::Unreadable);
    }

    /// A separate handle keeps the configured source busy through store-first opening.
    pub fn held_source(&self, mut machine: Machine) {
        let root = machine.reopen_root(DATA);
        let held = machine.open(root, STATE, How::DirectoryNoFollow).expect("floor's source");
        machine.close(root);
        machine.lock(held).expect("another process holds the source");
        let mut disk = Disk::new(machine);
        let (text, code) = self.check(&mut disk, |report, settings| {
            assert_eq!(
                report.end,
                Some(service::End::Stopped(service::Stop::StoreOpen {
                    why: OpenRefusal::Generations { why: Generations::Busy },
                }))
            );
            assert!(settings.is_none(), "the copy never opened");
        });
        assert_eq!(code, 1, "{text}");
        assert!(text.contains("temper-engine serve: Another process has /data/state open"), "{text}");
        disk.machine.close(held);
    }

    /// The configured, settled floor has no previous generation to reactivate.
    pub fn no_previous(&self, machine: Machine) {
        let (text, code) = self.back(&mut Disk::new(machine), BackRefusal::NoPrevious);
        assert_eq!(code, 1, "{text}");
        assert!(
            text.contains("temper-engine upgrade: no previous generation; restore a backup to go back\n"),
            "{text}"
        );
    }

    fn back(&self, disk: &mut Disk, refusal: BackRefusal) -> (String, u8) {
        let before = files::image_root(&mut disk.machine, DATA);
        let limits = startup::resolve_limits(&self.inputs(), &[], &[]).expect("stock limits");
        let mut service = self.service(&limits);
        service.engine_component().back_store(disk.directory(DATA, STATE), USER).expect("back adopts source");
        settle(disk, &mut service);
        let report = service.upgrade_report().expect("back report");
        assert!(report.done);
        assert!(report.events.iter().any(|event| matches!(
            event,
            Event::BackRefused { why, .. } if *why == refusal
        )));
        files::unchanged(&before, &files::image_root(&mut disk.machine, DATA));
        temper_engine_shell::upgrade::lines("temper-engine", report, &self.directories, SOURCE)
    }

    /// The copy's held identity limit refuses the kept person's record, not the owner seed.
    pub fn person(&self) {
        use jig_domain_core::{
            Key,
            start::{Failure, LiveSet, Phase},
        };
        use jig_domain_people::{Identity, IdentityKey, RestoreBound, Stored};
        let mut disk = disk(Some(IDENTITY_BOUND));
        let record = Stored::Person {
            number: PERSON,
            identity: Identity {
                key: IdentityKey { provider: 0, subject: b"a@b".as_slice().into() },
                login: b"owner".as_slice().into(),
                name: Box::new([]),
            },
        };
        self.seed(&mut disk, vec![Write::Save(Record::People(record))], PERSON);
        self.check(&mut disk, |report, _| {
            assert_eq!(
                report.end,
                Some(service::End::Stopped(service::Stop::StartRefused {
                    phase: Some(Phase::LoadCore),
                    failure: Failure::LiveSet {
                        set: LiveSet::PeopleBound(RestoreBound::IdentityBytes),
                        key: Key::People(jig_domain_people::Key::Person(PERSON)),
                        held: IDENTITY_BYTES,
                        limit: IDENTITY_BYTES - 1,
                    },
                }))
            );
        });
    }

    /// A pre-release change kept in a stock hole still restores after moving to own numbers.
    pub fn kept_and_moved(&self) {
        let mut disk = disk(None);
        self.seed(&mut disk, self.numbered(Numbers::Kept), 0);
        self.check(&mut disk, passed);
        self.seed(&mut disk, self.numbered(Numbers::Moved), 0);
        self.check(&mut disk, passed);
    }

    pub fn past(&self) {
        use jig_domain_core::start::{Failure, Numbered};
        let mut disk = disk(None);
        self.seed(&mut disk, self.numbered(Numbers::Past), 0);
        self.check(&mut disk, |report, _| {
            let Some(service::End::Stopped(service::Stop::StartRefused {
                failure: Failure::Numbers { settings }, ..
            })) = &report.end
            else {
                panic!("numbers refusal: {:?}", report.end)
            };
            assert!(settings.contains(&Numbered { setting: s::Setting::Charter(PAST_OWN), changed: true }));
        });
    }

    pub fn taken(&self) {
        use jig_domain_core::start::Failure;
        let mut disk = disk(None);
        self.seed(&mut disk, self.numbered(Numbers::Taken), 0);
        self.check(&mut disk, |report, _| {
            let Some(service::End::Stopped(service::Stop::StartRefused {
                failure: Failure::Taken { settings }, ..
            })) = &report.end
            else {
                panic!("taken numbers refusal: {:?}", report.end)
            };
            assert_eq!(settings.as_ref(), &[s::Setting::Charter(FIRST_CHARTER)]);
        });
    }

    /// The store holds one version past stock, even though its own durable limit admits it.
    pub fn limits_held(&self) {
        use jig_domain_core::{
            HeldLimit,
            start::{Failure, Phase},
        };
        let mut disk = disk(Some(b""));
        self.seed(&mut disk, self.raised_versions(), 0);
        self.check(&mut disk, |report, settings| {
            let state = settings.expect("the opened copy's limits");
            assert!(state.held);
            assert!(state.jig_taken.is_empty() && state.temper_taken.is_empty());
            assert_eq!(state.in_force.jig.core.settings.versions, STOCK_VERSIONS);
            assert_eq!(
                report.end,
                Some(service::End::Stopped(service::Stop::StartRefused {
                    phase: Some(Phase::LoadCore),
                    failure: Failure::HeldPastLimit {
                        which: HeldLimit::Versions,
                        held: HELD_VERSIONS,
                        limit: STOCK_VERSIONS
                    },
                }))
            );
        });
    }

    fn inputs(&self) -> startup::LimitInputs {
        startup::LimitInputs::new(&self.document, &self.jig, &self.files, None, &self.seeds, &self.given, false, None)
    }

    fn store(&self) -> Store {
        let models = jig_protocol_config::core(&self.engine, self.seeds.deployment, self.seeds.seed).settings.models;
        let mut store = Store::for_limits(&self.inputs(), &models).expect("stock-sized opening store");
        store.build_name(BUILD);
        store.prepared_schemas(temper_playground::programs::prepared_schemas());
        store
    }

    fn service(&self, limits: &startup::Resolved) -> service::Service {
        let startup::CheckedStart { engine, io, forges, .. } = startup::checked_start_with_resolved(
            &self.document,
            &self.jig,
            &self.files,
            None,
            &self.seeds,
            List::with_capacity(0),
            None,
            &self.given,
            limits,
        )
        .expect("checked stopped engine");
        startup::restoring_prepared(engine, &self.seeds, Some(accounts(&self.engine)), forges, io)
            .expect("stopped service")
    }

    fn check(
        &self,
        disk: &mut Disk,
        assert_report: impl FnOnce(&Report, Option<&service::limit_settings::LimitSettings>),
    ) -> (String, u8) {
        let before = files::image_root(&mut disk.machine, DATA);
        let mut store = self.store();
        let began = disk.sim.now();
        store.adopt_rehearsal(disk.directory(DATA, STATE), disk.root(SCRATCH), USER).expect("adopt rehearsal roots");
        let held = hold(&mut disk.machine, &self.directories);
        let mut diagnostics = Vec::new();
        let Opening { store, settings, began } =
            rehearse_opening(store, disk, began, self.inputs(), held.as_ref(), &mut diagnostics)
                .expect("check opening");
        let fallback;
        let limits = if let Some(settings) = &settings {
            &settings.in_force
        } else {
            fallback = startup::resolve_limits(&self.inputs(), &[], &[]).expect("stock limits");
            &fallback
        };
        let mut service = self.service(limits);
        service.engine_component().rehearse_opened(store, settings, began);
        settle(disk, &mut service);
        let engine = service.engine_component();
        let report = engine.upgrade_report().expect("check report");
        assert!(report.done);
        assert_report(report, engine.limits_in_force());
        let (text, code) = temper_engine_shell::upgrade::lines("temper-engine", report, &self.directories, SOURCE);
        assert_eq!(code, u8::from(!report.restored), "{text}");
        assert_eq!(text.contains("registry commit, jobs and restores pass; opened in "), report.restored, "{text}");
        if report.restored {
            assert!(!text.contains("settings out of their numbers"), "{text}");
        }
        assert!(diagnostics.is_empty(), "unexpected limit warning: {}", String::from_utf8_lossy(&diagnostics));
        files::unchanged(&before, &files::image_root(&mut disk.machine, DATA));
        assert_eq!(files::image_root(&mut disk.machine, SCRATCH).len(), EMPTY_ROOT_ENTRIES, "scratch emptied");
        (text, code)
    }

    fn seed(&self, disk: &mut Disk, mut writes: Vec<Write>, people: u64) {
        let mut store = self.store();
        store.adopt(disk.directory(DATA, STATE), USER).expect("adopt settings fixture");
        let Event::Opened { last, pending, .. } = disk.event(&mut store) else { panic!("fixture opened") };
        if pending {
            store.ask(Request::Switch { owner: OWNER });
            assert!(matches!(disk.event(&mut store), Event::Switched { .. }));
        }
        let number = last + 1;
        writes.extend([
            Write::Save(Record::Core(CoreRecord::Deployment(Deployment {
                id: self.seeds.deployment,
                restart_wait_ms: None,
                wait_floors: None,
                tasks: 0,
                people,
                sign_ins: 0,
                messages: 0,
                runs: 0,
                calls: 0,
                connector_rows: 0,
                applications: 0,
                commits: number,
                projects: 0,
            }))),
            Write::Wires,
        ]);
        store.ask(Request::Commit { number, writes: writes.into_boxed_slice() });
        assert!(matches!(disk.event(&mut store), Event::Committed { number: durable, .. } if durable == number));
        disk.close(store);
    }

    fn numbered(&self, case: Numbers) -> Vec<Write> {
        let moved = case == Numbers::Moved;
        let mut template = self.engine.templates.first().expect("stock template").clone();
        let mut charter = self.engine.charters.first().expect("stock charter").clone();
        template.number = if moved { FIRST_OWN } else { TEMPLATE_HOLE };
        charter.number = match case {
            Numbers::Kept => CHARTER_HOLE,
            Numbers::Moved => FIRST_OWN + 1,
            Numbers::Past => PAST_OWN,
            Numbers::Taken => FIRST_CHARTER,
        };
        let taken = (case == Numbers::Taken).then_some(s::Setting::Charter(charter.number));
        let mut writes = if moved { Vec::new() } else { self.defaults(taken) };
        let revision = if moved { 2 } else { 1 };
        writes.extend([
            changed(s::Setting::Template(template.number), Some(s::Value::Template(Box::new(template))), revision),
            changed(s::Setting::Charter(charter.number), Some(s::Value::Charter(Box::new(charter))), revision),
        ]);
        if moved {
            writes.extend([
                changed(s::Setting::Template(TEMPLATE_HOLE), None, revision),
                changed(s::Setting::Charter(CHARTER_HOLE), None, revision),
            ]);
        }
        writes.push(head(revision, if moved { s::Decider::Release } else { s::Decider::FirstStart }));
        writes
    }

    fn raised_versions(&self) -> Vec<Write> {
        let charter = self.engine.charters.first().expect("stock charter").number;
        let revision = HELD_VERSIONS + 1;
        let mut writes = self.defaults(None);
        let raise = s::Override {
            name: b"core.settings.versions".as_slice().into(),
            value: RAISED_VERSIONS,
            reason: b"the copy holds more versions than stock".as_slice().into(),
        };
        writes.push(changed(s::Setting::Limits, Some(s::Value::Limits(Box::new([raise]))), revision));
        for version in 1..=HELD_VERSIONS {
            writes.push(setting(s::Record::Charter {
                charter,
                revision: version,
                value: s::CharterVersion { overlay: None, charter: None },
            }));
        }
        writes.push(head(revision, s::Decider::FirstStart));
        writes
    }

    fn defaults(&self, taken: Option<s::Setting>) -> Vec<Write> {
        let core = jig_protocol_config::core(&self.engine, self.seeds.deployment, self.seeds.seed);
        let current = s::Current::new(core.defaults, &self.engine.limits.core, &[]);
        let mut writes = Vec::new();
        for (key, default) in &current.defaults {
            if Some(*key) != taken {
                writes.push(setting(s::Record::Default { setting: *key, default: default.clone() }));
            }
        }
        writes
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Numbers {
    Kept,
    Moved,
    Past,
    Taken,
}

fn disk(held: Option<&[u8]>) -> Disk {
    let mut machine = Machine::for_user(USER);
    let mut data = vec![Item::directory(STATE).mode(0o700)];
    if let Some(bytes) = held {
        data.push(Item::file(HOLD_FILE, bytes).mode(0o600));
    }
    for items in [data.as_slice(), &[]] {
        let root = machine.lay(items);
        machine.close(root);
    }
    Disk::new(machine)
}

fn hold(machine: &mut Machine, directories: &Directories) -> Option<temper_engine_shell::serve::Held> {
    use std::{os::unix::ffi::OsStrExt, path::Path};
    let root = machine.reopen_root(DATA);
    let file = machine.open(root, HOLD_FILE, How::ReadNoFollow);
    machine.close(root);
    match file {
        Ok(file) => {
            let length = u32::try_from(machine.stat(file).size).expect("small held fixture");
            let bytes = machine.read(file, 0, length).expect("held fixture bytes");
            machine.close(file);
            let path = Path::new(std::ffi::OsStr::from_bytes(&directories.data)).join("limits-held");
            Some(parse_held(&path, std::str::from_utf8(&bytes).expect("held fixture text")).expect("held limits"))
        }
        Err(skein_fake_machine::Refusal::NotFound) => None,
        Err(why) => panic!("held fixture: {why:?}"),
    }
}

fn passed(report: &Report, _: Option<&service::limit_settings::LimitSettings>) {
    assert!(report.restored, "restoring the numbered settings reaches the opening: {:?}", report.end);
    assert_eq!(report.end, None);
}

fn setting(record: s::Record) -> Write {
    Write::Save(Record::Core(CoreRecord::Settings(record)))
}

fn changed(key: s::Setting, value: Option<s::Value>, revision: u32) -> Write {
    setting(s::Record::Changed { setting: key, changed: s::Changed { value, revision } })
}

fn head(number: u32, by: s::Decider) -> Write {
    setting(s::Record::Revision {
        history: false,
        revision: s::Revision {
            head: s::Head { number, against: number - 1, at: Wall::EPOCH, by, proposal: None, moved: false },
            items: Box::new([]),
            applications: Box::new([]),
        },
    })
}
