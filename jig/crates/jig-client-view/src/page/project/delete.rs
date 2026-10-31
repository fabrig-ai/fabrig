//! Name-confirmed project deletion; temper/docs/design/draft/playground-reset.md, 5.
//! The confirmation stays frozen while pending, including after the page leaves.
use crate::{Request, Ui, intents::Origin, page::IntentPurpose};
use alloc::boxed::Box;
use jig_client::contract::{
    ActKind, Admission, Answer, Ask, Limit, Outcome, OwnerOutcome, OwnerProject, Refusal, Text,
};
use skein_lib::{Env, Queue, Token};

/// One exact-name confirmation and its pending request or refusal.
#[derive(Debug)]
pub struct Delete {
    pub typed: Box<str>,
    pub pending: Option<Token>,
    pub refused: Option<Refusal>,
}
impl Delete {
    pub(crate) fn new() -> Self {
        Self { typed: Box::from(""), pending: None, refused: None }
    }
}
/// Only an explicitly allowed deletion is shown.
#[must_use]
pub fn offered(doc: &OwnerProject) -> bool {
    for offer in &doc.acts {
        if offer.act == ActKind::RemoveProject {
            return match &offer.admission {
                Admission::Allowed => true,
                Admission::Proposed { .. } | Admission::Refused(_) | Admission::Bounded { .. } => false,
            };
        }
    }
    false
}
/// A partial document name cannot confirm deletion of the whole project.
#[must_use]
pub fn ready(doc: &OwnerProject, state: &Delete) -> bool {
    offered(doc) && !doc.name.cut && state.pending.is_none() && state.typed.as_bytes() == doc.name.bytes()
}
pub(crate) fn edit(domain: &mut Ui, project: u32, text: Text) {
    if domain.blocked || domain.project_page.delete.pending.is_some() {
        return;
    }
    let Some(doc) = &domain.project_page.owner else { return };
    if doc.number != project || !offered(doc) || !text.fits(domain.text_limit) {
        return;
    }
    #[expect(
        clippy::disallowed_methods,
        reason = "the name confirmation stores already validated Text as Box<str> unchanged"
    )]
    let typed = Box::from(core::str::from_utf8(text.bytes()).expect("Text is UTF-8"));
    domain.project_page.delete.typed = typed;
    domain.project_page.delete.refused = None;
    domain.changed();
}
pub(crate) fn send(domain: &mut Ui, project: u32, out: &mut Queue<Request>) {
    if domain.blocked {
        return;
    }
    let Some(scope) = domain.confirmed else { return };
    let Some(doc) = &domain.project_page.owner else { return };
    if doc.number != project || !ready(doc, &domain.project_page.delete) {
        return;
    }
    match domain.intents.ask(
        scope,
        Origin::Page { handle: domain.page_handle(), purpose: IntentPurpose::RemoveProject },
        Box::new(Ask::RemoveProject { project: doc.number }),
        out,
    ) {
        Some(id) => {
            domain.project_page.delete.pending = Some(id.token());
            domain.project_page.delete.refused = None;
        }
        None => domain.project_page.delete.refused = Some(Refusal::Limit { which: Limit::Requests }),
    }
    domain.changed();
}
pub(crate) fn answered(
    domain: &mut Ui,
    env: &Env<crate::Limits>,
    request: Token,
    receipt: crate::intents::Receipt,
    out: &mut Queue<Request>,
    lent: &jig_client::Client,
) {
    if domain.project_page.delete.pending != Some(request) {
        return;
    }
    domain.project_page.delete.pending = None;
    if domain.confirmed != Some(receipt.scope) {
        domain.project_page.delete = Delete::new();
        domain.changed();
        return;
    }
    let Ask::RemoveProject { project: requested } = *receipt.ask else { return };
    let same_page = domain.shown_project() == Some(requested);
    match receipt.answer {
        Answer::Done(Outcome::Owner(OwnerOutcome::ProjectRemoved { project })) => {
            let name = core::mem::replace(&mut domain.project_page.delete.typed, Box::from(""));
            if same_page && project == requested {
                crate::page::routing::navigate(domain, env, domain.navigation.home.place(), false, out, lent);
            }
            crate::notices::kept::show(domain, crate::notices::kept::Kind::ProjectDeleted { name });
        }
        Answer::Refused { refusal, .. } => {
            if same_page && domain.project_page.owner.is_some() {
                domain.project_page.delete.refused = Some(refusal);
            } else {
                crate::notices::kept::show(
                    domain,
                    crate::notices::kept::Kind::PageAnswer { answer: Answer::Refused { refusal, at: None } },
                );
            }
        }
        Answer::Done(outcome) => {
            crate::notices::kept::show(
                domain,
                crate::notices::kept::Kind::PageAnswer { answer: Answer::Done(outcome) },
            );
        }
    }
    domain.changed();
}
pub(crate) fn unknown(domain: &mut Ui, request: Token) {
    if domain.project_page.delete.pending == Some(request) {
        domain.project_page.delete.pending = None;
    }
}
