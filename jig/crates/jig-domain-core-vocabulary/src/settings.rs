//! Settings values, revisions and bounds (jig/docs/design/domain/engine/settings.md).
pub mod application;
pub mod catalogue;
pub mod change;
pub mod charters;
pub mod check;
pub mod fleet_needs;
pub mod models;
pub mod overlays;
pub mod revisions;
pub mod templates;
use crate::{Charter, ConfiguredPolicy, Template};
use alloc::boxed::Box;
pub use application::{Kind, KindScope};
pub use catalogue::{CatalogueModel, ModelSource};
pub use change::change_shape;
pub use check::check_change;
use jig_domain_authority as authority;
use jig_domain_people as people;
pub use models::{model_bit, moves, remap};
pub use overlays::{Overlay, overlaid};
pub use revisions::history_copy;
use skein_lib::{List, Map, Wall};

pub use people::{Running, Setting, SettingFault as Fault};
/// A party's setting value (jig/docs/design/domain/engine/settings.md, 1, 5).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Value {
    Charter(Box<Charter>),
    Overlay(Box<Overlay>),
    Template(Box<Template>),
    Policy(Box<ConfiguredPolicy>),
    Rules(Box<authority::Rules>),
    Model(Box<CatalogueModel>),
    Application(Box<[u8]>),
}
impl Value {
    #[must_use]
    pub fn as_model(&self) -> Option<&CatalogueModel> {
        match self {
            Self::Model(value) => Some(value),
            Self::Charter(_)
            | Self::Overlay(_)
            | Self::Template(_)
            | Self::Policy(_)
            | Self::Rules(_)
            | Self::Application(_) => None,
        }
    }
    #[must_use]
    pub fn as_charter(&self) -> Option<&Charter> {
        match self {
            Self::Charter(value) => Some(value),
            Self::Overlay(_)
            | Self::Template(_)
            | Self::Policy(_)
            | Self::Rules(_)
            | Self::Model(_)
            | Self::Application(_) => None,
        }
    }
    #[must_use]
    pub fn as_template(&self) -> Option<&Template> {
        match self {
            Self::Template(value) => Some(value),
            Self::Overlay(_)
            | Self::Charter(_)
            | Self::Policy(_)
            | Self::Rules(_)
            | Self::Model(_)
            | Self::Application(_) => None,
        }
    }
    #[must_use]
    pub fn as_policy(&self) -> Option<&ConfiguredPolicy> {
        match self {
            Self::Policy(value) => Some(value),
            Self::Charter(_)
            | Self::Overlay(_)
            | Self::Template(_)
            | Self::Rules(_)
            | Self::Model(_)
            | Self::Application(_) => None,
        }
    }
    #[must_use]
    pub fn as_rules(&self) -> Option<&authority::Rules> {
        match self {
            Self::Rules(value) => Some(value),
            Self::Charter(_)
            | Self::Overlay(_)
            | Self::Template(_)
            | Self::Policy(_)
            | Self::Model(_)
            | Self::Application(_) => None,
        }
    }
}
#[derive(Debug)]
pub struct Defaults {
    pub charters: Box<[Charter]>,
    pub templates: Box<[Template]>,
    pub policies: Map<u32, ConfiguredPolicy>,
    pub rules: authority::Rules,
    pub models: Box<[CatalogueModel]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Default {
    pub value: Value,
    pub revision: u32,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Changed {
    pub value: Option<Value>,
    pub revision: u32,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Edit {
    Set(Value),
    Removed,
    Reset,
}
/// What an item replaced (jig/docs/design/domain/engine/settings.md, 10).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Replaced {
    Value(Value),
    Default,
    Removed,
    NotOffered,
}
/// A revision's item and what it replaced (jig/docs/design/domain/engine/settings.md, 10).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct RevisionItem {
    pub setting: Setting,
    pub edit: Edit,
    pub replaced: Replaced,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Item {
    pub setting: Setting,
    pub edit: Edit,
}
/// Projects named by a proposal's items (jig/docs/design/domain/engine/settings.md, 6).
pub const PROPOSAL_PROJECTS: u32 = 32;
#[must_use]
pub fn touched_projects(items: &[Item]) -> List<u32> {
    proposal_projects(items, &[])
}
fn proposal_projects(items: &[Item], applications: &[application::Recorded]) -> List<u32> {
    let mut projects = skein_lib::Set::with_capacity(PROPOSAL_PROJECTS);
    for item in items {
        let project = match item.setting {
            Setting::Application { scope: 0, .. }
            | Setting::Charter(_)
            | Setting::Template(_)
            | Setting::Policy(_)
            | Setting::Model(_)
            | Setting::Rules => continue,
            Setting::Application { scope, .. } => scope,
            Setting::ProjectPolicy { project } => project,
        };
        include_project(&mut projects, project);
    }
    for item in applications {
        if item.scope != 0 {
            include_project(&mut projects, item.scope);
        }
    }
    let mut result = List::with_capacity(PROPOSAL_PROJECTS);
    for project in &projects {
        result.push(*project).expect("bounded touched projects");
    }
    result
}
fn include_project(projects: &mut skein_lib::Set<u32>, project: u32) {
    if let Err(project) = projects.insert(project) {
        let last = *projects.last().expect("full project set");
        if project < last {
            assert!(projects.remove(&last), "greatest project is held");
            projects.insert(project).expect("removed greatest project");
        }
    }
}

#[derive(Hash, Debug)]
pub struct Change {
    pub against: u32,
    pub items: Box<[Item]>,
    pub running: Running,
}
#[derive(Hash, Debug)]
pub struct Proposed {
    pub against: u32,
    pub items: Box<[Item]>,
    pub reason: Box<[u8]>,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Decider {
    FirstStart,
    Release,
    Party(u64),
    List { provider: u32 },
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Head {
    pub number: u32,
    pub against: u32,
    pub at: Wall,
    pub by: Decider,
    pub proposal: Option<(people::Proposer, u32)>,
    pub moved: bool,
}
/// A stored catalogue's identity; revision indices are relative to these names.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ModelName {
    pub name: Box<[u8]>,
    pub efforts: Box<[Box<[u8]>]>,
}
#[must_use]
pub fn model_names(models: &[crate::Model]) -> Box<[ModelName]> {
    let mut names = List::with_capacity(u32::try_from(models.len()).expect("configured model count"));
    for model in models {
        names.push(ModelName { name: model.name.clone(), efforts: model.efforts.clone() }).expect("model count");
    }
    names.into_boxed()
}
/// A revision and the state each item replaced (jig/docs/design/domain/engine/settings.md, 10).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Revision {
    pub head: Head,
    pub items: Box<[RevisionItem]>,
    pub applications: Box<[application::RevisionRecorded]>,
}
/// A failed setting check, retaining bounded document facts when applicable (domain/engine/settings.md, 9).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Failure {
    pub setting: Setting,
    pub fault: Fault,
    /// The document member and rule, when a charter document fails.
    pub detail: Option<people::SettingDetail>,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Limits {
    pub revisions: u32,
    pub versions: u32,
    pub history: u32,
    pub proposals: u32,
    pub alternatives: u32,
    pub permissions: u32,
}

/// Addresses sort in the same order as the store's tag 8 subfamilies, then tag 9.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Key {
    Default(Setting),
    Changed(Setting),
    Held(u32),
    Charter { charter: u32, revision: u32 },
    Proposal { proposer: people::Proposer, proposal: u32 },
    Extent,
    History(u32),
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct CharterVersion {
    pub overlay: Option<Box<Overlay>>,
    pub charter: Option<Box<Charter>>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Proposal {
    pub key: Option<u64>,
    pub against: u32,
    pub items: Box<[Item]>,
    pub applications: Box<[application::Recorded]>,
}
impl Proposal {
    /// Project scopes survive application values being kept by their connector (17.5–17.6).
    #[must_use]
    pub fn projects(&self) -> List<u32> {
        proposal_projects(&self.items, &self.applications)
    }
}

/// Defaults and changes own set values; resolved charters cache the catalogue.
/// Revision history stays in the store.
#[derive(Hash, Debug)]
pub struct Current {
    pub resolved: Map<u32, Charter>,
    pub defaults: Map<Setting, Default>,
    pub changes: Map<Setting, Changed>,
    pub applications: Map<Setting, application::State>,
    pub head: Head,
    pub oldest: u32,
    pub keys: u64,
}
impl Current {
    #[must_use]
    pub fn new(mut defaults: Defaults, limits: &crate::Limits, kinds: &[Kind]) -> Self {
        let mut values = Map::with_capacity(capacity(limits).expect("settings count"));
        for model in defaults.models {
            let previous = values
                .insert(Setting::Model(model.number), Default { value: Value::Model(Box::new(model)), revision: 1 })
                .expect("default model bound");
            assert!(previous.is_none(), "unique default model");
        }
        for charter in defaults.charters {
            let previous = values
                .insert(
                    Setting::Charter(charter.number),
                    Default { value: Value::Charter(Box::new(charter)), revision: 1 },
                )
                .expect("default charter bound");
            assert!(previous.is_none(), "unique default charter");
        }
        for template in defaults.templates {
            let previous = values
                .insert(
                    Setting::Template(template.number),
                    Default { value: Value::Template(Box::new(template)), revision: 1 },
                )
                .expect("default template bound");
            assert!(previous.is_none(), "unique default template");
        }
        for _ in 0..defaults.policies.len() {
            let number = *defaults.policies.first().expect("default policy count").0;
            let policy = defaults.policies.remove(&number).expect("default policy");
            values
                .insert(Setting::Policy(number), Default { value: Value::Policy(Box::new(policy)), revision: 1 })
                .expect("default policy bound");
        }
        values
            .insert(Setting::Rules, Default { value: Value::Rules(Box::new(defaults.rules)), revision: 1 })
            .expect("rules slot");
        Self {
            defaults: values,
            resolved: Map::with_capacity(limits.tasks.charters),
            applications: Map::with_capacity(application::capacity(kinds, limits).expect("application settings count")),
            changes: Map::with_capacity(capacity(limits).expect("settings count")),
            head: Head {
                number: 1,
                against: 0,
                at: Wall::EPOCH,
                by: Decider::FirstStart,
                proposal: None,
                moved: false,
            },
            oldest: 0,
            keys: 1,
        }
    }
    #[must_use]
    pub fn value(&self, setting: Setting) -> Option<&Value> {
        match self.changes.get(&setting) {
            Some(change) => change.value.as_ref(),
            None => match self.defaults.get(&setting) {
                Some(default) => Some(&default.value),
                None => None,
            },
        }
    }
    /// The current value and the origin it follows, including explicit removal.
    #[must_use]
    pub fn lookup(&self, setting: Setting) -> (Option<&Value>, Option<(bool, u32)>) {
        match self.changes.get(&setting) {
            Some(change) => (change.value.as_ref(), Some((true, change.revision))),
            None => match self.defaults.get(&setting) {
                Some(default) => (Some(&default.value), Some((false, default.revision))),
                None => (None, None),
            },
        }
    }
    /// The state before an edit (jig/docs/design/domain/engine/settings.md, 10).
    #[must_use]
    pub fn replaced(&self, setting: Setting) -> Replaced {
        match self.lookup(setting) {
            (Some(value), Some((true, _))) => Replaced::Value(value.clone()),
            (None, Some((true, _))) => Replaced::Removed,
            (_, Some((false, _))) => Replaced::Default,
            (_, None) => Replaced::NotOffered,
        }
    }
    #[must_use]
    pub fn default_value(&self, setting: Setting) -> Option<&Value> {
        match self.defaults.get(&setting) {
            Some(default) => Some(&default.value),
            None => None,
        }
    }
    #[must_use]
    pub fn charter(&self, number: u32) -> Option<&Charter> {
        if self.overlay(Setting::Charter(number)).is_some() {
            self.resolved.get(&number)
        } else {
            self.value(Setting::Charter(number))?.as_charter()
        }
    }
    #[must_use]
    pub fn made_charter(&self, setting: Setting, value: &Value) -> Option<Charter> {
        match value {
            Value::Overlay(overlay) => Some(overlaid(self.default_value(setting)?.as_charter()?, overlay)),
            Value::Charter(charter) => Some(charter.as_ref().clone()),
            Value::Template(_) | Value::Policy(_) | Value::Rules(_) | Value::Model(_) | Value::Application(_) => None,
        }
    }
    #[must_use]
    pub fn overlay(&self, setting: Setting) -> Option<&Overlay> {
        match self.value(setting) {
            Some(Value::Overlay(overlay)) => Some(overlay),
            Some(
                Value::Charter(_)
                | Value::Template(_)
                | Value::Policy(_)
                | Value::Rules(_)
                | Value::Model(_)
                | Value::Application(_),
            )
            | None => None,
        }
    }
    #[must_use]
    pub fn resolved_charter(&self, number: u32) -> Option<&Charter> {
        match self.resolved.get(&number) {
            Some(charter) => Some(charter),
            None => self.charter(number),
        }
    }
    #[must_use]
    pub fn template(&self, number: u32) -> Option<&Template> {
        self.value(Setting::Template(number))?.as_template()
    }
    #[must_use]
    pub fn policy(&self, number: u32) -> Option<&ConfiguredPolicy> {
        self.value(Setting::Policy(number))?.as_policy()
    }
    #[must_use]
    pub fn rules(&self) -> &authority::Rules {
        match self.value(Setting::Rules) {
            Some(Value::Rules(value)) => value,
            Some(
                Value::Charter(_)
                | Value::Overlay(_)
                | Value::Template(_)
                | Value::Policy(_)
                | Value::Model(_)
                | Value::Application(_),
            )
            | None => {
                unreachable!("checked settings always offer rules")
            }
        }
    }
    #[must_use]
    pub const fn newest(&self) -> u32 {
        self.head.number
    }
    #[must_use]
    pub const fn newest_head(&self) -> &Head {
        &self.head
    }
    #[must_use]
    pub const fn oldest(&self) -> u32 {
        self.oldest
    }
    #[must_use]
    pub fn changed(&self, setting: Setting) -> bool {
        self.changes.get(&setting).is_some()
    }
    /// Ordered setting names; this scratch holds no values.
    #[must_use]
    pub fn offered(&self) -> List<Setting> {
        offered(&self.defaults, &self.changes)
    }
    #[must_use]
    pub fn offered_charters(&self) -> Box<[u32]> {
        let names = self.offered();
        let mut numbers = List::with_capacity(names.len());
        for &setting in &names {
            match setting {
                Setting::Charter(number) => numbers.push(number).expect("offered charter count"),
                Setting::Template(_)
                | Setting::Policy(_)
                | Setting::Model(_)
                | Setting::Rules
                | Setting::ProjectPolicy { .. }
                | Setting::Application { .. } => {}
            }
        }
        numbers.into_boxed()
    }
    #[must_use]
    pub fn offered_templates(&self) -> Box<[u32]> {
        let names = self.offered();
        let mut numbers = List::with_capacity(names.len());
        for &setting in &names {
            match setting {
                Setting::Template(number) => numbers.push(number).expect("offered template count"),
                Setting::Charter(_)
                | Setting::Policy(_)
                | Setting::Model(_)
                | Setting::Rules
                | Setting::ProjectPolicy { .. }
                | Setting::Application { .. } => {}
            }
        }
        numbers.into_boxed()
    }
}
#[must_use]
pub fn capacity(limits: &crate::Limits) -> Option<u32> {
    limits
        .tasks
        .charters
        .checked_add(limits.templates)?
        .checked_add(limits.authority.projects)?
        .checked_add(1)?
        .checked_add(jig_domain_tasks::MODELS)
}
fn offered(defaults: &Map<Setting, Default>, changes: &Map<Setting, Changed>) -> List<Setting> {
    let mut names =
        Map::with_capacity(defaults.capacity().checked_add(changes.capacity()).expect("settings scratch count"));
    for (&setting, _) in defaults {
        names.insert(setting, ()).expect("default names");
    }
    for (&setting, _) in changes {
        names.insert(setting, ()).expect("change names");
    }
    let mut result = List::with_capacity(names.len());
    for (&setting, ()) in &names {
        if match changes.get(&setting) {
            Some(change) => change.value.is_some(),
            None => defaults.get(&setting).is_some(),
        } {
            result.push(setting).expect("offered names");
        }
    }
    result
}

/// Settings' six record shapes, shared by the core and its store codec.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Record {
    Default { setting: Setting, default: Default },
    Changed { setting: Setting, changed: Changed },
    Application { setting: Setting, state: application::State },
    Revision { history: bool, revision: Revision },
    Extent { oldest: u32, keys: u64 },
    Charter { charter: u32, revision: u32, value: CharterVersion },
    Proposal { proposer: people::Proposer, proposal: u32, value: Proposal },
}
impl Record {
    #[must_use]
    pub fn key(&self) -> Key {
        match self {
            Self::Default { setting, .. } => Key::Default(*setting),
            Self::Changed { setting, .. } | Self::Application { setting, .. } => Key::Changed(*setting),
            Self::Revision { history, revision } => {
                if *history {
                    Key::History(revision.head.number)
                } else {
                    Key::Held(revision.head.number)
                }
            }
            Self::Extent { .. } => Key::Extent,
            Self::Charter { charter, revision, .. } => Key::Charter { charter: *charter, revision: *revision },
            Self::Proposal { proposer, proposal, .. } => Key::Proposal { proposer: *proposer, proposal: *proposal },
        }
    }
}

/// One whole set of settings at the bounds the check enforces. The template
/// carrier shares the conservative construction bound.
#[must_use]
pub fn set_bound(limits: &crate::Limits) -> Option<u64> {
    let segment = u64::from(limits.authority.segment_bytes).checked_add(u64::try_from(size_of::<Box<[u8]>>()).ok()?)?;
    let document = u64::try_from(size_of::<crate::Document>())
        .ok()?
        .checked_add(u64::from(limits.authority.segments).checked_mul(segment)?)?
        .checked_add(u64::from(crate::DOCUMENT_PATH_BYTES))?
        .checked_add(u64::from(crate::DOCUMENT_HEADING_BYTES))?;
    let charters = u64::from(limits.tasks.charters).checked_mul(
        u64::from(limits.run_bytes)
            .checked_add(u64::try_from(size_of::<Charter>()).ok()?)?
            .checked_add(u64::from(limits.tasks.charter_documents).checked_mul(document)?)?,
    )?;
    let templates = u64::from(limits.templates).checked_mul(
        u64::try_from(size_of::<Template>())
            .ok()?
            .checked_add(u64::from(limits.documents.text_bytes))?
            .checked_add(crate::limits::task_carrier(limits)?)?,
    )?;
    // A change or proposal may carry one live project's policy beside all policy settings.
    let policies = u64::from(limits.authority.projects.checked_add(1)?)
        .checked_mul(limits.policy_bytes.checked_add(u64::try_from(size_of::<ConfiguredPolicy>()).ok()?)?)?;
    charters
        .checked_add(templates)?
        .checked_add(policies)?
        .checked_add(authority::rules_bound(&limits.authority)?)?
        .checked_add(u64::try_from(size_of::<authority::Rules>()).ok()?)?
        .checked_add(u64::from(jig_domain_tasks::MODELS).checked_mul(catalogue::bound()?)?)
}
/// Current settings own two set overlays and one resolved charter set (domain/engine/settings.md, 8).
#[must_use]
pub fn current_bound(limits: &crate::Limits) -> Option<u64> {
    let resolved = Map::<u32, Charter>::price(limits.tasks.charters)?.checked_add(
        u64::from(limits.tasks.charters).checked_mul(
            u64::from(limits.run_bytes).checked_add(
                u64::from(limits.tasks.charter_documents).checked_mul(
                    u64::try_from(size_of::<crate::Document>())
                        .ok()?
                        .checked_add(
                            u64::from(limits.authority.segments).checked_mul(
                                u64::from(limits.authority.segment_bytes)
                                    .checked_add(u64::try_from(size_of::<Box<[u8]>>()).ok()?)?,
                            )?,
                        )?
                        .checked_add(u64::from(crate::DOCUMENT_PATH_BYTES))?
                        .checked_add(u64::from(crate::DOCUMENT_HEADING_BYTES))?,
                )?,
            )?,
        )?,
    )?;
    set_bound(limits)?.checked_mul(2)?.checked_add(resolved)
}

/// One record's items and a whole set of replaced values, priced independently of the held heads
/// (jig/docs/design/domain/engine/settings.md, 10).
#[must_use]
pub fn revision_bound(limits: &crate::Limits) -> Option<u64> {
    set_bound(limits)?
        .checked_mul(2)?
        .checked_add(List::<RevisionItem>::price(capacity(limits)?.checked_mul(2)?.checked_add(1)?)?)?
        .checked_add(u64::try_from(size_of::<Head>()).ok()?)
}

#[cfg(test)]
impl Current {
    pub fn test_charter_mut(&mut self, number: u32) -> &mut Charter {
        drop(self.resolved.remove(&number));
        match &mut self.defaults.get_mut(&Setting::Charter(number)).expect("fixture default charter").value {
            Value::Charter(value) => value,
            Value::Overlay(_)
            | Value::Template(_)
            | Value::Policy(_)
            | Value::Rules(_)
            | Value::Model(_)
            | Value::Application(_) => {
                panic!("fixture charter value")
            }
        }
    }
    pub fn test_templates(&mut self, templates: Box<[Template]>) {
        let mut old = List::with_capacity(self.defaults.capacity());
        for (&setting, _) in &self.defaults {
            match setting {
                Setting::Template(_) => old.push(setting).expect("fixture defaults"),
                Setting::Charter(_)
                | Setting::Policy(_)
                | Setting::Model(_)
                | Setting::Rules
                | Setting::ProjectPolicy { .. } => {}
                Setting::Application { .. } => panic!("fixture defaults contain only jig settings"),
            }
        }
        for setting in old.into_boxed() {
            self.defaults.remove(&setting);
        }
        for template in templates {
            self.defaults
                .insert(
                    Setting::Template(template.number),
                    Default { value: Value::Template(Box::new(template)), revision: 1 },
                )
                .expect("fixture templates");
        }
    }
}

/// The first start or a revision's largest write set, including its deployment
/// header, one steady-state history erase and a list's account record (domain/engine/settings.md, 7, 12).
#[must_use]
pub fn writes_bound(limits: &crate::Limits) -> Option<u32> {
    let count = capacity(limits)?;
    let first = count
        .checked_add(limits.people.initial_owners)?
        .checked_add(limits.authority.projects.checked_mul(2)?)?
        .checked_add(4)?;
    first
        .max(count.checked_mul(4)?.checked_add(8)?.checked_add(limits.settings.versions.checked_mul(2)?)?)
        .checked_add(withdrawal_writes(limits)?)?
        .checked_add(move_writes(limits)?)?
        .checked_add(limits.people.holdings.checked_mul(2)?.checked_add(2)?)?
        .checked_add(1)
}

/// A party withdrawal takes five writes. A task withdrawal takes two activity
/// rows, its terminal history, up to three live rows (decision, inbox, wake),
/// and the retained set's erase: seven. Settings carry no task stubs.
#[must_use]
pub fn withdrawal_writes(limits: &crate::Limits) -> Option<u32> {
    limits.settings.proposals.checked_mul(7)
}
#[must_use]
pub fn proposal_heads(items: &[Item]) -> Box<[jig_domain_tasks::SettingHead]> {
    let mut heads = List::with_capacity(u32::try_from(items.len()).expect("bounded settings items"));
    for item in items {
        heads
            .push(jig_domain_tasks::SettingHead {
                setting: match item.setting {
                    Setting::Charter(n) => jig_domain_tasks::Setting::Charter(n),
                    Setting::Template(n) => jig_domain_tasks::Setting::Template(n),
                    Setting::Policy(n) => jig_domain_tasks::Setting::Policy(n),
                    Setting::Rules => jig_domain_tasks::Setting::Rules,
                    Setting::Model(n) => jig_domain_tasks::Setting::Model(n),
                    Setting::Application { .. } | Setting::ProjectPolicy { .. } => continue,
                },
                edit: match item.edit {
                    Edit::Set(_) => jig_domain_tasks::SettingEdit::Set,
                    Edit::Removed => jig_domain_tasks::SettingEdit::Removed,
                    Edit::Reset => jig_domain_tasks::SettingEdit::Reset,
                },
            })
            .expect("bounded heads");
    }
    heads.into_boxed()
}

/// One decision copies/erases each old head, drops each version, and saves one extent.
#[must_use]
pub fn release_writes(limits: &crate::Limits) -> Option<u32> {
    limits.settings.revisions.checked_mul(2)?.checked_add(limits.settings.versions)?.checked_add(1)
}

/// Each moved live record and at most two projection rows, plus release cleanup.
#[must_use]
pub fn move_writes(limits: &crate::Limits) -> Option<u32> {
    limits.tasks.tasks.checked_mul(3)?.checked_add(release_writes(limits)?)
}

/// A project's own policy is never retained among current settings (domain/engine/settings.md, 2).
#[must_use]
pub fn is_project_policy(setting: Setting) -> bool {
    match setting {
        Setting::ProjectPolicy { .. } => true,
        Setting::Charter(_)
        | Setting::Template(_)
        | Setting::Policy(_)
        | Setting::Model(_)
        | Setting::Rules
        | Setting::Application { .. } => false,
    }
}
