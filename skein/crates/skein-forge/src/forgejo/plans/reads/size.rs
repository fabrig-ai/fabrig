//! Answer row limits beside the allocation measure (skein/docs/design/forge.md, 3.7).
use crate::api;

fn rows(count: usize, limit: u32) -> Result<(), api::Error> {
    if skein_lib::len_u64(count) > u64::from(limit) {
        return Err(api::Error::TooLarge);
    }
    Ok(())
}

pub(super) fn summary(item: &api::Summary, limit: u32) -> Result<u64, api::Error> {
    rows(item.labels.len(), limit)?;
    crate::measure::summary(item).ok_or(api::Error::TooLarge)
}

pub fn answer(answer: &api::Answer, limit: u32) -> Result<u64, api::Error> {
    within(answer, limit)?;
    crate::measure::answer(answer).ok_or(api::Error::TooLarge)
}

fn within(answer: &api::Answer, limit: u32) -> Result<(), api::Error> {
    match answer {
        api::Answer::Items { items, .. } => {
            rows(items.len(), limit)?;
            for item in items {
                rows(item.labels.len(), limit)?;
            }
        }
        api::Answer::Item { item, comments, .. } => {
            rows(comments.len(), limit)?;
            rows(item.labels.len(), limit)?;
        }
        api::Answer::Pull(pull) => rows(pull.reviewers.len(), limit)?,
        api::Answer::Reviews { reviews, .. } => rows(reviews.len(), limit)?,
        api::Answer::Statuses { statuses, .. } | api::Answer::Checks(statuses) => rows(statuses.len(), limit)?,
        api::Answer::Remarks { remarks, .. } => rows(remarks.len(), limit)?,
        api::Answer::Branches(branches) => rows(branches.len(), limit)?,
        api::Answer::Protection(Some(protection)) => rows(protection.contexts.len(), limit)?,
        api::Answer::Collaborators { collaborators, .. } => rows(collaborators.len(), limit)?,
        api::Answer::Runs { runs, .. } => {
            rows(runs.len(), limit)?;
            for run in runs {
                rows(run.jobs.len(), limit)?;
            }
        }
        api::Answer::Commits { commits, .. } => {
            rows(commits.len(), limit)?;
            for commit in commits {
                rows(commit.parents.len(), limit)?;
            }
        }
        api::Answer::Run(run) => rows(run.jobs.len(), limit)?,
        api::Answer::Tree { entries, .. } => rows(entries.len(), limit)?,
        api::Answer::PullFiles { files, .. } => rows(files.len(), limit)?,
        api::Answer::Compare { files, commits, .. } => {
            rows(files.len(), limit)?;
            rows(commits.len(), limit)?;
        }
        api::Answer::Located { .. }
        | api::Answer::Commit(_)
        | api::Answer::Permission(_)
        | api::Answer::Protection(None)
        | api::Answer::Settings(_)
        | api::Answer::File { .. }
        | api::Answer::Job { .. } => {}
        api::Answer::Directory { .. }
        | api::Answer::Created(_)
        | api::Answer::Commented(_)
        | api::Answer::Reviewed(_)
        | api::Answer::Merged(_)
        | api::Answer::Queued
        | api::Answer::Branch(_)
        | api::Answer::Done => return Err(api::Error::InvalidAnswer),
    }
    Ok(())
}
