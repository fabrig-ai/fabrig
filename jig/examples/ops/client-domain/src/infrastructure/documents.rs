//! Keep: scoped whole documents, bounded facts and revision-naming requests.
//! Replace: infrastructure resources, procedures and the application's own forms.
//! The owner supplies prices, end facts and drift (examples.md, 3 and 8).
use alloc::boxed::Box;
use jig_client::contract::{ApplicationPart, ObjectKey, ProposalKey, Revision};
use skein_lib::List;
/// Scope supplied with every read and watch.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Scope {
    Project(u32),
    Deployment,
}
/// Whole-document identity; proposed effects and retained entries are distinct.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Kind {
    Environment(super::values::Environment),
    Environments {
        pool: Option<super::values::Pool>,
    },
    Panel(u64),
    Result(u64),
    Effect(ProposalKey),
    /// The connector's raw proposal identity, independent of a jig maker.
    OwnerEffect(u64),
    Outbox(u64),
    Hold(u64),
    Project(u32),
    Deployment,
}
/// A subject and its actual scope.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Subject {
    pub scope: Scope,
    pub kind: Kind,
}
/// One whole owner document, admitted without partial truncation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Document {
    Value(Box<super::values::Document>),
}
impl Document {
    /// The identity echoed by this whole document.
    #[must_use]
    pub fn subject(&self) -> Subject {
        match self {
            Self::Value(document) => document.subject.clone(),
        }
    }
    /// Whole-value admission against configured bounds.
    #[must_use]
    pub fn fits(&self, limits: &jig_client::DomainLimits) -> bool {
        let Some(identity) = subject_price(&self.subject(), limits) else { return false };
        let Some(bytes) = (match self {
            Self::Value(document) => super::values::price(document, limits),
        }) else {
            return false;
        };
        let Some(bytes) = bytes.checked_add(identity) else { return false };
        match bytes.checked_add(u64::try_from(size_of::<Document>()).expect("document layout")) {
            Some(bytes) => bytes <= limits.document_bytes,
            None => false,
        }
    }
    /// These family pages request no jig objects independently.
    #[must_use]
    pub fn objects(&self, rows: u32) -> List<ObjectKey> {
        List::with_capacity(rows.checked_add(1).expect("object bound"))
    }
}
/// A keyed request naming exactly the revision the person saw.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Ask {
    MoveEnd {
        object: skein_lib::Token,
        project: u32,
        environment: super::values::Environment,
        revision: Revision,
        until: u64,
    },
}
/// A faithful bounded refusal; admission never becomes a false Gone.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refusal {
    Role { held: u32 },
    Past,
    Exhausted,
    SignedOut,
    NotAllowed,
    Conflict,
    Invalid,
}
/// A durable environment-request terminal, or current facts showing its revision moved.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Answer {
    MovedEnd { environment: super::values::Environment, revision: Revision, until: u64 },
    Moved { environment: super::values::Environment, revision: Revision, until: u64 },
    Gone,
    Refused { why: Refusal },
    EntranceRefused(jig_client::contract::EntranceRefusal),
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
pub fn subject(part: &ApplicationPart, scope: Scope) -> Option<Subject> {
    let kind = match part {
        ApplicationPart::Panel { task, .. } => Kind::Panel(*task),
        ApplicationPart::Result { task, .. } => Kind::Result(*task),
        ApplicationPart::Effect { proposal, .. } => Kind::Effect(*proposal),
        ApplicationPart::OwnerEffect { proposal, .. } => match scope {
            Scope::Project(_) => Kind::OwnerEffect(*proposal),
            Scope::Deployment => return None,
        },
        ApplicationPart::Outbox { scope, entry, .. } => {
            let scope = match scope {
                jig_client::contract::PartScope::Project(project) => Scope::Project(*project),
                jig_client::contract::PartScope::Deployment => Scope::Deployment,
            };
            return Some(Subject { scope, kind: Kind::Outbox(*entry) });
        }
        ApplicationPart::Hold { task, .. } => Kind::Hold(*task),
        ApplicationPart::Project { project, .. } => {
            return Some(Subject { scope: Scope::Project(*project), kind: Kind::Project(*project) });
        }
        ApplicationPart::Deployment { .. } => {
            return Some(Subject { scope: Scope::Deployment, kind: Kind::Deployment });
        }
        ApplicationPart::Connector { .. } | ApplicationPart::Resource { .. } | ApplicationPart::Kind { .. } => {
            return None;
        }
    };
    Some(Subject { scope, kind })
}

/// Heap bytes of the opaque owner identity, checked before retaining it.
pub(crate) fn subject_price(subject: &Subject, limits: &jig_client::DomainLimits) -> Option<u64> {
    match &subject.kind {
        Kind::Environment(environment) => {
            if environment.pool.len() > usize::try_from(limits.text).ok()?
                || environment.name.len() > usize::try_from(limits.text).ok()?
            {
                return None;
            }
            u64::try_from(environment.pool.len()).ok()?.checked_add(u64::try_from(environment.name.len()).ok()?)
        }
        Kind::Environments { pool } => match pool {
            Some(pool) => {
                if pool.0.len() > usize::try_from(limits.text).ok()? {
                    return None;
                }
                u64::try_from(pool.0.len()).ok()
            }
            None => Some(0),
        },
        Kind::Panel(_)
        | Kind::Result(_)
        | Kind::Effect(_)
        | Kind::OwnerEffect(_)
        | Kind::Outbox(_)
        | Kind::Hold(_)
        | Kind::Project(_)
        | Kind::Deployment => Some(0),
    }
}
