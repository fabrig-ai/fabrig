//! Project checkout, charter and durable row fixtures.
use jig_domain_core::{self as core, CharterPart, ProjectCharter, ProjectCharters};
use jig_domain_tasks::Reasons;
use skein_forge::api::{self, Commit};
use temper_engine_domain_forge as forge;
use temper_engine_domain_store::Record;

pub const PROJECT: u32 = 1;
pub const HOME: api::Repository = api::Repository { forge: 1, repository: 1 };
pub const REPOSITORY: &[u8] = b"acme/shop";
pub const MAIN: &[u8] = b"main";
pub const WAYS: &[u8] = b".temper/ways.toml";
pub const IMPLEMENTER: &[u8] = b".temper/charters/implementer.md";
pub const CAREFUL: &[u8] = b".temper/charters/careful.md";
pub const OFFERS: &[u8] = b"offer = [\"build\"]\n";
pub const WAY: &[u8] = b".temper/ways/build.toml";
pub const BUILD: &[u8] = b"format = 1\n";
pub const WORDS: &[u8] = b"Be careful.";
pub const PART: &[u8] = b"Name the risk before proceeding.";
pub const CAREFUL_WORDS: &[u8] = b"+++\nsettings = \"implementer\"\n+++\nRead the brief.\n\n## part, on: first, question\nName the risk before proceeding.\n";
pub const BAD_WORDS: &[u8] = b"## part, on: frist\nThis part does not read.";
const EXAMPLE: Commit = [
    0x1a, 0x2b, 0x3c, 0x4d, 0x5e, 0x6f, 0x7a, 0x8b, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0,
];

#[must_use]
pub fn implementer() -> ProjectCharter {
    ProjectCharter {
        name: b"implementer".as_slice().into(),
        settings: b"implementer".as_slice().into(),
        instructions: WORDS.into(),
        parts: Box::new([]),
    }
}

#[must_use]
pub fn careful() -> ProjectCharter {
    ProjectCharter {
        name: b"careful".as_slice().into(),
        settings: b"implementer".as_slice().into(),
        instructions: b"Read the brief.".as_slice().into(),
        parts: Box::new([CharterPart { on: Reasons::FIRST.union(Reasons::QUESTION), instructions: PART.into() }]),
    }
}

#[must_use]
pub fn saved_set(set: ProjectCharters) -> Record {
    Record::Core(core::Record::Core(core::CoreRecord::Settings(core::settings::Record::ProjectCharters(set))))
}

#[expect(clippy::disallowed_methods, reason = "build the store fixture from its listed files")]
#[must_use]
pub fn revision(files: &[(&[u8], &[u8])]) -> Record {
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

#[must_use]
pub fn owned_repository(forge: u16) -> temper_engine_domain_forge::Repository {
    use temper_engine_domain_forge as domain;
    domain::Repository {
        contexts: Box::new([]),
        landing: b"main".as_slice().into(),
        refusals: Box::default(),
        workflows: Box::default(),
        project: 5,
        home: forge == 1,
        provider: skein_forge::api::Repository { forge, repository: 1 },
        host: Box::from(&b"forge.example"[..]),
        owner: Box::from(&b"org"[..]),
        name: Box::from(&b"repo"[..]),
        prefix: Box::from(&b"temper/"[..]),
        role: domain::Role::Owned,
        ci: true,
        checks: Box::new([]),
        kinds: domain::Kinds {
            read: true,
            push: true,
            open: true,
            land: true,
            review: true,
            status: true,
            comment: true,
            issue: true,
            branch: true,
        },
        protection: domain::Protection::Absent,
        settings: skein_forge::api::Settings {
            default_branch: Box::from(&b"main"[..]),
            merge: true,
            squash: true,
            rebase: false,
        },
    }
}

/// Bind the checkout and start drivers to the consuming binary's engine root.
#[macro_export]
macro_rules! project_charter_fixture {
    ($root:path, $start:path) => {
        pub mod step_project_fixture {
            use ::jig_domain_core::engine::store as engine_store;
            use ::jig_domain_core::{self as core, ProjectCharters};
            use ::skein_fake_forge_domain::{self as fake, api as raw};
            use ::skein_forge::api::{self, Answer, Error, Op, Read};
            use ::skein_lib::{Duration, Env, Queue, ReplyTo, Time, Token, Wall};
            use ::temper_engine_domain_forge as forge;
            use engine::{Domain, Event, Limits, Record, Request, Write};
            use $crate::project_charters::*;
            use $root as engine;
            use $start::{StartStore, config, limits};

            pub fn old_store(limits: &Limits, home: bool) -> StartStore {
                let mut store = StartStore::new();
                start(limits, &mut store, false);
                let mut repository = owned_repository(HOME.forge);
                repository.project = PROJECT;
                repository.owner = b"acme".as_slice().into();
                repository.name = b"shop".as_slice().into();
                repository.home = home;
                store.keep(Record::Forge { row: Box::new(forge::Stored::Repository(repository)) });
                store
            }

            pub fn start(limits: &Limits, store: &mut StartStore, pending: bool) -> (Domain, Env<Limits>) {
                let mut config = config(7);
                config.core.defaults.charters[0].name = Some(b"implementer".as_slice().into());
                let mut root = Domain::new(config, limits).unwrap();
                let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: *limits };
                engine::step(&mut root, &env, Event::Restart { pending });
                store.run(&mut root, &env, false);
                (root, env)
            }

            pub struct World {
                pub root: Domain,
                pub env: Env<Limits>,
                pub store: StartStore,
                pub forge: fake::Domain,
                pub fake_env: Env<fake::Config>,
            }

            impl World {
                #[expect(clippy::disallowed_methods, reason = "build the fake forge tree from its listed files")]
                pub fn new(files: &[(&[u8], &[u8])]) -> Self {
                    let mut limits = limits();
                    limits.tasks.charters = 2;
                    let mut store = old_store(&limits, true);
                    let (root, env) = start(&limits, &mut store, true);
                    assert_eq!(root.project_home(PROJECT).map(|repo| repo.provider), Some(HOME));
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

                pub fn tip(&self) -> u64 {
                    self.forge.branch(REPOSITORY, MAIN).unwrap()
                }

                pub fn advance(&mut self, path: &[u8], text: &[u8]) {
                    fake::advance(&mut self.forge, &self.fake_env, REPOSITORY, MAIN, path, text, 7).unwrap();
                }

                pub fn notice(&mut self) {
                    self.event(Event::Forge(Box::new(forge::Event::Hint {
                        hint: api::Hint { repository: HOME, change: api::Change::Branch(MAIN.into()), key: None },
                    })));
                    self.env.now = self.env.now.saturating_add(self.env.limits.forge.client.hinted);
                    self.fake_env.now = self.env.now;
                    engine::fire(&mut self.root, &self.env);
                }

                fn event(&mut self, event: Event) {
                    engine::step(&mut self.root, &self.env, event);
                    self.root.reclaim();
                }

                pub fn revision_commit(&mut self) -> (forge::DotTemperRevision, Option<ProjectCharters>) {
                    let mut out = Queue::uncapped();
                    for _ in 0..128_u32 {
                        engine::release(&mut self.root, &self.env, &mut out);
                        engine::release(&mut self.root, &self.env, &mut out);
                        match out.pop() {
                            Some(Request::Forge { call, repository, op: Op::Read(read) }) => {
                                assert_eq!(repository, HOME);
                                let result = self.answer(read);
                                self.event(Event::Forge(Box::new(forge::Event::Client(
                                    skein_forge_client::Event::Answered { call, result, cost: 1, free: 0, rate: None },
                                ))));
                            }
                            Some(Request::ToChild(released)) => self.event(Event::Released(released)),
                            Some(Request::Store(engine::StoreRequest::Commit { number, writes })) => {
                                self.store.apply(&writes);
                                self.event(Event::Store(engine::StoreInput::Committed {
                                    number,
                                    fill: self.store.fill,
                                }));
                                let mut row = None;
                                let mut set = None;
                                for write in writes {
                                    if let Write::Save(Record::Forge { row: saved }) = write {
                                        if let forge::Stored::DotTemperRevision(revision) = *saved {
                                            assert!(row.replace(revision).is_none());
                                        }
                                    } else if let Write::Save(Record::Core(core::Record::Core(
                                        core::CoreRecord::Settings(core::settings::Record::ProjectCharters(saved)),
                                    ))) = write
                                    {
                                        assert!(set.replace(saved).is_none(), "only one set per saved revision");
                                    }
                                }
                                if let Some(row) = row {
                                    return (row, set);
                                }
                                assert_eq!(set, None, "a live set must share the forge revision's commit");
                            }
                            Some(Request::Store(engine::StoreRequest::Load { owner, range, after, most, .. })) => {
                                let (rows, next) = self.store.load(&range, after.as_ref(), most);
                                self.event(Event::Store(engine::StoreInput::Loaded { owner, rows, next }));
                            }
                            Some(Request::Store(engine::StoreRequest::Engine(engine_store::Out::Load {
                                load,
                                range: engine_store::Range::Read(range),
                            }))) => {
                                let core::reads::Scope::Retention { .. } = range.scope else {
                                    panic!("only empty retention reads run beside the fake home read");
                                };
                                self.event(Event::Store(engine::StoreInput::Engine(engine_store::In::Loaded {
                                    load,
                                    result: engine_store::Answer::Rows(engine_store::Rows::Read {
                                        rows: Box::new([]),
                                        next: None,
                                    }),
                                })));
                            }
                            None => self.event(Event::Resume),
                            Some(other) => panic!("unexpected output while reading the home: {other:?}"),
                        }
                    }
                    panic!(
                        "home revision did not commit: tip={}, retained={:?}, next={:?}, now={:?}",
                        self.tip(),
                        self.root.project_temper_revision(PROJECT),
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
                            return Ok(Answer::Items {
                                cut: false,
                                items: Box::new([]),
                                more: false,
                                now: self.env.now,
                            });
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
                    let Some(fake::Request::Reply { result, .. }) = out.pop() else {
                        panic!("fake forge did not answer")
                    };
                    match (read, result) {
                        (_, Ok(raw::Answer::Commit(tip))) => Ok(Answer::Commit(fake::commit_bytes(tip))),
                        (_, Ok(raw::Answer::Comparison { base, head, ahead, behind, files, .. })) => {
                            Ok(Answer::Compare {
                                before: fake::commit_bytes(base),
                                after: fake::commit_bytes(head),
                                ahead,
                                behind,
                                changed: u64::try_from(files.len()).unwrap(),
                                files: files.into_vec().into_iter().map(|f| f.path).collect(),
                            })
                        }
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
        }
    };
}
