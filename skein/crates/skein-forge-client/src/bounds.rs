//! Admission validation and payload caps (forge-client.md, section 2.3).
use crate::Limits;
use skein_forge::api::{Answer, Bound, Excess, Op, Read, Write};
use skein_lib::{len_u64, usize_of};
pub(crate) fn op(value: &Op, l: &Limits) -> Result<(), Excess> {
    match value {
        Op::Read(Read::Job { max_bytes, .. } | Read::File { max_bytes, .. } | Read::Artifact { max_bytes, .. }) => {
            if *max_bytes == 0 || *max_bytes > l.answer_bytes {
                return Err(Excess::exact(Bound::AnswerBytes, u64::from(l.answer_bytes), u64::from(*max_bytes)));
            }
        }
        Op::Read(_) | Op::Write(_) => {}
    }
    let valid = match value {
        Op::Read(value) => read(value).is_some(),
        Op::Write(value) => write(value, l).is_some(),
    };
    if !valid {
        return Err(Excess::unnamed());
    }
    let held = skein_forge::measure::op(value).ok_or(Excess::unnamed())?;
    if held > u64::from(l.op_bytes) {
        Err(Excess::exact(Bound::OpBytes, u64::from(l.op_bytes), held))
    } else {
        Ok(())
    }
}
pub(crate) fn read(value: &Read) -> Option<u64> {
    let valid = match value {
        Read::Artifact { name, .. } => !name.is_empty(),
        Read::Items { page, .. }
        | Read::Reviews { page, .. }
        | Read::Statuses { page, .. }
        | Read::Remarks { page, .. }
        | Read::PullFiles { page, .. }
        | Read::Collaborators { page }
        | Read::Runs { page, .. }
        | Read::Commits { page, .. }
        | Read::Tree { page, .. } => *page != 0,
        Read::Locate { owner, name } => return skein_forge::locate::bytes(owner, name),
        Read::File { .. }
        | Read::PullFor { .. }
        | Read::Branch { .. }
        | Read::Protection { .. }
        | Read::Run { .. }
        | Read::Job { .. }
        | Read::Item { .. }
        | Read::Pull { .. }
        | Read::Compare { .. }
        | Read::Checks { .. }
        | Read::Branches { .. }
        | Read::Settings
        | Read::Permission { .. } => true,
    };
    if valid { Some(0) } else { None }
}
pub(crate) fn write(value: &Write, l: &Limits) -> Option<u64> {
    match value {
        Write::Dispatch { workflow, branch, inputs } => {
            if workflow.is_empty() || branch.is_empty() || inputs.len() > skein_forge::api::INPUTS {
                return None;
            }
            for input in inputs {
                if input.name.is_empty() {
                    return None;
                }
            }
        }
        Write::Status { description, .. } => {
            if description.len() > skein_forge::api::DESCRIPTION_BYTES {
                return None;
            }
        }
        Write::SetReviewers { reviewers, .. } => {
            if len_u64(reviewers.len()) > u64::from(l.rows) {
                return None;
            }
        }
        Write::Rerun { .. }
        | Write::CreateIssue { .. }
        | Write::OpenPull { .. }
        | Write::Post { .. }
        | Write::Review { .. }
        | Write::Edit { .. }
        | Write::CreateBranch { .. }
        | Write::DeleteBranch { .. }
        | Write::Close { .. }
        | Write::Reopen { .. }
        | Write::Update { .. }
        | Write::Merge { .. } => {}
    }
    skein_forge::measure::write(value)
}
pub(crate) fn answer(value: &Answer, l: &Limits) -> Result<(), Excess> {
    let rows = len_u64(skein_forge::measure::answer_list_rows(value));
    if rows > u64::from(l.rows) {
        return Err(Excess::exact(Bound::Rows, u64::from(l.rows), rows));
    }
    let bytes = skein_forge::measure::answer(value).ok_or(Excess::unnamed())?;
    if bytes > u64::from(l.answer_bytes) {
        Err(Excess::exact(Bound::AnswerBytes, u64::from(l.answer_bytes), bytes))
    } else {
        Ok(())
    }
}
pub(crate) fn cached(value: &crate::Cached, l: &Limits) -> bool {
    let item = match &value.item {
        Some(item) => {
            item.labels.len() <= usize_of(l.rows) && within(skein_forge::measure::summary(item), l.answer_bytes)
        }
        None => true,
    };
    let pull = match &value.pull {
        Some(pull) => {
            pull.reviewers.len() <= usize_of(l.rows) && within(skein_forge::measure::pull(pull), l.answer_bytes)
        }
        None => true,
    };
    item && pull
}

fn within(bytes: Option<u64>, cap: u32) -> bool {
    match bytes {
        Some(bytes) => bytes <= u64::from(cap),
        None => false,
    }
}
