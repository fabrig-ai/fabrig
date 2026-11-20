//! The core's bounded task, dependency, attempt and transcript sections
//! (domain/engine/briefs.md, section 1).

pub use jig_domain_core_vocabulary::runs::brief_text::*;

use alloc::boxed::Box;
use jig_domain_notes as notes;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_lib::{Decimal, List, Writer, length::len_u64};

use crate::{Core, RetainedResult, Transcript};

/// Render the bounded note index by durable name so a run can recall entries.
#[must_use]
pub fn note_index_text(lines: &List<notes::Line>, more: u32, budget: u32) -> Box<[u8]> {
    let limit = skein_lib::length::usize_of(budget);
    let mut kept = 0_u32;
    let mut length = 0_usize;
    for line in lines {
        let name = Decimal::of(line.name);
        let revision = Decimal::of(u64::from(line.revision));
        let bytes = name
            .as_bytes()
            .len()
            .saturating_add(revision.as_bytes().len())
            .saturating_add(line.description.len())
            .saturating_add(b" r: \n".len());
        if length.saturating_add(bytes).saturating_add(32) > limit {
            break;
        }
        length = length.saturating_add(bytes);
        kept = kept.checked_add(1).expect("bounded note lines");
    }
    let omitted = u64::from(more).saturating_add(u64::from(lines.len().saturating_sub(kept)));
    let marker = Decimal::of(omitted);
    let marker_length = b"[more: ".len().saturating_add(marker.as_bytes().len()).saturating_add(b"]\n".len());
    let show_more = omitted > 0 && length.saturating_add(marker_length) <= limit;
    if show_more {
        length = length.saturating_add(marker_length);
    }
    let show_empty = kept == 0 && omitted == 0 && length.saturating_add(b"No notes\n".len()) <= limit;
    if show_empty {
        length = length.saturating_add(b"No notes\n".len());
    }
    let mut writer = Writer::new(length);
    let mut at = 0_u32;
    for line in lines {
        if at >= kept {
            break;
        }
        let name = Decimal::of(line.name);
        let revision = Decimal::of(u64::from(line.revision));
        writer.put(name.as_bytes()).expect("measured note name");
        writer.put(b" r").expect("measured revision label");
        writer.put(revision.as_bytes()).expect("measured revision");
        writer.put(b": ").expect("measured separator");
        writer.put(&line.description).expect("measured description");
        writer.put(b"\n").expect("measured newline");
        at = at.checked_add(1).expect("bounded note lines");
    }
    if show_more {
        writer.put(b"[more: ").expect("measured marker");
        writer.put(marker.as_bytes()).expect("measured omitted count");
        writer.put(b"]\n").expect("measured marker end");
    } else if show_empty {
        writer.put(b"No notes\n").expect("measured empty index");
    }
    writer.finish()
}

/// A bounded fragment and the amount its root renderer omitted.
#[derive(Debug)]
struct BriefFragment {
    bytes: Box<[u8]>,
    left: u64,
}

/// The result of reading a root-owned part of a task.
enum BriefRead {
    Got(Box<[BriefFragment]>),
    Failed,
}

#[must_use]
pub fn read_brief_part(core: &Core, limits: BriefTextLimits, task: u64, part: BriefPart) -> Option<Box<[u8]>> {
    let tail = match part {
        BriefPart::TranscriptTail => true,
        BriefPart::Spec
        | BriefPart::Waiting { .. }
        | BriefPart::Dependencies
        | BriefPart::Delegates
        | BriefPart::Attempts
        | BriefPart::Preferences
        | BriefPart::Now => false,
    };
    let read = task_section(core, task, part, limits.parts, limits.read_bytes);
    let parts = match read {
        BriefRead::Got(parts) => parts,
        BriefRead::Failed => return None,
    };
    let mut length = 0_usize;
    let mut ended_line = true;
    for part in &parts {
        length = length.checked_add(part.bytes.len())?;
        if let Some(last) = part.bytes.last() {
            ended_line = *last == b'\n';
        }
        if !tail && part.left > 0 {
            length = length
                .checked_add(usize::from(!tail && !ended_line))?
                .checked_add(1)?
                .checked_add(Decimal::of(part.left).as_bytes().len())?
                .checked_add(b" bytes cut]\n".len())?;
            ended_line = true;
        }
    }
    if length > usize::try_from(limits.brief_bytes).ok()? {
        return None;
    }
    let mut writer = Writer::new(length);
    let mut ended_line = true;
    for part in parts {
        writer.put(&part.bytes).expect("measured part");
        if let Some(last) = part.bytes.last() {
            ended_line = *last == b'\n';
        }
        if !tail && part.left > 0 {
            if !ended_line {
                writer.put(b"\n").expect("measured break");
            }
            writer.put(b"[").expect("measured cut marker");
            writer.put(Decimal::of(part.left).as_bytes()).expect("measured lost count");
            writer.put(b" bytes cut]\n").expect("measured cut marker");
            ended_line = true;
        }
    }
    Some(writer.finish())
}

fn text_part(first: &[u8], second: &[u8], third: &[u8], fourth: &[u8], available: u32) -> BriefFragment {
    let total = first
        .len()
        .checked_add(second.len())
        .expect("bounded part bytes")
        .checked_add(third.len())
        .expect("bounded part bytes")
        .checked_add(fourth.len())
        .expect("bounded part bytes");
    let wanted = total.min(skein_lib::length::usize_of(available));
    let mut keep = 0_usize;
    for source in [first, second, third, fourth] {
        let demand = wanted.checked_sub(keep).expect("prefix within demand");
        let length = skein_lib::bytes::utf8_prefix(source, demand).len();
        keep = keep.checked_add(length).expect("bounded part prefix");
        if length < source.len() {
            break;
        }
    }
    let mut text = Writer::new(keep);
    for source in [first, second, third, fourth] {
        let bytes = skein_lib::bytes::utf8_prefix(source, text.room());
        text.put(bytes).expect("measured exact text part");
        if bytes.len() < source.len() {
            break;
        }
    }
    BriefFragment {
        bytes: text.finish(),
        left: u64::try_from(total.checked_sub(keep).expect("prefix in part")).expect("usize fits u64"),
    }
}

fn task_section(core: &Core, task: u64, part: BriefPart, parts: u32, bytes: u32) -> BriefRead {
    if parts == 0 {
        return BriefRead::Failed;
    }
    match part {
        BriefPart::Spec => match core.contexts.get(&task) {
            Some(context) => task_read(context, &core.people, parts, bytes),
            None => BriefRead::Failed,
        },
        BriefPart::Waiting { carried } => waiting_read(core, task, carried, bytes),
        BriefPart::Delegates => match core.contexts.get(&task) {
            Some(context) => delegates_read(&context.delegates, bytes),
            None => BriefRead::Failed,
        },
        BriefPart::Dependencies => match core.dependency_results.get(&task) {
            Some(results) => match core.contexts.get(&task) {
                Some(context) => dependency_read(results, &context.added, bytes),
                None => BriefRead::Failed,
            },
            None => BriefRead::Failed,
        },
        BriefPart::Attempts => match core.contexts.get(&task) {
            Some(context) => attempt_read(context.tries, context.invalid_result, bytes),
            None => BriefRead::Failed,
        },
        BriefPart::Preferences => match crate::routing::run::preferences::text(core, task) {
            Some(text) => BriefRead::Got(Box::new([text_part(&text, b"", b"", b"", bytes)])),
            None => BriefRead::Failed,
        },
        BriefPart::Now => match crate::routing::run::now::text(core, task) {
            Some(text) => BriefRead::Got(Box::new([text_part(&text, b"", b"", b"", bytes)])),
            None => BriefRead::Failed,
        },
        BriefPart::TranscriptTail => match core.transcripts.get(&task) {
            Some(transcript) => tail_read(transcript, bytes),
            None => BriefRead::Failed,
        },
    }
}

const WAITING_OPENING: &[u8] = b"Questions asked of you, still open, oldest first:\n";
const WORDS_OPENING: &[u8] = b"Words sent to you, not yet recorded, oldest first:\n";

fn question_heading(question: tasks::OpenQuestion) -> Box<[u8]> {
    let asker = Decimal::of(question.asker);
    let number = Decimal::of(question.number);
    let parts = [
        b"Task ".as_slice(),
        asker.as_bytes(),
        b" asks (question ",
        number.as_bytes(),
        b"); answer with message target=",
        asker.as_bytes(),
        b", form=answer, question=",
        number.as_bytes(),
        b":\n",
    ];
    let mut length = 0_usize;
    for part in parts {
        length = length.checked_add(part.len()).expect("bounded heading");
    }
    let mut writer = Writer::new(length);
    for part in parts {
        writer.put(part).expect("measured heading");
    }
    writer.finish()
}

fn words_heading(word: &tasks::Word) -> Box<[u8]> {
    let (label, sender) = match word.from {
        tasks::Party::Person(number) => (b"Person ".as_slice(), number),
        tasks::Party::Task(number) => (b"Task ".as_slice(), number),
        tasks::Party::Deployment { .. } => unreachable!("deployment words are never kept unrecorded"),
    };
    let sender = Decimal::of(sender);
    let number = Decimal::of(word.number);
    let parts = [
        label,
        sender.as_bytes(),
        b" wrote (message ",
        number.as_bytes(),
        b"); once it is in a spec or a note, mark_read messages=[",
        number.as_bytes(),
        b"], or name ",
        number.as_bytes(),
        b" in that note's records:\n",
    ];
    let mut length = 0_usize;
    for part in parts {
        length = length.checked_add(part.len()).expect("bounded words heading");
    }
    let mut writer = Writer::new(length);
    for part in parts {
        writer.put(part).expect("measured words heading");
    }
    writer.finish()
}

fn waiting_read(core: &Core, task: u64, carried: u32, bytes: u32) -> BriefRead {
    let Some(context) = core.contexts.get(&task) else { return BriefRead::Failed };
    let Some(row) = core.tasks.task(task) else { return BriefRead::Failed };
    let offered = crate::routing::run::parts::offered_kinds(
        core.charter_of(task),
        row.instructions.is_some(),
        context.reasons,
        row.flags.ended_by == tasks::EndedBy::Standing,
    );
    let messages = crate::routing::run::preparation::start_messages(context.inbox.clone(), carried, offered);
    let open = core.tasks.open_questions(task);
    let mut questions = List::with_capacity(open.len());
    for question in open.into_boxed() {
        let mut carried = false;
        for message in &messages {
            carried |= message.kind == tasks::MessageKind::Question && message.number == question.number;
        }
        if !carried {
            questions.push(question).expect("open question subset");
        }
    }
    waiting_text(core, &questions, &row.unrecorded, bytes)
}

fn waiting_text(
    core: &Core,
    questions: &List<tasks::OpenQuestion>,
    unrecorded: &[tasks::Word],
    bytes: u32,
) -> BriefRead {
    if questions.is_empty() && unrecorded.is_empty() {
        return BriefRead::Got(Box::new([]));
    }
    let opening = if questions.is_empty() { b"".as_slice() } else { WAITING_OPENING };
    let separator = if questions.is_empty() || unrecorded.is_empty() { b"".as_slice() } else { b"\n" };
    let words_opening = if unrecorded.is_empty() { b"".as_slice() } else { WORDS_OPENING };
    let mut length = opening.len();
    for question in questions {
        let Some(words) = core.open_question_words(*question) else { return BriefRead::Failed };
        length = length
            .checked_add(question_heading(*question).len())
            .expect("bounded headings")
            .checked_add(words.len())
            .expect("bounded question words")
            .checked_add(usize::from(!words.ends_with(b"\n")))
            .expect("question newline");
    }
    length = length
        .checked_add(separator.len())
        .expect("waiting separator")
        .checked_add(words_opening.len())
        .expect("bounded words opening");
    for word in unrecorded {
        length = length
            .checked_add(words_heading(word).len())
            .expect("bounded words headings")
            .checked_add(word.words.len())
            .expect("bounded unrecorded words")
            .checked_add(usize::from(!word.words.ends_with(b"\n")))
            .expect("words newline");
    }
    let bound = length.min(skein_lib::usize_of(bytes));
    let mut kept = skein_lib::bytes::utf8_prefix(opening, bound).len();
    let mut cut = kept < opening.len();
    for question in questions {
        let words = core.open_question_words(*question).expect("measured question words");
        let heading = question_heading(*question);
        let newline = if words.ends_with(b"\n") { b"".as_slice() } else { b"\n".as_slice() };
        for part in [heading.as_ref(), words, newline] {
            if cut {
                break;
            }
            let prefix = skein_lib::bytes::utf8_prefix(part, bound.checked_sub(kept).expect("bounded waiting text"));
            kept = kept.checked_add(prefix.len()).expect("bounded waiting prefix");
            cut = prefix.len() < part.len();
        }
    }
    for part in [separator, words_opening] {
        if cut {
            break;
        }
        let prefix = skein_lib::bytes::utf8_prefix(part, bound.checked_sub(kept).expect("bounded waiting text"));
        kept = kept.checked_add(prefix.len()).expect("bounded waiting prefix");
        cut = prefix.len() < part.len();
    }
    for word in unrecorded {
        let heading = words_heading(word);
        let newline = if word.words.ends_with(b"\n") { b"".as_slice() } else { b"\n".as_slice() };
        for part in [heading.as_ref(), word.words.as_ref(), newline] {
            if cut {
                break;
            }
            let prefix = skein_lib::bytes::utf8_prefix(part, bound.checked_sub(kept).expect("bounded waiting text"));
            kept = kept.checked_add(prefix.len()).expect("bounded waiting prefix");
            cut = prefix.len() < part.len();
        }
    }
    let mut exact = Writer::new(kept);
    exact.put(skein_lib::bytes::utf8_prefix(opening, exact.room())).expect("bounded opening");
    for question in questions {
        let heading = question_heading(*question);
        let words = core.open_question_words(*question).expect("measured words");
        let newline = if words.ends_with(b"\n") { b"".as_slice() } else { b"\n".as_slice() };
        for part in [heading.as_ref(), words, newline] {
            exact.put(skein_lib::bytes::utf8_prefix(part, exact.room())).expect("measured waiting text");
        }
    }
    for part in [separator, words_opening] {
        exact.put(skein_lib::bytes::utf8_prefix(part, exact.room())).expect("measured words opening");
    }
    for word in unrecorded {
        let heading = words_heading(word);
        let newline = if word.words.ends_with(b"\n") { b"".as_slice() } else { b"\n".as_slice() };
        for part in [heading.as_ref(), word.words.as_ref(), newline] {
            exact.put(skein_lib::bytes::utf8_prefix(part, exact.room())).expect("measured unrecorded words");
        }
    }
    BriefRead::Got(Box::new([BriefFragment {
        bytes: exact.finish(),
        left: u64::try_from(length.checked_sub(kept).expect("waiting prefix")).expect("usize fits u64"),
    }]))
}

fn attempt_read(tries: tasks::Tries, invalid: Option<tasks::InvalidResult>, bytes: u32) -> BriefRead {
    let classes: [(&[u8], u32); 7] = [
        (b"transient: ", tries.transient),
        (b"permanent: ", tries.permanent),
        (b"run: ", tries.run),
        (b"agent: ", tries.agent),
        (b"lost: ", tries.lost),
        (b"invalid: ", tries.invalid),
        (b"infrastructure: ", tries.infrastructure),
    ];
    let mut total = 0_usize;
    for (name, count) in classes {
        if count > 0 {
            total = total
                .checked_add(name.len())
                .expect("bounded attempt label")
                .checked_add(Decimal::of(u64::from(count)).as_bytes().len())
                .expect("bounded count")
                .checked_add(1)
                .expect("newline");
        }
    }
    let reason = match invalid {
        Some(tasks::InvalidResult::Form) => b"last invalid result: contract form\n".as_slice(),
        Some(tasks::InvalidResult::Verdict) => b"last invalid result: verdict code\n",
        Some(tasks::InvalidResult::Words) => b"last invalid result: word limit\n",
        Some(tasks::InvalidResult::Change) => b"last invalid result: change identity\n",
        Some(tasks::InvalidResult::Followups) => b"last invalid result: follow-up limit\n",
        None => b"",
    };
    total = total.checked_add(reason.len()).expect("bounded invalid reason");
    let mut writer = Writer::new(total.min(skein_lib::length::usize_of(bytes)));
    for (name, count) in classes {
        if count > 0 {
            for fragment in [name, Decimal::of(u64::from(count)).as_bytes(), b"\n"] {
                let kept = skein_lib::bytes::utf8_prefix(fragment, writer.room());
                writer.put(kept).expect("attempt prefix fits");
            }
        }
    }
    let kept = skein_lib::bytes::utf8_prefix(reason, writer.room());
    writer.put(kept).expect("invalid reason prefix fits");
    let text = writer.finish();
    BriefRead::Got(Box::new([BriefFragment {
        left: len_u64(total.checked_sub(text.len()).expect("written prefix")),
        bytes: text,
    }]))
}

fn delegate_phase(phase: &tasks::Phase) -> &'static [u8] {
    match phase {
        tasks::Phase::Waiting => b"waiting",
        tasks::Phase::Active(active) => match active {
            tasks::Active::Idle => b"idle",
            tasks::Active::Due | tasks::Active::Preparing => b"due",
            tasks::Active::Claimed { .. } | tasks::Active::Running { .. } => b"running",
            tasks::Active::BackingOff { .. } => b"backing off",
        },
        tasks::Phase::Closing(_) => b"closing",
        tasks::Phase::Held { .. } => b"held",
        tasks::Phase::Ended(_) => b"ended",
    }
}

fn delegates_read(delegates: &[tasks::DelegateState], bytes: u32) -> BriefRead {
    let mut total = 0_usize;
    for delegate in delegates {
        total = total
            .checked_add(b"delegate ".len())
            .expect("bounded delegate text")
            .checked_add(Decimal::of(delegate.task).as_bytes().len())
            .expect("bounded delegate ID")
            .checked_add(b": ".len())
            .expect("bounded delegate text")
            .checked_add(delegate_phase(&delegate.phase).len())
            .expect("bounded delegate phase")
            .checked_add(b", spec revision ".len())
            .expect("delegate spec label")
            .checked_add(Decimal::of(delegate.revisions.spec).as_bytes().len())
            .expect("delegate spec revision")
            .checked_add(match delegate.phase {
                tasks::Phase::Held { .. } => b", hold revision "
                    .len()
                    .checked_add(Decimal::of(delegate.revisions.hold).as_bytes().len())
                    .expect("delegate hold revision"),
                tasks::Phase::Waiting | tasks::Phase::Active(_) | tasks::Phase::Closing(_) | tasks::Phase::Ended(_) => {
                    0
                }
            })
            .expect("delegate hold revision")
            .checked_add(1)
            .expect("delegate newline");
    }
    let mut writer = Writer::new(total);
    for delegate in delegates {
        writer.put(b"delegate ").expect("measured delegate text");
        writer.put(Decimal::of(delegate.task).as_bytes()).expect("measured delegate ID");
        writer.put(b": ").expect("measured delegate text");
        writer.put(delegate_phase(&delegate.phase)).expect("measured delegate phase");
        writer.put(b", spec revision ").expect("measured spec label");
        writer.put(Decimal::of(delegate.revisions.spec).as_bytes()).expect("measured spec revision");
        if let tasks::Phase::Held { .. } = delegate.phase {
            writer.put(b", hold revision ").expect("measured hold label");
            writer.put(Decimal::of(delegate.revisions.hold).as_bytes()).expect("measured hold revision");
        }
        writer.put(b"\n").expect("measured delegate newline");
    }
    let text = writer.finish();
    BriefRead::Got(Box::new([text_part(&text, b"", b"", b"", bytes)]))
}

fn result_label(kind: &tasks::ResultKind) -> &'static [u8] {
    match kind {
        tasks::ResultKind::Approved { .. } => b"approved",
        tasks::ResultKind::Job => b"job",
        tasks::ResultKind::NoChange => b"no change",
        tasks::ResultKind::Report => b"report",
        tasks::ResultKind::Verdict { .. } => b"verdict",
        tasks::ResultKind::Change { .. } => b"change",
        tasks::ResultKind::Failed => b"failed",
        tasks::ResultKind::Cancelled => b"cancelled",
    }
}

fn waited_line(added: &tasks::Added, kind: &tasks::ResultKind) -> Box<[u8]> {
    let (party, number) = match added.by {
        tasks::Party::Task(task) => (b"task ".as_slice(), task),
        tasks::Party::Person(person) => (b"person ".as_slice(), person),
        tasks::Party::Deployment { project } => (b"deployment for project ".as_slice(), u64::from(project)),
    };
    let status = match kind {
        tasks::ResultKind::Failed => b"failed".as_slice(),
        tasks::ResultKind::Cancelled => b"cancelled".as_slice(),
        tasks::ResultKind::Approved { .. }
        | tasks::ResultKind::Job
        | tasks::ResultKind::NoChange
        | tasks::ResultKind::Report
        | tasks::ResultKind::Verdict { .. }
        | tasks::ResultKind::Change { .. } => b"done".as_slice(),
    };
    let task = Decimal::of(added.task);
    let at = Decimal::of(added.at.as_nanos());
    let number = Decimal::of(number);
    let fragments = [
        b"waited for task ".as_slice(),
        task.as_bytes(),
        b", added ",
        at.as_bytes(),
        b" by ",
        party,
        number.as_bytes(),
        b": ",
        status,
        b"\n",
    ];
    let mut length = 0;
    for fragment in fragments {
        add_text_len(&mut length, fragment);
    }
    let mut writer = Writer::new(length);
    for fragment in fragments {
        writer.put(fragment).expect("measured added dependency line");
    }
    writer.finish()
}

fn dependency_read(results: &[RetainedResult], added: &[tasks::Added], bytes: u32) -> BriefRead {
    let mut total = 0_usize;
    for result in results {
        for addition in added {
            if addition.task == result.task {
                add_text_len(&mut total, &waited_line(addition, &result.kind));
                break;
            }
        }
        total = total
            .checked_add(b"task ".len())
            .expect("bounded result text")
            .checked_add(Decimal::of(result.task).as_bytes().len())
            .expect("bounded result ID")
            .checked_add(b": ".len())
            .expect("bounded result text")
            .checked_add(result_label(&result.kind).len())
            .expect("bounded result kind")
            .checked_add(result.words.len())
            .expect("bounded result words")
            .checked_add(3)
            .expect("result separators");
        if let tasks::ResultKind::Verdict { code, .. } = &result.kind {
            total = total
                .checked_add(Decimal::of(u64::from(*code)).as_bytes().len())
                .expect("bounded verdict code")
                .checked_add(1)
                .expect("verdict space");
        }
    }
    let mut writer = Writer::new(total);
    for result in results {
        for addition in added {
            if addition.task == result.task {
                writer.put(&waited_line(addition, &result.kind)).expect("measured added dependency line");
                break;
            }
        }
        writer.put(b"task ").expect("measured result text");
        writer.put(Decimal::of(result.task).as_bytes()).expect("measured result ID");
        writer.put(b": ").expect("measured result text");
        writer.put(result_label(&result.kind)).expect("measured result kind");
        if let tasks::ResultKind::Verdict { code, .. } = &result.kind {
            writer.put(b" ").expect("measured verdict space");
            writer.put(Decimal::of(u64::from(*code)).as_bytes()).expect("measured verdict code");
        }
        writer.put(b"\n").expect("measured result newline");
        writer.put(&result.words).expect("measured result words");
        writer.put(b"\n\n").expect("measured result separator");
    }
    let text = writer.finish();
    BriefRead::Got(Box::new([text_part(&text, b"", b"", b"", bytes)]))
}

fn tail_read(transcript: &Transcript, bytes: u32) -> BriefRead {
    let mut kept = transcript.text_kept;
    let mut skipped = 0_u32;
    let mut cut = None;
    for text in &transcript.texts {
        if kept <= u64::from(bytes) {
            break;
        }
        kept = kept.checked_sub(len_u64(text.bytes.len())).expect("counted texts");
        skipped = skipped.checked_add(1).expect("bounded text count");
        if kept < u64::from(bytes) {
            cut = Some(super::transcript_text::cut_transcript_text(
                &text.bytes,
                usize::try_from(u64::from(bytes).checked_sub(kept).expect("room for oldest text"))
                    .expect("bounded read"),
            ));
            break;
        }
    }
    let mut shown = 0_u64;
    let extra = match &cut {
        Some(cut) => cut.bytes.len(),
        None => 0,
    };
    let mut writer =
        Writer::new(usize::try_from(kept).expect("bounded text").checked_add(extra).expect("bounded read"));
    if let Some(cut) = cut {
        writer.put(&cut.bytes).expect("measured oldest text");
        shown = cut.shown;
    }
    let mut position = 0_u32;
    for text in &transcript.texts {
        if position >= skipped {
            writer.put(&text.bytes).expect("measured whole text");
            shown = shown.checked_add(text.shown).expect("loaded text count");
        }
        position = position.checked_add(1).expect("bounded text count");
    }
    BriefRead::Got(Box::new([BriefFragment {
        left: transcript.text_bytes.checked_sub(shown).expect("shown original text was loaded"),
        bytes: writer.finish(),
    }]))
}

fn add_text_len(total: &mut usize, fragment: &[u8]) {
    *total = total.checked_add(fragment.len()).expect("bounded brief text");
}

fn choices_text_len(room: &mut usize, choices: &[tasks::Verdict]) {
    add_text_len(room, b"[Verdict choices:\n");
    for choice in choices {
        for fragment in [
            b"  ".as_slice(),
            Decimal::of(u64::from(choice.code)).as_bytes(),
            b": at most ",
            Decimal::of(u64::from(choice.words)).as_bytes(),
            b" bytes\n",
        ] {
            add_text_len(room, fragment);
        }
    }
    add_text_len(room, b"]\n");
}

fn write_choices(writer: &mut Writer, choices: &[tasks::Verdict]) {
    writer.put(b"[Verdict choices:\n").expect("contract text room");
    for choice in choices {
        writer.put(b"  ").expect("contract text room");
        writer.put(Decimal::of(u64::from(choice.code)).as_bytes()).expect("contract text room");
        writer.put(b": at most ").expect("contract text room");
        writer.put(Decimal::of(u64::from(choice.words)).as_bytes()).expect("contract text room");
        writer.put(b" bytes\n").expect("contract text room");
    }
    writer.put(b"]\n").expect("contract text room");
}

fn contract_text(contract: &tasks::Contract) -> Box<[u8]> {
    let mut room = 0_usize;
    match contract {
        tasks::Contract::Job { bytes } => {
            for fragment in [b"[Job: at most ".as_slice(), Decimal::of(u64::from(*bytes)).as_bytes(), b" bytes]\n"] {
                add_text_len(&mut room, fragment);
            }
        }
        tasks::Contract::Approval { .. } => {
            add_text_len(&mut room, b"[Approve connector state]\n");
        }
        tasks::Contract::Report { words } => {
            for fragment in [b"[Report: at most ".as_slice(), Decimal::of(u64::from(*words)).as_bytes(), b" bytes]\n"] {
                add_text_len(&mut room, fragment);
            }
        }
        tasks::Contract::Verdict { choices } => choices_text_len(&mut room, choices),
        tasks::Contract::Change { connector, kind, words, choices } => {
            for fragment in [
                b"[Change connector ".as_slice(),
                Decimal::of(u64::from(*connector)).as_bytes(),
                b", kind ",
                Decimal::of(u64::from(*kind)).as_bytes(),
                b", at most ",
                Decimal::of(u64::from(*words)).as_bytes(),
                b" bytes]\n",
            ] {
                add_text_len(&mut room, fragment);
            }
            if !choices.is_empty() {
                choices_text_len(&mut room, choices);
            }
        }
    }
    let mut writer = Writer::new(room);
    match contract {
        tasks::Contract::Job { bytes } => {
            writer.put(b"[Job: at most ").expect("contract text room");
            writer.put(Decimal::of(u64::from(*bytes)).as_bytes()).expect("contract text room");
            writer.put(b" bytes]\n").expect("contract text room");
        }
        tasks::Contract::Approval { .. } => {
            writer.put(b"[Approve connector state]\n").expect("contract text room");
        }
        tasks::Contract::Report { words } => {
            writer.put(b"[Report: at most ").expect("contract text room");
            writer.put(Decimal::of(u64::from(*words)).as_bytes()).expect("contract text room");
            writer.put(b" bytes]\n").expect("contract text room");
        }
        tasks::Contract::Verdict { choices } => write_choices(&mut writer, choices),
        tasks::Contract::Change { connector, kind, words, choices } => {
            writer.put(b"[Change connector ").expect("contract text room");
            writer.put(Decimal::of(u64::from(*connector)).as_bytes()).expect("contract text room");
            writer.put(b", kind ").expect("contract text room");
            writer.put(Decimal::of(u64::from(*kind)).as_bytes()).expect("contract text room");
            writer.put(b", at most ").expect("contract text room");
            writer.put(Decimal::of(u64::from(*words)).as_bytes()).expect("contract text room");
            writer.put(b" bytes]\n").expect("contract text room");
            if !choices.is_empty() {
                write_choices(&mut writer, choices);
            }
        }
    }
    writer.finish()
}

/// Render an agent task's spec and, if its result ends it, its contract.
fn task_read(record: &tasks::RunContext, people: &people::Domain, parts: u32, bytes: u32) -> BriefRead {
    if parts == 0 {
        return BriefRead::Failed;
    }
    let contract = match record.ended_by {
        tasks::EndedBy::Result | tasks::EndedBy::Closer(_) => contract_text(&record.contract),
        tasks::EndedBy::Requester | tasks::EndedBy::Standing => Box::new([]),
    };
    let first = text_part(&record.spec.words, b"\n", &contract, b"", bytes);
    let remaining =
        bytes.checked_sub(u32::try_from(first.bytes.len()).expect("bounded task part")).expect("part within read");
    let (requester_label, requester, name) = match record.requester {
        tasks::Party::Person(person) => {
            (b"[Requested by person ".as_slice(), person, people.member_name(record.project, person).unwrap_or(b""))
        }
        tasks::Party::Task(task) => (b"[Requested by task ".as_slice(), task, b"".as_slice()),
        tasks::Party::Deployment { project } => {
            (b"[Requested by deployment for project ".as_slice(), u64::from(project), b"".as_slice())
        }
    };
    let mut name = skein_lib::bytes::copy_of(name);
    for byte in &mut name {
        if *byte < 0x20 || *byte == 0x7f {
            *byte = b' ';
        }
    }
    let separator = if name.is_empty() { b"".as_slice() } else { b", ".as_slice() };
    let task = Decimal::of(record.task);
    let project = Decimal::of(u64::from(record.project));
    let requester = Decimal::of(requester);
    let fragments = [
        b"[This task: ".as_slice(),
        task.as_bytes(),
        b", in project ",
        project.as_bytes(),
        b"]\n",
        requester_label,
        requester.as_bytes(),
        separator,
        &name,
        b"]\n",
    ];
    let mut length = 0;
    for fragment in fragments {
        add_text_len(&mut length, fragment);
    }
    let mut writer = Writer::new(length);
    for fragment in fragments {
        writer.put(fragment).expect("measured task and requester lines");
    }
    let text = writer.finish();
    let part = text_part(&text, b"", b"", b"", remaining);
    let mut gathered = List::with_capacity(parts);
    gathered.push(first).expect("positive part room");
    if parts > 1 {
        gathered.push(part).expect("requester part room");
    } else {
        let last = gathered.get_mut(0).expect("first part");
        last.left = last
            .left
            .checked_add(len_u64(part.bytes.len()))
            .expect("bounded omitted bytes")
            .checked_add(part.left)
            .expect("bounded omitted bytes");
    }
    BriefRead::Got(gathered.into_boxed())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::fixtures::CurrentFixtures;
    use skein_lib::{Env, Queue, ReplyTo, Time, Token, Wall};

    const WORDS_TEXT: &[u8] = b"Words sent to you, not yet recorded, oldest first:\nPerson 42 wrote (message 77); once it is in a spec or a note, mark_read messages=[77], or name 77 in that note's records:\nAlso add a /ready endpoint beside /health.\nTask 9 wrote (message 78); once it is in a spec or a note, mark_read messages=[78], or name 78 in that note's records:\nKeep the caf\xc3\xa9.\nTwo lines.\n";
    const QUESTIONS_TEXT: &[u8] = b"Questions asked of you, still open, oldest first:\nTask 9 asks (question 76); answer with message target=9, form=answer, question=76:\nWhich form?\n";

    fn task_event(core: &mut Core, limits: &crate::Limits, event: tasks::Event) -> Queue<tasks::Request> {
        let mut out = Queue::uncapped();
        tasks::step(
            &mut core.tasks,
            &Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.tasks },
            event,
            &mut out,
        );
        for request in &out {
            if let tasks::Request::Refused { problem, .. } = request {
                panic!("fixture task event refused: {problem:?}");
            }
        }
        out
    }

    fn kept_words(question: bool) -> (Core, crate::Limits) {
        let (mut core, limits) = crate::tests::make_tree();
        core.current.test_charter_mut(1).policy.resume = false;
        let mut rows = List::with_capacity(4);
        for number in [1, 2, 3, 4] {
            let mut row = core.tasks.task(number).expect("fixture task").clone();
            row.root = 9;
            if number == 1 {
                row.number = 9;
                if question {
                    row.questions = Box::new([tasks::QuestionCredit {
                        number: 76,
                        answerer: tasks::Party::Task(2),
                        asked: tasks::Party::Task(2),
                        line: row.activity,
                    }]);
                }
            } else {
                row.requester = tasks::Party::Task(9);
                row.funder = tasks::Funder::Task(9);
            }
            if number == 2 {
                row.unrecorded = Box::new([
                    tasks::Word {
                        number: 77,
                        from: tasks::Party::Person(42),
                        kind: tasks::MessageKind::Words,
                        words: b"Also add a /ready endpoint beside /health.".as_slice().into(),
                        at: Wall::from_nanos(1),
                        hits: 1,
                        eligible: true,
                    },
                    tasks::Word {
                        number: 78,
                        from: tasks::Party::Task(9),
                        kind: tasks::MessageKind::Words,
                        words: b"Keep the caf\xc3\xa9.\nTwo lines.\n".as_slice().into(),
                        at: Wall::from_nanos(2),
                        hits: 1,
                        eligible: true,
                    },
                ]);
                if question {
                    row.question_lines = Box::new([tasks::QuestionLine { message: 76, line: 0, answer: None }]);
                }
            }
            rows.push(row).expect("four fixture rows");
        }
        let mut ledgers = List::with_capacity(core.tasks.funding_records().len());
        for (_, ledger) in core.tasks.funding_records() {
            ledgers.push(*ledger).expect("fixture ledger room");
        }
        crate::tests::fixtures::restore(&mut core, &limits, rows.into_boxed());
        for ledger in ledgers.into_boxed() {
            drop(task_event(&mut core, &limits, tasks::Event::Restore { record: tasks::Stored::Ledger(ledger) }));
        }
        drop(task_event(&mut core, &limits, tasks::Event::Check));
        assert_eq!(core.tasks.restore_failure(), None);
        drop(task_event(&mut core, &limits, tasks::Event::Restored));
        if question {
            assert!(core.remember_question(&tasks::ActivityLine {
                task: 9,
                position: core.tasks.task(9).expect("asker").activity,
                at: Wall::EPOCH,
                line: tasks::Line::Words {
                    message: 76,
                    by: tasks::Party::Task(9),
                    kind: tasks::MessageKind::Question,
                    words: b"Which form?".as_slice().into(),
                },
            }));
        }
        core.contexts.insert(2, core.tasks.run_context(2).expect("waiting task context")).expect("context room");
        (core, limits)
    }

    fn expected_waiting() -> Box<[u8]> {
        let mut writer = Writer::new(
            QUESTIONS_TEXT.len().checked_add(1).expect("separator").checked_add(WORDS_TEXT.len()).expect("words"),
        );
        writer.put(QUESTIONS_TEXT).expect("questions");
        writer.put(b"\n").expect("blank line");
        writer.put(WORDS_TEXT).expect("words");
        writer.finish()
    }

    #[test]
    fn waiting_keeps_unrecorded_words_after_questions_oldest_first() {
        let (core, _) = kept_words(true);
        let BriefRead::Got(parts) = waiting_read(&core, 2, 0, 4096) else { panic!("waiting text") };
        let expected = expected_waiting();
        assert_eq!(parts.len(), 1);
        assert_eq!(parts[0].bytes.as_ref(), expected.as_ref());
        assert_eq!(parts[0].left, 0);
    }

    #[test]
    fn waiting_cuts_unrecorded_words_from_the_end_and_counts_left() {
        let (core, _) = kept_words(true);
        let expected = expected_waiting();
        let accent = skein_lib::bytes::find(&expected, b"\xc3\xa9").expect("multibyte words");
        for budget in [0, 12, QUESTIONS_TEXT.len(), accent.checked_add(1).expect("inside accent")] {
            let BriefRead::Got(parts) = waiting_read(&core, 2, 0, u32::try_from(budget).expect("small budget")) else {
                panic!("cut waiting text")
            };
            let kept = skein_lib::bytes::utf8_prefix(&expected, budget);
            assert_eq!(parts.len(), 1);
            assert_eq!(parts[0].bytes.as_ref(), kept);
            assert_eq!(
                parts[0].left,
                u64::try_from(expected.len().checked_sub(kept.len()).expect("kept prefix")).expect("cut count")
            );
            assert!(parts[0].bytes.len() <= budget);
            let limits = BriefTextLimits {
                parts: 1,
                read_bytes: u32::try_from(budget).expect("small budget"),
                brief_bytes: 4096,
            };
            let rendered = read_brief_part(&core, limits, 2, BriefPart::Waiting { carried: 0 }).expect("cut marker");
            let newline = if kept.is_empty() || kept.ends_with(b"\n") { b"".as_slice() } else { b"\n" };
            let omitted = Decimal::of(parts[0].left);
            let end = b" bytes cut]\n";
            let length = kept
                .len()
                .checked_add(newline.len())
                .expect("line break")
                .checked_add(1)
                .expect("marker opening")
                .checked_add(omitted.as_bytes().len())
                .expect("cut count")
                .checked_add(end.len())
                .expect("marker end");
            let mut marked = Writer::new(length);
            for part in [kept, newline, b"[", omitted.as_bytes(), end] {
                marked.put(part).expect("expected cut marker");
            }
            assert_eq!(rendered.as_ref(), marked.finish().as_ref());
        }
    }

    #[test]
    fn recorded_words_leave_no_waiting_section() {
        let (mut core, limits) = kept_words(false);
        let BriefRead::Got(parts) = waiting_read(&core, 2, 0, 4096) else { panic!("unrecorded words") };
        assert_eq!(parts.len(), 1);
        assert_eq!(parts[0].bytes.as_ref(), WORDS_TEXT);
        let out = task_event(
            &mut core,
            &limits,
            tasks::Event::RecordWords { reply_to: ReplyTo::new(Token::new(1)), task: 2, messages: Box::new([77, 78]) },
        );
        let mut recorded_reply = false;
        for request in &out {
            if let tasks::Request::WordsRecorded { recorded, unknown, .. } = request {
                assert_eq!(recorded.as_ref(), [77, 78]);
                assert!(unknown.is_empty());
                recorded_reply = true;
            }
        }
        assert!(recorded_reply);
        assert!(core.tasks.task(2).expect("recorded task").unrecorded.is_empty());
        let BriefRead::Got(parts) = waiting_read(&core, 2, 0, 4096) else { panic!("empty waiting text") };
        assert!(parts.is_empty(), "no questions or unrecorded words means no waiting fragment");
        assert!(
            read_brief_part(
                &core,
                BriefTextLimits { parts: 1, read_bytes: 4096, brief_bytes: 4096 },
                2,
                BriefPart::Waiting { carried: 0 }
            )
            .expect("empty section")
            .is_empty()
        );
    }

    #[test]
    fn tail_reads_count_only_omitted_original_text_and_cut_the_oldest_text() {
        let mut transcript = Transcript {
            cut: false,
            window: None,
            closed: false,
            moved: false,
            loaded: true,
            previous_attempt: 1,
            last: None,
            from: 1,
            bytes: 0,
            kept: 0,
            turns: Queue::with_capacity(0),
            texts: Queue::with_capacity(128),
            text_bytes: 0,
            text_kept: 0,
        };
        for byte in *b"abc" {
            transcript.append_text(Some(Box::from(&[byte; 40][..])), 128);
        }
        let BriefRead::Got(parts) = tail_read(&transcript, 90) else { panic!("read tail") };
        let mut expected = [b'c'; 80];
        expected.get_mut(..40).unwrap().fill(b'b');
        assert_eq!(parts[0].bytes.as_ref(), expected);
        assert_eq!(parts[0].left, 40);
        let BriefRead::Got(parts) = tail_read(&transcript, 70) else { panic!("read tail") };
        assert_eq!(parts[0].bytes.get(..19).unwrap(), b"[earlier text cut]\n");
        assert_eq!(parts[0].bytes.get(19..30).unwrap(), &[b'b'; 11]);
        assert_eq!(parts[0].bytes.get(30..).unwrap(), &[b'c'; 40]);
        assert_eq!(parts[0].left, 69);
    }
}
