//! Root-configured admission and edge limits (domain/tasks/README.md, section 5).
use crate::{Class, Retries};

/// Maximum fields in one verdict, equal to the settings-v3 schema's `list 512` ceiling
/// (domain/tasks/README.md, section 6).
pub const VERDICT_FIELDS: u32 = 512;

/// Immutable root-configured limits for live task state, finite sources, owned semantic payloads
/// and retry policies; validate through `valid` before construction. (domain/tasks/README.md, section 5).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Limits {
    /// Placed holds in the deployment, including lifted holds kept for their tasks.
    pub placed_holds: u32,
    /// Placed holds retained in one project.
    pub project_placed_holds: u32,
    /// Placed holds retained for one party owner.
    pub party_placed_holds: u32,
    /// Tasks one standing placed hold may exempt; at most tasks.
    pub exemptions: u32,
    /// Maximum retained live task slots, name-index entries and retry alarms.
    pub tasks: u32,
    /// Maximum retained finite period, pool and recurring allotment identities.
    pub funders: u32,
    /// Maximum live tasks per project, including every member of an admitted batch.
    pub project_tasks: u32,
    /// Maximum live tracked tasks per project.
    pub project_goals: u32,
    /// Maximum lifetime tasks made in a requester tree, including its root; completed delegates do
    /// not return this capacity.
    pub tree_tasks: u32,
    /// Maximum structural depth below the requester-tree root, which has depth zero.
    pub depth: u32,
    /// Maximum live direct delegates of one requester task.
    pub delegates: u32,
    /// Maximum introduced peer references per live task.
    pub references: u32,
    /// Maximum standing interests per task.
    pub subscriptions: u32,
    /// Maximum directly created tasks in one atomic nonempty `Make` batch.
    pub batch: u32,
    /// Maximum bytes one batch's tasks hold in all, as `batch_bytes` measures them, a recurring
    /// member's template's tasks among them (domain/tasks/README.md, sections 4 and 6).
    pub batch_bytes: u32,
    /// Maximum immutable dependency identities per task; also bounds its remaining live
    /// `waiting_on` subset.
    pub dependencies: u32,
    /// Maximum resources required by one task.
    pub holdings: u32,
    /// Maximum connector resource kinds installed in this deployment.
    pub hold_kinds: u32,
    /// Maximum cached named connector admission reports.
    pub resource_reports: u32,
    /// Maximum distinct pools whose latest connector slot counts are retained.
    pub pools: u32,
    /// Maximum literal segments in one resource name.
    pub hold_segments: u32,
    /// Maximum aggregate literal bytes in one resource name.
    pub hold_bytes: u32,
    /// Maximum tasks waiting on any one resource.
    pub hold_waiters: u32,
    /// Longest span a cancel may give its task to finish.
    pub grace: skein_lib::Duration,
    /// Wall-clock bound before a task waiting for holds is held for review.
    pub hold_wait: skein_lib::Duration,
    /// Maximum tasks a circle of waits names in a refusal or a hold (domain/tasks/holds.md, 7).
    pub circle: u32,
    /// Shape bound for retained input identities checked by the root before delegation.
    pub inputs: u32,
    /// Maximum combined specification words and byte-valued parameter bytes per task.
    pub spec_bytes: u32,
    /// Maximum parameters per specification.
    pub parameters: u32,
    /// Maximum result/reason bytes; contracts cannot allow larger results and cancellation may
    /// retain one bounded reason plus one bounded partial result.
    pub result_bytes: u32,
    /// Messages a run's inbox takes: the most its start carries, and the most relayed
    /// and unread (domain/tasks/messages.md, 2). At most `inbox_messages`.
    pub run_inbox: u32,
    /// Maximum whole unread word messages kept by one live task.
    pub inbox_messages: u32,
    /// Maximum total unread word bytes kept by one live task.
    pub inbox_bytes: u32,
    /// Maximum bytes in one admitted word message.
    pub message_bytes: u32,
    /// Time an active person task waits before becoming news for its requester.
    pub person_stall: skein_lib::Duration,
    /// Time an undecided proposal waits at a nonfinal holder before passing upward.
    pub proposal_stall: skein_lib::Duration,
    /// Time an undecided escalation waits at a nonfinal holder before passing upward.
    pub escalation_stall: skein_lib::Duration,
    /// Maximum connector resources on which a task has saved state.
    pub saved_resources: u32,
    /// Maximum distinct-code choices in a nonempty verdict contract.
    pub contract_choices: u32,
    /// Maximum retained configured agent-charter numbers.
    pub charters: u32,
    /// Maximum documents per charter (domain/engine/settings.md, 9).
    pub charter_documents: u32,
    /// Bounds parts per charter (jig/docs/design/protocol/limits.md).
    pub charter_parts: u32,
    /// Maximum resource grants per task authority carrier.
    pub authority_grants: u32,
    /// Maximum base segments per carried grant pattern.
    pub authority_segments: u32,
    /// Maximum total bytes across every grant's base and terminal segments in one authority
    /// carrier.
    pub authority_bytes: u32,
    /// Maximum carried authority executor-permission entries.
    pub executor_kinds: u32,
    /// Validated per-class failure retry and pause policies.
    pub retries: Retries,
    /// Optional content-free observation capacity; overflow is diagnostic and changes no decision.
    pub facts: u32,
    /// Recent owner-proven room refusals; zero omits history and counts omissions.
    pub room_refusals: u32,
}

const _: () = assert!(
    crate::questions::OWED_WORDS.len() < crate::run::NOTHING_AWAITED_WORDS.len(),
    "the nothing-awaited notice must bound the question-owed notice's word bytes"
);

impl Limits {
    /// The largest retained word, including fixed notices (jig/docs/design/protocol/store/records.md, 1).
    #[must_use]
    pub fn word_bytes(&self) -> u32 {
        self.message_bytes
            .max(self.result_bytes)
            .max(u32::try_from(crate::run::NOTHING_AWAITED_WORDS.len()).expect("fixed notice length"))
    }
}

/// Maximum ended identities named by bounded live dependencies, inputs or references.
#[must_use]
pub(crate) fn stub_capacity(limits: &Limits) -> Option<u32> {
    limits.tasks.checked_mul(
        limits
            .dependencies
            .checked_add(limits.inputs)?
            .checked_add(limits.references)?
            .checked_add(limits.batch.checked_mul(limits.dependencies.checked_add(limits.inputs)?)?)?,
    )
}

/// Whether these limits are representable: every edge bound they size
/// is finite and every capacity they derive fits (programming-model.md, 6.3).
#[must_use]
pub fn valid(limits: &Limits, kept: crate::Kept) -> bool {
    if limits.placed_holds == 0
        || limits.project_placed_holds == 0
        || limits.party_placed_holds == 0
        || limits.exemptions == 0
        || limits.exemptions > limits.tasks
        || limits.circle == 0
        || limits.tasks == 0
        || limits.batch == 0
        || limits.batch_bytes == 0
        || limits.tree_tasks == 0
        || limits.run_inbox == 0
        || limits.run_inbox > limits.inbox_messages
        || limits.inbox_messages == 0
        || limits.inbox_bytes == 0
        || limits.message_bytes == 0
        || limits.message_bytes > limits.inbox_bytes
        || limits.grace == skein_lib::Duration::ZERO
        || limits.hold_wait == skein_lib::Duration::ZERO
        || limits.person_stall == skein_lib::Duration::ZERO
        || limits.proposal_stall == skein_lib::Duration::ZERO
        || limits.escalation_stall == skein_lib::Duration::ZERO
    {
        return false;
    }
    for class in [Class::Transient, Class::Permanent, Class::Run, Class::Agent, Class::Lost, Class::Invalid] {
        let retry = limits.retries.of(class);
        if retry.retries == u32::MAX || retry.base == skein_lib::Duration::ZERO || retry.max < retry.base {
            return false;
        }
    }
    stub_capacity(limits).is_some()
        && limits.tasks.checked_mul(limits.subscriptions).is_some()
        && limits.tasks.checked_mul(limits.holdings).is_some()
        && kept.revisions.checked_add(1).is_some()
        && crate::activity::line_bytes_limit(limits).is_some()
        && crate::waits::valid(limits)
        && limits.tasks.checked_mul(4).is_some()
        && charter_capacity(limits, kept).is_some()
}

pub(crate) fn charter_capacity(limits: &Limits, kept: crate::Kept) -> Option<u32> {
    limits.charters.checked_mul(2)?.checked_add(kept.versions)
}

#[cfg(test)]
mod tests {
    use crate::{Limits, tests::revisions::LIMITS};

    #[test]
    fn word_bytes_covers_fixed_notices_and_caller_limits() {
        // Byte length of the durable notice in run.
        const NOTICE: u32 = 97;
        for (bound, expected) in [(NOTICE - 1, NOTICE), (NOTICE + 1, 98)] {
            for (message, result, inbox, expected) in
                [(bound, 0, 0, expected), (0, bound, 0, expected), (0, 0, bound, NOTICE)]
            {
                let limits = Limits { message_bytes: message, result_bytes: result, inbox_bytes: inbox, ..LIMITS };
                assert_eq!(limits.word_bytes(), expected, "{message}, {result}, {inbox}");
            }
        }
    }
}
