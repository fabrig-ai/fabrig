//! Project closure and retained deletion progress (jig/docs/design/domain/people/acts.md, 2).
use super::Domain;
use crate::{Act, Ask, DeletionStep, KeptDeletion, Key, Outcome, OutcomeFamily, Request, Stored};
use skein_lib::{Duration, List, Queue};

pub(super) fn keep(
    domain: &mut Domain,
    project: u32,
    step: Option<DeletionStep>,
    grace: Option<Duration>,
    out: &mut Queue<Request>,
) {
    match step {
        Some(step) => {
            // Advancing a deletion never revives a mark evicted by a newer one.
            if !domain.deletions.contains_key(&project) {
                mark(domain, project, out);
            }
            domain.deletions.insert(project, KeptDeletion { step, grace }).expect("deletions fit the project room");
            out.push(Request::Save { record: Stored::Deletion { project, step, grace } });
        }
        None => {
            domain.deletions.remove(&project);
            forget(domain, project, out);
            out.push(Request::Erase { key: Key::Deletion(project) });
        }
    }
}

fn mark(domain: &mut Domain, project: u32, out: &mut Queue<Request>) {
    if domain.deleted.contains(&project) {
        return;
    }
    if domain.deleted.len() == domain.deleted.capacity() {
        let lowest = *domain.deleted.first().expect("a deletion has project room");
        domain.deleted.remove(&lowest);
        out.push(Request::Erase { key: Key::Deleted(lowest) });
    }
    domain.deleted.insert(project).expect("marks fit the project room");
    out.push(Request::Save { record: Stored::Deleted { project } });
}

pub(super) fn close(domain: &mut Domain, project: u32, out: &mut Queue<Request>) {
    domain.roles.remove(&project);
    super::prune_unheld_roles(domain, project);
    out.push(Request::Erase { key: Key::Roles(project) });
}

pub(super) fn forget(domain: &mut Domain, project: u32, out: &mut Queue<Request>) {
    forget_census(domain, project);
    let mut answers = List::with_capacity(domain.answers.len());
    for (key, answer) in &domain.answers {
        if let (Ask::RemoveProject { project, .. }, Outcome::ProjectRemoved { project: removed }) =
            (&answer.ask, &answer.outcome)
            && project == removed
        {
            continue;
        }
        if kept_answer(domain, &answer.outcome) {
            continue;
        }
        if ask_project(&answer.ask) == Some(project) || made_project(&answer.outcome) == Some(project) {
            answers.push(*key).expect("bounded retained answers");
        }
    }
    for key in answers.into_boxed() {
        domain.answers.remove(&key);
        domain.answer_alarms.cancel(key);
        out.push(Request::Erase { key: Key::Answer(key) });
    }
    let mut acts = List::with_capacity(domain.acted.len());
    for (key, answer) in &domain.acted {
        if kept_answer(domain, answer.outcome()) {
            continue;
        }
        if answer.project() == Some(project) {
            acts.push(*key).expect("bounded retained acts");
        }
    }
    for key in acts.into_boxed() {
        domain.acted.remove(&key);
        domain.act_alarms.cancel(key);
        out.push(Request::Erase { key: Key::Acted(key) });
    }
}

fn forget_census(domain: &mut Domain, project: u32) {
    let mut parties = List::with_capacity(domain.chat_updates.len());
    for (&(party, scope), _) in &domain.chat_updates {
        if scope == project {
            parties.push(party).expect("project reply census subset");
        }
    }
    for party in parties.into_boxed() {
        domain.chat_updates.remove(&(party, project));
    }
    let mut parties = List::with_capacity(domain.result_census.len());
    for (&(party, scope), _) in &domain.result_census {
        if scope == project {
            parties.push(party).expect("project result census subset");
        }
    }
    for party in parties.into_boxed() {
        drop(domain.result_census.remove(&(party, project)));
    }
}

/// Active removals survive row cleanup; party removals also keep ordinary answer retention
/// (jig/docs/design/domain/people/acts.md, 2).
pub(super) fn kept_answer(domain: &Domain, outcome: &Outcome) -> bool {
    for (&project, _) in &domain.deletions {
        if *outcome == (Outcome::ProjectRemoved { project }) {
            return true;
        }
    }
    false
}

pub(super) fn made_project(outcome: &Outcome) -> Option<u32> {
    match outcome.family() {
        OutcomeFamily::ProjectMade(project) => Some(project),
        OutcomeFamily::Again(_)
        | OutcomeFamily::SignedOut
        | OutcomeFamily::Compound
        | OutcomeFamily::Refused
        | OutcomeFamily::CompoundRefused
        | OutcomeFamily::ServiceMade(_)
        | OutcomeFamily::RolesSet(_)
        | OutcomeFamily::Settled => None,
    }
}

pub(super) fn act_project(act: &Act) -> Option<u32> {
    match act {
        Act::Close { project, .. } | Act::Adopt { project, .. } | Act::RemoveProject { project, .. } => Some(*project),
        Act::MakeProject { .. } | Act::AddKey { .. } | Act::RemoveAccount { .. } => None,
    }
}

pub(super) fn ask_project(ask: &Ask) -> Option<u32> {
    ask.family().named_project()
}
