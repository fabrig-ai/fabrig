//! Pull and file documents carry the shared domain's exact commits.
use super::{commit, issues::listing, optional_commit};
use crate::{
    answers::{self, Body},
    service::{Limits, Response},
};
use alloc::boxed::Box;
use skein_fake_forge_domain::api as a;
use skein_forge::github::document as d;
use skein_lib::bytes::concat;
use skein_lib::{List, usize_of};

pub(super) fn commits(answer: a::Answer, target: &[u8], page: u32, size: u32, limits: &Limits) -> Response {
    let a::Answer::Commits { commits } = answer else {
        return answers::failed();
    };
    let total = skein_lib::len_u64(commits.len());
    let Some(page) = answers::Page::new(page, size, total) else {
        return answers::failed();
    };
    let mut rows = List::with_capacity(size);
    for (index, row) in commits.into_iter().enumerate() {
        if page.contains(u64::try_from(index).expect("bounded history")) {
            let mut parents = List::with_capacity(u32::try_from(row.parents.len()).expect("bounded parents"));
            for parent in row.parents {
                parents.push(commit(parent)).expect("parent capacity");
            }
            rows.push(skein_forge::api::CommitMessage {
                id: commit(row.id),
                parents: parents.into_boxed(),
                message: row.message,
            })
            .expect("page capacity");
        }
    }
    page.link_next(answers::response(200, &Body::Commits(total, rows.as_slice()), limits), target, limits)
}

pub(super) fn document(row: a::Pull) -> Option<d::Pull> {
    Some(d::Pull {
        number: row.number,
        closed: row.state == a::State::Closed,
        merged: row.merged.is_some(),
        merge_commit: if row.queue_merged { None } else { optional_commit(row.merged) },
        mergeable: if row.checking { None } else { Some(row.mergeable) },
        head: row.head,
        base: row.base,
        head_commit: commit(row.commit),
        base_commit: commit(row.base_commit?),
        reviewers: row.reviewers,
        changed_files: Some(row.size.files),
        additions: Some(row.size.additions),
        deletions: Some(row.size.deletions),
    })
}
pub(super) fn pull(answer: a::Answer, limits: &Limits) -> Response {
    let a::Answer::Pull(row) = answer else {
        return answers::failed();
    };
    let Some(row) = document(row) else {
        return answers::missing(limits);
    };
    answers::response(200, &Body::Pull(&row), limits)
}
pub(super) fn pulls_for(answer: a::Answer, open: bool, limits: &Limits) -> Response {
    let a::Answer::Pull(row) = answer else {
        return answers::failed();
    };
    if open && row.state != a::State::Open {
        return answers::response(200, &Body::Pulls(&[]), limits);
    }
    let Some(row) = document(row) else {
        return answers::missing(limits);
    };
    answers::response(200, &Body::Pulls(&[row]), limits)
}
fn file(row: a::ChangedFile) -> d::File {
    d::File {
        path: row.path,
        previous: None,
        status: if row.before.is_none() {
            d::FileStatus::Added
        } else if row.after.is_none() {
            d::FileStatus::Removed
        } else {
            d::FileStatus::Modified
        },
    }
}
pub(super) fn files(answer: a::Answer, target: &[u8], page: u32, size: u32, limits: &Limits) -> Response {
    let a::Answer::PullFiles { files, more, .. } = answer else {
        return answers::failed();
    };
    let mut rows = List::with_capacity(size);
    for row in files {
        if rows.push(file(row)).is_err() {
            return answers::failed();
        }
    }
    let Some(page) = listing(page, size, rows.len(), more) else {
        return answers::failed();
    };
    page.link_next(answers::response(200, &Body::Files(rows.as_slice()), limits), target, limits)
}
pub(super) fn compare(answer: a::Answer, size: u32, limits: &Limits) -> Response {
    let a::Answer::Comparison { merge_base, ahead, behind, files, commits, .. } = answer else {
        return answers::failed();
    };
    let total_commits = skein_lib::len_u64(commits.len());
    let mut ids = List::with_capacity(size);
    for id in commits.iter().take(usize_of(size)) {
        ids.push(commit(*id)).expect("page capacity");
    }
    // GitHub caps the comparison's file array at 300; the dialect detects
    // that cap and refuses an incomplete comparison.
    let mut rows = List::with_capacity(300);
    for row in files.into_iter().take(300) {
        rows.push(file(row)).expect("comparison file cap");
    }
    let document =
        d::Compare { total_commits, ahead_by: ahead, behind_by: behind, merge_base: optional_commit(merge_base) };
    answers::response(200, &Body::Compare(&document, ids.as_slice(), rows.as_slice()), limits)
}

// Node IDs are opaque GitHub-layer identities; repository and number make them
// stable and distinct without another domain store.
#[expect(clippy::manual_map, reason = "step code spells Option transformations with exhaustive matches")]
pub(super) fn graphql(answer: a::Answer, repository: u32, number: u64, limits: &Limits) -> Response {
    let a::Answer::Pull(row) = answer else {
        return answers::failed();
    };
    let Some(id) = node_id(repository, number, limits) else {
        return answers::failed();
    };
    let queued = match row.queued {
        Some(entry) => Some(entry.position),
        None => None,
    };
    let dequeued = match row.dequeued {
        Some(a::QueueRemoval::MergeConflict) => Some(skein_lib::bytes::copy_of(crate::texts::MERGE_CONFLICT)),
        Some(a::QueueRemoval::CiFailure) => Some(skein_lib::bytes::copy_of(crate::texts::CI_FAILURE)),
        None => None,
    };
    let row = d::PullGraphql { id, merge_commit: optional_commit(row.merged), queued, dequeued };
    answers::response(200, &Body::PullGraphql(&row), limits)
}

pub(super) fn node_id(repository: u32, number: u64, limits: &Limits) -> Option<Box<[u8]>> {
    let id = concat(&[
        b"PR_",
        skein_lib::Decimal::of(u64::from(repository)).as_bytes(),
        b"_",
        skein_lib::Decimal::of(number).as_bytes(),
    ]);
    if skein_lib::len_u64(id.len()) > u64::from(limits.github.string_bytes) {
        return None;
    }
    Some(id)
}
