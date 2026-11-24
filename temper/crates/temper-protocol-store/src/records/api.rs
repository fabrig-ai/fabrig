use super::{FITS, w};
use skein_forge::api as d;
use skein_lib::List;

fn encode_verdict(value: d::Verdict) -> w::Verdict {
    crate::generated::store::verdict_to_record(&value)
}
fn decode_verdict(value: w::Verdict) -> d::Verdict {
    crate::generated::store::verdict_to_domain(value)
}
fn encode_check(value: d::Check) -> w::Check {
    crate::generated::store::check_to_record(&value)
}
fn decode_check(value: w::Check) -> d::Check {
    crate::generated::store::check_to_domain(value)
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

pub(super) fn encode_summary(value: &d::Summary) -> w::Summary {
    let d::Summary { labels, number, kind, state, author, key, title, body, updated } = value;
    w::Summary {
        labels: super::names::encode_piece(labels),
        number: *number,
        kind: crate::generated::store::kind_to_record(kind),
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
        kind: crate::generated::store::kind_to_domain(kind),
        state: super::topics::decode_state(state),
        author,
        key,
        title,
        body,
        updated: skein_lib::Time::from_nanos(updated),
    })
}
pub(super) fn encode_comment(value: &d::Comment) -> w::Comment {
    crate::generated::store::comment_to_record(value)
}
pub(super) fn decode_comment(value: w::Comment) -> d::Comment {
    crate::generated::store::comment_to_domain(value)
}
pub(super) fn encode_review(value: &d::Review) -> w::Review {
    crate::generated::store::review_to_record(value)
}
pub(super) fn decode_review(value: w::Review) -> d::Review {
    crate::generated::store::review_to_domain(value)
}

pub(super) fn encode_status(value: &d::Status) -> w::Status {
    crate::generated::store::status_to_record(value)
}
pub(super) fn decode_status(value: w::Status) -> d::Status {
    crate::generated::store::status_to_domain(value)
}
pub(super) fn encode_remark(value: &d::Remark) -> w::Remark {
    crate::generated::store::remark_to_record(value)
}
pub(super) fn decode_remark(value: w::Remark) -> d::Remark {
    crate::generated::store::remark_to_domain(value)
}
pub(super) fn encode_file(value: &d::File) -> w::File {
    crate::generated::store::file_to_record(value)
}
pub(super) fn decode_file(value: w::File) -> d::File {
    crate::generated::store::file_to_domain(value)
}
pub(super) fn encode_collaborator(value: &d::Collaborator) -> w::Collaborator {
    crate::generated::store::collaborator_to_record(value)
}
pub(super) fn decode_collaborator(value: w::Collaborator) -> d::Collaborator {
    crate::generated::store::collaborator_to_domain(value)
}

/// Owner-shaped `Summary` (skein/docs/design/data/codec/maps.md, 3.3).
#[must_use]
pub fn summary_to_record(value: &d::Summary) -> w::Summary {
    encode_summary(value)
}
pub fn summary_to_domain(value: w::Summary, context: &super::Context) -> Result<d::Summary, w::Problem> {
    decode_summary(value, context.bounds()).or(Err(w::paths::SUMMARY_LABELS.problem(skein_codec::Reason::Malformed)))
}

/// Owner-shaped `Error` (skein/docs/design/data/codec/maps.md, 3.3).
#[must_use]
pub fn error_to_record(value: &d::Error) -> w::Error {
    encode_error(*value)
}
#[must_use]
pub fn error_to_domain(value: w::Error) -> d::Error {
    decode_error(value)
}

/// Owner-shaped `Write` (skein/docs/design/data/codec/maps.md, 3.3).
#[must_use]
pub fn write_to_record(value: &d::Write) -> w::Write {
    encode_write(value)
}
#[must_use]
pub fn write_to_domain(value: w::Write) -> d::Write {
    decode_write(value)
}
