//! A goal's mechanical close, landing and report (temper/docs/design/domain/forge/branches.md, 2).
//! Landing targets use the adoption (temper/docs/design/domain/forge/adoption.md, 12).
use super::{Box, Decision, Domain, Env, Limits, List, ProcedureStep, StepEnd, Work, append, forge, tasks};
use jig_domain_core::StepCancel;
use temper_engine_domain_forge_closer as c;
pub(crate) mod goal_report;
mod relay;
#[derive(Hash, Debug)]
pub(crate) struct Pending {
    relay: c::relay::Relay,
    asked: bool,
    more: bool,
    replacement: Option<Replacement>,
}

#[derive(Hash, Debug)]
struct Replacement {
    change: u64,
    landed: Box<[u64]>,
}

pub(crate) fn resume_closer_step(domain: &mut Domain, env: &Env<Limits>, journal: &mut Decision, task: u64) {
    if let Some(context) = domain.core.tasks.run_context(task) {
        start_closer_step(domain, env, journal, &context);
    }
}

#[expect(clippy::too_many_lines, reason = "the procedure gathers its goal and submits one exhaustive step decision")]
pub(crate) fn start_closer_step(
    domain: &mut Domain,
    env: &Env<Limits>,
    journal: &mut Decision,
    context: &tasks::RunContext,
) {
    if domain.core.settling() {
        return;
    }
    let task = context.task;
    let Some((_, _, procedure_step)) = domain.core.tasks.procedure_due(task) else { return };
    let goal = match context.requester {
        tasks::Party::Task(goal) => goal,
        tasks::Party::Person(_) | tasks::Party::Deployment { .. } => {
            super::holds::hold(domain, task, b"the goal closer requires its goal as requester");
            return;
        }
    };
    let Some(goal_record) = domain.core.tasks.task(goal) else { return };
    let goal_words = goal_record.spec.words.clone();
    let bound = match goal_record.contract {
        tasks::Contract::Report { words } => words,
        tasks::Contract::Job { .. }
        | tasks::Contract::Approval { .. }
        | tasks::Contract::Verdict { .. }
        | tasks::Contract::Change { .. } => {
            super::holds::hold(domain, task, b"the goal closer requires its goal's report contract");
            return;
        }
    };
    let Some(repository) = domain.forge.home(context.project).cloned() else {
        super::holds::hold(domain, task, b"the goal closer requires the project's adopted home");
        return;
    };
    let mut row = domain.forge.closer(task).cloned().unwrap_or(forge::CloserRow {
        task,
        goal,
        closer: c::Closer::new(),
        making: None,
        close_changes: Box::new([]),
        superseded: Box::new([]),
        landing_changes: Box::new([]),
        relay: c::relay::Relay::new(),
        done: false,
    });
    row.done = false;
    // Its admission answer owns the pending change. A wake before that answer
    // must not delegate the same close or landing step again.
    if domain.closer_steps.get(&(task, procedure_step)).is_some() {
        return;
    }
    if row.making.is_some() {
        submit(domain, env, row, ProcedureStep::new(StepEnd::Wait), None);
        return;
    }
    if let Some(words) = domain.forge.closer_stall_words(task) {
        let words = forge::text::fit(words, skein_lib::usize_of(tasks::HOLD_ACCOUNT_BYTES), b"...");
        let decision = ProcedureStep::new(StepEnd::HoldFrom {
            why: tasks::Hold::Procedure,
            source: Box::new(super::holds::source(domain, None, words)),
        });
        submit(domain, env, row, decision, None);
        return;
    }
    let branch = match goal_branch(domain, env, goal, &repository) {
        Some((branch, name)) => {
            if domain.forge.goal_branch_waiting(&name) {
                let wait = if live_closer_delegate(domain, &row) {
                    forge::CloserPark::DelegateOrHeld
                } else {
                    forge::CloserPark::Making(name)
                };
                submit(domain, env, row, ProcedureStep::new(StepEnd::Wait), Some(wait));
                return;
            }
            Some(branch)
        }
        None => None,
    };
    let base_holds = match domain.forge.closer_checked(task, procedure_step) {
        Some(Ok(contains)) => Some(contains),
        Some(Err(_)) => {
            submit(domain, env, row, ProcedureStep::new(StepEnd::Wait), None);
            return;
        }
        None => None,
    };
    let dependencies = domain.core.tasks.task(task).expect("due closer").dependencies.len();
    let at = goal_report::At { step: procedure_step, dependencies };
    let Some(observation) = goal_report::observed(domain, env, task, at) else { return };
    let facts = facts(domain, &row, branch.as_deref(), base_holds, observation);
    let stepped = c::step(&row.closer, &facts);
    let mut replacement = None;
    let (decision, wait) = match stepped.decision {
        c::Decision::ReadReport => {
            // If new work invalidated the note observation, recheck its tip too.
            domain.forge.discard_closer_answer(task);
            if row.closer != stepped.closer {
                row.closer = stepped.closer;
                domain.work.push(Work::Forge(forge::Event::Closer { row }));
            }
            let scope = jig_domain_notes::Scope::Goal { project: context.project, goal };
            goal_report::start(domain, env, journal, task, scope, at);
            return;
        }
        c::Decision::HoldReport => {
            row.closer = stepped.closer;
            let decision = super::holds::decision(domain, env, task, report_hold_words(&row.closer));
            (decision, Some(forge::CloserPark::DelegateOrHeld))
        }
        c::Decision::Check => {
            row.closer = stepped.closer;
            domain.work.push(Work::Forge(forge::Event::Closer { row }));
            domain.work.push(Work::Forge(forge::Event::CheckCloser {
                task,
                step: procedure_step,
                repository: repository.provider,
                branch: branch.expect("only a held goal branch asks containment"),
                base: repository.landing.clone(),
            }));
            return;
        }
        making @ (c::Decision::Make(_) | c::Decision::Supersede { .. }) => {
            let (kind, supersedes) = match making {
                c::Decision::Make(kind) => (kind, None),
                c::Decision::Supersede { change } => (c::Step::Close, Some(change)),
                c::Decision::ReadReport
                | c::Decision::HoldReport
                | c::Decision::Check
                | c::Decision::Wake { .. }
                | c::Decision::Wait
                | c::Decision::Hold(_)
                | c::Decision::Report => {
                    unreachable!("only a new or superseded step makes a change")
                }
            };
            if kind == c::Step::Land && domain.forge.landing_held() {
                submit(domain, env, row, ProcedureStep::new(StepEnd::Wait), Some(forge::CloserPark::OperatorHold));
                return;
            }
            let branch = branch.expect("only a held goal branch makes a step");
            let draft = delegate(domain, env, &row, &repository, &branch, &goal_words, kind);
            let mut next = stepped.closer;
            if supersedes.is_some() {
                // Keep the old attempt until cancellation and replacement are admitted together.
                next.running = row.closer.running;
            }
            row.closer = next;
            let delegate = match draft {
                Ok(delegate) => delegate,
                Err(short) => {
                    if supersedes.is_none() {
                        row.closer.retry = Some(kind);
                    }
                    let decision = if short.is_spend() && goal_run_live(domain, goal) {
                        ProcedureStep::new(StepEnd::Wait)
                    } else {
                        super::holds::decision(domain, env, task, short_words(short))
                    };
                    submit(domain, env, row, decision, Some(forge::CloserPark::DelegateOrHeld));
                    return;
                }
            };
            let mut decision = ProcedureStep::new(StepEnd::Wait);
            decision.batch = Box::new([delegate]);
            if let Some(change) = supersedes {
                decision.cancels = Box::new([supersede(env, change, &row.closer, &facts.landed)]);
                replacement = Some(Replacement { change, landed: facts.landed });
            } else {
                row.making = Some(kind);
                if kind == c::Step::Close {
                    row.closer.closed = Some(facts.landed);
                }
            }
            (decision, None)
        }
        c::Decision::Wake { change } => {
            row.closer = stepped.closer;
            domain.work.push(Work::Tasks(tasks::Event::WakeProcedure { task: change }));
            (ProcedureStep::new(StepEnd::Wait), Some(forge::CloserPark::DelegateOrHeld))
        }
        c::Decision::Hold(hold) => {
            row.closer = stepped.closer;
            let decision = ProcedureStep::new(StepEnd::HoldFrom {
                why: tasks::Hold::Procedure,
                source: Box::new(super::holds::source(domain, None, hold_words(domain, &row, hold))),
            });
            (decision, Some(forge::CloserPark::DelegateOrHeld))
        }
        c::Decision::Wait => {
            row.closer = stepped.closer;
            // Parking an idle closer clears its wake; keep the report retry.
            let wait = match facts.report {
                c::ReportRead::Failed => None,
                c::ReportRead::Unread | c::ReportRead::Read(_) => Some(forge::CloserPark::DelegateOrHeld),
            };
            (ProcedureStep::new(StepEnd::Wait), wait)
        }
        c::Decision::Report => {
            row.closer = stepped.closer;
            let mut report = report(domain, &row, &goal_words, branch.as_deref());
            prepare_selections(domain, &mut row, &report);
            if c::selection_read(&row.closer).is_some() {
                domain.work.push(Work::Forge(forge::Event::Closer { row }));
                domain
                    .work
                    .push(Work::Forge(forge::Event::ReadCloserSelection { task, repository: repository.provider }));
                return;
            }
            report.selections = settled_selections(&row.closer);
            row.done = true;
            (ProcedureStep::new(StepEnd::Result(tasks::TaskResult::Report { words: c::words(&report, bound) })), None)
        }
    };
    submit_replacing(domain, env, row, decision, wait, replacement);
}

fn live_closer_delegate(domain: &Domain, row: &forge::CloserRow) -> bool {
    if let Some(running) = row.closer.running {
        match child(domain, running.change, running.step) {
            c::Child::Pending | c::Child::Idle { .. } => return true,
            c::Child::Done | c::Child::Ended { .. } => {}
        }
    }
    if let Some(task) = row.closer.landing {
        return child(domain, task, c::Step::Land) == c::Child::Pending;
    }
    false
}

fn supersede(env: &Env<Limits>, change: u64, closer: &c::Closer, landed: &[u64]) -> StepCancel {
    let previous = closer.closed.as_deref().unwrap_or(&[]);
    let mut reason = List::uncapped();
    append(&mut reason, b"superseded by the goal's new work: changes ");
    let mut comma = false;
    for number in landed {
        if previous.contains(number) {
            continue;
        }
        if comma {
            append(&mut reason, b", ");
        }
        comma = true;
        append(&mut reason, &super::decimal(*number));
    }
    append(&mut reason, b" landed into its branch");
    let reason = forge::text::fit(reason.as_slice(), skein_lib::usize_of(env.limits.tasks.result_bytes), b"...");
    StepCancel { task: change, reason, grace: None }
}

fn facts(
    domain: &Domain,
    row: &forge::CloserRow,
    branch: Option<&[u8]>,
    base_holds: Option<bool>,
    report: c::ReportRead,
) -> c::Facts {
    let mut landed = List::uncapped();
    for number in domain.forge.change_tasks() {
        let change = domain.forge.change(number).expect("enumerated change");
        if original(domain, row, change)
            && branch == Some(change.base.as_ref())
            && ending(change) == Some(forge::Ending::Landed)
        {
            landed.push(number).expect("goal tree bounds landed changes");
        }
    }
    let pending = !domain.core.tasks.task(row.task).expect("live closer").waiting_on.is_empty();
    let running = match row.closer.running {
        Some(run) => child(domain, run.change, run.step),
        None => c::Child::Done,
    };
    let landing_child = match row.closer.landing {
        Some(task) => child(domain, task, c::Step::Land),
        None => c::Child::Done,
    };
    c::Facts {
        report,
        goal_branch: branch.is_some(),
        close_job: domain.forge.close_job(domain.core.tasks.task(row.task).expect("live closer").project).is_some(),
        pending,
        landed: landed.into_boxed(),
        base_holds,
        child: running,
        close_ended: ended_count(domain, row, c::Step::Close),
        land_ended: ended_count(domain, row, c::Step::Land),
        bound: attempt_bound(domain, row.task),
        landing_child,
        released: domain.core.tasks.task(row.task).expect("live closer").due_reason == tasks::Why::Released,
    }
}

fn submit(
    domain: &mut Domain,
    env: &Env<Limits>,
    row: forge::CloserRow,
    decision: ProcedureStep,
    wait: Option<forge::CloserPark>,
) {
    submit_replacing(domain, env, row, decision, wait, None);
}

fn submit_replacing(
    domain: &mut Domain,
    env: &Env<Limits>,
    mut row: forge::CloserRow,
    mut decision: ProcedureStep,
    wait: Option<forge::CloserPark>,
    replacement: Option<Replacement>,
) {
    let task = row.task;
    let Some((connector, code, step)) = domain.core.tasks.procedure_due(task) else { return };
    goal_report::submitted(domain, task);
    let previous = row.relay.clone();
    let (asked, more) = relay::apply(domain, env, &mut row, &mut decision);
    let relay = core::mem::replace(&mut row.relay, previous);
    // Step progress keeps its creation marker; only the relay waits for admission.
    if domain.forge.closer(task) != Some(&row) {
        domain.work.push(Work::Forge(forge::Event::Closer { row }));
    }
    if let Some(wait) = wait {
        domain.work.push(Work::Forge(forge::Event::ParkCloser { task, wait }));
    }
    domain
        .closer_steps
        .insert((task, step), Pending { relay, asked, more, replacement })
        .expect("one pending step per closer");
    domain.work.push(Work::Core(jig_domain_core::Event::ProcedureStep { task, step, connector, code, decision }));
}

fn goal_branch(
    domain: &Domain,
    env: &Env<Limits>,
    goal: u64,
    repository: &forge::Repository,
) -> Option<(Box<[u8]>, forge::Name)> {
    let root = domain.core.tasks.root(goal)?;
    let branch = super::tree_branch(repository, root, b'g', goal, None, env.limits.forge.name_bytes)?;
    let name = forge::Name {
        forge: repository.provider.forge,
        repository: repository.provider.repository,
        what: super::branch_what(&branch, env.limits.forge.name_bytes)?,
    };
    if domain.forge.hold(&name)?.task == goal { Some((branch, name)) } else { None }
}

fn original(domain: &Domain, closer: &forge::CloserRow, row: &forge::ChangeRow) -> bool {
    if closer.close_changes.contains(&row.task)
        || closer.superseded.contains(&row.task)
        || closer.landing_changes.contains(&row.task)
    {
        return false;
    }
    match row.goal {
        Some(goal) => goal == closer.goal,
        None => super::tracked_ancestor(domain, tasks::Party::Task(row.task)) == Some(closer.goal),
    }
}

fn ending(row: &forge::ChangeRow) -> Option<forge::Ending> {
    if let Some(ending) = &row.ending {
        return Some(ending.clone());
    }
    match &row.change.state {
        super::forge_change::State::Landed { .. }
        | super::forge_change::State::Red { .. }
        | super::forge_change::State::Cleared { .. }
        | super::forge_change::State::Uncleared { .. } => Some(forge::Ending::Landed),
        super::forge_change::State::NoChange { .. } => Some(forge::Ending::NoChange),
        super::forge_change::State::Ended => Some(forge::Ending::Failed {
            line: c::first_line(row.producer_failure.as_deref().unwrap_or(b"change ended without an ending account"))
                .into(),
        }),
        super::forge_change::State::Taking { .. }
        | super::forge_change::State::Producing { .. }
        | super::forge_change::State::Opening { .. }
        | super::forge_change::State::Recreating { .. }
        | super::forge_change::State::Reopening { .. }
        | super::forge_change::State::Checking { .. }
        | super::forge_change::State::Judging { .. }
        | super::forge_change::State::Gating { .. }
        | super::forge_change::State::CancellingGates { .. }
        | super::forge_change::State::Ready { .. }
        | super::forge_change::State::Preparing { .. }
        | super::forge_change::State::Queued { .. }
        | super::forge_change::State::First { .. }
        | super::forge_change::State::Updating { .. }
        | super::forge_change::State::WaitingProducer { .. }
        | super::forge_change::State::Resolving { .. }
        | super::forge_change::State::Repairing { .. }
        | super::forge_change::State::Landing { .. }
        | super::forge_change::State::Held { .. } => None,
    }
}
fn child(domain: &Domain, task: u64, step: c::Step) -> c::Child {
    if let Some(row) = domain.core.tasks.task(task) {
        match &row.phase {
            tasks::Phase::Ended(_) => {}
            tasks::Phase::Waiting | tasks::Phase::Active(_) | tasks::Phase::Held { .. } if step == c::Step::Close => {
                return close_child(domain, row);
            }
            tasks::Phase::Waiting | tasks::Phase::Active(_) | tasks::Phase::Closing(_) | tasks::Phase::Held { .. } => {
                return c::Child::Pending;
            }
        }
    }
    match domain.forge.change(task) {
        Some(row) => match ending(row) {
            Some(forge::Ending::Landed | forge::Ending::NoChange) => c::Child::Done,
            Some(forge::Ending::Cancelled | forge::Ending::Failed { .. }) => {
                c::Child::Ended { branch: branch_reason(row.change.ended_held).is_some() }
            }
            None => c::Child::Pending,
        },
        None => c::Child::Pending,
    }
}

fn close_child(domain: &Domain, task: &tasks::TaskRecord) -> c::Child {
    use super::forge_change::{Hold, State};
    let Some(row) = domain.forge.change(task.number) else { return c::Child::Pending };
    if row.close.is_none() {
        return c::Child::Pending;
    }
    match &row.change.state {
        State::Held { was, why: Hold::Failed } if **was == (State::Producing { requested: true }) => {
            c::Child::Idle { held: true }
        }
        State::Producing { requested: false }
            if task.phase == tasks::Phase::Active(tasks::Active::Idle) && !live_delegate(domain, row) =>
        {
            c::Child::Idle { held: false }
        }
        State::Taking { .. }
        | State::Producing { .. }
        | State::Opening { .. }
        | State::Recreating { .. }
        | State::Reopening { .. }
        | State::Checking { .. }
        | State::Judging { .. }
        | State::Gating { .. }
        | State::CancellingGates { .. }
        | State::Ready { .. }
        | State::Preparing { .. }
        | State::Queued { .. }
        | State::First { .. }
        | State::Updating { .. }
        | State::WaitingProducer { .. }
        | State::Resolving { .. }
        | State::Repairing { .. }
        | State::Landing { .. }
        | State::Held { .. }
        | State::Landed { .. }
        | State::Red { .. }
        | State::Cleared { .. }
        | State::Uncleared { .. }
        | State::NoChange { .. }
        | State::Ended => c::Child::Pending,
    }
}

fn live_delegate(domain: &Domain, row: &forge::ChangeRow) -> bool {
    let Some((child, _)) = row.delegate else { return false };
    let Some(record) = domain.core.tasks.task(child) else { return false };
    match &record.phase {
        tasks::Phase::Ended(_) => false,
        tasks::Phase::Waiting | tasks::Phase::Active(_) | tasks::Phase::Closing(_) | tasks::Phase::Held { .. } => true,
    }
}

fn attempt_bound(domain: &Domain, task: u64) -> u8 {
    let project = domain.core.tasks.task(task).expect("live closer").project;
    match domain.forge.landing_settings(project) {
        Some(landing) => landing.landing_attempts,
        None => crate::LANDING_ATTEMPTS,
    }
}

fn ended_count(domain: &Domain, row: &forge::CloserRow, kind: c::Step) -> u32 {
    let changes = match kind {
        c::Step::Close => &row.close_changes,
        c::Step::Land => &row.landing_changes,
    };
    let mut count = 0_u32;
    for task in changes {
        if kind == c::Step::Close && row.superseded.contains(task) {
            continue;
        }
        if let Some(row) = domain.forge.change(*task) {
            match ending(row) {
                Some(forge::Ending::Cancelled | forge::Ending::Failed { .. }) => count = count.saturating_add(1),
                Some(forge::Ending::Landed | forge::Ending::NoChange) | None => {}
            }
        }
    }
    count
}

fn branch_reason(held: Option<super::forge_change::Hold>) -> Option<&'static [u8]> {
    use super::forge_change::Hold;
    match held {
        Some(Hold::Repairs) => Some(b"repairs"),
        Some(Hold::Resolutions) => Some(b"conflict resolutions"),
        Some(Hold::Rejected) => Some(b"a rejection"),
        Some(Hold::PullClosed) => Some(b"its pull request closed"),
        Some(Hold::BranchMissing) => Some(b"its branch missing"),
        Some(
            Hold::Stalled
            | Hold::Updates
            | Hold::BranchMoved
            | Hold::Retargeted
            | Hold::Failed
            | Hold::Flake
            | Hold::NoTest
            | Hold::Authority(_)
            | Hold::Questions,
        )
        | None => None,
    }
}

fn close_request(domain: &Domain, closer: &forge::CloserRow, branch: &[u8]) -> Box<[u8]> {
    let mut words = List::uncapped();
    append(&mut words, b"{\"version\":1,\"goal\":");
    append(&mut words, &super::decimal(closer.goal));
    append(&mut words, b",\"changes\":[");
    let mut comma = false;
    for number in domain.forge.change_tasks() {
        let row = domain.forge.change(number).expect("enumerated change");
        if !original(domain, closer, row) || row.base.as_ref() != branch {
            continue;
        }
        let Some(ended) = ending(row) else { continue };
        if comma {
            append(&mut words, b",");
        }
        comma = true;
        append(&mut words, b"{\"task\":");
        append(&mut words, &super::decimal(number));
        append(&mut words, b",\"landed\":");
        append(&mut words, if ended == forge::Ending::Landed { b"true" } else { b"false" });
        append(&mut words, b"}");
    }
    append(&mut words, b"]}");
    words.into_boxed()
}

fn landing_words(words: &[u8]) -> Box<[u8]> {
    let mut result = List::uncapped();
    append(&mut result, c::first_line(words));
    let mut offset = 0_usize;
    for _ in 0..=words.len() {
        let rest = words.get(offset..).expect("line offset within goal words");
        let end = skein_lib::bytes::find(rest, b"\n").unwrap_or(rest.len());
        let line = rest.get(..end).expect("line within goal words");
        if line.starts_with(b"Draft: ") {
            append(&mut result, b"\n\n");
            append(&mut result, line.strip_suffix(b"\r").unwrap_or(line));
            break;
        }
        if end == rest.len() {
            break;
        }
        offset =
            offset.checked_add(end).expect("line within goal words").checked_add(1).expect("newline within goal words");
    }
    result.into_boxed()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Short {
    Spend { left: u64, minimum: u64 },
    Tasks { left: u32 },
    Depth,
    Terms,
}

impl Short {
    fn is_spend(self) -> bool {
        match self {
            Self::Spend { .. } => true,
            Self::Tasks { .. } | Self::Depth | Self::Terms => false,
        }
    }
}

fn goal_run_live(domain: &Domain, goal: u64) -> bool {
    match domain.core.tasks.task(goal).expect("live goal").phase {
        tasks::Phase::Active(tasks::Active::Claimed { .. } | tasks::Active::Running { .. }) => true,
        tasks::Phase::Active(
            tasks::Active::Idle | tasks::Active::Due | tasks::Active::Preparing | tasks::Active::BackingOff { .. },
        )
        | tasks::Phase::Waiting
        | tasks::Phase::Closing(_)
        | tasks::Phase::Held { .. }
        | tasks::Phase::Ended(_) => false,
    }
}

fn short_refusal(why: tasks::Refusal, left: u64, minimum: u64, room: u32) -> Short {
    use tasks::Refusal;
    match why {
        Refusal::Funding => Short::Spend { left, minimum },
        Refusal::Tree => Short::Tasks { left: room },
        Refusal::Depth => Short::Depth,
        Refusal::NotStanding
        | Refusal::TalkOpen { .. }
        | Refusal::NoTalk
        | Refusal::Placed { .. }
        | Refusal::HoldOwner { .. }
        | Refusal::HoldRevision { .. }
        | Refusal::Account { .. }
        | Refusal::Grace
        | Refusal::Goals
        | Refusal::Unproposable
        | Refusal::PersonRevision { .. }
        | Refusal::ResourceUnavailable
        | Refusal::Unreported
        | Refusal::NotReady
        | Refusal::Unknown
        | Refusal::Busy
        | Refusal::Duplicate
        | Refusal::Empty
        | Refusal::Batch
        | Refusal::Live
        | Refusal::Project
        | Refusal::Delegates
        | Refusal::Reference
        | Refusal::Subscription
        | Refusal::Dependencies
        | Refusal::Circle { .. }
        | Refusal::DependencyEnded { .. }
        | Refusal::Kept
        | Refusal::Concluded { .. }
        | Refusal::KeptQuestion { .. }
        | Refusal::PersonAnswers { .. }
        | Refusal::QuestionAnswered { .. }
        | Refusal::Executor
        | Refusal::Spec
        | Refusal::Contract
        | Refusal::AuthorityShape
        | Refusal::Inputs
        | Refusal::State
        | Refusal::Attempt
        | Refusal::LiveDelegates
        | Refusal::Restore
        | Refusal::Read
        | Refusal::Turn
        | Refusal::HoldKind
        | Refusal::HoldTaken
        | Refusal::Holds
        | Refusal::Llm { .. } => Short::Terms,
    }
}

fn short_words(short: Short) -> Box<[u8]> {
    let mut words = List::uncapped();
    append(&mut words, b"the goal closer cannot delegate its next change within its authority: ");
    match short {
        Short::Spend { left, minimum } => {
            append(&mut words, b"spend: ");
            append(&mut words, &super::decimal(left));
            append(&mut words, b" left, its change's half must pass ");
            append(&mut words, &super::decimal(minimum));
        }
        Short::Tasks { left } => {
            append(&mut words, b"tasks: ");
            append(&mut words, &super::decimal(u64::from(left)));
            append(&mut words, b" left, its change needs 2");
        }
        Short::Depth => append(&mut words, b"depth"),
        Short::Terms => append(&mut words, b"its change's terms"),
    }
    words.into_boxed()
}

#[cfg(test)]
mod tests;

fn change_tasks(given: u32, room: u32, bound: u8, ended: u32) -> u32 {
    let attempts = u32::from(bound).saturating_sub(ended).max(1);
    let share = room.checked_div(attempts.saturating_add(1)).expect("at least two shares").saturating_sub(1);
    given.min(share).max(1)
}

fn delegate(
    domain: &Domain,
    env: &Env<Limits>,
    row: &forge::CloserRow,
    repository: &forge::Repository,
    branch: &[u8],
    goal_words: &[u8],
    kind: c::Step,
) -> Result<super::super::Delegate, Short> {
    let context = domain.core.tasks.delegation(row.task).ok_or(Short::Terms)?;
    let goal = domain.core.tasks.delegation(row.goal).ok_or(Short::Terms)?;
    let room = context.tasks_left.min(goal.tasks_left);
    if room < 2 {
        return Err(Short::Tasks { left: room });
    }
    if context.authority.delegation.depth < 2 {
        return Err(Short::Depth);
    }
    let available = context
        .numbers
        .budget
        .saturating_sub(context.numbers.spent)
        .saturating_sub(context.numbers.spent_below)
        .saturating_sub(context.numbers.reserved);
    let spend = available.checked_div(2).expect("nonzero divisor");
    let minimum = domain.core.authority.rules().minimum_run_spend;
    let (words, base, own) = match kind {
        c::Step::Close => {
            let mut words = List::uncapped();
            append(&mut words, b"Settle goal ");
            append(&mut words, &super::decimal(row.goal));
            append(&mut words, b"'s status.");
            (words.into_boxed(), branch.into(), None)
        }
        c::Step::Land => (landing_words(goal_words), repository.landing.clone(), Some(branch.into())),
    };
    let mut change = crate::ChangeCall::words(words, None);
    change.questions = Some(crate::ChangeQuestions::Up);
    let origin =
        super::ChangeOrigin { authority: context.authority.clone(), tasks_left: context.tasks_left, source: None };
    let mut delegate = match super::chat_change(domain, env, &origin, &change, repository, base, own, spend) {
        Ok(delegate) => delegate,
        Err(why) => return Err(short_refusal(why, available, minimum, room)),
    };
    let ended = ended_count(domain, row, kind);
    delegate.authority.delegation.tasks =
        change_tasks(delegate.authority.delegation.tasks, room, attempt_bound(domain, row.task), ended);
    let mut kinds = List::uncapped();
    for kind in &delegate.authority.delegation.kinds {
        kinds.push(*kind).expect("uncapped executor list");
    }
    if context.authority.delegation.kinds.contains(&tasks::AuthorityExecutor::Job(forge::CLOSE_KIND)) {
        kinds.push(tasks::AuthorityExecutor::Job(forge::CLOSE_KIND)).expect("uncapped executor list");
    }
    delegate.authority.delegation.kinds = kinds.into_boxed();
    if kind == c::Step::Close {
        let mut expanded = List::uncapped();
        for parameter in delegate.spec.parameters {
            expanded.push(parameter).expect("uncapped parameter list");
        }
        expanded
            .push(tasks::Parameter::Bytes { name: 17, value: close_request(domain, row, branch) })
            .expect("uncapped parameter list");
        delegate.spec.parameters = expanded.into_boxed();
    }
    Ok(delegate)
}

fn why(row: &forge::ChangeRow, ending: forge::Ending) -> c::Why {
    match ending {
        forge::Ending::Cancelled => c::Why::Cancelled,
        forge::Ending::Failed { line } => c::Why::Failed(line),
        forge::Ending::NoChange => {
            let words = match &row.change.state {
                super::forge_change::State::NoChange { words } => words.as_ref(),
                super::forge_change::State::Taking { .. }
                | super::forge_change::State::Producing { .. }
                | super::forge_change::State::Opening { .. }
                | super::forge_change::State::Recreating { .. }
                | super::forge_change::State::Reopening { .. }
                | super::forge_change::State::Checking { .. }
                | super::forge_change::State::Judging { .. }
                | super::forge_change::State::Gating { .. }
                | super::forge_change::State::CancellingGates { .. }
                | super::forge_change::State::Ready { .. }
                | super::forge_change::State::Preparing { .. }
                | super::forge_change::State::Queued { .. }
                | super::forge_change::State::First { .. }
                | super::forge_change::State::Updating { .. }
                | super::forge_change::State::WaitingProducer { .. }
                | super::forge_change::State::Resolving { .. }
                | super::forge_change::State::Repairing { .. }
                | super::forge_change::State::Landing { .. }
                | super::forge_change::State::Held { .. }
                | super::forge_change::State::Landed { .. }
                | super::forge_change::State::Red { .. }
                | super::forge_change::State::Cleared { .. }
                | super::forge_change::State::Uncleared { .. }
                | super::forge_change::State::Ended => row.report.as_ref(),
            };
            c::Why::NoChange(c::first_line(words).into())
        }
        forge::Ending::Landed => unreachable!("landed changes have no unlanded reason"),
    }
}
fn report(domain: &Domain, closer: &forge::CloserRow, words: &[u8], branch: Option<&[u8]>) -> c::Report {
    let mut landed = List::uncapped();
    let mut unlanded = List::uncapped();
    let mut selections = List::uncapped();
    for number in domain.forge.change_tasks() {
        if closer.superseded.contains(&number) {
            continue;
        }
        let row = domain.forge.change(number).expect("enumerated change");
        let is_original = original(domain, closer, row);
        let close = closer.close_changes.contains(&number);
        let landing = closer.landing_changes.contains(&number);
        if !is_original && !close && !landing {
            continue;
        }
        let Some(ended) = ending(row) else { continue };
        let title = if row.title.is_empty() { c::first_line(&row.body) } else { row.title.as_ref() };
        if ended == forge::Ending::Landed {
            let Some(pull) = row.pull else { continue };
            if !landing {
                let left = match c::left_section(&row.report) {
                    Some(left) => Some(left.into()),
                    None => None,
                };
                landed
                    .push(c::Landed { task: number, title: title.into(), pull, branch: row.base.clone(), left })
                    .expect("goal tree bounds report");
            }
            if let Some(repository) = domain.forge.repository(row.repository)
                && row.base == repository.landing
            {
                selections.push(c::Selection { pull, line: None }).expect("goal tree bounds selections");
            }
        } else if !landing {
            unlanded
                .push(c::Unlanded { task: number, title: title.into(), why: why(row, ended) })
                .expect("goal tree bounds report");
        }
    }
    let landing = match closer.closer.landing {
        Some(task) => match domain.forge.change(task) {
            Some(row) => match &row.change.state {
                super::forge_change::State::Landed { merge, .. }
                | super::forge_change::State::Red { merge, .. }
                | super::forge_change::State::Cleared { merge }
                | super::forge_change::State::Uncleared { merge } => c::Landing::Landed {
                    pull: row.pull.expect("a landed change has its pull"),
                    base: row.base.clone(),
                    commit: crate::commit_text(merge),
                    repairs: row.repair_tasks.clone(),
                    resolutions: row.resolution_tasks.clone(),
                },
                super::forge_change::State::Taking { .. }
                | super::forge_change::State::Producing { .. }
                | super::forge_change::State::Opening { .. }
                | super::forge_change::State::Recreating { .. }
                | super::forge_change::State::Reopening { .. }
                | super::forge_change::State::Checking { .. }
                | super::forge_change::State::Judging { .. }
                | super::forge_change::State::Gating { .. }
                | super::forge_change::State::CancellingGates { .. }
                | super::forge_change::State::Ready { .. }
                | super::forge_change::State::Preparing { .. }
                | super::forge_change::State::Queued { .. }
                | super::forge_change::State::First { .. }
                | super::forge_change::State::Updating { .. }
                | super::forge_change::State::WaitingProducer { .. }
                | super::forge_change::State::Resolving { .. }
                | super::forge_change::State::Repairing { .. }
                | super::forge_change::State::Landing { .. }
                | super::forge_change::State::Held { .. }
                | super::forge_change::State::NoChange { .. }
                | super::forge_change::State::Ended => c::Landing::Ended {
                    pull: row.pull,
                    task,
                    why: why(row, ending(row).expect("reported landing ended")),
                },
            },
            None => unreachable!("the closer retains its landing row"),
        },
        None => {
            if branch.is_some() {
                c::Landing::Held
            } else {
                c::Landing::NoBranch
            }
        }
    };
    c::Report {
        goal: closer.goal,
        title: c::first_line(words).into(),
        landed: landed.into_boxed(),
        unlanded: unlanded.into_boxed(),
        goal_left: closer.closer.goal_left.clone(),
        landing,
        selections: selections.into_boxed(),
        goal_finish: goal_finish(domain, closer.goal),
    }
}

fn goal_finish(domain: &Domain, goal: u64) -> Option<c::GoalFinish> {
    match domain.core.tasks.task(goal)?.parked_result.as_ref()? {
        tasks::TaskResult::Report { words }
        | tasks::TaskResult::Verdict { words, .. }
        | tasks::TaskResult::Change { words, .. }
        | tasks::TaskResult::NoChange { words }
        | tasks::TaskResult::Approved { words: Some(words), .. } => Some(c::GoalFinish::Words(words.clone())),
        tasks::TaskResult::Failure { reason } => Some(c::GoalFinish::Failed(reason.clone())),
        tasks::TaskResult::Approved { words: None, .. } | tasks::TaskResult::Job { .. } => None,
    }
}

fn report_hold_words(closer: &c::Closer) -> Box<[u8]> {
    const PREFIX: &[u8] = b"the goal's report leaves work undone: ";
    let bound = usize::try_from(tasks::HOLD_ACCOUNT_BYTES).expect("hold words bound");
    let line = closer.goal_left.as_deref().expect("a report hold names its Left line");
    let line = forge::text::fit(line, bound.saturating_sub(PREFIX.len()), "…".as_bytes());
    skein_lib::bytes::concat(&[PREFIX, &line])
}

fn hold_words(domain: &Domain, closer: &forge::CloserRow, hold: c::Hold) -> Box<[u8]> {
    let c::Hold::ClosingStep { step, change } = hold;
    let mut words = List::uncapped();
    append(
        &mut words,
        match step {
            c::Step::Close => b"the close step's change T",
            c::Step::Land => b"the landing T",
        },
    );
    append(&mut words, &super::decimal(change));
    append(&mut words, b" ended ");
    let ended = match domain.forge.change(change) {
        Some(row) => ending(row),
        None => None,
    };
    match ended {
        Some(forge::Ending::Cancelled) => append(&mut words, b"cancelled"),
        Some(forge::Ending::Failed { line }) => {
            append(&mut words, b"failed: ");
            append(&mut words, c::first_line(&line));
        }
        Some(forge::Ending::NoChange) => append(&mut words, b"with no change"),
        Some(forge::Ending::Landed) => append(&mut words, b"landed"),
        None => append(&mut words, b"without an ending account"),
    }
    let branch = match domain.forge.change(change) {
        Some(row) => branch_reason(row.change.ended_held),
        None => None,
    };
    if let Some(reason) = branch {
        append(&mut words, b", held for its branch: ");
        append(&mut words, reason);
    } else {
        append(&mut words, b"; attempts ended without landing: ");
        append(&mut words, &super::decimal(u64::from(ended_count(domain, closer, step))));
        append(&mut words, b", the project's bound ");
        append(&mut words, &super::decimal(u64::from(attempt_bound(domain, closer.task))));
    }
    words.into_boxed()
}

pub(crate) fn made(domain: &mut Domain, task: u64, step: u64, child: Option<u64>) {
    let mut pending = domain.closer_steps.remove(&(task, step));
    let Some(mut row) = domain.forge.closer(task).cloned() else { return };
    if let Some(pending) = &mut pending
        && let Some(replacement) = pending.replacement.take()
    {
        // Only an admitted cancellation and delegation supersede an attempt.
        let mut superseded = List::uncapped();
        for change in &row.superseded {
            superseded.push(*change).expect("admitted close changes");
        }
        superseded.push(replacement.change).expect("admitted close change");
        row.superseded = superseded.into_boxed();
        row.closer.closed = Some(replacement.landed);
        row.making = Some(c::Step::Close);
    }
    if let Some(child) = child
        && let Some(kind) = row.making.take()
    {
        row.closer.running = Some(c::Running { step: kind, change: child });
        let prior = match kind {
            c::Step::Close => &row.close_changes,
            c::Step::Land => &row.landing_changes,
        };
        let mut tasks = List::uncapped();
        for task in prior {
            tasks.push(*task).expect("admitted closer steps");
        }
        tasks.push(child).expect("admitted closer step");
        match kind {
            c::Step::Close => row.close_changes = tasks.into_boxed(),
            c::Step::Land => {
                row.landing_changes = tasks.into_boxed();
                row.closer.landing = Some(child);
            }
        }
        domain.work.push(Work::Forge(forge::Event::Closer { row: row.clone() }));
    }
    let Some(mut pending) = pending else { return };
    if pending.asked {
        let record = domain.core.tasks.task(task).expect("admitted closer step");
        let mut number = None;
        for credit in &record.questions {
            if credit.asked == tasks::Party::Task(row.goal) {
                number = Some(credit.number);
                break;
            }
        }
        c::relay::asked(&mut pending.relay, number.expect("admitted question has its credit"));
    }
    if pending.relay != row.relay {
        domain.work.push(Work::Forge(forge::Event::CloserRelayed { task, relay: pending.relay }));
    }
    if pending.more {
        domain.work.push(Work::Tasks(tasks::Event::WakeProcedure { task }));
    }
}

pub(crate) fn refused(domain: &mut Domain, task: u64, step: u64, why: &jig_domain_core::StepRefusal) -> bool {
    let Some((connector, code, _)) = domain.core.tasks.procedure_due(task) else { return false };
    if connector != domain.config.forge_connector || code != super::GOAL_CLOSER_CODE {
        return false;
    }
    domain.closer_steps.remove(&(task, step));
    match why {
        jig_domain_core::StepRefusal::Moved { .. } => false,
        jig_domain_core::StepRefusal::Denied { .. } | jig_domain_core::StepRefusal::Refused(_) => {
            if let Some(mut row) = domain.forge.closer(task).cloned() {
                if row.making == Some(c::Step::Close) {
                    row.closer.closed = None;
                }
                row.making = None;
                row.done = false;
                domain.work.push(Work::Forge(forge::Event::Closer { row }));
            }
            if let Some(record) = domain.core.tasks.task(task)
                && !record.waiting_on.is_empty()
            {
                let Some((connector, code, step)) = domain.core.tasks.procedure_due(task) else { return true };
                domain.work.push(Work::Core(jig_domain_core::Event::ProcedureStep {
                    task,
                    step,
                    connector,
                    code,
                    decision: ProcedureStep::new(StepEnd::Wait),
                }));
                return true;
            }
            super::holds::hold(domain, task, b"the goal closer's next change was refused");
            true
        }
    }
}

fn prepare_selections(domain: &Domain, row: &mut forge::CloserRow, report: &c::Report) {
    let mut pulls = skein_lib::Map::uncapped();
    for selection in &report.selections {
        let mut head = None;
        for task in domain.forge.change_tasks() {
            let change = domain.forge.change(task).expect("enumerated change");
            if change.pull == Some(selection.pull) {
                head = change.change.last_head;
                break;
            }
        }
        let mut progress = c::PullSelection::new(selection.pull, head);
        for previous in &row.closer.selections {
            if previous.pull == selection.pull && (head.is_none() || previous.head == head) {
                progress = previous.clone();
                break;
            }
        }
        pulls.insert(selection.pull, progress).expect("goal tree bounds selections");
    }
    let mut selections = List::uncapped();
    for (_, progress) in &pulls {
        selections.push(progress.clone()).expect("goal tree bounds selections");
    }
    row.closer.selections = selections.into_boxed();
}

fn settled_selections(closer: &c::Closer) -> Box<[c::Selection]> {
    let mut selections = List::uncapped();
    for selection in &closer.selections {
        let line = match &selection.progress {
            c::SelectionProgress::Done { line } => line.clone(),
            c::SelectionProgress::Pull | c::SelectionProgress::Checks | c::SelectionProgress::Jobs { .. } => {
                unreachable!("report waits for selections")
            }
        };
        selections.push(c::Selection { pull: selection.pull, line }).expect("goal tree bounds selections");
    }
    selections.into_boxed()
}
