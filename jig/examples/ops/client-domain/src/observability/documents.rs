//! Keep: scoped whole documents and bounded owner facts; missing facts stay unknown.
//! Replace: incident/service vocabulary and the application objects its pages name.
//! Observability owns incident history and health (examples.md, 3 and 8).
use alloc::boxed::Box;
use jig_client::contract::{ObjectKey, Part, ProposalKey};
use skein_lib::List;
/// Scope supplied with every read and watch.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Scope {
    Project(u32),
    Deployment,
}
/// Whole-document identity; an outbox entry is distinct from a proposal.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Kind {
    Incidents { open: bool },
    Incident(u64),
    Panel(u64),
    Result(u64),
    Effect(ProposalKey),
    Outbox(u64),
    Hold(u64),
    Project(u32),
    Deployment,
}
/// A subject and its actual scope.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Subject {
    pub scope: Scope,
    pub kind: Kind,
}
/// The service's names, copied from its owner.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Service {
    pub environment: Box<[u8]>,
    pub name: Box<[u8]>,
}
/// The incident's committed lifecycle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Alerting,
    Resolved,
}
/// One authoritative document for a family page or connector part.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Document {
    Value(Box<super::values::Document>),
}
impl Document {
    /// The identity echoed by this whole document.
    #[must_use]
    pub fn subject(&self) -> Subject {
        let Self::Value(document) = self;
        document.subject
    }
    /// Whole-value admission against the configured text, row and byte bounds.
    #[must_use]
    pub fn fits(&self, limits: &jig_client::DomainLimits) -> bool {
        let Self::Value(document) = self;
        let Some(bytes) = super::values::price(document, limits) else { return false };
        match bytes.checked_add(u64::try_from(size_of::<Document>()).expect("document layout")) {
            Some(bytes) => bytes <= limits.document_bytes,
            None => false,
        }
    }
    /// The incident's lead's plan, as jig's whole section (examples.md, 8).
    #[must_use]
    pub fn sections(&self) -> List<jig_client_view::page::Section> {
        let Self::Value(document) = self;
        let lead = match &document.value {
            super::values::Value::Incident { incident, .. } => incident.lead,
            super::values::Value::Service { .. }
            | super::values::Value::Alert { .. }
            | super::values::Value::Outbox { .. }
            | super::values::Value::Deployment { .. }
            | super::values::Value::Project { .. } => None,
        };
        let mut sections = List::with_capacity(1);
        if let Some(task) = lead {
            sections
                .push(jig_client_view::page::Section::Task {
                    task,
                    section: jig_client_view::address::TaskSection::Plan,
                })
                .expect("one lead section fits");
        }
        sections
    }
    /// jig objects supplied by the incident owner, never guessed from its watch.
    #[must_use]
    pub fn objects(&self, rows: u32) -> List<ObjectKey> {
        let mut objects = List::with_capacity(rows.checked_add(1).expect("incident object bound"));
        let Self::Value(document) = self;
        match &document.value {
            super::values::Value::Incident { incident, .. } => {
                if let Some(lead) = incident.lead {
                    objects.push(ObjectKey::Task(lead)).expect("lead capacity");
                }
            }
            super::values::Value::Service { .. }
            | super::values::Value::Alert { .. }
            | super::values::Value::Outbox { .. }
            | super::values::Value::Deployment { .. } => {}
            super::values::Value::Project { incidents, .. } => {
                for incident in incidents {
                    if let Some(lead) = incident.lead {
                        objects.push(ObjectKey::Task(lead)).expect("incident lead capacity");
                    }
                }
            }
        }
        objects
    }
}
/// A scoped whole-read terminal.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReadResult {
    NotAllowed,
    Value(Box<Document>),
    Unknown,
    Busy,
    SignedOut,
}
/// A level-triggered watch callback.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WatchEvent {
    Delivered(Box<Document>),
}
/// One terminal for every opened watch.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WatchEnd {
    NotAllowed,
    SubjectEnded,
    Closed,
    Unknown,
    SignedOut,
}
/// The subject for a part; resources and kinds need words only.
#[must_use]
pub fn subject(part: &Part, scope: Scope) -> Option<Subject> {
    let kind = match part {
        Part::Panel { task, .. } => Kind::Panel(*task),
        Part::Result { task, .. } => Kind::Result(*task),
        Part::Effect { proposal, .. } => Kind::Effect(*proposal),
        Part::Outbox { scope, entry, .. } => {
            let scope = match scope {
                jig_client::contract::PartScope::Project(project) => Scope::Project(*project),
                jig_client::contract::PartScope::Deployment => Scope::Deployment,
            };
            return Some(Subject { scope, kind: Kind::Outbox(*entry) });
        }
        Part::Hold { task, .. } => Kind::Hold(*task),
        Part::Project { project, .. } => {
            return Some(Subject { scope: Scope::Project(*project), kind: Kind::Project(*project) });
        }
        Part::Deployment { .. } => return Some(Subject { scope: Scope::Deployment, kind: Kind::Deployment }),
        // Connector proposal numbers do not name this side's typed maker effects.
        Part::OwnerEffect { .. } | Part::Connector { .. } | Part::Resource { .. } | Part::Kind { .. } => return None,
    };
    Some(Subject { scope, kind })
}
