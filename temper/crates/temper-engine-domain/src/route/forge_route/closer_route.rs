//! A goal's mechanical close, landing and report (temper/docs/design/domain/forge/branches.md, 2).
use super::{Box, Domain, Env, Limits, List, ProcedureStep, StepEnd, Work, append, forge, tasks};
use temper_engine_domain_forge_closer as c;

#[expect(clippy::too_many_lines, reason = "the procedure gathers its goal and submits one exhaustive step decision")]
pub(crate) fn start_closer_step(domain: &mut Domain, env: &Env<Limits>, context: &tasks::RunContext) {
    let task = context.task;
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
    let branch = goal_branch(domain, env, goal, &repository);
    let mut row = domain.forge.closer(task).cloned().unwrap_or(forge::CloserRow {
        task,
        goal,
        closer: c::Closer::new(),
        making: None,
        close_changes: Box::new([]),
        landing_changes: Box::new([]),
        done: false,
    });
    row.done = false;
    // Its creation answer owns the pending change. A wake before that answer
    // must not delegate the same close or landing step again.
    if row.making.is_some() {
        submit(domain, task, ProcedureStep::new(StepEnd::Wait));
        return;
    }
    let Some((_, _, procedure_step)) = domain.core.tasks.procedure_due(task) else { return };
    let base_holds = match domain.forge.closer_checked(task, procedure_step) {
        Some(Ok(contains)) => Some(contains),
        Some(Err(_)) => {
            submit(domain, task, ProcedureStep::new(StepEnd::Wait));
            return;
        }
        None => None,
    };
    let facts = facts(domain, &row, branch.as_deref(), base_holds);
    let stepped = c::step(&row.closer, &facts);
    let decision = match stepped.decision {
        c::Decision::Check => {
            row.closer = stepped.closer;
            domain.work.push(Work::Forge(forge::Event::Closer { row }));
            domain.work.push(Work::Forge(forge::Event::CheckCloser {
                task,
                step: procedure_step,
                repository: repository.provider,
                branch: branch.expect("only a held goal branch asks containment"),
                base: repository.settings.default_branch.clone(),
            }));
            return;
        }
        c::Decision::Make(kind) => {
            let branch = branch.expect("only a held goal branch makes a step");
            let Some(delegate) = delegate(domain, env, &row, &repository, &branch, &goal_words, kind) else {
                super::holds::hold(
                    domain,
                    task,
                    b"the goal closer cannot delegate its next change within its authority",
                );
                return;
            };
            row.closer = stepped.closer;
            row.making = Some(kind);
            if kind == c::Step::Close {
                row.closer.closed = Some(facts.landed);
            }
            let mut decision = ProcedureStep::new(StepEnd::Wait);
            decision.batch = Box::new([delegate]);
            decision
        }
        c::Decision::Hold(hold) => {
            row.closer = stepped.closer;
            ProcedureStep::new(StepEnd::HoldFrom {
                why: tasks::Hold::Procedure,
                source: Box::new(super::holds::source(domain, None, hold_words(domain, hold))),
            })
        }
        c::Decision::Wait => {
            row.closer = stepped.closer;
            ProcedureStep::new(StepEnd::Wait)
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
            ProcedureStep::new(StepEnd::Result(tasks::TaskResult::Report { words: c::words(&report, bound) }))
        }
    };
    domain.work.push(Work::Forge(forge::Event::Closer { row }));
    submit(domain, task, decision);
}

fn facts(domain: &Domain, row: &forge::CloserRow, branch: Option<&[u8]>, base_holds: Option<bool>) -> c::Facts {
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
        Some(run) => child(domain, run.change),
        None => c::Child::Done,
    };
    let landing_child = match row.closer.landing {
        Some(task) => child(domain, task),
        None => c::Child::Done,
    };
    c::Facts {
        goal_branch: branch.is_some(),
        close_job: domain.forge.close_job(domain.core.tasks.task(row.task).expect("live closer").project).is_some(),
        pending,
        landed: landed.into_boxed(),
        base_holds,
        child: running,
        landing_child,
        released: row.closer.hold.is_some()
            && domain.core.tasks.task(row.task).expect("live closer").due_reason == tasks::Why::Released,
    }
}

fn submit(domain: &mut Domain, task: u64, decision: ProcedureStep) {
    let Some((connector, code, step)) = domain.core.tasks.procedure_due(task) else { return };
    domain.work.push(Work::Core(jig_domain_core::Event::ProcedureStep { task, step, connector, code, decision }));
}

fn goal_branch(domain: &Domain, env: &Env<Limits>, goal: u64, repository: &forge::Repository) -> Option<Box<[u8]>> {
    let root = domain.core.tasks.root(goal)?;
    let branch = super::tree_branch(repository, root, b'g', goal, None, env.limits.forge.name_bytes)?;
    let name = forge::Name {
        forge: repository.provider.forge,
        repository: repository.provider.repository,
        what: super::branch_what(&branch, env.limits.forge.name_bytes)?,
    };
    if domain.forge.hold(&name)?.task == goal { Some(branch) } else { None }
}

fn original(domain: &Domain, closer: &forge::CloserRow, row: &forge::ChangeRow) -> bool {
    if closer.close_changes.contains(&row.task) || closer.landing_changes.contains(&row.task) {
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
        super::forge_change::State::Producing { .. }
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
fn child(domain: &Domain, task: u64) -> c::Child {
    if let Some(row) = domain.core.tasks.task(task) {
        match &row.phase {
            tasks::Phase::Ended(_) => {}
            tasks::Phase::Waiting | tasks::Phase::Active(_) | tasks::Phase::Closing(_) | tasks::Phase::Held { .. } => {
                return c::Child::Pending;
            }
        }
    }
    match domain.forge.change(task) {
        Some(row) => match ending(row) {
            Some(forge::Ending::Landed | forge::Ending::NoChange) => c::Child::Done,
            Some(forge::Ending::Cancelled | forge::Ending::Failed { .. }) => c::Child::Ended,
            None => c::Child::Pending,
        },
        None => c::Child::Pending,
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

fn delegate(
    domain: &Domain,
    env: &Env<Limits>,
    row: &forge::CloserRow,
    repository: &forge::Repository,
    branch: &[u8],
    goal_words: &[u8],
    kind: c::Step,
) -> Option<super::super::Delegate> {
    let context = domain.core.tasks.delegation(row.task)?;
    let available = context
        .numbers
        .budget
        .checked_sub(context.numbers.spent)?
        .checked_sub(context.numbers.spent_below)?
        .checked_sub(context.numbers.reserved)?;
    let spend = available.checked_div(2)?;
    let (words, base, own) = match kind {
        c::Step::Close => {
            let mut words = List::uncapped();
            append(&mut words, b"Settle goal ");
            append(&mut words, &super::decimal(row.goal));
            append(&mut words, b"'s status.");
            (words.into_boxed(), branch.into(), None)
        }
        c::Step::Land => (landing_words(goal_words), repository.settings.default_branch.clone(), Some(branch.into())),
    };
    let change = crate::ChangeCall::words(words, None);
    let mut delegate = super::chat_change(domain, env, &context, &change, repository, base, own, spend).ok()?;
    let mut kinds = List::uncapped();
    for kind in &delegate.authority.delegation.kinds {
        kinds.push(*kind).ok()?;
    }
    if context.authority.delegation.kinds.contains(&tasks::AuthorityExecutor::Job(forge::CLOSE_KIND)) {
        kinds.push(tasks::AuthorityExecutor::Job(forge::CLOSE_KIND)).ok()?;
    }
    delegate.authority.delegation.kinds = kinds.into_boxed();
    if kind == c::Step::Close {
        let mut expanded = List::uncapped();
        for parameter in delegate.spec.parameters {
            expanded.push(parameter).ok()?;
        }
        expanded.push(tasks::Parameter::Bytes { name: 17, value: close_request(domain, row, branch) }).ok()?;
        delegate.spec.parameters = expanded.into_boxed();
    }
    Some(delegate)
}

fn why(row: &forge::ChangeRow, ending: forge::Ending) -> c::Why {
    match ending {
        forge::Ending::Cancelled => c::Why::Cancelled,
        forge::Ending::Failed { line } => c::Why::Failed(line),
        forge::Ending::NoChange => {
            let words = match &row.change.state {
                super::forge_change::State::NoChange { words } => words.as_ref(),
                super::forge_change::State::Producing { .. }
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
                && row.base == repository.settings.default_branch
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
                super::forge_change::State::Producing { .. }
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
        landing,
        selections: selections.into_boxed(),
    }
}
fn hold_words(domain: &Domain, hold: c::Hold) -> Box<[u8]> {
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
    words.into_boxed()
}

pub(crate) fn made(domain: &mut Domain, task: u64, child: Option<u64>) {
    let Some(mut row) = domain.forge.closer(task).cloned() else { return };
    let Some(kind) = row.making.take() else { return };
    if let Some(child) = child {
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
    }
    domain.work.push(Work::Forge(forge::Event::Closer { row }));
}

pub(crate) fn refused(domain: &mut Domain, task: u64, why: &jig_domain_core::StepRefusal) -> bool {
    let Some((connector, code, _)) = domain.core.tasks.procedure_due(task) else { return false };
    if connector != domain.config.forge_connector || code != super::GOAL_CLOSER_CODE {
        return false;
    }
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
                submit(domain, task, ProcedureStep::new(StepEnd::Wait));
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

#[cfg(test)]
mod tests {
    use super::landing_words;
    #[test]
    fn a_closers_landing_uses_the_goal_title_and_draft() {
        assert_eq!(landing_words(b"TITLE\n\nBuild PATH.\nDraft: PATH").as_ref(), b"TITLE\n\nDraft: PATH");
        assert_eq!(landing_words(b"TITLE\n\nOther words").as_ref(), b"TITLE");
    }
}
