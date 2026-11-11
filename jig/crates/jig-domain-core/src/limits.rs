//! Limit validity and per-value policy and settings carriers for the core
//! (`domain/engine/README.md`, section 5.6, and `domain/engine/briefs.md`, section 2, and `domain/engine/settings.md`, section 9; programming-model.md, 6.3).

use alloc::boxed::Box;
use core::mem::size_of;
use jig_domain_accounts as accounts;
use jig_domain_authority as authority;
use jig_domain_brief as brief;
use jig_domain_fleet as fleet;
use jig_domain_link::{Membership, Problem};
use jig_domain_notes as notes;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use jig_domain_views as views;
use skein_lib::{List, length::len_u64};

use crate::Limits;

/// Maximum bytes in a charter document's path (domain/engine/settings.md, 9).
pub const DOCUMENT_PATH_BYTES: u32 = 256;
/// Maximum bytes in a charter document's heading (domain/engine/settings.md, 9).
pub const DOCUMENT_HEADING_BYTES: u32 = 80;
/// The most sections the core plans of its own (domain/engine/briefs.md, section 1, and domain/engine/settings.md, section 9).
pub const CORE_SECTIONS: u32 = 8;
/// The connector section kind the core plans a document under; the root may renumber it
/// (domain/engine/settings.md, 9).
pub const DOCUMENT_KIND: u16 = 0;
/// A charter document section's priority (domain/engine/settings.md, 9).
pub const DOCUMENT_PRIORITY: u16 = 5;

/// Whether a brief's sections hold the core's own most and a charter's documents
/// (domain/engine/settings.md, 9).
#[must_use]
pub fn brief_holds_documents(sections: u32, charter_documents: u32) -> bool {
    match CORE_SECTIONS.checked_add(charter_documents) {
        Some(needed) => sections >= needed,
        None => false,
    }
}

/// The bytes a charter owns beyond its struct: its policy's, as `policy_owned_bytes`,
/// its parts' as `parts_owned_bytes`, and its documents' structs, name segments, paths
/// and headings (jig/docs/design/domain/tasks/messages.md, 6; domain/engine/settings.md, 9).
#[must_use]
pub fn charter_owned_bytes(charter: &crate::Charter) -> Option<u64> {
    let mut bytes = policy_owned_bytes(&charter.policy)?
        .checked_add(parts_owned_bytes(&charter.parts)?)?
        .checked_add(sizeof(charter.documents.len())?.checked_mul(sizeof(size_of::<crate::Document>())?)?)?;
    for document in &charter.documents {
        bytes = bytes
            .checked_add(sizeof(document.name.segments.len())?.checked_mul(sizeof(size_of::<Box<[u8]>>())?)?)?
            .checked_add(sizeof(document.path.len())?)?;
        for segment in &document.name.segments {
            bytes = bytes.checked_add(sizeof(segment.len())?)?;
        }
        if let Some(heading) = &document.heading {
            bytes = bytes.checked_add(sizeof(heading.len())?)?;
        }
    }
    Some(bytes)
}

/// The bytes a charter's parts add to its stored policy: each part's own size, its
/// instructions and the two bytes of the blank line that joins them
/// (jig/docs/design/protocol/limits.md).
#[must_use]
pub fn parts_owned_bytes(parts: &[crate::CharterPart]) -> Option<u64> {
    let mut bytes = sizeof(parts.len())?.checked_mul(sizeof(size_of::<crate::CharterPart>())?)?;
    for part in parts {
        bytes = bytes.checked_add(sizeof(part.instructions.len())?)?.checked_add(2)?;
    }
    Some(bytes)
}

// A held authority policy is priced by its people snapshot's `policy_bytes`, which counts the
// people mirrors' sizes; each mirror is at least as large as the authority type it copies.
const _: () = assert!(
    size_of::<authority::Role>() <= size_of::<people::PolicyRole>(),
    "people's PolicyRole prices authority's Role"
);
const _: () = assert!(
    size_of::<authority::Authority>() <= size_of::<people::Authority>(),
    "people's Authority prices authority's Authority"
);
const _: () =
    assert!(size_of::<authority::Grant>() <= size_of::<people::Grant>(), "people's Grant prices authority's Grant");
const _: () = assert!(
    size_of::<authority::ResourceScope>() <= size_of::<people::ResourceScope>(),
    "people's ResourceScope prices authority's ResourceScope"
);
const _: () = assert!(
    size_of::<authority::Executor>() <= size_of::<people::Executor>(),
    "people's Executor prices authority's Executor"
);
const _: () = assert!(
    size_of::<authority::Requirement>() <= size_of::<people::Requirement>(),
    "people's Requirement prices authority's Requirement"
);

/// The bytes a run's policy from `policy`, a charter's, owns whatever its task's LLM choice: its
/// models, and its largest once more for a main model its sub-agents name again, each with its
/// longest declared effort chosen (domain/engine/executors.md, 1).
#[must_use]
pub fn policy_owned_bytes(policy: &crate::RunPolicy) -> Option<u64> {
    let mut largest = model_owned_bytes(&policy.model)?;
    let mut bytes = sizeof(size_of::<crate::RunPolicy>())?
        .checked_add(len_u64(policy.instructions.len()))?
        .checked_add(largest)?
        .checked_add(
            len_u64(policy.alternatives.len()).checked_add(1)?.checked_mul(sizeof(size_of::<crate::Model>())?)?,
        )?;
    for model in &policy.alternatives {
        let owned = model_owned_bytes(model)?;
        bytes = bytes.checked_add(owned)?;
        largest = largest.max(owned);
    }
    bytes.checked_add(largest)
}

/// A model's name, its efforts' names and its chosen one's, at most its longest (domain/tasks/README.md, 3).
fn model_owned_bytes(model: &crate::Model) -> Option<u64> {
    let mut longest = 0_u64;
    let mut bytes = len_u64(model.name.len())
        .checked_add(len_u64(model.efforts.len()).checked_mul(sizeof(size_of::<Box<[u8]>>())?)?)?;
    for effort in &model.efforts {
        let length = len_u64(effort.len());
        bytes = bytes.checked_add(length)?;
        longest = longest.max(length);
    }
    if let Some(effort) = &model.effort {
        longest = longest.max(len_u64(effort.len()));
    }
    if let Some(program) = &model.program {
        bytes = bytes.checked_add(len_u64(program.len()))?;
    }
    bytes.checked_add(longest)
}

fn bytes(count: u32, each: u64) -> Option<u64> {
    u64::from(count).checked_mul(each)
}

fn sizeof(value: usize) -> Option<u64> {
    u64::try_from(value).ok()
}

pub(crate) fn task_carrier(limits: &Limits) -> Option<u64> {
    let task = &limits.tasks;
    let names = bytes(
        task.holdings,
        sizeof(size_of::<tasks::Holding>())?
            .checked_add(bytes(task.hold_segments, sizeof(size_of::<Box<[u8]>>())?)?)?
            .checked_add(u64::from(task.hold_bytes))?,
    )?;
    let authority = bytes(
        task.authority_grants,
        sizeof(size_of::<tasks::Grant>())?
            .checked_add(bytes(task.authority_segments.checked_mul(2)?, sizeof(size_of::<Box<[u8]>>())?)?)?,
    )?
    .checked_add(u64::from(task.authority_bytes))?
    .checked_add(bytes(task.executor_kinds, sizeof(size_of::<tasks::AuthorityExecutor>())?)?)?
    .checked_add(bytes(task.authority_grants, sizeof(size_of::<tasks::ResourceScope>())?)?)?;
    sizeof(size_of::<tasks::New>())?
        .checked_add(u64::from(task.spec_bytes))?
        .checked_add(u64::from(task.message_bytes))?
        .checked_add(bytes(task.parameters, sizeof(size_of::<tasks::Parameter>())?)?)?
        .checked_add(bytes(task.inputs, sizeof(size_of::<u64>())?)?)?
        .checked_add(bytes(task.dependencies, sizeof(size_of::<u64>())?)?)?
        .checked_add(bytes(task.contract_choices, sizeof(size_of::<tasks::Verdict>())?)?)?
        .checked_add(bytes(
            task.contract_choices,
            u64::from(tasks::VERDICT_FIELDS.min(task.result_bytes))
                .checked_mul(sizeof(size_of::<tasks::FieldRule>())?)?
                .checked_add(u64::from(task.result_bytes))?,
        )?)?
        .checked_add(u64::from(task.result_bytes))?
        .checked_add(List::<Box<[u8]>>::price(task.hold_segments)?)?
        .checked_add(u64::from(task.hold_bytes))?
        .checked_add(names)?
        .checked_add(authority)?
        .checked_add(bytes(tasks::MODELS, sizeof(size_of::<tasks::ModelEffort>())?)?)
}

fn context_carrier(limits: &Limits) -> Option<u64> {
    let task = &limits.tasks;
    task_carrier(limits)?
        .checked_add(sizeof(size_of::<tasks::RunContext>())?)?
        .checked_add(bytes(task.dependencies, sizeof(size_of::<tasks::Added>())?.checked_add(8)?)?)?
        .checked_add(bytes(task.inbox_messages, sizeof(size_of::<tasks::Word>())?)?)?
        .checked_add(u64::from(task.inbox_bytes))?
        .checked_add(u64::from(task.inbox_bytes).checked_mul(sizeof(size_of::<tasks::ResultField>())?)?)?
        .checked_add(bytes(task.delegates, sizeof(size_of::<tasks::DelegateState>())?)?)?
        .checked_add(bytes(task.saved_resources, sizeof(size_of::<tasks::SavedResource>())?)?)?
        .checked_add(u64::from(task.result_bytes))
}

pub(crate) fn problems(names: Box<[Box<[u8]>]>, limits: &Limits) -> Result<Membership, Problem> {
    Membership::new(names, limits.fleet.configured_workers, limits.fleet.worker_name_bytes)
}

/// Whether the core's limits and edge bounds are representable (programming-model.md, 6.3).
#[must_use]
pub fn valid(limits: &Limits) -> bool {
    checked_valid(limits).is_some()
}

fn checked_valid(limits: &Limits) -> Option<()> {
    if limits.tasks.circle > people::NAMED_TASKS {
        return None;
    }
    if limits.people.program_hosts < limits.fleet.configured_workers.checked_add(1)?
        || limits.people.program_host_name_bytes < limits.fleet.worker_name_bytes
        || limits.accounts.program_hosts < limits.fleet.configured_workers.checked_add(1)?
        || limits.people.operations > limits.tasks.tasks
        || limits.accounts.program_names > limits.fleet.program_sources
        || limits.documents.roles < limits.authority.roles
        || limits.settings.revisions == 0
        || limits.settings.versions == 0
        || limits.settings.history == 0
        || limits.settings.proposals == 0
        || limits.routes == 0
        || limits.templates == 0
        || limits.resume_bytes == 0
        || limits.run_bytes == 0
        || limits.call_records == 0
        || limits.call_answer_bytes == 0
        || limits.tasks.inbox_messages < limits.people.inbox_entries
        || limits.people.inbox_tasks < limits.tasks.tasks.checked_mul(2)?
        || limits.people.inbox_recipients < crate::inbox::waiting_capacity(limits)?
        || limits.fleet.grants < limits.accounts.credentials.checked_add(limits.accounts.accounts.min(tasks::MODELS))?
        || limits.authority.accounts < limits.accounts.credentials.max(limits.accounts.accounts.min(tasks::MODELS))
        || limits.notes.scopes < 3_u32.checked_add(limits.authority.grants)?
    {
        return None;
    }
    if !tasks::valid(
        &limits.tasks,
        tasks::Kept { revisions: limits.settings.revisions, versions: limits.settings.versions },
    ) || !people::valid(&limits.people)
        || !fleet::valid(&limits.fleet, limits.settings.versions)
        || !brief::valid(&limits.brief)
        || !accounts::valid(&limits.accounts)
        || !notes::valid(&limits.notes)
        || !views::valid(&limits.views)
        || limits.load_bytes == 0
        || limits.load_slots == 0
    {
        return None;
    }
    limits.settings.revisions.checked_add(1)?;
    limits.settings.versions.checked_add(1)?;
    // Constructor counts must fit independently of the inbox and settings subtotals.
    crate::inbox::question_capacity(limits)?;
    limits.people.people.checked_mul(limits.people.inbox_entries)?.checked_add(1)?;
    limits.settings.versions.checked_add(crate::settings::capacity(limits)?.checked_mul(4)?)?;
    limits.tasks.charters.checked_mul(2)?.checked_add(limits.settings.versions)?;
    crate::documents::document_with_context_bound(limits, &[])?;
    crate::settings::current_bound(limits)?;
    crate::settings::revision_bound(limits)?;
    crate::reads::query_bound(limits)?;
    crate::reads::answer_bound(limits, &[])?;
    task_carrier(limits)?;
    context_carrier(limits)?;
    Some(())
}

#[cfg(test)]
mod tests {
    use super::{CORE_SECTIONS, brief_holds_documents, charter_owned_bytes, policy_owned_bytes};
    use crate::Document;
    use alloc::boxed::Box;
    use core::mem::size_of;
    use jig_domain_authority::Name;

    #[test]
    fn brief_holds_documents_at_the_section_boundary() {
        for documents in [0, 1, 2, 8] {
            let sections = CORE_SECTIONS + documents;
            assert!(brief_holds_documents(sections, documents));
            assert!(!brief_holds_documents(sections - 1, documents));
        }
        assert!(!brief_holds_documents(u32::MAX, u32::MAX));
    }

    #[test]
    fn charter_owned_bytes_counts_each_document_and_its_name_path_and_heading() {
        let config = crate::tests::config(1);
        let mut charter = config.defaults.charters.first().expect("fixture charter").clone();
        let policy = policy_owned_bytes(&charter.policy).expect("fixture policy bytes");
        assert_eq!(charter_owned_bytes(&charter), Some(policy));

        let first = Document {
            connector: 0,
            name: Name { segments: Box::new([Box::from(b"one".as_slice()), Box::from(b"two".as_slice())]) },
            path: Box::from(b"rules.md".as_slice()),
            heading: None,
        };
        let first_bytes = u64::try_from(size_of::<Document>() + 2 * size_of::<Box<[u8]>>() + 3 + 3 + 8)
            .expect("first document bytes");
        charter.documents = Box::new([first.clone()]);
        assert_eq!(charter_owned_bytes(&charter), Some(policy + first_bytes));

        let second = Document {
            connector: 1,
            name: Name { segments: Box::new([Box::from(b"other".as_slice())]) },
            path: Box::from(b"guide.md".as_slice()),
            heading: Some(Box::from(b"Work".as_slice())),
        };
        let second_bytes =
            u64::try_from(size_of::<Document>() + size_of::<Box<[u8]>>() + 5 + 8 + 4).expect("second document bytes");
        charter.documents = Box::new([first, second]);
        assert_eq!(charter_owned_bytes(&charter), Some(policy + first_bytes + second_bytes));
    }
}
