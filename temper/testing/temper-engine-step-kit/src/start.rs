//! Small root configuration and a typed driver for the startup ports.
use skein_lib::Duration;
use temper_engine_domain_limits::{self as engine, JournalLimits};
use temper_engine_domain_store::loads;

// A commit's unique writes in this fixture, including its deployment header.
const COMMIT_WRITES: u32 = 2118;

// Twelve fleet program sources can emit 25 requests; one continuation needs the next slot.
const DELIVERIES: u32 = 26;

#[must_use]
pub fn limits() -> engine::Limits {
    let tasks = temper_limits_fixtures::root_resume::tasks();
    let people = temper_limits_fixtures::root_resume::people();
    let fleet = temper_limits_fixtures::root_resume::fleet();
    let journal = JournalLimits {
        commits: 3,
        held: DELIVERIES
            .checked_mul(3)
            .expect("fixture journal capacity")
            .checked_add(16)
            .expect("fixture journal capacity"),
        writes: COMMIT_WRITES,
        deliveries: DELIVERIES,
        transcript_bytes: 8192,
        result_bytes: 384,
        run_bytes: 4096,
    };
    engine::Limits {
        streams: 1,
        routes: jig_protocol_config_fixtures::small().routes,
        // The same configured core byte budgets as the engine world fixture.
        policy_bytes: jig_protocol_config_fixtures::small().core.policy_bytes,
        call_answer_bytes: jig_protocol_config_fixtures::small().core.call_answer_bytes,
        host_reply_bytes: None,
        rerun_processes: 4,
        settings: temper_limits_fixtures::root_resume::settings(),
        templates: 2,
        documents: temper_limits_fixtures::root_resume::documents(&authority_limits(), &tasks),
        authority: authority_limits(),
        transcript_memory: u64::MAX,
        watermark: engine::default_watermark(&journal),
        public_watermark: engine::default_watermark(&journal).checked_div(2).expect("half the watermark"),
        public_reads: 0,
        public: jig_domain_core::PublicIndexLimits::OFF,
        retention: jig_domain_core::RetentionLimits {
            turns: Duration::from_secs(14 * 86_400),
            records: Duration::from_secs(30 * 86_400),
            summaries: Duration::from_secs(365 * 86_400),
        },
        waits: jig_protocol_config_fixtures::small().core.waits,
        journal,
        loads: loads::Limits {
            read_bytes: 65_536,
            loads: 2,
            rows: 2,
            bytes: 16384,
            reply_bytes: 16384,
            transcript_bytes: 8192,
        },
        tasks,
        people,
        fleet,
        call_records: 2,
        brief: engine::BriefLimits {
            briefs: 2,
            sections: 3,
            parts: 2,
            read_bytes: 256,
            budgets: engine::BriefBudgets {
                document: 4096,
                task: 128,
                dependencies: 128,
                ci: 128,
                reviews: 128,
                pull: 128,
                attempts: 128,
                plan: 128,
                notes: 64,
                preferences: 64,
                now: 64,
            },
            brief_bytes: 384,
            gather: Duration::from_secs(1),
        },
        accounts: temper_limits_fixtures::root_resume::accounts(),
        notes: temper_limits_fixtures::root_resume::notes(),
        forge: temper_engine_domain_forge::Limits {
            waits: jig_protocol_config_fixtures::small().core.waits,
            ..temper_limits_fixtures::root_resume::FORGE_LIMITS
        },
    }
}

pub use jig_test_profile::start_values::{authority_limits, charter, models};

/// Bind the shared startup fixture to the root crate of the consuming test binary.
#[macro_export]
macro_rules! start_fixture {
    ($root:path) => {
        pub mod step_start_fixture {
            use ::skein_lib::{Duration, Env};
            pub use $crate::start::limits;
            use $root as engine;
            /// Real authority policy and an authenticated owner, rather than allow flags.
            #[must_use]
            pub fn config(seed: u64) -> engine::Config {
                config_with_project_room(seed, 1)
            }

            pub fn config_with_project_room(seed: u64, projects: u32) -> engine::Config {
                engine::Config {
                    host_part: engine::room::HostPart { allowance: u64::MAX, ..engine::room::HostPart::EMPTY },
                    kinds_stock: engine::landing_stock(&[], &skein_lib::Map::with_capacity(0), 1, 4),
                    ways_stock: Box::default(),
                    period_span: Duration::from_secs(7 * 24 * 60 * 60),
                    core: jig_test_profile::start_values::core_config(seed, projects),
                    landing_held: false,
                    forge: temper_engine_domain_forge::Config {
                        push: None,
                        namespace: Box::from(&b"root"[..]),
                        writers: Box::new([skein_forge_client::Writer { forge: 1, author: 1 }]),
                    },
                    // The forge's git identity is application credential 1, which the
                    // worlds name at start (D10).
                    identities: Box::new([temper_engine_domain_forge::GitIdentity { forge: 1, credential: 1 }]),
                    machine: None,
                    forges: temper_limits_fixtures::start_forge::configured_forges(),
                    under_box: false,
                    playground: None,
                }
            }

            /// A typed durable store for the root's startup protocol; every release is an input.
            pub struct StartStore {
                pub rows: std::collections::BTreeMap<engine::Key, engine::Record>,
                pub wires: u32,
                pub switches: u32,
                pub fill: u64,
            }

            pub enum Until {
                Open,
                WiresDurable,
                Phase(jig_domain_core::start::Phase),
                Load(engine::Range),
                ForgeRead,
            }

            impl StartStore {
                pub fn new() -> Self {
                    Self { rows: std::collections::BTreeMap::new(), wires: 0, switches: 0, fill: 0 }
                }

                pub fn keep(&mut self, row: engine::Record) {
                    self.rows.insert(row.key(), row);
                }

                pub fn apply(&mut self, writes: &[engine::Write]) {
                    for write in writes {
                        match write {
                            engine::Write::Opening(_)
                            | engine::Write::DropHistory { .. }
                            | engine::Write::DropTranscript { .. } => {}
                            engine::Write::Wires => self.wires = self.wires.checked_add(1).unwrap(),
                            engine::Write::Save(row) => {
                                self.rows.insert(row.key(), row.clone());
                            }
                            engine::Write::Erase(key) => {
                                self.rows.remove(key);
                            }
                            engine::Write::CopySettingsRevision(number) => {
                                let key = engine::Key::Core(jig_domain_core::Key::Core(
                                    jig_domain_core::CoreKey::Settings(jig_domain_core::settings::Key::Held(*number)),
                                ));
                                let engine::Record::Core(jig_domain_core::Record::Core(
                                    jig_domain_core::CoreRecord::Settings(
                                        jig_domain_core::settings::Record::Revision { revision, .. },
                                    ),
                                )) = self.rows.get(&key).expect("held revision before its copy")
                                else {
                                    panic!("held revision shape");
                                };
                                let row = engine::Record::Core(jig_domain_core::Record::Core(
                                    jig_domain_core::CoreRecord::Settings(
                                        jig_domain_core::settings::Record::Revision {
                                            revision: revision.clone(),
                                            history: true,
                                        },
                                    ),
                                ));
                                self.rows.insert(row.key(), row);
                            }
                        }
                    }
                }

                pub fn load(
                    &self,
                    range: &engine::Range,
                    after: Option<&engine::Key>,
                    most: u32,
                ) -> (Box<[engine::Record]>, Option<engine::Key>) {
                    let mut ordered = std::collections::BTreeMap::new();
                    for (key, row) in &self.rows {
                        if range.contains(key) {
                            ordered.insert(start_position(range, key), row.clone());
                        }
                    }
                    let mut rows = skein_lib::List::with_capacity(most);
                    let mut more = false;
                    for (position, row) in ordered {
                        if let Some(key) = after
                            && position <= start_position(range, key)
                        {
                            continue;
                        }
                        if rows.room() == 0 {
                            more = true;
                            break;
                        }
                        rows.push(row).unwrap();
                    }
                    let next = if more { Some(rows.as_slice().last().unwrap().key()) } else { None };
                    (rows.into_boxed(), next)
                }

                pub fn run(&mut self, root: &mut engine::Domain, env: &Env<engine::Limits>, pause_at_wires: bool) {
                    let until = if pause_at_wires { Until::WiresDurable } else { Until::Open };
                    assert!(self.advance(root, env, until).is_none());
                    assert_eq!(root.start_stop(), None, "start did not refuse");
                }

                #[expect(clippy::too_many_lines, reason = "one store driver handles the typed startup outputs")]
                pub fn advance(
                    &mut self,
                    root: &mut engine::Domain,
                    env: &Env<engine::Limits>,
                    until: Until,
                ) -> Option<skein_lib::Token> {
                    let mut out = skein_lib::Queue::uncapped();
                    for _ in 0_u32..1024 {
                        if root.start_stop().is_some() {
                            return None;
                        }
                        if let Until::Phase(target) = &until
                            && root.start_phase() == Some(*target)
                        {
                            assert!(out.is_empty(), "no released work kept by the fixture at a phase boundary");
                            root.reclaim();
                            return None;
                        }
                        engine::release(root, env, &mut out);
                        engine::release(root, env, &mut out);
                        if let Some(request) = out.pop() {
                            match request {
                                engine::Request::ToChild(released) => {
                                    engine::step(root, env, engine::Event::Released(released))
                                }
                                engine::Request::Store(engine::StoreRequest::Catalogue { .. })
                                | engine::Request::ForgeKept(_)
                                | engine::Request::Party(
                                    engine::PartyRequest::Catalogue { .. }
                                    | engine::PartyRequest::Deliver(engine::Delivery::Providers(_)),
                                )
                                | engine::Request::Worker(engine::WorkerRequest::Settings(_)) => {}
                                engine::Request::Account(::jig_domain_accounts::Request::Signing(
                                    ::jig_domain_accounts::Signing::Restored,
                                )) => {
                                    assert!(self.wires > 0, "account restoration follows durable wires");
                                }
                                engine::Request::Store(engine::StoreRequest::Commit { number, writes }) => {
                                    let wires = writes.contains(&engine::Write::Wires);
                                    self.apply(&writes);
                                    engine::step(
                                        root,
                                        env,
                                        engine::Event::Store(engine::StoreInput::Committed { number, fill: self.fill }),
                                    );
                                    if wires && let Until::WiresDurable = until {
                                        root.reclaim();
                                        return None;
                                    }
                                }
                                engine::Request::Store(engine::StoreRequest::Switch) => {
                                    self.switches = self.switches.checked_add(1).unwrap();
                                    engine::step(root, env, engine::Event::Store(engine::StoreInput::Switched));
                                }
                                engine::Request::Store(engine::StoreRequest::Load {
                                    owner,
                                    range,
                                    after,
                                    most,
                                    ..
                                }) => {
                                    if let Until::Load(wanted) = &until
                                        && *wanted == range
                                    {
                                        assert!(out.is_empty(), "only one startup load in flight");
                                        root.reclaim();
                                        return Some(owner);
                                    }
                                    let (rows, next) = self.load(&range, after.as_ref(), most);
                                    engine::step(
                                        root,
                                        env,
                                        engine::Event::Store(engine::StoreInput::Loaded { owner, rows, next }),
                                    );
                                }
                                engine::Request::Forge { call, op: skein_forge::api::Op::Read(_), .. }
                                    if let Until::ForgeRead = until =>
                                {
                                    root.reclaim();
                                    return Some(call);
                                }
                                engine::Request::Stop => panic!("start stopped: {:?}", root.start_stop()),
                                request @ (engine::Request::Unsettled(_)
                                | engine::Request::Bound { .. }
                                | engine::Request::BoxRemove { .. }
                                | engine::Request::ChoosePlace { .. }
                                | engine::Request::BoxGroup { .. }
                                | engine::Request::ForgeGone { .. }
                                | engine::Request::PlaygroundActs { .. }
                                | engine::Request::ResetAct { .. }
                                | engine::Request::MakingCopied { .. }
                                | engine::Request::SetUp { .. }
                                | engine::Request::Connect { .. }
                                | engine::Request::Replace { .. }
                                | engine::Request::Manifest { .. }
                                | engine::Request::AdoptAccess { .. }
                                | engine::Request::Reachable { .. }
                                | engine::Request::Accept { .. }
                                | engine::Request::Install { .. }
                                | engine::Request::EraseCarried { .. }
                                | engine::Request::Setup { .. }
                                | engine::Request::Store(_)
                                | engine::Request::Worker(_)
                                | engine::Request::Party(_)
                                | engine::Request::Forge { .. }
                                | engine::Request::Account(_)
                                | engine::Request::Busy(_)
                                | engine::Request::Restarting { .. }
                                | engine::Request::LimitSettingsCommitted { .. }
                                | engine::Request::Drained) => panic!("unexpected startup output: {request:?}"),
                            }
                        } else if root.ready()
                            && let Until::Open | Until::WiresDurable | Until::Phase(_) | Until::Load(_) = until
                        {
                            assert_eq!(root.start_phase(), None);
                            return None;
                        } else {
                            engine::step(root, env, engine::Event::Resume);
                        }
                        root.reclaim();
                    }
                    panic!("start did not open: {:?}", root.start_phase());
                }
            }

            fn start_position(range: &engine::Range, key: &engine::Key) -> (u32, engine::Key) {
                match (range, key) {
                    (engine::Range::Core(range), engine::Key::Core(key)) => {
                        let (interval, key) = jig_domain_core::restore_cursor(range.restore_ranges(), key).unwrap();
                        (interval, engine::Key::Core(key))
                    }
                    (engine::Range::Census(census), engine::Key::Core(key)) => {
                        (census.interval(key).unwrap(), engine::Key::Core(key.clone()))
                    }
                    (engine::Range::Forge, key) => (0, key.clone()),
                    _ => panic!("startup key belongs to its range"),
                }
            }
        }
    };
}
