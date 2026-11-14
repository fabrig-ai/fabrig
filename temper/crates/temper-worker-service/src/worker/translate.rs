//! Worker payloads and values (protocol/channel.md, 7).
use alloc::boxed::Box;
use skein_lib::List;
use temper_protocol_worker_wire::{self as wire, v1};

/// Encode the worker's typed read, with the run identity supplied by jig.
#[must_use]
pub fn ask(ask: temper_worker_domain::wire::Ask) -> Box<[u8]> {
    use temper_worker_domain::wire as worker;
    let value = match ask {
        worker::Ask::Reds => v1::Ask::Reds,
        worker::Ask::Results(results) => {
            let mut passed = List::with_capacity(v1::CEILINGS.check_results_passed);
            for key in results.passed {
                passed.push(key).expect("bounded passed keys");
            }
            let mut failed = List::with_capacity(v1::CEILINGS.check_results_failed);
            for key in results.failed {
                failed.push(key).expect("bounded failed keys");
            }
            let mut flakes = List::with_capacity(v1::CEILINGS.check_results_flakes);
            for flake in results.flakes {
                flakes.push(v1::Flake::new(
                    &v1::CEILINGS, v1::FlakeParts { key: flake.key, log: flake.log },
                ).expect("bounded flake")).expect("bounded flakes");
            }
            v1::Ask::Results(v1::CheckResults::new(&v1::CEILINGS, v1::CheckResultsParts {
                tag: results.tag,
                check_number: results.check,
                tree: results.tree,
                base: results.base,
                verdict: match results.verdict {
                    worker::Verdict::Passed => v1::Verdict::Passed,
                    worker::Verdict::Refused => v1::Verdict::Refused,
                },
                passed, failed, flakes,
            }).expect("bounded check report"))
        }
    };
    wire::ask(&value)
}

/// Decode a whole answer before handing its typed snapshot to the worker.
#[must_use]
pub fn answer(bytes: &[u8], limits: &v1::Limits) -> Option<temper_worker_domain::wire::Answer> {
    use temper_worker_domain::wire as worker;
    let mut reader = skein_lib::Reader::new(bytes);
    let answer = v1::Answer::decode(limits, &mut reader).ok()?;
    if !reader.is_empty() {
        return None;
    }
    Some(match answer {
        v1::Answer::Kept => worker::Answer::Kept,
        v1::Answer::Reds(value) => {
            let mut reds = List::with_capacity(value.reds().len());
            for red in value.into_parts().reds.into_boxed() {
                if !wire::red_head(red.head()) {
                    return None;
                }
                let parts = red.into_parts();
                reds.push(worker::Red { key: parts.key, head: parts.head }).expect("one row per decoded red");
            }
            worker::Answer::Reds { reds: reds.into_boxed() }
        }
    })
}

/// The worker's decoded workspace with per-repository saved-work branches (protocol/channel.md, 7).
pub type Assigned = temper_worker_domain::wire::Workspace;

/// Copy every workspace field from its channel record, or an empty workspace.
#[must_use]
pub fn workspace(items: Option<&v1::Items>) -> Assigned {
    use temper_worker_domain::wire as worker;
    let Some(items) = items else { return worker::Workspace { repositories: Box::new([]) } };
    let mut repositories = List::with_capacity(items.repositories().len());
    for repository in items.repositories() {
        let start = match repository.start() {
            v1::Start::Merged(value) => {
                worker::Start::Merged { branch: value.branch().into(), heads: value.heads().clone().into_boxed() }
            }
            v1::Start::Merge(value) => worker::Start::Merge { branch: value.branch().into(), base: *value.base() },
            v1::Start::Base(value) => worker::Start::Base { branch: value.branch().into() },
            v1::Start::Branch(value) => worker::Start::Branch { branch: value.branch().into() },
            v1::Start::Commit(value) => worker::Start::Commit { commit: *value.commit() },
            v1::Start::Saved(value) => worker::Start::Saved { branch: value.branch().into() },
        };
        let access = match repository.access() {
            v1::Access::Writable(value) => worker::Access::Writable {
                push: value.push().into(),
                expected: *value.expected(),
                landing: value.landing().clone(),
            },
            v1::Access::ReadOnly => worker::Access::ReadOnly,
            v1::Access::Prefixes(value) => worker::Access::Prefixes { branches: value.branches().clone().into_boxed() },
        };
        repositories
            .push(worker::Repository {
                save: repository.save().clone(),
                tag: repository.tag(),
                name: repository.name().into(),
                remote: repository.remote().into(),
                git: repository.git().clone(),
                start,
                access,
                identity: repository.identity(),
                account: *repository.account(),
            })
            .expect("one result per decoded repository");
    }
    worker::Workspace { repositories: repositories.into_boxed() }
}

/// What a run left and saved, field for field under the payload's ceilings.
#[must_use]
pub fn work(work: &temper_worker_domain::wire::Work) -> (v1::Left, Option<v1::Saved>) {
    let limits = &v1::CEILINGS;
    let mut landed = List::with_capacity(u32::try_from(work.landed.len()).expect("bounded repositories"));
    for head in &work.landed {
        landed
            .push(
                v1::Landed::new(limits, v1::LandedParts { tag: head.tag, commit: head.commit, checked: head.checked })
                    .expect("fixed fields"),
            )
            .expect("one head per repository");
    }
    let left = v1::Left::new(limits, v1::LeftParts { landed }).expect("root repositories fit the payload ceilings");
    let saved = match &work.saved {
        Some(rows) => {
            let mut repositories = List::with_capacity(u32::try_from(rows.len()).expect("bounded repositories"));
            for row in rows {
                let landing = landing(&row.landing);
                repositories
                    .push(
                        v1::SavedRepository::new(
                            limits,
                            v1::SavedRepositoryParts { tag: row.tag, from_saved: row.from_saved, landing },
                        )
                        .expect("fixed fields"),
                    )
                    .expect("one saved row per repository");
            }
            Some(
                v1::Saved::new(limits, v1::SavedParts { repositories })
                    .expect("root repositories fit payload ceilings"),
            )
        }
        None => None,
    };
    (left, saved)
}

fn landing(landing: &temper_worker_domain::wire::Landing) -> v1::Landing {
    use temper_worker_domain::wire as worker;
    let limits = &v1::CEILINGS;
    match landing {
        worker::Landing::Conflicted { files } => {
            let mut paths = List::with_capacity(u32::try_from(files.len()).expect("bounded conflict paths"));
            for path in files {
                paths.push(path.clone()).expect("one path per conflict");
            }
            v1::Landing::Conflicted(
                v1::Files::new(limits, v1::FilesParts { files: paths }).expect("root paths fit ceilings"),
            )
        }
        worker::Landing::Explained { failure } => {
            let reason = push_reason(failure.reason);
            let diagnostic = v1::Diagnostic::new(
                limits,
                v1::DiagnosticParts { output: failure.diagnostic.output().into(), cut: failure.diagnostic.cut() },
            )
            .expect("fixed diagnostic cap fits payload");
            v1::Landing::Explained(
                v1::PushFailure::new(
                    limits,
                    v1::PushFailureParts { repository: failure.repository, reason, diagnostic },
                )
                .expect("fixed fields"),
            )
        }
        worker::Landing::Landed { commit } => {
            v1::Landing::Landed(v1::Commit::new(limits, v1::CommitParts { commit: *commit }).expect("fixed id"))
        }
        worker::Landing::Moved => v1::Landing::Moved,
        worker::Landing::Failed => v1::Landing::Failed,
        worker::Landing::Refused => v1::Landing::Refused,
        worker::Landing::Unchanged => v1::Landing::Unchanged,
    }
}
fn push_reason(reason: temper_worker_domain::wire::PushReason) -> v1::PushReason {
    use temper_worker_domain::wire::PushReason;
    match reason {
        PushReason::MissingRepository => v1::PushReason::MissingRepository,
        PushReason::MissingBranch => v1::PushReason::MissingBranch,
        PushReason::MissingCommit => v1::PushReason::MissingCommit,
        PushReason::Refused => v1::PushReason::Refused,
        PushReason::Unreachable => v1::PushReason::Unreachable,
        PushReason::Broken => v1::PushReason::Broken,
        PushReason::TimedOut => v1::PushReason::TimedOut,
        PushReason::Cancelled => v1::PushReason::Cancelled,
        PushReason::Unavailable => v1::PushReason::Unavailable,
        PushReason::Busy => v1::PushReason::Busy,
        PushReason::TooLarge => v1::PushReason::TooLarge,
        PushReason::Nothing => v1::PushReason::Nothing,
        PushReason::Unknown => v1::PushReason::Unknown,
    }
}

/// A permanent failed preparation by tag; no resource for a transient one.
#[must_use]
pub fn preparation(preparation: &temper_worker_domain::wire::Preparation) -> Option<v1::Preparation> {
    use temper_worker_domain::wire as worker;
    let limits = &v1::CEILINGS;
    match preparation {
        worker::Preparation::Conflicted { tag, head } => Some(v1::Preparation::Conflicted(
            v1::Conflicted::new(limits, v1::ConflictedParts { tag: *tag, head: *head }).expect("fixed fields"),
        )),
        worker::Preparation::Transient => None,
        worker::Preparation::Refused { tag } => {
            Some(v1::Preparation::Refused(v1::Tag::new(limits, v1::TagParts { tag: *tag }).expect("fixed tag")))
        }
        worker::Preparation::Missing { tag, missing } => {
            let missing = match missing {
                worker::Missing::Repository => v1::Missing::Repository,
                worker::Missing::Branch => v1::Missing::Branch,
                worker::Missing::Commit => v1::Missing::Commit,
            };
            Some(v1::Preparation::Missing(
                v1::Missed::new(limits, v1::MissedParts { tag: *tag, missing }).expect("fixed fields"),
            ))
        }
    }
}

/// The worker's value, once its login and author fields pass their rules.
#[must_use]
pub fn value(identity: &v1::Identity) -> Option<crate::identities::Value> {
    use crate::identities::{Entry, Value};
    if !wire::valid(identity) {
        return None;
    }
    Some(match identity.kind() {
        v1::Kind::Forgejo(value) => Value::Forgejo(temper_worker_protocol_git::Identity {
            token: value.token().clone(),
            login: identity.login().into(),
            name: identity.name().into(),
            email: identity.email().into(),
        }),
        v1::Kind::Github(value) => {
            let mut entries = List::with_capacity(value.entries().len());
            for entry in value.entries() {
                entries
                    .push(Entry { account: entry.account(), token: entry.token().clone() })
                    .expect("one entry per decoded installation");
            }
            Value::GitHub {
                login: identity.login().into(),
                name: identity.name().into(),
                email: identity.email().into(),
                entries,
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefix_access_keeps_every_branch_in_wire_order_without_a_writable_push() {
        let limits = &v1::CEILINGS;
        let mut branches = List::with_capacity(3);
        for branch in [b"prefix-1".as_slice(), b"prefix-2", b"prefix-3"] {
            branches.push(Box::from(branch)).expect("fixture branches");
        }
        let access = v1::Access::Prefixes(v1::Prefixes::new(limits, v1::PrefixesParts { branches }).expect("prefixes"));
        let repository = v1::Repository::new(
            limits,
            v1::RepositoryParts {
                tag: 7,
                name: b"work".as_slice().into(),
                remote: b"org/work".as_slice().into(),
                git: None,
                start: v1::Start::Commit(v1::Commit::new(limits, v1::CommitParts { commit: [1; 32] }).expect("commit")),
                access,
                identity: 3,
                account: None,
                save: None,
            },
        )
        .expect("repository");
        let mut repositories = List::with_capacity(1);
        repositories.push(repository).expect("fixture repository");
        let items = v1::Items::new(limits, v1::ItemsParts { repositories }).expect("items");
        let bytes = wire::items(&items);
        let decoded = v1::Items::decode(limits, &mut skein_lib::Reader::new(&bytes)).expect("wire items");
        let translated = workspace(Some(&decoded));
        assert_eq!(
            translated.repositories[0].access,
            temper_worker_domain::wire::Access::Prefixes {
                branches: Box::new([
                    Box::from(b"prefix-1".as_slice()),
                    Box::from(b"prefix-2".as_slice()),
                    Box::from(b"prefix-3".as_slice())
                ]),
            }
        );
    }
}
