//! Question credits, requester moves and unanswered exits (domain/tasks/lifecycle.md, 8 and domain/tasks/changes.md).
use crate::domain::{Domain, publish, record, refused, task_mut};
use crate::{Limits, MessageKind, Party, Refusal, Request, Word};
use alloc::boxed::Box;
use skein_lib::{Decimal, Env, List, Map, Queue, ReplyTo, Writer};

/// Maximum bytes of the answer carried by its requester's notice, including a cut marker.
pub(crate) const ANSWER_NOTICE_BYTES: usize = 256;

/// An unanswered received question whose asker still reserves an answer from this task.
/// Words stay on the asker's activity line (domain/tasks/lifecycle.md, 8).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct OpenQuestion {
    pub asker: u64,
    pub number: u64,
    pub line: u64,
}

/// The bounded reason for an exhausted question wake (domain/tasks/lifecycle.md, 2).
#[must_use]
pub fn open_question_reason(question: OpenQuestion, words: &[u8]) -> Box<[u8]> {
    let asker = Decimal::of(question.asker);
    let number = Decimal::of(question.number);
    let mut line = words;
    for (at, byte) in words.iter().enumerate() {
        if *byte == b'\n' {
            line = words.get(..at).expect("first line in words");
            break;
        }
    }
    let parts = [b"Task ".as_slice(), asker.as_bytes(), b"'s question ", number.as_bytes(), b" is still open: ", line];
    let bound = skein_lib::usize_of(crate::REASON_BYTES);
    let mut length = 0_usize;
    for part in parts {
        let prefix = skein_lib::bytes::utf8_prefix(part, bound.checked_sub(length).expect("bounded reason"));
        length = length.checked_add(prefix.len()).expect("bounded reason");
        if prefix.len() < part.len() {
            break;
        }
    }
    let mut writer = Writer::new(length);
    for part in parts {
        let prefix = skein_lib::bytes::utf8_prefix(part, writer.room());
        writer.put(prefix).expect("bounded question reason");
        if prefix.len() < part.len() {
            break;
        }
    }
    writer.finish()
}

impl Domain {
    /// Open questions asked of this task, oldest first (domain/tasks/messages.md, 2).
    #[must_use]
    pub fn open_questions(&self, number: u64) -> List<OpenQuestion> {
        let Some(row) = record(self, number) else { return List::with_capacity(0) };
        let capacity = u32::try_from(row.question_lines.len()).expect("bounded received questions");
        let mut ordered = Map::with_capacity(capacity);
        for position in &row.question_lines {
            if position.answer.is_some() {
                continue;
            }
            for (asker, _) in &self.names {
                let source = record(self, *asker).expect("live credit owner");
                for credit in &source.questions {
                    if credit.number == position.message && credit.answerer == Party::Task(number) {
                        ordered
                            .insert(
                                credit.number,
                                OpenQuestion { asker: *asker, number: credit.number, line: credit.line },
                            )
                            .expect("one credit per globally numbered question");
                    }
                }
            }
        }
        let mut questions = List::with_capacity(capacity);
        for (_, question) in &ordered {
            questions.push(*question).expect("received question subset");
        }
        questions
    }

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

pub(crate) fn person_question(
    domain: &mut Domain,
    env: &Env<Limits>,
    reply_to: ReplyTo,
    (project, person): (u32, u64),
    task: u64,
    word: Word,
    out: &mut Queue<Request>,
) {
    let Some(recipient) = record(domain, task) else { return refused(reply_to, Some(task), Refusal::Unknown, out) };
    if recipient.requester != Party::Person(person) || word.kind != MessageKind::Question {
        return refused(reply_to, Some(task), Refusal::Reference, out);
    }
    let plan = match crate::inbox::check_message(
        domain,
        &crate::compound::Staged::new(0),
        env,
        project,
        crate::Message { task, word },
        false,
    ) {
        Ok(plan) => plan,
        Err(problem) => return crate::capacity::refused(domain, env, reply_to, *problem, out),
    };
    let source = plan.asked_by;
    let (task, word) = crate::inbox::apply_message(domain, env, plan, out);
    if let Some(source) = source {
        for credit in &mut task_mut(domain, source).expect("admitted question owner").record.questions {
            if credit.number == word.number {
                credit.answerer = Party::Person(person);
            }
        }
        changed(domain, source);
    }
    out.push(Request::Sent { reply_to, task, word });
}

pub(crate) struct Answering {
    pub(crate) project: u32,
    pub(crate) recipient: u64,
    pub(crate) person: u64,
    pub(crate) question: u64,
    pub(crate) word: Word,
    pub(crate) notice: u64,
}

pub(crate) fn person_answer(
    domain: &mut Domain,
    env: &Env<Limits>,
    reply_to: ReplyTo,
    answering: Answering,
    out: &mut Queue<Request>,
) {
    let Answering { project, recipient, person, question, mut word, notice } = answering;
    let Some(asker) = domain.question_asker(recipient, person, question) else {
        return refused(reply_to, Some(recipient), Refusal::Reference, out);
    };
    let row = record(domain, asker).expect("credit owner live");
    if row.project != project
        || !crate::moving::mutable(&row.phase)
        || word.from != Party::Person(person)
        || word.kind != (MessageKind::Answer { question })
        || word.words.is_empty()
        || word.words.len() > skein_lib::length::usize_of(env.limits.message_bytes)
    {
        return refused(reply_to, Some(recipient), Refusal::State, out);
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
    let words = answer_notice_words(word.words.clone());
    let Some(prepared) = crate::inbox::prepare(domain, &env.limits, asker, word, crate::inbox::Admission::RoomKept)
    else {
        return refused(reply_to, Some(asker), Refusal::Read, out);
    };
    let (task, word) = crate::inbox::apply_message(
        domain,
        env,
        crate::inbox::MessagePlan { number: asker, prepared, answer: Some(question), asked_by: None },
        out,
    );
    // The inbox owner retains the answer's location even when its sender is a
    // person after a move. Its reserved notice replaces that open position.
    let heard = crate::AnswerLine { task: asker, position: record(domain, asker).expect("answered asker").activity };
    crate::positions::sent_answer(domain, env, Party::Task(recipient), question, heard, out);
    if recipient != asker
        && let Some((previous, word)) = append_notice(
            domain,
            env,
            recipient,
            Notice { message: notice, from: asker, kind: MessageKind::QuestionAnswered { question }, words },
            out,
        )
    {
        changed(domain, recipient);
        crate::wake::after_message(domain, env, recipient, previous, word, out);
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
    forget_credit(domain, env, task, question);
    match credit.asked {
        Party::Task(asked) => {
            forget_received(domain, env, asked, question);
        }
        Party::Person(_) => forget_received(domain, env, task, question),
        Party::Deployment { .. } => unreachable!("question recipient is a task or person"),
    }
    owe(domain, task, question);
    out.push(Request::Done { reply_to: to });
}

pub(crate) fn forget_credit(domain: &mut Domain, env: &Env<Limits>, task: u64, question: u64) {
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

pub(crate) fn forget_received(domain: &mut Domain, env: &Env<Limits>, task: u64, question: u64) {
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
    assert!(notices.len() <= skein_lib::length::usize_of(env.limits.inbox_messages), "bounded notice batch");
    let mut last = None;
    for notice in notices {
        last = append_notice(
            domain,
            env,
            task,
            Notice {
                message: notice.message,
                from: task,
                kind: MessageKind::QuestionEnded { question: notice.question },
                words: Box::new([]),
            },
            out,
        );
    }
    if let Some((previous, word)) = last {
        changed(domain, task);
        crate::wake::after_message(domain, env, task, previous, word, out);
    }
}

struct Notice {
    message: u64,
    from: u64,
    kind: MessageKind,
    words: Box<[u8]>,
}

fn append_notice(
    domain: &mut Domain,
    env: &Env<Limits>,
    task: u64,
    notice: Notice,
    out: &mut Queue<Request>,
) -> Option<(Option<u64>, Word)> {
    let Notice { message, from, kind, words } = notice;
    let row = record(domain, task)?;
    if !crate::moving::mutable(&row.phase) {
        return None;
    }
    let word = Word { number: message, from: Party::Task(from), words, kind, at: env.wall, hits: 1, eligible: true };
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
pub(crate) fn moved(domain: &mut Domain, env: &Env<Limits>, number: u64, person: u64) {
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
                forget_received(domain, env, asked, credit.number);
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

pub(crate) fn owe(domain: &mut Domain, task: u64, question: u64) {
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
    if let Some((previous, word)) = append_notice(
        domain,
        env,
        task,
        Notice { message, from: task, kind: MessageKind::QuestionOwed, words: Box::from(OWED_WORDS) },
        out,
    ) {
        changed(domain, task);
        crate::wake::after_message(domain, env, task, previous, word, out);
    }
}

fn answer_notice_words(words: Box<[u8]>) -> Box<[u8]> {
    if words.len() <= ANSWER_NOTICE_BYTES {
        return words;
    }
    let mut keep = skein_lib::bytes::utf8_prefix(&words, ANSWER_NOTICE_BYTES);
    for _ in 0..ANSWER_NOTICE_BYTES {
        let lost = words.len().checked_sub(keep.len()).expect("prefix within answer");
        let digits = Decimal::of(u64::try_from(lost).expect("bounded answer"));
        let bytes = keep.len().checked_add(b"\n[ bytes cut]\n".len()).expect("bounded notice marker");
        let bytes = bytes.checked_add(digits.as_bytes().len()).expect("bounded lost count");
        if bytes <= ANSWER_NOTICE_BYTES {
            let mut writer = Writer::new(bytes);
            writer.put(keep).expect("measured answer prefix");
            writer.put(b"\n[").expect("measured marker");
            writer.put(digits.as_bytes()).expect("measured lost count");
            writer.put(b" bytes cut]\n").expect("measured marker");
            return writer.finish();
        }
        keep = skein_lib::bytes::utf8_prefix(keep, keep.len().checked_sub(1).expect("notice has prefix room"));
    }
    unreachable!("the fixed notice bound holds its cut marker");
}
