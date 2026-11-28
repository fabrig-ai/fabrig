//! Root decisions over a fake forge and the held startup store.
use super::resume_fixture::{StartStore, config, limits};
use crate::{Domain, Event, Fact, Limits, ProjectChartersDiagnostic, ProjectChartersFault, Record, Request, Write};
use jig_domain_core::engine::store as engine_store;
use jig_domain_core::{self as core, CharterPart, ProjectCharter, ProjectCharters};
use jig_domain_tasks::{self as tasks, Reasons};
use skein_fake_forge_domain::{self as fake, api as raw};
use skein_forge::api::{self, Answer, Commit, Error, Op, Read};
use skein_lib::{Duration, Env, Queue, ReplyTo, Time, Token, Wall, bytes};
use temper_engine_domain_forge as forge;

const PROJECT: u32 = 1;
const HOME: api::Repository = api::Repository { forge: 1, repository: 1 };
const REPOSITORY: &[u8] = b"acme/shop";
const MAIN: &[u8] = b"main";
const WAYS: &[u8] = b".temper/ways.toml";
const IMPLEMENTER: &[u8] = b".temper/charters/implementer.md";
const CAREFUL: &[u8] = b".temper/charters/careful.md";
const OFFERS: &[u8] = b"offer = [\"build\"]\n";
const WAY: &[u8] = b".temper/ways/build.toml";
const BUILD: &[u8] = b"format = 1\n";
const WORDS: &[u8] = b"Be careful.";
const PART: &[u8] = b"Name the risk before proceeding.";
const CAREFUL_WORDS: &[u8] = b"+++\nsettings = \"implementer\"\n+++\nRead the brief.\n\n## part, on: first, question\nName the risk before proceeding.\n";
const BAD_WORDS: &[u8] = b"## part, on: frist\nThis part does not read.";
const EXAMPLE: Commit = [
    0x1a, 0x2b, 0x3c, 0x4d, 0x5e, 0x6f, 0x7a, 0x8b, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0,
];

#[test]
fn a_moved_home_revision_gives_or_clears_charters_in_its_own_commit() {
    let mut world = World::new(&[(IMPLEMENTER, WORDS)]);
    let charters_only = world.tip();
    let (first, sets) = world.revision_commit();
    assert!(first.method && first.refused.is_none());
    assert_eq!(first.number, 1);
    assert_eq!(sets, None, "charters without ways.toml do not opt in");
    assert_eq!(world.root.engine.core().project_charters(PROJECT), 0);

    world.advance(CAREFUL, CAREFUL_WORDS);
    world.advance(WAY, BUILD);
    world.advance(WAYS, OFFERS);
    let picked = world.tip();
    world.notice();
    let (row, sets) = world.revision_commit();
    assert_eq!(row.number, 2);
    assert_eq!(row.commit, Some(fake::commit_bytes(picked)));
    let hex = crate::commit_text(&row.commit.unwrap());
    let source = bytes::concat(&[REPOSITORY, b" at ", &hex[..12]]);
    assert_eq!(
        sets,
        Some(ProjectCharters {
            project: PROJECT,
            revision: row.number,
            source,
            charters: Box::new([careful(), implementer()]),
        })
    );
    assert_eq!(world.root.engine.core().project_charters(PROJECT), row.number);
    assert!(world.root.pop_fact().is_none());

    // Hooks are read but are not part of the revision. This good read merely
    // advances the observation, not the revision number or its set.
    world.advance(b".temper/hooks/config.toml", b"");
    world.notice();
    let (same, sets) = world.revision_commit();
    assert_eq!(same.observed, row.observed + 1);
    assert_eq!((same.number, same.commit, same.refused), (row.number, row.commit, None));
    assert_eq!(sets, None);
    assert!(world.root.pop_fact().is_none(), "the unchanged number is not even offered");

    fake::reset_branch(&mut world.forge, &world.fake_env, REPOSITORY, MAIN, charters_only, 7).unwrap();
    world.notice();
    let (removed, sets) = world.revision_commit();
    assert_eq!((removed.number, removed.commit, removed.refused), (3, Some(fake::commit_bytes(charters_only)), None));
    let cleared = sets.expect("removing ways.toml clears even retained charter files");
    assert_eq!((cleared.project, cleared.revision), (PROJECT, 3));
    assert!(cleared.charters.is_empty(), "the replacement set is empty");
    assert_eq!(world.root.engine.core().project_charters(PROJECT), removed.number);
}

#[test]
fn a_project_without_method_or_home_keeps_the_zero_task_stamp() {
    let mut world = World::new(&[]);
    let (row, sets) = world.revision_commit();
    assert!(!row.method);
    assert_eq!(sets, None);
    assert_zero_stamp(&mut world.root, &world.env);

    let limits = limits();
    let mut store = old_store(&limits, false);
    store.keep(revision(&[(IMPLEMENTER, WORDS), (WAYS, OFFERS), (WAY, BUILD)]));
    let (mut root, env) = start(&limits, &mut store, true);
    assert!(root.forge.home(PROJECT).is_none());
    assert!(root.forge.temper_revision(PROJECT).is_some(), "last good revision outlives its home status");
    assert_zero_stamp(&mut root, &env);
}

#[test]
fn start_backfills_an_old_revision_and_does_not_offer_it_again() {
    let limits = limits();
    let mut store = old_store(&limits, true);
    store.keep(revision(&[(IMPLEMENTER, WORDS), (WAYS, OFFERS), (WAY, BUILD)]));
    let expected = ProjectCharters {
        project: PROJECT,
        revision: 4,
        source: b"acme/shop at 1a2b3c4d5e6f".as_slice().into(),
        charters: Box::new([implementer()]),
    };
    let durable = saved_set(expected.clone()).key();
    assert!(!store.rows.contains_key(&durable));
    for _ in 0..2_u32 {
        let (mut root, _) = start(&limits, &mut store, true);
        assert_eq!(root.engine.core().project_charters(PROJECT), 4);
        assert_eq!(root.engine.core().project_charter_set(PROJECT, 4), Some(&expected));
        assert_eq!(store.rows.get(&durable), Some(&saved_set(expected.clone())));
        assert!(root.pop_fact().is_none(), "a matching restored revision is not offered again");
    }
}

#[test]
fn start_reports_a_core_refusal_or_an_unreadable_file_and_keeps_the_previous_set() {
    let limits = limits();
    for (text, expected) in [
        (
            CAREFUL_WORDS,
            ProjectChartersFault::Refused(core::ProjectChartersFault::Count {
                charter: b"implementer".as_slice().into(),
                bound: 1,
            }),
        ),
        (
            BAD_WORDS,
            ProjectChartersFault::Unreadable {
                path: CAREFUL.into(),
                line: 1,
                why: "`## part, on: frist`: `frist` is no reason a part is selected by".into(),
            },
        ),
    ] {
        let mut store = old_store(&limits, true);
        let previous = ProjectCharters {
            project: PROJECT,
            revision: 3,
            source: REPOSITORY.into(),
            charters: Box::new([implementer()]),
        };
        store.keep(saved_set(previous.clone()));
        store.keep(revision(&[(CAREFUL, text), (IMPLEMENTER, WORDS), (WAYS, OFFERS), (WAY, BUILD)]));
        let (mut root, _) = start(&limits, &mut store, true);
        assert_eq!(
            root.pop_fact(),
            Some(Fact::ProjectCharters(ProjectChartersDiagnostic { project: PROJECT, revision: 4, fault: expected }))
        );
        assert!(root.pop_fact().is_none(), "the rejected revision was offered once");
        assert_eq!(root.engine.core().project_charters(PROJECT), 3);
        assert_eq!(root.engine.core().project_charter_set(PROJECT, 3), Some(&previous));
    }
}

fn assert_zero_stamp(root: &mut Domain, env: &Env<Limits>) {
    let task = super::projection_settled::make_task(
        root,
        env,
        None,
        tasks::Executor::Agent { charter: 1 },
        tasks::Party::Deployment { project: PROJECT },
    );
    assert_eq!(root.engine.core().project_charters(PROJECT), 0);
    assert_eq!(root.tasks().task(task).unwrap().charters, 0);
}

fn implementer() -> ProjectCharter {
    ProjectCharter {
        name: b"implementer".as_slice().into(),
        settings: b"implementer".as_slice().into(),
        instructions: WORDS.into(),
        parts: Box::new([]),
    }
}

fn careful() -> ProjectCharter {
    ProjectCharter {
        name: b"careful".as_slice().into(),
        settings: b"implementer".as_slice().into(),
        instructions: b"Read the brief.".as_slice().into(),
        parts: Box::new([CharterPart { on: Reasons::FIRST.union(Reasons::QUESTION), instructions: PART.into() }]),
    }
}

fn saved_set(set: ProjectCharters) -> Record {
    Record::Core(core::Record::Core(core::CoreRecord::Settings(core::settings::Record::ProjectCharters(set))))
}

#[expect(clippy::disallowed_methods, reason = "build the store fixture from its listed files")]
fn revision(files: &[(&[u8], &[u8])]) -> Record {
    Record::Forge {
        row: Box::new(forge::Stored::DotTemperRevision(forge::DotTemperRevision {
            project: PROJECT,
            number: 4,
            home: HOME,
            commit: Some(EXAMPLE),
            method: true,
            files: files
                .iter()
                .map(|(path, bytes)| forge::DotTemperFile { path: (*path).into(), bytes: (*bytes).into() })
                .collect(),
            observed: 5,
            refused: None,
        })),
    }
}

fn old_store(limits: &Limits, home: bool) -> StartStore {
    let mut store = StartStore::new();
    start(limits, &mut store, false);
    let mut repository = super::owned_repository(HOME.forge);
    repository.project = PROJECT;
    repository.owner = b"acme".as_slice().into();
    repository.name = b"shop".as_slice().into();
    repository.home = home;
    store.keep(Record::Forge { row: Box::new(forge::Stored::Repository(repository)) });
    store
}

fn start(limits: &Limits, store: &mut StartStore, pending: bool) -> (Domain, Env<Limits>) {
    let mut config = config(7);
    config.core.defaults.charters[0].name = Some(b"implementer".as_slice().into());
    let mut root = Domain::new(config, limits).unwrap();
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: *limits };
    crate::step(&mut root, &env, Event::Restart { pending });
    store.run(&mut root, &env, false);
    (root, env)
}

struct World {
    root: Domain,
    env: Env<Limits>,
    store: StartStore,
    forge: fake::Domain,
    fake_env: Env<fake::Config>,
}

impl World {
    #[expect(clippy::disallowed_methods, reason = "build the fake forge tree from its listed files")]
    fn new(files: &[(&[u8], &[u8])]) -> Self {
        let mut limits = limits();
        limits.tasks.charters = 2;
        let mut store = old_store(&limits, true);
        let (root, env) = start(&limits, &mut store, true);
        assert_eq!(root.forge.home(PROJECT).map(|repo| repo.provider), Some(HOME));
        let mut config = fake::Config::test();
        config.latency_min = Duration::ZERO;
        config.latency_max = Duration::ZERO;
        config.limits.files = 5; // The offered way, two charters, and the hook-only change.
        let fake_env = env.with(config);
        let mut forge = fake::Domain::new(&fake_env.limits, 7);
        fake::repository(
            &mut forge,
            &fake_env.limits,
            raw::Setup {
                id: HOME.repository,
                name: REPOSITORY.into(),
                default: MAIN.into(),
                tree: files
                    .iter()
                    .map(|(path, bytes)| raw::File { path: (*path).into(), content: (*bytes).into() })
                    .collect(),
                labels: Box::new([]),
                checks: fake_env.limits.made_checks.clone(),
                workflows: Box::new([]),
                protection: None,
                hooked: false,
            },
        );
        fake::grant(&mut forge, REPOSITORY, 7, raw::Permission::Admin);
        Self { root, env, store, forge, fake_env }
    }

    fn tip(&self) -> u64 {
        self.forge.branch(REPOSITORY, MAIN).unwrap()
    }

    fn advance(&mut self, path: &[u8], text: &[u8]) {
        fake::advance(&mut self.forge, &self.fake_env, REPOSITORY, MAIN, path, text, 7).unwrap();
    }

    fn notice(&mut self) {
        self.event(Event::Forge(Box::new(forge::Event::Hint {
            hint: api::Hint { repository: HOME, change: api::Change::Branch(MAIN.into()), key: None },
        })));
        self.env.now = self.env.now.saturating_add(self.env.limits.forge.client.hinted);
        self.fake_env.now = self.env.now;
        crate::fire(&mut self.root, &self.env);
    }

    fn event(&mut self, event: Event) {
        crate::step(&mut self.root, &self.env, event);
        self.root.reclaim();
    }

    fn revision_commit(&mut self) -> (forge::DotTemperRevision, Option<ProjectCharters>) {
        let mut out = Queue::uncapped();
        for _ in 0..128_u32 {
            crate::release(&mut self.root, &self.env, &mut out);
            crate::release(&mut self.root, &self.env, &mut out);
            match out.pop() {
                Some(Request::Forge { call, repository, op: Op::Read(read) }) => {
                    assert_eq!(repository, HOME);
                    let result = self.answer(read);
                    self.event(Event::Forge(Box::new(forge::Event::Client(skein_forge_client::Event::Answered {
                        call,
                        result,
                        cost: 1,
                        free: 0,
                        rate: None,
                    }))));
                }
                Some(Request::ToChild(released)) => self.event(Event::Released(released)),
                Some(Request::Store(crate::StoreRequest::Commit { number, writes })) => {
                    self.store.apply(&writes);
                    self.event(Event::Store(crate::StoreInput::Committed { number, fill: self.store.fill }));
                    let mut row = None;
                    let mut set = None;
                    for write in writes {
                        if let Write::Save(Record::Forge { row: saved }) = write {
                            if let forge::Stored::DotTemperRevision(revision) = *saved {
                                assert!(row.replace(revision).is_none());
                            }
                        } else if let Write::Save(Record::Core(core::Record::Core(core::CoreRecord::Settings(
                            core::settings::Record::ProjectCharters(saved),
                        )))) = write
                        {
                            assert!(set.replace(saved).is_none(), "only one set per saved revision");
                        }
                    }
                    if let Some(row) = row {
                        return (row, set);
                    }
                    assert_eq!(set, None, "a live set must share the forge revision's commit");
                }
                Some(Request::Store(crate::StoreRequest::Load { owner, range, after, most, .. })) => {
                    let (rows, next) = self.store.load(&range, after.as_ref(), most);
                    self.event(Event::Store(crate::StoreInput::Loaded { owner, rows, next }));
                }
                Some(Request::Store(crate::StoreRequest::Engine(engine_store::Out::Load {
                    load,
                    range: engine_store::Range::Read(range),
                }))) => {
                    let core::reads::Scope::Retention { .. } = range.scope else {
                        panic!("only empty retention reads run beside the fake home read");
                    };
                    self.event(Event::Store(crate::StoreInput::Engine(engine_store::In::Loaded {
                        load,
                        result: engine_store::Answer::Rows(engine_store::Rows::Read { rows: Box::new([]), next: None }),
                    })));
                }
                None => self.event(Event::Resume),
                Some(other) => panic!("unexpected output while reading the home: {other:?}"),
            }
        }
        panic!(
            "home revision did not commit: tip={}, retained={:?}, next={:?}, now={:?}",
            self.tip(),
            self.root.forge.temper_revision(PROJECT),
            self.root.next_deadline(&self.env),
            self.env.now
        );
    }

    #[expect(clippy::disallowed_methods, reason = "translate the fake forge's ordinary Rust reads")]
    #[expect(clippy::wildcard_enum_match_arm, reason = "only the home's reads belong on this fake route")]
    fn answer(&mut self, read: Read) -> Result<Answer, Error> {
        let op = match &read {
            Read::Branch { branch } => raw::Read::Branch { branch: branch.clone() },
            Read::File { head, path, .. } => {
                raw::Read::File { commit: fake::commit_count(head).unwrap(), path: path.clone() }
            }
            Read::Compare { before, after } => raw::Read::Compare {
                base: fake::commit_count(before).unwrap(),
                head: fake::commit_count(after).unwrap(),
                page: 1,
                limit: 100,
            },
            Read::Items { .. } => {
                return Ok(Answer::Items { cut: false, items: Box::new([]), more: false, now: self.env.now });
            }
            other => panic!("unexpected home read: {other:?}"),
        };
        let mut out = Queue::uncapped();
        fake::step(
            &mut self.forge,
            &self.fake_env,
            fake::Event::Call {
                reply_to: ReplyTo::new(Token::new(1)),
                user: 7,
                repository: REPOSITORY.into(),
                op: raw::Op::Read(op),
            },
            &mut out,
        );
        fake::fire(&mut self.forge, &self.fake_env, &mut out);
        self.forge.reclaim();
        let Some(fake::Request::Reply { result, .. }) = out.pop() else { panic!("fake forge did not answer") };
        match (read, result) {
            (_, Ok(raw::Answer::Commit(tip))) => Ok(Answer::Commit(fake::commit_bytes(tip))),
            (_, Ok(raw::Answer::Comparison { base, head, ahead, behind, files, .. })) => Ok(Answer::Compare {
                before: fake::commit_bytes(base),
                after: fake::commit_bytes(head),
                ahead,
                behind,
                changed: u64::try_from(files.len()).unwrap(),
                files: files.into_vec().into_iter().map(|f| f.path).collect(),
            }),
            (Read::File { head, path, .. }, Ok(raw::Answer::Directory(entries))) => {
                let entries = entries
                    .into_vec()
                    .into_iter()
                    .map(|entry| api::Entry {
                        name: entry.name,
                        kind: match entry.kind {
                            raw::EntryKind::File => api::EntryKind::File,
                            raw::EntryKind::Directory => api::EntryKind::Directory,
                            raw::EntryKind::Link => api::EntryKind::Link,
                        },
                    })
                    .collect();
                Ok(Answer::Directory { head, path, entries, truncated: false })
            }
            (Read::File { head, path, max_bytes }, Ok(raw::Answer::File(bytes))) => {
                assert!(bytes.len() <= usize::try_from(max_bytes).unwrap());
                Ok(Answer::File { head, path, bytes, truncated: false })
            }
            (_, Err(raw::Error::Missing(_))) => Err(Error::Missing),
            (read, result) => panic!("{read:?}: {result:?}"),
        }
    }
}
