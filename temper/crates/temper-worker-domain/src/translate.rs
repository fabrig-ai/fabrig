//! The translations between the host's vocabulary and its siblings' (4.5):
//! siblings share no types, so the host's workspaces meet the checkout's
//! specs and outcomes here, and its agents smith's parent face, through small
//! total functions, each an exhaustive match, so that a variant added on
//! either side breaks the build in one place. No payload codec: smith's
//! payloads are the protocol layer's (jig's `domain/hosts/hub.md`, 8).

use alloc::boxed::Box;

use crate::wire;
use jig_domain_host as host;
use skein_lib::bytes::copy_of;
use skein_lib::{List, Token};
use smith_host_domain as agent;
use temper_worker_domain_checkout::{self as checkout, git};

/// The title of the commit that saves a run's unfinished work.
pub(crate) const SAVED: &[u8] = b"Save unfinished work";

/// The checkout's spec for a run's workspace: keyed by the workstream, the
/// holding task's number in eight big-endian bytes; the same repositories, in
/// the same order, each reached as its identity.
pub(crate) fn spec(workstream: u64, workspace: wire::Workspace) -> checkout::Spec {
    let wire::Workspace { repositories } = workspace;
    let count = u32::try_from(repositories.len()).expect("the root checked the repositories against its limits");
    let mut specs = List::with_capacity(count);
    for repository in repositories {
        specs.push(self::repository(repository)).expect("room for every repository");
    }
    checkout::Spec { key: Box::new(workstream.to_be_bytes()), repositories: specs.into_boxed() }
}

/// The workstream a checkout's key names: the inverse of [`spec`]'s, for the
/// keys the root made.
pub(crate) fn workstream(key: &[u8]) -> u64 {
    u64::from_be_bytes(key.try_into().expect("the root keys every checkout by eight bytes"))
}

fn repository(repository: wire::Repository) -> checkout::Repository {
    let wire::Repository { tag: _, name, remote, git, start, access, identity, account, save } = repository;
    let (push, expected, landing, prefixes): (_, _, _, Box<[Box<[u8]>]>) = match access {
        wire::Access::ReadOnly => (None, None, None, Box::new([])),
        wire::Access::Prefixes { branches } => (None, None, None, branches),
        wire::Access::Writable { push, expected, landing } => {
            let expected = match expected {
                Some(raw) => Some(git::Commit::new(raw)),
                None => None,
            };
            (Some(push), expected, landing, Box::new([]))
        }
    };
    checkout::Repository {
        save,
        name,
        remote,
        git,
        start: self::start(start),
        identity,
        account,
        push,
        prefixes,
        expected,
        landing,
    }
}

fn start(start: wire::Start) -> checkout::Start {
    match start {
        wire::Start::Base { branch } => checkout::Start::Base { branch },
        wire::Start::Branch { branch } => checkout::Start::Branch { branch },
        wire::Start::Commit { commit } => checkout::Start::Commit { commit: git::Commit::new(commit) },
        wire::Start::Saved { branch } => checkout::Start::Saved { branch },
        wire::Start::Merged { branch, heads } => {
            let mut commits = List::with_capacity(u32::try_from(heads.len()).expect("admitted head count"));
            for head in heads {
                commits.push(git::Commit::new(head)).expect("counted heads");
            }
            checkout::Start::Merged { branch, heads: commits.into_boxed() }
        }
        wire::Start::Merge { branch, base } => checkout::Start::Merge { branch, base: git::Commit::new(base) },
    }
}

/// Why a prepare failed, as the host tells the engine: a retry may work, or
/// what the forge lacks or refused, for which repository.
pub(crate) fn failure(failure: checkout::Failure, tags: &[u64]) -> wire::Preparation {
    match failure {
        checkout::Failure::Conflicted { repository, head } => {
            let place = usize::try_from(repository).expect("a u32 fits usize");
            let tag = *tags.get(place).expect("every failed repository has a tag");
            wire::Preparation::Conflicted { tag, head: head.raw() }
        }
        checkout::Failure::Transient => wire::Preparation::Transient,
        checkout::Failure::Missing { repository, missing } => {
            let place = skein_lib::usize_of(repository);
            let tag = *tags.get(place).expect("every failed repository has a tag");
            wire::Preparation::Missing { tag, missing: self::missing(missing) }
        }
        checkout::Failure::Refused { repository } => {
            let place = skein_lib::usize_of(repository);
            let tag = *tags.get(place).expect("every failed repository has a tag");
            wire::Preparation::Refused { tag }
        }
    }
}

const fn missing(missing: git::Missing) -> wire::Missing {
    match missing {
        git::Missing::Repository => wire::Missing::Repository,
        git::Missing::Branch => wire::Missing::Branch,
        git::Missing::Commit => wire::Missing::Commit,
    }
}

/// Why a prepare was refused at the checkout's entrance, as the host tells the
/// engine: a workstream whose workspace another run holds, or a disk with
/// every workspace held, may have room later. The host admits no workspace
/// the checkout cannot take, as `valid` checks their limits agree.
pub(crate) fn checkout_refusal(refusal: checkout::Refusal) -> wire::Preparation {
    match refusal {
        checkout::Refusal::Busy | checkout::Refusal::Full => wire::Preparation::Transient,
        checkout::Refusal::Invalid => unreachable!("the host admits only workspaces the checkout takes"),
    }
}

pub(crate) const fn preparation(failure: wire::Preparation, resource: Token) -> host::Preparation {
    match failure {
        wire::Preparation::Transient => host::Preparation::Transient,
        wire::Preparation::Missing { .. }
        | wire::Preparation::Refused { .. }
        | wire::Preparation::Conflicted { .. } => host::Preparation::Permanent { resource: Some(resource) },
    }
}

/// The commit message of a save.
pub(crate) fn saved() -> checkout::Message {
    checkout::Message { title: copy_of(SAVED), body: Box::new([]) }
}

/// What became of each of the workspace's `repositories` in a push or a save,
/// as the host hears it. One refused at the checkout's entrance pushed
/// nothing, so failed everywhere; a repository the push did not reach, as it
/// was aborted, failed.
pub(crate) fn landings(outcome: checkout::Outcome, repositories: u32) -> Box<[wire::Landing]> {
    let mut landings = List::with_capacity(repositories);
    match outcome {
        checkout::Outcome::Pushed { landings: pushed } => {
            for landed in pushed {
                landings.push(landing(landed)).expect("one landing for each repository");
            }
        }
        checkout::Outcome::Refused { refusal: _ } => {
            for _ in 0..repositories {
                landings.push(wire::Landing::Failed).expect("one landing for each repository");
            }
        }
    }
    landings.into_boxed()
}

fn landing(landing: checkout::Landing) -> wire::Landing {
    match landing {
        checkout::Landing::Conflicted { files } | checkout::Landing::DesignConflict { files } => {
            wire::Landing::Conflicted { files }
        }
        checkout::Landing::Explained { fault, diagnostic } => wire::Landing::Explained {
            failure: wire::PushFailure {
                repository: None,
                reason: push_reason(fault),
                diagnostic: wire::PushDiagnostic::new(diagnostic.output(), diagnostic.cut()),
            },
        },
        checkout::Landing::Landed { commit }
        | checkout::Landing::Prefixes { commit, .. }
        | checkout::Landing::DesignMerged { commit, .. } => wire::Landing::Landed { commit: commit.raw() },
        checkout::Landing::Moved => wire::Landing::Moved,
        checkout::Landing::Unchanged => wire::Landing::Unchanged,
        checkout::Landing::DesignCode { .. }
        | checkout::Landing::DesignMoving
        | checkout::Landing::DesignUnchecked
        | checkout::Landing::Refused => wire::Landing::Refused,
        checkout::Landing::Failed | checkout::Landing::Aborted => wire::Landing::Failed,
    }
}

pub(crate) const fn delivery_outcome(result: &agent::Delivery) -> host::DeliveryOutcome {
    match result {
        agent::Delivery::Delivered(_) => host::DeliveryOutcome::Delivered,
        agent::Delivery::Nothing => host::DeliveryOutcome::Nothing,
        agent::Delivery::Stale => host::DeliveryOutcome::Stale,
        agent::Delivery::Refused(_) => host::DeliveryOutcome::Refused,
        agent::Delivery::Failed(_) => host::DeliveryOutcome::Failed,
    }
}

pub(crate) const fn fault(fault: agent::Fault) -> host::AgentFailure {
    match fault {
        agent::Fault::Exited => host::AgentFailure::Exited,
        agent::Fault::Rules | agent::Fault::TooLarge => host::AgentFailure::Rules,
        agent::Fault::NoProgress => host::AgentFailure::NoProgress,
        agent::Fault::WallTime => host::AgentFailure::WallTime,
    }
}

/// Why smith refused a message, as the hub keeps it: one to one.
pub(crate) const fn refusal(refusal: agent::MessageRefusal) -> host::MessageRefusal {
    match refusal {
        agent::MessageRefusal::TooLarge => host::MessageRefusal::TooLarge,
        agent::MessageRefusal::Full => host::MessageRefusal::Full,
        agent::MessageRefusal::Ending => host::MessageRefusal::Ending,
        agent::MessageRefusal::NameInUse => host::MessageRefusal::NameInUse,
    }
}

/// A long operation smith reports, as jig's activity names it.
pub(crate) const fn operation(operation: agent::Operation) -> jig_domain_link::Operation {
    match operation {
        agent::Operation::Checks => jig_domain_link::Operation::Checks,
        agent::Operation::Completion => jig_domain_link::Operation::Completion,
        agent::Operation::Command => jig_domain_link::Operation::Command,
    }
}

/// Why a run failed, as it reports it. A configured bound on the model call
/// is `Exceeded`, which jig holds at once; a limited or refused account stays
/// a failure of its own, for the engine to retry the run once the account has
/// room (`domain/worker/README.md`, 4.3; jig's `domain/hosts/hub.md`, 3).
pub(crate) const fn run_failure(failure: &agent::RunFailure) -> host::RunFailure {
    match failure {
        agent::RunFailure::Model(agent::ModelFault::Limited { account, reset }) => {
            host::RunFailure::Limited { account: *account, reset: *reset }
        }
        agent::RunFailure::Model(agent::ModelFault::Refused { account, code: _ }) => {
            host::RunFailure::Refused { account: *account }
        }
        agent::RunFailure::Model(agent::ModelFault::Completion {
            failure: agent::CompletionFailure::Limit { which, bound: value },
            ..
        }) => match exceeded(which) {
            Some(bound) => host::RunFailure::Exceeded(host::Exceeded { bound, value: *value }),
            None => host::RunFailure::Model,
        },
        agent::RunFailure::Transcript(_) => host::RunFailure::Transcript,
        agent::RunFailure::Model(
            agent::ModelFault::Completion { .. }
            | agent::ModelFault::Provider
            | agent::ModelFault::Unauthorized { .. }
            | agent::ModelFault::Program { .. }
            | agent::ModelFault::Declined
            | agent::ModelFault::Truncated
            | agent::ModelFault::Malformed,
        ) => host::RunFailure::Model,
        agent::RunFailure::Budget { .. } => host::RunFailure::Budget,
        agent::RunFailure::Policy(_) => host::RunFailure::Policy,
        agent::RunFailure::Cancelled => host::RunFailure::Cancelled,
        agent::RunFailure::Stale => host::RunFailure::Stale,
    }
}

/// The configured call bounds that jig holds instead of retrying (domain/worker/README.md, 4.3).
const fn exceeded(which: &agent::Limit) -> Option<host::Bound> {
    match which {
        agent::Limit::Path(cap) => match cap {
            agent::Cap::Reasoning => Some(host::Bound::Reasoning),
            agent::Cap::Metadata => Some(host::Bound::Metadata),
            agent::Cap::OutputItems => Some(host::Bound::Items),
            agent::Cap::Answer => Some(host::Bound::Answer),
            agent::Cap::Tools => Some(host::Bound::Tools),
            agent::Cap::HistoryItems => Some(host::Bound::HistoryItems),
            agent::Cap::Output => Some(host::Bound::Output),
            agent::Cap::Request
            | agent::Cap::RequestHead
            | agent::Cap::ResponseHead
            | agent::Cap::ResponseFields
            | agent::Cap::Skip
            | agent::Cap::Field
            | agent::Cap::Retained
            | agent::Cap::Tokens
            | agent::Cap::Depth
            | agent::Cap::Strings
            | agent::Cap::Number
            | agent::Cap::ErrorBody
            | agent::Cap::AccessToken => None,
        },
        agent::Limit::History => Some(host::Bound::History),
        agent::Limit::Window => Some(host::Bound::Window),
        agent::Limit::Pool { .. } | agent::Limit::Call | agent::Limit::Message => None,
    }
}

/// What smith refused of a start before any process ran, in words for
/// operators; nothing for an agent that started.
pub(crate) const fn unstarted(end: agent::End) -> Option<&'static [u8]> {
    match end {
        agent::End::Busy => Some(b"busy"),
        agent::End::Invalid(invalid) => Some(match invalid {
            agent::Invalid::Messages => b"messages",
            agent::Invalid::Activation => b"activation",
            agent::Invalid::Charter => b"charter",
            agent::Invalid::Transcript => b"transcript",
            agent::Invalid::Answered => b"answered calls",
            agent::Invalid::Directories => b"directories",
            agent::Invalid::Grants => b"grants",
            agent::Invalid::Limits => b"limits",
        }),
        agent::End::Unspawned | agent::End::Exited { .. } | agent::End::Settled => None,
    }
}

pub(crate) const fn push_reason(fault: git::Fault) -> wire::PushReason {
    match fault {
        git::Fault::Missing { missing: git::Missing::Repository } => wire::PushReason::MissingRepository,
        git::Fault::Missing { missing: git::Missing::Branch } => wire::PushReason::MissingBranch,
        git::Fault::Missing { missing: git::Missing::Commit } => wire::PushReason::MissingCommit,
        git::Fault::Refused => wire::PushReason::Refused,
        git::Fault::Unreachable => wire::PushReason::Unreachable,
        git::Fault::Broken => wire::PushReason::Broken,
        git::Fault::TimedOut => wire::PushReason::TimedOut,
        git::Fault::Cancelled => wire::PushReason::Cancelled,
    }
}

pub(crate) fn grant(value: jig_domain_link::Grant) -> Option<agent::Grant> {
    let form = match value.form {
        jig_domain_link::Form::Credential(jig_domain_link::Scheme::Bearer) => {
            agent::Form::Credential(agent::Scheme::Bearer)
        }
        jig_domain_link::Form::Credential(jig_domain_link::Scheme::Key) => agent::Form::Credential(agent::Scheme::Key),
        jig_domain_link::Form::Program(name) => {
            agent::Form::Program(agent::SourceName::new(name.as_bytes()).expect("same source name grammar"))
        }
        jig_domain_link::Form::Application => return None,
    };
    Some(agent::Grant { account: value.account, generation: value.generation, valid: value.valid, form })
}

/// Preserve agent start policy separately from its diagnostic bytes.
#[must_use]
pub const fn start_cause(why: agent::RunInvalid) -> host::StartCause {
    match why {
        agent::RunInvalid::Grants => host::StartCause::Grants,
        agent::RunInvalid::HostPart { .. }
        | agent::RunInvalid::Room { .. }
        | agent::RunInvalid::Messages
        | agent::RunInvalid::CharterVersion
        | agent::RunInvalid::MalformedCharter
        | agent::RunInvalid::Activation
        | agent::RunInvalid::Window
        | agent::RunInvalid::Conventions
        | agent::RunInvalid::TooLarge
        | agent::RunInvalid::Workspace
        | agent::RunInvalid::Outcome
        | agent::RunInvalid::Budget(_)
        | agent::RunInvalid::Price(_)
        | agent::RunInvalid::Llm
        | agent::RunInvalid::Effort
        | agent::RunInvalid::Conversation
        | agent::RunInvalid::Endpoint => host::StartCause::Other,
    }
}

/// Keep model evidence in the operator terminal, outside the encoded LLM result.
pub(crate) fn failure_detail(result: &agent::RunResult, detail: &[u8], bound: u32) -> Box<[u8]> {
    let mut text = List::with_capacity(bound);
    match result {
        agent::RunResult::Failed {
            failure: agent::RunFailure::Model(agent::ModelFault::Program { name, exit, detail: program }),
        } => {
            diagnostic_part(&mut text, b"program ");
            diagnostic_part(&mut text, name);
            if let Some(exit) = exit {
                diagnostic_part(&mut text, b" exit ");
                diagnostic_part(&mut text, skein_lib::Decimal::of(u64::from(*exit)).as_bytes());
            }
            diagnostic_part(&mut text, b": ");
            diagnostic_tail(&mut text, program);
            if !detail.is_empty() {
                diagnostic_part(&mut text, b"\n");
                diagnostic_tail(&mut text, detail);
            }
        }
        agent::RunResult::Failed { .. }
        | agent::RunResult::Accepted { .. }
        | agent::RunResult::Parked
        | agent::RunResult::Refused { .. } => diagnostic_tail(&mut text, detail),
    }
    text.into_boxed()
}

fn diagnostic_tail(text: &mut List<u8>, bytes: &[u8]) {
    let from = bytes.len().saturating_sub(skein_lib::usize_of(text.room()));
    diagnostic_part(text, bytes.get(from..).expect("bounded suffix"));
}

fn diagnostic_part(text: &mut List<u8>, bytes: &[u8]) {
    for byte in bytes {
        if text.room() == 0 {
            break;
        }
        let byte = if byte.is_ascii_control() && *byte != b'\n' && *byte != b'\t' { b'?' } else { *byte };
        text.push(byte).expect("diagnostic room");
    }
}
