//! Unit configuration uses the existing engine world's tiny limits and charter,
//! with no templates. The empty startup retention pass is settled before runtime measurements;
//! no worker or connector driver is added.
use crate as engine;
use jig_domain_fleet as fleet;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_lib::{Duration, Env, Time, Wall};

use crate::{JournalLimits, loads};
use jig_domain_accounts as accounts;

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

/// Real authority policy and an authenticated owner, rather than allow flags.
#[must_use]
pub fn config(seed: u64) -> engine::Config {
    config_with_project_room(seed, 1)
}

pub fn config_with_project_room(seed: u64, projects: u32) -> engine::Config {
    engine::Config {
        host_part: crate::room::HostPart { allowance: u64::MAX, ..crate::room::HostPart::EMPTY },
        kinds_stock: crate::landing_stock(&[], &skein_lib::Map::with_capacity(0), 1, 4),
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

pub use jig_test_profile::start_values::{authority_limits, charter, models};

// Phases and durable continuations for one connector, followed by Idle.
const RESTORE_PASSES: u32 = 20;

pub fn root() -> (engine::Domain, Env<engine::Limits>) {
    let (mut root, env) = root_with_config(config(7));
    // Settle the startup placement probe before tests measure empty runtime resumes.
    let mut out = skein_lib::Queue::uncapped();
    fleet::resume(&mut root.core.fleet, &env.with(env.limits.fleet), true, &mut out);
    assert!(out.is_empty(), "the fixture restores no claims or workers");
    settle_retention(&mut root, &env);
    (root, env)
}

fn settle_retention(root: &mut engine::Domain, env: &Env<engine::Limits>) {
    use jig_domain_core as core;
    let env = crate::limits::environment_core(env);
    let mut requests = core::fire(&mut root.core, &env, core::Timer::Retention);
    for _ in 0..RESTORE_PASSES {
        let Some(request) = requests.pop() else { break };
        let core::Request::Held(held) = request else { panic!("an empty retention pass writes nothing") };
        let core::Held::ReadLoad { load, range } = *held else { panic!("retention loads its due range") };
        let core::reads::Scope::Retention { .. } = range.scope else { panic!("the fixture has no retention owner") };
        assert!(requests.is_empty(), "one internal read at a time");
        requests = core::step(
            &mut root.core,
            &env,
            core::Event::ReadLoaded { load, result: core::reads::Loaded::Page { rows: Box::new([]), next: None } },
        );
        root.core.reclaim();
    }
    assert!(requests.is_empty(), "empty-store retention completed without a write");
}
pub fn root_with_config(config: engine::Config) -> (engine::Domain, Env<engine::Limits>) {
    root_with_config_and_limits(config, &limits())
}

pub fn root_with_config_and_limits(
    config: engine::Config,
    limits: &engine::Limits,
) -> (engine::Domain, Env<engine::Limits>) {
    root_with_records(config, limits, &[])
}

pub fn root_with_records(
    config: engine::Config,
    limits: &engine::Limits,
    records: &[engine::Record],
) -> (engine::Domain, Env<engine::Limits>) {
    // Model a later start: the store supplies the complete project rows and
    // their deployment high-water mark, rather than configuration reseeding.
    let mut projects = skein_lib::List::with_capacity(limits.authority.projects);
    let mut project_mark = 0;
    for project in &config.core.seed_projects {
        project_mark = project_mark.max(u64::from(project.number));
        let value = jig_domain_core::policy_snapshot(
            &project.policy.policy,
            &project.policy.permissions,
            &project.policy.templates,
        )
        .expect("fixture policy snapshot");
        projects
            .push(people::Stored::Policy { project: project.number, name: project.name.clone(), value })
            .expect("bounded fixture projects");
    }
    let mut root = engine::Domain::new(config, limits).unwrap();
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: *limits };
    root.start_wall = Some(env.wall);
    let mut request = root.core.start_begin(false, env.wall);
    let mut deployment = root.core.counters.deployment();
    deployment.projects = project_mark;
    for record in records {
        match record {
            engine::Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row))) => {
                deployment.tasks = deployment.tasks.max(row.number);
            }
            engine::Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::Projection(
                jig_domain_core::ProjectionRecord::GoalHot { goal, .. },
            ))) => deployment.tasks = deployment.tasks.max(goal.number),
            engine::Record::Core(_) | engine::Record::Forge { .. } => {}
        }
    }
    engine::restart::restore_page_row(
        &mut root,
        &env,
        engine::Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::Deployment(deployment))),
    );
    for record in projects.into_boxed() {
        engine::restart::restore_page_row(
            &mut root,
            &env,
            engine::Record::Core(jig_domain_core::Record::People(record)),
        );
    }
    let mut decision = engine::Decision::decide(&mut root.journal, &limits.journal);
    for record in records {
        if let engine::Record::Forge { .. } = record {
            root.forge.restore_begin(env.wall);
        }
        engine::restart::restore_page_row(&mut root, &env, record.clone());
        engine::route::route_into(&mut root, &env, &mut decision);
    }
    // In production the page's own write-free decision takes the header's
    // commit count while the journal is still idle.
    if let Some(commits) = root.restored_commits.take() {
        root.journal = engine::Journal::uncapped(1, commits);
        root.sent_commit = commits;
        root.durable_commit = commits;
    }
    for _ in 0..RESTORE_PASSES {
        let mut output = skein_lib::Queue::uncapped();
        request = match request {
            jig_domain_core::start::Action::Step(phase) => match phase {
                jig_domain_core::start::Phase::Sweep => root.core.start_sweep(
                    &crate::limits::environment_core(&env),
                    jig_domain_core::start::SweepInput::Finished,
                    &mut output,
                ),
                phase @ (jig_domain_core::start::Phase::LoadCore
                | jig_domain_core::start::Phase::LoadInbox
                | jig_domain_core::start::Phase::RestoreConnector { .. }
                | jig_domain_core::start::Phase::Link
                | jig_domain_core::start::Phase::Switch
                | jig_domain_core::start::Phase::Wires
                | jig_domain_core::start::Phase::AdoptRuns
                | jig_domain_core::start::Phase::Open) => {
                    root.core.start_done(&crate::limits::environment_core(&env), phase, &mut output)
                }
            },
            jig_domain_core::start::Action::Commit(continuation) => {
                root.core.start_resume(&crate::limits::environment_core(&env), continuation, &mut output)
            }
            jig_domain_core::start::Action::Idle => {
                root.opened = true;
                break;
            }
            jig_domain_core::start::Action::Refused(failure) => panic!("fixture start: {failure:?}"),
        };
    }
    assert!(root.ready(), "fixture store restored");
    (root, env)
}

/// A typed durable store for the root's startup protocol; every release is an input.
pub struct StartStore {
    pub rows: alloc::collections::BTreeMap<engine::Key, engine::Record>,
    pub wires: u32,
    pub switches: u32,
}

impl StartStore {
    pub fn new() -> Self {
        Self { rows: alloc::collections::BTreeMap::new(), wires: 0, switches: 0 }
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
                    let key = engine::Key::Core(jig_domain_core::Key::Core(jig_domain_core::CoreKey::Settings(
                        jig_domain_core::settings::Key::Held(*number),
                    )));
                    let engine::Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::Settings(
                        jig_domain_core::settings::Record::Revision { revision, .. },
                    ))) = self.rows.get(&key).expect("held revision before its copy")
                    else {
                        panic!("held revision shape");
                    };
                    let row =
                        engine::Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::Settings(
                            jig_domain_core::settings::Record::Revision { revision: revision.clone(), history: true },
                        )));
                    self.rows.insert(row.key(), row);
                }
            }
        }
    }

    fn load(
        &self,
        range: &engine::Range,
        after: Option<&engine::Key>,
        most: u32,
    ) -> (Box<[engine::Record]>, Option<engine::Key>) {
        let mut ordered = alloc::collections::BTreeMap::new();
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
        let mut out = skein_lib::Queue::uncapped();
        for _ in 0_u32..1024 {
            engine::release(root, env, &mut out);
            engine::release(root, env, &mut out);
            if let Some(request) = out.pop() {
                match request {
                    engine::Request::ToChild(released) => engine::step(root, env, engine::Event::Released(released)),
                    engine::Request::Store(engine::StoreRequest::Catalogue { .. })
                    | engine::Request::ForgeKept(_)
                    | engine::Request::Party(
                        engine::PartyRequest::Catalogue { .. }
                        | engine::PartyRequest::Deliver(engine::Delivery::Providers(_)),
                    )
                    | engine::Request::Worker(engine::WorkerRequest::Settings(_)) => {}
                    engine::Request::Account(accounts::Request::Signing(accounts::Signing::Restored)) => {
                        assert!(self.wires > 0, "account restoration follows durable wires");
                    }
                    engine::Request::Store(engine::StoreRequest::Commit { number, writes }) => {
                        let wires = writes.contains(&engine::Write::Wires);
                        self.apply(&writes);
                        engine::step(
                            root,
                            env,
                            engine::Event::Store(engine::StoreInput::Committed { number, fill: 0 }),
                        );
                        if wires && pause_at_wires {
                            return;
                        }
                    }
                    engine::Request::Store(engine::StoreRequest::Switch) => {
                        self.switches = self.switches.checked_add(1).unwrap();
                        engine::step(root, env, engine::Event::Store(engine::StoreInput::Switched));
                    }
                    engine::Request::Store(engine::StoreRequest::Load { owner, range, after, most, .. }) => {
                        let (rows, next) = self.load(&range, after.as_ref(), most);
                        engine::step(
                            root,
                            env,
                            engine::Event::Store(engine::StoreInput::Loaded { owner, rows, next, texts: None }),
                        );
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
            } else if root.ready() {
                assert_eq!(root.start_phase(), None);
                return;
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
