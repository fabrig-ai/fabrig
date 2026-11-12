//! Root committed call answers as bounded, UTF-8 Smith host feedback. The
//! root's typed record is authoritative; rendering has no effects or retry
//! policy (domain/agent/tools.md; domain/routing.md, 2).

use alloc::boxed::Box;
use skein_forge::api;
use skein_lib::{Decimal, List, Writer, bytes};
use smith_domain_run::HostAnswer;
use temper_engine_domain::{AskHow, CallAnswer, Changed};

use crate::Problem;

/// Configured forge-client answer bounds named in read refusals (domain/agent/tools.md).
#[derive(Clone, Copy, Debug)]
pub struct ForgeReadBounds {
    pub rows: u32,
    pub answer_bytes: u32,
}

impl ForgeReadBounds {
    /// The stock figures for renderers without an engine's limits.
    pub const STOCK: Self = Self { rows: 8, answer_bytes: 65_536 };
}

pub(crate) struct Text {
    parts: List<Box<[u8]>>,
    len: usize,
    count_only: bool,
}

impl Text {
    pub(crate) fn new() -> Self {
        // Every retained fragment has at least one byte; the host byte bound
        // bounds their count too (domain/agent/tools.md).
        Self {
            parts: List::with_capacity(u32::try_from(HostAnswer::CAPACITY).expect("host byte bound fits")),
            len: 0,
            count_only: false,
        }
    }

    fn counter() -> Self {
        Self { parts: List::with_capacity(0), len: 0, count_only: true }
    }

    pub(crate) fn add(&mut self, text: &[u8]) -> Result<(), Problem> {
        const DIGITS: &[u8; 16] = b"0123456789abcdef";
        let len = escaped_len(text);
        self.len = self.len.checked_add(len).ok_or(Problem::TooLarge)?;
        if self.count_only || text.is_empty() {
            return Ok(());
        }
        if self.len > HostAnswer::CAPACITY {
            return Err(Problem::TooLarge);
        }
        let mut writer = Writer::new(len);
        for byte in text {
            if (*byte >= b' ' && *byte <= b'~') || *byte == b'\n' {
                if writer.put(&[*byte]).is_err() {
                    return Err(Problem::TooLarge);
                }
            } else {
                let high = usize::from(byte.checked_div(16).ok_or(Problem::Range(b"answer"))?);
                let low = usize::from(byte.checked_rem(16).ok_or(Problem::Range(b"answer"))?);
                if writer
                    .put(&[
                        b'\\',
                        b'x',
                        *DIGITS.get(high).ok_or(Problem::Range(b"answer"))?,
                        *DIGITS.get(low).ok_or(Problem::Range(b"answer"))?,
                    ])
                    .is_err()
                {
                    return Err(Problem::TooLarge);
                }
            }
        }
        let Ok(()) = self.parts.push(writer.finish()) else { return Err(Problem::TooLarge) };
        Ok(())
    }

    // Fixed UTF-8 punctuation in the note acknowledgement is already trusted text.
    pub(crate) fn literal(&mut self, text: &[u8]) -> Result<(), Problem> {
        if text.is_empty() {
            return Ok(());
        }
        self.len = self.len.checked_add(text.len()).ok_or(Problem::TooLarge)?;
        if self.len > HostAnswer::CAPACITY {
            return Err(Problem::TooLarge);
        }
        let part = bytes::copy_of(text);
        if self.parts.push(part).is_err() {
            return Err(Problem::TooLarge);
        }
        Ok(())
    }

    pub(crate) fn number(&mut self, value: u64) -> Result<(), Problem> {
        self.add(Decimal::of(value).as_bytes())
    }

    fn commit(&mut self, commit: &api::Commit) -> Result<(), Problem> {
        self.add(&temper_engine_domain::commit_text(commit))
    }

    pub(crate) fn finish(self, error: bool) -> Result<HostAnswer, Problem> {
        let mut writer = Writer::new(self.len);
        for part in &self.parts {
            if writer.put(part).is_err() {
                return Err(Problem::TooLarge);
            }
        }
        HostAnswer::new(writer.finish(), error).ok_or(Problem::TooLarge)
    }
}

/// Render file, pull-file and job reads within the host body bound, counting
/// metadata and escaping. Files keep a marked prefix; cut logs keep the end.
#[must_use]
pub fn answer_bounded(answer: &CallAnswer, max_bytes: u32) -> HostAnswer {
    bounded(answer, &[], max_bytes, None, ForgeReadBounds::STOCK, None)
}

/// Render a decided answer with its host tool's name, naming a refused family.
/// File, pull-file, compare and job reads fit the host body bound.
#[must_use]
pub fn answer_for_tool(answer: &CallAnswer, tool: &[u8], max_bytes: u32, forge: ForgeReadBounds) -> HostAnswer {
    bounded(answer, tool, max_bytes, None, forge, None)
}

/// Render a cut listing with the page and skip that its call asked for.
#[must_use]
pub fn answer_for_page(
    answer: &CallAnswer,
    tool: &[u8],
    max_bytes: u32,
    forge: ForgeReadBounds,
    page: u32,
    skip: u32,
) -> HostAnswer {
    if let CallAnswer::ForgeRead(read) = answer
        && let Ok(
            value @ (api::Answer::Reviews { cut: true, .. }
            | api::Answer::Remarks { cut: true, .. }
            | api::Answer::PullFiles { cut: true, .. }),
        ) = read.as_ref()
    {
        let cap = skein_lib::usize_of(max_bytes).min(HostAnswer::CAPACITY);
        let mut count = Text::counter();
        forge_answer_at(&mut count, value, (page, skip)).expect("counting allocates no answer text");
        if count.len <= cap {
            let mut text = Text::new();
            forge_answer_at(&mut text, value, (page, skip)).expect("measured page fits");
            return text.finish(false).expect("measured page fits");
        }
        let mut text = Text::new();
        read_size(&mut text, tool, count.len, cap, forge).expect("fixed size guidance");
        answer_configuration(&mut text).expect("configuration recovery");
        return clipped_refusal(text, cap);
    }
    answer_for_tool(answer, tool, max_bytes, forge)
}

/// Render host feedback with the proposing task's ctl name, retaining its holder.
/// Task, file, pull-file and job reads use the configured host body bound.
#[must_use]
pub fn answer_for_task(
    answer: &CallAnswer,
    tool: &[u8],
    max_bytes: u32,
    task: u64,
    forge: ForgeReadBounds,
) -> HostAnswer {
    bounded(answer, tool, max_bytes, Some(task), forge, None)
}

/// Render a job read with its own pin, including evidence when the attempt is missing.
#[must_use]
pub fn answer_for_job(
    answer: &CallAnswer,
    tool: &[u8],
    max_bytes: u32,
    forge: ForgeReadBounds,
    attempt: api::JobAttempt,
) -> HostAnswer {
    bounded(answer, tool, max_bytes, None, forge, Some(attempt))
}

fn job_missing(attempt: api::JobAttempt, max_bytes: u32) -> HostAnswer {
    let mut text = Text::new();
    missing_attempt(&mut text, attempt).expect("fixed job refusal and bounded attempt");
    clipped_refusal(text, skein_lib::usize_of(max_bytes).min(HostAnswer::CAPACITY))
}

fn missing_attempt(text: &mut Text, attempt: api::JobAttempt) -> Result<(), Problem> {
    text.add(b"Forge read failed: job missing. No job attempt at head ")?;
    text.commit(&attempt.head)?;
    text.add(b", run ")?;
    text.number(attempt.run)?;
    text.add(b", job ")?;
    text.number(attempt.job)?;
    text.add(b", attempt ")?;
    text.number(u64::from(attempt.attempt))?;
    text.add(b", as asked. read_checks at this head names each Actions job's run, job and attempt as read_job takes them; a link's run number and its index of the job are not those ids.")
}

/// The kind of a forge read's answer no agent read asks for, as the log's
/// reason (temper/docs/design/domain/agent/agents.md).
#[must_use]
pub fn unasked_kind(answer: &CallAnswer) -> Option<&'static str> {
    if let CallAnswer::ForgeRead(read) = answer {
        return match read.as_ref() {
            Ok(answer) => match answer {
                api::Answer::Runs { .. } => Some("answer of kind Runs"),
                api::Answer::Run(_) => Some("answer of kind Run"),
                api::Answer::Commits { .. } => Some("answer of kind Commits"),
                api::Answer::Tree { .. } => Some("answer of kind Tree"),
                api::Answer::Queued => Some("answer of kind Queued"),
                api::Answer::Located { .. }
                | api::Answer::Items { .. }
                | api::Answer::Item { .. }
                | api::Answer::Pull(_)
                | api::Answer::Reviews { .. }
                | api::Answer::Statuses { .. }
                | api::Answer::Remarks { .. }
                | api::Answer::Commit(_)
                | api::Answer::Branches(_)
                | api::Answer::PullFiles { .. }
                | api::Answer::Compare { .. }
                | api::Answer::Checks(_)
                | api::Answer::Job { .. }
                | api::Answer::File { .. }
                | api::Answer::Directory { .. }
                | api::Answer::Protection(_)
                | api::Answer::Settings(_)
                | api::Answer::Collaborators { .. }
                | api::Answer::Permission(_)
                | api::Answer::Created(_)
                | api::Answer::Commented(_)
                | api::Answer::Reviewed(_)
                | api::Answer::Merged(_)
                | api::Answer::Branch(_)
                | api::Answer::Done => None,
            },
            Err(_) => None,
        };
    }
    None
}

fn unasked_refusal(tool: &[u8], reason: &'static str, cap: usize) -> HostAnswer {
    let mut text = Text::new();
    text.add(b"Forge read refused: ").expect("fixed unasked refusal");
    text.add(if tool.is_empty() { b"the read" } else { tool }).expect("bounded tool name");
    text.add(b" got an ").expect("fixed unasked refusal");
    text.add(reason.as_bytes()).expect("fixed unasked kind");
    text.add(b", which it never asks for. Nothing was read; read again.").expect("fixed unasked refusal");
    clipped_refusal(text, cap)
}

fn bounded(
    answer: &CallAnswer,
    tool: &[u8],
    max_bytes: u32,
    task: Option<u64>,
    forge: ForgeReadBounds,
    job: Option<api::JobAttempt>,
) -> HostAnswer {
    let cap = skein_lib::usize_of(max_bytes).min(HostAnswer::CAPACITY);
    if let Some(reason) = unasked_kind(answer) {
        return unasked_refusal(tool, reason, cap);
    }
    if let CallAnswer::Finding(answer) = answer {
        return crate::finding::render(answer, cap);
    }
    if let CallAnswer::ForgeRead(read) = answer {
        if let Some(job) = job
            && read.as_ref() == &Err(api::Error::MissingJob)
        {
            return job_missing(job, max_bytes);
        }
        if let Ok(value @ (api::Answer::File { .. } | api::Answer::Directory { .. } | api::Answer::PullFiles { .. })) =
            read.as_ref()
        {
            return file_read_bounded(value, cap, forge);
        }
        if tool == b"read_compare" {
            return compare_bounded(read, cap, forge);
        }
        if read.is_err() {
            let mut text = Text::new();
            forge_read(&mut text, read, tool, forge).expect("fixed forge refusal");
            return clipped_refusal(text, cap);
        }
        if let Ok(api::Answer::Job { attempt, step, log, truncated }) = read.as_ref() {
            return job_bounded(attempt, step.as_ref(), log, *truncated, max_bytes);
        }
    }
    if let CallAnswer::TaskRead(read) = answer {
        return match task_read(read, cap) {
            Ok(answer) => answer,
            Err(_) => {
                let why = b"Task metadata exceeds the host answer limit.";
                HostAnswer::new(bytes::copy_of(why.get(..cap.min(why.len())).expect("bounded refusal")), true)
                    .expect("bounded task refusal")
            }
        };
    }
    answer_text(answer, tool, task, forge, AskHow::None)
}

fn compare_bounded(read: &Result<api::Answer, api::Error>, cap: usize, forge: ForgeReadBounds) -> HostAnswer {
    if let Err(api::Error::TooLarge(excess)) = read {
        let mut text = Text::new();
        forge_refused_size(&mut text, *excess).expect("measured compare refusal");
        text.add(b"This comparison is too large to return whole. Its commit ids were read; another id won't help. Never retry with a changed or guessed id.\n")
            .expect("compare refusal");
        compare_contents(&mut text, forge).expect("compare contents");
        compare_paged_recovery(&mut text).expect("compare recovery");
        return clipped_refusal(text, cap);
    }
    let mut count = Text::counter();
    let error = forge_read(&mut count, read, b"read_compare", forge).expect("counting compare");
    if count.len <= cap {
        let mut text = Text::new();
        if forge_read(&mut text, read, b"read_compare", forge).is_ok()
            && let Ok(answer) = text.finish(error)
        {
            return answer;
        }
    }
    let measured = count.len;
    let mut text = Text::new();
    read_size(&mut text, b"read_compare", measured, cap, forge).expect("compare size");
    compare_recovery(&mut text, forge).expect("compare recovery");
    answer_configuration(&mut text).expect("configuration recovery");
    clipped_refusal(text, cap)
}

fn job_bounded(
    attempt: &api::JobAttempt,
    step: Option<&api::Step>,
    log: &[u8],
    truncated: bool,
    max_bytes: u32,
) -> HostAnswer {
    let mut text = Text::new();
    let result = job_text(&mut text, attempt, step, log, truncated, max_bytes);
    let result = match result {
        Ok(()) => text.finish(false),
        Err(problem) => Err(problem),
    };
    match result {
        Ok(answer) => answer,
        Err(_) => {
            const WHY: &[u8] = b"Job metadata exceeds the host answer limit.";
            let length = skein_lib::usize_of(max_bytes).min(WHY.len());
            HostAnswer::new(bytes::copy_of(WHY.get(..length).expect("bounded feedback")), true)
                .expect("fixed host feedback fits")
        }
    }
}

fn file_read_bounded(value: &api::Answer, cap: usize, forge: ForgeReadBounds) -> HostAnswer {
    let mut count = Text::counter();
    forge_answer(&mut count, value).expect("counting allocates no answer text");
    let mut text = Text::new();
    if count.len <= cap {
        let rendered = forge_answer(&mut text, value);
        if rendered.is_ok()
            && let Ok(answer) = text.finish(false)
        {
            return answer;
        }
    }
    if let api::Answer::File { head, path, bytes, .. } = value {
        let mut text = Text::new();
        if file_prefix(&mut text, head, path, bytes, count.len, cap, forge).is_ok() {
            return text.finish(false).expect("file prefix fits");
        }
        let mut text = Text::new();
        read_size(&mut text, b"read_file", count.len, cap, forge).expect("file size");
        text.add(b"File metadata leaves no room for content at the host answer limit. Use a shorter path; lowering max_bytes cannot fit metadata.\n").expect("fixed feedback");
        answer_configuration(&mut text).expect("configuration recovery");
        return clipped_refusal(text, cap);
    }
    if let api::Answer::PullFiles { head, files, more, .. } = value {
        return pull_inventory(head, files, *more, count.len, cap, forge);
    }
    file_read_refusal(b"read_file", count.len, cap, forge)
}

fn file_heading(text: &mut Text, head: &api::Commit, path: &[u8]) -> Result<(), Problem> {
    text.add(path)?;
    text.add(b" at ")?;
    text.commit(head)?;
    text.add(b":\n")
}

fn file_prefix(
    text: &mut Text,
    head: &api::Commit,
    path: &[u8],
    bytes: &[u8],
    measured: usize,
    cap: usize,
    forge: ForgeReadBounds,
) -> Result<(), Problem> {
    let mut footer = Text::new();
    footer.add(b"\n")?;
    read_size(&mut footer, b"read_file", measured, cap, forge)?;
    file_recovery(&mut footer)?;
    answer_configuration(&mut footer)?;
    footer.add(b"\n[file truncated]")?;
    let footer = footer.finish(false)?;
    file_heading(text, head, path)?;
    let room = cap.checked_sub(text.len).ok_or(Problem::TooLarge)?;
    let feedback_room = room.saturating_sub(footer.text().len());
    let footer = if fitting_prefix(bytes, feedback_room).is_empty() { b"\n[file truncated]" } else { footer.text() };
    let room = room.checked_sub(footer.len()).ok_or(Problem::TooLarge)?;
    let prefix = fitting_prefix(bytes, room);
    if prefix.is_empty() && !bytes.is_empty() {
        return Err(Problem::TooLarge);
    }
    text.add(prefix)?;
    text.add(footer)
}

fn fitting_prefix(bytes: &[u8], room: usize) -> &[u8] {
    let mut used = 0_usize;
    let mut count = 0_usize;
    for byte in bytes {
        let size = escaped_len(core::slice::from_ref(byte));
        if used.saturating_add(size) > room {
            break;
        }
        used = used.saturating_add(size);
        count = count.saturating_add(1);
    }
    bytes::utf8_prefix(bytes, count)
}

fn read_size(
    text: &mut Text,
    tool: &[u8],
    measured: usize,
    cap: usize,
    _forge: ForgeReadBounds,
) -> Result<(), Problem> {
    text.add(tool)?;
    text.add(b" rendered size ")?;
    text.number(u64::try_from(measured).expect("answer size fits u64"))?;
    text.add(b" bytes; host answer limit ")?;
    text.number(u64::try_from(cap).expect("host bound fits u64"))?;
    text.add(b" bytes.\n")
}

fn file_recovery(text: &mut Text) -> Result<(), Problem> {
    text.add(b"Retry read_file with smaller max_bytes, leaving room for path/commit metadata and byte escaping overhead (up to 4 bytes per input byte). A truncated prefix is not the whole file; max_bytes is a prefix limit, not an offset. A directory lists paths for selected reads.\n")
}

fn pull_recovery(text: &mut Text) -> Result<(), Problem> {
    text.add(b"Contents [omitted]. Use read_file for selected paths at the head commit (after) and the comparison's base commit (before; resolve separately), with max_bytes leaving room for metadata and escaping overhead. If paths are omitted, list directories there with read_file. Advancing page alone cannot recover this oversized page; later pages only list other files.\n")
}

fn clipped_refusal(text: Text, cap: usize) -> HostAnswer {
    let whole = text.finish(true).expect("fixed size guidance fits host capacity");
    let end = cap.min(whole.text().len());
    HostAnswer::new(bytes::copy_of(whole.text().get(..end).expect("bounded refusal")), true)
        .expect("refusal fits host capacity")
}

fn file_read_refusal(tool: &[u8], measured: usize, cap: usize, forge: ForgeReadBounds) -> HostAnswer {
    let mut text = Text::new();
    read_size(&mut text, tool, measured, cap, forge).expect("fixed size guidance");
    if tool == b"read_pull_files" {
        pull_recovery(&mut text).expect("fixed pull recovery");
        text.add(b"No changed-path inventory available; use read_file directories at both endpoints.\n")
            .expect("fixed inventory guidance");
    } else {
        file_recovery(&mut text).expect("fixed file recovery");
    }
    answer_configuration(&mut text).expect("configuration recovery");
    clipped_refusal(text, cap)
}

fn inventory_row(text: &mut Text, file: &api::File) -> Result<(), Problem> {
    text.add(&file.path)?;
    text.add(b": before ")?;
    text.add(if file.before.is_some() { b"[omitted]" } else { b"[absent]" })?;
    text.add(b", after ")?;
    text.add(if file.after.is_some() { b"[omitted]" } else { b"[absent]" })?;
    text.add(b"\n")
}

fn pull_inventory(
    head: &api::Commit,
    files: &[api::File],
    more: bool,
    size: usize,
    cap: usize,
    forge: ForgeReadBounds,
) -> HostAnswer {
    const CUT_ROOM: usize = 48;
    // Six row parts and three for the omitted-path count.
    const ROW_AND_CUT_PARTS: u32 = 9;
    let mut text = Text::new();
    read_size(&mut text, b"read_pull_files", size, cap, forge).expect("fixed size guidance");
    pull_recovery(&mut text).expect("fixed pull recovery");
    answer_configuration(&mut text).expect("configuration recovery");
    text.add(b"Head commit ").expect("fixed head label");
    text.commit(head).expect("commit fits");
    text.add(if more { b"\nMore files available on next page.\n" } else { b"\nNo more file pages.\n" })
        .expect("fixed pagination");
    let mut shown = 0_usize;
    for file in files {
        let mut count = Text::counter();
        inventory_row(&mut count, file).expect("counting inventory");
        if text.parts.room() < ROW_AND_CUT_PARTS || text.len.saturating_add(count.len).saturating_add(CUT_ROOM) > cap {
            break;
        }
        inventory_row(&mut text, file).expect("measured inventory fits");
        shown = shown.saturating_add(1);
    }
    if shown < files.len() {
        text.add(b"[changed paths omitted: ").expect("reserved marker");
        text.number(u64::try_from(files.len().saturating_sub(shown)).expect("file count fits"))
            .expect("reserved count");
        text.add(b"]\n").expect("reserved marker end");
    }
    clipped_refusal(text, cap)
}

fn job_text(
    text: &mut Text,
    attempt: &api::JobAttempt,
    step: Option<&api::Step>,
    log: &[u8],
    truncated: bool,
    max_bytes: u32,
) -> Result<(), Problem> {
    const CUT: &[u8] = b"[log bytes cut]\n";
    text.add(b"Job ")?;
    text.number(attempt.job)?;
    text.add(b" run ")?;
    text.number(attempt.run)?;
    if let Some(step) = step {
        text.add(b", failed step ")?;
        text.number(u64::from(step.number))?;
        if !step.name.is_empty() {
            text.add(b" ")?;
            text.add(&step.name)?;
        }
    } else {
        text.add(b", no step failed")?;
    }
    text.add(b":\n")?;
    let cap = skein_lib::usize_of(max_bytes).min(HostAnswer::CAPACITY);
    let room = cap.checked_sub(text.len).ok_or(Problem::TooLarge)?;
    let whole = escaped_len(log);
    if !truncated && whole <= room {
        return text.add(log);
    }
    let room = room.checked_sub(CUT.len()).ok_or(Problem::TooLarge)?;
    let mut used = 0_usize;
    let mut start = log.len();
    for byte in log.iter().rev() {
        let size = escaped_len(core::slice::from_ref(byte));
        if used.saturating_add(size) > room {
            break;
        }
        used = used.saturating_add(size);
        start = start.saturating_sub(1);
    }
    text.add(CUT)?;
    text.add(log.get(start..).expect("bounded suffix"))
}

fn escaped_len(text: &[u8]) -> usize {
    let mut len = 0_usize;
    for byte in text {
        len = len.saturating_add(if (*byte >= b' ' && *byte <= b'~') || *byte == b'\n' { 1 } else { 4 });
    }
    len
}

/// Render a durable root answer for Smith's LLM. Any unrenderable payload
/// becomes a bounded host error, so no decided call is left unanswered.
#[must_use]
pub fn answer(answer: &CallAnswer) -> HostAnswer {
    answer_text(answer, &[], None, ForgeReadBounds::STOCK, AskHow::None)
}

/// Render a decision with the asking route the root read from its caller, beside
/// the unchanged stored answer (domain/agent/tools.md).
#[must_use]
pub fn answer_for_decided(answer: &CallAnswer, how: AskHow) -> HostAnswer {
    answer_text(answer, b"decided", None, ForgeReadBounds::STOCK, how)
}

fn answer_text(answer: &CallAnswer, tool: &[u8], task: Option<u64>, forge: ForgeReadBounds, how: AskHow) -> HostAnswer {
    match rendered(answer, tool, task, forge, how) {
        Ok(answer) => answer,
        Err(Problem::Unasked(reason)) => unasked_refusal(tool, reason, HostAnswer::CAPACITY),
        Err(Problem::TooLarge) => HostAnswer::new(
            bytes::copy_of(b"The recorded answer exceeded the host text limit; ask a narrower read."),
            true,
        )
        .expect("fixed host feedback fits"),
        Err(
            Problem::Malformed
            | Problem::Missing(_)
            | Problem::Needs { .. }
            | Problem::Type(_)
            | Problem::Range(_)
            | Problem::Beside(_)
            | Problem::UnknownTool,
        ) => HostAnswer::new(bytes::copy_of(b"The recorded answer cannot be rendered as text."), true)
            .expect("fixed host feedback fits"),
    }
}

fn proposed(
    text: &mut Text,
    proposal: u64,
    holder: jig_domain_tasks::ProposalHolder,
    task: Option<u64>,
) -> Result<(), Problem> {
    let settings = holder
        == (jig_domain_tasks::ProposalHolder::Policy { project: 0, kind: jig_domain_tasks::ProposalKind::Settings });
    text.add(if settings { b"proposed as " } else { b"Proposed as " })?;
    if let Some(task) = task {
        text.add(b"t")?;
        text.number(task)?;
        text.add(b"/")?;
    } else if settings {
        text.add(b"proposal ")?;
    }
    text.number(proposal)?;
    if settings {
        text.add(b" for the deployment's owners to decide: it waits as a card in this chat and in Needs you, and nothing changes until an owner accepts it. Tell the person so, without ")?;
        if let Some(task) = task {
            text.add(b"t")?;
            text.number(task)?;
            text.add(b"/")?;
        } else {
            text.add(b"proposal ")?;
        }
        text.number(proposal)?;
        text.add(b".")?;
    }
    Ok(())
}

// The judgement's reason is kept with the answer (temper/docs/design/domain/agent/tools.md).
fn ask_reason(text: &mut Text, reach: Option<temper_protocol_reaches::Reach>) -> Result<(), Problem> {
    use temper_protocol_reaches::{Class, Reason};
    let Some(reach) = reach else {
        return text.add(b"no person is asked through you");
    };
    if reach.decided() {
        return text.add(b"your person's decided list is full for today");
    }
    match reach.reason {
        Reason::Always => text.add(b"your words say Asks: always"),
        Reason::Kind | Reason::Set(_) => {
            if reach.class == Class::Wait {
                text.add(reach.kind.name())?;
                return text.add(b" always waits for your person");
            }
            match reach.reason {
                Reason::Kind => {
                    text.add(reach.kind.name())?;
                    text.add(b" asks your person")?;
                }
                Reason::Set(number) => {
                    text.add(b"changes to path set ")?;
                    text.number(u64::from(number))?;
                    text.add(b" ask your person")?;
                }
                Reason::Always => unreachable!("handled above"),
            }
            match reach.class {
                Class::AskAway => text.add(b", even when away"),
                Class::Ask => text.add(b" while they are here"),
                Class::Decide | Class::Wait => unreachable!("handled above"),
            }
        }
    }
}

#[expect(clippy::too_many_lines, reason = "one exhaustive durable answer vocabulary")]
fn rendered(
    answer: &CallAnswer,
    tool: &[u8],
    task: Option<u64>,
    forge: ForgeReadBounds,
    how: AskHow,
) -> Result<HostAnswer, Problem> {
    let mut text = Text::new();
    let error = match answer {
        CallAnswer::Finding(answer) => return Ok(crate::finding::render(answer, HostAnswer::CAPACITY)),
        CallAnswer::Decided { took: choice, reach } => {
            match choice {
                Some(words) => {
                    text.add(b"Decided: carry on with ")?;
                    text.add(words)?;
                    text.add(b".")?;
                }
                None => {
                    text.add(if how == AskHow::None { b"Not decided: " } else { b"Ask: " })?;
                    ask_reason(&mut text, *reach)?;
                    match how {
                        AskHow::Message => text.add(b". Ask your requester with message, as a question.")?,
                        AskHow::Person(person) => {
                            text.add(b". Ask person ")?;
                            text.number(person)?;
                            text.add(b" with delegate, a person task addressed to them.")?;
                        }
                        AskHow::None => text.add(b". Nothing you hold asks anyone: decide within your brief, or leave the choice and your recommendation in your result.")?,
                    }
                }
            }
            false
        }
        CallAnswer::TaskRead(read) => return task_read(read, HostAnswer::CAPACITY),
        CallAnswer::SettingsRefused(why) => {
            settings_refused(&mut text, why)?;
            true
        }
        CallAnswer::ForgeEffect { entry, outcome, .. } => {
            text.add(b"Forge effect ")?;
            text.number(*entry)?;
            match outcome {
                Some(outcome) => {
                    text.add(b": ")?;
                    forge_outcome(&mut text, outcome)?;
                }
                None => text.add(b" committed and pending")?,
            }
            false
        }
        CallAnswer::ReadRepositoryRefused { named, repositories }
        | CallAnswer::SubscriptionRepositoryRefused { named, repositories, .. }
        | CallAnswer::ChangeRefused { named, repositories, .. } => {
            if repositories.is_empty() {
                if let CallAnswer::ReadRepositoryRefused { .. } | CallAnswer::SubscriptionRepositoryRefused { .. } =
                    answer
                {
                    text.add(b"this project has no repository to read")?;
                } else {
                    text.add(b"this project has no repository to change")?;
                }
            } else {
                match named {
                    Some(named) => {
                        text.add(b"no repository ")?;
                        text.add(named)?;
                        text.add(b" here; this project has: ")?;
                    }
                    None => text.add(b"name the repository: ")?,
                }
                for (at, repository) in repositories.iter().enumerate() {
                    if at != 0 {
                        text.add(b", ")?;
                    }
                    text.add(repository)?;
                }
            }
            true
        }
        CallAnswer::ForgeEffectRefused(refusal) => {
            text.add(b"Forge effect refused: ")?;
            forge_error(&mut text, *refusal)?;
            true
        }
        CallAnswer::ForgeEffectDenied { answer, findings } | CallAnswer::ToolDenied { answer, findings } => {
            authority_findings(&mut text, *answer, findings, tool, false)?;
            true
        }
        CallAnswer::DelegationDenied { answer, findings } => {
            authority_findings(&mut text, *answer, findings, tool, true)?;
            true
        }
        CallAnswer::ForgeRead(read) => forge_read(&mut text, read, tool, forge)?,
        CallAnswer::HoldOwner { task, hold, revision, owner } => {
            hold_owner(&mut text, *task, *hold, *revision, *owner)?;
            true
        }
        CallAnswer::EscalationDecided { task, revision, .. } => {
            text.add(b"Escalation for task ")?;
            text.number(*task)?;
            text.add(b" revision ")?;
            text.number(*revision)?;
            text.add(b" decided")?;
            false
        }
        CallAnswer::ForgeSubscriptionRefused { task, topic, cause, limit, existing } => {
            subscription_refusal(&mut text, *task, topic, cause, *limit, *existing)?;
            true
        }
        CallAnswer::MessageRefused(problem) => refused_problem(&mut text, problem, true)?,
        CallAnswer::EscalationRefused(problem)
        | CallAnswer::ProposalRefused(problem)
        | CallAnswer::EffectRefused(problem)
        | CallAnswer::ControlRefused(problem)
        | CallAnswer::SubscriptionRefused(problem)
        | CallAnswer::DelegationRefused(problem) => refused_problem(&mut text, problem, false)?,
        CallAnswer::Proposed { proposal, holder } => {
            proposed(&mut text, *proposal, *holder, task)?;
            false
        }
        CallAnswer::ProposalDecided { proposal, .. } => {
            text.add(b"Proposal ")?;
            text.number(*proposal)?;
            text.add(b" decided")?;
            false
        }
        CallAnswer::Again { task, required } => {
            text.add(b"Refused: task ")?;
            text.number(*task)?;
            text.add(if *required {
                b" holds an effect whose outcome is uncertain; say again: true or false"
            } else {
                b" holds no uncertain effect; leave again out"
            })?;
            true
        }
        CallAnswer::PassedUp { task } => {
            text.add(b"Beyond your authority: task ")?;
            text.number(*task)?;
            text.add(b"'s escalation was passed up")?;
            false
        }
        CallAnswer::Amended { task, revision, changed } => {
            text.add(b"Done: task ")?;
            text.number(*task)?;
            text.add(b" is at spec revision ")?;
            text.number(*revision)?;
            let mut separator = b"; changed: ".as_slice();
            match changed.tracked {
                Some(jig_domain_tasks::Tracking::Tracked { priority }) => {
                    text.add(separator)?;
                    text.add(b"priority ")?;
                    text.number(u64::from(priority))?;
                    separator = b", ";
                }
                Some(jig_domain_tasks::Tracking::Untracked) => {
                    text.add(separator)?;
                    text.add(b"untracked")?;
                    separator = b", ";
                }
                None => {}
            }
            for (bit, name) in [
                (Changed::SPEC, b"spec".as_slice()),
                (Changed::WAKE, b"wake"),
                (Changed::DEPENDENCIES, b"dependencies"),
                (Changed::AUTHORITY, b"authority"),
                (Changed::INSTRUCTIONS, b"instructions"),
                (Changed::LLM, b"llm"),
            ] {
                if changed.fields & bit != 0 {
                    text.add(separator)?;
                    text.add(name)?;
                    separator = b", ";
                }
            }
            false
        }
        CallAnswer::Revised { task, revision } => {
            text.add(b"Done: task ")?;
            text.number(*task)?;
            text.add(b" is at spec revision ")?;
            text.number(*revision)?;
            false
        }
        CallAnswer::Moved { task, now } => {
            moved(&mut text, *task, now)?;
            true
        }
        CallAnswer::Controlled => {
            text.add(b"Done")?;
            false
        }
        CallAnswer::ControlDenied { answer } => {
            text.add(b"Control denied: ")?;
            authority(&mut text, *answer)?;
            true
        }
        CallAnswer::Sent { message } => {
            text.add(b"Message ")?;
            text.number(*message)?;
            text.add(b" sent")?;
            false
        }
        CallAnswer::Introduced => {
            text.add(b"Tasks introduced")?;
            false
        }
        CallAnswer::Subscribed { subscription } => {
            text.add(b"Subscription ")?;
            text.number(*subscription)?;
            text.add(b" added")?;
            false
        }
        CallAnswer::Unsubscribed => {
            text.add(b"Subscription removed")?;
            false
        }
        CallAnswer::Delegated(ids) => {
            // The client's chat cards read this line (temper-client-domain pages/delegation.rs `delegated`).
            text.add(b"Delegated tasks: ")?;
            for (at, id) in ids.iter().enumerate() {
                if at > 0 {
                    text.add(b", ")?;
                }
                text.number(id.task)?;
            }
            false
        }
        CallAnswer::NoteDeleted { name } => {
            text.add(b"Deleted note ")?;
            text.number(*name)?;
            false
        }
        CallAnswer::NoteWritten { name, revision, lines, added, removed, first } => {
            note_written(
                &mut text,
                *name,
                *revision,
                jig_domain_notes::LineChange { lines: *lines, added: *added, removed: *removed, first: *first },
            )?;
            false
        }
        CallAnswer::NoteRecalled { entries, more } => {
            for entry in entries {
                text.add(b"Note ")?;
                text.number(entry.name)?;
                text.add(b" revision ")?;
                text.number(u64::from(entry.revision))?;
                text.add(b": ")?;
                text.add(&entry.description)?;
                text.add(b"\n")?;
                text.add(&entry.body)?;
                text.add(b"\n")?;
            }
            if *more {
                text.add(b"More notes are available.")?;
            }
            false
        }
        CallAnswer::Parts(parts) => crate::settings::parts(&mut text, parts)?,
        CallAnswer::Document(documented) => crate::document::render(&mut text, documented)?,
        CallAnswer::Remember(remembered) => crate::remember::render(&mut text, remembered)?,
        CallAnswer::NoteRefused(why) => {
            text.add(b"Note refused: ")?;
            let ending: &[u8] = match why {
                jig_domain_notes::Refusal::Busy => b"busy",
                jig_domain_notes::Refusal::Oversized => b"oversized",
                jig_domain_notes::Refusal::Full => b"full",
                jig_domain_notes::Refusal::Exists => b"already exists",
                jig_domain_notes::Refusal::Missing => b"missing",
                jig_domain_notes::Refusal::Moved { revision } => {
                    text.add(b"revision moved, now ")?;
                    text.number(u64::from(*revision))?;
                    b""
                }
                jig_domain_notes::Refusal::PastEnd { lines } => {
                    text.add(b"range past the end, ")?;
                    text.number(u64::from(*lines))?;
                    b" lines"
                }
                jig_domain_notes::Refusal::PastBound { bytes, bound } => {
                    text.add(b"body past its bound, ")?;
                    text.number(u64::from(*bytes))?;
                    text.add(b" bytes of ")?;
                    text.number(u64::from(*bound))?;
                    b""
                }
                jig_domain_notes::Refusal::RevisionExhausted => b"revision exhausted",
            };
            text.add(ending)?;
            true
        }
        CallAnswer::Unavailable => {
            text.add(b"This engine tool is unavailable.")?;
            true
        }
    };
    text.finish(error)
}

/// The client's note line reads this acknowledgement (temper/docs/design/domain/agent/agents.md, 3).
pub(crate) fn note_written(
    text: &mut Text,
    name: u64,
    revision: u32,
    change: jig_domain_notes::LineChange,
) -> Result<(), Problem> {
    text.add(b"Note ")?;
    text.number(name)?;
    text.add(b" revision ")?;
    text.number(u64::from(revision))?;
    text.literal(" written · ".as_bytes())?;
    if revision == 1 {
        text.number(u64::from(change.lines))?;
        text.add(b" lines")?;
    } else {
        text.add(b"+")?;
        text.number(u64::from(change.added))?;
        text.literal(" −".as_bytes())?;
        text.number(u64::from(change.removed))?;
        text.literal(" · line ".as_bytes())?;
        text.number(u64::from(change.first))?;
        text.literal(" · ".as_bytes())?;
        text.number(u64::from(change.lines))?;
        text.add(b" lines")?;
    }
    Ok(())
}

fn forge_outcome(text: &mut Text, outcome: &skein_forge_client::Outcome) -> Result<(), Problem> {
    match outcome {
        skein_forge_client::Outcome::Made { made, found } => {
            if *found {
                text.add(b"found ")?;
            } else {
                text.add(b"made ")?;
            }
            made_text(text, *made)
        }
        skein_forge_client::Outcome::Failed(failure) => forge_error(text, *failure),
        skein_forge_client::Outcome::Raced { made, why } => {
            made_text(text, *made)?;
            text.add(b" raced: ")?;
            forge_error(text, *why)
        }
        skein_forge_client::Outcome::Queued => text.add(b"queued"),
        skein_forge_client::Outcome::Uncertain => text.add(b"uncertain; connector will find it"),
        skein_forge_client::Outcome::Held => text.add(b"uncertain; task held for a person"),
        skein_forge_client::Outcome::Withdrawn => text.add(b"withdrawn before write"),
    }
}

fn made_text(text: &mut Text, made: skein_forge_client::Made) -> Result<(), Problem> {
    match made {
        skein_forge_client::Made::Created(number)
        | skein_forge_client::Made::Commented(number)
        | skein_forge_client::Made::Reviewed(number) => text.number(number),
        skein_forge_client::Made::Merged(commit)
        | skein_forge_client::Made::Updated(commit)
        | skein_forge_client::Made::Branch(commit) => text.commit(&commit),
        skein_forge_client::Made::Set => text.add(b"state set"),
    }
}

fn authority(text: &mut Text, answer: jig_domain_authority::Answer) -> Result<(), Problem> {
    match answer {
        jig_domain_authority::Answer::Allow => text.add(b"allowed"),
        jig_domain_authority::Answer::Wait => text.add(b"waiting"),
        jig_domain_authority::Answer::Propose => text.add(b"needs proposal"),
        jig_domain_authority::Answer::Refuse => text.add(b"refused"),
    }
}

pub(crate) fn authority_findings(
    text: &mut Text,
    answer: jig_domain_authority::Answer,
    findings: &[jig_domain_authority::Finding],
    tool: &[u8],
    delegation: bool,
) -> Result<(), Problem> {
    use jig_domain_authority::Finding;
    text.add(b"Authority ")?;
    authority(text, answer)?;
    text.add(b": ")?;
    let mut family_seen = false;
    let mut finding_seen = false;
    let mut reference_hint = false;
    let mut change_hint = false;
    for finding in findings {
        if *finding == Finding::Tool && family_seen {
            continue;
        }
        if finding_seen {
            text.add(b"; ")?;
        }
        finding_seen = true;
        if *finding == Finding::Tool {
            family_seen = true;
            text.add(b"tool family")?;
            if let Some(family) = tool_family(tool) {
                text.add(b" ")?;
                text.add(family)?;
            }
        } else {
            finding_text(text, *finding)?;
        }
        if *finding == Finding::Reference && tool == b"message" {
            reference_hint = true;
        }
        if delegation
            && tool != b"delegate"
            && let Finding::Executor { .. } | Finding::Grant { .. } = finding
        {
            change_hint = true;
        }
    }
    if reference_hint {
        text.add(b". Your brief's [Requested by ...] line names your requester; [This task: N] is you. If it says [Requested by person N], they have no task to message: ask with delegate, a person task addressed to person N.")?;
    }
    if change_hint {
        text.add(b". For a change to a repository, call change with what to change.")?;
    }
    Ok(())
}

fn tool_family(tool: &[u8]) -> Option<&'static [u8]> {
    Some(match tool {
        b"decided" | b"amend" | b"cancel" | b"release" => b"control",
        b"goal" | b"change" | b"delegate" => b"delegate",
        b"message" | b"introduce" => b"message",
        b"decide" | b"withdraw" | b"decide_escalation" => b"decide",
        b"propose" => b"propose",
        b"subscribe" | b"subscribe_forge" | b"unsubscribe" => b"subscribe",
        b"effect_forge" => b"effect",
        b"note" => b"note",
        b"recall" => b"recall",
        b"settings" | b"propose_settings" => b"settings",
        b"see" | b"document" | b"remember" => b"asking",
        name if name.starts_with(b"read_") => b"read",
        _ => return None,
    })
}

fn finding_text(text: &mut Text, finding: jig_domain_authority::Finding) -> Result<(), Problem> {
    use jig_domain_authority::Finding;
    match finding {
        Finding::Oversized => text.add(b"too large"),
        Finding::UnknownProject => text.add(b"unknown project"),
        Finding::UnknownRole => text.add(b"unknown role"),
        Finding::Authority { source, lacks } => authority_lacks(text, source, lacks),
        Finding::Executor { source } => missing_authority(text, source, b"executor permission", b"delegation.kinds"),
        Finding::Tasks { source } => missing_authority(text, source, b"task allowance", b"delegation.tasks"),
        Finding::Spend { source } => missing_authority(text, source, b"spend allowance", b"budget.spend"),
        Finding::Price { source } => missing_authority(text, source, b"effect price", b"budget.spend"),
        Finding::Arithmetic => text.add(b"arithmetic limit"),
        Finding::RunBudget => text.add(b"run budget"),
        Finding::RunCap => text.add(b"run cap"),
        Finding::Deadline => text.add(b"deadline"),
        Finding::Account => text.add(b"account unavailable"),
        Finding::Writer => text.add(b"writer held"),
        Finding::Tool => text.add(b"tool family"),
        Finding::Grant { source } => missing_authority(text, source, b"resource grant", b"grants"),
        Finding::ResourceUnavailable => text.add(b"resource unavailable"),
        Finding::ResourceAccess => text.add(b"resource write unavailable"),
        Finding::Reference => text.add(b"task reference"),
        Finding::Scope { source } => missing_authority(text, source, b"note scope", b"notes"),
        Finding::Required { .. } => text.add(b"required fact"),
        Finding::Failed { .. } => text.add(b"failed fact"),
        Finding::Unpermitted => text.add(b"request not permitted"),
        Finding::Undecidable => text.add(b"proposal decision not permitted"),
        Finding::PeriodSpend => text.add(b"period spend ceiling"),
        Finding::Unguarded { .. } => text.add(b"required guard unavailable"),
    }
}

fn authority_source(source: jig_domain_authority::Source) -> &'static [u8] {
    use jig_domain_authority::Source;
    match source {
        Source::Task => b"task authority",
        Source::Role => b"project role authority",
        Source::Project => b"project policy.ceiling",
        Source::Deployment => b"rules.ceiling",
    }
}

fn missing_authority(
    text: &mut Text,
    source: jig_domain_authority::Source,
    dimension: &[u8],
    setting: &[u8],
) -> Result<(), Problem> {
    text.add(b"missing ")?;
    text.add(dimension)?;
    text.add(b" (")?;
    text.add(authority_source(source))?;
    text.add(b".")?;
    text.add(setting)?;
    text.add(b")")
}

fn authority_lacks(
    text: &mut Text,
    source: jig_domain_authority::Source,
    lacks: jig_domain_authority::Lacks,
) -> Result<(), Problem> {
    let mut first = true;
    for (missing, dimension, setting) in [
        (lacks.tools, b"tool permission".as_slice(), b"tools".as_slice()),
        (lacks.grants, b"resource grant", b"grants"),
        (lacks.executors, b"executor permission", b"delegation.kinds"),
        (lacks.tasks, b"task allowance", b"delegation.tasks"),
        (lacks.depth, b"descendant depth", b"delegation.depth"),
        (lacks.spend, b"spend allowance", b"budget.spend"),
        (lacks.deadline, b"deadline", b"budget.deadline"),
        (lacks.notes, b"note scope", b"notes"),
        (lacks.models, b"model permission", b"models"),
    ] {
        if missing {
            if !first {
                text.add(b", ")?;
            }
            missing_authority(text, source, dimension, setting)?;
            first = false;
        }
    }
    if first {
        text.add(authority_source(source))?;
        text.add(b" authority ceiling")?;
    }
    Ok(())
}

// These refusals name their tasks, matching jig-protocol-tools' render vocabulary.
fn dependency_problem(text: &mut Text, problem: &jig_domain_tasks::Problem) -> Result<(), Problem> {
    use jig_domain_tasks::Refusal;
    let blocked = problem.blocked_by.as_deref().unwrap_or(&[]);
    if let Refusal::Circle { cut } = problem.why {
        text.add(b"these tasks would wait for one another")?;
        for (index, task) in blocked.iter().enumerate() {
            text.add(if index == 0 { b": T" } else { b", T" })?;
            text.number(*task)?;
        }
        if cut && !blocked.is_empty() {
            text.add(b"; some task lists are incomplete")?;
        }
        return Ok(());
    }
    if let Refusal::DependencyEnded { how } = problem.why {
        let Some(task) = blocked.first() else { return text.add(b"a dependency has ended") };
        if how != jig_domain_tasks::Status::Done {
            text.add(b"dependency ")?;
        }
        text.add(b"T")?;
        text.number(*task)?;
        return text.add(match how {
            jig_domain_tasks::Status::Failed => b" failed",
            jig_domain_tasks::Status::Cancelled => b" was cancelled",
            jig_domain_tasks::Status::Done => b" is done",
        });
    }
    if blocked.is_empty() {
        return text.add(b"kept by its dependencies");
    }
    for (index, task) in blocked.iter().enumerate() {
        text.add(if index == 0 { b"kept until T" } else { b", kept until T" })?;
        text.number(*task)?;
        text.add(b" is done")?;
    }
    Ok(())
}

fn task_problem(text: &mut Text, problem: &jig_domain_tasks::Problem) -> Result<(), Problem> {
    use jig_domain_tasks::Refusal;
    if let Refusal::Circle { .. } | Refusal::DependencyEnded { .. } | Refusal::Kept = problem.why {
        return dependency_problem(text, problem);
    }
    if let Some(task) = problem.task {
        text.add(b"task ")?;
        text.number(task)?;
        text.add(b" ")?;
    }
    match problem.why {
        Refusal::Account { account, why } => {
            text.add(b"account ")?;
            text.number(u64::from(account))?;
            text.add(b": ")?;
            text.add(match why {
                jig_domain_tasks::AccountWhy::Unknown => b"not found",
                jig_domain_tasks::AccountWhy::Provider => b"belongs to another provider",
                jig_domain_tasks::AccountWhy::Uses => b"does not serve the payer",
                jig_domain_tasks::AccountWhy::Limited => b"is limited",
            })
        }

        Refusal::Placed { .. } => text.add(b"held by a placed hold"),
        Refusal::HoldOwner { .. } => text.add(b"only the placed hold's owner may release it"),
        Refusal::HoldRevision { .. } => text.add(b"placed hold revision changed"),
        Refusal::NotStanding => text.add(b"the task does not stand"),
        Refusal::NoTalk => text.add(b"no talk of yours to leave"),
        Refusal::TalkOpen { party } => {
            text.add(b"talk open with party ")?;
            text.number(party)
        }
        Refusal::NotReady => text.add(b"not ready"),
        Refusal::Unknown => text.add(b"unknown"),
        Refusal::Busy => {
            text.add(b"busy")?;
            task_capacity(text, problem)
        }
        Refusal::Duplicate => text.add(b"duplicate"),
        Refusal::Empty => text.add(b"empty batch"),
        Refusal::Batch => text.add(b"batch limit"),
        Refusal::Grace => text.add(b"finish grace"),
        Refusal::Goals => text.add(b"goal limit"),
        Refusal::Live => text.add(b"live task limit"),
        Refusal::Project => text.add(b"project"),
        Refusal::Tree => text.add(b"task tree limit"),
        Refusal::Depth => text.add(b"depth limit"),
        Refusal::Delegates => text.add(b"delegate limit"),
        Refusal::Reference => text.add(b"no reference"),
        Refusal::Subscription => text.add(b"subscription"),
        Refusal::Dependencies => {
            if let Some([dependency]) = problem.blocked_by.as_deref() {
                text.add(b"dependency T")?;
                text.number(*dependency)?;
                return text.add(b" is outside your reach");
            }
            text.add(b"dependencies")
        }
        Refusal::Circle { .. } => text.add(b"circle"),
        Refusal::DependencyEnded { .. } => text.add(b"dependency"),
        Refusal::Kept | Refusal::KeptQuestion { .. } => text.add(b"kept"),
        Refusal::PersonAnswers { question } => {
            text.add(b"your person answers question ")?;
            text.number(question)?;
            text.add(b" in their inbox")
        }
        Refusal::Executor => text.add(b"executor"),
        Refusal::Spec => text.add(b"specification"),
        Refusal::Contract => text.add(b"result contract"),
        Refusal::AuthorityShape => text.add(b"authority shape"),
        Refusal::Inputs => text.add(b"historical inputs"),
        Refusal::State => text.add(b"task state"),
        Refusal::Attempt => text.add(b"attempt"),
        Refusal::LiveDelegates => text.add(b"live delegates"),
        Refusal::Restore => text.add(b"restoration"),
        Refusal::Read => text.add(b"read fence"),
        Refusal::Turn => text.add(b"turn order"),
        Refusal::Funding => text.add(b"funding"),
        Refusal::HoldKind => text.add(b"resource hold kind"),
        Refusal::HoldTaken => text.add(b"resource held by"),
        Refusal::ResourceUnavailable => text.add(b"resource unavailable"),
        Refusal::Holds => text.add(b"resource hold limit"),
        Refusal::Unproposable => text.add(b"no one may accept this proposal"),
        Refusal::PersonRevision { .. } => text.add(b"older person task revision"),
        // As jig-protocol-tools' render names each fault.
        Refusal::Llm { why, .. } => text.add(match why {
            jig_domain_tasks::LlmFault::Unconfigured => b"model not configured",
            jig_domain_tasks::LlmFault::Effort => b"effort not declared",
            jig_domain_tasks::LlmFault::Charter => b"model outside the charter",
            jig_domain_tasks::LlmFault::Authority => b"model outside authority",
            jig_domain_tasks::LlmFault::Procedure => b"no model runs this task",
            jig_domain_tasks::LlmFault::Repeated => b"model repeated",
        }),
    }?;
    if let Some(blocked_by) = &problem.blocked_by {
        for task in blocked_by {
            text.add(b" ")?;
            text.number(*task)?;
        }
    }
    Ok(())
}

fn subscription_refusal(
    text: &mut Text,
    task: u64,
    topic: &temper_engine_domain_forge::Topic,
    cause: &[u8],
    limit: Option<u32>,
    existing: Option<u64>,
) -> Result<(), Problem> {
    text.add(b"Refused: task ")?;
    text.number(task)?;
    text.add(b" forge subscription ")?;
    subscription_topic(text, topic)?;
    text.add(b": ")?;
    text.add(cause)?;
    if let Some(existing) = existing {
        text.add(b" (existing subscription ")?;
        text.number(existing)?;
        text.add(b")")?;
    }
    if let Some(limit) = limit {
        text.add(b" (limit ")?;
        text.number(u64::from(limit))?;
        text.add(b")")?;
    }
    Ok(())
}

fn subscription_topic(text: &mut Text, topic: &temper_engine_domain_forge::Topic) -> Result<(), Problem> {
    use temper_engine_domain_forge::Topic;
    let repository = match topic {
        Topic::Findings { project } => {
            text.add(b"findings project ")?;
            return text.number(u64::from(*project));
        }
        Topic::Landings { repository, branch } => {
            text.add(b"landings branch ")?;
            text.add(branch)?;
            *repository
        }
        Topic::Ci { repository, head } => {
            text.add(b"ci head ")?;
            text.commit(head)?;
            *repository
        }
        Topic::Pull { repository, number } | Topic::Participation { repository, number } => {
            text.add(match topic {
                Topic::Pull { .. } => b"pull ",
                Topic::Participation { .. } => b"participation ",
                Topic::Landings { .. } | Topic::Ci { .. } | Topic::Findings { .. } => unreachable!("numbered topic"),
            })?;
            text.number(*number)?;
            *repository
        }
    };
    text.add(b" repository {forge: ")?;
    text.number(u64::from(repository.forge))?;
    text.add(b", repository: ")?;
    text.number(u64::from(repository.repository))?;
    text.add(b"}")
}

fn task_capacity(text: &mut Text, problem: &jig_domain_tasks::Problem) -> Result<(), Problem> {
    use jig_domain_tasks::CapacityCause;
    let Some(capacity) = problem.capacity else { return Ok(()) };
    match capacity.cause {
        CapacityCause::Inbox { task, messages, bytes } => {
            if problem.task == Some(task) {
                text.add(b": its inbox is full")?;
            } else {
                text.add(b": task ")?;
                text.number(task)?;
                text.add(b"'s inbox is full")?;
            }
            text.add(b" (at most ")?;
            text.number(u64::from(messages))?;
            text.add(b" messages and ")?;
            text.number(u64::from(bytes))?;
            text.add(b" bytes, counting the room it keeps for the answers to its open questions, its delegates' results and a proposal's decision); room returns as those arrive and are read: retry then, or ask fewer at once")
        }
        CapacityCause::Proposals { limit } => {
            text.add(b": it has a proposal pending (at most ")?;
            text.number(u64::from(limit))?;
            text.add(b" at once): wait for its decision")
        }
        CapacityCause::Subscriptions { limit } => {
            text.add(b": it holds its most subscriptions (at most ")?;
            text.number(u64::from(limit))?;
            text.add(b", counting the topics the engine keeps for a goal); unsubscribe one it made to make room")
        }
        CapacityCause::PlacedHolds { .. }
        | CapacityCause::ProjectPlacedHolds { .. }
        | CapacityCause::PartyPlacedHolds { .. }
        | CapacityCause::Exemptions { .. }
        | CapacityCause::Tasks { .. }
        | CapacityCause::CreationFlights { .. }
        | CapacityCause::ResourceReports { .. }
        | CapacityCause::ProjectTasks { .. }
        | CapacityCause::Tree { .. }
        | CapacityCause::Delegates { .. }
        | CapacityCause::Holdings { .. }
        | CapacityCause::HoldWaiters { .. }
        | CapacityCause::References { .. }
        | CapacityCause::Funders { .. }
        | CapacityCause::Writers { .. } => Ok(()),
    }
}

fn forge_read(
    text: &mut Text,
    read: &Result<api::Answer, api::Error>,
    tool: &[u8],
    _forge: ForgeReadBounds,
) -> Result<bool, Problem> {
    match read {
        Ok(read) => {
            forge_answer(text, read)?;
            Ok(false)
        }
        Err(refusal) => {
            text.add(b"Forge read failed: ")?;
            forge_error(text, *refusal)?;
            if let api::Error::TooLarge(_) = refusal {
                text.add(b". ")?;
                read_on(text, tool)?;
            } else if tool == b"read_branches" && *refusal == api::Error::Empty {
                text.add(b". ")?;
                branches_empty_recovery(text)?;
            } else if tool == b"read_branch" && *refusal == api::Error::Missing {
                text.add(b". ")?;
                branch_missing_recovery(text)?;
            }
            Ok(true)
        }
    }
}

fn branches_empty_recovery(text: &mut Text) -> Result<(), Problem> {
    const WORDS: &[u8] = b"read_branches needs a nonempty prefix, as temper/ for temper's branches. read_settings names the default branch, and read_branch reads one branch by its name.";
    text.add(WORDS)
}

fn branch_missing_recovery(text: &mut Text) -> Result<(), Problem> {
    const WORDS: &[u8] = b"The forge has no branch of that name in this repository. read_settings names its default branch, and read_branches lists the names under a prefix, as temper/. A goal's own branch is there only where its goal holds one, and a run branch from its first push.";
    text.add(WORDS)
}

fn compare_recovery(text: &mut Text, forge: ForgeReadBounds) -> Result<(), Problem> {
    compare_contents(text, forge)?;
    compare_paged_recovery(text)
}

fn compare_contents(text: &mut Text, forge: ForgeReadBounds) -> Result<(), Problem> {
    text.add(b"read_compare returns a whole comparison, with no pages. Its files and commits each must fit ")?;
    forge_rows(text, forge)?;
    text.add(b"; both whole sides of every changed file must fit ")?;
    forge_answer_bytes(text, forge)?;
    text.add(b".\n")
}

fn forge_rows(text: &mut Text, forge: ForgeReadBounds) -> Result<(), Problem> {
    text.add(b"forge.client.rows (")?;
    text.number(u64::from(forge.rows))?;
    text.add(b")")
}

fn forge_answer_bytes(text: &mut Text, forge: ForgeReadBounds) -> Result<(), Problem> {
    text.add(b"forge.client.answer_bytes (")?;
    text.number(u64::from(forge.answer_bytes))?;
    text.add(b")")
}

fn compare_paged_recovery(text: &mut Text) -> Result<(), Problem> {
    text.add(b"For an existing pull, use read_pull to get its head, then read_pull_files with that head and page=1; advance page while more follows. Pull-file sides must fit whole too. Otherwise, or if a pull-file page is too large, use read_file to list directories at both endpoints, then read selected files at each endpoint within max_bytes.")
}

fn answer_configuration(text: &mut Text) -> Result<(), Problem> {
    text.add(b"To read the whole content, increase the limiting core.call_answer_bytes (including tool name) or agent.json profile.declared.tool_payload (agent.limits.bodies.host_reply_text). Raising core.call_answer_bytes alone cannot overcome a smaller agent allowance.\n")
}

fn forge_error(text: &mut Text, error: api::Error) -> Result<(), Problem> {
    match error {
        api::Error::Unavailable => text.add(b"unavailable"),
        api::Error::Timeout => text.add(b"timed out"),
        api::Error::RateLimited { .. } => text.add(b"rate limited"),
        api::Error::Forbidden => text.add(b"forbidden"),
        api::Error::Missing => text.add(b"missing"),
        api::Error::MissingJob => text.add(b"job missing"),
        api::Error::TooLarge(excess) => {
            text.add(b"too large: past ")?;
            text.add(&temper_engine_domain_forge::bound_words(excess))
        }
        api::Error::Empty => text.add(b"empty"),
        api::Error::Full => text.add(b"full"),
        api::Error::Exists => text.add(b"already exists"),
        api::Error::NothingToMerge => text.add(b"nothing to merge"),
        api::Error::Closed => text.add(b"closed"),
        api::Error::Stale => text.add(b"stale head"),
        api::Error::Conflict => text.add(b"conflict"),
        api::Error::Protected => text.add(b"protected"),
        api::Error::Refused => text.add(b"refused"),
        api::Error::InvalidAnswer => text.add(b"invalid provider answer"),
        api::Error::Busy => text.add(b"busy"),
        api::Error::NotInstalled => text.add(b"not installed"),
        api::Error::NotConnected => text.add(b"not connected"),
    }
}

fn forge_answer(text: &mut Text, answer: &api::Answer) -> Result<(), Problem> {
    forge_answer_at(text, answer, (1, 0))
}
fn page_end(text: &mut Text, more: bool, cut: bool, kept: usize, at: (u32, u32)) -> Result<(), Problem> {
    if cut {
        text.add(b"Page cut: read on with page ")?;
        text.number(u64::from(at.0))?;
        text.add(b", skip ")?;
        text.number(u64::from(at.1).saturating_add(skein_lib::len_u64(kept)))?;
        text.add(b".\n")?;
    } else if more {
        text.add(b"More pages available.\n")?;
    }
    Ok(())
}
#[expect(clippy::too_many_lines, reason = "one exhaustive forge answer vocabulary")]
fn forge_answer_at(text: &mut Text, answer: &api::Answer, at: (u32, u32)) -> Result<(), Problem> {
    match answer {
        api::Answer::Located { repository } => {
            text.add(b"Repository ")?;
            text.number(u64::from(*repository))?;
        }
        api::Answer::Items { items, more, cut, .. } => {
            text.add(b"Items:\n")?;
            for item in items {
                summary(text, item)?;
            }
            if *cut {
                text.add(b"Page cut at forge.client.answer_bytes.\n")?;
            }
            if *more {
                text.add(b"More pages available.\n")?;
            }
        }
        api::Answer::Item { item, comments, more } => {
            summary(text, item)?;
            for comment in comments {
                text.add(b"Comment ")?;
                text.number(comment.id)?;
                text.add(b": ")?;
                text.add(&comment.body)?;
                text.add(b"\n")?;
            }
            if *more {
                text.add(b"More comments available.\n")?;
            }
        }
        api::Answer::Pull(pull) => {
            text.add(b"Pull ")?;
            text.number(pull.number)?;
            text.add(b" head ")?;
            text.add(&pull.head)?;
            text.add(b" commit ")?;
            text.commit(&pull.commit)?;
            text.add(b" base ")?;
            text.add(&pull.base)?;
            text.add(b" state ")?;
            match pull.state {
                api::State::Open => text.add(b"open")?,
                api::State::Closed => text.add(b"closed")?,
            }
            if let Some(merge) = &pull.merged {
                text.add(b" merged commit ")?;
                text.commit(merge)?;
            }
            text.add(b" CI ")?;
            ci(text, pull.ci)?;
        }
        api::Answer::Reviews { reviews, more, cut } => {
            for review in reviews {
                text.add(b"Review ")?;
                text.number(review.id)?;
                text.add(b" by ")?;
                text.number(review.author)?;
                text.add(b": ")?;
                text.add(&review.body)?;
                text.add(b"\n")?;
            }
            page_end(text, *more, *cut, reviews.len(), at)?;
        }
        api::Answer::Statuses { ci: status, statuses, more } => {
            text.add(b"CI ")?;
            ci(text, *status)?;
            text.add(b"\n")?;
            for status in statuses {
                status_text(text, status)?;
            }
            if *more {
                text.add(b"More statuses available.\n")?;
            }
        }
        api::Answer::Remarks { remarks, more, cut } => {
            for remark in remarks {
                text.add(b"Remark on ")?;
                text.add(&remark.path)?;
                text.add(b": ")?;
                text.add(&remark.body)?;
                text.add(b"\n")?;
            }
            page_end(text, *more, *cut, remarks.len(), at)?;
        }
        api::Answer::Commit(commit) | api::Answer::Merged(commit) => text.commit(commit)?,
        api::Answer::Branches(branches) => {
            for branch in branches {
                text.add(branch)?;
                text.add(b"\n")?;
            }
        }
        api::Answer::PullFiles { files, more, cut, .. } => {
            for file in files {
                file_text(text, file)?;
            }
            page_end(text, *more, *cut, files.len(), at)?;
        }
        api::Answer::Compare { before, after, ahead, behind, files, changed } => {
            text.commit(after)?;
            text.add(b" is ")?;
            text.number(*ahead)?;
            text.add(b" commits ahead of ")?;
            text.commit(before)?;
            text.add(b" and ")?;
            text.number(*behind)?;
            text.add(b" behind it.\n")?;
            text.commit(after)?;
            text.add(if *behind == 0 { b" contains " } else { b" lacks " })?;
            text.commit(before)?;
            text.add(b".\nChanged files: ")?;
            text.number(*changed)?;
            text.add(b"\n")?;
            for path in files {
                text.add(path)?;
                text.add(b"\n")?;
            }
            let left = changed.saturating_sub(skein_lib::len_u64(files.len()));
            if left > 0 {
                text.add(b"[")?;
                text.number(left)?;
                text.add(
                    b" not listed: read_pull_files lists a pull's files; read_file reads a path at each commit]\n",
                )?;
            }
        }
        api::Answer::Checks(statuses) => {
            for status in statuses {
                status_text(text, status)?;
            }
        }
        api::Answer::Job { attempt, step, log, truncated } => {
            job_text(text, attempt, step.as_ref(), log, *truncated, u32::try_from(HostAnswer::CAPACITY).expect("cap"))?;
        }
        api::Answer::Directory { head, path, entries, truncated } => {
            directory_text(text, head, path, entries, *truncated)?;
        }
        api::Answer::File { head, path, bytes, truncated } => {
            file_heading(text, head, path)?;
            text.add(bytes)?;
            if *truncated {
                text.add(b"\n[file truncated]")?;
            }
        }
        api::Answer::Protection(protection) => match protection {
            Some(rule) => {
                text.add(b"Protected branch ")?;
                text.add(&rule.branch)?;
                text.add(b" approvals ")?;
                text.number(u64::from(rule.approvals))?;
            }
            None => text.add(b"No protection rule")?,
        },
        api::Answer::Settings(settings) => {
            text.add(b"Default branch ")?;
            text.add(&settings.default_branch)?;
            text.add(b"; merge ")?;
            text.add(if settings.merge { b"yes" } else { b"no" })?;
        }
        api::Answer::Collaborators { collaborators, more } => {
            for collaborator in collaborators {
                text.add(b"User ")?;
                text.number(collaborator.user)?;
                text.add(b"\n")?;
            }
            if *more {
                text.add(b"More collaborators available.\n")?;
            }
        }
        api::Answer::Permission(value) => {
            text.add(match value.permission {
                api::Permission::None => b"No permission",
                api::Permission::Read => b"Read",
                api::Permission::Write => b"Write",
                api::Permission::Admin => b"Admin",
            })?;
            text.add(b"; forge user ")?;
            text.number(value.user)?;
            if !value.login.is_empty() {
                text.add(b", login ")?;
                text.add(&value.login)?;
            }
        }
        api::Answer::Created(number) | api::Answer::Commented(number) | api::Answer::Reviewed(number) => {
            text.number(*number)?;
        }
        api::Answer::Branch(branch) => match branch {
            api::BranchCreation::Created => text.add(b"Branch created")?,
            api::BranchCreation::Exists => text.add(b"Branch already exists")?,
        },
        api::Answer::Runs { .. } => return Err(Problem::Unasked("answer of kind Runs")),
        api::Answer::Run(_) => return Err(Problem::Unasked("answer of kind Run")),
        api::Answer::Commits { .. } => return Err(Problem::Unasked("answer of kind Commits")),
        api::Answer::Tree { .. } => return Err(Problem::Unasked("answer of kind Tree")),
        api::Answer::Queued => return Err(Problem::Unasked("answer of kind Queued")),
        api::Answer::Done => text.add(b"Done")?,
    }
    Ok(())
}

fn summary(text: &mut Text, item: &api::Summary) -> Result<(), Problem> {
    text.add(b"#")?;
    text.number(item.number)?;
    text.add(b" ")?;
    text.add(&item.title)?;
    text.add(b"\n")?;
    text.add(&item.body)?;
    text.add(b"\n")
}

fn ci(text: &mut Text, state: api::Ci) -> Result<(), Problem> {
    match state {
        api::Ci::None => text.add(b"none"),
        api::Ci::Pending => text.add(b"pending"),
        api::Ci::Passed => text.add(b"passed"),
        api::Ci::Failed => text.add(b"failed"),
    }
}

fn status_text(text: &mut Text, status: &api::Status) -> Result<(), Problem> {
    text.add(&status.context)?;
    text.add(b": ")?;
    match status.check {
        api::Check::Pending => text.add(b"pending")?,
        api::Check::Passed => text.add(b"passed")?,
        api::Check::Failed => text.add(b"failed")?,
    }
    text.add(b" ")?;
    text.add(&status.description)?;
    text.add(b" ")?;
    text.add(&status.url)?;
    match status.job {
        Some(attempt) => {
            text.add(b"\nread_job head ")?;
            text.commit(&attempt.head)?;
            text.add(b" run ")?;
            text.number(attempt.run)?;
            text.add(b" job ")?;
            text.number(attempt.job)?;
            text.add(b" attempt ")?;
            text.number(u64::from(attempt.attempt))?;
        }
        None => text.add(b"\nNo job attempt resolved from this check's link.")?,
    }
    text.add(b"\n")
}

fn file_text(text: &mut Text, file: &api::File) -> Result<(), Problem> {
    text.add(&file.path)?;
    text.add(b"\n")?;
    match &file.before {
        Some(before) => text.add(before)?,
        None => text.add(b"[absent]")?,
    }
    text.add(b"\n->\n")?;
    match &file.after {
        Some(after) => text.add(after)?,
        None => text.add(b"[absent]")?,
    }
    text.add(b"\n")
}

fn moved(text: &mut Text, task: u64, now: &jig_domain_people::Moved) -> Result<(), Problem> {
    use jig_domain_people::Moved;
    text.add(b"Moved: task ")?;
    text.number(task)?;
    let (kind, revision) = match now {
        Moved::PlacedHold { revision } => (b" is held at placed hold revision ".as_slice(), *revision),
        Moved::Spec { revision } => (b" is at spec revision ".as_slice(), *revision),
        Moved::Person { revision } => (b" is at person revision ".as_slice(), *revision),
        Moved::Note { revision } => (b" is at note revision ".as_slice(), u64::from(*revision)),
        Moved::Hold { revision, .. } => (b" is held at hold revision ".as_slice(), *revision),
    };
    text.add(kind)?;
    text.number(revision)?;
    text.add(b" now")?;
    if let Moved::Hold { reason, .. } = now {
        text.add(b", ")?;
        text.add(hold_reason(reason))?;
    }
    Ok(())
}

fn hold_reason(reason: &jig_domain_people::HoldReason) -> &'static [u8] {
    use jig_domain_people::{FailureClass, HoldReason};
    match reason {
        HoldReason::Failures { class } => match class {
            FailureClass::Transient => b"for transient failures",
            FailureClass::Permanent => b"for permanent failures",
            FailureClass::Run => b"for run failures",
            FailureClass::Agent => b"for agent failures",
            FailureClass::Lost => b"for lost runs",
            FailureClass::Invalid => b"for invalid results",
            FailureClass::Infrastructure => b"for infrastructure failures",
        },
        HoldReason::Exceeded(_) => b"for its configured model allowance",
        HoldReason::Circle => b"for a circle of waits",
        HoldReason::Stalled => b"because it stalled",
        HoldReason::Dependency { .. } => b"for its dependency",
        HoldReason::Stopped => b"because it was stopped",
        HoldReason::Placed { .. } => b"because a placed hold holds it",
        HoldReason::Drift => b"for drift",
        HoldReason::Effects => b"for its effects",
        HoldReason::EffectFailed => b"for a failed effect",
        HoldReason::Uncertain { .. } => b"for an uncertain effect",
        HoldReason::Budget => b"for its budget",
        HoldReason::Deadline => b"for its deadline",
        HoldReason::HoldsWaited => b"for waiting on holds",
        HoldReason::ResourceUnavailable => b"for an unavailable resource",
        HoldReason::ResourceAccess => b"for resource access",
        HoldReason::PoolLost { .. } => b"for a lost pool",
        HoldReason::Procedure => b"for its procedure",
    }
}

fn settings_refused(text: &mut Text, why: &jig_domain_people::Refusal) -> Result<(), Problem> {
    text.add(&crate::settings::refusal(why))
}

fn hold_owner(
    text: &mut Text,
    task: u64,
    hold: u64,
    revision: u64,
    owner: jig_domain_people::HoldOwner,
) -> Result<(), Problem> {
    text.add(b"Refused: task ")?;
    text.number(task)?;
    text.add(b" is held by hold ")?;
    text.number(hold)?;
    text.add(b" (revision ")?;
    text.number(revision)?;
    text.add(b"), placed by ")?;
    match owner {
        jig_domain_people::HoldOwner::Party { person } => {
            text.add(b"person ")?;
            text.number(person)?;
            text.add(b"; only they may release it. Propose it: ")?;
        }
        jig_domain_people::HoldOwner::Task { task } => {
            text.add(b"task ")?;
            text.number(task)?;
            text.add(b"; only it or a task above it may release it. Propose it: ")?;
        }
    }
    text.add(b"{\"action\": \"release\", \"task\": ")?;
    text.number(task)?;
    text.add(b", \"hold\": ")?;
    text.number(hold)?;
    text.add(b", \"revision\": ")?;
    text.number(revision)?;
    text.add(b", \"reason\": \"...\"}")
}

fn directory_text(
    text: &mut Text,
    head: &api::Commit,
    path: &[u8],
    entries: &[api::Entry],
    truncated: bool,
) -> Result<(), Problem> {
    text.add(if path.is_empty() { b"." } else { path })?;
    text.add(b" at ")?;
    text.commit(head)?;
    text.add(b" (a directory):\n")?;
    for (index, entry) in entries.iter().enumerate() {
        if index != 0 {
            text.add(b"\n")?;
        }
        text.add(&entry.name)?;
        match entry.kind {
            api::EntryKind::File => {}
            api::EntryKind::Directory => text.add(b"/")?,
            api::EntryKind::Submodule => text.add(b" (submodule)")?,
        }
    }
    if truncated {
        if !entries.is_empty() {
            text.add(b"\n")?;
        }
        text.add(b"[directory cut]")?;
    }
    Ok(())
}

fn task_read(
    read: &Result<jig_domain_core::reads::run_task::TaskRead, jig_domain_tasks::Refusal>,
    maximum: usize,
) -> Result<HostAnswer, Problem> {
    let mut text = Text::new();
    let read = match read {
        Ok(read) => read,
        Err(why) => {
            text.add(b"Task read refused: ")?;
            if *why == jig_domain_tasks::Refusal::Unknown {
                text.add(b"unknown: no such task, no result yet, or no text at that offset. A task has a result only once it has ended, and its requester hears it then as a result message; while it runs, read its specification (leave out field) for its state, holds and questions.")?;
            } else {
                task_problem(&mut text, &jig_domain_tasks::Problem::new(None, *why))?;
            }
            if text.len > maximum {
                return Err(Problem::TooLarge);
            }
            return text.finish(true);
        }
    };
    task_read_metadata(&mut text, read)?;
    if read.result_pending {
        text.add(b"\nResult: none yet; it comes when the task ends. Its specification follows; continue it with field left out.")?;
    }
    if let Some(code) = read.verdict_code {
        text.add(b"\nVerdict: ")?;
        text.number(u64::from(code))?;
    }
    text.add(match read.words.field {
        jig_domain_core::reads::TaskTextField::Specification => b"\nWords from ",
        jig_domain_core::reads::TaskTextField::Result => b"\nResult from ",
        jig_domain_core::reads::TaskTextField::Instructions => b"\nInstructions from ",
    })?;
    text.number(read.words.from)?;
    text.add(b":\n")?;
    task_read_words(&mut text, &read.words, maximum)?;
    text.finish(false)
}

fn task_read_metadata(text: &mut Text, read: &jig_domain_core::reads::run_task::TaskRead) -> Result<(), Problem> {
    use jig_domain_core::reads::run_task::TaskState;
    text.add(b"Task ")?;
    text.number(read.task)?;
    text.add(b", spec revision ")?;
    text.number(read.spec_revision)?;
    text.add(b"\nState: ")?;
    text.add(match read.state {
        TaskState::Waiting => b"waiting",
        TaskState::Active(_) => b"active",
        TaskState::Closing(_) => b"closing",
        TaskState::Held => b"held",
        TaskState::Ended(jig_domain_tasks::Status::Done) => b"done",
        TaskState::Ended(jig_domain_tasks::Status::Failed) => b"failed",
        TaskState::Ended(jig_domain_tasks::Status::Cancelled) => b"cancelled",
    })?;
    text.add(b"\nDependencies:")?;
    for task in &read.dependencies {
        text.add(b" ")?;
        text.number(*task)?;
    }
    task_read_questions(text, read)?;
    if let Some(hold) = &read.hold {
        text.add(b"\nHold revision ")?;
        text.number(hold.revision)?;
        text.add(b": ")?;
        task_hold_reason(text, hold.reason)?;
        if let Some(words) = &hold.words {
            text.add(b"\n")?;
            text.add(words)?;
        }
    }
    Ok(())
}

fn task_hold_reason(text: &mut Text, reason: jig_domain_tasks::Hold) -> Result<(), Problem> {
    use jig_domain_tasks::{Class, Hold};
    let (label, number): (&[u8], Option<u64>) = match reason {
        Hold::Dependency(task) => (b"dependency ", Some(task)),
        Hold::Placed { hold } => (b"placed hold ", Some(hold)),
        Hold::Uncertain { entry } => (b"effect outcome uncertain, entry ", Some(entry)),
        Hold::PoolLost { pool } => (b"pool lost ", Some(pool)),
        Hold::Exceeded(exceeded) => return task_exceeded(text, exceeded),
        Hold::Failures(class) => {
            return text.add(match class {
                Class::Transient => b"retry allowance exhausted: transient failures",
                Class::Permanent => b"retry allowance exhausted: permanent failures",
                Class::Run => b"retry allowance exhausted: run failures",
                Class::Agent => b"retry allowance exhausted: agent failures",
                Class::Lost => b"retry allowance exhausted: lost runs",
                Class::Invalid => b"retry allowance exhausted: invalid results",
                Class::Infrastructure => b"retry allowance exhausted: infrastructure failures",
            });
        }
        Hold::Stalled => (b"executor stalled", None),
        Hold::Stopped => (b"stopped", None),
        Hold::Drift => (b"resource drift", None),
        Hold::ResourceUnavailable => (b"resource unavailable", None),
        Hold::ResourceAccess => (b"resource access", None),
        Hold::Effects => (b"effects", None),
        Hold::EffectFailed => (b"effect failed", None),
        Hold::Budget => (b"budget spent", None),
        Hold::Deadline => (b"deadline", None),
        Hold::HoldsWaited => (b"hold wait exceeded", None),
        Hold::Procedure => (b"procedure held", None),
        Hold::Circle => (b"circle of waits", None),
    };
    text.add(label)?;
    match number {
        Some(number) => text.number(number),
        None => Ok(()),
    }
}

fn task_exceeded(text: &mut Text, exceeded: jig_domain_tasks::Exceeded) -> Result<(), Problem> {
    use jig_domain_tasks::Bound;
    text.add(b"configured limit exceeded: ")?;
    text.add(match exceeded.bound {
        Bound::Reasoning => b"reasoning bytes ",
        Bound::Metadata => b"metadata bytes ",
        Bound::Items => b"answer items ",
        Bound::Answer => b"answer bytes ",
        Bound::Tools => b"tools ",
        Bound::HistoryItems => b"history items ",
        Bound::History => b"history ",
        Bound::Window => b"window tokens ",
        Bound::Output => b"output tokens ",
    })?;
    text.number(exceeded.value)
}

fn task_read_words(text: &mut Text, words: &jig_domain_core::reads::TaskText, maximum: usize) -> Result<(), Problem> {
    // A continuation contains a newline, label and at most twenty decimal digits.
    const CONTINUATION: usize = 27;
    if words.next.is_none() && text.len.saturating_add(escaped_len(&words.bytes)) <= maximum {
        return text.add(&words.bytes);
    }
    let room =
        maximum.checked_sub(text.len).ok_or(Problem::TooLarge)?.checked_sub(CONTINUATION).ok_or(Problem::TooLarge)?;
    let prefix = fitting_prefix(&words.bytes, room);
    if prefix.is_empty() && !words.bytes.is_empty() {
        return Err(Problem::TooLarge);
    }
    text.add(prefix)?;
    text.add(b"\nnext: ")?;
    text.number(words.from.checked_add(skein_lib::len_u64(prefix.len())).ok_or(Problem::TooLarge)?)
}

fn task_read_questions(text: &mut Text, read: &jig_domain_core::reads::run_task::TaskRead) -> Result<(), Problem> {
    for question in &read.questions {
        text.add(b"\nAnswer target ")?;
        text.number(read.task)?;
        text.add(b", question ")?;
        text.number(question.number)?;
        text.add(b" answerer ")?;
        let (kind, number): (&[u8], u64) = match question.answerer {
            jig_domain_tasks::Party::Task(number) => (b"task ", number),
            jig_domain_tasks::Party::Person(number) => (b"person ", number),
            jig_domain_tasks::Party::Deployment { project } => (b"deployment for project ", u64::from(project)),
        };
        text.add(kind)?;
        text.number(number)?;
        text.add(b", activity ")?;
        text.number(question.line)?;
    }
    Ok(())
}

fn refused_problem(
    text: &mut Text,
    problem: &jig_domain_tasks::Problem,
    recover_question: bool,
) -> Result<bool, Problem> {
    text.add(b"Refused: ")?;
    task_problem(text, problem)?;
    if recover_question && problem.why == jig_domain_tasks::Refusal::Reference {
        text.add(b". Use read_task on the target task to find its open question number and authorized answerer; send message with target, form=answer and question set to that number.")?;
    }
    Ok(true)
}

fn forge_refused_size(text: &mut Text, excess: api::Excess) -> Result<(), Problem> {
    text.add(b"Forge read failed: ")?;
    forge_error(text, api::Error::TooLarge(excess))?;
    text.add(b".\n")
}

fn read_on(text: &mut Text, tool: &[u8]) -> Result<(), Problem> {
    match tool {
        b"read_file" => file_recovery(text),
        b"read_pull_files" => pull_recovery(text),
        b"read_compare" => compare_paged_recovery(text),
        b"read_branches" => text.add(b"Use a longer prefix, or read_branch for one name."),
        b"read_item" => text.add(b"Advance after to read later comments; read_items lists item summaries."),
        b"read_items" | b"read_reviews" | b"read_statuses" | b"read_remarks" | b"read_collaborators" => {
            text.add(b"Advance page to list the rest; a later page skips this oversized page.")
        }
        b"read_checks" => text.add(b"Use read_statuses by page, then read_job for a failed job."),
        b"read_job" => text.add(b"Read the same attempt with smaller max_bytes."),
        b"read_pull" | b"read_pull_for" => text.add(
            b"Use read_items for a summary and read_pull_files by page or read_file by path at the head or the base.",
        ),
        b"read_branch" => text.add(b"Use read_branches with a longer prefix."),
        b"read_protection" | b"read_settings" | b"read_permission" => {
            text.add(b"This object has no pages; read_items or read_branch for narrower context.")
        }
        _ => text.add(b"Read a narrower object by page or path with read_file."),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn note_answers_give_the_first_writes_line_count() {
        for (lines, expected) in
            [(0, "Note 12 revision 1 written · 0 lines"), (64, "Note 12 revision 1 written · 64 lines")]
        {
            let rendered =
                answer(&CallAnswer::NoteWritten { name: 12, revision: 1, lines, added: lines, removed: 0, first: 1 });
            assert_eq!(rendered.text(), expected.as_bytes());
            assert!(!rendered.error());
        }
    }

    #[test]
    fn note_answers_give_later_writes_changes_and_first_line() {
        for (added, removed, first, expected) in [
            (18, 6, 31, "Note 12 revision 3 written · +18 −6 · line 31 · 64 lines"),
            (0, 0, 1, "Note 12 revision 3 written · +0 −0 · line 1 · 64 lines"),
        ] {
            let rendered = answer(&CallAnswer::NoteWritten { name: 12, revision: 3, lines: 64, added, removed, first });
            assert_eq!(rendered.text(), expected.as_bytes());
            assert!(!rendered.error());
        }
    }
    #[test]
    fn note_refusals_say_what_to_do_next() {
        for (why, expected) in [
            (jig_domain_notes::Refusal::Moved { revision: 7 }, "Note refused: revision moved, now 7"),
            (jig_domain_notes::Refusal::PastEnd { lines: 64 }, "Note refused: range past the end, 64 lines"),
            (
                jig_domain_notes::Refusal::PastBound { bytes: 70000, bound: 65536 },
                "Note refused: body past its bound, 70000 bytes of 65536",
            ),
        ] {
            let rendered = answer(&CallAnswer::NoteRefused(why));
            assert_eq!(rendered.text(), expected.as_bytes());
            assert!(rendered.error());
        }
    }
}
