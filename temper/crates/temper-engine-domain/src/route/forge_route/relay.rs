//! The root's half of a change's relay (temper/docs/design/domain/forge/change/questions.md, 1).
pub(super) use super::forge::relay_route::questions_held;
use super::forge::relay_route::{
    asking_delegate, find_question, has_ask, hold_words, is_answer, left_words, question_words, word_message,
};
use super::{
    Box, Domain, Env, Limits, List, ProcedureStep, append, forge_change as change, holds, takes_no_messages, tasks,
};
use jig_domain_core::{StepAmendment, StepMessage};

/// A proposed step's relay is saved only after jig admits the whole step.
#[derive(Hash, Debug)]
pub(crate) struct ChangeStep {
    pub(crate) delegate: Option<(change::Delegate, Option<change::CloseAt>)>,
    pub(crate) relayed: Option<change::Relayed>,
    pub(crate) refusal: Option<Box<[u8]>>,
    pub(crate) released: bool,
    pub(crate) passing: Option<change::Passed>,
}

#[expect(clippy::too_many_lines, reason = "one change step assembles its relay with its existing decision")]
pub(super) fn apply(domain: &mut Domain, env: &Env<Limits>, task: u64, mut decision: ProcedureStep) -> ProcedureStep {
    let Some(row) = domain.core.tasks.task(task) else { return decision };
    let offered = domain.core.tasks.procedure_offered(task).unwrap_or(0);
    let change_row = domain.forge.change(task);
    let questions = match change_row {
        Some(row) => row.questions.clone(),
        None => change::Questions::default(),
    };
    let delegate = match change_row {
        Some(row) => row.delegate,
        None => None,
    };
    let mut messages = List::with_capacity(env.limits.tasks.batch);
    for message in core::mem::replace(&mut decision.messages, Box::new([])) {
        messages.push(message).expect("decision messages fit the step batch");
    }
    let producer = live_producer(domain, delegate);
    let Inbox { mut messages, asked, answer, amended, words } =
        take(row, offered, &questions, delegate, producer, env, messages);
    let passed =
        change::pass(&questions, words.as_slice(), producer, usize::try_from(messages.room()).expect("bounded batch"));
    for act in &passed.acts {
        messages
            .push(word_message(act, row.requester, questions.route, env.limits.tasks.message_bytes))
            .expect("passing respects message room");
    }
    let mut passing_questions = questions.clone();
    passing_questions.waiting.clone_from(&passed.waiting);
    let mut ended = List::with_capacity(u32::try_from(change::QUESTIONS_OPEN).expect("question bound fits u32"));
    for question in &questions.open {
        let done = match domain.core.tasks.task(question.asker) {
            Some(task) => match task.phase {
                tasks::Phase::Ended(_) | tasks::Phase::Closing(_) => true,
                tasks::Phase::Waiting | tasks::Phase::Active(_) | tasks::Phase::Held { .. } => false,
            },
            None => true,
        };
        if done {
            ended.push(question.asker).expect("at most the open questions");
        }
    }
    let (_, _, step) = domain.core.tasks.procedure_due(task).expect("an offered change step is due");
    let released = match domain.forge_delegating.get(&(task, step)) {
        Some(pending) => pending.released,
        None => false,
    };
    let mut held = false;
    for dependency in &row.waiting_on {
        if row.started_from.contains(dependency) {
            continue;
        }
        if let Some(dependency) = domain.forge.change(*dependency)
            && let Some(change) = change_row
            && dependency.repository != change.repository
        {
            continue;
        }
        held = true;
        break;
    }
    let heard = change::RelayHeard {
        revision: row.revisions.spec,
        amended: amended.as_slice(),
        message_bytes: usize::try_from(env.limits.tasks.message_bytes).expect("message bound"),
        asked: asked.as_slice(),
        answer,
        ended: ended.as_slice(),
        released,
        held,
    };
    let mut relayed = change::relay(
        &passing_questions,
        &heard,
        &env.limits.forge.change_policy,
        usize::try_from(messages.room()).expect("bounded batch"),
    );
    if let Some(reason) = amended.as_slice().last() {
        amend_delegate(domain, env, task, reason, &relayed.questions, &mut decision);
    }
    for act in &relayed.acts {
        match act {
            change::RelayAct::Answer { asker, number, words } => {
                messages
                    .push(StepMessage {
                        target: *asker,
                        kind: tasks::MessageKind::Answer { question: *number },
                        words: super::forge::text::fit(
                            words,
                            usize::try_from(env.limits.tasks.message_bytes).expect("message bound"),
                            "…".as_bytes(),
                        ),
                    })
                    .expect("relay respects message room");
            }
            change::RelayAct::Ask { asker, number, reasons } => {
                let tasks::Party::Task(target) = row.requester else {
                    unreachable!("only a task's change relays questions")
                };
                let question =
                    find_question(&relayed.questions.open, *asker, *number).expect("an asked question stays open");
                messages
                    .push(StepMessage {
                        target,
                        kind: tasks::MessageKind::Question,
                        words: question_words(&question.words, reasons, env.limits.tasks.message_bytes),
                    })
                    .expect("relay respects message room");
            }
            change::RelayAct::Left { asker, number, line, answered } => {
                let tasks::Party::Task(target) = row.requester else {
                    unreachable!("only a task's change relays questions")
                };
                messages
                    .push(StepMessage {
                        target,
                        kind: tasks::MessageKind::Words,
                        words: left_words(*asker, *number, line, *answered, env.limits.tasks.message_bytes),
                    })
                    .expect("relay respects message room");
            }
            change::RelayAct::Hold => {
                let held = holds::decision(
                    domain,
                    env,
                    task,
                    hold_words(&relayed.questions, env.limits.forge.change_policy.questions),
                );
                decision.end = held.end;
                decision.batch = Box::new([]);
                // A hold has no relay messages; keep its requester account first.
                for message in held.messages {
                    if messages.room() > 0 {
                        messages.push(message).expect("hold account room checked");
                    }
                }
            }
        }
    }
    relayed.more |= passed.more;
    decision.messages = messages.into_boxed();
    if relayed.questions != questions || relayed.more || !passed.acts.is_empty() {
        if let Some(pending) = domain.forge_delegating.get_mut(&(task, step)) {
            pending.relayed = Some(relayed);
            pending.passing = Some(passed);
        } else {
            domain
                .forge_delegating
                .insert(
                    (task, step),
                    ChangeStep {
                        delegate: None,
                        relayed: Some(relayed),
                        refusal: None,
                        released: false,
                        passing: Some(passed),
                    },
                )
                .expect("one pending relay per change");
        }
    }
    decision
}

fn live_producer(domain: &Domain, delegate: Option<(u64, change::Delegate)>) -> Option<u64> {
    let (child, change::Delegate::Produce) = delegate? else { return None };
    let record = domain.core.tasks.task(child)?;
    match record.phase {
        tasks::Phase::Waiting | tasks::Phase::Active(_) | tasks::Phase::Held { .. } => Some(child),
        tasks::Phase::Closing(_) | tasks::Phase::Ended(_) => None,
    }
}

fn amend_delegate(
    domain: &Domain,
    env: &Env<Limits>,
    task: u64,
    reason: &[u8],
    questions: &change::Questions,
    decision: &mut ProcedureStep,
) {
    let Some(row) = domain.forge.change(task) else { return };
    let Some((child, kind)) = row.delegate else { return };
    let Some(record) = domain.core.tasks.task(child) else { return };
    match record.phase {
        tasks::Phase::Waiting | tasks::Phase::Active(_) | tasks::Phase::Held { .. } => {}
        tasks::Phase::Closing(_) | tasks::Phase::Ended(_) => return,
    }
    let Some(change) = domain.core.tasks.task(task) else { return };
    let (words, reason) = match kind {
        change::Delegate::Produce => {
            let Some(words) = super::production_words(
                (change.revisions.spec, &change.spec.words),
                row,
                questions,
                env.limits.tasks.spec_bytes,
            ) else {
                return;
            };
            if super::before_context(&words) == super::before_context(&record.spec.words) {
                return;
            }
            (
                words,
                super::producer_amendment_words(task, change.revisions.spec, reason, env.limits.tasks.message_bytes),
            )
        }
        change::Delegate::Repair(_) | change::Delegate::Resolve { .. } => {
            let Some(words) =
                super::amended_context_words(task, change, &record.spec, questions, env.limits.tasks.spec_bytes)
            else {
                return;
            };
            if words == record.spec.words {
                return;
            }
            (
                words,
                super::delegate_amendment_words(task, change.revisions.spec, reason, env.limits.tasks.message_bytes),
            )
        }
        change::Delegate::Gate { .. } | change::Delegate::Merge { .. } | change::Delegate::Close => return,
    };
    let mut spec = record.spec.clone();
    spec.words = words;
    let mut amendments = List::uncapped();
    for amendment in core::mem::replace(&mut decision.amendments, Box::new([])) {
        amendments.push(amendment).expect("existing step amendments");
    }
    amendments
        .push(StepAmendment {
            task: child,
            revision: record.revisions.spec,
            amendment: tasks::Amendment {
                tracked: None,
                addressed: None,
                instructions: None,
                release: None,
                spec: Some(spec),
                wake: None,
                dependencies: None,
                authority: None,
                llm: None,
                reason,
            },
        })
        .expect("one live delegate amendment");
    decision.amendments = amendments.into_boxed();
}

/// A refused Ask stays taken; commit its failed-effect hold without retrying the full inbox.
pub(crate) fn refused(
    domain: &mut Domain,
    env: &Env<Limits>,
    task: u64,
    step: u64,
    pending: ChangeStep,
    why: &jig_domain_core::StepRefusal,
) -> bool {
    let jig_domain_core::StepRefusal::Refused(problem) = why else { return false };
    let Some(mut relayed) = pending.relayed else { return false };
    let passing = match &pending.passing {
        Some(passed) => !passed.acts.is_empty(),
        None => false,
    };
    if (!has_ask(&relayed.acts) && !passing) || problem.why != tasks::Refusal::Busy {
        return false;
    }
    let Some(capacity) = problem.capacity else { return false };
    let tasks::CapacityCause::Inbox { task: full, messages, bytes } = capacity.cause else { return false };
    let mut words = List::with_capacity(tasks::HOLD_ACCOUNT_BYTES);
    append(&mut words, b"task ");
    append(&mut words, skein_lib::Decimal::of(full).as_bytes());
    append(&mut words, b" busy: its inbox is full (at most ");
    append(&mut words, skein_lib::Decimal::of(u64::from(messages)).as_bytes());
    append(&mut words, b" messages and ");
    append(&mut words, skein_lib::Decimal::of(u64::from(bytes)).as_bytes());
    append(&mut words, b" bytes, counting the room it keeps for the answers to its open questions, its delegates' results and a proposal's decision); room returns as those arrive and are read: retry then, or ask fewer at once");
    let words = words.into_boxed();
    let Some(row) = domain.forge.change(task) else { return false };
    let mut open = List::with_capacity(u32::try_from(change::QUESTIONS_OPEN).expect("question bound fits u32"));
    for question in &row.questions.open {
        open.push(question.clone()).expect("current questions fit");
    }
    for question in &relayed.questions.open {
        if find_question(open.as_slice(), question.asker, question.number).is_none() {
            open.push(change::Open { stands: change::Stands::Taken, ..question.clone() })
                .expect("proposed questions fit");
        }
    }
    let mut open = open.into_boxed();
    if let Some(first) = open.first_mut()
        && first.stands == change::Stands::Asked
        && let Some(task) = domain.core.tasks.task(task)
    {
        for word in &task.inbox {
            if word.from == task.requester && is_answer(&word.kind) {
                first.stands = change::Stands::Held { answer: word.words.clone() };
                break;
            }
        }
    }
    let waiting = match pending.passing {
        Some(passed) => passed.retained,
        None => row.questions.waiting.clone(),
    };
    let answered = relayed.questions.answered;
    relayed.questions = change::Questions { open, waiting, answered, ..row.questions.clone() };
    relayed.acts = Box::new([]);
    relayed.more = false;
    let mut decision = holds::decision_as(domain, env, task, tasks::Hold::EffectFailed, words.clone());
    decision.messages = Box::new([]);
    domain
        .forge_delegating
        .insert(
            (task, step),
            ChangeStep { delegate: None, relayed: Some(relayed), refusal: Some(words), released: false, passing: None },
        )
        .expect("the refused step's slot is free");
    domain.work.push(super::Work::Core(jig_domain_core::Event::ProcedureStep {
        task,
        step,
        connector: domain.config.forge_connector,
        code: super::CHANGE_CODE,
        decision,
    }));
    if domain.facts.try_push(crate::Fact::StepRefused { task, step, problem: problem.clone() }).is_err() {
        domain.facts_lost = domain.facts_lost.saturating_add(1);
    }
    true
}

struct Inbox<'a> {
    messages: List<StepMessage>,
    asked: List<change::Asked<'a>>,
    answer: Option<&'a [u8]>,
    amended: List<&'a [u8]>,
    words: List<change::Waiting>,
}

fn take<'a>(
    row: &'a tasks::TaskRecord,
    offered: u64,
    questions: &change::Questions,
    delegate: Option<(u64, change::Delegate)>,
    producer: Option<u64>,
    env: &Env<Limits>,
    mut messages: List<StepMessage>,
) -> Inbox<'a> {
    let mut asked = List::with_capacity(u32::try_from(change::QUESTIONS_OPEN).expect("question bound fits u32"));
    let mut answer = None;
    let mut amended = List::uncapped();
    let mut words = List::uncapped();
    let accepts_answer = match questions.open.first() {
        Some(question) => question.stands == change::Stands::Asked,
        None => false,
    };
    for word in &row.inbox {
        if word.number > offered {
            continue;
        }
        let kind = match word.kind {
            tasks::MessageKind::Amendment { .. } => {
                amended.push(word.words.as_ref()).expect("offered amendments fit a list");
                continue;
            }
            tasks::MessageKind::Question => tasks::MessageKind::Answer { question: word.number },
            tasks::MessageKind::Words | tasks::MessageKind::Answer { .. } => tasks::MessageKind::Words,
            tasks::MessageKind::Result(_)
            | tasks::MessageKind::Escalation { .. }
            | tasks::MessageKind::Proposal { .. }
            | tasks::MessageKind::ProposalDecision { .. }
            | tasks::MessageKind::SettingsDecision { .. }
            | tasks::MessageKind::FinishRequested { .. }
            | tasks::MessageKind::Met { .. }
            | tasks::MessageKind::Notice { .. }
            | tasks::MessageKind::QuestionAnswered { .. }
            | tasks::MessageKind::QuestionOwed
            | tasks::MessageKind::NothingAwaited
            | tasks::MessageKind::QuestionEnded { .. }
            | tasks::MessageKind::Timer { .. }
            | tasks::MessageKind::PersonStalled { .. }
            | tasks::MessageKind::EffectNotMade { .. }
            | tasks::MessageKind::News { .. } => continue,
        };
        let tasks::Party::Task(target) = word.from else { continue };
        if questions.route != change::Route::None {
            if word.kind == tasks::MessageKind::Question
                && asking_delegate(delegate, target)
                && questions.open.len().saturating_add(usize::try_from(asked.len()).expect("bounded questions"))
                    < change::QUESTIONS_OPEN
            {
                asked
                    .push(change::Asked { asker: target, number: word.number, words: &word.words })
                    .expect("question room checked");
                continue;
            }
            if is_answer(&word.kind) && word.from == row.requester && accepts_answer && answer.is_none() {
                answer = Some(word.words.as_ref());
                continue;
            }
        }
        if word.kind == tasks::MessageKind::Words
            && word.from == row.requester
            && (producer.is_some() || !questions.waiting.is_empty())
        {
            words
                .push(change::Waiting { from: target, number: word.number, words: word.words.clone(), passed: None })
                .expect("offered words fit a list");
            continue;
        }
        if messages.room() == 0 {
            continue;
        }

        if row.requester != word.from && !row.delegates.contains(&target) && !row.references.contains(&target) {
            continue;
        }
        let words = if word.kind != tasks::MessageKind::Question
            && questions.route != change::Route::None
            && asking_delegate(delegate, target)
        {
            REPORT_IN_FINISH.into()
        } else {
            takes_no_messages(row.requester, questions.route, env.limits.tasks.message_bytes)
        };
        messages.push(StepMessage { target, kind, words }).expect("reply room checked");
    }
    Inbox { messages, asked, answer, amended, words }
}

/// What an agent hears of its words to its own change: its report goes in `finish`
/// (temper/docs/design/domain/forge/change/questions.md, 1).
const REPORT_IN_FINISH: &[u8] =
    b"Your report goes in finish; this change takes no words. To ask: message target=this change, form=question.";

/// Refuse `task`'s words to `target` where `target` is the change it works for, under an enabled route.
pub(crate) fn own_agents_words(domain: &Domain, task: u64, target: u64) -> Option<&'static [u8]> {
    let row = domain.forge.change(target)?;
    if row.questions.route != change::Route::None && asking_delegate(row.delegate, task) {
        Some(REPORT_IN_FINISH)
    } else {
        None
    }
}

/// Releasing a refused question keeps its still-live producing delegate.
pub(super) fn continuing_asker(domain: &Domain, task: u64, kind: change::Delegate) -> bool {
    let Some(row) = domain.forge.change(task) else { return false };
    let Some((child, current)) = row.delegate else { return false };
    if kind != current {
        return false;
    }
    let live = match domain.core.tasks.task(child) {
        Some(row) => match row.phase {
            tasks::Phase::Waiting | tasks::Phase::Active(_) | tasks::Phase::Held { .. } => true,
            tasks::Phase::Closing(_) | tasks::Phase::Ended(_) => false,
        },
        None => false,
    };
    if !live {
        return false;
    }
    if !row.questions.waiting.is_empty() && kind == change::Delegate::Produce {
        return true;
    }
    for question in &row.questions.open {
        if question.asker == child {
            return true;
        }
    }
    false
}
