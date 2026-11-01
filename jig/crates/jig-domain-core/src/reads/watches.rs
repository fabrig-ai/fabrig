//! Finished task and tree watches reuse the bounded durable task load (contract.md, 7.2).
use crate::{
    Core, Held, Limits, Request,
    documents::{self, Document, Missing, Subject},
    reads::{Answer, Destination, EndedValue, Loaded, Pending, Range, Row, Scope, rows},
};
use alloc::boxed::Box;
use jig_domain_views as views;
use skein_lib::{Env, Id, Queue, Token};

pub(crate) fn start_watch(core: &mut Core, env: &Env<Limits>, watcher: Token) -> Queue<Request> {
    let task = crate::routing::view::routes::retained_task(core, watcher).expect("task construction");
    let binding = core.watch_binding(watcher).expect("bound construction");
    let range = Range {
        party: binding.party,
        scope: Scope::Task { task },
        after: None,
        backwards: false,
        rows: env.limits.documents.rows,
        bytes: env.limits.load_bytes,
    };
    let pending = Pending {
        destination: Destination::Watch(watcher),
        sign_in: binding.sign_in,
        party: binding.party,
        range: range.clone(),
        tasks: None,
    };
    let Ok(load) = core.reads.insert(pending) else {
        return crate::routing::view::routes::retained_made(core, env, watcher, Err(views::End::Gone));
    };
    let mut out = Queue::uncapped();
    out.push(Request::Held(Box::new(Held::ReadLoad { load: load.token(), range: Box::new(range) })));
    out
}

pub(super) fn watch_loaded(
    core: &mut Core,
    env: &Env<Limits>,
    load: Id<Pending>,
    watcher: Token,
    result: Loaded,
) -> Queue<Request> {
    let pending = core.reads.get_mut(load).expect("pending watch load");
    pending.destination = Destination::Done;
    let sign_in = pending.sign_in;
    let party = pending.party;
    let range = pending.range.clone();
    core.reads.retire(load);
    let document = if core.people.person(sign_in, env.now, env.wall) == Some(party) {
        from_store(core, env, watcher, &range, result)
    } else {
        Err(views::End::SignedOut)
    };
    crate::routing::view::routes::retained_made(core, env, watcher, document)
}

fn from_store(
    core: &Core,
    env: &Env<Limits>,
    watcher: Token,
    range: &Range,
    result: Loaded,
) -> Result<Document, views::End> {
    let (raw, next) = match result {
        Loaded::Page { rows, next } => (rows, next),
        Loaded::TooLinked { .. } | Loaded::Busy | Loaded::Gone | Loaded::TurnProblem { .. } => {
            return Err(views::End::Gone);
        }
    };
    let Some(binding) = core.watch_binding(watcher) else { return Err(views::End::Gone) };
    let subject = &core.view_names.get(binding.subject).expect("bound task subject").subject;
    // Validate the same row, byte, identity and visibility bounds as an ordinary task read.
    match subject {
        Subject::Tree { .. } => return tree(core, &env.limits, range, &raw, next.is_some()),
        Subject::Task { .. } => {}
        Subject::Frame { .. }
        | Subject::Chips { .. }
        | Subject::Inbox { .. }
        | Subject::Accounts { .. }
        | Subject::Query { .. }
        | Subject::Run { .. }
        | Subject::Settings
        | Subject::Deployment
        | Subject::Proposal { .. }
        | Subject::Escalation { .. }
        | Subject::Question { .. }
        | Subject::PersonTask { .. }
        | Subject::Result { .. }
        | Subject::Project { .. }
        | Subject::Note { .. }
        | Subject::Application { .. } => return Err(views::End::Gone),
    }
    match rows::from_store(core, &env.limits, range, raw, next, env.wall) {
        Answer::Ended(value) => {
            let document = match value.value {
                EndedValue::Task { document, .. } => document,
                EndedValue::PersonProposal { .. }
                | EndedValue::Proposal(_)
                | EndedValue::Escalation { .. }
                | EndedValue::Question { .. }
                | EndedValue::PersonTask { .. }
                | EndedValue::Turn(_)
                | EndedValue::Reply(_) => return Err(views::End::Gone),
            };
            Ok(Document::Task(document))
        }
        Answer::Setting(_)
        | Answer::Revision(_)
        | Answer::Revisions(_)
        | Answer::TaskText(_)
        | Answer::Tasks(_)
        | Answer::Document(_)
        | Answer::Page(_)
        | Answer::None(_) => Err(views::End::Gone),
    }
}

// A point tree needs only its retained root, never the task page's whole-value bounds.
fn tree(core: &Core, limits: &Limits, range: &Range, raw: &[Row], more: bool) -> Result<Document, views::End> {
    if raw.len() != 1 || range.rows == 0 || more {
        return Err(views::End::Gone);
    }
    let row = raw.first().expect("one retained root");
    let Some(bytes) = row.owned_bytes() else { return Err(views::End::Gone) };
    if bytes > range.bytes || !super::admits(range.party, &range.scope, &row.value.key()) {
        return Err(views::End::Gone);
    }
    let Some(owner) = super::vocabulary::row_owner(row) else { return Err(views::End::Gone) };
    match documents::task_tree::ended_tree(core, limits, range.party, owner) {
        Ok(tree) => Ok(Document::Tree(tree)),
        Err(Missing::NotAllowed | Missing::Unknown | Missing::Ended | Missing::TooLarge) => Err(views::End::Gone),
    }
}

/// A watch's authenticated owner and concrete construction inputs.
/// `client/domain/contract.md`, sections 7.1 and 7.2.
#[derive(Clone, PartialEq, Eq, Debug)]
pub(crate) struct WatchBinding {
    /// The party whose document is constructed.
    pub(crate) party: u64,
    /// The sign-in that admitted this watch.
    pub(crate) sign_in: u64,
    /// The core-owned subject.
    pub(crate) subject: Id<WatchedSubject>,
    /// The requested bounded window.
    pub(crate) window: documents::Window,
}

/// A concrete subject shared by watches, with its opaque child name.
#[derive(Debug)]
pub(crate) struct WatchedSubject {
    pub(crate) subject: Subject,
    pub(crate) watchers: u32,
}
