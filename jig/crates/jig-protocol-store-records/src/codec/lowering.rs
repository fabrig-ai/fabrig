//! Refused rows are retried at the opening stock before naming a lowering.
//! Generated codec problems carry a path and `Bound`, but not the held figure
//! (jig/docs/design/domain/engine/settings-store.md, 1).
use super::{View, decode, limits, tasks};
use jig_domain_core_vocabulary::{self as core, Key, Record};

/// A value-free lowering. Both figures are in the named setting's own units:
/// `held` is the least source-limit figure that admits this stock-readable
/// record, not a derived codec field's length. Other probed lowerings are
/// restored to stock when measuring it (jig/docs/design/domain/engine/settings-store.md, 1).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Lowered {
    pub name: &'static [u8],
    pub configured: u64,
    pub held: u64,
}

/// Only called after the configured decoder refused. Verify the whole row at
/// stock, including its identity, before considering any lowered source.
#[must_use]
pub fn lowering(
    key: &Key,
    bytes: &[u8],
    cold: Option<&[u8]>,
    configured: &core::Limits,
    stock: &core::Limits,
    names: &[core::settings::ModelName],
) -> Option<Lowered> {
    let stock_view = limits(stock)?;
    if !fits(key, bytes, cold, &stock_view, names) {
        return None;
    }
    if fits(key, bytes, cold, &limits(configured)?, names) {
        return None;
    }
    let mut probe = *configured;
    for name in SOURCES {
        let low = value(configured, name)?;
        let high = value(stock, name)?;
        if low >= high {
            continue;
        }
        set(&mut probe, name, high)?;
        let Some(view) = limits(&probe) else { continue };
        if !fits(key, bytes, cold, &view, names) {
            continue;
        }
        // Other lowered sources have already been restored to stock. This
        // source is necessary even if several fields refused, or a ceiling
        // follows a maximum of several limits. Search only its finite interval.
        let mut refused = low;
        let mut accepted = high;
        for _ in 0_u32..64 {
            if accepted.saturating_sub(refused) <= 1 {
                break;
            }
            let middle = refused.checked_add(accepted.checked_sub(refused)?.checked_div(2)?)?;
            set(&mut probe, name, middle)?;
            let accepted_here = match limits(&probe) {
                Some(view) => fits(key, bytes, cold, &view, names),
                None => false,
            };
            if accepted_here {
                accepted = middle;
            } else {
                refused = middle;
            }
        }
        return Some(Lowered { name, configured: low, held: accepted });
    }
    None
}

fn fits(key: &Key, bytes: &[u8], cold: Option<&[u8]>, view: &View, names: &[core::settings::ModelName]) -> bool {
    let record = match cold {
        Some(cold) => match tasks::decode_live(bytes, cold, &view.tasks, names) {
            Ok(row) => Some(Record::Tasks(row)),
            Err(_) => None,
        },
        None => decode(key, bytes, view, names),
    };
    match record {
        Some(record) => record.key() == *key,
        None => false,
    }
}

// Source names, not generated field paths: deriving the probes through `limits`
// keeps composite/piece ceilings and nested family decoders in the same check.
const SOURCES: &[&[u8]] = &[
    b"tasks.exemptions",
    b"tasks.delegates",
    b"tasks.references",
    b"tasks.subscriptions",
    b"tasks.batch",
    b"tasks.batch_bytes",
    b"tasks.dependencies",
    b"tasks.circle",
    b"tasks.holdings",
    b"tasks.hold_segments",
    b"tasks.hold_bytes",
    b"tasks.inputs",
    b"tasks.spec_bytes",
    b"tasks.parameters",
    b"tasks.result_bytes",
    b"tasks.run_inbox",
    b"tasks.inbox_messages",
    b"tasks.message_bytes",
    b"tasks.saved_resources",
    b"tasks.contract_choices",
    b"tasks.charters",
    b"tasks.charter_parts",
    b"tasks.charter_documents",
    b"tasks.authority_grants",
    b"tasks.authority_segments",
    b"tasks.authority_bytes",
    b"tasks.executor_kinds",
    b"people.program_hosts",
    b"people.program_host_name_bytes",
    b"people.operations",
    b"people.batch",
    b"people.holdings",
    b"people.identity_bytes",
    b"people.words",
    b"people.amendment_bytes",
    b"authority.roles",
    b"authority.requirements",
    b"authority.grants",
    b"authority.executors",
    b"authority.segments",
    b"authority.segment_bytes",
    b"authority.implications",
    b"notes.pattern_bytes",
    b"notes.description_bytes",
    b"notes.body_bytes",
    b"notes.references",
    b"notes.recalled",
    b"documents.text_bytes",
    b"accounts.credentials",
    b"core.settings.alternatives",
    b"core.settings.permissions",
    b"authority.projects",
    b"authority.batch",
    b"authority.writes",
    b"authority.accounts",
    b"core.templates",
    b"core.connectors",
    b"core.run_bytes",
    b"core.policy_bytes",
    b"core.escalation_reason_bytes",
    b"core.call_answer_bytes",
];

fn value(limits: &core::Limits, name: &[u8]) -> Option<u64> {
    Some(match name {
        b"tasks.exemptions" => u64::from(limits.tasks.exemptions),
        b"tasks.delegates" => u64::from(limits.tasks.delegates),
        b"tasks.references" => u64::from(limits.tasks.references),
        b"tasks.subscriptions" => u64::from(limits.tasks.subscriptions),
        b"tasks.batch" => u64::from(limits.tasks.batch),
        b"tasks.batch_bytes" => u64::from(limits.tasks.batch_bytes),
        b"tasks.dependencies" => u64::from(limits.tasks.dependencies),
        b"tasks.circle" => u64::from(limits.tasks.circle),
        b"tasks.holdings" => u64::from(limits.tasks.holdings),
        b"tasks.hold_segments" => u64::from(limits.tasks.hold_segments),
        b"tasks.hold_bytes" => u64::from(limits.tasks.hold_bytes),
        b"tasks.inputs" => u64::from(limits.tasks.inputs),
        b"tasks.spec_bytes" => u64::from(limits.tasks.spec_bytes),
        b"tasks.parameters" => u64::from(limits.tasks.parameters),
        b"tasks.result_bytes" => u64::from(limits.tasks.result_bytes),
        b"tasks.run_inbox" => u64::from(limits.tasks.run_inbox),
        b"tasks.inbox_messages" => u64::from(limits.tasks.inbox_messages),
        b"tasks.message_bytes" => u64::from(limits.tasks.message_bytes),
        b"tasks.saved_resources" => u64::from(limits.tasks.saved_resources),
        b"tasks.contract_choices" => u64::from(limits.tasks.contract_choices),
        b"tasks.charters" => u64::from(limits.tasks.charters),
        b"tasks.charter_parts" => u64::from(limits.tasks.charter_parts),
        b"tasks.charter_documents" => u64::from(limits.tasks.charter_documents),
        b"tasks.authority_grants" => u64::from(limits.tasks.authority_grants),
        b"tasks.authority_segments" => u64::from(limits.tasks.authority_segments),
        b"tasks.authority_bytes" => u64::from(limits.tasks.authority_bytes),
        b"tasks.executor_kinds" => u64::from(limits.tasks.executor_kinds),
        b"people.program_hosts" => u64::from(limits.people.program_hosts),
        b"people.program_host_name_bytes" => u64::from(limits.people.program_host_name_bytes),
        b"people.operations" => u64::from(limits.people.operations),
        b"people.batch" => u64::from(limits.people.batch),
        b"people.holdings" => u64::from(limits.people.holdings),
        b"people.identity_bytes" => u64::from(limits.people.identity_bytes),
        b"people.words" => u64::from(limits.people.words),
        b"people.amendment_bytes" => u64::from(limits.people.amendment_bytes),
        b"authority.roles" => u64::from(limits.authority.roles),
        b"authority.requirements" => u64::from(limits.authority.requirements),
        b"authority.grants" => u64::from(limits.authority.grants),
        b"authority.executors" => u64::from(limits.authority.executors),
        b"authority.segments" => u64::from(limits.authority.segments),
        b"authority.segment_bytes" => u64::from(limits.authority.segment_bytes),
        b"authority.implications" => u64::from(limits.authority.implications),
        b"notes.pattern_bytes" => u64::from(limits.notes.pattern_bytes),
        b"notes.description_bytes" => u64::from(limits.notes.description_bytes),
        b"notes.body_bytes" => u64::from(limits.notes.body_bytes),
        b"notes.references" => u64::from(limits.notes.references),
        b"notes.recalled" => u64::from(limits.notes.recalled),
        b"documents.text_bytes" => u64::from(limits.documents.text_bytes),
        b"accounts.credentials" => u64::from(limits.accounts.credentials),
        b"core.settings.alternatives" => u64::from(limits.settings.alternatives),
        b"core.settings.permissions" => u64::from(limits.settings.permissions),
        b"authority.projects" => u64::from(limits.authority.projects),
        b"authority.batch" => u64::from(limits.authority.batch),
        b"authority.writes" => u64::from(limits.authority.writes),
        b"authority.accounts" => u64::from(limits.authority.accounts),
        b"core.templates" => u64::from(limits.templates),
        b"core.connectors" => u64::from(limits.connectors),
        b"core.run_bytes" => u64::from(limits.run_bytes),
        b"core.policy_bytes" => limits.policy_bytes,
        b"core.escalation_reason_bytes" => u64::from(limits.escalation_reason_bytes),
        b"core.call_answer_bytes" => u64::from(limits.call_answer_bytes),
        _ => return None,
    })
}

fn set(limits: &mut core::Limits, name: &[u8], value: u64) -> Option<()> {
    match name {
        b"tasks.exemptions" => limits.tasks.exemptions = u32::try_from(value).ok()?,
        b"tasks.delegates" => limits.tasks.delegates = u32::try_from(value).ok()?,
        b"tasks.references" => limits.tasks.references = u32::try_from(value).ok()?,
        b"tasks.subscriptions" => limits.tasks.subscriptions = u32::try_from(value).ok()?,
        b"tasks.batch" => limits.tasks.batch = u32::try_from(value).ok()?,
        b"tasks.batch_bytes" => limits.tasks.batch_bytes = u32::try_from(value).ok()?,
        b"tasks.dependencies" => limits.tasks.dependencies = u32::try_from(value).ok()?,
        b"tasks.circle" => limits.tasks.circle = u32::try_from(value).ok()?,
        b"tasks.holdings" => limits.tasks.holdings = u32::try_from(value).ok()?,
        b"tasks.hold_segments" => limits.tasks.hold_segments = u32::try_from(value).ok()?,
        b"tasks.hold_bytes" => limits.tasks.hold_bytes = u32::try_from(value).ok()?,
        b"tasks.inputs" => limits.tasks.inputs = u32::try_from(value).ok()?,
        b"tasks.spec_bytes" => {
            limits.tasks.spec_bytes = u32::try_from(value).ok()?;
            limits.people.spec_bytes = limits.tasks.spec_bytes;
        }
        b"tasks.parameters" => limits.tasks.parameters = u32::try_from(value).ok()?,
        b"tasks.result_bytes" => limits.tasks.result_bytes = u32::try_from(value).ok()?,
        b"tasks.run_inbox" => limits.tasks.run_inbox = u32::try_from(value).ok()?,
        b"tasks.inbox_messages" => limits.tasks.inbox_messages = u32::try_from(value).ok()?,
        b"tasks.message_bytes" => limits.tasks.message_bytes = u32::try_from(value).ok()?,
        b"tasks.saved_resources" => limits.tasks.saved_resources = u32::try_from(value).ok()?,
        b"tasks.contract_choices" => limits.tasks.contract_choices = u32::try_from(value).ok()?,
        b"tasks.charters" => limits.tasks.charters = u32::try_from(value).ok()?,
        b"tasks.charter_parts" => limits.tasks.charter_parts = u32::try_from(value).ok()?,
        b"tasks.charter_documents" => limits.tasks.charter_documents = u32::try_from(value).ok()?,
        b"tasks.authority_grants" => limits.tasks.authority_grants = u32::try_from(value).ok()?,
        b"tasks.authority_segments" => limits.tasks.authority_segments = u32::try_from(value).ok()?,
        b"tasks.authority_bytes" => limits.tasks.authority_bytes = u32::try_from(value).ok()?,
        b"tasks.executor_kinds" => limits.tasks.executor_kinds = u32::try_from(value).ok()?,
        b"people.program_hosts" => limits.people.program_hosts = u32::try_from(value).ok()?,
        b"people.program_host_name_bytes" => limits.people.program_host_name_bytes = u32::try_from(value).ok()?,
        b"people.operations" => limits.people.operations = u32::try_from(value).ok()?,
        b"people.batch" => limits.people.batch = u32::try_from(value).ok()?,
        b"people.holdings" => limits.people.holdings = u32::try_from(value).ok()?,
        b"people.identity_bytes" => limits.people.identity_bytes = u32::try_from(value).ok()?,
        b"people.words" => limits.people.words = u32::try_from(value).ok()?,
        b"people.amendment_bytes" => limits.people.amendment_bytes = u32::try_from(value).ok()?,
        b"authority.roles" => limits.authority.roles = u32::try_from(value).ok()?,
        b"authority.requirements" => limits.authority.requirements = u32::try_from(value).ok()?,
        b"authority.grants" => limits.authority.grants = u32::try_from(value).ok()?,
        b"authority.executors" => limits.authority.executors = u32::try_from(value).ok()?,
        b"authority.segments" => limits.authority.segments = u32::try_from(value).ok()?,
        b"authority.segment_bytes" => limits.authority.segment_bytes = u32::try_from(value).ok()?,
        b"authority.implications" => limits.authority.implications = u32::try_from(value).ok()?,
        b"notes.pattern_bytes" => limits.notes.pattern_bytes = u32::try_from(value).ok()?,
        b"notes.description_bytes" => limits.notes.description_bytes = u32::try_from(value).ok()?,
        b"notes.body_bytes" => limits.notes.body_bytes = u32::try_from(value).ok()?,
        b"notes.references" => limits.notes.references = u32::try_from(value).ok()?,
        b"notes.recalled" => limits.notes.recalled = u32::try_from(value).ok()?,
        b"documents.text_bytes" => limits.documents.text_bytes = u32::try_from(value).ok()?,
        b"accounts.credentials" => limits.accounts.credentials = u32::try_from(value).ok()?,
        b"core.settings.alternatives" => limits.settings.alternatives = u32::try_from(value).ok()?,
        b"core.settings.permissions" => limits.settings.permissions = u32::try_from(value).ok()?,
        b"authority.projects" => limits.authority.projects = u32::try_from(value).ok()?,
        b"authority.batch" => limits.authority.batch = u32::try_from(value).ok()?,
        b"authority.writes" => limits.authority.writes = u32::try_from(value).ok()?,
        b"authority.accounts" => limits.authority.accounts = u32::try_from(value).ok()?,
        b"core.templates" => limits.templates = u32::try_from(value).ok()?,
        b"core.connectors" => limits.connectors = u32::try_from(value).ok()?,
        b"core.run_bytes" => limits.run_bytes = u32::try_from(value).ok()?,
        b"core.policy_bytes" => limits.policy_bytes = value,
        b"core.escalation_reason_bytes" => limits.escalation_reason_bytes = u32::try_from(value).ok()?,
        b"core.call_answer_bytes" => limits.call_answer_bytes = u32::try_from(value).ok()?,
        _ => return None,
    }
    Some(())
}
