//! One repository's unread facts hold none of another's decisions (forge/catching-up.md, 2 and 4).
#![expect(clippy::disallowed_types, clippy::disallowed_methods, reason = "step stories collect and inspect requests")]
use super::*;
use skein_forge::api::{self, Answer, Error, Op, Read, Write};
use temper_engine_domain_forge_change as c;

const OTHER: api::Repository = api::Repository { forge: 1, repository: 3 };
const BRANCH: &[u8] = b"temper/51";
const TASK: u64 = 51;
const OTHER_TASK: u64 = 52;

fn name(repository: api::Repository) -> Name {
    crate::tiers::name(repository, BRANCH)
}
fn restore(d: &mut Domain, record: Stored) {
    outputs(d, Event::Restore { record });
}
fn watched() -> Domain {
    let mut d = fixtures::connected_domain();
    for (repository, task) in [(REPO, TASK), (OTHER, OTHER_TASK)] {
        restore(&mut d, Stored::Repository(Repository { provider: repository, ..adopted() }));
        restore(&mut d, Stored::Names { task, resources: Box::new([name(repository)]) });
        restore(&mut d, Stored::BranchHead(BranchHead { name: name(repository), commit: fixtures::HEAD, owned: true }));
    }
    d
}
fn begin(d: &mut Domain) {
    outputs(d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    assert!(!d.repository_fresh(REPO));
    assert!(!d.repository_fresh(OTHER), "a pass not yet begun has no fresh repository");
    outputs(d, Event::CatchUp);
}
fn answer(read: Read) -> Result<Answer, Error> {
    match read {
        Read::Branch { branch } => {
            Ok(Answer::Commit(if branch.as_ref() == b"main" { fixtures::BASE } else { fixtures::HEAD }))
        }
        Read::Items { .. } => Ok(Answer::Items { cut: false, items: Box::new([]), more: false, now: env().now }),
        Read::Item { number, .. } => Ok(Answer::Item {
            item: api::Summary {
                number,
                kind: api::Kind::Issue,
                state: api::State::Open,
                author: 7,
                key: None,
                labels: Box::new([]),
                title: Box::new([]),
                body: Box::new([]),
                updated: env().now,
            },
            comments: Box::new([]),
            more: false,
        }),
        Read::Pull { .. } => Ok(Answer::Pull(fixtures::open_pull())),
        Read::Commits { .. } => Ok(Answer::Commits { commits: Box::new([]), more: false }),
        Read::Reviews { .. } => Ok(Answer::Reviews { reviews: Box::new([]), more: false, cut: false }),
        Read::Checks { .. } => Ok(fixtures::checks(api::Check::Passed)),
        Read::Tree { .. } => Ok(Answer::Tree { entries: Box::new([]), more: false }),
        Read::Runs { .. } => Ok(Answer::Runs { runs: Box::new([]), more: false }),
        Read::Branches { .. }
        | Read::Locate { .. }
        | Read::Collaborators { .. }
        | Read::Permission { .. }
        | Read::Protection { .. }
        | Read::Settings
        | Read::File { .. }
        | Read::Job { .. }
        | Read::Run { .. }
        | Read::PullFor { .. }
        | Read::PullFiles { .. }
        | Read::Statuses { .. }
        | Read::Remarks { .. }
        | Read::Compare { .. }
        | Read::JobState { .. }
        | Read::Artifact { .. } => Err(Error::Missing),
    }
}
/// Drive the existing client, leaving every attempt on repository 2 rate limited.
fn available(d: &mut Domain, e: &Env<Limits>, blocked: bool) -> Vec<Request> {
    let mut seen = Vec::new();
    for _ in 0_u32..64 {
        let mut out = Queue::uncapped();
        for _ in 0..e.limits.client.calls {
            resume(d, e, &mut out);
        }
        if out.is_empty() {
            return seen;
        }
        while let Some(request) = out.pop() {
            if let Request::Call { call, repository, op, .. } = &request {
                let result = match op {
                    Op::Read(read) if !blocked || *repository != REPO => answer(read.clone()),
                    Op::Read(_) => Err(Error::RateLimited { after: Duration::from_secs(1) }),
                    Op::Write(_) => panic!("test must commit and inspect writes separately: {op:?}"),
                };
                step(
                    d,
                    e,
                    Event::Client(skein_forge_client::Event::Answered {
                        free: 0,
                        rate: None,
                        call: *call,
                        cost: 1,
                        result,
                    }),
                    &mut out,
                );
            }
            seen.push(request);
        }
    }
    panic!("bounded fresh reads did not quiesce")
}
fn one_wake(requests: &[Request], task: u64) {
    assert_eq!(requests.iter().filter(|r| **r == Request::WakeProcedure { task }).count(), 1, "{requests:?}");
}
fn no_report(requests: &[Request]) {
    assert!(!requests.contains(&Request::RestartDone { stage: RestartStage::ReadAfresh }));
}
fn hold(repository: api::Repository, task: u64) -> Hold {
    Hold { name: name(repository), task, writer: None, design_writers: Box::new([]), beside: None, drift: None }
}
fn release(task: u64) -> ReleaseRow {
    ReleaseRow {
        task,
        root: task,
        ending: ReleaseEnding::Done,
        close_pull: None,
        comment: None,
        pending: None,
        failed: false,
        effects_settled: true,
        releasing: true,
        room: None,
    }
}

#[test]
fn repositories_finish_independently_and_a_dropped_watch_ends_the_forge_pass() {
    assert!(domain().repository_fresh(OTHER));
    let mut d = watched();
    begin(&mut d);
    let mut e = env();
    for _ in 0_u32..4 {
        let seen = available(&mut d, &e, true);
        no_report(&seen);
        assert!(d.repository_fresh(OTHER), "repository 3: {:?}; {seen:?}", d.catching_up(1));
        assert!(!d.repository_fresh(REPO));
        assert!(!d.fresh_reads_in());
        e.now = e.now.saturating_add(Duration::from_secs(50));
        let mut out = Queue::uncapped();
        fire(&mut d, &e, &mut out);
        assert!(!out.iter().any(|request| *request == Request::RestartDone { stage: RestartStage::ReadAfresh }));
    }
    let done = outputs(
        &mut d,
        Event::Client(skein_forge_client::Event::Keep { owner: Token::new(900), watches: Box::new([]) }),
    );
    assert!(done.contains(&Request::RestartDone { stage: RestartStage::ReadAfresh }));
    assert!(d.fresh_reads_in());
    assert!(d.repository_fresh(REPO));
    assert!(d.repository_fresh(OTHER));
}

#[test]
fn releases_wake_once_on_their_repository_and_send_their_effects() {
    let mut d = watched();
    for (repository, task) in [(REPO, TASK), (OTHER, OTHER_TASK)] {
        restore(&mut d, Stored::Hold(hold(repository, task)));
        restore(&mut d, Stored::Release(release(task)));
    }
    begin(&mut d);
    for task in [TASK, OTHER_TASK] {
        let parked = outputs(&mut d, Event::ContinueRelease { task, entry: 90 });
        assert!(!parked.iter().any(|r| matches!(r, Request::Save { record: Stored::Entry { .. }, .. })));
    }
    let mut e = env();
    let seen = available(&mut d, &e, true);
    assert_eq!(seen.iter().filter(|r| **r == Request::ContinueRelease { task: OTHER_TASK }).count(), 1, "{seen:?}");
    assert!(!seen.contains(&Request::ContinueRelease { task: TASK }));
    no_report(&seen);
    outputs(&mut d, Event::ContinueRelease { task: OTHER_TASK, entry: 90 });
    assert_eq!(d.entries.get(&90).unwrap().entry.repository, OTHER);
    assert_eq!(d.entries.get(&90).unwrap().entry.effect.write, Write::DeleteBranch { branch: BRANCH.into() });
    let unchanged = available(&mut d, &e, true);
    assert!(!unchanged.contains(&Request::ContinueRelease { task: OTHER_TASK }));
    e.now = e.now.saturating_add(Duration::from_secs(50));
    let mut clock = Queue::uncapped();
    fire(&mut d, &e, &mut clock);
    let finished = available(&mut d, &e, false);
    assert_eq!(finished.iter().filter(|r| **r == Request::ContinueRelease { task: TASK }).count(), 1, "{finished:?}");
    assert!(!finished.contains(&Request::ContinueRelease { task: OTHER_TASK }));
    outputs(&mut d, Event::ContinueRelease { task: TASK, entry: 91 });
    assert_eq!(d.entries.get(&91).unwrap().entry.repository, REPO);
}

#[test]
fn a_done_producer_finishes_its_release_while_the_other_repository_is_limited() {
    let mut d = watched();
    let mut row = fixtures::opening_row();
    row.repository = OTHER;
    row.task = OTHER_TASK;
    row.change.task = OTHER_TASK;
    row.change.state = c::State::Producing { requested: true };
    row.pending = None;
    row.effect = c::EffectResult::None;
    row.delegate = Some((72, c::Delegate::Produce));
    row.result.head = Some(fixtures::HEAD);
    restore(&mut d, Stored::Change(Box::new(row)));
    let mut held = hold(OTHER, OTHER_TASK);
    held.writer = Some(Writer::Lost { task: 72, attempt: 1 });
    restore(&mut d, Stored::Hold(held));
    restore(&mut d, Stored::Release(ReleaseRow { root: OTHER_TASK, effects_settled: false, ..release(72) }));
    begin(&mut d);
    outputs(&mut d, Event::ContinueRelease { task: 72, entry: 90 });
    let mut e = env();
    let seen = retry_available(&mut d, &mut e, true);
    assert_eq!(seen.iter().filter(|r| **r == Request::ContinueRelease { task: 72 }).count(), 1, "{seen:?}");
    assert!(!d.fresh_reads_in());
    let settled = outputs(&mut d, Event::ContinueRelease { task: 72, entry: 90 });
    assert!(settled.contains(&Request::EffectsSettled { task: 72 }), "{settled:?}");
    let released = outputs(&mut d, Event::ContinueRelease { task: 72, entry: 90 });
    assert!(released.contains(&Request::Released { task: 72 }), "{released:?}");
    assert_eq!(d.hold(&name(OTHER)).unwrap().writer, None);
    assert!(!d.repository_fresh(REPO));
    no_report(&seen);
}

#[test]
fn a_procedure_waiting_on_two_repositories_never_wakes_on_just_one() {
    let mut d = watched();
    begin(&mut d);
    assert!(domain::wait_fresh(&mut d, 60, &[OTHER]));
    assert!(domain::wait_fresh(&mut d, 61, &[REPO]));
    assert!(domain::wait_fresh(&mut d, 61, &[OTHER]));
    let mut e = env();
    let first = available(&mut d, &e, true);
    one_wake(&first, 60);
    assert!(!first.contains(&Request::WakeProcedure { task: 61 }));
    assert!(!available(&mut d, &e, true).contains(&Request::WakeProcedure { task: 60 }));
    e.now = e.now.saturating_add(Duration::from_secs(50));
    fire(&mut d, &e, &mut Queue::uncapped());
    let last = available(&mut d, &e, false);
    one_wake(&last, 61);
    assert!(!last.contains(&Request::WakeProcedure { task: 60 }));
}

#[test]
fn write_claims_and_workspace_items_use_their_own_repository() {
    let mut d = watched();
    for (repository, task) in [(REPO, TASK), (OTHER, OTHER_TASK)] {
        restore(&mut d, Stored::Hold(hold(repository, task)));
    }
    begin(&mut d);
    for (repository, task) in [(REPO, TASK), (OTHER, OTHER_TASK)] {
        outputs(
            &mut d,
            Event::Claim {
                task,
                attempt: 1,
                writes: Box::new([name(repository)]),
                designs: Box::new([]),
                holders: Box::new([]),
            },
        );
        assert_eq!(d.hold(&name(repository)).unwrap().writer, None);
    }
    let seen = available(&mut d, &env(), true);
    one_wake(&seen, OTHER_TASK);
    assert!(!seen.contains(&Request::WakeProcedure { task: TASK }));
    assert_eq!(d.hold(&name(OTHER)).unwrap().writer, Some(Writer::Run { task: OTHER_TASK, attempt: 1 }));
    assert_eq!(d.hold(&name(REPO)).unwrap().writer, None);
    for (repository, task) in [(REPO, TASK), (OTHER, OTHER_TASK)] {
        let item = items::Seed {
            project: 5,
            modify: true,
            task,
            parent: None,
            repository,
            run_branch: b"temper/7/r7".as_slice().into(),
            saved_branch: None,
        };
        assert_eq!(d.workspace_item(item).is_some(), repository == OTHER);
    }
}

fn retry_available(d: &mut Domain, e: &mut Env<Limits>, blocked: bool) -> Vec<Request> {
    let mut seen = Vec::new();
    for _ in 0_u32..16 {
        seen.extend(available(d, e, blocked));
        e.now = e.now.saturating_add(Duration::from_secs(1));
        let mut out = Queue::uncapped();
        fire(d, e, &mut out);
        while let Some(request) = out.pop() {
            seen.push(request);
        }
    }
    seen
}

#[test]
fn projections_wake_only_when_their_repository_is_fresh() {
    let mut d = watched();
    for (repository, goal) in [(REPO, 71), (OTHER, 72)] {
        let mut view = projection::goal_view();
        view.goal = goal;
        view.repository = u64::from(repository.repository);
        restore(
            &mut d,
            Stored::Issue(IssueRow {
                goal,
                repository,
                state: issues::Projected::default(),
                number: None,
                pending: None,
                before: None,
                failed: false,
                desired: Some(view),
            }),
        );
    }
    begin(&mut d);
    for goal in [71, 72] {
        let parked = outputs(&mut d, Event::ProjectDesired { entry: 90, goal });
        assert!(!parked.iter().any(|r| matches!(r, Request::ProjectionEffect { .. })));
    }
    let mut e = env();
    let seen = retry_available(&mut d, &mut e, true);
    assert_eq!(seen.iter().filter(|r| **r == Request::ProjectAfter { goal: 72, when: None }).count(), 1, "{seen:?}");
    assert!(!seen.contains(&Request::ProjectAfter { goal: 71, when: None }));
    let next = outputs(&mut d, Event::ProjectDesired { entry: 90, goal: 72 });
    assert!(next.iter().any(|r| matches!(r, Request::ProjectionEffect { .. })), "{next:?}");
    no_report(&seen);
}

#[test]
fn tier_restart_starts_only_the_repository_whose_tip_is_in() {
    use temper_engine_domain_forge_tiers as p;
    let live = tiers::dispatched();
    let template = tiers::row(&live).clone();
    let mut d = watched();
    let mut repositories = Vec::new();
    for repository in [REPO, OTHER] {
        repositories.push(RepositoryGates {
            repository,
            gates: Box::new([]),
            tiers: Box::new([Tier {
                name: b"browser".as_slice().into(),
                workflow: b"tier.yml".as_slice().into(),
                ignored: Box::new([]),
                artifact: b"report".as_slice().into(),
                format: p::Format::Lines,
                follows: Follows::Branch(BRANCH.into()),
                span: None,
                description: Box::new([]),
            }]),
        });
        let mut row = template.clone();
        row.repository = repository;
        row.record.run = None;
        row.record.judged = Some(p::Judged { tip: fixtures::BASE, key: fixtures::BASE });
        row.baseline = Some(TierLanding { commit: fixtures::HEAD, key: Some(fixtures::HEAD) });
        row.pending = Box::new([]);
        row.task = None;
        row.requested = false;
        row.flight = None;
        restore(&mut d, Stored::Tier(row));
        restore(
            &mut d,
            Stored::BranchHead(BranchHead {
                name: crate::tiers::name(repository, b"main"),
                commit: fixtures::HEAD,
                owned: false,
            }),
        );
    }
    assert!(d.project_landing(
        5,
        c::Way::Queue,
        repositories.into_boxed_slice(),
        temper_engine_domain_forge_landing::Project::DEFAULT,
        AfterLanding {
            wait: c::Wait::Cleared,
            verdict: p::Settings { retries: 2, suspects: p::Suspects::Bisect },
            bound: Duration::from_secs(7200),
        }
    ));
    begin(&mut d);
    let mut e = env();
    let seen = retry_available(&mut d, &mut e, true);
    assert!(d.repository_fresh(OTHER));
    assert!(!d.repository_fresh(REPO));
    assert!(seen.iter().any(|r| matches!(r, Request::TierRun { repository: OTHER, .. })), "{seen:?}");
    assert!(!seen.iter().any(|r| matches!(r, Request::TierRun { repository: REPO, .. })));
    no_report(&seen);
}

#[test]
fn landing_judgement_waits_for_all_member_repositories() {
    let mut d = watched();
    let mut change = fixtures::opening_row();
    change.repository = OTHER;
    change.pending = None;
    restore(&mut d, Stored::Change(Box::new(change)));
    begin(&mut d);
    let mut row = landing_routes::merge_landing_row(true);
    row.landing.members[0].repository =
        temper_engine_domain_forge_landing::Repository { forge: OTHER.forge, repository: OTHER.repository };
    let evidence = ChangeEvidence {
        touched: Touched::UNKNOWN,
        prior: fixtures::opening_row().change,
        head: Some(fixtures::HEAD),
        base_tip: fixtures::BASE,
        contains_base: c::Status::Passed,
        ci: c::Status::Passed,
        gates: Box::new([]),
        reviews: Box::new([]),
        reviews_complete: true,
        size: None,
    };
    assert_eq!(landing::judge(&d, &row, Criterion::UpToDate, fixtures::HEAD, &evidence, &[]), Some(JudgeVerdict::Wait));
    available(&mut d, &env(), true);
    assert_eq!(landing::judge(&d, &row, Criterion::UpToDate, fixtures::HEAD, &evidence, &[]), Some(JudgeVerdict::Met));
    let mut held = row.landing.members[0].clone();
    held.repository = temper_engine_domain_forge_landing::Repository { forge: REPO.forge, repository: REPO.repository };
    row.landing.members = vec![row.landing.members[0].clone(), held].into_boxed_slice();
    assert_eq!(landing::judge(&d, &row, Criterion::UpToDate, fixtures::HEAD, &evidence, &[]), Some(JudgeVerdict::Wait));
}

#[test]
fn verified_changes_judge_fresh_heads_without_waiting_for_another_repository() {
    let mut d = watched();
    for (repository, task) in [(REPO, TASK), (OTHER, OTHER_TASK)] {
        let mut row = fixtures::opening_row();
        row.repository = repository;
        row.task = task;
        row.change.task = task;
        row.pending = None;
        row.change.state = c::State::Landed { merge: fixtures::BASE, since: env().wall };
        restore(&mut d, Stored::Change(Box::new(row)));
        restore(
            &mut d,
            Stored::Verification(Verification {
                repository,
                commit: fixtures::BASE,
                changes: Box::new([task]),
                tiers: Box::new([TierVerification {
                    tier: b"browser".as_slice().into(),
                    verdict: None,
                    repaired: None,
                }]),
            }),
        );
    }
    begin(&mut d);
    for task in [TASK, OTHER_TASK] {
        let Event::StepChange { merge_job, entry, heard, gates, queue_repair_active, pending, kept, .. } =
            fixtures::change_step()
        else {
            unreachable!()
        };
        let parked = outputs(
            &mut d,
            Event::StepChange { task, merge_job, entry, heard, gates, queue_repair_active, pending, kept },
        );
        assert!(!parked.iter().any(|r| matches!(r, Request::ChangeDecision { .. })));
    }
    let seen = available(&mut d, &env(), true);
    assert!(seen.iter().any(|r| matches!(r, Request::ChangeDecision { task: OTHER_TASK, .. })), "{seen:?}");
    assert!(!seen.iter().any(|r| matches!(r, Request::ChangeDecision { task: TASK, .. })));
    no_report(&seen);
}
