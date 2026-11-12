//! Dense revision read values (domain/engine/settings.md, 7).
use super::{RevisionAnswer, RevisionsPage, Row, SettingDocument};
use crate::{CoreRecord, Limits, Record, settings};
use skein_lib::Map;

/// Run reads use the same documents and load slots as party reads (domain/engine/executors.md, 3).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum SettingsRead {
    Inventory,
    Setting { setting: settings::Setting, revision: u32 },
    Revision { number: u32 },
    Revisions { before: Option<u32> },
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum SettingsAnswer {
    Inventory(crate::documents::Settings),
    Setting(SettingDocument),
    Revision(RevisionAnswer),
    Revisions(RevisionsPage),
    Unknown,
}
/// One proposal of settings, as a run reads it (domain/engine/executors.md, 3).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
#[cfg_attr(
    target_pointer_width = "64",
    expect(clippy::large_enum_variant, reason = "the proposal read contract keeps both documents inline")
)]
pub enum ProposalRead {
    /// Waiting, with its review, as a party's live proposal read gives it, with no acts.
    Waiting(crate::documents::Proposal),
    /// Decided, withdrawn or replaced, as its history row keeps it.
    Decided(crate::reads::Ended),
    /// None, not settings, or a task's of another project.
    Unknown,
}
/// A selected revision is decoded whole before its values are admitted separately.
#[must_use]
pub fn load_bytes(limits: &Limits, rows: u32) -> Option<u64> {
    settings::revision_bound(limits)?
        .checked_add(u64::try_from(size_of::<Row>()).ok()?)?
        .checked_add(u64::try_from(size_of::<Record>()).ok()?)?
        .checked_mul(u64::from(rows))
}
/// Validate one decoded history record before it crosses a load boundary.
/// Its set values occupy at most one whole bounded settings set.
#[must_use]
pub fn valid_revision(revision: &settings::Revision, limits: &Limits) -> bool {
    let Some(capacity) = settings::capacity(limits) else { return false };
    let Some(capacity) = capacity.checked_mul(2) else { return false };
    let Some(capacity) = capacity.checked_add(1) else { return false };
    let Ok(count) = u32::try_from(revision.items.len()) else { return false };
    if count > capacity {
        return false;
    }
    let mut seen = Map::with_capacity(count);
    let mut charters = 0_u32;
    let mut templates = 0_u32;
    let mut policies = 0_u32;
    for item in &revision.items {
        if seen.insert(item.setting, ()).ok() != Some(None) {
            return false;
        }
        match &item.edit {
            settings::Edit::Set(value) => {
                if settings::check::shape(item.setting, value, limits).is_err() {
                    return false;
                }
                match value {
                    settings::Value::Charter(value) => {
                        charters = charters.saturating_add(1);
                        if charters > limits.tasks.charters
                            || value.policy.alternatives.len()
                                > skein_lib::length::usize_of(limits.settings.alternatives)
                        {
                            return false;
                        }
                    }
                    settings::Value::Overlay(_) => {
                        charters = charters.saturating_add(1);
                        if charters > limits.tasks.charters {
                            return false;
                        }
                    }
                    settings::Value::Template(_) => {
                        templates = templates.saturating_add(1);
                        if templates > limits.templates {
                            return false;
                        }
                    }
                    settings::Value::Policy(_) => {
                        if settings::is_project_policy(item.setting) {
                            continue;
                        }
                        policies = policies.saturating_add(1);
                        if policies > limits.authority.projects {
                            return false;
                        }
                    }
                    settings::Value::Rules(_) | settings::Value::Model(_) => {}
                    settings::Value::Application(_) => unreachable!("an application's setting: apps-core"),
                }
            }
            settings::Edit::Removed | settings::Edit::Reset => {}
        }
    }
    true
}
#[must_use]
pub fn record_revision(value: &Record) -> Option<&settings::Revision> {
    match value {
        Record::Core(CoreRecord::Settings(settings::Record::Revision { revision, .. })) => Some(revision),
        Record::Core(
            CoreRecord::Settings(
                settings::Record::Application { .. }
                | settings::Record::Default { .. }
                | settings::Record::Changed { .. }
                | settings::Record::Charter { .. }
                | settings::Record::Proposal { .. }
                | settings::Record::Extent { .. },
            )
            | CoreRecord::Deployment(_)
            | CoreRecord::RunProof(_)
            | CoreRecord::Call(_)
            | CoreRecord::Terminal(_)
            | CoreRecord::Turn(_)
            | CoreRecord::Projection(_)
            | CoreRecord::EscalationDecision(_)
            | CoreRecord::ProposalDecision(_),
        )
        | Record::People(_)
        | Record::Tasks(_)
        | Record::Notes(_)
        | Record::Accounts(_) => None,
    }
}
