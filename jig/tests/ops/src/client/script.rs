//! Explicitly scoped owner fixtures, with checked whole-document admission.
use jig_client_domain::contract::Revision;
use jig_client_scripted_engine::{Application, ApplicationValue, Bounds, Change, Decision, Truth};
use jig_ops_client_domain::{infrastructure as infra, observability as obs};
use skein_lib::Wall;
use std::collections::BTreeMap;

/// The two connector subjects retain their supplied scope and owner identity.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Subject {
    Observability(obs::Subject),
    Infrastructure(infra::Subject),
}
/// Owner documents; no link or price is inferred by this adapter.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Document {
    Observability(obs::Document),
    Infrastructure(infra::Document),
}
/// Finite Ops fixtures beside jig's supplied truth.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OpsScript {
    pub documents: BTreeMap<Subject, Document>,
}
impl Application for OpsScript {
    type Ask = infra::Ask;
    type Answer = infra::Answer;
    type Subject = Subject;
    type Document = Document;
    fn decide(&mut self, _: &Truth, _: u64, ask: &infra::Ask, _: Wall) -> Decision<infra::Answer> {
        let infra::Ask::MoveEnd { project, environment, revision, until, .. } = ask;
        let subject = Subject::Infrastructure(infra::Subject {
            scope: infra::Scope::Project(*project),
            kind: infra::Kind::Environment(environment.clone()),
        });
        let answer = match self.documents.get_mut(&subject) {
            Some(Document::Infrastructure(infra::Document::Value(document))) => match &mut document.value {
                infra::values::Value::Environment { end_revision, end, fact, .. } => {
                    if *end_revision == revision.value {
                        *end_revision = end_revision.checked_add(1).expect("finite fixture revision");
                        *end = Some(infra::values::EnvironmentEnd { revision: *end_revision, until: *until });
                        infra::Answer::MovedEnd {
                            environment: environment.clone(),
                            revision: Revision { value: *end_revision },
                            until: *until,
                        }
                    } else {
                        infra::Answer::Moved {
                            environment: environment.clone(),
                            revision: Revision { value: *end_revision },
                            until: end.map_or(fact.until, |value| value.until),
                        }
                    }
                }
                infra::values::Value::Panel { .. }
                | infra::values::Value::Result { .. }
                | infra::values::Value::Effect { .. }
                | infra::values::Value::Outbox { .. }
                | infra::values::Value::Hold { .. }
                | infra::values::Value::Project { .. }
                | infra::values::Value::Deployment { .. } => infra::Answer::Gone,
            },
            Some(Document::Observability(_)) | None => infra::Answer::Gone,
        };
        Decision { answer, changes: vec![] }
    }
    fn document(&self, truth: &Truth, party: u64, subject: &Subject) -> Option<Document> {
        let project = match subject {
            Subject::Observability(obs::Subject { scope: obs::Scope::Project(project), .. })
            | Subject::Infrastructure(infra::Subject { scope: infra::Scope::Project(project), .. }) => Some(*project),
            Subject::Observability(obs::Subject { scope: obs::Scope::Deployment, .. })
            | Subject::Infrastructure(infra::Subject { scope: infra::Scope::Deployment, .. }) => None,
        };
        let frame = truth.frames.get(&party)?;
        let visible = project.map_or_else(
            || frame.deployment_role.is_some(),
            |project| frame.projects.iter().any(|row| row.project == project),
        );
        visible.then(|| self.documents.get(subject).cloned()).flatten()
    }
    fn marks(&self, _: &Change) -> Vec<Subject> {
        self.documents.keys().cloned().collect()
    }
    fn fits(&self, value: ApplicationValue<'_, Self>, bounds: &Bounds) -> bool {
        let mut limits = super::limits();
        limits.client.domain.rows = bounds.rows;
        limits.client.domain.text = bounds.text;
        limits.client.domain.document_bytes = bounds.document_bytes;
        let fits = |doc: &Document| match doc {
            Document::Observability(doc) => doc.fits(&limits.client.domain),
            Document::Infrastructure(doc) => doc.fits(&limits.client.domain),
        };
        match value {
            ApplicationValue::Document(doc) => fits(doc),
            ApplicationValue::State(state) => state.documents.iter().all(|(subject, doc)| {
                let same = match (subject, doc) {
                    (Subject::Observability(subject), Document::Observability(doc)) => *subject == doc.subject(),
                    (Subject::Infrastructure(subject), Document::Infrastructure(doc)) => *subject == doc.subject(),
                    (Subject::Observability(_), Document::Infrastructure(_))
                    | (Subject::Infrastructure(_), Document::Observability(_)) => false,
                };
                same && fits(doc)
            }),
            ApplicationValue::Ask(_) | ApplicationValue::Answer(_) | ApplicationValue::Subject(_) => true,
        }
    }
}
