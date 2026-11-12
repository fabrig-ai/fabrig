//! Forge-owned brief sections, gathered from current provider facts
//! (jig's domain/connectors.md, section 9).
use crate::brief_words::{self, bounds, busy, moved, room, unnamed};
use crate::{BriefCommit, BriefItem, BriefRead, BriefSource, Domain, Request};
use alloc::boxed::Box;
use skein_lib::{List, Queue, Token};
use temper_engine_domain_forge_change as change;

/// The home-access reason's 209 fixed bytes plus three owner/repository pairs.
/// Adoption bounds each name by `Limits::name_bytes`, whose ceiling is
/// `PANEL_NAME_BYTES` (1,024 bytes for each owner and repository name).
pub const HOME_SECTION_BYTES: u32 = 209 + 6 * crate::PANEL_NAME_BYTES;

/// The adoption hold's words (domain/forge/branches.md), bounded by the
/// caller's read allotment and the longest admitted names. Omitted bytes follow
/// the other forge sections' cuts.
#[must_use]
pub fn home_access_reason(owner: &[u8], repository: &[u8], bytes: u32) -> BriefRead {
    let mut words = BriefWords::with_capacity(bytes.min(HOME_SECTION_BYTES), false);
    let parts: [&[u8]; 12] = [
        b"This chat began before ",
        owner,
        b"/",
        repository,
        b" was adopted, so you can read ",
        owner,
        b"/",
        repository,
        b" but not push to it. Tell the person, in your next reply: \"This chat began before ",
        owner,
        b"/",
        repository,
    ];
    for part in parts {
        append(&mut words, part);
    }
    append(&mut words, b" was adopted, so I can read it but not push. Start a new chat to push.\"");
    brief_part(&words, 1, bytes)
}

#[derive(Hash, Debug)]
pub(crate) struct BriefFetch {
    owner: Token,
    task: Option<u64>,
    identity: u32,
    item: BriefItem,
    head: BriefCommit,
    repository: skein_forge::api::Repository,
    parts: u32,
    bytes: u32,
    stage: BriefStage,
    read: skein_forge::api::Read,
    refused: Option<u64>,
    deadline: Option<skein_lib::Time>,
    mode: PullMode,
    base: Option<skein_forge::api::Commit>,
    comparison: Option<(skein_forge::api::Commit, skein_forge::api::Commit)>,
    changed: Box<[Box<[u8]>]>,
    file_page: u32,
    file_skip: u32,
    request: Option<RequestReviews>,
    review_ids: Box<[u64]>,
    next_review: u32,
    checks: Box<[skein_forge::api::Status]>,
    next_check: u32,
    max_job_bytes: u32,
    words: BriefWords,
    ci: Option<crate::brief_ci::Section>,
}

/// A request quotes the reviewed head while the section stays pinned to the change's head
/// (domain/forge/change/repairs.md).
#[derive(Hash, Debug)]
struct RequestReviews {
    reviewed: BriefCommit,
    has_remarks: bool,
}

#[derive(Hash, Debug)]
pub(crate) struct BriefWords {
    bytes: List<u8>,
    left: u64,
    tail: bool,
}

impl BriefWords {
    pub(crate) fn with_capacity(capacity: u32, tail: bool) -> BriefWords {
        BriefWords { bytes: List::with_capacity(capacity), left: 0, tail }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum BriefStage {
    Item,
    Pull,
    Files,
    Compare,
    Reviews,
    Remarks,
    Checks,
    Run,
    Job,
}

#[derive(Clone, Copy, Hash, Debug)]
enum PullMode {
    Review,
    Conflict(skein_forge::api::Commit),
    Semantic(Option<skein_forge::api::Commit>),
    Other,
}

pub(crate) fn append(out: &mut BriefWords, bytes: &[u8]) {
    for byte in bytes {
        if out.bytes.room() > 0 {
            out.bytes.push(*byte).expect("checked message room");
        } else {
            out.left = out.left.saturating_add(1);
            if out.tail && !out.bytes.is_empty() {
                for index in 1..out.bytes.len() {
                    *out.bytes.get_mut(index.saturating_sub(1)).expect("Within words") =
                        *out.bytes.get(index).expect("Within words");
                }
                *out.bytes.get_mut(out.bytes.len().saturating_sub(1)).expect("Nonempty words") = *byte;
            }
        }
    }
}

/// One byte's two lowercase hexadecimal digits, the high nibble first.
#[must_use]
pub fn hex(byte: u8) -> [u8; 2] {
    let high = *b"0123456789abcdef".get(usize::from(byte >> 4_u8)).expect("high nibble is hexadecimal");
    let low = *b"0123456789abcdef".get(usize::from(byte & 15_u8)).expect("low nibble is hexadecimal");
    [high, low]
}

pub(crate) fn brief_part(words: &BriefWords, parts: u32, bytes: u32) -> BriefRead {
    if parts == 0 {
        return BriefRead::Failed(bounds());
    }
    let kept = words.bytes.as_slice().len().min(skein_lib::usize_of(bytes));
    let start = if words.tail { words.bytes.as_slice().len().saturating_sub(kept) } else { 0 };
    BriefRead::Got {
        bytes: Box::from(words.bytes.as_slice().get(start..start.saturating_add(kept)).expect("Kept is within source")),
        left: words.left.saturating_add(
            u64::try_from(words.bytes.as_slice().len().checked_sub(kept).expect("Kept is within source"))
                .expect("bounded section"),
        ),
    }
}

/// Reviews and repairs share the pull identity (temper/docs/design/domain/forge/change/gates.md, 1 and temper/docs/design/domain/forge/change/repairs.md).
fn pull_identity(domain: &Domain, row: &crate::ChangeRow, head: BriefCommit, bytes: u32) -> Option<BriefWords> {
    let number = row.pull?;
    let mut words = BriefWords::with_capacity(bytes, false);
    for part in [
        b"Pull request: #".as_slice(),
        decimal(number).as_ref(),
        b"\nBranch: ",
        &row.branch,
        b"\nBase: ",
        &row.base,
        b"\nHead: ",
        &crate::commit_text(&head.0),
        b"\n",
    ] {
        append(&mut words, part);
    }
    if let Some(configured) = domain.configured(row.repository.forge)
        && let Some(repository) = domain.repository(row.repository)
        && let Some(address) = crate::pull_address(
            configured.kind,
            &configured.git,
            configured.web.as_deref(),
            &repository.owner,
            &repository.name,
            number,
        )
    {
        append(&mut words, b"Address: ");
        append(&mut words, &address);
        append(&mut words, b"\n");
    }
    if words.left > 0 {
        return None;
    }
    Some(words)
}

pub(super) fn brief_pull(
    domain: &mut Domain,
    owner: Token,
    item: BriefItem,
    head: BriefCommit,
    parts: u32,
    bytes: u32,
    out: &mut Queue<Request>,
) -> Option<BriefRead> {
    let Some(row) = domain.change_for_pull(item.repository, item.number) else {
        return Some(BriefRead::Failed(unnamed()));
    };
    if row.change.last_head != Some(head.0) {
        return Some(moved_change(domain, owner, item, head, BriefStage::Pull, out));
    }
    if parts == 0 {
        return Some(BriefRead::Failed(bounds()));
    }
    if domain.brief_fetches.len() == domain.brief_fetches.capacity() {
        return Some(BriefRead::Failed(room()));
    }
    let repository = row.repository;
    let mode = pull_mode(&row.change.state, &row.change.clean);
    let Some(words) = pull_identity(domain, row, head, bytes) else { return Some(BriefRead::Failed(unnamed())) };
    let call = Token::new(owner.raw() | (1_u64 << 61_u32) | (1_u64 << 59_u32));
    let fetch = BriefFetch {
        owner,
        task: None,
        identity: words.bytes.len(),
        item,
        head,
        repository,
        parts,
        bytes,
        stage: BriefStage::Item,
        read: skein_forge::api::Read::Item { number: item.number, after: 0, since: skein_lib::Time::ZERO },
        refused: None,
        deadline: None,
        mode,
        base: None,
        comparison: None,
        changed: Box::new([]),
        file_page: 1,
        file_skip: 0,
        request: None,
        review_ids: Box::new([]),
        next_review: 0,
        checks: Box::new([]),
        next_check: 0,
        max_job_bytes: 0,
        words,
        ci: None,
    };
    domain.brief_fetches.insert(call, fetch).expect("brief read room checked");
    out.push(Request::BriefClient {
        event: skein_forge_client::Event::Read {
            owner: call,
            repository,
            read: skein_forge::api::Read::Item { number: item.number, after: 0, since: skein_lib::Time::ZERO },
        },
    });
    // The brief child waits for the connector's bounded answer.
    None
}

#[expect(clippy::too_many_lines, reason = "the bounded forge brief fetch handles each connector reply in order")]
pub(crate) fn brief_answer(
    domain: &mut Domain,
    owner: Token,
    result: Result<skein_forge::api::Answer, skein_forge::api::Error>,
    rows: u32,
    out: &mut Queue<Request>,
) -> bool {
    if domain.document_fetches.contains_key(&owner) {
        return crate::brief_document::answer(domain, owner, result, out);
    }
    let Some(mut fetch) = domain.brief_fetches.remove(&owner) else { return false };
    if completed_item(domain, fetch.item) {
        drop_section(domain, fetch.owner);
        return true;
    }
    if result == Err(skein_forge::api::Error::Busy) {
        fetch.refused = Some(domain.brief_reclaim);
        domain.brief_fetches.insert(owner, fetch).expect("same refused brief room");
        return true;
    }
    let unread = brief_words::unread_of(&result);
    let reason = brief_words::reason_of(&result);
    let error = match &result {
        Ok(_) => None,
        Err(error) => Some(*error),
    };
    let next = match result {
        Ok(skein_forge::api::Answer::Item { item, .. })
            if fetch.stage == BriefStage::Item && item.number == fetch.item.number =>
        {
            match fetch.mode {
                PullMode::Review | PullMode::Other => append(&mut fetch.words, b"Pull request at head "),
                PullMode::Conflict(_) => append(&mut fetch.words, b"Resolve the conflict from change head "),
                PullMode::Semantic(_) => append(&mut fetch.words, b"Repair after the base merged into change head "),
            }
            append(&mut fetch.words, &crate::commit_text(&fetch.head.0));
            append(&mut fetch.words, b"\nTitle: ");
            append(&mut fetch.words, &item.title);
            if fetch.words.left > 0 {
                crate::held::ready(domain, fetch.owner, BriefRead::Failed(unnamed()), out);
                return true;
            }
            append(&mut fetch.words, b"\nBody: ");
            append(&mut fetch.words, &item.body);
            append(&mut fetch.words, b"\n");
            fetch.stage = BriefStage::Pull;
            Some(skein_forge::api::Read::Pull { number: fetch.item.number, ci: false })
        }
        Ok(skein_forge::api::Answer::Pull(pull)) if fetch.stage == BriefStage::Pull => {
            // temper/docs/design/domain/forge/change/repairs.md,
            // "A pull section is pinned at the change's head": any attempt of
            // this brief's task may have pushed beyond the recorded pin.
            if pull.commit != fetch.head.0 {
                if !own_writer(domain, &fetch) {
                    refused(
                        fetch.owner,
                        fetch.stage,
                        "the change's head moved",
                        Some((fetch.head.0, pull.commit)),
                        None,
                        out,
                    );
                    wake_change(domain, fetch.item);
                    crate::held::ready(domain, fetch.owner, BriefRead::Failed(moved()), out);
                    return true;
                }
                own_head(&mut fetch, pull.commit);
            }
            fetch.base = pull.base_commit;
            fetch.stage = BriefStage::Files;
            Some(skein_forge::api::Read::PullFiles {
                number: fetch.item.number,
                head: fetch.head.0,
                page: 1,
                skip: 0,
                sides: true,
            })
        }
        Err(skein_forge::api::Error::Stale) if fetch.stage == BriefStage::Files => {
            refused(
                fetch.owner,
                fetch.stage,
                "the change's head moved",
                None,
                Some(skein_forge::api::Error::Stale),
                out,
            );
            wake_change(domain, fetch.item);
            crate::held::ready(domain, fetch.owner, BriefRead::Failed(moved()), out);
            return true;
        }
        Ok(skein_forge::api::Answer::PullFiles { head, files, more, cut })
            if fetch.stage == BriefStage::Files && head == fetch.head.0 =>
        {
            if fetch.file_page == 1 && fetch.file_skip == 0 {
                append(&mut fetch.words, b"Files and diff:\n");
            }
            let nonempty = !files.is_empty();
            let kept = u32::try_from(files.len()).expect("bounded files");
            let mut changed = List::with_capacity(rows);
            if fetch.file_skip > 0 {
                for path in &fetch.changed {
                    if changed.room() > 0 {
                        changed.push(path.clone()).expect("checked path room");
                    }
                }
            }
            for file in files {
                if changed.room() > 0 {
                    changed.push(file.path.clone()).expect("checked path room");
                }
                match fetch.mode {
                    PullMode::Conflict(_) => {
                        if name_conflict_if_past_room(&mut fetch.words, &file) {
                            continue;
                        }
                    }
                    PullMode::Review | PullMode::Semantic(_) | PullMode::Other => {}
                }
                append(&mut fetch.words, b"File: ");
                append(&mut fetch.words, &file.path);
                append(&mut fetch.words, b"\nBefore:\n");
                if let Some(before) = file.before {
                    append(&mut fetch.words, &before);
                }
                append(&mut fetch.words, b"\nAfter:\n");
                if let Some(after) = file.after {
                    append(&mut fetch.words, &after);
                }
                append(&mut fetch.words, b"\n");
            }
            fetch.changed = changed.into_boxed();
            // A review needs each page's contents. Every nonempty page consumes
            // section room, so the byte bound also bounds these sequential reads.
            let review = match fetch.mode {
                PullMode::Review => true,
                PullMode::Conflict(_) | PullMode::Semantic(_) | PullMode::Other => false,
            };
            if (review || cut) && more {
                if nonempty && fetch.words.bytes.room() > 0 {
                    if cut {
                        fetch.file_skip = fetch.file_skip.saturating_add(kept);
                    } else {
                        fetch.file_page = fetch.file_page.checked_add(1).expect("pages bounded by section bytes");
                        fetch.file_skip = 0;
                    }
                    let read = skein_forge::api::Read::PullFiles {
                        number: fetch.item.number,
                        head: fetch.head.0,
                        page: fetch.file_page,
                        skip: fetch.file_skip,
                        sides: true,
                    };
                    fetch.read = read.clone();
                    let repository = fetch.repository;
                    domain.brief_fetches.insert(owner, fetch).expect("same brief read room");
                    out.push(Request::BriefClient {
                        event: skein_forge_client::Event::Read { owner, repository, read },
                    });
                    return true;
                }
                append(&mut fetch.words, b"[more files omitted]\n");
            }
            comparison(&mut fetch)
        }
        Err(skein_forge::api::Error::TooLarge(excess)) if fetch.stage == BriefStage::Files => {
            refused(fetch.owner, fetch.stage, reason, None, error, out);
            append(&mut fetch.words, b"[files omitted from page ");
            append(&mut fetch.words, &decimal(u64::from(fetch.file_page)));
            append(&mut fetch.words, b": past ");
            append(&mut fetch.words, &crate::bound_words(excess));
            append(&mut fetch.words, b"; read_pull_files from page ");
            let next_page = fetch.file_page.checked_add(1).expect("pages bounded by section bytes");
            append(&mut fetch.words, &decimal(u64::from(next_page)));
            append(&mut fetch.words, b" lists the rest, and read_file reads a path at the head or the base]\n");
            comparison(&mut fetch)
        }
        Ok(skein_forge::api::Answer::Compare { before, after, files, ahead, changed, .. })
            if fetch.stage == BriefStage::Compare && Some((before, after)) == fetch.comparison =>
        {
            comparison_words(&mut fetch, files, ahead, changed, rows);
            None
        }
        Err(skein_forge::api::Error::TooLarge(excess)) if fetch.stage == BriefStage::Compare => {
            refused(fetch.owner, fetch.stage, reason, None, error, out);
            append(&mut fetch.words, b"[comparison omitted: past ");
            append(&mut fetch.words, &crate::bound_words(excess));
            append(&mut fetch.words, b"]\n");
            None
        }
        Ok(skein_forge::api::Answer::Reviews { reviews, .. }) if fetch.stage == BriefStage::Reviews => {
            let mut ids = List::with_capacity(u32::try_from(reviews.len()).expect("bounded review list"));
            for review in reviews {
                let reviewed = match &fetch.request {
                    Some(request) => request.reviewed,
                    None => fetch.head,
                };
                if review.commit == reviewed.0 {
                    if let Some(request) = &mut fetch.request {
                        request.has_remarks |= !review.body.is_empty();
                    }
                    append(&mut fetch.words, b"Review: ");
                    append(&mut fetch.words, &review.body);
                    append(&mut fetch.words, b"\n");
                    ids.push(review.id).expect("sized from bounded review list");
                }
            }
            fetch.review_ids = ids.into_boxed();
            next_remarks(&mut fetch)
        }
        Ok(skein_forge::api::Answer::Remarks { remarks, .. }) if fetch.stage == BriefStage::Remarks => {
            for remark in remarks {
                if let Some(request) = &mut fetch.request {
                    request.has_remarks = true;
                }
                append(&mut fetch.words, b"Remark at ");
                append(&mut fetch.words, &remark.path);
                append(&mut fetch.words, b":");
                append(&mut fetch.words, &decimal(u64::from(remark.line)));
                append(&mut fetch.words, b" ");
                append(&mut fetch.words, &remark.body);
                append(&mut fetch.words, b"\n");
            }
            next_remarks(&mut fetch)
        }
        Ok(skein_forge::api::Answer::Checks(checks)) if fetch.stage == BriefStage::Checks => {
            let count = u32::try_from(checks.len()).expect("bounded check list");
            fetch.ci = Some(crate::brief_ci::Section::new(fetch.head, count, fetch.bytes));
            fetch.checks = checks;
            next_failed_job(&mut fetch)
        }
        answer if fetch.stage == BriefStage::Run => {
            let attempt = fetch
                .checks
                .get(skein_lib::usize_of(fetch.next_check.saturating_sub(1)))
                .expect("current check")
                .job
                .expect("pinned job");
            if let Ok(skein_forge::api::Answer::Run(run)) = answer
                && let Some(reason) = crate::change_ci::runner(&run, attempt, fetch.head.0)
            {
                append(&mut fetch.words, b"the runner's: ");
                append(&mut fetch.words, reason);
                let reruns = attempt.attempt.saturating_sub(1);
                if reruns > 0 {
                    append(&mut fetch.words, b"; run again ");
                    append(&mut fetch.words, &decimal(u64::from(reruns)));
                    append(&mut fetch.words, if reruns == 1 { b" time" } else { b" times" });
                    append(&mut fetch.words, b" at this head, failed again");
                }
                append(&mut fetch.words, b"\n");
            }
            Some(job_log(&mut fetch, attempt))
        }
        answer if fetch.stage == BriefStage::Job => {
            match answer {
                Ok(skein_forge::api::Answer::Job { attempt, step, log, truncated }) if attempt.head == fetch.head.0 => {
                    if let Some(skein_forge::api::Step { number, name }) = step {
                        append(&mut fetch.words, b"Failed step: ");
                        append(&mut fetch.words, &decimal(u64::from(number)));
                        if !name.is_empty() {
                            append(&mut fetch.words, b" ");
                            append(&mut fetch.words, &name);
                        }
                        append(&mut fetch.words, b"\nEnd of its log:\n");
                    } else {
                        append(&mut fetch.words, b"No step failed. End of the job's log:\n");
                    }
                    keep_check(&mut fetch, log, truncated);
                }
                Ok(_) | Err(_) => {
                    refused(fetch.owner, fetch.stage, reason, None, error, out);
                    append(&mut fetch.words, b"[job log could not be read]\n");
                    keep_check(&mut fetch, Box::new([]), false);
                }
            }
            next_failed_job(&mut fetch)
        }
        Ok(_) | Err(_) => {
            refused(fetch.owner, fetch.stage, reason, None, error, out);
            if unread.fate == crate::BriefFate::Capacity || !crate::held::stand_on_partner(domain, fetch.owner) {
                crate::held::ready(domain, fetch.owner, BriefRead::Failed(unread), out);
                return true;
            }
            let marker = match fetch.stage {
                BriefStage::Item | BriefStage::Pull | BriefStage::Files | BriefStage::Compare => {
                    b"[the pull request could not be read: ".as_slice()
                }
                BriefStage::Reviews | BriefStage::Remarks => b"[the reviews could not be read: ",
                BriefStage::Checks => b"[the failed checks could not be read: ",
                BriefStage::Job | BriefStage::Run => unreachable!("job failures keep their check's marker"),
            };
            append(&mut fetch.words, marker);
            append(&mut fetch.words, &unread.words);
            append(&mut fetch.words, b"]\n");
            None
        }
    };
    if let Some(read) = next {
        fetch.read = read.clone();
        let repository = fetch.repository;
        domain.brief_fetches.insert(owner, fetch).expect("same brief read room");
        out.push(Request::BriefClient { event: skein_forge_client::Event::Read { owner, repository, read } });
    } else if let Some(ci) = fetch.ci {
        crate::held::ready_ci(domain, fetch.owner, ci, fetch.bytes, out);
    } else {
        if let Some(request) = &fetch.request
            && !request.has_remarks
        {
            append(&mut fetch.words, b"[the request for changes gave no remarks]\n");
        }
        let read = brief_part(&fetch.words, fetch.parts, fetch.bytes);
        crate::held::ready(domain, fetch.owner, read, out);
    }
    true
}

/// The writer slot, not the change's current delegate, identifies this brief's
/// own push (temper/docs/design/domain/forge/change/repairs.md, pinned pull sections).
fn own_writer(domain: &Domain, fetch: &BriefFetch) -> bool {
    let Some(task) = fetch.task else { return false };
    let Some(row) = domain.change_for_pull(fetch.item.repository, fetch.item.number) else { return false };
    for (name, hold) in &domain.holds {
        if name.forge == row.repository.forge
            && name.repository == row.repository.repository
            && crate::items::branch_matches(name, &row.branch)
        {
            return match hold.writer {
                Some(crate::Writer::Run { task: writer, .. } | crate::Writer::Lost { task: writer, .. }) => {
                    writer == task
                }
                Some(crate::Writer::Entry(_)) | None => false,
            };
        }
    }
    false
}

/// Insert the own-head notice after the identity, within the same section room
/// (temper/docs/design/domain/forge/change/repairs.md, pinned pull sections).
fn own_head(fetch: &mut BriefFetch, head: skein_forge::api::Commit) {
    let mut words = BriefWords::with_capacity(fetch.bytes, false);
    let at = skein_lib::usize_of(fetch.identity);
    append(&mut words, fetch.words.bytes.as_slice().get(..at).expect("identity precedes content"));
    append(&mut words, b"[the pull's head is ");
    let read = crate::commit_text(&head);
    let pinned = crate::commit_text(&fetch.head.0);
    append(&mut words, read.get(..12).expect("commit has twelve digits"));
    append(&mut words, b", past the change's recorded ");
    append(&mut words, pinned.get(..12).expect("commit has twelve digits"));
    append(&mut words, b": an earlier attempt of this task pushed it, and this section reads at ");
    append(&mut words, read.get(..12).expect("commit has twelve digits"));
    append(&mut words, b"]\n");
    append(&mut words, fetch.words.bytes.as_slice().get(at..).expect("content follows identity"));
    words.left = words.left.saturating_add(fetch.words.left);
    fetch.words = words;
    fetch.head = BriefCommit(head);
}

fn wake_change(domain: &mut Domain, item: BriefItem) {
    if let Some(row) = domain.change_for_pull(item.repository, item.number) {
        let task = row.task;
        domain.procedure_wakes.insert(task, true).expect("one pending wake per change");
    }
}

/// A stale plan pauses without spending a try and wakes its change to judge the
/// move (temper/docs/design/domain/forge/change/repairs.md, pinned pull sections).
fn moved_change(
    domain: &mut Domain,
    section: Token,
    item: BriefItem,
    pinned: BriefCommit,
    stage: BriefStage,
    out: &mut Queue<Request>,
) -> BriefRead {
    let heads = match domain.change_for_pull(item.repository, item.number) {
        Some(row) => match row.change.last_head {
            Some(head) if head != pinned.0 => Some((pinned.0, head)),
            Some(_) | None => None,
        },
        None => None,
    };
    refused(section, stage, "the change's head moved", heads, None, out);
    wake_change(domain, item);
    BriefRead::Failed(moved())
}

/// Name a conflicting file whose whole contents exceed the section's room
/// (temper/docs/design/domain/forge/change/repairs.md, A resolution).
fn name_conflict_if_past_room(words: &mut BriefWords, file: &skein_forge::api::File) -> bool {
    let before = match file.before.as_deref() {
        Some(bytes) => bytes,
        None => b"",
    };
    let after = match file.after.as_deref() {
        Some(bytes) => bytes,
        None => b"",
    };
    let mut whole = 0_u64;
    for part in [b"File: ".as_slice(), &file.path, b"\nBefore:\n", before, b"\nAfter:\n", after, b"\n"] {
        whole = whole.saturating_add(u64::try_from(part.len()).expect("bounded file bytes"));
    }
    if whole <= u64::from(words.bytes.room()) {
        return false;
    }
    append(words, b"File: ");
    append(words, &file.path);
    append(words, b"\n[not carried, past the brief's room: ");
    match file.before.as_deref() {
        Some(bytes) => {
            append(words, &decimal(u64::try_from(bytes.len()).expect("bounded file bytes")));
            append(words, b" bytes before");
        }
        None => append(words, b"absent before"),
    }
    append(words, b", ");
    match file.after.as_deref() {
        Some(bytes) => {
            append(words, &decimal(u64::try_from(bytes.len()).expect("bounded file bytes")));
            append(words, b" bytes after");
        }
        None => append(words, b"absent after"),
    }
    append(words, b"; read it in your checkout]\n");
    true
}

/// Every pull section goes from its last gathered or omitted files page to its comparison
/// (temper/docs/design/domain/forge/change/repairs.md).
fn comparison(fetch: &mut BriefFetch) -> Option<skein_forge::api::Read> {
    let comparison = match fetch.mode {
        PullMode::Review | PullMode::Other => match fetch.base {
            Some(base) => Some((base, fetch.head.0)),
            None => None,
        },
        PullMode::Conflict(base) => Some((fetch.head.0, base)),
        PullMode::Semantic(before) => match before {
            Some(before) => Some((before, fetch.head.0)),
            None => None,
        },
    };
    fetch.comparison = comparison;
    match comparison {
        Some((before, after)) => {
            fetch.stage = BriefStage::Compare;
            Some(skein_forge::api::Read::Compare { before, after })
        }
        None => None,
    }
}

/// Comparison names follow the pinned pull identity and gathered diff (domain/forge/change/repairs.md).
fn comparison_words(fetch: &mut BriefFetch, files: Box<[Box<[u8]>]>, ahead: u64, changed: u64, rows: u32) {
    let base = match fetch.mode {
        PullMode::Conflict(_) | PullMode::Semantic(_) => true,
        PullMode::Review | PullMode::Other => false,
    };
    if base {
        append(&mut fetch.words, b"What landed in the base: ");
        append(&mut fetch.words, &decimal(ahead));
        append(&mut fetch.words, b" commits\n");
    } else {
        append(&mut fetch.words, b"Against base:\n");
    }
    for path in &files {
        if base {
            append(&mut fetch.words, b"Base file: ");
        }
        append(&mut fetch.words, path);
        append(&mut fetch.words, b"\n");
        if let PullMode::Conflict(_) = fetch.mode {
            for own in &fetch.changed {
                if own == path {
                    append(&mut fetch.words, b"Conflicting file: ");
                    append(&mut fetch.words, path);
                    append(&mut fetch.words, b"\n");
                }
            }
        }
    }
    let left = changed.saturating_sub(skein_lib::len_u64(files.len()));
    if left > 0 {
        append(&mut fetch.words, b"[");
        append(&mut fetch.words, &decimal(left));
        append(
            &mut fetch.words,
            if base {
                b" more base files not listed: past forge.client.rows ("
            } else {
                b" more files not listed: past forge.client.rows ("
            },
        );
        append(&mut fetch.words, &decimal(u64::from(rows)));
        append(
            &mut fetch.words,
            if base { b"); read them in your checkout]\n" } else { b"); read_pull_files lists them]\n" },
        );
    }
}

/// Start gathering a connector-owned section while its parent holds only
/// the section token and its budget.
#[expect(clippy::too_many_arguments, reason = "one connector gather names its owner, source and bounded read")]
pub(crate) fn gather(
    domain: &mut Domain,
    owner: Token,
    source: BriefSource,
    parts: u32,
    bytes: u32,
    max_job_bytes: u32,
    limit: u32,
    out: &mut Queue<Request>,
) {
    let completed = match source {
        BriefSource::Ci { item, .. }
        | BriefSource::Reviews { item, .. }
        | BriefSource::Request { item, .. }
        | BriefSource::Pull { item, .. } => completed_item(domain, item),
        BriefSource::Document | BriefSource::Home { .. } => false,
    };
    if completed {
        drop_section(domain, owner);
        return;
    }
    if bytes > limit {
        crate::held::ready(domain, owner, BriefRead::Failed(bounds()), out);
        return;
    }
    let read = match source {
        BriefSource::Document => {
            crate::brief_document::gather(domain, owner, parts, max_job_bytes, out);
            None
        }
        BriefSource::Home { repository, design } => Some(match domain.repository(repository) {
            Some(_) if parts > 0 && design => {
                let mut words = BriefWords::with_capacity(bytes.min(HOME_SECTION_BYTES), false);
                append(&mut words, b"deliver pushes the checkout's Markdown straight to the goal branch, merging its moved tip first; code goes through change.");
                brief_part(&words, 1, bytes)
            }
            Some(home) if parts > 0 => home_access_reason(&home.owner, &home.name, bytes),
            Some(_) => BriefRead::Failed(bounds()),
            None => BriefRead::Failed(unnamed()),
        }),
        BriefSource::Ci { item, head } => brief_ci(domain, owner, item, head, parts, bytes, max_job_bytes, out),
        BriefSource::Reviews { item, head } => brief_reviews(domain, owner, item, head, None, parts, bytes, out),
        BriefSource::Request { item, head, reviewed } => {
            brief_reviews(domain, owner, item, head, Some(reviewed), parts, bytes, out)
        }
        BriefSource::Pull { item, head } => brief_pull(domain, owner, item, head, parts, bytes, out),
    };
    if let Some(read) = read {
        crate::held::ready(domain, owner, read, out);
    }
}

fn next_remarks(fetch: &mut BriefFetch) -> Option<skein_forge::api::Read> {
    let index = skein_lib::usize_of(fetch.next_review);
    let review = *fetch.review_ids.get(index)?;
    fetch.next_review = fetch.next_review.checked_add(1).expect("bounded review count");
    fetch.stage = BriefStage::Remarks;
    Some(skein_forge::api::Read::Remarks { number: fetch.item.number, review, page: 1, skip: 0 })
}

/// Append each failed status before asking for its pinned attempt's log.
fn next_failed_job(fetch: &mut BriefFetch) -> Option<skein_forge::api::Read> {
    let total = u32::try_from(fetch.checks.len()).expect("bounded check list");
    for index in fetch.next_check..total {
        let status = fetch.checks.get(skein_lib::usize_of(index)).expect("index in check list");
        fetch.next_check = index.checked_add(1).expect("bounded check count");
        if status.check != skein_forge::api::Check::Failed {
            continue;
        }
        if fetch.ci.as_ref().expect("CI fetch").has_omissions() {
            fetch.ci.as_mut().expect("CI fetch").omit();
            continue;
        }
        append(&mut fetch.words, b"Failed check: ");
        append(&mut fetch.words, &status.context);
        append(&mut fetch.words, b"\nDescription: ");
        append(&mut fetch.words, &status.description);
        append(&mut fetch.words, b"\nLink: ");
        append(&mut fetch.words, &status.url);
        append(&mut fetch.words, b"\n");
        match status.job {
            Some(attempt) if attempt.head == fetch.head.0 => {
                fetch.stage = BriefStage::Run;
                return Some(skein_forge::api::Read::Run { run: attempt.run });
            }
            Some(_) | None => {
                append(&mut fetch.words, b"[no pinned job log for this head]\n");
                keep_check(fetch, Box::new([]), false);
            }
        }
    }
    None
}

/// Keep a whole check head; an oversized head ends the admitted prefix.
fn job_log(fetch: &mut BriefFetch, attempt: skein_forge::api::JobAttempt) -> skein_forge::api::Read {
    fetch.stage = BriefStage::Job;
    let count = u32::try_from(fetch.checks.len()).expect("bounded checks").max(1);
    let share = fetch.bytes.checked_div(count).expect("positive check count").checked_div(2).expect("positive divisor");
    skein_forge::api::Read::Job {
        part: skein_forge::api::LogPart::End,
        attempt,
        max_bytes: share.max(1).min(fetch.max_job_bytes),
    }
}

fn keep_check(fetch: &mut BriefFetch, log: Box<[u8]>, truncated: bool) {
    let words = core::mem::replace(&mut fetch.words, BriefWords::with_capacity(fetch.bytes, false));
    let ci = fetch.ci.as_mut().expect("CI fetch");
    if words.left > 0 {
        ci.omit();
    } else {
        ci.push(words.bytes.into_boxed(), log, truncated);
    }
}

#[expect(clippy::too_many_arguments, reason = "a request pins both the change head and the reviewed head")]
pub(super) fn brief_reviews(
    domain: &mut Domain,
    owner: Token,
    item: BriefItem,
    head: BriefCommit,
    reviewed: Option<BriefCommit>,
    parts: u32,
    bytes: u32,
    out: &mut Queue<Request>,
) -> Option<BriefRead> {
    let Some(row) = domain.change_for_pull(item.repository, item.number) else {
        return Some(BriefRead::Failed(unnamed()));
    };
    if row.change.last_head != Some(head.0) {
        return Some(moved_change(domain, owner, item, head, BriefStage::Reviews, out));
    }
    if parts == 0 {
        return Some(BriefRead::Failed(bounds()));
    }
    if domain.brief_fetches.len() == domain.brief_fetches.capacity() {
        return Some(BriefRead::Failed(room()));
    }
    let repository = row.repository;
    let mut words = BriefWords::with_capacity(bytes, false);
    for remark in &row.gate_remarks {
        if reviewed.is_none() && remark.head == head.0 {
            append(&mut words, b"Gate remarks: ");
            append(&mut words, &remark.words);
            append(&mut words, b"\n");
        }
    }
    let call = Token::new(owner.raw() | (1_u64 << 61_u32) | (1_u64 << 59_u32));
    let fetch = BriefFetch {
        owner,
        task: None,
        identity: words.bytes.len(),
        item,
        head,
        repository,
        parts,
        bytes,
        stage: BriefStage::Reviews,
        read: skein_forge::api::Read::Reviews { number: item.number, page: 1, skip: 0 },
        refused: None,
        deadline: None,
        mode: PullMode::Other,
        base: None,
        comparison: None,
        changed: Box::new([]),
        file_page: 1,
        file_skip: 0,
        request: match reviewed {
            Some(reviewed) => Some(RequestReviews { reviewed, has_remarks: false }),
            None => None,
        },
        review_ids: Box::new([]),
        next_review: 0,
        checks: Box::new([]),
        next_check: 0,
        max_job_bytes: 0,
        words,
        ci: None,
    };
    domain.brief_fetches.insert(call, fetch).expect("brief read room checked");
    out.push(Request::BriefClient {
        event: skein_forge_client::Event::Read {
            owner: call,
            repository,
            read: skein_forge::api::Read::Reviews { number: item.number, page: 1, skip: 0 },
        },
    });
    None
}

#[expect(clippy::too_many_arguments, reason = "the pinned CI section carries its read and job-log bounds")]
pub(super) fn brief_ci(
    domain: &mut Domain,
    owner: Token,
    item: BriefItem,
    head: BriefCommit,
    parts: u32,
    bytes: u32,
    max_job_bytes: u32,
    out: &mut Queue<Request>,
) -> Option<BriefRead> {
    let Some(row) = domain.change_for_pull(item.repository, item.number) else {
        return Some(BriefRead::Failed(unnamed()));
    };
    let change::State::Repairing { .. } = row.change.state else {
        return Some(moved_change(domain, owner, item, head, BriefStage::Checks, out));
    };
    if row.change.last_head != Some(head.0) {
        return Some(moved_change(domain, owner, item, head, BriefStage::Checks, out));
    }
    if parts == 0 {
        return Some(BriefRead::Failed(bounds()));
    }
    if domain.brief_fetches.len() == domain.brief_fetches.capacity() {
        return Some(BriefRead::Failed(room()));
    }
    let repository = row.repository;
    let words = BriefWords::with_capacity(bytes, false);
    let call = Token::new(owner.raw() | (1_u64 << 61_u32) | (1_u64 << 59_u32));
    let fetch = BriefFetch {
        owner,
        task: None,
        identity: words.bytes.len(),
        item,
        head,
        repository,
        parts,
        bytes,
        stage: BriefStage::Checks,
        read: skein_forge::api::Read::Checks { commit: head.0 },
        refused: None,
        deadline: None,
        mode: PullMode::Other,
        base: None,
        comparison: None,
        changed: Box::new([]),
        file_page: 1,
        file_skip: 0,
        request: None,
        review_ids: Box::new([]),
        next_review: 0,
        checks: Box::new([]),
        next_check: 0,
        max_job_bytes,
        words,
        ci: None,
    };
    domain.brief_fetches.insert(call, fetch).expect("brief read room checked");
    out.push(Request::BriefClient {
        event: skein_forge_client::Event::Read {
            owner: call,
            repository,
            read: skein_forge::api::Read::Checks { commit: head.0 },
        },
    });
    None
}

fn decimal(number: u64) -> Box<[u8]> {
    let mut digits = List::with_capacity(20);
    let mut value = number;
    for _ in 0_u32..20_u32 {
        digits
            .push(b'0'.checked_add(u8::try_from(value % 10).expect("decimal digit")).expect("decimal ascii digit"))
            .expect("u64 decimal width");
        value /= 10;
        if value == 0 {
            break;
        }
    }
    let mut forward = List::with_capacity(20);
    for at in (0..digits.len()).rev() {
        forward.push(*digits.get(at).expect("measured decimal digit")).expect("same width");
    }
    forward.into_boxed()
}

fn completed_item(domain: &Domain, item: BriefItem) -> bool {
    match domain.change_for_pull(item.repository, item.number) {
        Some(row) => row.terminal(),
        None => false,
    }
}

/// Admission refusals wait across the client's reclamation boundary.
fn waiting_owner(domain: &Domain) -> Option<Token> {
    for (owner, fetch) in &domain.brief_fetches {
        if let Some(reclaim) = fetch.refused
            && reclaim != domain.brief_reclaim
        {
            return Some(*owner);
        }
    }
    None
}

pub(crate) fn waiting(domain: &Domain) -> bool {
    waiting_owner(domain).is_some() || crate::brief_document::waiting(domain)
}

pub(crate) fn retry(domain: &mut Domain, out: &mut Queue<Request>) {
    let Some(owner) = waiting_owner(domain) else {
        crate::brief_document::retry(domain, out);
        return;
    };
    let mut fetch = domain.brief_fetches.remove(&owner).expect("waiting brief");
    if completed_item(domain, fetch.item) {
        drop_section(domain, fetch.owner);
        return;
    }
    fetch.refused = None;
    out.push(Request::BriefClient {
        event: skein_forge_client::Event::Read { owner, repository: fetch.repository, read: fetch.read.clone() },
    });
    domain.brief_fetches.insert(owner, fetch).expect("same waiting brief room");
}

fn section_owner(domain: &Domain, section: Token) -> Option<Token> {
    for (owner, fetch) in &domain.brief_fetches {
        if fetch.owner == section {
            return Some(*owner);
        }
    }
    None
}

pub(crate) fn set_context(domain: &mut Domain, section: Token, task: u64, deadline: skein_lib::Time) {
    crate::brief_document::set_deadline(domain, section, deadline);
    if let Some(owner) = section_owner(domain, section) {
        let fetch = domain.brief_fetches.get_mut(&owner).expect("section fetch");
        fetch.deadline = Some(deadline);
        fetch.task = Some(task);
    }
}

pub(crate) fn next_deadline(domain: &Domain) -> Option<skein_lib::Time> {
    let mut next = crate::brief_document::next_deadline(domain);
    for (_, fetch) in &domain.brief_fetches {
        if let Some(deadline) = fetch.deadline
            && fetch.refused.is_some()
        {
            next = Some(match next {
                Some(previous) => previous.min(deadline),
                None => deadline,
            });
        }
    }
    next
}

fn expired_owner(domain: &Domain, now: skein_lib::Time) -> Option<Token> {
    for (owner, fetch) in &domain.brief_fetches {
        if let Some(deadline) = fetch.deadline
            && deadline <= now
            && fetch.refused.is_some()
        {
            return Some(*owner);
        }
    }
    None
}

/// Finish Busy-retained reads at the parent's deadline; in-flight reads stay late.
pub(crate) fn expire(domain: &mut Domain, now: skein_lib::Time, out: &mut Queue<Request>) {
    crate::brief_document::expire(domain, now, out);
    for _ in 0..domain.brief_fetches.capacity() {
        let Some(owner) = expired_owner(domain, now) else { break };
        let mut fetch = domain.brief_fetches.remove(&owner).expect("expired brief");
        refused(fetch.owner, fetch.stage, "the forge client is busy", None, Some(skein_forge::api::Error::Busy), out);
        if fetch.stage == BriefStage::Job || fetch.stage == BriefStage::Run {
            // Each remaining pinned job has the same unavailable-log marker.
            for _ in 0..=fetch.checks.len() {
                append(&mut fetch.words, b"[job log could not be read]\n");
                keep_check(&mut fetch, Box::new([]), false);
                if next_failed_job(&mut fetch).is_none() {
                    break;
                }
            }
        }
        if let Some(ci) = fetch.ci {
            crate::held::ready_ci(domain, fetch.owner, ci, fetch.bytes, out);
        } else {
            crate::held::ready(domain, fetch.owner, BriefRead::Failed(busy()), out);
        }
    }
}

pub(crate) fn drop_section(domain: &mut Domain, section: Token) {
    domain.document_fetches.remove(&crate::brief_document::call(section));
    if let Some(owner) = section_owner(domain, section) {
        domain.brief_fetches.remove(&owner);
    }
    domain.brief_pending.remove(&section);
}

fn pull_mode(state: &change::State, clean: &[[u8; 32]]) -> PullMode {
    match state {
        change::State::Resolving { base, .. } => PullMode::Conflict(*base),
        change::State::Repairing { why: change::Repair::Semantic, .. } => PullMode::Semantic(clean.last().copied()),
        change::State::Gating { .. } => PullMode::Review,
        change::State::Producing { .. }
        | change::State::WaitingProducer { .. }
        | change::State::Opening { .. }
        | change::State::Recreating { .. }
        | change::State::Reopening { .. }
        | change::State::CancellingGates { .. }
        | change::State::Checking { .. }
        | change::State::Queued { .. }
        | change::State::First { .. }
        | change::State::Ready { .. }
        | change::State::Preparing { .. }
        | change::State::Updating { .. }
        | change::State::Repairing { .. }
        | change::State::Landing { .. }
        | change::State::NoChange { .. }
        | change::State::Ended
        | change::State::Landed { .. }
        | change::State::Red { .. }
        | change::State::Cleared { .. }
        | change::State::Uncleared { .. }
        | change::State::Held { .. } => PullMode::Other,
    }
}

fn refused(
    section: Token,
    stage: BriefStage,
    reason: &'static str,
    heads: Option<(skein_forge::api::Commit, skein_forge::api::Commit)>,
    error: Option<skein_forge::api::Error>,
    out: &mut Queue<Request>,
) {
    let stage = match stage {
        BriefStage::Item => "item",
        BriefStage::Pull => "pull",
        BriefStage::Files => "files",
        BriefStage::Compare => "compare",
        BriefStage::Reviews => "reviews",
        BriefStage::Remarks => "remarks",
        BriefStage::Checks => "checks",
        BriefStage::Job => "job",
        BriefStage::Run => "run",
    };
    out.push(Request::BriefReadRefused { section, stage, reason, heads, error });
}
