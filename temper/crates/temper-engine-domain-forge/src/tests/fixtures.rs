//! Shared adoption and change fixtures.
use super::*;

/// Change reads start after routing has reported access.
pub fn connected_domain() -> Domain {
    let mut d = domain();
    connect(&mut d);
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
    }
}

pub fn opening_row() -> ChangeRow {
    use temper_engine_domain_forge_change as change;
    ChangeRow {
        task: CHANGE,
        repository: REPO,
        branch: Box::from(b"temper/51".as_slice()),
        base: Box::from(b"main".as_slice()),
        title: Box::from(b"Small fix".as_slice()),
        body: Box::new([]),
        priority: 1,
        ci: None,
        change: change::Change {
            early: change::Early::None,
            way: temper_engine_domain_forge_change::Way::Queue,
            landing: None,
            task: CHANGE,
            state: change::State::Opening { requested: true },
            gates: Box::new([]),
            clean: Box::new([]),
            gate_failures: Box::new([]),
            released: None,
            repairs: 0,
            resolutions: 0,
            updates: 0,
            since: env().wall,
            ready_since: None,
            owns_turn: false,
            last_head: Some(HEAD),
        },
        pull: None,
        pending: Some(OPEN),
        effect: change::EffectResult::Pending,
        delegate: None,
        delegate_status: change::Status::Passed,
        verdicts: Box::new([]),
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
        foreman_charter: 6,
        questions: temper_engine_domain_forge_change::Questions::default(),
        effect_account: None,
        producer_failure: None,
        report: Box::new([]),
        repairs: None,
        catch_up: None,
        close: None,
        merge_result: None,
    }
}

pub fn opening() -> (Domain, Token) {
    let mut d = connected_domain();
    outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
    outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    outputs(&mut d, Event::ReadAfresh);
    outputs(&mut d, Event::SettleOutbox);
    let row = opening_row();
    let entry = skein_forge_client::Entry {
        number: OPEN,
        task: CHANGE,
        repository: REPO,
        effect: skein_forge_client::Effect {
            write: skein_forge::api::Write::OpenPull {
                title: row.title.clone(),
                body: row.body.clone(),
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
            delegate_live: false,
            review_request: None,
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
    let green = checks(skein_forge::api::Check::Passed);
    answer_next(d, Ok(green.clone()));
    answer_next(d, Ok(green));
    answer_next(d, Ok(skein_forge::api::Answer::Commits { commits: Box::new([]), more: false }))
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
