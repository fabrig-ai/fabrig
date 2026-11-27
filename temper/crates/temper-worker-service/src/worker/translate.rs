//! Worker payloads and values (protocol/channel.md, 7).
use crate::generated::worker_wire;
use alloc::boxed::Box;
use skein_codec::Reason;
use skein_lib::List;
use temper_protocol_worker_wire::{
    self as wire,
    v1::{self, Record},
};
use temper_worker_domain::wire as worker;

/// Encode the worker's typed read, with the run identity supplied by jig.
#[must_use]
pub fn ask(ask: worker::Ask) -> Box<[u8]> {
    let value = worker_wire::ask_to_record(&ask).expect("bounded check report counts");
    drop(ask);
    wire::ask(&value)
}

/// Decode a whole answer before handing its typed snapshot to the worker.
#[must_use]
pub fn answer(bytes: &[u8], limits: &v1::Limits) -> Option<worker::Answer> {
    let mut reader = skein_lib::Reader::new(bytes);
    let answer = v1::Answer::decode(limits, &mut reader).ok()?;
    if !reader.is_empty() {
        return None;
    }
    worker_wire::answer_to_domain(answer).ok()
}

pub(crate) fn head_to_domain(head: Box<[u8]>) -> Result<Box<[u8]>, Reason> {
    if !wire::red_head(&head) {
        return Err(Reason::Bound);
    }
    Ok(head)
}

/// The worker's decoded workspace with per-repository saved-work branches (protocol/channel.md, 7).
pub type Assigned = worker::Workspace;

#[must_use]
pub(crate) fn items_to_domain(items: v1::Items) -> worker::Workspace {
    let mut repositories = List::with_capacity(items.repositories.len());
    for repository in items.repositories.into_boxed() {
        repositories.push(worker_wire::repository_to_domain(repository)).expect("one output per repository");
    }
    worker::Workspace {
        repositories: repositories.into_boxed(),
        rerun: items.rerun,
        hook_timeout_max_seconds: items.hook_timeout_max_seconds,
    }
}

#[must_use]
pub(crate) fn job_spec_to_domain(spec: v1::JobSpec) -> worker::JobSpec {
    worker::JobSpec {
        command: spec.command,
        request: spec.request,
        refusal: spec.refusal,
        fallback: if spec.find { Some(spec.fallback) } else { None },
    }
}

/// Copy the workspace from its channel record, or give an empty workspace.
#[must_use]
pub fn workspace(items: Option<&v1::Items>) -> Assigned {
    match items {
        Some(items) => items_to_domain(items.clone()),
        None => worker::Workspace {
            rerun: None,
            repositories: Box::new([]),
            hook_timeout_max_seconds: temper_service_core::limits::HOOK_TIMEOUT_MAX_DEFAULT,
        },
    }
}

/// What a run left and saved, field for field under the payload's ceilings.
#[must_use]
pub fn work(work: &worker::Work) -> (v1::Left, Option<v1::Saved>) {
    let mut landed = List::with_capacity(u32::try_from(work.landed.len()).expect("bounded repositories"));
    for head in &work.landed {
        landed.push(worker_wire::landed_to_record(head)).expect("one head per repository");
    }
    let left = v1::Left { landed };
    let saved = match &work.saved {
        Some(rows) => {
            let mut repositories = List::with_capacity(u32::try_from(rows.len()).expect("bounded repositories"));
            for row in rows {
                repositories.push(worker_wire::saved_repository_to_record(row)).expect("one saved row per repository");
            }
            Some(v1::Saved { repositories })
        }
        None => None,
    };
    (left, saved)
}

pub(crate) fn landing_to_record(landing: &worker::Landing) -> v1::Landing {
    match landing {
        worker::Landing::Conflicted { files } => {
            let mut paths = List::with_capacity(u32::try_from(files.len()).expect("bounded conflict paths"));
            for path in files {
                paths.push(path.clone()).expect("one path per conflict");
            }
            v1::Landing::Conflicted(v1::Files { files: paths })
        }
        worker::Landing::Explained { failure } => v1::Landing::Explained(worker_wire::push_failure_to_record(failure)),
        worker::Landing::Landed { commit } => v1::Landing::Landed(v1::Commit { commit: *commit }),
        worker::Landing::Moved => v1::Landing::Moved,
        worker::Landing::Failed => v1::Landing::Failed,
        worker::Landing::Refused => v1::Landing::Refused,
        worker::Landing::Unchanged => v1::Landing::Unchanged,
    }
}

pub(crate) fn diagnostic_to_record(diagnostic: &worker::PushDiagnostic) -> v1::Diagnostic {
    v1::Diagnostic { output: diagnostic.output().into(), cut: diagnostic.cut() }
}

/// A permanent failed preparation by tag; no resource for a transient one.
#[must_use]
pub fn preparation(preparation: &worker::Preparation) -> Option<v1::Preparation> {
    let record = worker_wire::preparation_to_record(preparation);
    match record {
        v1::Preparation::Transient => None,
        v1::Preparation::Missing(_) | v1::Preparation::Refused(_) | v1::Preparation::Conflicted(_) => Some(record),
    }
}

/// The worker's value, once its login and author fields pass their rules.
#[must_use]
pub fn value(identity: &v1::Identity) -> Option<crate::identities::Value> {
    use crate::identities::{Entry, Value};
    if !wire::valid(identity) {
        return None;
    }
    Some(match &identity.kind {
        v1::Kind::Forgejo(value) => Value::Forgejo(temper_worker_protocol_git::Identity {
            token: value.token.clone(),
            login: identity.login.clone(),
            name: identity.name.clone(),
            email: identity.email.clone(),
        }),
        v1::Kind::Github(value) => {
            let mut entries = List::with_capacity(value.entries.len());
            for entry in &value.entries {
                entries
                    .push(Entry { account: entry.account, token: entry.token.clone() })
                    .expect("one entry per decoded installation");
            }
            Value::GitHub {
                login: identity.login.clone(),
                name: identity.name.clone(),
                email: identity.email.clone(),
                entries,
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn items_with_access(access: v1::Access) -> v1::Items {
        let repository = v1::Repository {
            admit: false,
            tag: 7,
            name: b"work".as_slice().into(),
            remote: b"org/work".as_slice().into(),
            git: None,
            start: v1::Start::Commit(v1::Commit { commit: [1; 32] }),
            access,
            identity: 3,
            account: None,
            save: None,
        };
        let mut repositories = List::with_capacity(1);
        repositories.push(repository).expect("fixture repository");
        v1::Items { repositories, rerun: None, hook_timeout_max_seconds: 3600 }
    }

    #[test]
    fn writable_access_carries_the_tasks_own_delivery_to_the_domain() {
        for delivered in [None, Some([7; 32])] {
            let access = v1::Access::Writable(v1::Writable {
                push: b"change/7".as_slice().into(),
                expected: None,
                landing: None,
                delivered,
                pin: None,
            });
            let bytes = wire::items(&items_with_access(access));
            let decoded = v1::Items::decode(&v1::CEILINGS, &mut skein_lib::Reader::new(&bytes)).expect("wire items");
            let translated = workspace(Some(&decoded));
            assert_eq!(
                translated.repositories.first().expect("repository").access,
                temper_worker_domain::wire::Access::Writable {
                    push: b"change/7".as_slice().into(),
                    expected: None,
                    landing: None,
                    delivered,
                    pin: None,
                },
            );
        }
    }

    #[test]
    fn a_named_pin_reaches_the_workers_workspace_items() {
        for pin in [None, Some([8; 32])] {
            let access = v1::Access::Writable(v1::Writable {
                push: b"change/7".as_slice().into(),
                expected: None,
                landing: None,
                delivered: Some([7; 32]),
                pin,
            });
            let bytes = wire::items(&items_with_access(access));
            let decoded = v1::Items::decode(&v1::CEILINGS, &mut skein_lib::Reader::new(&bytes)).expect("wire items");
            let translated = workspace(Some(&decoded));
            assert_eq!(
                translated.repositories.first().expect("repository").access,
                temper_worker_domain::wire::Access::Writable {
                    push: b"change/7".as_slice().into(),
                    expected: None,
                    landing: None,
                    delivered: Some([7; 32]),
                    pin,
                }
            );
        }
    }

    #[test]
    fn prefix_access_keeps_every_branch_in_wire_order_without_a_writable_push() {
        let mut branches = List::with_capacity(3);
        for branch in [b"prefix-1".as_slice(), b"prefix-2", b"prefix-3"] {
            branches.push(Box::from(branch)).expect("fixture branches");
        }
        let access = v1::Access::Prefixes(v1::Prefixes { branches });
        let bytes = wire::items(&items_with_access(access));
        let decoded = v1::Items::decode(&v1::CEILINGS, &mut skein_lib::Reader::new(&bytes)).expect("wire items");
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
