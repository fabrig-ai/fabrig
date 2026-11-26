//! Application-owned setting values (jig/docs/design/domain/engine/settings-application.md, 1).
use super::Setting;
use crate::Limits;
use alloc::boxed::Box;
pub use jig_domain_contract::settings::{Edit, Item, Restored, RestoredEdit};

/// Where a kind's values are kept (jig/docs/design/domain/engine/settings-application.md, 1).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum KindScope {
    Deployment,
    Project,
}
/// A declared application kind and its bounds (jig/docs/design/domain/engine/settings-application.md, 1).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Kind {
    pub number: u32,
    pub scope: KindScope,
    pub connector: u16,
    pub value_bytes: u32,
    pub per_scope: u32,
    pub charters: u8,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Follows {
    Default,
    Changed { revision: u32 },
    Removed { revision: u32 },
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct State {
    pub follows: Follows,
    pub size: u32,
    pub charters: Box<[u32]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Recorded {
    pub kind: u32,
    pub scope: u32,
    pub number: u32,
    pub edit: SizedEdit,
}
/// The state and size an application item replaced (jig/docs/design/domain/engine/settings.md, 10).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Replaced {
    Changed { size: u32 },
    Default,
    Removed,
    NotOffered,
}
/// A revision's application item (jig/docs/design/domain/engine/settings.md, 10).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct RevisionRecorded {
    pub kind: u32,
    pub scope: u32,
    pub number: u32,
    pub edit: SizedEdit,
    pub replaced: Replaced,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum SizedEdit {
    Set { size: u32 },
    Remove,
    Reset,
}
#[must_use]
pub fn capacity(kinds: &[Kind], limits: &Limits) -> Option<u32> {
    let mut count = 0_u32;
    for kind in kinds {
        let scopes = match kind.scope {
            KindScope::Deployment => 1,
            KindScope::Project => limits.authority.projects,
        };
        count = count.checked_add(kind.per_scope.checked_mul(scopes)?)?;
    }
    Some(count)
}
#[must_use]
pub fn is_application(setting: Setting) -> bool {
    match setting {
        Setting::Application { .. } => true,
        Setting::Charter(_)
        | Setting::Template(_)
        | Setting::Policy(_)
        | Setting::Provider(_)
        | Setting::Worker(_)
        | Setting::Model(_)
        | Setting::Rules
        | Setting::Limits
        | Setting::ProjectPolicy { .. } => false,
    }
}
