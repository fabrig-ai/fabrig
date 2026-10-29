//! Question credits, requester moves and unanswered exits (domain/tasks.md, 5.7 and 7).
use crate::domain::{Domain, publish, record, refused, task_mut};
use crate::{Event, Limits, MessageKind, Party, Refusal, Request, Word};
use alloc::boxed::Box;
use skein_lib::{Env, List, Map, Queue, ReplyTo};

impl Domain {
    /// The credit owner of one question offered in this person's inbox.
    /// The recipient's current requester authorizes its answer, including when
    /// startup enables direct answers for a credit retaining its task answerer.
    #[must_use]
    pub fn question_asker(&self, asked: u64, person: u64, question: u64) -> Option<u64> {
        let recipient = record(self, asked)?;
        if recipient.requester != Party::Person(person) {
            return None;
        }
        for (number, _) in &self.names {
            let row = record(self, *number).expect("live asker");
            for credit in &row.questions {
                if credit.number == question
                    && (credit.asked == Party::Task(asked) || *number == asked && credit.asked == Party::Person(person))
                {
                    return Some(*number);
                }
            }
        }
        None
    }
}

pub(crate) fn person_question(domain: &mut Domain, env: &Env<Limits>, event: Event, out: &mut Queue<Request>) {
    let Event::PersonQuestion { reply_to, project, task, person, word } = event else {
        unreachable!("person question event")
    };
    let Some(recipient) = record(domain, task) else { return refused(reply_to, Some(task), Refusal::Unknown, out) };
    if recipient.requester != Party::Person(person) || word.kind != MessageKind::Question {
        return refused(reply_to, Some(task), Refusal::Reference, out);
    }
    let plan = match crate::inbox::check_message(
        domain,
        &crate::compound::Staged::new(0),
        env,
        crate::inbox::Message { project, number: task, word, authorized: false },
    ) {
        Ok(plan) => plan,
        Err(problem) => return crate::capacity::refused(domain, env, reply_to, *problem, out),
    };
    let source = match plan.prepared.word.from {
        Party::Task(source) => source,
        Party::Person(_) | Party::Deployment { .. } => unreachable!("question source checked"),
    };
    let (task, word) = crate::inbox::apply_message(domain, env, plan, out);
    for credit in &mut task_mut(domain, source).expect("admitted question owner").record.questions {
        if credit.number == word.number {
            credit.answerer = Party::Person(person);
        }
    }
    changed(domain, source);
    out.push(Request::Sent { reply_to, task, word });
}

pub(crate) fn person_answer(domain: &mut Domain, env: &Env<Limits>, event: Event, out: &mut Queue<Request>) {
    let Event::PersonQuestionAnswer { reply_to, project, asked, person, question, mut word, notice } = event else {
        unreachable!("person answer event")
    };
    let Some(asker) = domain.question_asker(asked, person, question) else {
        return refused(reply_to, Some(asked), Refusal::Reference, out);
    };
    let row = record(domain, asker).expect("credit owner live");
    if row.project != project
        || !crate::moving::mutable(&row.phase)
        || word.from != Party::Person(person)
        || word.kind != (MessageKind::Answer { question })
        || word.words.is_empty()
        || word.words.len() > usize::try_from(env.limits.message_bytes).expect("byte bound")
    {
        return refused(reply_to, Some(asked), Refusal::State, out);
    }
    let mut selected = None;
    for credit in &row.questions {
        if credit.number == question {
            selected = Some(credit.clone());
        }
    }
    let credit = selected.expect("selected credit");
    word.eligible = credit.asked == row.requester;
    word.from = row.requester;
    let Some(prepared) = crate::inbox::prepare(domain, &env.limits, asker, word, crate::inbox::Admission::RoomKept)
    else {
        return refused(reply_to, Some(asker), Refusal::Read, out);
    };
    let (task, word) = crate::inbox::apply_message(
        domain,
        env,
        crate::inbox::MessagePlan { number: asker, prepared, answer: Some(question) },
        out,
    );
    // The inbox owner retains the answer's location even when its sender is a
    // person after a move. Its reserved notice replaces that open position.
    let heard = crate::AnswerLine { task: asker, position: record(domain, asker).expect("answered asker").activity };
    crate::positions::sent_answer(domain, env, Party::Task(asked), question, heard, out);
    if asked != asker
        && let Some((previous, word)) =
            append_notice(domain, env, asked, notice, MessageKind::QuestionAnswered { question }, out)
    {
        changed(domain, asked);
        crate::wake::after_message(domain, env, asked, previous, word, out);
    }
    out.push(Request::Sent { reply_to, task, word });
}

pub(crate) fn cancel(
    domain: &mut Domain,
    env: &Env<Limits>,
    to: ReplyTo,
    task: u64,
    question: u64,
    out: &mut Queue<Request>,
) {
    let Some(row) = record(domain, task) else {
        return refused(to, Some(task), Refusal::Unknown, out);
    };
    let mut selected = None;
    for credit in &row.questions {
        if credit.number == question {
            selected = Some(credit.clone());
        }
    }
    let Some(credit) = selected else {
        return refused(to, Some(task), Refusal::Reference, out);
    };
    forget_credit(domain, env, task, question, out);
    match credit.asked {
        Party::Task(asked) => {
            forget_received(domain, env, asked, question, out);
        }
        Party::Person(_) => forget_received(domain, env, task, question, out),
        Party::Deployment { .. } => unreachable!("question recipient is a task or person"),
    }
    owe(domain, env, task, question);
    out.push(Request::Done { reply_to: to });
}

pub(crate) fn forget_credit(
    domain: &mut Domain,
    env: &Env<Limits>,
    task: u64,
    question: u64,
    _out: &mut Queue<Request>,
) {
    let row = task_mut(domain, task).expect("credit owner live");
    let mut credits = List::with_capacity(env.limits.inbox_messages);
    for credit in &row.record.questions {
        if credit.number != question {
            credits.push(credit.clone()).expect("credit subset");
        }
    }
    row.record.questions = credits.into_boxed();
    changed(domain, task);
}

pub(crate) fn forget_received(
    domain: &mut Domain,
    env: &Env<Limits>,
    task: u64,
    question: u64,
    _out: &mut Queue<Request>,
) {
    let Some(row) = task_mut(domain, task) else {
        return;
    };
    let mut lines = List::with_capacity(env.limits.inbox_messages);
    for line in &row.record.question_lines {
        if line.message != question {
            lines.push(*line).expect("question subset");
        }
    }
    row.record.question_lines = lines.into_boxed();
    changed(domain, task);
}

pub(crate) fn ended(
    domain: &mut Domain,
    env: &Env<Limits>,
    task: u64,
    notices: &[crate::QuestionNotice],
    out: &mut Queue<Request>,
) {
    assert!(
        notices.len() <= usize::try_from(env.limits.inbox_messages).expect("question bound"),
        "bounded notice batch"
    );
    let mut last = None;
    for notice in notices {
        last = append_notice(
            domain,
            env,
            task,
            notice.message,
            MessageKind::QuestionEnded { question: notice.question },
            out,
        );
    }
    if let Some((previous, word)) = last {
        changed(domain, task);
        crate::wake::after_message(domain, env, task, previous, word, out);
    }
}

fn append_notice(
    domain: &mut Domain,
    env: &Env<Limits>,
    task: u64,
    message: u64,
    kind: MessageKind,
    out: &mut Queue<Request>,
) -> Option<(Option<u64>, Word)> {
    let row = record(domain, task)?;
    if !crate::moving::mutable(&row.phase) {
        return None;
    }
    let word = Word {
        number: message,
        from: Party::Task(task),
        words: if kind == MessageKind::QuestionOwed { Box::from(OWED_WORDS) } else { Box::new([]) },
        kind,
        at: env.wall,
        hits: 1,
        eligible: true,
    };
    let admission = if word.kind == MessageKind::QuestionOwed {
        crate::inbox::Admission::Merged
    } else {
        crate::inbox::Admission::RoomKept
    };
    let prepared = crate::inbox::prepare(domain, &env.limits, task, word, admission)
        .expect("question reserved its terminal notice");
    let row = task_mut(domain, task).expect("notice owner live");
    row.record.last_message = message;
    row.record.inbox = prepared.inbox;
    crate::activity::message(&mut row.record, &prepared.activity_word, env.limits.inbox_messages, out);
    Some((prepared.previous, prepared.word))
}

/// Move's question decisions stay with the child that owns their credits.
pub(crate) fn moved(domain: &mut Domain, env: &Env<Limits>, number: u64, person: u64, out: &mut Queue<Request>) {
    // A received question follows the current person's inbox. Task answerers
    // keep their relay form until the application enables direct answers.
    let mut sources = List::uncapped();
    for (source, _) in &domain.names {
        sources.push(*source).expect("uncapped question owners");
    }
    for source in sources.into_boxed() {
        let mut updated = false;
        for credit in &mut task_mut(domain, source).expect("question owner live").record.questions {
            if credit.asked == Party::Task(number)
                && let Party::Person(_) = credit.answerer
            {
                credit.answerer = Party::Person(person);
                updated = true;
            }
        }
        if updated {
            changed(domain, source);
        }
    }
    let credits = record(domain, number).expect("moving asker live").questions.clone();
    for credit in &mut task_mut(domain, number).expect("moving asker live").record.questions {
        credit.answerer = Party::Person(person);
        credit.asked = Party::Person(person);
    }
    for credit in credits {
        match credit.asked {
            Party::Task(asked) => {
                forget_received(domain, env, asked, credit.number, out);
                let row = task_mut(domain, number).expect("moved asker live");
                row.record.last_message = row.record.last_message.max(credit.number);
                crate::positions::question(&mut row.record, credit.number, credit.line, env.limits.inbox_messages);
                changed(domain, number);
            }
            Party::Person(_) | Party::Deployment { .. } => {}
        }
    }
}

/// Coalesce all credit-only publications and terminal routing in one task step.
pub(crate) fn changed(domain: &mut Domain, task: u64) {
    for old in &domain.question_changes {
        if *old == task {
            return;
        }
    }
    domain.question_changes.push(task).expect("uncapped dirty rows");
}

pub(crate) fn published(domain: &mut Domain, task: u64) {
    for at in 0..domain.question_changes.len() {
        let named = domain.question_changes.get_mut(at).expect("dirty row position");
        if *named == task {
            *named = 0;
        }
    }
}

pub(crate) fn owe(domain: &mut Domain, _env: &Env<Limits>, task: u64, question: u64) {
    if !domain.question_ends.contains_key(&task) {
        domain.question_ends.insert(task, List::uncapped()).expect("uncapped task batches");
    }
    domain.question_ends.get_mut(&task).expect("question batch").push(question).expect("retired credits were bounded");
}

pub(crate) fn flush(domain: &mut Domain, env: &Env<Limits>, output_start: u32, out: &mut Queue<Request>) {
    let changed_rows = !domain.question_changes.is_empty();
    let changed = core::mem::replace(&mut domain.question_changes, List::uncapped());
    for task in changed.into_boxed() {
        if record(domain, task).is_some() {
            publish(domain, env, task, out);
        }
    }
    let mut names = List::uncapped();
    for (task, _) in &domain.question_ends {
        names.push(*task).expect("bounded batches");
    }
    for task in names.into_boxed() {
        let questions = domain.question_ends.remove(&task).expect("named batch");
        if let Some(row) = record(domain, task)
            && crate::moving::mutable(&row.phase)
        {
            out.push(Request::QuestionEndedDue { task, questions: questions.into_boxed() });
        }
    }
    if changed_rows {
        coalesce(output_start, out);
    }
}

/// One final live row per task in this decision, including Move's funding and
/// topology changes. Earlier caller output and immutable history stay ordered.
fn coalesce(output_start: u32, out: &mut Queue<Request>) {
    let mut last = Map::uncapped();
    for (position, request) in out.iter().enumerate() {
        let position = u32::try_from(position).expect("queue position fits");
        if position >= output_start {
            let Request::Save { record: crate::Stored::Live(row) } = request else { continue };
            let _old = last.insert(row.number, position).expect("uncapped publication index");
        }
    }
    let count = out.len();
    for position in 0..count {
        let request = out.pop().expect("entry output count");
        if position < output_start {
            out.push(request);
            continue;
        }
        let Request::Save { record: crate::Stored::Live(row) } = &request else {
            out.push(request);
            continue;
        };
        if last.get(&row.number) == Some(&position) {
            out.push(request);
        }
    }
}

/// The reminder is durable message text, carried unchanged to the run.
pub(crate) const OWED_WORDS: &[u8] =
    b"You owe your requester a question: ask your plan with message, as a question, and wait.";

pub(crate) fn owed(domain: &mut Domain, env: &Env<Limits>, task: u64, message: u64, out: &mut Queue<Request>) {
    let Some(row) = record(domain, task) else { return };
    if !row.spec.asks || row.asked_requester || row.phase != crate::Phase::Active(crate::Active::Idle) {
        return;
    }
    if let Some((previous, word)) = append_notice(domain, env, task, message, MessageKind::QuestionOwed, out) {
        changed(domain, task);
        crate::wake::after_message(domain, env, task, previous, word, out);
    }
}
