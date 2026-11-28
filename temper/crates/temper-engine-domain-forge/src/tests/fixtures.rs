//! Shared adoption and change fixtures.
use super::*;

/// Admit the worker's push before the fake exposes it to the connector.
pub fn admit_push(
    d: &mut Domain,
    name: &Name,
    task: u64,
    attempt: u64,
    before: Option<skein_forge::api::Commit>,
    after: skein_forge::api::Commit,
) {
    d.admit_push(&env(), name, before, after, Sender::Attempt { task, attempt }, &mut Queue::uncapped())
        .expect("fixture push is admitted");
}

/// Save the outbox flight of a write whose forge reply the test scripts.
pub fn send_write(d: &mut Domain, number: u64) {
    let mut entry = d.entries.get(&number).expect("admitted outbox entry").entry.clone();
    let clock = env();
    entry.start = Some(skein_forge_client::Position { at: clock.now, comment: 0, review: 0, review_page: 1 });
    entry.attempt = Some(skein_forge_client::Attempt {
        sent: clock.now,
        deadline: clock.now.saturating_add(clock.limits.client.poll),
        wall: clock.wall,
        expires: Wall::from_nanos(clock.wall.as_nanos().saturating_add(clock.limits.client.poll.as_nanos())),
    });
    domain::save_entry_progress(d, entry, None, &mut Queue::uncapped());
}

/// Change reads start after routing has reported access.
pub fn connected_domain() -> Domain {
    let mut d = domain();
    connect(&mut d);
    d
}

pub fn adopted_domain() -> Domain {
    let mut d = connected_domain();
    outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
    outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    outputs(&mut d, Event::CatchUp);
    d
}

pub fn connect(d: &mut Domain) {
    outputs(d, Event::Connection { forge: 1, state: Connection::Connected { id: 7, login: b"bot".as_slice().into() } });
}
pub fn adoption() -> Adoption {
    Adoption {
        project: 5,
        home: true,
        forge: REPO.forge,
        host: Box::from(&b"forge.example"[..]),
        owner: Box::from(&b"org"[..]),
        name: Box::from(&b"repo"[..]),
        prefix: Box::from(&b"temper/"[..]),
        role: Role::Owned,
        landing: Box::from(&b"main"[..]),
        ci: true,
        checks: Box::new([]),
    }
}

/// Play the core after the connector's locate, returning the adoption outputs.
pub fn begin_adoption(d: &mut Domain, request: Token, adoption: Adoption) -> Box<[Request]> {
    let started = outputs(
        d,
        Event::Application {
            role: None,
            carried: None,
            request,
            token: request,
            retention: Duration::from_secs(60),
            ask: Ask::Adopt(adoption),
        },
    );
    for output in &started {
        if let Request::AdoptAccess { forge, request, .. } = output {
            outputs(d, Event::AccessAnswered { forge: *forge, request: *request, answer: Ok(()) });
        }
    }
    let sent = ready(d);
    let [Request::Call { call, .. }] = sent.as_ref() else { panic!("one locate call") };
    outputs(
        d,
        Event::Client(skein_forge_client::Event::Answered {
            free: 0,
            rate: None,
            call: *call,
            cost: 1,
            result: Ok(skein_forge::api::Answer::Located { repository: REPO.repository }),
        }),
    );
    outputs(d, Event::Adopt { request })
}

/// This change and its outbox entry identify the one opening in flight.
pub const CHANGE: u64 = 51;
pub const OPEN: u64 = 11;
/// The branch pair differs so each fresh read has an unambiguous subject.
pub const HEAD: skein_forge::api::Commit = [3; 32];
pub const BASE: skein_forge::api::Commit = [4; 32];

pub fn open_pull() -> skein_forge::api::Pull {
    skein_forge::api::Pull {
        reviewers: Box::new([]),
        number: 1,
        state: skein_forge::api::State::Open,
        head: Box::from(b"temper/51".as_slice()),
        base: Box::from(b"main".as_slice()),
        commit: HEAD,
        base_commit: Some(BASE),
        merged: None,
        mergeable: true,
        ci: skein_forge::api::Ci::Passed,
        queued: None,
        dequeued: None,
        size: None,
    }
}

pub fn frozen_rows() -> Box<[ChangeRow]> {
    use temper_engine_domain_forge_change::State;
    let mut rows = skein_lib::List::uncapped();
    for state in [
        State::Landed { merge: HEAD, since: env().wall },
        State::Red { merge: HEAD, since: env().wall },
        State::Cleared { merge: HEAD },
        State::Uncleared { merge: HEAD },
        State::NoChange { words: Box::new([]) },
        State::Ended,
    ] {
        let mut row = opening_row();
        row.change.state = state;
        rows.push(row).unwrap();
    }
    for ending in
        [Ending::Landed, Ending::NoChange, Ending::Cancelled, Ending::Failed { line: b"failed".as_slice().into() }]
    {
        let mut row = opening_row();
        row.change.state = State::Checking { head: HEAD };
        row.ending = Some(ending);
        rows.push(row).unwrap();
    }
    rows.into_boxed()
}

pub fn opening_row() -> ChangeRow {
    use temper_engine_domain_forge_change as change;
    ChangeRow {
        settling: false,
        behind_wait: None,
        room: None,
        source: None,
        kept: None,
        regate: None,
        ci_recheck: None,
        task_release: None,
        job: None,
        signed: Box::new([]),
        task: CHANGE,
        repository: REPO,
        branch: Box::from(b"temper/51".as_slice()),
        base: Box::from(b"main".as_slice()),
        title: Box::from(b"Small fix".as_slice()),
        body: Box::new([]),
        priority: 1,
        ci: None,
        asks: Box::new([]),
        handed: 0,
        previous: None,
        change: change::Change {
            ended_held: None,
            early: change::Early::None,
            way: temper_engine_domain_forge_change::Way::Queue,

            task: CHANGE,
            state: change::State::Opening { requested: true },
            gates: Box::new([]),
            clean: Box::new([]),
            gate_failures: Box::new([]),
            released_resolution: false,
            released: None,
            repairs: 0,
            resolutions: 0,
            updates: 0,
            stale: None,
            since: env().wall,
            ready_since: None,
            owns_turn: false,
            last_head: Some(HEAD),
            last_checked: None,
            last_checked_by: None,
            clock: None,
        },
        pull: None,
        pending: Some(OPEN),
        effect: change::EffectResult::Pending,
        delegate: None,
        delegate_status: change::Status::Passed,
        spec_revision: 0,
        gate_revision: 0,
        delegate_flake: false,
        delegate_no_test: false,
        delegate_head: None,
        delegate_words: None,
        verdicts: Box::new([]),
        milestone: None,
        person: None,
        goal_title: None,
        draft: None,
        goal: None,
        ending: None,
        repair_tasks: Box::new([]),
        resolution_tasks: Box::new([]),
        result: ChangeResult::empty(),
        gate_remarks: Box::new([]),
        drift: None,
        base_repair: false,
        queue_repair: None,
        queue_repairs: 0,
        producing_charter: 2,
        review_charter: 4,
        producing_llm: None,
        review_llm: None,

        questions: temper_engine_domain_forge_change::Questions::default(),
        effect_account: None,
        producer_failure: None,
        temper_dir_fault: None,
        runner_ci: None,
        close_at: None,
        report: Box::new([]),
        repairs: None,
        follows: None,
        catch_up: None,
        close: None,
        merge_result: None,
        history: None,
        follow_check: None,
    }
}

pub fn opening() -> (Domain, Token) {
    opening_with(opening_row())
}

pub fn opening_with(row: ChangeRow) -> (Domain, Token) {
    let mut d = adopted_domain();
    // Opening is admitted against the head already observed for the change.
    let name = domain::writes::branch(&LIMITS, row.repository, &row.branch).expect("fixture branch name");
    let commit = row.change.last_head.expect("opening head");
    d.heads.insert(name.clone(), BranchHead { name, commit, owned: true }).expect("fixture head room");

    let entry = skein_forge_client::Entry {
        number: OPEN,
        task: row.task,
        repository: REPO,
        effect: skein_forge_client::Effect {
            write: skein_forge::api::Write::OpenPull {
                title: row.title.clone(),
                // The tiny outbox holds the fitted effect, not the stored brief.
                body: text::fit(&row.body, 256, b"..."),
                head: row.branch.clone(),
                base: row.base.clone(),
            },
            condition: skein_forge_client::Condition::None,
        },
        start: None,
        attempt: None,
        failures: 0,
        queued: false,
    };
    outputs(&mut d, Event::Change { row });
    outputs(&mut d, Event::Enqueue { entry });
    outputs(&mut d, Event::Committed { entry: OPEN });
    answer_next(
        &mut d,
        Ok(skein_forge::api::Answer::Items { cut: false, items: Box::new([]), more: false, now: env().now }),
    );
    let (call, _) = next_call(&mut d);
    (d, call)
}

/// Take a call under the caller's clock, including connector-owned outage waits.
pub fn next_at(d: &mut Domain, env: &Env<Limits>) -> (Token, skein_forge::api::Op) {
    landing_seen(d);
    let mut out = Queue::uncapped();
    for _ in 0..env.limits.client.pending {
        resume(d, env, &mut out);
        while let Some(request) = out.pop() {
            if let Request::Call { call, op, .. } = request {
                return (call, op);
            }
        }
    }
    panic!("one forge call at {:?}", env.now)
}

pub fn answer_at(
    d: &mut Domain,
    env: &Env<Limits>,
    call: Token,
    result: Result<skein_forge::api::Answer, skein_forge::api::Error>,
) -> Queue<Request> {
    let mut out = Queue::uncapped();
    step(
        d,
        env,
        Event::Client(skein_forge_client::Event::Answered { free: 0, rate: None, call, cost: 1, result }),
        &mut out,
    );
    d.reclaim();
    out
}

/// Finish the background revision reads before scripting a scenario's own calls.
/// These homes have no `.temper/`: asking the forge to list it returns Missing.
pub fn settle_home_reads(d: &mut Domain, clock: &Env<Limits>, tip: skein_forge::api::Commit) {
    use skein_forge::api::{Answer, Error, Op, Read};
    while d.temper_revisions.has_pending_read() {
        assert!(crate::temper_revision::ready(d), "settle home reads before other forge work");
        let mut out = Queue::uncapped();
        crate::temper_revision::resume(d, clock, &mut out);
        assert!(out.is_empty(), "revision read is queued in the forge client: {out:?}");
        // Send just the queued read; leave procedure wakes and outbox work for the scenario.
        let mut client = Queue::uncapped();
        skein_forge_client::resume(&mut d.client, &clock.with(clock.limits.client), &mut client);
        let Some(skein_forge_client::Request::Call { call, repository, op }) = client.pop() else {
            panic!("expected a home revision call: {client:?}");
        };
        assert!(client.is_empty(), "one home revision call at a time");
        let home = d.repository(repository).expect("revision reads an adopted repository");
        assert!(home.home, "only a home has a revision read");
        let result = match &op {
            Op::Read(Read::Branch { branch }) => {
                assert_eq!(*branch, home.landing);
                Ok(Answer::Commit(tip))
            }
            Op::Read(Read::File { head, path, .. }) => {
                assert_eq!(*head, tip);
                assert_eq!(path.as_ref(), b".temper");
                Err(Error::Missing)
            }
            other @ (Op::Read(_) | Op::Write(_)) => panic!("unexpected home revision call: {other:?}"),
        };
        let mut answered = answer_at(d, clock, call, result);
        if matches!(op, Op::Read(Read::File { .. })) {
            assert!(matches!(answered.pop(), Some(Request::SaveTemperRevision { row, .. })
                if row.home == repository && !row.method && row.refused.is_none()));
        }
        assert!(answered.is_empty(), "ordinary home read has no other effects: {answered:?}");
    }
}

/// Finish a change's path comparison in a fixture that changes no `.temper/` path.
pub fn ordinary_paths(d: &mut Domain, clock: &Env<Limits>, mut out: Queue<Request>) -> Queue<Request> {
    for (_, step) in &d.steps {
        if step.comparing_paths() {
            let (call, op) = next_at(d, clock);
            let skein_forge::api::Op::Read(skein_forge::api::Read::Compare { before, after }) = op else {
                panic!("expected the change's comparison, got {op:?}");
            };
            let paths = skein_forge::api::Answer::Compare {
                before,
                after,
                ahead: 1,
                behind: 0,
                changed: 0,
                files: Box::new([]),
            };
            let mut finished = answer_at(d, clock, call, Ok(paths));
            while let Some(request) = finished.pop() {
                out.push(request);
            }
            break;
        }
    }
    out
}

fn next_method_probe(d: &Domain) -> Option<skein_forge::api::Read> {
    for (_, step) in &d.steps {
        if let Some(read) = step.method_probe() {
            return Some(read.clone());
        }
    }
    None
}

/// Answer a change's method probes when neither its head nor its base has `.temper/`.
pub fn ordinary_method_probes(d: &mut Domain, clock: &Env<Limits>, mut out: Queue<Request>) -> Queue<Request> {
    use skein_forge::api::{Error, Op, Read};
    while let Some(read) = next_method_probe(d) {
        assert!(matches!(&read, Read::File { path, .. } if path.as_ref() == b".temper"));
        let (call, op) = next_at(d, clock);
        assert_eq!(op, Op::Read(read));
        let mut answered = answer_at(d, clock, call, Err(Error::Missing));
        while let Some(request) = answered.pop() {
            out.push(request);
        }
    }
    out
}

pub fn answer_ordinary_at(
    d: &mut Domain,
    clock: &Env<Limits>,
    call: Token,
    result: Result<skein_forge::api::Answer, skein_forge::api::Error>,
) -> Queue<Request> {
    let reviewed = matches!(result, Ok(skein_forge::api::Answer::Reviews { .. }));
    let out = answer_at(d, clock, call, result);
    if reviewed { ordinary_paths(d, clock, out) } else { out }
}

pub fn answer_ordinary(
    d: &mut Domain,
    call: Token,
    result: Result<skein_forge::api::Answer, skein_forge::api::Error>,
) -> Box<[Request]> {
    collected(answer_ordinary_at(d, &env(), call, result))
}

pub fn answer_next_ordinary(
    d: &mut Domain,
    result: Result<skein_forge::api::Answer, skein_forge::api::Error>,
) -> Box<[Request]> {
    let (call, _) = next_call(d);
    answer_ordinary(d, call, result)
}

pub fn answer_next_without_method(
    d: &mut Domain,
    result: Result<skein_forge::api::Answer, skein_forge::api::Error>,
) -> Box<[Request]> {
    let clock = env();
    let (call, _) = next_call(d);
    let out = answer_ordinary_at(d, &clock, call, result);
    collected(ordinary_method_probes(d, &clock, out))
}

pub fn advance(d: &mut Domain, env: &mut Env<Limits>, by: Duration) -> Queue<Request> {
    env.now = env.now.saturating_add(by);
    env.wall = Wall::from_nanos(env.wall.as_nanos().saturating_add(by.as_nanos()));
    let mut out = Queue::uncapped();
    fire(d, env, &mut out);
    out
}

pub fn ancestry_cases()
-> [(Result<skein_forge::api::Answer, skein_forge::api::Error>, temper_engine_domain_forge_change::Status); 8] {
    use skein_forge::api::{Answer, CommitMessage, Error};
    use temper_engine_domain_forge_change::Status;
    let listed = CommitMessage { id: BASE, parents: Box::new([]), message: Box::new([]) };
    [
        (Ok(Answer::Commits { commits: Box::new([]), more: false }), Status::Passed),
        (Ok(Answer::Commits { commits: Box::new([listed.clone()]), more: false }), Status::Failed),
        (Ok(Answer::Commits { commits: Box::new([listed]), more: true }), Status::Failed),
        (Err(Error::TooLarge(skein_forge::api::Excess::unnamed())), Status::Failed),
        (Ok(Answer::Commits { commits: Box::new([]), more: true }), Status::Unknown),
        (Err(Error::Unavailable), Status::Unknown),
        (Err(Error::Forbidden), Status::Unknown),
        (Err(Error::Missing), Status::Unknown),
    ]
}

/// A kept repository's delivery establishes hearing without a resource read.
pub fn repository_heard(d: &mut Domain, clock: &Env<Limits>, repository: skein_forge::api::Repository) {
    let mut out = Queue::uncapped();
    step(
        d,
        clock,
        Event::Client(skein_forge_client::Event::Keep {
            owner: Token::new(777),
            watches: Box::new([skein_forge_client::Watch {
                resource: skein_forge_client::Resource { repository, what: skein_forge_client::What::Repository },
                participating: false,
            }]),
        }),
        &mut out,
    );
    step(
        d,
        clock,
        Event::Client(skein_forge_client::Event::Hint {
            hint: skein_forge::api::Hint {
                repository,
                change: skein_forge::api::Change::Branch(b"main".as_slice().into()),
                key: None,
            },
        }),
        &mut out,
    );
    d.reclaim();
}

// Deliberately differs from the stock second used for Busy admission.
pub const GATHER_BACKOFF: Duration = Duration::from_secs(3);

pub fn gather_clock() -> Env<Limits> {
    let mut clock = env();
    clock.limits.client.backoff = GATHER_BACKOFF;
    clock.limits.client.read_attempts = 1;
    clock
}

pub fn wall_after(clock: &Env<Limits>, span: Duration) -> Wall {
    Wall::from_nanos(clock.wall.as_nanos().saturating_add(span.as_nanos()))
}

pub fn assert_gather_wait(out: &[Request], until: Wall) {
    use temper_engine_domain_forge_change::Decision;
    for request in out {
        if let Request::ChangeDecision { task: CHANGE, decision: Decision::Wait { until: Some(at) }, .. } = request
            && *at == until
        {
            return;
        }
    }
    panic!("expected a wait until {until:?}: {out:?}");
}

/// One terminal read, without the client's transient retries, for reader tables.
pub fn answer_once(
    d: &mut Domain,
    call: Token,
    result: Result<skein_forge::api::Answer, skein_forge::api::Error>,
) -> Box<[Request]> {
    let mut e = env();
    e.limits.client.read_attempts = 1;
    let mut out = Queue::uncapped();
    step(
        d,
        &e,
        Event::Client(skein_forge_client::Event::Answered { free: 0, rate: None, call, cost: 1, result }),
        &mut out,
    );
    d.reclaim();
    let mut requests = skein_lib::List::with_capacity(out.len());
    for _ in 0..out.len() {
        requests.push(out.pop().expect("counted request")).expect("counted output room");
    }
    requests.into_boxed()
}

pub fn next_call(d: &mut Domain) -> (Token, skein_forge::api::Op) {
    if !d.steps.is_empty() {
        landing_seen(d);
    }
    for request in ready(d) {
        if let Request::Call { call, op, .. } = request {
            return (call, op);
        }
    }
    panic!("one forge call")
}

pub fn answer(
    d: &mut Domain,
    call: Token,
    result: Result<skein_forge::api::Answer, skein_forge::api::Error>,
) -> Box<[Request]> {
    let answered =
        outputs(d, Event::Client(skein_forge_client::Event::Answered { free: 0, rate: None, call, cost: 1, result }));
    d.reclaim();
    answered
}

pub fn answer_next(
    d: &mut Domain,
    result: Result<skein_forge::api::Answer, skein_forge::api::Error>,
) -> Box<[Request]> {
    let (call, _) = next_call(d);
    answer(d, call, result)
}

pub fn step_change(d: &mut Domain) {
    outputs(d, change_step());
}

pub fn change_step() -> Event {
    use temper_engine_domain_forge_change as change;
    Event::StepChange {
        merge_job: false,
        task: CHANGE,
        entry: OPEN + 1,
        heard: change::Heard {
            delegate_running: false,
            flake: false,
            no_test: false,
            delegate_live: false,
            review_request: None,
            closer_waiting: false,
            start_conflict: None,
            lacking: None,
            spend: None,
            delegate: change::Status::Passed,
            effect: change::EffectResult::None,
            released: false,
            cancelled: false,
        },
        gates: Box::new([]),
        queue_repair_active: false,
        pending: Box::new([]),
        kept: false,
    }
}

pub fn missing_pull(d: &mut Domain) {
    step_change(d);
    missing_pull_reads(d);
}

pub fn missing_pull_reads(d: &mut Domain) {
    answer_next(d, Ok(skein_forge::api::Answer::Commit(HEAD)));
    answer_next(d, Ok(skein_forge::api::Answer::Commit(BASE)));
    answer_next(d, Err(skein_forge::api::Error::Missing));
}

pub fn finish_missing(d: &mut Domain) -> Box<[Request]> {
    let [head, base, ancestry] = missing_tail();
    answer_next(d, head);
    answer_next(d, base);
    answer_next(d, ancestry)
}

pub fn finish_missing_at(d: &mut Domain, clock: &Env<Limits>) -> Box<[Request]> {
    let [head, base, ancestry] = missing_tail();
    answer_next_at(d, clock, head);
    answer_next_at(d, clock, base);
    answer_next_at(d, clock, ancestry)
}

pub fn answer_next_at(
    d: &mut Domain,
    clock: &Env<Limits>,
    result: Result<skein_forge::api::Answer, skein_forge::api::Error>,
) -> Box<[Request]> {
    let (call, _) = next_at(d, clock);
    collected(answer_at(d, clock, call, result))
}

fn missing_tail() -> [Result<skein_forge::api::Answer, skein_forge::api::Error>; 3] {
    use skein_forge::api::{Answer, Check};
    let green = checks(Check::Passed);
    [Ok(green.clone()), Ok(green), Ok(Answer::Commits { commits: Box::new([]), more: false })]
}

/// `OpenPull` succeeds after the current chain has already read Missing.
pub fn opened_during_chain() -> Domain {
    let (mut d, opening) = opening();
    missing_pull(&mut d);
    answer(&mut d, opening, Ok(skein_forge::api::Answer::Created(1)));
    d
}

/// A reported CI row, shared by the fresh-read fixtures.
pub fn checks(check: skein_forge::api::Check) -> skein_forge::api::Answer {
    skein_forge::api::Answer::Checks(Box::new([skein_forge::api::Status {
        at: Time::ZERO,
        context: Box::from(b"ci".as_slice()),
        check,
        description: Box::new([]),
        url: Box::new([]),
        job: None,
    }]))
}

/// Finish the CI reads this gather asks for, then its ancestry range.
pub fn finish_open(d: &mut Domain) -> Box<[Request]> {
    use skein_forge::api::{Answer, Op, Read};
    for _ in 0_u32..3 {
        let (call, op) = next_call(d);
        if let Op::Read(Read::Checks { .. }) = op {
            answer(d, call, Ok(checks(skein_forge::api::Check::Passed)));
        } else {
            let Op::Read(Read::Commits { .. }) = op else { panic!("CI or ancestry: {op:?}") };
            return answer(d, call, Ok(Answer::Commits { commits: Box::new([]), more: false }));
        }
    }
    panic!("at most two commits' CI before ancestry")
}

/// Fresh change chains start with an observed landing branch, apart from their subject.
fn landing_seen(d: &mut Domain) {
    let name = Name {
        forge: REPO.forge,
        repository: REPO.repository,
        what: What::Branch(Box::new([b"main".as_slice().into()])),
    };
    if !d.heads.contains_key(&name) {
        d.heads.insert(name.clone(), BranchHead { name, commit: BASE, owned: false }).expect("fixture head room");
    }
}

/// Complete adoption's independent workflow read with absent directories.
pub fn workflows_absent(d: &mut Domain) {
    use skein_forge::api::{Answer, Error, Op, Read};
    let (call, op) = next_call(d);
    assert_eq!(op, Op::Read(Read::Branch { branch: b"main".as_slice().into() }));
    answer(d, call, Ok(Answer::Commit(BASE)));
    for path in [b".forgejo/workflows".as_slice(), b".github/workflows"] {
        let (call, op) = next_call(d);
        assert_eq!(op, Op::Read(Read::File { head: BASE, path: path.into(), max_bytes: LIMITS.client.answer_bytes }));
        answer(d, call, Err(Error::Missing));
    }
}

/// A veto or deferral gives back the unsent entry's room in every branch record.
pub fn assert_write_withdrawn(d: &Domain, task: u64, entry: u64) {
    let sender = Sender::Entry { task, entry };
    for (_, record) in &d.writes {
        for write in &record.entries {
            assert_ne!(write.sender, sender, "an unsent withdrawal frees its branch write record");
        }
    }
}

/// A job's first failed step, including the runner's set-up and completion.
pub fn failed_run(attempt: skein_forge::api::JobAttempt, step: Option<u32>) -> skein_forge::api::Run {
    use skein_forge::api::{Conclusion, Run, RunJob, RunState, Step};
    Run {
        id: attempt.run,
        number: attempt.run,
        workflow: b"ci".as_slice().into(),
        commit: attempt.head,
        event: skein_forge::api::Event::Other,
        actor: 7,
        state: RunState::Completed,
        conclusion: Some(Conclusion::Failure),
        url: Box::new([]),
        created: Time::ZERO,
        jobs: Box::new([RunJob {
            id: attempt.job,
            name: b"test".as_slice().into(),
            attempt: attempt.attempt,
            state: RunState::Completed,
            conclusion: Some(Conclusion::Failure),
            steps: 4,
            step: match step {
                Some(number) => Some(Step { number, name: b"step".as_slice().into() }),
                None => None,
            },
        }]),
    }
}
