use super::{FITS, w};
use skein_forge::api as d;
use skein_lib::List;

fn encode_verdict(value: d::Verdict) -> w::Verdict {
    match value {
        d::Verdict::Approve => w::Verdict::Approve,
        d::Verdict::RequestChanges => w::Verdict::RequestChanges,
        d::Verdict::Comment => w::Verdict::Comment,
    }
}
fn decode_verdict(value: w::Verdict) -> d::Verdict {
    match value {
        w::Verdict::Approve => d::Verdict::Approve,
        w::Verdict::RequestChanges => d::Verdict::RequestChanges,
        w::Verdict::Comment => d::Verdict::Comment,
    }
}
fn encode_check(value: d::Check) -> w::Check {
    match value {
        d::Check::Pending => w::Check::Pending,
        d::Check::Passed => w::Check::Passed,
        d::Check::Failed => w::Check::Failed,
    }
}
fn decode_check(value: w::Check) -> d::Check {
    match value {
        w::Check::Pending => d::Check::Pending,
        w::Check::Passed => d::Check::Passed,
        w::Check::Failed => d::Check::Failed,
    }
}
pub(super) fn encode_write(value: &d::Write) -> w::Write {
    match value {
        d::Write::Dispatch { workflow, branch, inputs } => {
            let mut values = List::with_capacity(u32::try_from(inputs.len()).expect(FITS));
            for input in inputs {
                values.push(w::DispatchInput { name: input.name.clone(), value: input.value.clone() }).expect(FITS);
            }
            w::Write::Dispatch(w::DispatchWrite { workflow: workflow.clone(), branch: branch.clone(), inputs: values })
        }
        d::Write::Rerun { attempt } => w::Write::Rerun(super::answers::encode_job_attempt(attempt)),
        d::Write::CreateIssue { key, title, body } => {
            w::Write::CreateIssue(w::CreateIssue { key: key.clone(), title: title.clone(), body: body.clone() })
        }
        d::Write::OpenPull { title, body, head, base } => w::Write::OpenPull(w::OpenPull {
            title: title.clone(),
            body: body.clone(),
            head: head.clone(),
            base: base.clone(),
        }),
        d::Write::Post { number, key, body } => {
            w::Write::Post(w::Post { number: *number, key: key.clone(), body: body.clone() })
        }
        d::Write::Review { number, key, verdict, body } => w::Write::Review(w::PostReview {
            number: *number,
            key: key.clone(),
            verdict: encode_verdict(*verdict),
            body: body.clone(),
        }),
        d::Write::Edit { number, title, body } => {
            w::Write::Edit(w::Edit { number: *number, title: title.clone(), body: body.clone() })
        }
        d::Write::SetReviewers { number, reviewers } => {
            w::Write::SetReviewers(w::SetReviewers { number: *number, reviewers: encode_reviewers(reviewers) })
        }
        d::Write::Close { number } => w::Write::Close(w::Number { number: *number }),
        d::Write::Reopen { number } => w::Write::Reopen(w::Number { number: *number }),
        d::Write::Merge { number, head, how } => {
            let how = match how {
                d::MergeHow::Style => w::MergeHow::Style,
                d::MergeHow::FastForwardOnly => w::MergeHow::FastForwardOnly,
                d::MergeHow::Recorded { commit } => w::MergeHow::Recorded(w::RecordedMerge { commit: *commit }),
            };
            w::Write::Merge(w::MergeWrite { number: *number, head: *head, how })
        }
        d::Write::Update { number, style } => {
            let style = match style {
                d::UpdateStyle::Merge => w::UpdateStyle::Merge,
                d::UpdateStyle::Rebase => w::UpdateStyle::Rebase,
            };
            w::Write::Update(w::UpdateWrite { number: *number, style })
        }
        d::Write::Status { commit, context, check, description, link } => w::Write::Status(w::SetStatus {
            commit: *commit,
            context: context.clone(),
            status: encode_check(*check),
            description: description.clone(),
            link: link.clone(),
        }),
        d::Write::CreateBranch { branch, commit } => {
            w::Write::CreateBranch(w::CreateBranch { branch: branch.clone(), commit: *commit })
        }
        d::Write::DeleteBranch { branch } => w::Write::DeleteBranch(w::DeleteBranch { branch: branch.clone() }),
    }
}
pub(super) fn decode_write(value: w::Write) -> d::Write {
    match value {
        w::Write::Dispatch(value) => {
            let w::DispatchWrite { workflow, branch, inputs } = value;
            let mut values = List::with_capacity(inputs.len());
            for input in inputs.into_boxed() {
                let w::DispatchInput { name, value } = input;
                values.push(d::Input { name, value }).expect(FITS);
            }
            d::Write::Dispatch { workflow, branch, inputs: values.into_boxed() }
        }
        w::Write::Rerun(value) => d::Write::Rerun { attempt: super::answers::decode_job_attempt(value) },
        w::Write::CreateIssue(value) => {
            let w::CreateIssue { key, title, body } = value;
            d::Write::CreateIssue { key, title, body }
        }
        w::Write::OpenPull(value) => {
            let w::OpenPull { title, body, head, base } = value;
            d::Write::OpenPull { title, body, head, base }
        }
        w::Write::Post(value) => {
            let w::Post { number, key, body } = value;
            d::Write::Post { number, key, body }
        }
        w::Write::Review(value) => {
            let w::PostReview { number, key, verdict, body } = value;
            d::Write::Review { number, key, verdict: decode_verdict(verdict), body }
        }
        w::Write::Edit(value) => {
            let w::Edit { number, title, body } = value;
            d::Write::Edit { number, title, body }
        }
        w::Write::SetReviewers(value) => {
            let w::SetReviewers { number, reviewers } = value;
            d::Write::SetReviewers { number, reviewers: reviewers.into_boxed() }
        }
        w::Write::Close(value) => {
            let w::Number { number } = value;
            d::Write::Close { number }
        }
        w::Write::Reopen(value) => {
            let w::Number { number } = value;
            d::Write::Reopen { number }
        }
        w::Write::Merge(value) => {
            let w::MergeWrite { number, head, how } = value;
            d::Write::Merge {
                number,
                head,
                how: match how {
                    w::MergeHow::Style => d::MergeHow::Style,
                    w::MergeHow::FastForwardOnly => d::MergeHow::FastForwardOnly,
                    w::MergeHow::Recorded(value) => d::MergeHow::Recorded { commit: value.commit },
                },
            }
        }
        w::Write::Update(value) => {
            let w::UpdateWrite { number, style } = value;
            d::Write::Update {
                number,
                style: match style {
                    w::UpdateStyle::Merge => d::UpdateStyle::Merge,
                    w::UpdateStyle::Rebase => d::UpdateStyle::Rebase,
                },
            }
        }
        w::Write::Status(value) => {
            let w::SetStatus { commit, context, status, description, link } = value;
            d::Write::Status { commit, context, check: decode_check(status), description, link }
        }
        w::Write::CreateBranch(value) => {
            let w::CreateBranch { branch, commit } = value;
            d::Write::CreateBranch { branch, commit }
        }
        w::Write::DeleteBranch(value) => {
            let w::DeleteBranch { branch } = value;
            d::Write::DeleteBranch { branch }
        }
    }
}
fn encode_reviewers(values: &[u64]) -> List<u64> {
    let mut result = List::with_capacity(u32::try_from(values.len()).expect("admitted count"));
    for value in values {
        result.push(*value).expect("same count");
    }
    result
}

pub(crate) fn encode_error(value: d::Error) -> w::Error {
    match value {
        d::Error::NotInstalled => w::Error::NotInstalled,
        d::Error::NotConnected | d::Error::Unavailable => w::Error::Unavailable,
        d::Error::Timeout => w::Error::Timeout,
        d::Error::RateLimited { after } => w::Error::RateLimited(w::RateLimited { after }),
        d::Error::Forbidden => w::Error::Forbidden,
        d::Error::Missing => w::Error::Missing,
        d::Error::MissingJob => w::Error::MissingJob,
        d::Error::TooLarge(value) => w::Error::TooLarge(w::Excess {
            bound: match value.bound {
                d::Bound::OpBytes => w::Bound::OpBytes,
                d::Bound::AnswerBytes => w::Bound::AnswerBytes,
                d::Bound::Rows => w::Bound::Rows,
                d::Bound::Forge => w::Bound::Forge,
                d::Bound::Unnamed => w::Bound::Unnamed,
            },
            limit: value.limit,
            size: value.size,
            exact: value.exact,
        }),
        d::Error::Empty => w::Error::Empty,
        d::Error::Full => w::Error::Full,
        d::Error::Exists => w::Error::Exists,
        d::Error::NothingToMerge => w::Error::NothingToMerge,
        d::Error::Closed => w::Error::Closed,
        d::Error::Stale => w::Error::Stale,
        d::Error::BaseMoved { base } => w::Error::BaseMoved(w::BaseMoved { base }),
        d::Error::Conflict => w::Error::Conflict,
        d::Error::Protected => w::Error::Protected,
        d::Error::Refused => w::Error::Refused,
        d::Error::InvalidAnswer => w::Error::InvalidAnswer,
        d::Error::Busy => w::Error::Busy,
    }
}
pub(super) fn decode_error(value: w::Error) -> d::Error {
    match value {
        w::Error::NotInstalled => d::Error::NotInstalled,
        w::Error::Unavailable => d::Error::Unavailable,
        w::Error::Timeout => d::Error::Timeout,
        w::Error::RateLimited(value) => d::Error::RateLimited { after: value.after },
        w::Error::Forbidden => d::Error::Forbidden,
        w::Error::Missing => d::Error::Missing,
        w::Error::MissingJob => d::Error::MissingJob,
        w::Error::TooLarge(value) => d::Error::TooLarge(d::Excess {
            bound: match value.bound {
                w::Bound::OpBytes => d::Bound::OpBytes,
                w::Bound::AnswerBytes => d::Bound::AnswerBytes,
                w::Bound::Rows => d::Bound::Rows,
                w::Bound::Forge => d::Bound::Forge,
                w::Bound::Unnamed => d::Bound::Unnamed,
            },
            limit: value.limit,
            size: value.size,
            exact: value.exact,
        }),
        w::Error::Empty => d::Error::Empty,
        w::Error::Full => d::Error::Full,
        w::Error::Exists => d::Error::Exists,
        w::Error::NothingToMerge => d::Error::NothingToMerge,
        w::Error::Closed => d::Error::Closed,
        w::Error::Stale => d::Error::Stale,
        w::Error::BaseMoved(value) => d::Error::BaseMoved { base: value.base },
        w::Error::Conflict => d::Error::Conflict,
        w::Error::Protected => d::Error::Protected,
        w::Error::Refused => d::Error::Refused,
        w::Error::InvalidAnswer => d::Error::InvalidAnswer,
        w::Error::Busy => d::Error::Busy,
    }
}
fn encode_made(value: skein_forge_client::Made) -> w::Made {
    match value {
        skein_forge_client::Made::Created(number) => w::Made::Created(w::Number { number }),
        skein_forge_client::Made::Commented(number) => w::Made::Commented(w::Number { number }),
        skein_forge_client::Made::Reviewed(number) => w::Made::Reviewed(w::Number { number }),
        skein_forge_client::Made::Merged(commit) => w::Made::Merged(w::Commit { commit }),
        skein_forge_client::Made::Updated(commit) => w::Made::Updated(w::Commit { commit }),
        skein_forge_client::Made::Branch(commit) => w::Made::Branch(w::Commit { commit }),
        skein_forge_client::Made::Set => w::Made::Set,
    }
}
fn decode_made(value: w::Made) -> skein_forge_client::Made {
    match value {
        w::Made::Created(value) => skein_forge_client::Made::Created(value.number),
        w::Made::Commented(value) => skein_forge_client::Made::Commented(value.number),
        w::Made::Reviewed(value) => skein_forge_client::Made::Reviewed(value.number),
        w::Made::Merged(value) => skein_forge_client::Made::Merged(value.commit),
        w::Made::Updated(value) => skein_forge_client::Made::Updated(value.commit),
        w::Made::Branch(value) => skein_forge_client::Made::Branch(value.commit),
        w::Made::Set => skein_forge_client::Made::Set,
    }
}
pub(super) fn encode_outcome(value: skein_forge_client::Outcome) -> w::Outcome {
    match value {
        skein_forge_client::Outcome::Made { made, found } => {
            w::Outcome::Made(w::OutcomeMade { made: encode_made(made), found })
        }
        skein_forge_client::Outcome::Failed(error) => w::Outcome::Failed(w::ErrorValue { error: encode_error(error) }),
        skein_forge_client::Outcome::Raced { made, why } => {
            w::Outcome::Raced(w::OutcomeRaced { made: encode_made(made), why: encode_error(why) })
        }
        skein_forge_client::Outcome::Queued => w::Outcome::Queued,
        skein_forge_client::Outcome::Uncertain => w::Outcome::Uncertain,
        skein_forge_client::Outcome::Held => w::Outcome::Held,
        skein_forge_client::Outcome::Withdrawn => w::Outcome::Withdrawn,
    }
}
pub(super) fn decode_outcome(value: w::Outcome) -> skein_forge_client::Outcome {
    match value {
        w::Outcome::Made(value) => {
            let w::OutcomeMade { made, found } = value;
            skein_forge_client::Outcome::Made { made: decode_made(made), found }
        }
        w::Outcome::Failed(error) => skein_forge_client::Outcome::Failed(decode_error(error.error)),
        w::Outcome::Raced(value) => {
            let w::OutcomeRaced { made, why } = value;
            skein_forge_client::Outcome::Raced { made: decode_made(made), why: decode_error(why) }
        }
        w::Outcome::Queued => skein_forge_client::Outcome::Queued,
        w::Outcome::Uncertain => skein_forge_client::Outcome::Uncertain,
        w::Outcome::Held => skein_forge_client::Outcome::Held,
        w::Outcome::Withdrawn => skein_forge_client::Outcome::Withdrawn,
    }
}

fn encode_kind(value: d::Kind) -> w::Kind {
    match value {
        d::Kind::Issue => w::Kind::Issue,
        d::Kind::Pull => w::Kind::Pull,
    }
}
fn decode_kind(value: w::Kind) -> d::Kind {
    match value {
        w::Kind::Issue => d::Kind::Issue,
        w::Kind::Pull => d::Kind::Pull,
    }
}
fn encode_provenance(value: d::Provenance) -> w::Provenance {
    match value {
        d::Provenance::Original => w::Provenance::Original,
        d::Provenance::Revised => w::Provenance::Revised,
        d::Provenance::Unknown => w::Provenance::Unknown,
    }
}
fn decode_provenance(value: w::Provenance) -> d::Provenance {
    match value {
        w::Provenance::Original => d::Provenance::Original,
        w::Provenance::Revised => d::Provenance::Revised,
        w::Provenance::Unknown => d::Provenance::Unknown,
    }
}
pub(super) fn encode_summary(value: &d::Summary) -> w::Summary {
    let d::Summary { labels, number, kind, state, author, key, title, body, updated } = value;
    w::Summary {
        labels: super::names::encode_piece(labels),
        number: *number,
        kind: encode_kind(*kind),
        state: super::topics::encode_state(*state),
        author: *author,
        key: key.clone(),
        title: title.clone(),
        body: body.clone(),
        updated: updated.as_nanos(),
    }
}
pub(super) fn decode_summary(value: w::Summary, bounds: &super::Bounds) -> Result<d::Summary, super::Undecodable> {
    let w::Summary { labels, number, kind, state, author, key, title, body, updated } = value;
    Ok(d::Summary {
        labels: super::names::decode_contexts(&labels, bounds)?,
        number,
        kind: decode_kind(kind),
        state: super::topics::decode_state(state),
        author,
        key,
        title,
        body,
        updated: skein_lib::Time::from_nanos(updated),
    })
}
pub(super) fn encode_comment(value: &d::Comment) -> w::Comment {
    let d::Comment { id, provenance, author, created, revision, key, body } = value;
    w::Comment {
        id: *id,
        provenance: encode_provenance(*provenance),
        author: *author,
        created: created.as_nanos(),
        revision: *revision,
        key: key.clone(),
        body: body.clone(),
    }
}
pub(super) fn decode_comment(value: w::Comment) -> d::Comment {
    let w::Comment { id, provenance, author, created, revision, key, body } = value;
    d::Comment {
        id,
        provenance: decode_provenance(provenance),
        author,
        created: skein_lib::Time::from_nanos(created),
        revision,
        key,
        body,
    }
}
pub(super) fn encode_review(value: &d::Review) -> w::Review {
    let d::Review { id, provenance, revision, author, verdict, commit, key, body, at, official } = value;
    w::Review {
        id: *id,
        provenance: encode_provenance(*provenance),
        revision: *revision,
        author: *author,
        verdict: encode_verdict(*verdict),
        commit: *commit,
        key: key.clone(),
        body: body.clone(),
        at: at.as_nanos(),
        official: *official,
    }
}
pub(super) fn decode_review(value: w::Review) -> d::Review {
    let w::Review { id, provenance, revision, author, verdict, commit, key, body, at, official } = value;
    d::Review {
        id,
        provenance: decode_provenance(provenance),
        revision,
        author,
        verdict: decode_verdict(verdict),
        commit,
        key,
        body,
        at: skein_lib::Time::from_nanos(at),
        official,
    }
}

pub(super) fn encode_status(value: &d::Status) -> w::Status {
    let d::Status { at, context, check, description, url, job } = value;
    w::Status {
        at: at.as_nanos(),
        context: context.clone(),
        verdict: encode_check(*check),
        description: description.clone(),
        url: url.clone(),
        job: match job {
            Some(value) => Some(super::answers::encode_job_attempt(value)),
            None => None,
        },
    }
}
pub(super) fn decode_status(value: w::Status) -> d::Status {
    let w::Status { at, context, verdict, description, url, job } = value;
    d::Status {
        at: skein_lib::Time::from_nanos(at),
        context,
        check: decode_check(verdict),
        description,
        url,
        job: match job {
            Some(value) => Some(super::answers::decode_job_attempt(value)),
            None => None,
        },
    }
}
pub(super) fn encode_remark(value: &d::Remark) -> w::Remark {
    let d::Remark { id, author, path, line, body } = value;
    w::Remark { id: *id, author: *author, path: path.clone(), line: *line, body: body.clone() }
}
pub(super) fn decode_remark(value: w::Remark) -> d::Remark {
    let w::Remark { id, author, path, line, body } = value;
    d::Remark { id, author, path, line, body }
}
pub(super) fn encode_file(value: &d::File) -> w::File {
    let d::File { path, before, after } = value;
    w::File { path: path.clone(), before: before.clone(), after: after.clone() }
}
pub(super) fn decode_file(value: w::File) -> d::File {
    let w::File { path, before, after } = value;
    d::File { path, before, after }
}
pub(super) fn encode_collaborator(value: &d::Collaborator) -> w::Collaborator {
    let d::Collaborator { user, permission } = value;
    w::Collaborator { user: *user, permission: super::answers::encode_permission(*permission) }
}
pub(super) fn decode_collaborator(value: w::Collaborator) -> d::Collaborator {
    let w::Collaborator { user, permission } = value;
    d::Collaborator { user, permission: super::answers::decode_permission(permission) }
}
