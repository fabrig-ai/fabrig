use super::api;
use super::{Bounds, Undecodable, names, topics, w};
use alloc::boxed::Box;
use skein_codec::Record;
use skein_forge::api as d;
use skein_lib::{List, Reader, Writer};
pub(super) fn encode_permission(value: d::Permission) -> w::Permission {
    crate::generated::store::permission_to_record(&value, &super::Context::UNBOUNDED).expect(super::FITS)
}
pub(super) fn decode_permission(value: w::Permission) -> d::Permission {
    crate::generated::store::permission_to_domain(value, &super::Context::UNBOUNDED).expect(super::FITS)
}
fn encode_branch_creation(value: d::BranchCreation) -> w::BranchCreation {
    crate::generated::store::branch_creation_to_record(&value, &super::Context::UNBOUNDED).expect(super::FITS)
}
fn decode_branch_creation(value: w::BranchCreation) -> d::BranchCreation {
    crate::generated::store::branch_creation_to_domain(value, &super::Context::UNBOUNDED).expect(super::FITS)
}
fn encode_reviewers(values: &[u64]) -> List<u64> {
    let mut result = List::with_capacity(u32::try_from(values.len()).expect("admitted count"));
    for value in values {
        result.push(*value).expect("same count");
    }
    result
}
pub(super) fn encode_pull(value: &d::Pull) -> w::Pull {
    let d::Pull {
        reviewers,
        number,
        state,
        head,
        base,
        commit,
        base_commit,
        merged,
        mergeable,
        ci,
        queued: _,
        dequeued: _,
        size: _,
    } = value;
    w::Pull {
        reviewers: encode_reviewers(reviewers),
        number: *number,
        state: topics::encode_state(*state),
        head: head.clone(),
        base: base.clone(),
        commit: *commit,
        base_commit: *base_commit,
        merged: *merged,
        mergeable: *mergeable,
        ci: topics::encode_ci(*ci),
    }
}
pub(super) fn decode_pull(value: w::Pull) -> d::Pull {
    let w::Pull { reviewers, number, state, head, base, commit, base_commit, merged, mergeable, ci } = value;
    d::Pull {
        reviewers: reviewers.into_boxed(),
        number,
        state: topics::decode_state(state),
        head,
        base,
        commit,
        base_commit,
        merged,
        mergeable,
        ci: topics::decode_ci(ci),
        queued: None,
        dequeued: None,
        size: None,
    }
}
fn encode_step(value: &d::Step) -> w::Step {
    crate::generated::store::step_to_record(value, &super::Context::UNBOUNDED).expect(super::FITS)
}
fn decode_step(value: w::Step) -> d::Step {
    crate::generated::store::step_to_domain(value, &super::Context::UNBOUNDED).expect(super::FITS)
}
pub(super) fn encode_job_attempt(value: &d::JobAttempt) -> w::JobAttempt {
    crate::generated::store::job_attempt_to_record(value, &super::Context::UNBOUNDED).expect(super::FITS)
}
pub(super) fn decode_job_attempt(value: w::JobAttempt) -> d::JobAttempt {
    crate::generated::store::job_attempt_to_domain(value, &super::Context::UNBOUNDED).expect(super::FITS)
}
pub(super) fn encode(value: &d::Answer) -> w::Answer {
    match value {
        d::Answer::Located { repository } => w::Answer::Located(w::Located { repository: *repository }),
        d::Answer::Statuses { ci, statuses, more } => w::Answer::Statuses(encode_statuses_answer(*ci, statuses, *more)),
        d::Answer::Remarks { remarks, more, cut } => w::Answer::Remarks(encode_remarks_answer(remarks, *more, *cut)),
        d::Answer::PullFiles { head, files, more, cut } => {
            w::Answer::PullFiles(encode_pull_files_answer(*head, files, *more, *cut))
        }
        d::Answer::Compare { before, after, ahead, behind, files, changed } => {
            w::Answer::Compare(encode_compare_answer(*before, *after, *ahead, *behind, files, *changed))
        }
        d::Answer::Checks(statuses) => w::Answer::Checks(encode_checks_answer(statuses)),
        d::Answer::Collaborators { collaborators, more } => {
            w::Answer::Collaborators(encode_collaborators_answer(collaborators, *more))
        }

        d::Answer::JobState { .. }
        | d::Answer::Runs { .. }
        | d::Answer::Run(_)
        | d::Answer::Commits { .. }
        | d::Answer::Tree { .. } => {
            unreachable!("queue does not persist Actions or tree answers")
        }
        d::Answer::Items { items, more, now, cut } => w::Answer::Items(encode_items_answer(items, *more, *now, *cut)),
        d::Answer::Item { item, comments, more } => w::Answer::Item(encode_item_answer(item, comments, *more)),
        d::Answer::Reviews { reviews, more, cut } => w::Answer::Reviews(encode_reviews_answer(reviews, *more, *cut)),
        d::Answer::Pull(value) => w::Answer::Pull(encode_pull(value)),
        d::Answer::Commit(commit) => w::Answer::Commit(w::Commit { commit: *commit }),
        d::Answer::Branches(values) => w::Answer::Branches(w::BranchesAnswer { branches: names::encode_piece(values) }),
        d::Answer::Job { attempt, step, log, truncated } => w::Answer::Job(w::JobAnswer {
            attempt: encode_job_attempt(attempt),
            step: match step {
                Some(step) => Some(encode_step(step)),
                None => None,
            },
            log: log.clone(),
            truncated: *truncated,
        }),
        d::Answer::Directory { head, path, entries, truncated } => w::Answer::Directory(w::DirectoryAnswer {
            head: *head,
            path: path.clone(),
            entries: encode_directory(entries),
            truncated: *truncated,
        }),
        d::Answer::File { head, path, bytes, truncated } => w::Answer::File(w::FileAnswer {
            head: *head,
            path: path.clone(),
            bytes: bytes.clone(),
            truncated: *truncated,
        }),
        d::Answer::Protection(value) => w::Answer::Protection(w::ProtectionAnswer {
            protection: match value {
                Some(value) => Some(encode_protection(value)),
                None => None,
            },
        }),
        d::Answer::Settings(value) => w::Answer::Settings(names::encode_settings(value)),
        d::Answer::Permission(value) => w::Answer::Permission(w::PermissionValue {
            permission: encode_permission(value.permission),
            user: value.user,
            login: value.login.clone(),
        }),
        d::Answer::Created(number) => w::Answer::Created(w::Number { number: *number }),
        d::Answer::Commented(number) => w::Answer::Commented(w::Number { number: *number }),
        d::Answer::Reviewed(number) => w::Answer::Reviewed(w::Number { number: *number }),
        d::Answer::Merged(commit) => w::Answer::Merged(w::Commit { commit: *commit }),
        d::Answer::Branch(value) => {
            w::Answer::Branch(w::BranchCreationValue { creation: encode_branch_creation(*value) })
        }
        d::Answer::Queued => w::Answer::Queued,
        d::Answer::Done => w::Answer::Done,
    }
}
pub(super) fn decode(value: w::Answer, bounds: &Bounds) -> Result<d::Answer, Undecodable> {
    Ok(match value {
        w::Answer::Queued => d::Answer::Queued,
        w::Answer::Located(value) => d::Answer::Located { repository: value.repository },
        w::Answer::Statuses(value) => {
            let w::StatusesAnswer { ci, statuses, more } = value;
            d::Answer::Statuses { ci: topics::decode_ci(ci), statuses: decode_statuses(&statuses, bounds)?, more }
        }
        w::Answer::Remarks(value) => {
            let w::RemarksAnswer { remarks, more, cut } = value;
            d::Answer::Remarks { remarks: decode_remarks(&remarks, bounds)?, more, cut }
        }
        w::Answer::PullFiles(value) => {
            let w::PullFilesAnswer { head, files, more, cut } = value;
            d::Answer::PullFiles { head, files: decode_files(&files, bounds)?, more, cut }
        }
        w::Answer::Compare(value) => {
            let w::CompareAnswer { before, after, ahead, behind, files, changed } = value;
            d::Answer::Compare { before, after, ahead, behind, changed, files: files.into_boxed() }
        }
        w::Answer::Checks(value) => {
            let w::ChecksAnswer { statuses } = value;
            d::Answer::Checks(decode_statuses(&statuses, bounds)?)
        }
        w::Answer::Collaborators(value) => {
            let w::CollaboratorsAnswer { collaborators, more } = value;
            d::Answer::Collaborators { collaborators: decode_collaborators(collaborators), more }
        }

        w::Answer::Items(value) => {
            let w::ItemsAnswer { items, more, now, cut } = value;
            d::Answer::Items {
                cut,
                items: decode_summaries(&items, bounds)?,
                more,
                now: skein_lib::Time::from_nanos(now),
            }
        }
        w::Answer::Item(value) => {
            let w::ItemAnswer { item, comments, more } = value;
            d::Answer::Item {
                item: api::decode_summary(item, bounds)?,
                comments: decode_comments(&comments, bounds)?,
                more,
            }
        }
        w::Answer::Reviews(value) => {
            let w::ReviewsAnswer { reviews, more, cut } = value;
            d::Answer::Reviews { reviews: decode_reviews(&reviews, bounds)?, more, cut }
        }
        w::Answer::Pull(value) => d::Answer::Pull(decode_pull(value)),
        w::Answer::Commit(value) => d::Answer::Commit(value.commit),
        w::Answer::Branches(value) => d::Answer::Branches(names::decode_branches(&value.branches, bounds)?),
        w::Answer::Job(value) => {
            let w::JobAnswer { attempt, step, log, truncated } = value;
            d::Answer::Job {
                attempt: decode_job_attempt(attempt),
                step: match step {
                    Some(step) => Some(decode_step(step)),
                    None => None,
                },
                log,
                truncated,
            }
        }
        w::Answer::Directory(value) => {
            let w::DirectoryAnswer { head, path, entries, truncated } = value;
            let room = bounds
                .forge()
                .client
                .answer_bytes
                .checked_sub(u32::try_from(path.len()).or(Err(Undecodable::Malformed))?)
                .ok_or(Undecodable::Malformed)?;
            d::Answer::Directory { head, path, entries: decode_directory(&entries, bounds, room)?, truncated }
        }
        w::Answer::File(value) => {
            let w::FileAnswer { head, path, bytes, truncated } = value;
            d::Answer::File { head, path, bytes, truncated }
        }
        w::Answer::Protection(value) => d::Answer::Protection(match value.protection {
            Some(value) => Some(decode_protection(value, bounds)?),
            None => None,
        }),
        w::Answer::Settings(value) => d::Answer::Settings(names::decode_settings(value)),
        w::Answer::Permission(value) => {
            let w::PermissionValue { permission, user, login } = value;
            d::Answer::Permission(d::UserPermission { permission: decode_permission(permission), user, login })
        }
        w::Answer::Created(value) => d::Answer::Created(value.number),
        w::Answer::Commented(value) => d::Answer::Commented(value.number),
        w::Answer::Reviewed(value) => d::Answer::Reviewed(value.number),
        w::Answer::Merged(value) => d::Answer::Merged(value.commit),
        w::Answer::Branch(value) => d::Answer::Branch(decode_branch_creation(value.creation)),
        w::Answer::Done => d::Answer::Done,
    })
}
pub(super) fn encode_protection(value: &d::Protection) -> w::Protection {
    let d::Protection { branch, contexts, approvals, dismiss_stale, queue, signatures, linear, code_owners, bypass } =
        value;
    w::Protection {
        branch: branch.clone(),
        contexts: names::encode_piece(contexts),
        approvals: *approvals,
        dismiss_stale: *dismiss_stale,
        queue: *queue,
        signatures: *signatures,
        linear: *linear,
        code_owners: *code_owners,
        bypass: match bypass {
            d::Bypass::Never => w::Bypass::Never,
            d::Bypass::PullsOnly => w::Bypass::PullsOnly,
            d::Bypass::Always => w::Bypass::Always,
        },
    }
}
pub(super) fn decode_protection(value: w::Protection, bounds: &Bounds) -> Result<d::Protection, Undecodable> {
    let w::Protection { branch, contexts, approvals, dismiss_stale, queue, signatures, linear, code_owners, bypass } =
        value;
    Ok(d::Protection {
        queue,
        signatures,
        linear,
        code_owners,
        bypass: match bypass {
            w::Bypass::Never => d::Bypass::Never,
            w::Bypass::PullsOnly => d::Bypass::PullsOnly,
            w::Bypass::Always => d::Bypass::Always,
        },
        branch,
        contexts: names::decode_contexts(&contexts, bounds)?,
        approvals,
        dismiss_stale,
    })
}

fn encode_summaries(values: &[d::Summary]) -> Box<[u8]> {
    let mut size = 0_usize;
    for value in values {
        size = size
            .checked_add(
                usize::try_from(api::encode_summary(value).measure(&w::CEILINGS).expect(super::FITS))
                    .expect("item length fits"),
            )
            .expect("bounded piece")
            .checked_add(4)
            .expect("bounded piece");
    }
    let mut writer = Writer::new(size);
    for value in values {
        let item = api::encode_summary(value);
        let encoded = item.encode(&w::CEILINGS).expect(super::FITS);
        skein_codec::piece::write(&mut writer, &encoded).expect("measured piece room");
    }
    writer.finish()
}
fn decode_summaries(bytes: &[u8], bounds: &Bounds) -> Result<Box<[d::Summary]>, Undecodable> {
    let mut reader = Reader::new(bytes);
    let mut values = List::with_capacity(bounds.forge().client.rows);
    for _ in 0..=bounds.forge().client.rows {
        let Some(item) = skein_codec::piece::split(&mut reader).or(Err(Undecodable::Malformed))? else {
            return Ok(values.into_boxed());
        };
        let item = w::Summary::decode(bounds.limits(), &mut Reader::new(&item)).or(Err(Undecodable::Malformed))?;
        let value = api::decode_summary(item, bounds)?;
        values.push(value).or(Err(Undecodable::Malformed))?;
    }
    Err(Undecodable::Malformed)
}
fn encode_comments(values: &[d::Comment]) -> Box<[u8]> {
    let mut size = 0_usize;
    for value in values {
        size = size
            .checked_add(
                usize::try_from(api::encode_comment(value).measure(&w::CEILINGS).expect(super::FITS))
                    .expect("item length fits"),
            )
            .expect("bounded piece")
            .checked_add(4)
            .expect("bounded piece");
    }
    let mut writer = Writer::new(size);
    for value in values {
        let item = api::encode_comment(value);
        let encoded = item.encode(&w::CEILINGS).expect(super::FITS);
        skein_codec::piece::write(&mut writer, &encoded).expect("measured piece room");
    }
    writer.finish()
}
fn decode_comments(bytes: &[u8], bounds: &Bounds) -> Result<Box<[d::Comment]>, Undecodable> {
    let mut reader = Reader::new(bytes);
    let mut values = List::with_capacity(bounds.forge().client.rows);
    for _ in 0..=bounds.forge().client.rows {
        let Some(item) = skein_codec::piece::split(&mut reader).or(Err(Undecodable::Malformed))? else {
            return Ok(values.into_boxed());
        };
        let item = w::Comment::decode(bounds.limits(), &mut Reader::new(&item)).or(Err(Undecodable::Malformed))?;
        let value = api::decode_comment(item);
        values.push(value).or(Err(Undecodable::Malformed))?;
    }
    Err(Undecodable::Malformed)
}
fn encode_reviews(values: &[d::Review]) -> Box<[u8]> {
    let mut size = 0_usize;
    for value in values {
        size = size
            .checked_add(
                usize::try_from(api::encode_review(value).measure(&w::CEILINGS).expect(super::FITS))
                    .expect("item length fits"),
            )
            .expect("bounded piece")
            .checked_add(4)
            .expect("bounded piece");
    }
    let mut writer = Writer::new(size);
    for value in values {
        let item = api::encode_review(value);
        let encoded = item.encode(&w::CEILINGS).expect(super::FITS);
        skein_codec::piece::write(&mut writer, &encoded).expect("measured piece room");
    }
    writer.finish()
}
fn decode_reviews(bytes: &[u8], bounds: &Bounds) -> Result<Box<[d::Review]>, Undecodable> {
    let mut reader = Reader::new(bytes);
    let mut values = List::with_capacity(bounds.forge().client.rows);
    for _ in 0..=bounds.forge().client.rows {
        let Some(item) = skein_codec::piece::split(&mut reader).or(Err(Undecodable::Malformed))? else {
            return Ok(values.into_boxed());
        };
        let item = w::Review::decode(bounds.limits(), &mut Reader::new(&item)).or(Err(Undecodable::Malformed))?;
        let value = api::decode_review(item);
        values.push(value).or(Err(Undecodable::Malformed))?;
    }
    Err(Undecodable::Malformed)
}

fn encode_statuses(values: &[d::Status]) -> Box<[u8]> {
    let mut size = 0_usize;
    for value in values {
        size = size
            .checked_add(
                usize::try_from(api::encode_status(value).measure(&w::CEILINGS).expect(super::FITS))
                    .expect("item length fits"),
            )
            .expect("bounded piece")
            .checked_add(4)
            .expect("bounded piece");
    }
    let mut writer = Writer::new(size);
    for value in values {
        let item = api::encode_status(value);
        let encoded = item.encode(&w::CEILINGS).expect(super::FITS);
        skein_codec::piece::write(&mut writer, &encoded).expect("measured piece room");
    }
    writer.finish()
}
fn decode_statuses(bytes: &[u8], bounds: &Bounds) -> Result<Box<[d::Status]>, Undecodable> {
    let mut reader = Reader::new(bytes);
    let mut values = List::with_capacity(bounds.forge().client.rows);
    for _ in 0..=bounds.forge().client.rows {
        let Some(item) = skein_codec::piece::split(&mut reader).or(Err(Undecodable::Malformed))? else {
            return Ok(values.into_boxed());
        };
        let item = w::Status::decode(bounds.limits(), &mut Reader::new(&item)).or(Err(Undecodable::Malformed))?;
        let value = api::decode_status(item);
        values.push(value).or(Err(Undecodable::Malformed))?;
    }
    Err(Undecodable::Malformed)
}
fn encode_remarks(values: &[d::Remark]) -> Box<[u8]> {
    let mut size = 0_usize;
    for value in values {
        size = size
            .checked_add(
                usize::try_from(api::encode_remark(value).measure(&w::CEILINGS).expect(super::FITS))
                    .expect("item length fits"),
            )
            .expect("bounded piece")
            .checked_add(4)
            .expect("bounded piece");
    }
    let mut writer = Writer::new(size);
    for value in values {
        let item = api::encode_remark(value);
        let encoded = item.encode(&w::CEILINGS).expect(super::FITS);
        skein_codec::piece::write(&mut writer, &encoded).expect("measured piece room");
    }
    writer.finish()
}
fn decode_remarks(bytes: &[u8], bounds: &Bounds) -> Result<Box<[d::Remark]>, Undecodable> {
    let mut reader = Reader::new(bytes);
    let mut values = List::with_capacity(bounds.forge().client.rows);
    for _ in 0..=bounds.forge().client.rows {
        let Some(item) = skein_codec::piece::split(&mut reader).or(Err(Undecodable::Malformed))? else {
            return Ok(values.into_boxed());
        };
        let item = w::Remark::decode(bounds.limits(), &mut Reader::new(&item)).or(Err(Undecodable::Malformed))?;
        let value = api::decode_remark(item);
        values.push(value).or(Err(Undecodable::Malformed))?;
    }
    Err(Undecodable::Malformed)
}
fn encode_files(values: &[d::File]) -> Box<[u8]> {
    let mut size = 0_usize;
    for value in values {
        size = size
            .checked_add(
                usize::try_from(api::encode_file(value).measure(&w::CEILINGS).expect(super::FITS))
                    .expect("item length fits"),
            )
            .expect("bounded piece")
            .checked_add(4)
            .expect("bounded piece");
    }
    let mut writer = Writer::new(size);
    for value in values {
        let item = api::encode_file(value);
        let encoded = item.encode(&w::CEILINGS).expect(super::FITS);
        skein_codec::piece::write(&mut writer, &encoded).expect("measured piece room");
    }
    writer.finish()
}
fn decode_files(bytes: &[u8], bounds: &Bounds) -> Result<Box<[d::File]>, Undecodable> {
    let mut reader = Reader::new(bytes);
    let mut values = List::with_capacity(bounds.forge().client.rows);
    for _ in 0..=bounds.forge().client.rows {
        let Some(item) = skein_codec::piece::split(&mut reader).or(Err(Undecodable::Malformed))? else {
            return Ok(values.into_boxed());
        };
        let item = w::File::decode(bounds.limits(), &mut Reader::new(&item)).or(Err(Undecodable::Malformed))?;
        let value = api::decode_file(item);
        values.push(value).or(Err(Undecodable::Malformed))?;
    }
    Err(Undecodable::Malformed)
}
fn encode_paths(values: &[Box<[u8]>]) -> List<Box<[u8]>> {
    let mut paths = List::with_capacity(u32::try_from(values.len()).expect("bounded paths"));
    for path in values {
        paths.push(path.clone()).expect("sized paths");
    }
    paths
}

fn encode_collaborators(values: &[d::Collaborator]) -> List<w::Collaborator> {
    let mut result = List::with_capacity(u32::try_from(values.len()).expect("admitted count"));
    for value in values {
        let item = api::encode_collaborator(value);
        result.push(item).expect("same count");
    }
    result
}
fn decode_collaborators(values: List<w::Collaborator>) -> Box<[d::Collaborator]> {
    let mut result = List::with_capacity(values.len());
    for value in values.into_boxed() {
        let item = api::decode_collaborator(value);
        result.push(item).expect("same count");
    }
    result.into_boxed()
}

fn encode_items_answer(items: &[d::Summary], more: bool, now: skein_lib::Time, cut: bool) -> w::ItemsAnswer {
    w::ItemsAnswer { items: encode_summaries(items), more, now: now.as_nanos(), cut }
}
fn encode_item_answer(item: &d::Summary, comments: &[d::Comment], more: bool) -> w::ItemAnswer {
    w::ItemAnswer { item: api::encode_summary(item), comments: encode_comments(comments), more }
}
fn encode_reviews_answer(reviews: &[d::Review], more: bool, cut: bool) -> w::ReviewsAnswer {
    w::ReviewsAnswer { reviews: encode_reviews(reviews), more, cut }
}
fn encode_statuses_answer(ci: d::Ci, statuses: &[d::Status], more: bool) -> w::StatusesAnswer {
    w::StatusesAnswer { ci: topics::encode_ci(ci), statuses: encode_statuses(statuses), more }
}
fn encode_remarks_answer(remarks: &[d::Remark], more: bool, cut: bool) -> w::RemarksAnswer {
    w::RemarksAnswer { remarks: encode_remarks(remarks), more, cut }
}
pub(super) fn encode_pull_files_answer(
    head: d::Commit,
    files: &[d::File],
    more: bool,
    cut: bool,
) -> w::PullFilesAnswer {
    w::PullFilesAnswer { head, files: encode_files(files), more, cut }
}
fn encode_compare_answer(
    before: d::Commit,
    after: d::Commit,
    ahead: u64,
    behind: u64,
    files: &[Box<[u8]>],
    changed: u64,
) -> w::CompareAnswer {
    w::CompareAnswer { before, after, ahead, behind, changed, files: encode_paths(files) }
}
fn encode_checks_answer(statuses: &[d::Status]) -> w::ChecksAnswer {
    w::ChecksAnswer { statuses: encode_statuses(statuses) }
}
fn encode_collaborators_answer(collaborators: &[d::Collaborator], more: bool) -> w::CollaboratorsAnswer {
    w::CollaboratorsAnswer { collaborators: encode_collaborators(collaborators), more }
}

fn directory_entry(value: &d::Entry) -> w::DirectoryEntry {
    crate::generated::store::directory_entry_to_record(value, &super::Context::UNBOUNDED).expect(super::FITS)
}
fn encode_directory(values: &[d::Entry]) -> Box<[u8]> {
    let mut size = 0_usize;
    for value in values {
        size = size
            .checked_add(
                usize::try_from(directory_entry(value).measure(&w::CEILINGS).expect(super::FITS))
                    .expect("entry length fits"),
            )
            .expect("bounded piece")
            .checked_add(4)
            .expect("bounded piece");
    }
    let mut writer = Writer::new(size);
    for value in values {
        let item = directory_entry(value);
        let encoded = item.encode(&w::CEILINGS).expect(super::FITS);
        skein_codec::piece::write(&mut writer, &encoded).expect("measured piece room");
    }
    writer.finish()
}
fn decode_directory(bytes: &[u8], bounds: &Bounds, mut room: u32) -> Result<Box<[d::Entry]>, Undecodable> {
    let width = u32::try_from(skein_forge::measure::ENTRY_BYTES).or(Err(Undecodable::Malformed))?;
    let capacity = room.checked_div(width).ok_or(Undecodable::Malformed)?;
    let mut values = List::with_capacity(capacity);
    let mut reader = Reader::new(bytes);
    for _ in 0..=capacity {
        let Some(item) = skein_codec::piece::split(&mut reader).or(Err(Undecodable::Malformed))? else {
            return Ok(values.into_boxed());
        };
        let item =
            w::DirectoryEntry::decode(bounds.limits(), &mut Reader::new(&item)).or(Err(Undecodable::Malformed))?;
        let w::DirectoryEntry { name, kind } = item;
        let charge = u32::try_from(name.len())
            .or(Err(Undecodable::Malformed))?
            .checked_add(width)
            .ok_or(Undecodable::Malformed)?;
        room = room.checked_sub(charge).ok_or(Undecodable::Malformed)?;
        let kind =
            crate::generated::store::directory_kind_to_domain(kind, &super::Context::UNBOUNDED).expect(super::FITS);
        values.push(d::Entry { name, kind }).or(Err(Undecodable::Malformed))?;
    }
    Err(Undecodable::Malformed)
}

/// Owner-shaped `Protection` (skein/docs/design/data/codec/maps.md, 3.3).
pub fn protection_to_record(
    value: &skein_forge::api::Protection,
    _context: &super::Context,
) -> Result<w::Protection, w::Problem> {
    Ok(encode_protection(value))
}
pub fn protection_to_domain(
    value: w::Protection,
    context: &super::Context,
) -> Result<skein_forge::api::Protection, w::Problem> {
    decode_protection(value, context.bounds())
        .or(Err(w::paths::PROTECTION_BRANCH.problem(skein_codec::Reason::Malformed)))
}

/// Owner-shaped `Pull` (skein/docs/design/data/codec/maps.md, 3.3).
pub fn pull_to_record(value: &skein_forge::api::Pull, _context: &super::Context) -> Result<w::Pull, w::Problem> {
    Ok(encode_pull(value))
}
pub fn pull_to_domain(value: w::Pull, _context: &super::Context) -> Result<skein_forge::api::Pull, w::Problem> {
    Ok(decode_pull(value))
}

/// Owner-shaped `Answer` (skein/docs/design/data/codec/maps.md, 3.3).
pub fn answer_to_record(value: &d::Answer, _context: &super::Context) -> Result<w::Answer, w::Problem> {
    Ok(encode(value))
}
pub fn answer_to_domain(value: w::Answer, context: &super::Context) -> Result<d::Answer, w::Problem> {
    decode(value, context.bounds()).or(Err(w::paths::ANSWER_TAG.problem(skein_codec::Reason::Malformed)))
}
