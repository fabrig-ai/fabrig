//! The root's half of a change's relay (temper/docs/design/domain/forge/change/questions.md, 1).
use super::{
    Box, Domain, Env, Limits, List, ProcedureStep, append, foreman, forge_change as change, holds, takes_no_messages,
    tasks,
};
use jig_domain_core::{StepCancel, StepMessage};

/// A proposed step's relay is saved only after jig admits the whole step.
#[derive(Hash, Debug)]
pub(crate) struct ChangeStep {
    pub(crate) foreman: bool,
    pub(crate) delegate: Option<change::Delegate>,
    pub(crate) relayed: Option<change::Relayed>,
    pub(crate) refusal: Option<Box<[u8]>>,
    pub(crate) released: bool,
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
    for message in decision.messages {
        messages.push(message).expect("decision messages fit the step batch");
    }
    let Inbox { mut messages, asked, answer } = take(row, offered, &questions, delegate, env, messages);
    if questions.route == change::Route::None {
        decision.messages = messages.into_boxed();
        return decision;
    }
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
    let foreman = foreman::settled(row, offered, &questions);
    let heard = change::RelayHeard {
        asked: asked.as_slice(),
        foreman,
        answer,
        ended: ended.as_slice(),
        kept: !row.waiting_on.is_empty(),
        released,
    };
    let deferred = answer.is_some() && messages.room() == 0 && !heard.kept;
    let heard = change::RelayHeard { kept: heard.kept || deferred, ..heard };
    let mut relayed = change::relay(
        &questions,
        &heard,
        &env.limits.forge.change_policy,
        usize::try_from(messages.room()).expect("bounded batch"),
    );
    relayed.more |= deferred;
    let mut foreman_delegate = None;
    if has_foreman(&relayed.acts) {
        // Learn one delegate number per step. Commit other work before asking a foreman.
        if !decision.batch.is_empty() || !messages.is_empty() || relayed.acts.len() > 1 {
            defer_foreman(&mut relayed);
        } else if let Some(cause) = super::change_authority_lack(domain, task) {
            relayed = failed_foreman(relayed, &heard, env, &change::authority_words(cause), messages.room());
        } else {
            let question = relayed.questions.open.first().expect("the foreman has a head question");
            foreman_delegate = foreman::delegate(domain, env, task, question);
            if foreman_delegate.is_none() {
                relayed = failed_foreman(
                    relayed,
                    &heard,
                    env,
                    b"foreman delegate cannot be described within its authority and brief bounds",
                    messages.room(),
                );
            }
        }
    }
    let mut cancels = List::with_capacity(env.limits.tasks.batch);
    for cancel in decision.cancels {
        cancels.push(cancel).expect("decision cancels fit the step batch");
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
            change::RelayAct::Cancel { task } => {
                cancels
                    .push(StepCancel { task: *task, reason: b"its asker ended".as_slice().into(), grace: None })
                    .expect("foreman cancels fit the step batch");
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
            change::RelayAct::Foreman { .. } => {
                decision.batch = Box::new([foreman_delegate.take().expect("the foreman is described")]);
            }
        }
    }
    decision.messages = messages.into_boxed();
    decision.cancels = cancels.into_boxed();
    if relayed.questions != questions || relayed.more {
        if let Some(pending) = domain.forge_delegating.get_mut(&(task, step)) {
            pending.foreman = has_foreman(&relayed.acts);
            pending.relayed = Some(relayed);
        } else {
            domain
                .forge_delegating
                .insert(
                    (task, step),
                    ChangeStep {
                        foreman: has_foreman(&relayed.acts),
                        delegate: None,
                        relayed: Some(relayed),
                        refusal: None,
                        released: false,
                    },
                )
                .expect("one pending relay per change");
        }
    }
    decision
}

fn question_words(words: &[u8], reasons: &[u8], bound: u32) -> Box<[u8]> {
    let mut text = List::uncapped();
    append(&mut text, words);
    if !reasons.is_empty() {
        append(&mut text, b"\n\n");
        append(&mut text, reasons);
    }
    super::forge::text::fit(text.as_slice(), usize::try_from(bound).expect("message bound"), "…".as_bytes())
}

fn left_words(asker: u64, number: u64, line: &[u8], answered: bool, bound: u32) -> Box<[u8]> {
    let mut words = List::uncapped();
    append(&mut words, b"Task ");
    append(&mut words, skein_lib::Decimal::of(asker).as_bytes());
    if answered {
        append(&mut words, b" ended; your answer to its question ");
    } else {
        append(&mut words, b" ended with its question ");
    }
    append(&mut words, skein_lib::Decimal::of(number).as_bytes());
    if answered {
        append(&mut words, b" goes no further: ");
    } else {
        append(&mut words, b" open; it reached no one: ");
    }
    append(&mut words, line);
    super::forge::text::fit(words.as_slice(), usize::try_from(bound).expect("message bound"), "…".as_bytes())
}

fn hold_words(questions: &change::Questions, bound: u32) -> Box<[u8]> {
    let mut words = List::with_capacity(tasks::HOLD_ACCOUNT_BYTES);
    let bound = skein_lib::Decimal::of(u64::from(bound));
    append(&mut words, b"its questions are spent: ");
    append(&mut words, bound.as_bytes());
    append(&mut words, b" of ");
    append(&mut words, bound.as_bytes());
    append(&mut words, b" taken");
    if let Some(question) = questions.open.first() {
        append(&mut words, b"; question ");
        append(&mut words, skein_lib::Decimal::of(question.number).as_bytes());
        append(&mut words, b" of task ");
        append(&mut words, skein_lib::Decimal::of(question.asker).as_bytes());
        append(&mut words, b" waits; release it to pass the question up");
    }
    words.into_boxed()
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
    if pending.foreman {
        return refused_foreman(domain, env, task, step, pending, why);
    }
    let jig_domain_core::StepRefusal::Refused(problem) = why else { return false };
    let Some(mut relayed) = pending.relayed else { return false };
    if !has_ask(&relayed.acts) || problem.why != tasks::Refusal::Busy {
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
    relayed.questions = change::Questions { open, ..row.questions.clone() };
    relayed.acts = Box::new([]);
    relayed.more = false;
    let mut decision = holds::decision_as(domain, env, task, tasks::Hold::EffectFailed, words.clone());
    decision.messages = Box::new([]);
    domain
        .forge_delegating
        .insert(
            (task, step),
            ChangeStep {
                foreman: false,
                delegate: None,
                relayed: Some(relayed),
                refusal: Some(words),
                released: false,
            },
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
}

fn take<'a>(
    row: &'a tasks::TaskRecord,
    offered: u64,
    questions: &change::Questions,
    delegate: Option<(u64, change::Delegate)>,
    env: &Env<Limits>,
    mut messages: List<StepMessage>,
) -> Inbox<'a> {
    let mut asked = List::with_capacity(u32::try_from(change::QUESTIONS_OPEN).expect("question bound fits u32"));
    let mut answer = None;
    let accepts_answer = match questions.open.first() {
        Some(question) => question.stands == change::Stands::Asked,
        None => false,
    };
    for word in &row.inbox {
        if word.number > offered {
            continue;
        }
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
        if messages.room() == 0 {
            continue;
        }
        let kind = match word.kind {
            tasks::MessageKind::Question => tasks::MessageKind::Answer { question: word.number },
            tasks::MessageKind::Words | tasks::MessageKind::Answer { .. } => tasks::MessageKind::Words,
            tasks::MessageKind::Result(_)
            | tasks::MessageKind::Escalation { .. }
            | tasks::MessageKind::Proposal { .. }
            | tasks::MessageKind::ProposalDecision { .. }
            | tasks::MessageKind::SettingsDecision { .. }
            | tasks::MessageKind::FinishRequested { .. }
            | tasks::MessageKind::Amendment { .. }
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
    Inbox { messages, asked, answer }
}

fn has_foreman(acts: &[change::RelayAct]) -> bool {
    for act in acts {
        match act {
            change::RelayAct::Foreman { .. } => return true,
            change::RelayAct::Answer { .. }
            | change::RelayAct::Ask { .. }
            | change::RelayAct::Left { .. }
            | change::RelayAct::Cancel { .. }
            | change::RelayAct::Hold => {}
        }
    }
    false
}

fn defer_foreman(relayed: &mut change::Relayed) {
    let question = relayed.questions.open.first_mut().expect("the foreman has a head question");
    question.stands = change::Stands::Taken;
    relayed.questions.taken = relayed.questions.taken.saturating_sub(1);
    let mut acts = List::uncapped();
    for act in &relayed.acts {
        match act {
            change::RelayAct::Foreman { .. } => {}
            change::RelayAct::Answer { .. }
            | change::RelayAct::Ask { .. }
            | change::RelayAct::Left { .. }
            | change::RelayAct::Cancel { .. }
            | change::RelayAct::Hold => {
                acts.push(act.clone()).expect("bounded relay acts");
            }
        }
    }
    relayed.acts = acts.into_boxed();
    relayed.more = true;
}

fn failed_foreman(
    relayed: change::Relayed,
    heard: &change::RelayHeard<'_>,
    env: &Env<Limits>,
    words: &[u8],
    room: u32,
) -> change::Relayed {
    let settled = change::RelayHeard {
        asked: &[],
        foreman: Some((0, change::Settled::Failed(words))),
        answer: None,
        ended: &[],
        kept: heard.kept,
        released: false,
    };
    change::relay(
        &relayed.questions,
        &settled,
        &env.limits.forge.change_policy,
        usize::try_from(room).expect("bounded batch"),
    )
}

fn refused_foreman(
    domain: &mut Domain,
    env: &Env<Limits>,
    task: u64,
    step: u64,
    pending: ChangeStep,
    why: &jig_domain_core::StepRefusal,
) -> bool {
    let Some(relayed) = pending.relayed else { return false };
    let mut words = List::with_capacity(tasks::HOLD_ACCOUNT_BYTES);
    append(&mut words, b"Foreman delegate refused");
    match why {
        jig_domain_core::StepRefusal::Denied { findings, .. } => holds::findings(&mut words, findings),
        jig_domain_core::StepRefusal::Refused(problem) => {
            append(&mut words, b": ");
            append(&mut words, foreman::refusal_words(problem.why));
        }
        jig_domain_core::StepRefusal::Moved { .. } => return false,
    }
    let heard =
        change::RelayHeard { asked: &[], foreman: None, answer: None, ended: &[], kept: false, released: false };
    let relayed = failed_foreman(relayed, &heard, env, words.as_slice(), env.limits.tasks.batch);
    let Some(question) = relayed.questions.open.first() else { return false };
    let Some(row) = domain.core.tasks.task(task) else { return false };
    let tasks::Party::Task(target) = row.requester else { return false };
    let mut messages = List::with_capacity(env.limits.tasks.batch);
    for act in &relayed.acts {
        if let change::RelayAct::Ask { reasons, .. } = act {
            messages
                .push(StepMessage {
                    target,
                    kind: tasks::MessageKind::Question,
                    words: question_words(&question.words, reasons, env.limits.tasks.message_bytes),
                })
                .expect("one foreman escalation");
        }
    }
    let mut decision = ProcedureStep::new(super::StepEnd::Wait);
    decision.messages = messages.into_boxed();
    domain
        .forge_delegating
        .insert(
            (task, step),
            ChangeStep { foreman: false, delegate: None, relayed: Some(relayed), refusal: None, released: false },
        )
        .expect("the refused foreman's step is free");
    domain.work.push(super::Work::Core(jig_domain_core::Event::ProcedureStep {
        task,
        step,
        connector: domain.config.forge_connector,
        code: super::CHANGE_CODE,
        decision,
    }));
    true
}

fn has_ask(acts: &[change::RelayAct]) -> bool {
    for act in acts {
        match act {
            change::RelayAct::Ask { .. } => return true,
            change::RelayAct::Answer { .. }
            | change::RelayAct::Foreman { .. }
            | change::RelayAct::Left { .. }
            | change::RelayAct::Cancel { .. }
            | change::RelayAct::Hold => {}
        }
    }
    false
}

#[expect(clippy::manual_find, reason = "the programming model uses bounded loops instead of iterator closures")]
fn find_question(open: &[change::Open], asker: u64, number: u64) -> Option<&change::Open> {
    for question in open {
        if question.asker == asker && question.number == number {
            return Some(question);
        }
    }
    None
}

fn is_answer(kind: &tasks::MessageKind) -> bool {
    match kind {
        tasks::MessageKind::Answer { .. } => true,
        tasks::MessageKind::Question
        | tasks::MessageKind::Words
        | tasks::MessageKind::Result(_)
        | tasks::MessageKind::Escalation { .. }
        | tasks::MessageKind::Proposal { .. }
        | tasks::MessageKind::ProposalDecision { .. }
        | tasks::MessageKind::SettingsDecision { .. }
        | tasks::MessageKind::FinishRequested { .. }
        | tasks::MessageKind::Amendment { .. }
        | tasks::MessageKind::Met { .. }
        | tasks::MessageKind::Notice { .. }
        | tasks::MessageKind::QuestionAnswered { .. }
        | tasks::MessageKind::QuestionOwed
        | tasks::MessageKind::NothingAwaited
        | tasks::MessageKind::QuestionEnded { .. }
        | tasks::MessageKind::Timer { .. }
        | tasks::MessageKind::PersonStalled { .. }
        | tasks::MessageKind::EffectNotMade { .. }
        | tasks::MessageKind::News { .. } => false,
    }
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

fn asking_delegate(delegate: Option<(u64, change::Delegate)>, target: u64) -> bool {
    match delegate {
        Some((child, change::Delegate::Produce | change::Delegate::Repair(_) | change::Delegate::Resolve { .. })) => {
            child == target
        }
        Some((_, change::Delegate::Gate { .. } | change::Delegate::Merge { .. } | change::Delegate::Close)) | None => {
            false
        }
    }
}

pub(super) fn questions_held(state: &change::State) -> bool {
    match state {
        change::State::Held { why, .. } => *why == change::Hold::Questions,
        change::State::Producing { .. }
        | change::State::Opening { .. }
        | change::State::Recreating { .. }
        | change::State::Reopening { .. }
        | change::State::Checking { .. }
        | change::State::Judging { .. }
        | change::State::Gating { .. }
        | change::State::CancellingGates { .. }
        | change::State::Ready { .. }
        | change::State::Preparing { .. }
        | change::State::Queued { .. }
        | change::State::First { .. }
        | change::State::Updating { .. }
        | change::State::WaitingProducer { .. }
        | change::State::Resolving { .. }
        | change::State::Repairing { .. }
        | change::State::Landing { .. }
        | change::State::Landed { .. }
        | change::State::Red { .. }
        | change::State::Cleared { .. }
        | change::State::Uncleared { .. }
        | change::State::NoChange { .. }
        | change::State::Ended => false,
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
    for question in &row.questions.open {
        if question.asker == child {
            return true;
        }
    }
    false
}
