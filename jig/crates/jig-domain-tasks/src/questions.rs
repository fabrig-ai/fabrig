//! Question credits, requester moves and unanswered exits (domain/tasks/lifecycle-endings.md, 1 and domain/tasks/changes.md).
use crate::domain::{Domain, publish, record, refused, task_mut};
use crate::{
    EscalationHolder, Limits, MessageKind, Party, QuestionAnswerNotice, QuestionAnswerer, QuestionCredit,
    QuestionOffer, QuestionResolution, Refusal, Request, Word,
};
use alloc::boxed::Box;
use skein_lib::{Decimal, Env, List, Map, Queue, ReplyTo, Writer};

/// Maximum bytes of the answer carried by its requester's notice, including a cut marker.
pub const ANSWER_NOTICE_BYTES: usize = 256;

/// An unanswered received question whose asker still reserves an answer from this task.
/// Words stay on the asker's activity line (domain/tasks/lifecycle-endings.md, 1).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct OpenQuestion {
    pub asker: u64,
    pub number: u64,
    pub line: u64,
}

/// One live question or its kept first answer. A party link may name the
/// original addressee or (for an offer) the asker.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct QuestionTicket {
    pub asker: u64,
    pub asked: u64,
    pub first: QuestionAnswerer,
    pub holder: Option<EscalationHolder>,
    pub by: Option<QuestionAnswerer>,
    pub line: u64,
}

impl QuestionCredit {
    pub(crate) fn ticket(&self, asker: u64) -> QuestionTicket {
        let recipient = match self.asked {
            Party::Task(task) => task,
            Party::Person(_) => asker,
            Party::Deployment { .. } => unreachable!("questions are addressed to tasks or people"),
        };
        let first = match self.answerer {
            Party::Task(task) => QuestionAnswerer::Task(task),
            Party::Person(person) => QuestionAnswerer::Person(person),
            Party::Deployment { .. } => unreachable!("questions are answered by tasks or people"),
        };
        let holder = match &self.offer {
            Some(offer) => Some(offer.holder),
            None => None,
        };
        QuestionTicket { asker, asked: recipient, first, holder, by: None, line: self.line }
    }
}

impl QuestionTicket {
    pub(crate) fn task_may_answer(self, task: u64) -> bool {
        self.first == QuestionAnswerer::Task(task) || self.holder == Some(EscalationHolder::Task(task))
    }
}

/// The bounded reason for an exhausted question wake (domain/tasks/lifecycle.md, 2).
#[must_use]
pub fn open_question_reason(question: OpenQuestion, chain: &[crate::StuckLink], words: &[u8]) -> Box<[u8]> {
    let mut line = words;
    if let Some(at) = skein_lib::bytes::find(words, b"\n") {
        line = words.get(..at).expect("first line in words");
    }
    let mut measure = Writer::measure(skein_lib::usize_of(crate::REASON_BYTES));
    write_reason(&mut measure, question, chain, line);
    let mut writer = Writer::new(measure.written());
    write_reason(&mut writer, question, chain, line);
    writer.finish()
}

fn write_reason(writer: &mut Writer, question: OpenQuestion, chain: &[crate::StuckLink], line: &[u8]) {
    let asker = Decimal::of(question.asker);
    let number = Decimal::of(question.number);
    if !put_reason(writer, &[b"Task ", asker.as_bytes(), b"'s question ", number.as_bytes(), b" is still open"]) {
        return;
    }
    if !chain.is_empty() {
        if !put_reason(writer, &[b", through tasks "]) {
            return;
        }
        let mut comma = b"".as_slice();
        for link in chain {
            let task = Decimal::of(link.task);
            if !put_reason(writer, &[comma, task.as_bytes()]) {
                return;
            }
            if let Some(question) = link.question {
                let number = Decimal::of(question);
                if !put_reason(writer, &[b" (question ", number.as_bytes(), b")"]) {
                    return;
                }
            }
            comma = b", ";
        }
    }
    put_reason(writer, &[b": ", line]);
}

fn put_reason(writer: &mut Writer, parts: &[&[u8]]) -> bool {
    for part in parts {
        let prefix = skein_lib::bytes::utf8_prefix(part, writer.room());
        writer.put(prefix).expect("bounded question reason");
        if prefix.len() < part.len() {
            return false;
        }
    }
    true
}

/// Index only the owner: the credit and received position stay authoritative.
pub(crate) fn index(domain: &mut Domain, number: u64) {
    let id = *domain.names.get(&number).expect("indexed task name");
    let task = domain.tasks.get(id).expect("indexed task");
    if let Some(previous) = task.published.as_deref() {
        if previous.questions == task.record.questions {
            return;
        }
        for credit in &previous.questions {
            domain.question_owners.remove(&credit.number);
        }
    }
    for credit in &task.record.questions {
        domain.question_owners.insert(credit.number, number).expect("bounded question credits");
    }
}

pub(crate) fn unindex(domain: &mut Domain, number: u64) {
    let id = *domain.names.get(&number).expect("indexed task name");
    let task = domain.tasks.get(id).expect("indexed task");
    if let Some(previous) = task.published.as_deref() {
        for credit in &previous.questions {
            domain.question_owners.remove(&credit.number);
        }
    }
}

impl Domain {
    fn open_position(&self, number: u64, position: &crate::QuestionLine) -> Option<OpenQuestion> {
        if position.answer.is_some() {
            return None;
        }
        let asker = *self.question_owners.get(&position.message)?;
        let source = record(self, asker)?;
        for credit in &source.questions {
            if credit.number == position.message && credit.answerer == Party::Task(number) {
                return Some(OpenQuestion { asker, number: credit.number, line: credit.line });
            }
        }
        None
    }

    pub(crate) fn oldest_open_question(&self, row: &crate::TaskRecord) -> Option<OpenQuestion> {
        let mut oldest: Option<OpenQuestion> = None;
        for position in &row.question_lines {
            if let Some(open) = self.open_position(row.number, position) {
                oldest = match oldest {
                    Some(old) if old.number < open.number => Some(old),
                    Some(_) | None => Some(open),
                };
            }
        }
        oldest
    }

    /// Open questions asked of this task, oldest first (domain/tasks/messages.md, 2).
    #[must_use]
    pub fn open_questions(&self, number: u64) -> List<OpenQuestion> {
        let Some(row) = record(self, number) else { return List::with_capacity(0) };
        let capacity = u32::try_from(row.question_lines.len()).expect("bounded received questions");
        let mut ordered = Map::with_capacity(capacity);
        for position in &row.question_lines {
            if let Some(open) = self.open_position(number, position) {
                ordered.insert(open.number, open).expect("one position per globally numbered question");
            }
        }
        let mut questions = List::with_capacity(ordered.len());
        for (_, question) in &ordered {
            questions.push(*question).expect("received question subset");
        }
        questions
    }

    /// The bounded durable question state, using the link carried by an inbox entry.
    #[must_use]
    pub fn question_ticket(&self, link: u64, question: u64) -> Option<QuestionTicket> {
        let row = record(self, link)?;
        // Offers are linked to the asker; the original inbox entry to its addressee.
        if let Some(ticket) = credit(row, question) {
            return Some(ticket);
        }
        for position in &row.question_lines {
            if position.message == question {
                if let Some(ticket) = settled(row.number, position) {
                    return Some(ticket);
                }
                if position.answer.is_none() {
                    for (number, _) in &self.names {
                        let source = record(self, *number).expect("live question owner");
                        if let Some(ticket) = credit(source, question)
                            && ticket.asked == link
                        {
                            return Some(ticket);
                        }
                    }
                }
                return None;
            }
        }
        self.settled_question(link, question)
    }

    fn settled_question(&self, asker: u64, question: u64) -> Option<QuestionTicket> {
        for (number, _) in &self.names {
            let row = record(self, *number).expect("live question addressee");
            for position in &row.question_lines {
                if position.message == question
                    && let Some(ticket) = settled(*number, position)
                    && ticket.asker == asker
                {
                    return Some(ticket);
                }
            }
        }
        None
    }

    /// The answer form gets only this question's reference, not ordinary messaging rights.
    #[must_use]
    pub fn question_answer_reference(&self, asker: u64, task: u64, question: u64) -> bool {
        if let Some(row) = record(self, asker)
            && let Some(ticket) = credit(row, question)
            && ticket.task_may_answer(task)
        {
            return true;
        }
        self.late_task_answer(asker, task, question).is_some()
    }

    pub(crate) fn late_task_answer(&self, asker: u64, task: u64, question: u64) -> Option<Refusal> {
        if let Some(ticket) = self.settled_question(asker, question)
            && ticket.task_may_answer(task)
            && let Some(by) = ticket.by
        {
            return Some(Refusal::QuestionAnswered { question, by });
        }
        None
    }
}

fn credit(row: &crate::TaskRecord, question: u64) -> Option<QuestionTicket> {
    for credit in &row.questions {
        if credit.number == question {
            return Some(credit.ticket(row.number));
        }
    }
    None
}

fn settled(asked: u64, line: &crate::QuestionLine) -> Option<QuestionTicket> {
    let answer = line.answer?;
    let resolution = line.resolution.as_ref()?;
    Some(QuestionTicket {
        asker: answer.task,
        asked,
        first: resolution.first,
        holder: resolution.holder,
        by: Some(resolution.by),
        line: line.line,
    })
}

fn offered_word(asker: u64, question: u64, offer: &QuestionOffer) -> Option<Word> {
    let relay = offer.relay.as_ref()?;
    Some(Word {
        number: relay.entry,
        from: Party::Task(asker),
        kind: MessageKind::PassedQuestion { question },
        words: relay.words.clone(),
        at: offer.since,
        hits: 1,
        eligible: true,
    })
}

pub(crate) fn waiting_for(domain: &Domain, holder: u64) -> Box<[Word]> {
    let mut entries = List::uncapped();
    for (asker, _) in &domain.names {
        let row = record(domain, *asker).expect("live question owner");
        for credit in &row.questions {
            if let Some(offer) = &credit.offer
                && offer.holder == EscalationHolder::Task(holder)
                && let Some(word) = offered_word(*asker, credit.number, offer)
            {
                entries.push(word).expect("bounded by credits of live tasks");
            }
        }
    }
    if let Some(row) = record(domain, holder) {
        for position in &row.question_lines {
            if let Some(resolution) = &position.resolution
                && let Some(notice) = &resolution.notice
            {
                entries
                    .push(notice_word(position.message, resolution.by, notice))
                    .expect("bounded by question positions");
            }
        }
    }
    entries.into_boxed()
}

pub(crate) fn pass(
    domain: &mut Domain,
    env: &Env<Limits>,
    asker: u64,
    question: u64,
    offer: QuestionOffer,
    out: &mut Queue<Request>,
) {
    let Some(row) = task_mut(domain, asker) else { return };
    for credit in &mut row.record.questions {
        if credit.number != question {
            continue;
        }
        let word = offered_word(asker, question, &offer);
        let holder = offer.holder;
        credit.offer = Some(offer);
        publish(domain, env, asker, out);
        if let EscalationHolder::Task(task) = holder
            && let Some(word) = word
        {
            crate::wake::after_message(domain, env, task, None, word, out);
        }
        return;
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
    let gathering = crate::gathering::executor(recipient.executor);
    let mut plan = match crate::inbox::check_message(
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
    // A gathering keeps the question's activity, not an unread word.
    if gathering {
        let mut kept = List::with_capacity(env.limits.inbox_messages);
        for word in &plan.prepared.inbox {
            if word.number != plan.prepared.word.number {
                kept.push(word.clone()).expect("inbox subset fits");
            }
        }
        plan.prepared.inbox = kept.into_boxed();
    }
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
    let Some(ticket) = domain.question_ticket(recipient, question) else {
        return refused(reply_to, Some(recipient), Refusal::Reference, out);
    };
    let Some(row) = record(domain, recipient) else {
        return refused(reply_to, Some(recipient), Refusal::Reference, out);
    };
    if ticket.asked != recipient || row.requester != Party::Person(person) {
        return refused(reply_to, Some(recipient), Refusal::Reference, out);
    }
    if let Some(by) = ticket.by {
        return refused(reply_to, Some(recipient), Refusal::QuestionAnswered { question, by }, out);
    }
    let Some(row) = record(domain, ticket.asker) else {
        return refused(reply_to, Some(recipient), Refusal::Reference, out);
    };
    if word.from != Party::Person(person) || word.kind != (MessageKind::Answer { question }) {
        return refused(reply_to, Some(recipient), Refusal::State, out);
    }
    let words = answer_notice_words(word.words.clone());
    word.from = row.requester;
    let (task, word) = match party_answer(
        domain,
        env,
        project,
        ticket.asker,
        question,
        word,
        QuestionAnswerer::Person(person),
        false,
        out,
    ) {
        Ok(answer) => answer,
        Err(why) => return refused(reply_to, Some(recipient), why, out),
    };
    if recipient != task
        && !crate::gathering::executor(record(domain, recipient).expect("question recipient live").executor)
        && let Some((previous, word)) = append_notice(
            domain,
            env,
            recipient,
            Notice { message: notice, from: task, kind: MessageKind::QuestionAnswered { question }, words },
            out,
        )
    {
        changed(domain, recipient);
        crate::wake::after_message(domain, env, recipient, previous, word, out);
    }
    out.push(Request::Sent { reply_to, task, word });
}

pub(crate) fn offered_answer(
    domain: &mut Domain,
    env: &Env<Limits>,
    reply_to: ReplyTo,
    project: u32,
    asker: u64,
    mut word: Word,
    out: &mut Queue<Request>,
) {
    let MessageKind::Answer { question } = word.kind else {
        return refused(reply_to, Some(asker), Refusal::Reference, out);
    };
    let Party::Person(person) = word.from else {
        return refused(reply_to, Some(asker), Refusal::Reference, out);
    };
    let Some(row) = record(domain, asker) else {
        return refused(reply_to, Some(asker), Refusal::Unknown, out);
    };
    let Some(ticket) = domain.question_ticket(asker, question) else {
        return refused(reply_to, Some(asker), Refusal::Reference, out);
    };
    let admitted = match ticket.holder {
        Some(EscalationHolder::Person(holder)) => holder == person,
        Some(EscalationHolder::Role { project: held_project, .. }) => held_project == project,
        Some(EscalationHolder::Task(_)) | None => false,
    };
    if !admitted || ticket.asker != asker {
        return refused(reply_to, Some(asker), Refusal::Reference, out);
    }
    if let Some(by) = ticket.by {
        return refused(reply_to, Some(asker), Refusal::QuestionAnswered { question, by }, out);
    }
    word.from = row.requester;
    let (task, word) =
        match party_answer(domain, env, project, asker, question, word, QuestionAnswerer::Person(person), true, out) {
            Ok(answer) => answer,
            Err(why) => return refused(reply_to, Some(asker), why, out),
        };
    out.push(Request::Sent { reply_to, task, word });
}

#[expect(clippy::too_many_arguments, reason = "one answer shares the message plan and durable settlement")]
fn party_answer(
    domain: &mut Domain,
    env: &Env<Limits>,
    project: u32,
    asker: u64,
    question: u64,
    word: Word,
    by: QuestionAnswerer,
    notice: bool,
    out: &mut Queue<Request>,
) -> Result<(u64, Word), Refusal> {
    let row = record(domain, asker).ok_or(Refusal::Unknown)?;
    if row.project != project
        || !crate::moving::mutable(&row.phase)
        || word.words.is_empty()
        || word.words.len() > skein_lib::length::usize_of(env.limits.message_bytes)
    {
        return Err(Refusal::State);
    }
    let ticket = credit(row, question).ok_or(Refusal::Reference)?;
    let mut word = word;
    word.eligible = if ticket.asked == asker {
        ticket.first.party() == row.requester
    } else {
        Party::Task(ticket.asked) == row.requester
    };
    let prepared = crate::inbox::prepare(domain, &env.limits, asker, word, crate::inbox::Admission::RoomKept)
        .ok_or(Refusal::Read)?;
    let answer = crate::inbox::Answering { question, by, notice };
    Ok(crate::inbox::apply_message(
        domain,
        env,
        crate::inbox::MessagePlan { number: asker, prepared, answer: Some(answer), asked_by: None, notice: None },
        out,
    ))
}

fn notice_word(question: u64, by: QuestionAnswerer, notice: &QuestionAnswerNotice) -> Word {
    Word {
        number: notice.number,
        from: by.party(),
        kind: MessageKind::PassedAnswer { question },
        words: notice.words.clone(),
        at: notice.at,
        hits: 1,
        eligible: true,
    }
}

pub(crate) fn settle(
    domain: &mut Domain,
    env: &Env<Limits>,
    credit: &QuestionCredit,
    answer: crate::inbox::Answering,
    heard: crate::AnswerLine,
    word: &Word,
    out: &mut Queue<Request>,
) {
    let ticket = credit.ticket(heard.task);
    let Some(row) = task_mut(domain, ticket.asked) else { return };
    let notice = if answer.notice
        && ticket.first != answer.by
        && ticket.asked != heard.task
        && crate::moving::mutable(&row.record.phase)
        && !crate::gathering::executor(row.record.executor)
    {
        Some(QuestionAnswerNotice { number: word.number, at: word.at, words: answer_notice_words(word.words.clone()) })
    } else {
        None
    };
    let arrival = match &notice {
        Some(notice) => Some(notice_word(answer.question, answer.by, notice)),
        None => None,
    };
    let resolution = QuestionResolution { by: answer.by, first: ticket.first, holder: ticket.holder, notice };
    if crate::positions::settle(&mut row.record, answer.question, heard, resolution) {
        changed(domain, ticket.asked);
        if let Some(word) = arrival {
            crate::wake::after_message(domain, env, ticket.asked, None, word, out);
        }
    }
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
            lines.push(line.clone()).expect("question subset");
        }
    }
    row.record.question_lines = lines.into_boxed();
    changed(domain, task);
    crate::notice_room::forget(domain, &env.limits, task, question);
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
            let Some(key) = live_save(request) else { continue };
            let _old = last.insert(key, position).expect("uncapped publication index");
        }
    }
    let count = out.len();
    for position in 0..count {
        let request = out.pop().expect("entry output count");
        if position < output_start {
            out.push(request);
            continue;
        }
        let key = live_save(&request);
        let Some(key) = key else {
            out.push(request);
            continue;
        };
        if last.get(&key) == Some(&position) {
            out.push(request);
        }
    }
}

fn live_save(request: &Request) -> Option<crate::Key> {
    let Request::Save { record } = request else { return None };
    match record {
        crate::Stored::Live(row) => Some(crate::Key::Live(row.number)),
        crate::Stored::LiveCold { number, .. } => Some(crate::Key::LiveCold(*number)),
        crate::Stored::Summary(_)
        | crate::Stored::Retention(_)
        | crate::Stored::Attempt(_)
        | crate::Stored::Hold(_)
        | crate::Stored::PlacedHold(_)
        | crate::Stored::Update(_)
        | crate::Stored::Result(_)
        | crate::Stored::EndedByParty(_)
        | crate::Stored::EndedByProject(_)
        | crate::Stored::EndedDelegate(_)
        | crate::Stored::Delegate(_)
        | crate::Stored::Activity(_)
        | crate::Stored::PersonProposal(_)
        | crate::Stored::DecidedProposal(_)
        | crate::Stored::ClosedLedger(_)
        | crate::Stored::History(_)
        | crate::Stored::Ended(_)
        | crate::Stored::Stub(_)
        | crate::Stored::Ledger(_)
        | crate::Stored::Writer(_)
        | crate::Stored::Pool(_) => None,
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
