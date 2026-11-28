//! The root's half of a change's relay (temper/docs/design/domain/forge/change/questions.md, 1).
pub(super) use super::forge::relay_route::questions_held;
use super::forge::relay_route::{asking_delegate, find_question, has_ask, left_words, question_words, word_message};
use super::forge::routing::answer_refusal;
use super::forge::text::fit;
use super::{
    Box, Domain, Env, Limits, List, ProcedureStep, StepEnd, append, forge_change as change, holds, takes_no_messages,
    tasks,
};
use jig_domain_core::{StepAmendment, StepMessage};

/// A proposed step's relay is saved only after jig admits the whole step.
#[derive(Hash, Debug)]
pub(crate) struct ChangeStep {
    pub(crate) delegate: Option<(change::Delegate, Option<change::CloseAt>)>,
    pub(crate) relayed: Option<change::Relayed>,
    pub(crate) refusal: Option<Box<[u8]>>,
    pub(crate) released: bool,
    pub(crate) passing: Option<Passing>,
}

#[derive(Hash, Debug)]
pub(crate) struct Passing {
    words: change::Passed,
    /// If admission instead holds and consumes the inbox, none of these replies went out.
    replies: Box<[change::Reply]>,
}

#[expect(clippy::too_many_lines, reason = "one change step assembles its relay with its existing decision")]
pub(super) fn apply(domain: &mut Domain, env: &Env<Limits>, task: u64, mut decision: ProcedureStep) -> ProcedureStep {
    let Some(row) = domain.engine.core().tasks.task(task) else { return decision };
    let offered = domain.engine.core().tasks.procedure_offered(task).unwrap_or(0);
    let change_row = domain.forge.change(task);
    let questions = match change_row {
        Some(row) => &row.questions,
        None => &change::Questions::default(),
    };
    // With no offered words or retained relay work there is no step to
    // propose. In particular, a held answer is still open and must be tried.
    if offered == 0
        && questions.open.is_empty()
        && questions.waiting.is_empty()
        && questions.replies.is_empty()
        && questions.amended.len() <= change::AMENDMENTS_KEPT
        && decision.messages.len() <= skein_lib::usize_of(env.limits.tasks.batch)
    {
        return decision;
    }
    let delegate = match change_row {
        Some(row) => row.delegate,
        None => None,
    };
    let mut messages = List::with_capacity(env.limits.tasks.batch);
    for message in core::mem::replace(&mut decision.messages, Box::new([])) {
        messages.push(message).expect("decision messages fit the step batch");
    }
    let producer = live_producer(domain, delegate);
    let Inbox { asked, answer, amended, words, replies } = take(row, offered, questions, delegate, producer, env);
    let sent = send_replies(&replies, &mut messages, env.limits.tasks.message_bytes);
    let passed =
        change::pass(questions, words.as_slice(), producer, usize::try_from(messages.room()).expect("bounded batch"));
    for act in &passed.acts {
        messages
            .push(word_message(act, row.requester, questions.route, env.limits.tasks.message_bytes))
            .expect("passing respects message room");
    }
    let mut passing_questions = questions.clone();
    passing_questions.waiting.clone_from(&passed.waiting);
    passing_questions.replies = replies.get(sent..).expect("sent reply prefix").into();
    let mut ended = List::with_capacity(u32::try_from(change::QUESTIONS_OPEN).expect("question bound fits u32"));
    for question in &questions.open {
        let done = match domain.engine.core().tasks.task(question.asker) {
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
    let (_, _, step) = domain.engine.core().tasks.procedure_due(task).expect("an offered change step is due");
    let released = match domain.forge_delegating.get(&(task, step)) {
        Some(pending) => pending.released,
        None => false,
    };
    let mut blocker = None;
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
        blocker = Some(*dependency);
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
        dependency: blocker,
        now: env.wall,
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
    let mut notice = None;
    for act in &relayed.acts {
        match act {
            change::RelayAct::Answer { asker, number, words } => {
                messages
                    .push(StepMessage {
                        notice: None,
                        target: *asker,
                        kind: tasks::MessageKind::Answer { question: *number },
                        words: fit(
                            words,
                            usize::try_from(env.limits.tasks.message_bytes).expect("message bound"),
                            "…".as_bytes(),
                        ),
                    })
                    .expect("relay respects message room");
            }
            change::RelayAct::Held { question, blocked } => {
                let tasks::Party::Task(target) = row.requester else {
                    unreachable!("only a task's change relays questions")
                };
                notice = Some(held_notice(target, *question, *blocked));
            }
            change::RelayAct::Delivered { question, since, asker } => {
                let tasks::Party::Task(target) = row.requester else {
                    unreachable!("only a task's change relays questions")
                };
                let mut words = answer_notice(*question);
                append(&mut words, b", held since ");
                append(&mut words, &since.utc_second());
                append(&mut words, b", is delivered to task ");
                append(&mut words, skein_lib::Decimal::of(*asker).as_bytes());
                append(&mut words, b".");
                messages
                    .push(StepMessage {
                        notice: None,
                        target,
                        kind: tasks::MessageKind::Words,
                        words: words.into_boxed(),
                    })
                    .expect("relay reserves ordinary room for delivery and its notice together");
            }
            change::RelayAct::Ask { asker, number, reasons } => {
                let tasks::Party::Task(target) = row.requester else {
                    unreachable!("only a task's change relays questions")
                };
                let question =
                    find_question(&relayed.questions.open, *asker, *number).expect("an asked question stays open");
                messages
                    .push(StepMessage {
                        notice: None,
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
                        notice: None,
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
                    holds::questions_account(Some(&relayed.questions), env.limits.forge.change_policy.questions),
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
    relayed.more |= passed.more || !relayed.questions.replies.is_empty();
    if !relayed.questions.replies.is_empty()
        && let StepEnd::Result(_) = decision.end
    {
        decision.end = StepEnd::Wait;
    }
    if let Some(notice) = notice {
        messages.raise(messages.capacity().saturating_add(1));
        messages.push(notice).expect("the question's notice is counted apart from ordinary messages");
    }
    decision.messages = messages.into_boxed();
    if relayed.questions != *questions || relayed.more || !passed.acts.is_empty() {
        let passing = Passing { words: passed, replies };
        if let Some(pending) = domain.forge_delegating.get_mut(&(task, step)) {
            pending.relayed = Some(relayed);
            pending.passing = Some(passing);
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
                        passing: Some(passing),
                    },
                )
                .expect("one pending relay per change");
        }
    }
    decision
}

// The fixed text, two u64 decimals and a UTC second fit the notice bound.
fn answer_notice(question: u64) -> List<u8> {
    let cap = u32::try_from(tasks::ANSWER_NOTICE_BYTES).expect("fixed notice bound");
    let mut words = List::with_capacity(cap);
    append(&mut words, b"Your answer to question ");
    append(&mut words, skein_lib::Decimal::of(question).as_bytes());
    words
}

fn held_notice(target: u64, question: u64, blocked: change::Blocked) -> StepMessage {
    let mut words = answer_notice(question);
    match blocked.dependency {
        Some(task) => {
            append(&mut words, b" is held until task ");
            append(&mut words, skein_lib::Decimal::of(task).as_bytes());
            append(&mut words, b" ends");
        }
        None => append(&mut words, b" is held until this change has message room"),
    }
    append(&mut words, b"; held since ");
    append(&mut words, &blocked.since.utc_second());
    append(&mut words, b".");
    StepMessage { notice: Some(question), target, kind: tasks::MessageKind::Words, words: words.into_boxed() }
}

pub(in crate::route) fn live_producer(domain: &Domain, delegate: Option<(u64, change::Delegate)>) -> Option<u64> {
    let (child, change::Delegate::Produce) = delegate? else { return None };
    let record = domain.engine.core().tasks.task(child)?;
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
    let Some(record) = domain.engine.core().tasks.task(child) else { return };
    match record.phase {
        tasks::Phase::Waiting | tasks::Phase::Active(_) | tasks::Phase::Held { .. } => {}
        tasks::Phase::Closing(_) | tasks::Phase::Ended(_) => return,
    }
    let Some(change) = domain.engine.core().tasks.task(task) else { return };
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
        Some(passed) => !passed.words.acts.is_empty(),
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
    let notice = keep_refused_answer(domain, env, task, &relayed, &mut open);
    let (waiting, replies) = match pending.passing {
        Some(passed) => (passed.words.retained, passed.replies),
        None => (row.questions.waiting.clone(), row.questions.replies.clone()),
    };
    let answered = relayed.questions.answered;
    relayed.questions = change::Questions { open, waiting, answered, replies, ..row.questions.clone() };
    relayed.acts = Box::new([]);
    relayed.more = false;
    let mut decision = holds::decision_as(domain, env, task, tasks::Hold::EffectFailed, words.clone());
    decision.messages = match notice {
        Some(notice) => Box::new([notice]),
        None => Box::new([]),
    };
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

fn keep_refused_answer(
    domain: &Domain,
    env: &Env<Limits>,
    task: u64,
    relayed: &change::Relayed,
    open: &mut [change::Open],
) -> Option<StepMessage> {
    let first = open.first_mut()?;
    if first.stands != change::Stands::Asked {
        return None;
    }
    let offered = domain.engine.core().tasks.procedure_offered(task)?;
    let row = domain.engine.core().tasks.task(task)?;
    let tasks::Party::Task(target) = row.requester else { return None };
    for word in &row.inbox {
        if word.number <= offered
            && word.from == row.requester
            && let tasks::MessageKind::Answer { question } = word.kind
        {
            let mut dependency = None;
            for act in &relayed.acts {
                if let change::RelayAct::Held { blocked, .. } = act {
                    dependency = blocked.dependency;
                    break;
                }
            }
            let blocked = change::Blocked { dependency, since: env.wall };
            first.stands = change::Stands::Held { answer: word.words.clone(), question, blocked: Some(blocked) };
            return Some(held_notice(target, question, blocked));
        }
    }
    None
}

struct Inbox<'a> {
    replies: Box<[change::Reply]>,
    asked: List<change::Asked<'a>>,
    answer: Option<change::RelayAnswer<'a>>,
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
) -> Inbox<'a> {
    let mut asked = List::with_capacity(u32::try_from(change::QUESTIONS_OPEN).expect("question bound fits u32"));
    let mut answer = None;
    let mut amended = List::uncapped();
    let mut words = List::uncapped();
    let mut replies = List::uncapped();
    for reply in &questions.replies {
        replies.push(reply.clone()).expect("retained replies");
    }
    let bound = env.limits.tasks.message_bytes;
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
            tasks::MessageKind::Question => Some(word.number),
            tasks::MessageKind::Words | tasks::MessageKind::Answer { .. } => None,
            tasks::MessageKind::Result(_)
            | tasks::MessageKind::Escalation { .. }
            | tasks::MessageKind::Proposal { .. }
            | tasks::MessageKind::ProposalDecision { .. }
            | tasks::MessageKind::SettingsDecision { .. }
            | tasks::MessageKind::FinishRequested { .. }
            | tasks::MessageKind::Met { .. }
            | tasks::MessageKind::Notice { .. }
            | tasks::MessageKind::PassedQuestion { .. }
            | tasks::MessageKind::PassedAnswer { .. }
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
            // Jig has already admitted the answer against this relay's one question credit.
            if let tasks::MessageKind::Answer { question } = word.kind
                && word.from == row.requester
                && accepts_answer
                && answer.is_none()
            {
                answer = Some(change::RelayAnswer { question, words: &word.words });
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
        let words = if let tasks::MessageKind::Answer { question } = word.kind {
            let open = if answer.is_none() { open_to(row, questions, target) } else { None };
            answer_refusal(row.number, question, open)
        } else {
            if row.requester != word.from && !row.delegates.contains(&target) && !row.references.contains(&target) {
                continue;
            }
            if word.kind == tasks::MessageKind::Words
                && questions.route != change::Route::None
                && asking_delegate(delegate, target)
            {
                REPORT_IN_FINISH.into()
            } else {
                takes_no_messages(row.requester, questions.route, bound)
            }
        };
        let words = fit(&words, usize::try_from(bound).expect("message bound"), "…".as_bytes());
        replies.push(change::Reply { target, question: kind, words }).expect("offered replies");
    }
    Inbox { replies: replies.into_boxed(), asked, answer, amended, words }
}

fn send_replies(replies: &[change::Reply], messages: &mut List<StepMessage>, bound: u32) -> usize {
    let mut sent = 0_usize;
    for reply in replies {
        if messages.room() == 0 {
            break;
        }
        let kind = match reply.question {
            Some(question) => tasks::MessageKind::Answer { question },
            None => tasks::MessageKind::Words,
        };
        let words = fit(&reply.words, usize::try_from(bound).expect("message bound"), "…".as_bytes());
        messages.push(StepMessage { notice: None, target: reply.target, kind, words }).expect("reply room checked");
        sent = sent.saturating_add(1);
    }
    sent
}

fn open_to<'a>(row: &tasks::TaskRecord, questions: &'a change::Questions, from: u64) -> Option<(u64, &'a [u8])> {
    let open = questions.open.first()?;
    if open.stands == change::Stands::Asked {
        for credit in &row.questions {
            if credit.answerer == tasks::Party::Task(from) {
                return Some((credit.number, &open.line));
            }
        }
    }
    None
}

/// Replace jig's reference refusal only at the two relays.
pub(crate) fn refused_answer(domain: &Domain, from: u64, target: u64, named: u64, bound: u32) -> Option<Box<[u8]>> {
    let open = if let Some(change) = domain.forge.change(target) {
        match domain.engine.core().tasks.task(target) {
            Some(row) => open_to(row, &change.questions, from),
            None => None,
        }
    } else {
        let closer = domain.forge.closer(target)?;
        closer.relay.open_to(closer.goal, from)
    };
    if let Some((number, _)) = open
        && number == named
    {
        return None;
    }
    let words = answer_refusal(target, named, open);
    Some(fit(&words, usize::try_from(bound).expect("message bound"), "…".as_bytes()))
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
    let live = match domain.engine.core().tasks.task(child) {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn take_refuses_an_answer_before_the_question_is_asked() {
        let (domain, mut env, task) =
            crate::tests::live_task(None, tasks::Executor::Procedure { connector: 0, code: super::super::CHANGE_CODE });
        env.limits.tasks.message_bytes = 128;
        let mut row = domain.engine.core().tasks.task(task).unwrap().clone();
        let requester = 48;
        row.requester = tasks::Party::Task(requester);
        row.inbox = Box::new([tasks::Word {
            number: 12,
            from: row.requester,
            kind: tasks::MessageKind::Answer { question: 11 },
            words: b"Keep both".as_slice().into(),
            at: env.wall,
            hits: 1,
            eligible: true,
        }]);
        let questions = change::Questions {
            route: change::Route::Up,
            open: Box::new([change::Open {
                asker: 50,
                number: 9,
                line: b"Which one?".as_slice().into(),
                words: b"Which one?".as_slice().into(),
                stands: change::Stands::Taken,
            }]),
            ..change::Questions::default()
        };
        let taken = take(&row, 12, &questions, None, None, &env);
        assert!(taken.answer.is_none());
        let [reply] = taken.replies.as_ref() else { panic!("one reply: {:?}", taken.replies) };
        assert_eq!(reply.target, requester);
        assert!(reply.question.is_none());
        let number = skein_lib::Decimal::of(task);
        let text = skein_lib::bytes::concat(&[
            b"Task ",
            number.as_bytes(),
            b" has no question open to you; read_task on task ",
            number.as_bytes(),
            b" shows its questions.",
        ]);
        assert_eq!(reply.words, text);
    }
}
