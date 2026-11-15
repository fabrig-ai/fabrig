//! Project release survives cold starts (temper/docs/design/domain/forge/adoption.md, 1;
//! domain/root.md, 8.1; jig/docs/design/domain/people/acts.md, 2).
use std::collections::BTreeMap;

use jig_domain_core as core;
use jig_domain_people as people;
use skein_forge::api::{self, Repository};
use skein_forge_client as client;
use skein_lib::{ReplyTo, Token, Wall};
use temper_engine_domain::{self as engine, Key, Record, Write};
use temper_engine_domain_forge as forge;
use temper_engine_domain_forge_change as change;
use temper_engine_domain_forge_tiers as tiers;
use temper_engine_domain_world::{direct::Driver, fixtures, forge_stand::ForgeStand, store::Store, walking};

const R: Repository = Repository { forge: 1, repository: 10 };
const Q: Repository = Repository { forge: 1, repository: 11 };
const CHANGE: u64 = 501;
const STEPS: usize = 400;
const SEED: u64 = 71;
const BULK: u64 = 600;

type Rows = BTreeMap<Key, Record>;

fn config() -> engine::Config {
    let mut config = walking::config(SEED);
    let first = &config.core.seed_projects[0];
    let second = core::SeedProject { number: 2, name: "Project 2".into(), policy: first.policy.clone() };
    let mut projects = config.core.seed_projects.into_vec();
    projects.push(second);
    config.core.seed_projects = projects.into_boxed_slice();
    let identity = config.core.owners[0].identity.clone();
    config.core.owners = Box::new([
        people::InitialOwner { project: 1, identity: identity.clone() },
        people::InitialOwner { project: 2, identity },
    ]);
    config
}

fn limits() -> engine::Limits {
    let mut limits = fixtures::forge_limits();
    limits.authority.projects = 2;
    limits.people.projects = 3;
    limits.people.initial_owners = 2;
    // The between-pages story restores 600 observations without changing release's 512-write page.
    limits.forge.client.resources = 610;
    limits.loads.rows = 32;
    limits
}

fn stand() -> ForgeStand {
    ForgeStand::with(&[(b"org", b"repo", R.repository), (b"org", b"other", Q.repository)])
}

fn cold(store: Store) -> Driver {
    let mut d = Driver::cold(store, config(), &limits());
    d.forge = Some(stand());
    d.connect_forges();
    d
}

fn ready(d: &Driver) {
    assert_eq!(d.root.start_stop(), None, "no ForgeLink refusal");
    assert!(d.root.ready() && !d.stopped, "cold start reaches decisions: {:?}", d.root);
}

fn adopt(d: &mut Driver, project: u32, name: &[u8], key: u8, repository: Repository) {
    let mut adoption = fixtures::adoption(1);
    adoption.project = project;
    adoption.name = name.into();
    d.send(engine::Event::Party(engine::PartyInput::Application {
        scope: people::ApplicationScope::Project(project),
        reply_to: ReplyTo::new(Token::new(u64::from(key))),
        sign_in: d.session(),
        key: [key; 16],
        digest: [key; 32],
        carried: None,
        request: engine::Application::Forge(forge::Ask::Adopt(adoption)),
    }));
    d.pump();
    assert!(d.delivered.iter().any(|delivery| matches!(delivery,
        engine::Delivery::Application { outcome: Some(engine::ApplicationAnswer::Adopt(
            forge::ApplicationOutcome::Adopted(row))), .. } if row.repository == repository)));
}

fn saved(row: forge::Stored) -> Record {
    Record::Forge { row: Box::new(row) }
}

fn pull(repository: Repository, number: u64) -> forge::Name {
    forge::Name { forge: repository.forge, repository: repository.repository, what: forge::What::Pull(number) }
}

fn tier(repository: Repository) -> forge::Stored {
    forge::Stored::Tier(forge::TierRow {
        repository,
        tier: b"focused".as_slice().into(),
        record: tiers::Record {
            sequence: 0,
            judged: None,
            failing: tiers::Failures::Tests(Box::new([])),
            flakes: Box::new([]),
            run: None,
        },
        task: None,
        observations: Box::new([]),
        requested: false,
        cancelled: false,
        purpose: None,
        pending: Box::new([]),
        baseline: None,
        branch: forge::TierBranch::Absent,
        probe: None,
        flight: None,
        held: None,
        hold: None,
        result: None,
        posting: None,
        received: false,
        goals: Box::new([]),
        goal: None,
        started: None,
    })
}

// An ended change's panel outlives its task; the release must erase both parts.
fn ended_change(repository: Repository) -> forge::Stored {
    forge::Stored::Change(Box::new(forge::ChangeRow {
        source: None,
        kept: None,
        task: CHANGE,
        repository,
        branch: b"temper/old".as_slice().into(),
        base: b"main".as_slice().into(),
        title: b"Kept change".as_slice().into(),
        body: Box::new([]),
        change: change::Change {
            task: CHANGE,
            way: change::Way::Queue,
            state: change::State::Ended,
            early: change::Early::None,
            landing: None,
            ended_held: None,
            gates: Box::new([]),
            clean: Box::new([]),
            gate_failures: Box::new([]),
            released: None,
            released_resolution: false,
            repairs: 0,
            resolutions: 0,
            updates: 0,
            since: Wall::EPOCH,
            ready_since: None,
            owns_turn: false,
            last_head: None,
        },
        task_release: None,
        job: None,
        signed: Box::new([]),
        goal: None,
        ending: Some(forge::Ending::NoChange),
        repair_tasks: Box::new([]),
        resolution_tasks: Box::new([]),
        result: forge::ChangeResult::empty(),
        priority: 0,
        ci: None,
        pull: None,
        pending: None,
        effect: change::EffectResult::Pending,
        delegate: None,
        delegate_status: change::Status::Passed,
        verdicts: Box::new([]),
        gate_remarks: Box::new([]),
        drift: None,
        base_repair: false,
        queue_repair: None,
        queue_repairs: 0,
        producing_charter: 1,
        review_charter: 1,
        producing_llm: None,
        review_llm: None,
        effect_account: None,
        producer_failure: None,
        report: Box::new([]),
        questions: change::Questions::default(),
        repairs: None,
        follows: None,
        follow_check: None,
        catch_up: None,
        close: None,
        merge_result: None,
        history: None,
    }))
}

fn observations(repository: Repository, project: u32) -> Vec<Record> {
    vec![
        saved(forge::Stored::Ci(forge::CiState { repository, head: [3; 32], status: api::Ci::Passed })),
        saved(tier(repository)),
        saved(forge::Stored::Verification(forge::Verification {
            repository,
            commit: [3; 32],
            changes: Box::new([]),
            tiers: Box::new([]),
        })),
        saved(forge::Stored::Receipt {
            key: forge::ReceiptKey {
                project,
                repository,
                check: forge::Check::Gate(b"test".as_slice().into()),
                key: [4; 32],
            },
            receipt: forge::Receipt {
                passed: true,
                logs: Box::new([]),
                failures: None,
                run: forge::ReceiptRun { id: 1, number: 1, url: Box::new([]) },
                candidate: [3; 32],
                ended: Wall::EPOCH,
            },
        }),
    ]
}

#[expect(clippy::default_trait_access, reason = "the issue projection type is not re-exported by the connector")]
fn fixture(bulk: u64) -> (Driver, Repository, Repository) {
    let mut d = cold(Store::with_limits(&limits()));
    d.pump();
    ready(&d);
    d.sign_in();
    d.pump();
    adopt(&mut d, 1, b"repo", 91, R);
    adopt(&mut d, 2, b"other", 92, Q);
    let mut rows = observations(R, 1);
    rows.extend(observations(Q, 2));
    // Pull 45 on repository 10 is the staging orphan; pull 1 below follows real reads.
    rows.push(saved(forge::Stored::PullState(forge::PullState {
        name: pull(R, 45),
        head: [3; 32],
        ci: api::Ci::Passed,
        state: api::State::Closed,
    })));
    for number in 1000..1000 + bulk {
        rows.push(saved(forge::Stored::PullState(forge::PullState {
            name: pull(R, number),
            head: [3; 32],
            ci: api::Ci::Passed,
            state: api::State::Closed,
        })));
    }
    rows.push(saved(ended_change(R)));
    let mut issue = forge::IssueRow {
        goal: 502,
        repository: R,
        state: Default::default(),
        number: Some(46),
        pending: None,
        before: None,
        failed: false,
        desired: None,
    };
    issue.state.closed = true;
    rows.push(saved(forge::Stored::Issue(issue)));
    let required: Vec<_> =
        rows.iter().map(Record::key).filter(|key| *key != Key::Forge(forge::Key::Change(CHANGE))).collect();
    d.store.seed(rows.into_boxed_slice(), Box::new([]));
    let mut d = cold(d.take_store().fork());
    d.pump();
    ready(&d); // All seeded rows have their parents, before any deletion can hide a broken link.
    let stand = d.forge.as_mut().expect("fake forge");
    stand.push_branch(R.repository, b"temper/observed");
    stand.call(
        Token::new(900),
        R,
        api::Op::Write(api::Write::OpenPull {
            title: b"Observed pull".as_slice().into(),
            body: Box::new([]),
            head: b"temper/observed".as_slice().into(),
            base: b"main".as_slice().into(),
        }),
        &d.env.limits.forge.client,
    );
    assert_eq!(stand.replies().len(), 1, "fake's external pull created");
    d.send(engine::Event::Forge(Box::new(forge::Event::Client(client::Event::Keep {
        owner: Token::new(901),
        watches: Box::new([
            client::Watch {
                resource: client::Resource {
                    repository: R,
                    what: client::What::Branch(b"temper/observed".as_slice().into()),
                },
                participating: false,
            },
            client::Watch {
                resource: client::Resource { repository: R, what: client::What::Pull(1) },
                participating: false,
            },
        ]),
    }))));
    d.pump();
    for key in required {
        assert!(d.store.rows.contains_key(&key), "seeded family still present before deletion: {key:?}");
    }
    assert!(d.store.rows.contains_key(&Key::Forge(forge::Key::ChangeCold(CHANGE))));
    assert!(d.store.rows.contains_key(&Key::Forge(forge::Key::ChangeHot(CHANGE))));
    assert!(d.store.rows.contains_key(&Key::Forge(forge::Key::PullState(pull(R, 1)))), "real pull read saved");
    assert!(d.store.rows.contains_key(&Key::Forge(forge::Key::PullState(pull(R, 45)))));
    assert!(d.store.rows.values().any(|row| matches!(row, Record::Forge { row }
        if matches!(row.as_ref(), forge::Stored::BranchHead(head) if name_is(&head.name, R)))));
    assert!(d.store.rows.values().any(|row| matches!(row, Record::Forge { row }
        if matches!(row.as_ref(), forge::Stored::Client(client::Stored::Live(live)) if live.watch.resource.repository == R))));
    (d, R, Q)
}

fn name_is(name: &forge::Name, repository: Repository) -> bool {
    name.forge == repository.forge && name.repository == repository.repository
}

fn topic_is(topic: &forge::Topic, repository: Repository) -> bool {
    match topic {
        forge::Topic::Landings { repository: r, .. }
        | forge::Topic::Ci { repository: r, .. }
        | forge::Topic::Pull { repository: r, .. }
        | forge::Topic::Participation { repository: r, .. } => *r == repository,
        forge::Topic::Findings { .. } => false,
    }
}

fn client_key_is(key: &client::Key, repository: Repository) -> bool {
    match key {
        client::Key::Live(resource) => resource.repository == repository,
        client::Key::Repository(r) => *r == repository,
        client::Key::Writer(_) => false,
    }
}

fn key_is(key: &forge::Key, repository: Repository) -> bool {
    match key {
        forge::Key::Repository(r)
        | forge::Key::Ci { repository: r, .. }
        | forge::Key::Tier { repository: r, .. }
        | forge::Key::Verification { repository: r, .. }
        | forge::Key::LandingLog { repository: r }
        | forge::Key::LandedDraft { repository: r, .. } => *r == repository,
        forge::Key::Hold(name) | forge::Key::BranchHead(name) | forge::Key::PullState(name) => {
            name_is(name, repository)
        }
        forge::Key::Subscription { topic, .. } => topic_is(topic, repository),
        forge::Key::Client(key) => client_key_is(key, repository),
        forge::Key::Receipt(key) => key.repository == repository,
        forge::Key::Call(_)
        | forge::Key::ProposedEffect(_)
        | forge::Key::Names(_)
        | forge::Key::Landed(_)
        | forge::Key::Entry(_)
        | forge::Key::Change(_)
        | forge::Key::Issue(_)
        | forge::Key::Release(_)
        | forge::Key::Application(_)
        | forge::Key::Writer(_)
        | forge::Key::Binding(_)
        | forge::Key::Setting(_)
        | forge::Key::Flight { .. }
        | forge::Key::Landing(_)
        | forge::Key::Forge(_)
        | forge::Key::ForgeNumber
        | forge::Key::Playground
        | forge::Key::Finding { .. }
        | forge::Key::Dismissed { .. }
        | forge::Key::Making(_)
        | forge::Key::ProjectFacts { .. }
        | forge::Key::Looked { .. }
        | forge::Key::Away { .. }
        | forge::Key::Decided { .. }
        | forge::Key::Closer(_)
        | forge::Key::Device { .. }
        | forge::Key::Follow { .. }
        | forge::Key::Red { .. }
        | forge::Key::RedNumber
        | forge::Key::RedPass { .. }
        | forge::Key::ChangeCold(_)
        | forge::Key::ChangeHot(_)
        | forge::Key::HistoryProject { .. }
        | forge::Key::HistoryEvent { .. }
        | forge::Key::HistoryChange { .. }
        | forge::Key::HistoryLanding { .. }
        | forge::Key::HistoryLast { .. } => false,
    }
}

fn setting_is(edit: &forge::settings::Edit, repository: Repository) -> bool {
    use forge::settings::{Edit, Value};
    match edit {
        Edit::Set(value) => match &value.value {
            Value::Follow(follow) => follow.source.repository == repository || follow.target.repository == repository,
            Value::Rule(_)
            | Value::Charter(_)
            | Value::Questions(_)
            | Value::Ways(_)
            | Value::Classes(_)
            | Value::CharterWords(_)
            | Value::ProjectWords(_)
            | Value::Goal(_)
            | Value::Dependency(_)
            | Value::ProjectAreas(_)
            | Value::ProjectTriager(_) => false,
        },
        Edit::Remove | Edit::Reset => false,
    }
}

// Exhaustive, deliberately independent of the production release's list.
fn names_repository(row: (&Key, &Record), repository: Repository) -> bool {
    let (key, record) = row;
    if let Key::Forge(key) = key
        && key_is(key, repository)
    {
        return true;
    }
    let Record::Forge { row } = record else { return false };
    match row.as_ref() {
        forge::Stored::ProposedEffect(row) => row.entry.repository == repository,
        forge::Stored::Repository(row) => row.provider == repository,
        forge::Stored::Hold(row) => name_is(&row.name, repository),
        forge::Stored::Names { resources, .. } => resources.iter().any(|name| name_is(name, repository)),
        forge::Stored::Subscription(row) => topic_is(&row.topic, repository),
        forge::Stored::BranchHead(row) => name_is(&row.name, repository),
        forge::Stored::PullState(row) => name_is(&row.name, repository),
        forge::Stored::Ci(row) => row.repository == repository,
        forge::Stored::Entry { entry, .. } => entry.repository == repository,
        forge::Stored::Client(row) => match row {
            client::Stored::Live(row) => row.watch.resource.repository == repository,
            client::Stored::Repository(row) => row.repository == repository,
            client::Stored::Writer(_) => false,
        },
        forge::Stored::Change(row) => row.repository == repository,
        forge::Stored::ChangeCold(row) => row.repository == repository,
        forge::Stored::Issue(row) => row.repository == repository,
        forge::Stored::Release(row) => {
            row.pending.as_ref().is_some_and(|(_, name)| name.as_ref().is_some_and(|name| name_is(name, repository)))
        }
        forge::Stored::Application { outcome, .. } => match outcome {
            forge::ApplicationOutcome::Adopted(row) => row.repository == repository,
            forge::ApplicationOutcome::Follow(_)
            | forge::ApplicationOutcome::CoreRefused(_)
            | forge::ApplicationOutcome::ForgeRefused(_)
            | forge::ApplicationOutcome::MakeKept
            | forge::ApplicationOutcome::MakeRefused(_) => false,
        },
        forge::Stored::Landing(row) => row.landing.members.iter().any(|member| {
            member.repository.forge == repository.forge && member.repository.repository == repository.repository
        }),
        forge::Stored::Receipt { key, .. } => key.repository == repository,
        forge::Stored::Follow(row) => row.source == repository,
        forge::Stored::Tier(row) => row.repository == repository,
        forge::Stored::Verification(row) => row.repository == repository,
        forge::Stored::LandingLog(row) => row.repository == repository,
        forge::Stored::LandedDraft(row) => row.repository == repository,
        forge::Stored::Finding(row) => {
            row.signature.as_ref().is_some_and(|signature| signature.repository == repository)
        }
        forge::Stored::HistoryEvent(row) => row.repository == repository,
        forge::Stored::Setting(row) => setting_is(&row.edit, repository),
        forge::Stored::Making(row) => {
            row.key.forge == repository.forge
                && match row.state {
                    forge::MakingState::Made { id, .. } | forge::MakingState::Adopting { id, .. } => {
                        id == u64::from(repository.repository)
                    }
                    forge::MakingState::Waiting | forge::MakingState::Making | forge::MakingState::Failed(_) => false,
                }
        }
        // These contain task, project, forge or setting identities, not repository identities.
        // ChangeHot's ownership is pinned with the cold part's key in assert_whole.
        forge::Stored::RefusedSetting(_)
        | forge::Stored::Call { .. }
        | forge::Stored::Landed { .. }
        | forge::Stored::Closer(_)
        | forge::Stored::Writer(_)
        | forge::Stored::Binding { .. }
        | forge::Stored::Flight { .. }
        | forge::Stored::Forge(_)
        | forge::Stored::ForgeNumber { .. }
        | forge::Stored::Playground(_)
        | forge::Stored::Dismissed { .. }
        | forge::Stored::ProjectFacts(_)
        | forge::Stored::Looked(_)
        | forge::Stored::Away(_)
        | forge::Stored::Decided(_)
        | forge::Stored::Device(_)
        | forge::Stored::Red(_)
        | forge::Stored::RedPass(_)
        | forge::Stored::RedNumber { .. }
        | forge::Stored::ChangeHot(_)
        | forge::Stored::HistoryProject(_)
        | forge::Stored::HistoryChange(_)
        | forge::Stored::HistoryLanding(_)
        | forge::Stored::HistoryLast(_) => false,
    }
}

fn deletion_key() -> Key {
    Key::Core(core::Key::People(people::Key::Deletion(1)))
}

fn deletion(d: &Driver) -> Option<people::DeletionStep> {
    if let Some(Record::Core(core::Record::People(people::Stored::Deletion { step, .. }))) =
        d.store.rows.get(&deletion_key())
    {
        Some(*step)
    } else {
        None
    }
}

fn repository_rows(d: &Driver, repository: Repository) -> Rows {
    d.store
        .rows
        .iter()
        .filter(|row| names_repository(*row, repository))
        .map(|(key, row)| (key.clone(), row.clone()))
        .collect()
}

fn remove(d: &mut Driver) {
    let sign_in = d
        .store
        .rows
        .values()
        .find_map(|record| {
            if let Record::Core(core::Record::People(people::Stored::SignIn { number, .. })) = record {
                Some(*number)
            } else {
                None
            }
        })
        .expect("durable owner session");
    d.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(99)),
        sign_in,
        key: [99; 16],
        ask: people::Ask::RemoveProject { project: 1, grace: None },
    }));
}

fn assert_whole(d: &Driver, r: Repository, q: Repository, kept: &Rows) {
    ready(d);
    assert!(!d.store.rows.contains_key(&deletion_key()), "deletion ended, not merely admitted");
    for row in &d.store.rows {
        assert!(!names_repository(row, r), "released repository still named: {row:?}");
    }
    for key in [forge::Key::ChangeCold(CHANGE), forge::Key::ChangeHot(CHANGE)] {
        assert!(!d.store.rows.contains_key(&Key::Forge(key)), "both change parts erased");
    }
    assert!(!kept.is_empty(), "other project's rows were present");
    for (key, row) in kept {
        assert!(names_repository((key, row), q));
        assert_eq!(d.store.rows.get(key), Some(row), "other project's row unchanged: {key:?}");
    }
}

#[derive(Clone, Copy, Debug)]
enum Cut {
    Whole,
    BeforeRelease,
    BetweenPages,
    AfterRelease,
}

fn run(cut: Cut) -> Rows {
    let bulk = if matches!(cut, Cut::BetweenPages) { BULK } else { 0 };
    let (mut d, r, q) = fixture(bulk);
    let kept = repository_rows(&d, q);
    let owned = repository_rows(&d, r);
    remove(&mut d);
    if matches!(cut, Cut::Whole) {
        d.pump();
        assert_whole(&d, r, q, &kept);
    } else {
        let mut reached = false;
        for _ in 0..STEPS {
            if let Some((_, writes)) = d.store.pending.front() {
                let release = writes.iter().any(|write| {
                    matches!(
                        write,
                        Write::Save(Record::Core(core::Record::People(people::Stored::Deletion {
                            project: 1,
                            step: people::DeletionStep::Release,
                            ..
                        })))
                    )
                });
                if release && matches!(cut, Cut::BeforeRelease) {
                    assert!(
                        matches!(deletion(&d), Some(people::DeletionStep::Tasks | people::DeletionStep::Rows)),
                        "last durable step precedes the atomic Release save and first page"
                    );
                    assert_eq!(repository_rows(&d, r), owned, "all of R is still durable before release");
                    reached = true;
                    break;
                }
                let last = writes.contains(&Write::Erase(Key::Forge(forge::Key::Repository(r))));
                let number = d.store.apply();
                let left = repository_rows(&d, r).len();
                if matches!(cut, Cut::BetweenPages)
                    && deletion(&d) == Some(people::DeletionStep::Release)
                    && left > 0
                    && left < owned.len()
                {
                    assert!(
                        d.store.rows.contains_key(&Key::Forge(forge::Key::Repository(r))),
                        "repository erased last"
                    );
                    reached = true;
                    break; // First page durable, its terminal (and every later page) lost.
                }
                if last && matches!(cut, Cut::AfterRelease) {
                    assert_eq!(
                        deletion(&d),
                        Some(people::DeletionStep::Release),
                        "forge committed, core still waits for the answer"
                    );
                    assert_eq!(left, 0);
                    reached = true;
                    break; // Lose the commit terminal, hence the answer to the core.
                }
                d.send(engine::Event::Store(engine::StoreInput::Committed { number, fill: 0 }));
            } else {
                d.advance(false);
            }
        }
        assert!(reached, "cut reached: {cut:?}");
        let step = deletion(&d);
        let left = repository_rows(&d, r);
        eprintln!("cut {cut:?}: step={step:?}, R rows left={}/{}", left.len(), owned.len());
        d.store.crash(SEED);
        assert_eq!(deletion(&d), step, "crash keeps the durable deletion step");
        assert_eq!(repository_rows(&d, r), left, "crash keeps the applied pages alone");
    }
    let mut restarted = cold(d.take_store().fork());
    restarted.pump();
    assert_whole(&restarted, r, q, &kept);
    if matches!(cut, Cut::AfterRelease) {
        let completion = restarted
            .transactions
            .iter()
            .find(|writes| writes.contains(&Write::Erase(deletion_key())))
            .expect("restart re-asks the connector and ends the kept deletion");
        assert!(
            !completion
                .iter()
                .any(|write| matches!(write, Write::Save(Record::Forge { .. }) | Write::Erase(Key::Forge(_)))),
            "second release writes nothing but answers: {completion:?}"
        );
        assert!(!restarted.transactions.iter().flatten().any(|write| matches!(write,
            Write::Save(Record::Forge { row }) if matches!(row.as_ref(), forge::Stored::Repository(row) if row.provider == r))));
    }
    restarted.store.rows.clone()
}

#[test]
fn a_deleted_project_restarts_without_its_repository() {
    assert_eq!(run(Cut::Whole), run(Cut::Whole), "seed replays");
}

#[test]
fn a_deletion_cut_before_release_restarts_and_releases() {
    assert_eq!(run(Cut::BeforeRelease), run(Cut::BeforeRelease), "seed replays");
}

#[test]
fn a_deletion_cut_between_release_pages_restarts_and_releases() {
    assert_eq!(run(Cut::BetweenPages), run(Cut::BetweenPages), "seed replays");
}

#[test]
fn a_deletion_cut_after_the_forge_commit_reasks_without_a_write() {
    assert_eq!(run(Cut::AfterRelease), run(Cut::AfterRelease), "seed replays");
}
