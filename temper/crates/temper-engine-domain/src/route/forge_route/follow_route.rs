//! Standing follow decisions (temper/docs/design/domain/forge/follows.md, 4–5).
use super::{Box, Domain, Env, Limits, List, ProcedureStep, StepEnd, Work, append, forge, tasks};
use forge::follows::{FollowQuestion, FollowRefusal};
use jig_domain_core::{Dependency, StepCancel, StepMessage};
use skein_forge::api::Commit;
mod acts;
#[derive(Hash, Debug)]
pub(crate) struct Pending {
    row: forge::FollowRow,
    made: Box<[Made]>,
    act: Option<forge::follows::FollowAct>,
}
#[derive(Hash, Debug)]
enum Made {
    Change(Commit),
    Question(u64),
}
impl Domain {
    pub fn follow_start(&mut self, project: u32, follow: u32, after: Commit) -> Result<(), FollowRefusal> {
        let row = self.forge.follow_start(project, follow, after)?;
        self.work.push(Work::Forge(forge::Event::Follow { row }));
        Ok(())
    }
}
pub(super) fn start(domain: &mut Domain, env: &Env<Limits>, project: u32, follow: u32) {
    let Some(mut row) = domain.forge.follow_row(project, follow).cloned() else { return };
    if row.task.is_some() || row.stopped.is_some() {
        return;
    }
    let Some(setting) = domain.forge.follow(project, follow) else { return };
    let Some(source) = domain.forge.repository(setting.source.repository) else { return };
    let Some(target) = domain.forge.repository(setting.target.repository) else { return };
    let mut words = List::uncapped();
    append(&mut words, b"Follow ");
    append(&mut words, &super::decimal(u64::from(follow)));
    append(&mut words, b" of project ");
    append(&mut words, &super::decimal(u64::from(project)));
    append(&mut words, b": ");
    append(&mut words, &super::repository_name(source));
    append(&mut words, b" ");
    append(&mut words, branch(&setting.source.branch, source));
    append(&mut words, b" into ");
    append(&mut words, &super::repository_name(target));
    append(&mut words, b" ");
    append(&mut words, branch(&setting.target.branch, target));
    let seed = match domain.core.repair_seed(project) {
        Some(seed) => seed,
        None => {
            let Some(seed) = domain.core.procedure_seed(project) else { return };
            seed
        }
    };
    row.task = Some(seed.number());
    domain.work.push(Work::Core(jig_domain_core::Event::ConnectorRepair {
        seed: Box::new(seed),
        executor: tasks::Executor::Procedure { connector: domain.config.forge_connector, code: super::FOLLOW_CODE },
        spec: tasks::Spec {
            job: None,
            asks: false,
            words: words.into_boxed(),
            parameters: Box::new([]),
            inputs: Box::new([]),
        },
        contract: tasks::Contract::Report { words: env.limits.tasks.result_bytes },
        holdings: Box::new([]),
    }));
    domain.work.push(Work::Forge(forge::Event::Follow { row }));
}
fn branch<'a>(named: &'a [u8], repository: &'a forge::Repository) -> &'a [u8] {
    if named.is_empty() { &repository.settings.default_branch } else { named }
}
#[expect(clippy::too_many_lines, reason = "the owner act and ordered delegate batch form one procedure decision")]
pub(crate) fn step(domain: &mut Domain, env: &Env<Limits>, context: &tasks::RunContext) {
    let task = context.task;
    let Some(mut row) = domain.forge.follow_task(task).cloned() else { return };
    refresh(domain, &mut row);
    take_questions(domain, task, &mut row);
    let Some(setting) = domain.forge.follow(row.project, row.follow).cloned() else { return };
    match setting.owner {
        forge::settings::FollowOwner::Person(_) => {}
        forge::settings::FollowOwner::Task(_) => {
            for question in &mut row.questions {
                if question.person.is_none() && question.answer.is_none() {
                    question.answer = Some(b"The follow's owner gave no answer.".as_slice().into());
                }
            }
        }
    }
    let act = row.pending.clone();
    let mut decision = ProcedureStep::new(StepEnd::Wait);
    let mut made = List::with_capacity(env.limits.tasks.batch);
    let mut batch = List::with_capacity(env.limits.tasks.batch);
    answer_questions(domain, env, &mut row, &mut decision);
    if acts::apply(domain, env, &mut row, &mut decision).is_none() {
        super::holds::hold(domain, task, b"the follow cannot apply its owner act within the specification bound");
        return;
    }
    let Some(delegation) = domain.core.tasks.delegation(task) else { return };
    // A remake is one batch, including the tail. No held task is released.
    let mut remake = None;
    for (at, entry) in row.queue.iter().enumerate() {
        if entry.remake
            && let Some(change) = entry.change
        {
            remake = Some((at, change));
            break;
        }
    }
    let cancels = match remake {
        Some((from, ended)) => remake_tail(domain, &mut row, from, ended),
        None => Box::new([]),
    };
    let mut count = 0_u32;
    for entry in &row.queue {
        if entry.change.is_none() {
            count = count.saturating_add(1);
        }
    }
    if count > env.limits.tasks.batch && remake.is_some() {
        super::holds::hold(domain, task, b"the follow's tail exceeds its step batch bound");
        return;
    }
    count = count.min(env.limits.tasks.batch);
    let spend = super::change_batch_spend(&delegation, count.max(1)).unwrap_or(0);
    let mut previous = None;
    for entry in &row.queue {
        if entry.change.is_none() && batch.room() > 0 {
            let Some(mut member) = delegate(domain, env, &delegation, &row, entry, &setting, spend, count.max(1))
            else {
                super::holds::hold(
                    domain,
                    task,
                    b"the follow cannot fund or describe its next change within its authority and specification bound",
                );
                return;
            };
            if let Some(dependency) = previous {
                member.dependencies = Box::new([dependency]);
            }
            previous = Some(Dependency::Batch(batch.len()));
            batch.push(member).expect("follow step batch");
            made.push(Made::Change(entry.after)).expect("batch mapping");
        } else if let Some(change) = entry.change
            && live(domain, change)
        {
            previous = Some(Dependency::Existing(change));
        }
    }
    for question in &row.questions {
        if question.person.is_none() && question.answer.is_none() && batch.room() > 0 {
            let Some(member) = person(env, &delegation, &row, question, &setting) else {
                super::holds::hold(
                    domain,
                    task,
                    b"the follow cannot make its owner's question task within its authority",
                );
                return;
            };
            batch.push(member).expect("person batch room");
            made.push(Made::Question(question.number)).expect("batch mapping");
        }
    }
    decision.batch = batch.into_boxed();
    let mut all_cancels = List::uncapped();
    for cancel in decision.cancels {
        all_cancels.push(cancel).expect("question cancels");
    }
    for cancel in cancels {
        all_cancels.push(cancel).expect("tail cancels");
    }
    decision.cancels = all_cancels.into_boxed();
    let mut cancelled = List::uncapped();
    for cancel in &decision.cancels {
        cancelled.push(cancel.task).expect("step cancels");
    }
    if let Err(words) =
        check_batch(domain, env, row.project, tasks::Party::Task(task), &decision.batch, cancelled.as_slice())
    {
        super::holds::hold(domain, task, &words);
        return;
    }
    let Some((connector, code, step)) = domain.core.tasks.procedure_due(task) else { return };
    domain
        .follow_steps
        .insert((task, step), Pending { row, made: made.into_boxed(), act })
        .expect("one pending follow step");
    domain.work.push(Work::Core(jig_domain_core::Event::ProcedureStep { task, step, connector, code, decision }));
}
/// Shared tail planner for start and skip of an ended entry. Keeps every owner's word.
pub(crate) fn remake_tail(domain: &Domain, row: &mut forge::FollowRow, from: usize, ended: u64) -> Box<[StepCancel]> {
    let mut cancels = List::uncapped();
    for (at, entry) in row.queue.iter_mut().enumerate() {
        if at >= from {
            if let Some(task) = entry.change
                && live(domain, task)
            {
                let mut reason = List::uncapped();
                append(&mut reason, b"made again after change ");
                append(&mut reason, &super::decimal(ended));
                append(&mut reason, b" ended");
                cancels.push(StepCancel { task, reason: reason.into_boxed(), grace: None }).expect("follow queue");
            }
            entry.change = None;
            entry.state = forge::FollowState::NotMade;
            entry.remake = false;
        }
    }
    cancels.into_boxed()
}
fn live(domain: &Domain, task: u64) -> bool {
    match domain.core.tasks.task(task) {
        Some(row) => match row.phase {
            tasks::Phase::Waiting | tasks::Phase::Active(_) | tasks::Phase::Held { .. } => true,
            tasks::Phase::Closing(_) | tasks::Phase::Ended(_) => false,
        },
        None => false,
    }
}
fn refresh(domain: &Domain, row: &mut forge::FollowRow) {
    let Some(setting) = domain.forge.follow(row.project, row.follow) else { return };
    for entry in &mut row.queue {
        let Some(change) = entry.change else { continue };
        if matches_owner_hold(&entry.state) || entry.remake {
            continue;
        }
        let Some(task) = domain.core.tasks.task(change) else { continue };
        entry.state = match &task.phase {
            tasks::Phase::Held { source, .. } => forge::FollowState::Held(match source.as_deref() {
                Some(tasks::HoldSource::Account { words, .. }) => words.clone(),
                Some(_) | None => b"dependency or task hold".as_slice().into(),
            }),
            tasks::Phase::Closing(closing) => forge::FollowState::Ended(ending(&closing.ending)),
            tasks::Phase::Ended(end) => forge::FollowState::Ended(ending(end)),
            tasks::Phase::Waiting | tasks::Phase::Active(_) => {
                if let Some(dependency) = task.waiting_on.first() {
                    forge::FollowState::WaitsFor(*dependency)
                } else if setting.start == forge::settings::FollowStart::Owner
                    && !entry.word
                    && unstarted(domain, change)
                {
                    forge::FollowState::WaitsWord
                } else {
                    forge::FollowState::Producing
                }
            }
        };
    }
}
fn ending(end: &tasks::Ending) -> Box<[u8]> {
    let words = match end {
        tasks::Ending::Cancelled { reason, .. } | tasks::Ending::Failed { reason } => reason.as_ref(),
        tasks::Ending::Done(_) => b"the change ended without landing",
    };
    temper_engine_domain_forge_closer::first_line(words).into()
}
#[expect(clippy::too_many_arguments, reason = "the follow, entry, authority and bounds are independent step inputs")]
fn delegate(
    domain: &Domain,
    env: &Env<Limits>,
    context: &tasks::DelegationContext,
    row: &forge::FollowRow,
    entry: &forge::FollowEntry,
    setting: &forge::settings::Follow,
    spend: u64,
    count: u32,
) -> Option<super::super::Delegate> {
    let target = domain.forge.repository(setting.target.repository)?;
    let base = Box::from(branch(&setting.target.branch, target));
    let mut call = crate::ChangeCall::words(b"Follow".as_slice().into(), None);
    if let Some(producer) = &setting.producer {
        (call.producer, call.llm) = super::change_llm::read(producer)?;
    }
    // Up is encoded by parameter 18, not parameter 14.
    call.questions = Some(crate::ChangeQuestions::None);
    let mut context = context.clone();
    context.tasks_left = context.tasks_left.checked_div(count)?;
    let mut member = super::chat_change(domain, env, &context, &call, target, base, None, spend).ok()?;
    member.authority.grants = super::change_grants(domain, env, target, &context.authority.grants, true).ok()?;
    member.spec = describe(domain, env, row, core::slice::from_ref(entry), member.spec)?;
    Some(member)
}
fn describe(
    domain: &Domain,
    env: &Env<Limits>,
    row: &forge::FollowRow,
    entries: &[forge::FollowEntry],
    mut spec: tasks::Spec,
) -> Option<tasks::Spec> {
    let setting = domain.forge.follow(row.project, row.follow)?;
    let source = domain.forge.repository(setting.source.repository)?;
    let target = domain.forge.repository(setting.target.repository)?;
    let mut parameters = List::with_capacity(env.limits.tasks.parameters);
    let mut bytes = 0_u32;
    for parameter in spec.parameters {
        if let tasks::Parameter::Bytes { name: 18, .. } = &parameter {
            continue;
        }
        if let tasks::Parameter::Bytes { value, .. } = &parameter {
            bytes = bytes.checked_add(u32::try_from(value.len()).ok()?)?;
        }
        parameters.push(parameter).ok()?;
    }
    let mut follows = List::uncapped();
    append(&mut follows, &super::decimal(u64::from(row.follow)));
    append(&mut follows, b":");
    let mut first = true;
    for entry in entries {
        if !first {
            append(&mut follows, b",");
        }
        append(&mut follows, &super::commit_text(&entry.after));
        first = false;
    }
    bytes = bytes.checked_add(follows.len())?;
    parameters.push(tasks::Parameter::Bytes { name: 18, value: follows.into_boxed() }).ok()?;
    let configured = domain.forge.configured(source.provider.forge)?;
    let pages = configured.web.as_deref().unwrap_or(&configured.git);
    spec.words = forge::follows::words::build(
        entries,
        &forge::follows::words::Words {
            follow: row.follow,
            source: super::repository_name(source),
            branch: branch(&setting.source.branch, source).into(),
            target: super::repository_name(target),
            target_branch: branch(&setting.target.branch, target).into(),
            pages: pages.into(),
            paths: setting.source.paths.clone(),
        },
        env.limits.tasks.spec_bytes.checked_sub(bytes)?,
    )?;
    spec.parameters = parameters.into_boxed();
    Some(spec)
}
fn take_questions(domain: &Domain, task: u64, row: &mut forge::FollowRow) {
    let Some(record) = domain.core.tasks.task(task) else { return };
    let offered = domain.core.tasks.procedure_offered(task).unwrap_or(0);
    let mut questions = List::uncapped();
    for question in &row.questions {
        questions.push(question.clone()).expect("retained questions");
    }
    for word in &record.inbox {
        if word.number > offered || word.kind != tasks::MessageKind::Question {
            continue;
        }
        let tasks::Party::Task(change) = word.from else { continue };
        let mut own = false;
        for entry in &row.queue {
            own |= entry.change == Some(change);
        }
        let mut known = false;
        for question in questions.as_slice() {
            known |= question.number == word.number;
        }
        if own && !known {
            questions
                .push(FollowQuestion {
                    change,
                    number: word.number,
                    words: word.words.clone(),
                    person: None,
                    answer: None,
                })
                .expect("task inbox bounds questions");
        }
    }
    row.questions = questions.into_boxed();
}
fn person(
    env: &Env<Limits>,
    context: &tasks::DelegationContext,
    row: &forge::FollowRow,
    question: &FollowQuestion,
    setting: &forge::settings::Follow,
) -> Option<super::super::Delegate> {
    let mut authority = super::child_authority(context)?;
    authority.tools = tasks::Tools(0);
    authority.grants = Box::new([]);
    authority.budget.spend = 0;
    authority.delegation = tasks::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 };
    let owner = match setting.owner {
        forge::settings::FollowOwner::Person(owner) => owner,
        forge::settings::FollowOwner::Task(_) => return None,
    };
    let mut words = List::uncapped();
    append(&mut words, b"Follow ");
    append(&mut words, &super::decimal(u64::from(row.follow)));
    append(&mut words, b"'s change ");
    append(&mut words, &super::decimal(question.change));
    append(&mut words, b" asks (question ");
    append(&mut words, &super::decimal(question.number));
    append(&mut words, b"): ");
    append(&mut words, &question.words);
    Some(super::super::Delegate {
        executor: tasks::Executor::Person(tasks::PersonAddress::Person(owner)),
        spec: tasks::Spec {
            job: None,
            asks: false,
            words: forge::text::fit(words.as_slice(), skein_lib::usize_of(env.limits.tasks.spec_bytes), "…".as_bytes()),
            parameters: Box::new([]),
            inputs: Box::new([]),
        },
        contract: tasks::Contract::Report { words: env.limits.tasks.message_bytes.min(env.limits.tasks.result_bytes) },
        authority,
        deadline: tasks::Deadline::Named,
        llm: None,
        symbolic_grants: Box::new([]),
        dependencies: Box::new([]),
        wake: Some(tasks::WakePolicy::DEFAULT),
        tracked: None,
        label: 0,
        flags: tasks::Flags::PLAIN,
        budget: super::people::Budget::Named,
    })
}
fn answer_questions(domain: &Domain, env: &Env<Limits>, row: &mut forge::FollowRow, decision: &mut ProcedureStep) {
    let mut kept = List::uncapped();
    let mut messages = List::with_capacity(env.limits.tasks.batch);
    let mut cancels = List::with_capacity(env.limits.tasks.batch);
    for question in &row.questions {
        if !live(domain, question.change) {
            if let Some(person) = question.person
                && live(domain, person)
            {
                if cancels.room() == 0 {
                    kept.push(question.clone()).expect("deferred cancellation");
                    continue;
                }
                cancels
                    .push(StepCancel {
                        task: person,
                        reason: b"its follow change ended".as_slice().into(),
                        grace: None,
                    })
                    .expect("question cancellation room");
            }
            continue;
        }
        if let Some(words) = question.answer.as_deref()
            && messages.room() > 0
        {
            messages
                .push(StepMessage {
                    target: question.change,
                    kind: tasks::MessageKind::Answer { question: question.number },
                    words: words.into(),
                })
                .expect("answer room");
        } else {
            kept.push(question.clone()).expect("retained questions");
        }
    }
    row.questions = kept.into_boxed();
    decision.messages = messages.into_boxed();
    decision.cancels = cancels.into_boxed();
}
pub(crate) fn made(domain: &mut Domain, task: u64, step: u64, first: Option<u64>) {
    let Some(mut pending) = domain.follow_steps.remove(&(task, step)) else { return };
    if let Some(first) = first {
        for (offset, made) in pending.made.iter().enumerate() {
            let child = first.checked_add(skein_lib::len_u64(offset)).expect("batch task number");
            match made {
                Made::Change(after) => {
                    for entry in &mut pending.row.queue {
                        if entry.after == *after {
                            entry.change = Some(child);
                        }
                    }
                }
                Made::Question(number) => {
                    for question in &mut pending.row.questions {
                        if question.number == *number {
                            question.person = Some(child);
                        }
                    }
                }
            }
        }
    }
    for entry in &pending.row.queue {
        if let Some(task) = entry.change
            && let Some(record) = domain.core.tasks.task(task)
            && let Some(change) = domain.forge.change(task)
        {
            for parameter in &record.spec.parameters {
                if let tasks::Parameter::Bytes { name: 18, value } = parameter {
                    let follows = super::change_follows(value, u32::MAX);
                    if change.follows != follows
                        && let Some(follows) = follows
                    {
                        domain.work.push(Work::Forge(forge::Event::FollowChangeAmended {
                            task,
                            follows,
                            words: record.spec.words.clone(),
                        }));
                    }
                }
            }
        }
    }
    if let Some(current) = domain.forge.follow_task(task)
        && current.pending != pending.act
    {
        pending.row.pending.clone_from(&current.pending);
    }
    refresh(domain, &mut pending.row);
    domain.work.push(Work::Forge(forge::Event::Follow { row: pending.row }));
}
pub(crate) fn refused(domain: &mut Domain, task: u64, step: u64, why: &jig_domain_core::StepRefusal) -> bool {
    if domain.follow_steps.remove(&(task, step)).is_none() {
        return false;
    }
    let mut words = List::uncapped();
    append(&mut words, b"the follow's step was refused: ");
    match why {
        jig_domain_core::StepRefusal::Denied { findings, .. } => super::holds::findings(&mut words, findings),
        jig_domain_core::StepRefusal::Refused(problem) => {
            let reason: &[u8] = if problem.why == tasks::Refusal::Tree {
                b"lifetime task capacity".as_slice()
            } else if problem.why == tasks::Refusal::Funding {
                b"insufficient spend"
            } else if problem.why == tasks::Refusal::Batch {
                b"step batch capacity"
            } else if problem.why == tasks::Refusal::Spec {
                b"specification bound"
            } else if problem.why == tasks::Refusal::Depth {
                b"delegation depth"
            } else if problem.why == tasks::Refusal::Busy {
                b"inbox or live task capacity"
            } else {
                b"task admission failed"
            };
            append(&mut words, reason);
        }
        jig_domain_core::StepRefusal::Moved { .. } => append(&mut words, b"a delegate's revision moved"),
    }
    super::holds::hold(domain, task, words.as_slice());
    true
}
pub(crate) fn saved(domain: &mut Domain, record: &jig_domain_core::Record) {
    let jig_domain_core::Record::Tasks(stored) = record else { return };
    let (tasks::Stored::Live(task) | tasks::Stored::Ended(task)) = stored else { return };
    let number = task.number;
    if let Some(mut row) = domain.forge.follow_task(number).cloned() {
        match &task.phase {
            tasks::Phase::Closing(closing) => row.stopped = Some(ending(&closing.ending)),
            tasks::Phase::Ended(end) => row.stopped = Some(ending(end)),
            tasks::Phase::Waiting | tasks::Phase::Active(_) | tasks::Phase::Held { .. } => return,
        }
        domain.work.push(Work::Forge(forge::Event::Follow { row }));
    } else if let tasks::Party::Task(parent) = task.requester
        && let Some(mut row) = domain.forge.follow_task(parent).cloned()
    {
        for ((task, _), _) in &domain.follow_steps {
            if *task == parent {
                return;
            }
        }
        refresh(domain, &mut row);
        if let tasks::Phase::Ended(end) = &task.phase {
            for entry in &mut row.queue {
                if entry.change == Some(number) && !entry.remake {
                    entry.state = forge::FollowState::Ended(ending(end));
                }
            }
            for question in &mut row.questions {
                if question.person == Some(number) {
                    question.answer = Some(match end {
                        tasks::Ending::Done(tasks::TaskResult::Report { words }) => words.clone(),
                        tasks::Ending::Done(_) | tasks::Ending::Failed { .. } | tasks::Ending::Cancelled { .. } => {
                            b"The follow's owner gave no answer.".as_slice().into()
                        }
                    });
                }
            }
        }
        if domain.forge.follow_task(parent) != Some(&row) {
            domain.work.push(Work::Forge(forge::Event::Follow { row }));
        }
    }
}
fn unstarted(domain: &Domain, task: u64) -> bool {
    use super::forge_change::State;
    let Some(row) = domain.forge.change(task) else { return true };
    match &row.change.state {
        State::Producing { requested } => !requested,
        State::Opening { .. }
        | State::Recreating { .. }
        | State::Reopening { .. }
        | State::Checking { .. }
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
        | State::Ended => false,
    }
}
pub(crate) fn owner_wait(domain: &Domain, task: u64, project: u32, follows: &forge::ChangeFollows) -> bool {
    if !unstarted(domain, task) {
        return false;
    }
    let Some(setting) = domain.forge.follow(project, follows.follow) else { return true };
    let Some(row) = domain.forge.follow_row(project, follows.follow) else { return true };

    for after in &follows.commits {
        for entry in &row.queue {
            if entry.after == *after
                && (matches_owner_hold(&entry.state)
                    || pending_hold(row, after)
                    || (setting.start == forge::settings::FollowStart::Owner && !entry.word))
            {
                return true;
            }
        }
    }
    false
}
/// Admission before allocating delegates. Changes cancelled by this decision do not occupy commits.
pub(crate) fn check(
    domain: &Domain,
    project: u32,
    requester: tasks::Party,
    follows: &forge::ChangeFollows,
    own: Option<u64>,
    cancelled: &[u64],
) -> Result<(), Box<[u8]>> {
    let row = domain.forge.follow_row(project, follows.follow);
    let mut previous = None;
    for commit in &follows.commits {
        let mut reason = None;
        match row {
            None => reason = Some(b"the follow has no queue".as_slice()),
            Some(row) => {
                if Some(requester)
                    != match row.task {
                        Some(task) => Some(tasks::Party::Task(task)),
                        None => None,
                    }
                {
                    reason = Some(b"the requester is not that follow's task");
                }
                let mut found = None;
                for (at, entry) in row.queue.iter().enumerate() {
                    if entry.after == *commit {
                        found = Some(at);
                    }
                }
                match found {
                    None => reason = Some(b"the follow's queue lacks this commit"),
                    Some(at) => {
                        if let Some(prior) = previous
                            && at <= prior
                        {
                            reason = Some(b"the commits are not in queue order");
                        }
                        previous = Some(at);
                    }
                }
                for task in domain.forge.change_tasks() {
                    if Some(task) == own || cancelled.contains(&task) || !live(domain, task) {
                        continue;
                    }
                    if let Some(change) = domain.forge.change(task)
                        && let Some(other) = &change.follows
                        && other.follow == follows.follow
                        && other.commits.contains(commit)
                        && let Some(repository) = domain.forge.repository(change.repository)
                        && repository.project == project
                    {
                        reason = Some(b"another live follow change names this commit");
                    }
                }
            }
        }
        if let Some(reason) = reason {
            let mut words = List::uncapped();
            append(&mut words, b"follows: ");
            append(&mut words, &super::commit_text(commit));
            append(&mut words, b": ");
            append(&mut words, reason);
            return Err(words.into_boxed());
        }
    }
    Ok(())
}
pub(crate) fn check_batch(
    domain: &Domain,
    env: &Env<Limits>,
    project: u32,
    requester: tasks::Party,
    batch: &[super::super::Delegate],
    cancelled: &[u64],
) -> Result<(), Box<[u8]>> {
    let mut seen = List::uncapped();
    for member in batch {
        for parameter in &member.spec.parameters {
            if let tasks::Parameter::Bytes { name: 18, value } = parameter {
                let Some(follows) = super::change_follows(value, env.limits.forge.follow_merge) else {
                    let mut words = List::uncapped();
                    append(&mut words, b"follows: malformed parameter 18: ");
                    append(&mut words, value);
                    return Err(words.into_boxed());
                };
                check(domain, project, requester, &follows, None, cancelled)?;
                for commit in &follows.commits {
                    if seen.as_slice().contains(commit) {
                        let mut words = List::uncapped();
                        append(&mut words, b"follows: ");
                        append(&mut words, &super::commit_text(commit));
                        append(&mut words, b": another change in this batch names this commit");
                        return Err(words.into_boxed());
                    }
                    seen.push(*commit).expect("bounded batch commits");
                }
            }
        }
    }
    Ok(())
}

fn matches_owner_hold(state: &forge::FollowState) -> bool {
    match state {
        forge::FollowState::HeldByOwner(_) => true,
        forge::FollowState::NotMade
        | forge::FollowState::WaitsWord
        | forge::FollowState::WaitsFor(_)
        | forge::FollowState::Producing
        | forge::FollowState::Held(_)
        | forge::FollowState::Ended(_) => false,
    }
}
fn pending_hold(row: &forge::FollowRow, commit: &Commit) -> bool {
    match &row.pending {
        Some(forge::follows::FollowAct::Hold { after, .. }) => after == commit,
        Some(
            forge::follows::FollowAct::Start { .. }
            | forge::follows::FollowAct::Skip { .. }
            | forge::follows::FollowAct::Merge { .. },
        )
        | None => false,
    }
}
