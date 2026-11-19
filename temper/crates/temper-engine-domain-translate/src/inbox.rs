//! Live messages and starting inboxes share one envelope (temper/docs/design/domain/agent/README.md, 4).
use alloc::boxed::Box;
use jig_domain_host as host;
use jig_domain_tasks as tasks;
use skein_lib::{Decimal, Token, Writer};

/// One committed inbox word for its live host, its kind kept in its rendered line.
#[must_use]
pub fn message(run: Token, attempt: Token, word: &tasks::Word) -> host::LinkDown {
    host::LinkDown::Message {
        run,
        attempt,
        name: Token::new(word.number),
        sender: inbox_sender(word),
        words: inbox_words(run.raw(), word),
        finish_requested: word.finish_requested(),
    }
}

/// Preserve the task-owned message kind when translating its opaque words,
/// for a live run's message and a starting run's inbox alike. `task` is
/// the receiving task, so an answer notice can distinguish its delegate's question.
#[must_use]
pub fn inbox_words(task: u64, word: &tasks::Word) -> Box<[u8]> {
    if let tasks::MessageKind::SettingsDecision { outcome, .. } = word.kind {
        return settings_decision_words(outcome, &word.words);
    }
    if let tasks::MessageKind::Escalation { task, revision, .. } = word.kind {
        return escalation_words(task, revision, &word.words);
    }

    let (label, number) = match word.kind {
        tasks::MessageKind::QuestionAnswered { question } => {
            if let tasks::Party::Task(asker) = word.from
                && asker != task
            {
                return delegated_answer_notice(asker, question, &word.words);
            }
            return question_notice(b"Person answered question ", question, b" in their inbox", &word.words);
        }
        tasks::MessageKind::QuestionEnded { question } => {
            return question_notice(b"Question ", question, b" ended", &[]);
        }
        tasks::MessageKind::SettingsDecision { .. } => unreachable!("settings decision rendered above"),
        tasks::MessageKind::Result(tasks::ResultKind::Verdict { code, .. }) => {
            return verdict_words(word, code);
        }
        tasks::MessageKind::Result(_) => (RESULT_TASK_LABEL, source_number(word.from)),
        tasks::MessageKind::ProposalDecision { proposal, accepted } => {
            (if accepted { b"Proposal accepted ".as_slice() } else { b"Proposal rejected ".as_slice() }, proposal)
        }
        tasks::MessageKind::PersonStalled { task } => (b"Person task past its waiting deadline ".as_slice(), task),
        tasks::MessageKind::EffectNotMade { entry } => (b"Effect not made ".as_slice(), entry),
        tasks::MessageKind::Met { dependency, removed } => return met_words(dependency, removed, &word.words),
        tasks::MessageKind::Words
        | tasks::MessageKind::Question
        | tasks::MessageKind::Answer { .. }
        | tasks::MessageKind::Notice { .. }
        | tasks::MessageKind::QuestionOwed
        | tasks::MessageKind::NothingAwaited
        | tasks::MessageKind::News { .. }
        | tasks::MessageKind::Timer { .. }
        | tasks::MessageKind::FinishRequested { .. }
        | tasks::MessageKind::Amendment { .. }
        | tasks::MessageKind::Proposal { .. }
        | tasks::MessageKind::Escalation { .. } => return word.words.clone(),
    };
    numbered_words(label, number, &word.words)
}

const RESULT_TASK_LABEL: &[u8] = b"Result from task ";
/// A landing result's live message adds its label, separator and sender
/// (temper/docs/design/domain/agent/charter.md, 3). Both numbers can be `u64::MAX`.
pub const CHANGE_RESULT_ENVELOPE_BYTES: usize = RESULT_TASK_LABEL
    .len()
    .saturating_add(b"18446744073709551615".len())
    .saturating_add(b":\n".len())
    .saturating_add(b"18446744073709551615".len());

fn numbered_words(label: &[u8], number: u64, words: &[u8]) -> Box<[u8]> {
    let number = Decimal::of(number);
    let bytes = label.len().checked_add(number.as_bytes().len()).expect("bounded message label");
    let bytes = bytes.checked_add(2).expect("bounded message separator");
    let bytes = bytes.checked_add(words.len()).expect("bounded message envelope");
    let mut writer = Writer::new(bytes);
    writer.put(label).expect("measured label");
    writer.put(number.as_bytes()).expect("measured source");
    writer.put(b":\n").expect("measured separator");
    writer.put(words).expect("measured words");
    writer.finish()
}

fn source_number(from: tasks::Party) -> u64 {
    match from {
        tasks::Party::Task(task) => task,
        tasks::Party::Person(person) => person,
        tasks::Party::Deployment { .. } => 0,
    }
}

fn verdict_words(word: &tasks::Word, code: u32) -> Box<[u8]> {
    let reason = question_notice(b"verdict ", u64::from(code), b"", &word.words);
    question_notice(b"Result from task ", source_number(word.from), b"", &reason)
}

fn delegated_answer_notice(asker: u64, question: u64, words: &[u8]) -> Box<[u8]> {
    let asker = Decimal::of(asker);
    let question = Decimal::of(question);
    let separator = if words.is_empty() { b"".as_slice() } else { b":\n".as_slice() };
    let parts = [
        b"Person answered task ".as_slice(),
        asker.as_bytes(),
        b"'s question ",
        question.as_bytes(),
        b" in their inbox; task ",
        asker.as_bytes(),
        b" has the answer and does the work",
        separator,
        words,
    ];
    let mut bytes = 0_usize;
    for part in parts {
        bytes = bytes.checked_add(part.len()).expect("bounded answer notice");
    }
    let mut writer = Writer::new(bytes);
    for part in parts {
        writer.put(part).expect("measured answer notice");
    }
    writer.finish()
}

fn question_notice(prefix: &[u8], question: u64, suffix: &[u8], words: &[u8]) -> Box<[u8]> {
    let separator = if words.is_empty() { b"".as_slice() } else { b":\n".as_slice() };
    let question = Decimal::of(question);
    let bytes = prefix.len().checked_add(question.as_bytes().len()).expect("bounded question label");
    let bytes = bytes.checked_add(suffix.len()).expect("bounded question notice");
    let bytes = bytes.checked_add(separator.len()).expect("bounded message separator");
    let bytes = bytes.checked_add(words.len()).expect("bounded message envelope");
    let mut writer = Writer::new(bytes);
    writer.put(prefix).expect("measured prefix");
    writer.put(question.as_bytes()).expect("measured question");
    writer.put(suffix).expect("measured suffix");
    writer.put(separator).expect("measured separator");
    writer.put(words).expect("measured words");
    writer.finish()
}

fn met_words(dependency: u64, removed: bool, words: &[u8]) -> Box<[u8]> {
    let dependency = Decimal::of(dependency);
    let line = if removed {
        b" was removed from its dependencies; the task can go on".as_slice()
    } else {
        b" is done; the task can go on".as_slice()
    };
    let separator = if words.is_empty() { b"".as_slice() } else { b":\n".as_slice() };
    let bytes = 1_usize.checked_add(dependency.as_bytes().len()).expect("bounded dependency label");
    let bytes = bytes.checked_add(line.len()).expect("bounded dependency notice");
    let bytes = bytes.checked_add(separator.len()).expect("bounded message separator");
    let bytes = bytes.checked_add(words.len()).expect("bounded message envelope");
    let mut writer = Writer::new(bytes);
    writer.put(b"T").expect("measured label");
    writer.put(dependency.as_bytes()).expect("measured dependency");
    writer.put(line).expect("measured notice");
    writer.put(separator).expect("measured separator");
    writer.put(words).expect("measured words");
    writer.finish()
}

/// The sender label and question reference that smith prefixes to the words.
/// Shared by live delivery and the inbox carried at start (domain/agent/README.md, 4).
#[must_use]
pub fn inbox_sender(word: &tasks::Word) -> Box<[u8]> {
    let label = word.from.sender();
    let (kind, question, end, show_answer_form) = match word.kind {
        tasks::MessageKind::Question => {
            let show_answer_form = match word.from {
                tasks::Party::Task(_) => true,
                tasks::Party::Person(_) | tasks::Party::Deployment { .. } => false,
            };
            (b" asks (question ".as_slice(), word.number, b")".as_slice(), show_answer_form)
        }
        tasks::MessageKind::Answer { question } => (b" answers question ".as_slice(), question, b"".as_slice(), false),
        tasks::MessageKind::Words
        | tasks::MessageKind::Result(_)
        | tasks::MessageKind::ProposalDecision { .. }
        | tasks::MessageKind::SettingsDecision { .. }
        | tasks::MessageKind::PersonStalled { .. }
        | tasks::MessageKind::EffectNotMade { .. }
        | tasks::MessageKind::Met { .. }
        | tasks::MessageKind::Notice { .. }
        | tasks::MessageKind::QuestionAnswered { .. }
        | tasks::MessageKind::QuestionOwed
        | tasks::MessageKind::NothingAwaited
        | tasks::MessageKind::QuestionEnded { .. }
        | tasks::MessageKind::News { .. }
        | tasks::MessageKind::Timer { .. }
        | tasks::MessageKind::FinishRequested { .. }
        | tasks::MessageKind::Amendment { .. }
        | tasks::MessageKind::Proposal { .. }
        | tasks::MessageKind::Escalation { .. } => return label,
    };
    let question = Decimal::of(question);
    let [answer, target, form, number]: [&[u8]; 4] = if show_answer_form {
        [b"; answer with message target=", &label, b", form=answer, question=", question.as_bytes()]
    } else {
        [b""; 4]
    };
    let parts: [&[u8]; 8] = [&label, kind, question.as_bytes(), answer, target, form, number, end];
    let mut bytes = 0_usize;
    for part in parts {
        bytes = bytes.checked_add(part.len()).expect("bounded message label");
    }
    let mut writer = Writer::new(bytes);
    for part in parts {
        writer.put(part).expect("measured message label");
    }
    writer.finish()
}

fn escalation_words(task: u64, revision: u64, words: &[u8]) -> Box<[u8]> {
    let task = Decimal::of(task);
    let revision = Decimal::of(revision);
    let parts: [&[u8]; 6] = [
        words,
        b"\nHeld task ",
        task.as_bytes(),
        b", hold revision ",
        revision.as_bytes(),
        b". Use read_task on that task.",
    ];
    let mut size = 0_usize;
    for part in parts {
        size = size.checked_add(part.len()).expect("bounded escalation envelope");
    }
    let mut writer = Writer::new(size);
    for part in parts {
        writer.put(part).expect("sized escalation envelope");
    }
    writer.finish()
}

fn settings_decision_words(outcome: tasks::SettingsOutcome, reason: &[u8]) -> Box<[u8]> {
    let number = Decimal::of(match outcome {
        tasks::SettingsOutcome::Accepted { revision } => u64::from(revision),
        tasks::SettingsOutcome::Stale { newest } => u64::from(newest),
        tasks::SettingsOutcome::Rejected => 0,
    });
    let parts: [&[u8]; 3] = match outcome {
        tasks::SettingsOutcome::Accepted { .. } => [b"accepted: revision ", number.as_bytes(), b" made"],
        tasks::SettingsOutcome::Rejected => [b"rejected: ", reason, b""],
        tasks::SettingsOutcome::Stale { .. } => [b"withdrawn as stale: the newest is ", number.as_bytes(), b""],
    };
    let mut size = 0_usize;
    for part in parts {
        size = size.checked_add(part.len()).expect("bounded inbox words");
    }
    let mut out = Writer::new(size);
    for part in parts {
        out.put(part).expect("measured inbox words");
    }
    out.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    fn rendered_line(from: tasks::Party, kind: tasks::MessageKind) -> Box<[u8]> {
        let word = tasks::Word {
            number: 4,
            from,
            kind,
            words: b"Go ahead?\nKeep the newline.".as_slice().into(),
            at: skein_lib::Wall::EPOCH,
            hits: 1,
            eligible: true,
        };
        let host::LinkDown::Message { sender, words, name, .. } = message(Token::new(1), Token::new(2), &word) else {
            panic!("message translation")
        };
        assert_eq!(name, Token::new(4));
        let bytes = sender.len().checked_add(2).expect("test sender and separator");
        let bytes = bytes.checked_add(words.len()).expect("test line");
        let mut line = Writer::new(bytes);
        line.put(&sender).expect("sender fits");
        line.put(b": ").expect("separator fits");
        line.put(&words).expect("words fit");
        line.finish()
    }

    #[test]
    fn words_render_with_their_sender() {
        assert_eq!(
            rendered_line(tasks::Party::Task(2), tasks::MessageKind::Words).as_ref(),
            b"2: Go ahead?\nKeep the newline."
        );
        assert_eq!(
            rendered_line(tasks::Party::Person(1), tasks::MessageKind::Words).as_ref(),
            b"1: Go ahead?\nKeep the newline."
        );
    }

    #[test]
    fn a_question_renders_its_number_for_the_answer_form() {
        assert_eq!(
            rendered_line(tasks::Party::Task(2), tasks::MessageKind::Question).as_ref(),
            b"2 asks (question 4; answer with message target=2, form=answer, question=4): Go ahead?\nKeep the newline."
        );
    }

    #[test]
    fn non_task_questions_keep_their_number_without_a_task_answer_form() {
        for (party, expected) in [
            (tasks::Party::Person(1), b"1 asks (question 4): Go ahead?\nKeep the newline.".as_slice()),
            (
                tasks::Party::Deployment { project: 1 },
                b"deployment asks (question 4): Go ahead?\nKeep the newline.".as_slice(),
            ),
        ] {
            assert_eq!(rendered_line(party, tasks::MessageKind::Question).as_ref(), expected);
        }
    }

    #[test]
    fn an_answer_renders_the_question_it_answers() {
        assert_eq!(
            rendered_line(tasks::Party::Person(1), tasks::MessageKind::Answer { question: 3 }).as_ref(),
            b"1 answers question 3: Go ahead?\nKeep the newline."
        );
    }

    #[test]
    fn question_notices_render_their_number_and_any_answer_words() {
        let cases: &[(u64, tasks::MessageKind, &[u8], &[u8])] = &[
            (2, tasks::MessageKind::Result(tasks::ResultKind::Verdict { code: 2, fields: Box::new([]) }),
             b"bold", b"Result from task 2:\nverdict 2:\nbold"),
            (2, tasks::MessageKind::Result(tasks::ResultKind::Verdict { code: 2, fields: Box::new([]) }),
             b"", b"Result from task 2:\nverdict 2"),
            (2, tasks::MessageKind::QuestionAnswered { question: 42 }, b"", b"Person answered question 42 in their inbox"),
            (
                2, tasks::MessageKind::QuestionAnswered { question: 42 },
                b"fruit,count\nGo ahead.",
                b"Person answered question 42 in their inbox:\nfruit,count\nGo ahead.",
            ),
            (1, tasks::MessageKind::QuestionAnswered { question: 42 }, b"", b"Person answered task 2's question 42 in their inbox; task 2 has the answer and does the work"),
            (1, tasks::MessageKind::QuestionAnswered { question: 42 }, b"fruit,count\nGo ahead.", b"Person answered task 2's question 42 in their inbox; task 2 has the answer and does the work:\nfruit,count\nGo ahead."),
            (2, tasks::MessageKind::QuestionEnded { question: 42 }, b"", b"Question 42 ended"),
        ];
        for (task, kind, words, expected) in cases {
            let word = tasks::Word {
                number: 7,
                from: tasks::Party::Task(2),
                kind: kind.clone(),
                words: (*words).into(),
                at: skein_lib::Wall::EPOCH,
                hits: 1,
                eligible: true,
            };
            assert_eq!(inbox_words(*task, &word).as_ref(), *expected);
            let host::LinkDown::Message { words: live, .. } = message(Token::new(*task), Token::new(1), &word) else {
                panic!("live message");
            };
            assert_eq!(live.as_ref(), *expected);
        }
    }

    #[test]
    fn a_landing_result_fits_its_live_label_and_sender_at_the_longest_task_number() {
        const RESULT_BYTES: usize = 10_660; // The stock's result and hub event bound.
        const RESULT_WORDS: usize = 10_601; // 59 bytes for the label, two u64s and separator.
        // Both envelope rows measured 3.8 ms CPU together.
        for (source, bytes) in [(1, 10_622), (u64::MAX, RESULT_BYTES)] {
            let word = tasks::Word {
                number: 7,
                from: tasks::Party::Task(source),
                kind: tasks::MessageKind::Result(tasks::ResultKind::Change { connector: 0, kind: 1, resource: 1 }),
                words: Box::new([b'r'; RESULT_WORDS]),
                at: skein_lib::Wall::EPOCH,
                hits: 1,
                eligible: true,
            };
            let host::LinkDown::Message { words, sender, .. } = message(Token::new(2), Token::new(1), &word) else {
                panic!("live message")
            };
            assert_eq!(words.len() + sender.len(), bytes);
            assert!(bytes <= RESULT_BYTES);
            assert_eq!(RESULT_WORDS + CHANGE_RESULT_ENVELOPE_BYTES, RESULT_BYTES);
        }
    }

    #[test]
    fn met_notices_name_the_dependency_and_preserve_any_words() {
        let cases: &[(bool, &[u8], &[u8])] = &[
            (false, b"", b"T42 is done; the task can go on"),
            (true, b"", b"T42 was removed from its dependencies; the task can go on"),
            (false, b"landed", b"T42 is done; the task can go on:\nlanded"),
            (
                true,
                b"no longer needed",
                b"T42 was removed from its dependencies; the task can go on:\nno longer needed",
            ),
        ];
        for &(removed, words, expected) in cases {
            let word = tasks::Word {
                number: 2,
                from: tasks::Party::Task(7),
                kind: tasks::MessageKind::Met { dependency: 42, removed },
                words: words.into(),
                at: skein_lib::Wall::EPOCH,
                hits: 1,
                eligible: true,
            };
            assert_eq!(inbox_words(2, &word).as_ref(), expected);
        }
    }
}
