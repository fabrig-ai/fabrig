//! Spine routing of page generations, watches, reads, names and parts.
//! Detailed page policies belong to their page modules. See client/domain/client.md, 5–7.
use crate::{
    Request, Ui,
    state::{
        address::{Address, Place},
        handles::Holder,
        objects::{Body, Source},
        page::{Page, Section},
    },
};
use alloc::boxed::Box;
use jig_client::contract;
use jig_client::contract::{Delivery, Limit, ObjectKey, Part, PartyKind, Read, ReadResult, ReadValue, Subject};
use jig_client::thread::window::{Edge, Edges, End};
use skein_lib::{Env, List, Queue, Token};

fn blank_edges() -> Edges {
    Edges { before: Edge::End, after: Edge::End }
}
pub(crate) fn page_edges(before: Option<contract::Cursor>, after: Option<contract::Cursor>) -> Edges {
    let before = match before {
        Some(cursor) => Edge::More(cursor),
        None => Edge::End,
    };
    let after = match after {
        Some(cursor) => Edge::More(cursor),
        None => Edge::End,
    };
    Edges { before, after }
}
pub(crate) fn refuse(domain: &mut Ui, which: Limit) {
    match &mut domain.page {
        Page::Work { refused, .. } => *refused = Some(which),
        Page::Elsewhere if !domain.sections.is_empty() && domain.section_context != 0 => {
            super::shown::status(domain, Some(which), None, false);
        }
        Page::Starting | Page::Elsewhere | Page::SigningIn | Page::Missing | Page::OutOfDate | Page::SomeoneElse => {
            crate::tree::notices::kept::show(domain, crate::tree::notices::kept::Kind::Refused(which));
        }
    }
    domain.changed();
}
pub(crate) fn excess(domain: &mut Ui, count: u32) {
    match &mut domain.page {
        Page::Work { excess, .. } => *excess = count,
        Page::Elsewhere => super::shown::status(domain, None, Some(count), false),
        Page::Starting | Page::SigningIn | Page::Missing | Page::OutOfDate | Page::SomeoneElse => {}
    }
    domain.changed();
}
pub(crate) fn open_once(domain: &mut Ui, subject: Subject, owner: Holder, out: &mut Queue<Request>) -> bool {
    for (_, watch) in &domain.handles.watches {
        if watch.owner == owner && watch.subject == subject {
            return false;
        }
    }
    if domain.hold_watch(subject, owner, out).is_none() {
        refuse(domain, Limit::Watches);
    }
    true
}
pub(crate) fn read(domain: &mut Ui, number: u16, what: Read, out: &mut Queue<Request>) {
    crate::state::page::shown::context_read(domain, &what);
    if domain.handles.pending(&what) {
        return;
    }
    let source = Source::Window { number };
    match domain.handles.read(source, what, out) {
        Some(_) => {}
        None => refuse(domain, Limit::Reads),
    }
}
/// Leaves in the order the contract requires; shell places replace their current entry.
pub(crate) fn navigate(
    domain: &mut Ui,
    env: &Env<crate::Limits>,
    place: Place,
    replace: bool,
    out: &mut Queue<Request>,
    lent: &jig_client::Client,
) {
    if domain.blocked {
        return;
    }
    leave_page(domain, out);
    match place {
        Place::Jig(address) => {
            domain.signin.application = false;
            domain.address = address;
            match domain.frame(lent).document {
                Some(_) => enter(domain, env, out, lent),
                None => {
                    domain.page = Page::Starting;
                    domain.changed();
                }
            }
            out.push(Request::Show {
                place: Place::Jig(address),
                how: if replace { crate::How::Replace } else { crate::How::Push },
            });
            crate::state::store::place(domain, out);
        }
        Place::Elsewhere => {
            domain.signin.application = true;
            domain.page = Page::Elsewhere;
            domain.changed();
        }
        Place::Missing => {
            domain.page = Page::Missing;
            domain.changed();
        }
    }
}
/// Releases a page's handles and owned read pages together (view.md, 18.2).
pub(crate) fn leave_page(domain: &mut Ui, out: &mut Queue<Request>) {
    domain.leave_handles(false, out);
    domain.handles.leave_reads(out);
    out.push(Request::PartsLeft);
    domain.sections.clear();
    domain.objects.clear();
    crate::state::confirm::proposal::forget(domain);
    domain.task_page.clear();
    domain.project_page.clear();
    domain.deployment_page.owner = None;
    domain.clock.page_times = false;
    domain.inbox.clear();
    domain.inbox.opening_updates = false;
    domain.object_rows.rows.clear();
    domain.object_rows.edges = blank_edges();
    domain.conversation_rows.rows.clear();
    domain.conversation_rows.cut = false;
    domain.conversation_rows.edges = blank_edges();
    let rows = domain.conversation.lines.rows.capacity();
    let words = domain.conversation.words.rows.capacity();
    domain.conversation = jig_client::thread::Thread::new(
        0,
        &jig_client::thread::Limits { lines: rows, bytes: domain.limits.document_bytes, text: domain.text_limit },
    );
    domain.conversation.words = jig_client::thread::KeptWords::new(words);
    domain.history_rows.rows.clear();
    domain.history_rows.cut = false;
    domain.history_rows.edges = blank_edges();
}
/// Opens only what the selected work page shows; services have no page.
pub(crate) fn enter(domain: &mut Ui, env: &Env<crate::Limits>, out: &mut Queue<Request>, lent: &jig_client::Client) {
    domain.sections.clear();
    let hidden = match domain.confirmed {
        Some(party) => crate::state::ui::foreign_kept_requests(domain, party),
        None => true,
    };
    if hidden {
        domain.page = Page::SomeoneElse;
        domain.changed();
        return;
    }
    if let Some(frame) = &domain.frame(lent).document {
        match frame.party.kind {
            PartyKind::Service => {
                domain.page = Page::Elsewhere;
                domain.changed();
                return;
            }
            PartyKind::Person => {}
        }
    }
    if domain.address == Address::Deployment {
        let allowed = match &domain.frame(lent).document {
            Some(frame) => frame.deployment_role.is_some(),
            None => false,
        };
        if !allowed {
            domain.page = Page::Missing;
            domain.changed();
            return;
        }
    }
    domain.page = Page::Work { address: domain.address, behind: false, excess: 0, refused: None };
    let section = match domain.address {
        Address::Inbox { project } => {
            domain
                .sections
                .push(Section::Inbox { group: contract::InboxReadGroup::Needs, project })
                .expect("page sections fit their fixed bound");
            Section::Inbox { group: contract::InboxReadGroup::Updates, project }
        }
        Address::Conversations { project } => Section::Conversations { project },
        Address::Task { task, section } => Section::Task { task, section },
        Address::Project { project, section } => Section::Project { project, section },
        Address::Deployment => Section::Deployment,
    };
    domain.sections.push(section).expect("page sections fit their fixed bound");
    super::shown::arrange(domain);
    domain.changed();
    let owner = Holder::Page;
    for index in 0..domain.sections().len() {
        let section = *domain.sections().get(index).expect("section index is in the page");
        open_section(domain, env, section, owner, out, lent);
    }
}
/// Opens the sources one section needs (client/domain/client.md, 5.1).
pub(crate) fn open_section(
    domain: &mut Ui,
    env: &Env<crate::Limits>,
    section: Section,
    owner: Holder,
    out: &mut Queue<Request>,
    lent: &jig_client::Client,
) {
    super::opening::section(domain, env, section, owner, out, lent);
}
pub(crate) fn document_read(key: ObjectKey) -> Read {
    match key {
        ObjectKey::Task(task) => Read::Task { task },
        ObjectKey::Proposal(proposal) => Read::Proposal { proposal },
        ObjectKey::Escalation(task) => Read::Escalation { task },
        ObjectKey::Question { task, message } => Read::Question { task, message },
        ObjectKey::PersonTask(task) => Read::PersonTask { task },
        ObjectKey::Result(task) => Read::Result { task },
        ObjectKey::Note { project, .. } => {
            Read::Notes { scope: contract::NoteScope::Project(project), description: None, after: None }
        }
    }
}
pub(crate) fn names(
    domain: &mut Ui,
    env: &Env<crate::Limits>,
    source: Source,
    keys: List<ObjectKey>,
    out: &mut Queue<Request>,
    lent: &jig_client::Client,
) {
    let missed = domain.objects.replace(source, keys.as_slice());
    excess(domain, missed);
    domain.object_rows.rows.clear();
    for key in &keys {
        if domain.objects.named(*key, source) {
            match domain.object_rows.rows.push(*key) {
                Ok(()) | Err(_) => {}
            }
            let id = domain.objects.find(*key).expect("named object is present");
            let object = domain.borrow(lent).objects().get(id).expect("indexed object exists");
            let needs_read = match object.body {
                None | Some(Body::Chip(_)) => true,
                Some(
                    Body::Task(_)
                    | Body::Historical(_)
                    | Body::OwnerProposal(_)
                    | Body::OwnerEscalation(_)
                    | Body::Question(_)
                    | Body::OwnerPersonTask(_)
                    | Body::Result(_)
                    | Body::Note { .. },
                ) => false,
            };
            if needs_read {
                let what = document_read(*key);
                if !domain.handles.pending(&what) {
                    match domain.handles.read(source, what, out) {
                        Some(_) => {}
                        None => refuse(domain, Limit::Reads),
                    }
                }
            }
        }
    }
    if missed > 0 {
        refuse(domain, Limit::Objects);
    }
    // The page's names are bounded even when the incoming document has more rows.
    assert!(domain.object_rows.rows.len() <= env.limits.rows, "page rows fit their window");
}
pub(crate) fn parts(
    domain: &mut Ui,
    env: &Env<crate::Limits>,
    values: &[Part],
    out: &mut Queue<Request>,
    lent: &jig_client::Client,
) {
    let mut count = 0_u32;
    for part in values {
        if count == env.limits.parts {
            refuse(domain, Limit::Objects);
            break;
        }
        let scope = crate::state::application::scope(domain, part, lent);
        if let Some(scope) = scope {
            out.push(Request::ShowPart { part: part.clone(), scope });
        } else {
            refuse(domain, Limit::Reads);
        }
        count = count.saturating_add(1);
    }
}

fn one(
    domain: &mut Ui,
    env: &Env<crate::Limits>,
    source: Source,
    key: ObjectKey,
    body: Body,
    out: &mut Queue<Request>,
    lent: &jig_client::Client,
) {
    let missed = domain.objects.replace(source, &[key]);
    if missed > 0 {
        refuse(domain, Limit::Objects);
        return;
    }
    match domain.objects.find(key) {
        Some(_) => {
            let stored = crate::state::application::put(domain, env, key, body, out, lent, false);
            assert!(stored, "named object receives its facts");
        }
        None => refuse(domain, Limit::Objects),
    }
    domain.changed();
}
/// Applies one whole delivery to its source, never to an old generation.
pub(crate) fn delivery(
    domain: &mut Ui,
    env: &Env<crate::Limits>,
    watch: jig_client::Handle,
    value: Delivery,
    out: &mut Queue<Request>,
    lent: &jig_client::Client,
) {
    let source = Source::Watch(watch);
    match value {
        Delivery::Me(_) | Delivery::OwnerProject(_) | Delivery::OwnerAccounts(_) | Delivery::Settings(_) => {}
        Delivery::RunState(_) => {
            crate::state::activity::routing::text(domain, env, watch, out, lent);
        }
        Delivery::OwnerProposal(proposal) => {
            let key = ObjectKey::Proposal(proposal.key);
            one(domain, env, source, key, Body::OwnerProposal(proposal), out, lent);
        }
        Delivery::Task(task) => {
            crate::state::activity::routing::document(domain, &task, out, lent);
            crate::state::page::task::delivered(domain, source, task);
            crate::state::page::task::document(domain, env, out, lent);
        }
        Delivery::OwnerTree(tree) => owner_tree_delivery(domain, env, watch, source, tree, out, lent),
        Delivery::Chips(chips) => {
            let mut keys = List::with_capacity(env.limits.rows);
            for chip in &chips {
                match keys.push(ObjectKey::Task(chip.task)) {
                    Ok(()) => {}
                    Err(_) => break,
                }
            }
            names(domain, env, source, keys, out, lent);
            domain.changed();
        }
        Delivery::OwnerInbox(_) => {
            let projected = domain.borrow(lent).inbox();
            let timed = !projected.rows.is_empty();
            domain.inbox = projected.into_state();
            crate::state::page::replies::inbox(domain, env, source, out, lent);
            crate::state::page::conversations::mark_results(domain, out, lent);
            domain.timed_page(timed);
            domain.changed();
        }
        Delivery::Tasks(tasks) => {
            if let Source::Watch(handle) = source
                && let Some(watch) = domain.handles.watches.get(&handle)
                && let Subject::Tasks { query, .. } = &watch.subject
                && domain.shown_conversations() == Some(query.project)
                && let Some(me) = domain.frame(lent).document
                && crate::state::queries::chats(me, query.project) == Some(*query)
            {
                super::conversations::delivered(domain, env, source, *tasks, out, lent);
            } else {
                crate::state::page::project::task_rows(domain, env, source, *tasks, out, lent);
            }
        }
        Delivery::OwnerDeployment(_) => {
            crate::state::page::deployment::watched(domain, env, out, lent);
        }
    }
}
fn read_body(domain: &mut Ui, key: ObjectKey, body: Body) {
    // Reading does not name objects: a source may have stopped naming this
    // object while its read was in flight in the same generation.
    if domain.objects.put(key, body) {
        domain.changed();
    }
}
fn owner_proposal_read(
    domain: &mut Ui,
    env: &Env<crate::Limits>,
    proposal: Box<contract::OwnerProposal>,
    out: &mut Queue<Request>,
    lent: &jig_client::Client,
) {
    let key = ObjectKey::Proposal(proposal.key);
    if crate::state::application::put(domain, env, key, Body::OwnerProposal(proposal), out, lent, true) {
        domain.changed();
    }
}
/// Consumes one read terminal and drops old-generation values before applying them.
#[expect(clippy::too_many_lines, reason = "the closed read terminal table routes every public read explicitly")]
pub(crate) fn read_result(
    domain: &mut Ui,
    env: &Env<crate::Limits>,
    token: Token,
    result: ReadResult,
    out: &mut Queue<Request>,
    lent: &jig_client::Client,
) {
    if !original_read_valid(domain, env, token, &result, lent) {
        rejected_read(domain, token);
        refuse(domain, Limit::Reads);
        return;
    }
    let Some(receipt) = domain.handles.received(crate::state::handles::handle(token)) else {
        return;
    };
    super::shown::context_read(domain, &receipt.what);
    match receipt.source {
        Source::Window { number: crate::state::application::WINDOW, .. } => {
            crate::state::application::read_result(domain, env, receipt.what, result, out, lent);
            return;
        }
        Source::Window { .. } | Source::Watch(_) | Source::Card(_) => {}
    }
    let inbox_read = match &receipt.what {
        Read::History { subtree, .. } => {
            if *subtree != domain.task_page.subtree_history {
                return;
            }
            false
        }
        Read::Ended { task, .. } => {
            if Some(*task) != domain.task_page.ended_task {
                return;
            }
            false
        }
        Read::Notes { scope, description, .. } => {
            let valid = match &domain.project_page.note_filter {
                Some(filter) => filter.scope == *scope && filter.description == *description,
                None => false,
            };
            if !valid {
                return;
            }
            false
        }
        Read::Tasks { .. }
        | Read::Me
        | Read::Task { .. }
        | Read::Proposal { .. }
        | Read::Escalation { .. }
        | Read::Question { .. }
        | Read::PersonTask { .. }
        | Read::Result { .. }
        | Read::Activity { .. }
        | Read::Turn { .. }
        | Read::Spend { .. }
        | Read::Project { .. }
        | Read::Settings
        | Read::Setting(_)
        | Read::Revision(_)
        | Read::Revisions(_)
        | Read::Deployment => false,
        Read::Inbox { .. } => true,
    };
    match receipt.source {
        Source::Window { number, .. } if inbox_read => {
            let Some(group) = super::conversations::inbox_group(number) else {
                return;
            };
            let window = domain.inbox.window_mut(group);
            if window.reading != Some(token) {
                return;
            }
            window.reading = None;
        }
        Source::Window { .. } | Source::Watch(_) | Source::Card(_) => {}
    }
    match result {
        ReadResult::None(why) => {
            if domain.page == Page::Elsewhere {
                super::shown::status(domain, None, None, true);
            }
            match receipt.source {
                Source::Window { number: crate::state::activity::routing::ACTIVITY_READ, .. } => {
                    domain.conversation.failed();
                }
                Source::Window { .. } | Source::Watch(_) | Source::Card(_) => {}
            }
            match why {
                contract::ReadNone::TooLinked { joins } => {
                    crate::tree::notices::kept::show(domain, crate::tree::notices::kept::Kind::TooLinked { joins });
                }
                contract::ReadNone::TooLarge { .. }
                | contract::ReadNone::Malformed
                | contract::ReadNone::Unknown
                | contract::ReadNone::Gone
                | contract::ReadNone::Refused => {
                    if domain.page != Page::Elsewhere {
                        domain.page = Page::Missing;
                    }
                    domain.changed();
                }
                contract::ReadNone::Busy | contract::ReadNone::Unreachable => {
                    refuse(domain, Limit::Reads);
                }
                contract::ReadNone::SignedOut => {}
            }
        }
        ReadResult::Value(value) => received_value(domain, env, receipt.source, receipt.what, value, out, lent),
    }
}
fn original_read_valid(
    domain: &Ui,
    env: &Env<crate::Limits>,
    token: Token,
    result: &ReadResult,
    lent: &jig_client::Client,
) -> bool {
    owner_escalation_read_valid(domain, token, result, lent)
        && owner_project_read_valid(domain, token, result)
        && owner_inbox_read_valid(domain, env, token, result)
        && historical_read_valid(domain, env, token, result)
}

fn read_value(
    domain: &mut Ui,
    env: &Env<crate::Limits>,
    source: Source,
    value: ReadValue,
    out: &mut Queue<Request>,
    lent: &jig_client::Client,
) {
    match value {
        ReadValue::Me(_)
        | ReadValue::OwnerInbox(_)
        | ReadValue::OwnerEnded(_)
        | ReadValue::OwnerSpend(_)
        | ReadValue::Settings(_)
        | ReadValue::Setting(_)
        | ReadValue::Revision(_)
        | ReadValue::Revisions(_) => {}
        ReadValue::OwnerProject(value) => crate::state::page::project::owner_document(domain, value),
        ReadValue::Notes(page) => crate::state::page::project::notes::page(domain, source, *page),
        ReadValue::Task(task) => task_read(domain, env, task, out, lent),
        ReadValue::Historical(value) => read_body(domain, value.key, Body::Historical(value)),
        ReadValue::OwnerProposal(proposal) => owner_proposal_read(domain, env, proposal, out, lent),
        ReadValue::OwnerEscalation(value) => owner_escalation_read(domain, value),
        ReadValue::Question(question) => {
            let key = ObjectKey::Question { task: question.task.task, message: question.message };
            read_body(domain, key, Body::Question(question));
        }
        ReadValue::OwnerPersonTask(task) => {
            let key = ObjectKey::PersonTask(task.task.task);
            read_body(domain, key, Body::OwnerPersonTask(task));
        }
        ReadValue::Result(result) => {
            let key = ObjectKey::Result(result.task.task);
            read_body(domain, key, Body::Result(result));
        }
        ReadValue::Activity(page) => crate::state::activity::routing::page(domain, env, *page, out, lent),
        ReadValue::History(page) => {
            domain.history_rows.cut |= page.cut;
            let edges = page_edges(page.before, page.after);
            let dropped = domain.history_rows.slide(page.rows, edges, End::After);
            if dropped > 0 {
                domain.changed();
            }
            domain.changed();
        }
        ReadValue::Tasks(page) if source == (Source::Window { number: super::project::GOALS_READ }) => {
            crate::state::page::project::ended(domain, env, source, *page, out, lent);
        }
        ReadValue::Tasks(page) => {
            crate::state::page::conversations::read_page(domain, source, *page, lent);
            crate::state::page::replies::conversations(domain, env, source, out, lent);
            domain.changed();
        }
        ReadValue::Turn(_) => unreachable!("reference pages never request a turn"),
        ReadValue::OwnerDeployment(document) => {
            crate::state::page::deployment::owner_document(domain, env, document, out, lent);
        }
    }
}

fn owner_escalation_read(domain: &mut Ui, value: Box<contract::OwnerEscalation>) {
    let key = ObjectKey::Escalation(value.task.chip.task);
    read_body(domain, key, Body::OwnerEscalation(value));
}

fn task_read(
    domain: &mut Ui,
    env: &Env<crate::Limits>,
    task: Box<contract::Task>,
    out: &mut Queue<Request>,
    lent: &jig_client::Client,
) {
    crate::state::activity::routing::document(domain, &task, out, lent);
    let key = ObjectKey::Task(task.chip.task);
    read_body(domain, key, Body::Task(task));
    crate::state::page::task::document(domain, env, out, lent);
}

// The new carrier must match its original read before retiring the obligation.
fn owner_project_read_valid(domain: &Ui, token: Token, result: &ReadResult) -> bool {
    let value = match result {
        ReadResult::Value(value) => match value {
            ReadValue::OwnerProject(value) => value,
            ReadValue::Tasks(_)
            | ReadValue::Me(_)
            | ReadValue::Historical(_)
            | ReadValue::Task(_)
            | ReadValue::OwnerProposal(_)
            | ReadValue::OwnerEscalation(_)
            | ReadValue::Question(_)
            | ReadValue::OwnerPersonTask(_)
            | ReadValue::Result(_)
            | ReadValue::OwnerEnded(_)
            | ReadValue::OwnerSpend(_)
            | ReadValue::History(_)
            | ReadValue::Activity(_)
            | ReadValue::Turn(_)
            | ReadValue::OwnerInbox(_)
            | ReadValue::Notes(_)
            | ReadValue::Settings(_)
            | ReadValue::Setting(_)
            | ReadValue::Revision(_)
            | ReadValue::Revisions(_)
            | ReadValue::OwnerDeployment(_) => return true,
        },
        ReadResult::None(_) => return true,
    };
    let Some(read) = domain.handles.reads.get(&crate::state::handles::handle(token)) else {
        return true;
    };
    match &read.what {
        Read::Project { project } => *project == value.number,
        Read::Tasks { .. }
        | Read::Me
        | Read::Task { .. }
        | Read::Proposal { .. }
        | Read::Escalation { .. }
        | Read::Question { .. }
        | Read::PersonTask { .. }
        | Read::Result { .. }
        | Read::Ended { .. }
        | Read::History { .. }
        | Read::Activity { .. }
        | Read::Turn { .. }
        | Read::Inbox { .. }
        | Read::Spend { .. }
        | Read::Notes { .. }
        | Read::Settings
        | Read::Setting(_)
        | Read::Revision(_)
        | Read::Revisions(_)
        | Read::Deployment => false,
    }
}

// Validate the original query and entire group before consuming its terminal.
fn owner_inbox_read_valid(domain: &Ui, env: &Env<crate::Limits>, token: Token, result: &ReadResult) -> bool {
    let page = match result {
        ReadResult::Value(value) => match value {
            ReadValue::OwnerInbox(page) => page,
            ReadValue::Tasks(_)
            | ReadValue::Me(_)
            | ReadValue::Task(_)
            | ReadValue::Historical(_)
            | ReadValue::OwnerProposal(_)
            | ReadValue::OwnerEscalation(_)
            | ReadValue::Question(_)
            | ReadValue::OwnerPersonTask(_)
            | ReadValue::Result(_)
            | ReadValue::History(_)
            | ReadValue::Activity(_)
            | ReadValue::Turn(_)
            | ReadValue::OwnerEnded(_)
            | ReadValue::OwnerSpend(_)
            | ReadValue::OwnerProject(_)
            | ReadValue::Notes(_)
            | ReadValue::Settings(_)
            | ReadValue::Setting(_)
            | ReadValue::Revision(_)
            | ReadValue::Revisions(_)
            | ReadValue::OwnerDeployment(_) => return true,
        },
        ReadResult::None(_) => return true,
    };
    let Some(read) = domain.handles.reads.get(&crate::state::handles::handle(token)) else {
        return true;
    };
    let group = match &read.what {
        Read::Inbox { group, .. } => *group,
        Read::Tasks { .. }
        | Read::Me
        | Read::Task { .. }
        | Read::Proposal { .. }
        | Read::Escalation { .. }
        | Read::Question { .. }
        | Read::PersonTask { .. }
        | Read::Result { .. }
        | Read::Ended { .. }
        | Read::History { .. }
        | Read::Activity { .. }
        | Read::Turn { .. }
        | Read::Spend { .. }
        | Read::Project { .. }
        | Read::Notes { .. }
        | Read::Settings
        | Read::Setting(_)
        | Read::Revision(_)
        | Read::Revisions(_)
        | Read::Deployment => return false,
    };
    let number = match read.source {
        Source::Window { number, .. } => number,
        Source::Watch(_) | Source::Card(_) => return false,
    };
    // A superseded terminal retires its own old obligation; the routing receipt
    // will discard it without consuming or replacing the current query/window.
    if domain.inbox.window(group).reading != Some(token) {
        return true;
    }
    if super::conversations::inbox_group(number) != Some(group)
        || super::inbox::query_filter(Some(&read.what)) != Some(domain.inbox.filter)
        || page.rows.capacity() > env.limits.rows
    {
        return false;
    }
    for row in &page.rows {
        if contract::InboxReadGroup::of(row.group()) != group {
            return false;
        }
    }
    true
}

fn received_value(
    domain: &mut Ui,
    env: &Env<crate::Limits>,
    source: Source,
    query: Read,
    value: ReadValue,
    out: &mut Queue<Request>,
    lent: &jig_client::Client,
) {
    match value {
        ReadValue::OwnerEnded(page) => super::task::owner_ended(domain, page),
        ReadValue::OwnerSpend(page) => super::project::owner_spend(domain, page),
        ReadValue::OwnerProject(value) => match query {
            Read::Project { project } => {
                if project == value.number {
                    crate::state::page::project::owner_document(domain, value);
                }
            }
            Read::Tasks { .. }
            | Read::Me
            | Read::Task { .. }
            | Read::Proposal { .. }
            | Read::Escalation { .. }
            | Read::Question { .. }
            | Read::PersonTask { .. }
            | Read::Result { .. }
            | Read::Ended { .. }
            | Read::History { .. }
            | Read::Activity { .. }
            | Read::Turn { .. }
            | Read::Inbox { .. }
            | Read::Spend { .. }
            | Read::Notes { .. }
            | Read::Settings
            | Read::Setting(_)
            | Read::Revision(_)
            | Read::Revisions(_)
            | Read::Deployment => {}
        },
        ReadValue::OwnerDeployment(page) => {
            if query == Read::Deployment {
                crate::state::page::deployment::owner_document(domain, env, page, out, lent);
            }
        }
        ReadValue::OwnerInbox(page) => {
            let group = match &query {
                Read::Inbox { group, .. } => *group,
                Read::Tasks { .. }
                | Read::Me
                | Read::Task { .. }
                | Read::Proposal { .. }
                | Read::Escalation { .. }
                | Read::Question { .. }
                | Read::PersonTask { .. }
                | Read::Result { .. }
                | Read::Ended { .. }
                | Read::History { .. }
                | Read::Activity { .. }
                | Read::Turn { .. }
                | Read::Spend { .. }
                | Read::Project { .. }
                | Read::Notes { .. }
                | Read::Settings
                | Read::Setting(_)
                | Read::Revision(_)
                | Read::Revisions(_)
                | Read::Deployment => return,
            };
            domain.inbox.owner_page(group, query, *page);
            crate::state::page::replies::inbox(domain, env, source, out, lent);
            crate::state::page::conversations::mark_results(domain, out, lent);
            domain.changed();
        }
        ReadValue::Tasks(_)
        | ReadValue::Me(_)
        | ReadValue::Task(_)
        | ReadValue::Historical(_)
        | ReadValue::OwnerProposal(_)
        | ReadValue::OwnerEscalation(_)
        | ReadValue::Question(_)
        | ReadValue::OwnerPersonTask(_)
        | ReadValue::Result(_)
        | ReadValue::History(_)
        | ReadValue::Activity(_)
        | ReadValue::Turn(_)
        | ReadValue::Notes(_)
        | ReadValue::Settings(_)
        | ReadValue::Setting(_)
        | ReadValue::Revision(_)
        | ReadValue::Revisions(_) => read_value(domain, env, source, value, out, lent),
    }
}

fn owner_tree_delivery(
    domain: &mut Ui,
    env: &Env<crate::Limits>,
    watch: jig_client::Handle,
    source: Source,
    tree: Box<contract::OwnerTree>,
    out: &mut Queue<Request>,
    lent: &jig_client::Client,
) {
    let Some(watch) = domain.handles.watches.get(&watch) else {
        return;
    };
    match &watch.subject {
        Subject::Tree { task, .. } => {
            if *task != tree.root.task {
                return;
            }
        }
        Subject::Me
        | Subject::Task { .. }
        | Subject::Chips { .. }
        | Subject::Inbox { .. }
        | Subject::Tasks { .. }
        | Subject::Run { .. }
        | Subject::Accounts
        | Subject::Settings
        | Subject::Deployment => return,
    }
    let owner = &tree;
    let mut keys = List::with_capacity(crate::sizes::limits::ui::tree_names(&env.limits).expect("tree names bounded"));
    keys.push(ObjectKey::Task(owner.root.task)).expect("root fits");
    for row in &owner.rows {
        let key = ObjectKey::Task(row.chip.task);
        if !keys.as_slice().contains(&key) {
            keys.push(key).expect("tree row names fit");
        }
        for proposal in &row.proposals {
            let key = ObjectKey::Proposal(*proposal);
            if !keys.as_slice().contains(&key) {
                keys.push(key).expect("tree proposal names fit");
            }
        }
    }
    names(domain, env, source, keys, out, lent);
    crate::state::page::task::owner_tree(domain, &tree);
    domain.changed();
}

fn historical_funders_fit(page: &contract::OwnerSpendPage, project: u32) -> bool {
    for row in &page.rows {
        match row.funder {
            contract::Funder::Period { project: owner, .. } => {
                if owner != project {
                    return false;
                }
            }
            contract::Funder::Pool { .. } | contract::Funder::Task(_) | contract::Funder::Recurring { .. } => {
                return false;
            }
        }
    }
    true
}

/// Validate the retained original request before consuming its terminal.
#[expect(clippy::too_many_lines, reason = "the original query checks all closed read variants before applying history")]
fn historical_read_valid(domain: &Ui, env: &Env<crate::Limits>, token: Token, result: &ReadResult) -> bool {
    let value = match result {
        ReadResult::Value(value) => value,
        ReadResult::None(_) => return true,
    };
    let Some(read) = domain.handles.reads.get(&crate::state::handles::handle(token)) else {
        return true;
    };
    match value {
        ReadValue::OwnerEnded(page) => {
            let task = match &read.what {
                Read::Ended { task, .. } => *task,
                Read::Tasks { .. }
                | Read::Me
                | Read::Task { .. }
                | Read::Proposal { .. }
                | Read::Escalation { .. }
                | Read::Question { .. }
                | Read::PersonTask { .. }
                | Read::Result { .. }
                | Read::History { .. }
                | Read::Activity { .. }
                | Read::Turn { .. }
                | Read::Inbox { .. }
                | Read::Spend { .. }
                | Read::Project { .. }
                | Read::Notes { .. }
                | Read::Settings
                | Read::Setting(_)
                | Read::Revision(_)
                | Read::Revisions(_)
                | Read::Deployment => return false,
            };
            let source = match read.source {
                Source::Window { number: super::task::ENDED_READ, .. } => true,
                Source::Window { .. } | Source::Watch(_) | Source::Card(_) => false,
            };
            source
                && domain.task_page.ended_task == Some(task)
                && page.rows.capacity() <= env.limits.rows
                && historical_edges_fit(page.before.as_ref(), page.after.as_ref(), env.limits.text)
        }
        ReadValue::OwnerSpend(page) => {
            let project = match &read.what {
                Read::Spend { project, .. } => *project,
                Read::Tasks { .. }
                | Read::Me
                | Read::Task { .. }
                | Read::Proposal { .. }
                | Read::Escalation { .. }
                | Read::Question { .. }
                | Read::PersonTask { .. }
                | Read::Result { .. }
                | Read::Ended { .. }
                | Read::History { .. }
                | Read::Activity { .. }
                | Read::Turn { .. }
                | Read::Inbox { .. }
                | Read::Project { .. }
                | Read::Notes { .. }
                | Read::Settings
                | Read::Setting(_)
                | Read::Revision(_)
                | Read::Revisions(_)
                | Read::Deployment => return false,
            };
            let source = match read.source {
                Source::Window { number: super::project::SPEND_READ, .. } => true,
                Source::Window { .. } | Source::Watch(_) | Source::Card(_) => false,
            };
            if !source
                || super::project::number(domain) != Some(project)
                || page.rows.capacity() > env.limits.rows
                || !historical_edges_fit(page.before.as_ref(), page.after.as_ref(), env.limits.text)
            {
                return false;
            }
            historical_funders_fit(page, project)
        }
        ReadValue::Tasks(_)
        | ReadValue::Me(_)
        | ReadValue::Task(_)
        | ReadValue::Historical(_)
        | ReadValue::OwnerProposal(_)
        | ReadValue::OwnerEscalation(_)
        | ReadValue::Question(_)
        | ReadValue::OwnerPersonTask(_)
        | ReadValue::Result(_)
        | ReadValue::History(_)
        | ReadValue::Activity(_)
        | ReadValue::Turn(_)
        | ReadValue::OwnerInbox(_)
        | ReadValue::OwnerProject(_)
        | ReadValue::Notes(_)
        | ReadValue::OwnerDeployment(_)
        | ReadValue::Settings(_)
        | ReadValue::Setting(_)
        | ReadValue::Revision(_)
        | ReadValue::Revisions(_) => true,
    }
}
fn historical_edges_fit(before: Option<&contract::Cursor>, after: Option<&contract::Cursor>, text: u32) -> bool {
    if let Some(cursor) = before
        && !cursor.value.fits(text)
    {
        return false;
    }
    if let Some(cursor) = after
        && !cursor.value.fits(text)
    {
        return false;
    }
    true
}

// Check the original read and supplied scope before consuming its one terminal.
fn owner_escalation_read_valid(domain: &Ui, token: Token, result: &ReadResult, lent: &jig_client::Client) -> bool {
    let document = match result {
        ReadResult::Value(ReadValue::OwnerEscalation(document)) => document,
        ReadResult::Value(
            ReadValue::Tasks(_)
            | ReadValue::Me(_)
            | ReadValue::Task(_)
            | ReadValue::Historical(_)
            | ReadValue::OwnerProposal(_)
            | ReadValue::OwnerProject(_)
            | ReadValue::Question(_)
            | ReadValue::OwnerPersonTask(_)
            | ReadValue::Result(_)
            | ReadValue::OwnerEnded(_)
            | ReadValue::History(_)
            | ReadValue::Activity(_)
            | ReadValue::Turn(_)
            | ReadValue::OwnerInbox(_)
            | ReadValue::OwnerSpend(_)
            | ReadValue::Notes(_)
            | ReadValue::Settings(_)
            | ReadValue::Setting(_)
            | ReadValue::Revision(_)
            | ReadValue::Revisions(_)
            | ReadValue::OwnerDeployment(_),
        )
        | ReadResult::None(_) => return true,
    };
    let Some(read) = domain.handles.reads.get(&crate::state::handles::handle(token)) else {
        return true;
    };
    let task = match read.what {
        Read::Escalation { task } => task,
        Read::Tasks { .. }
        | Read::Me
        | Read::Task { .. }
        | Read::Proposal { .. }
        | Read::Question { .. }
        | Read::PersonTask { .. }
        | Read::Result { .. }
        | Read::Ended { .. }
        | Read::History { .. }
        | Read::Activity { .. }
        | Read::Turn { .. }
        | Read::Inbox { .. }
        | Read::Spend { .. }
        | Read::Project { .. }
        | Read::Notes { .. }
        | Read::Settings
        | Read::Setting(_)
        | Read::Revision(_)
        | Read::Revisions(_)
        | Read::Deployment => return false,
    };
    if task != document.task.chip.task {
        return false;
    }
    let Some(id) = domain.objects.find(ObjectKey::Escalation(task)) else {
        return true;
    };
    match domain.borrow(lent).objects().get(id) {
        Some(object) => match object.project {
            Some(project) => project == document.task.project,
            None => true,
        },
        None => true,
    }
}

fn rejected_read(domain: &mut Ui, token: Token) {
    if let Some(receipt) = domain.handles.received(crate::state::handles::handle(token))
        && let Read::Inbox { group, .. } = receipt.what
        && domain.inbox.window(group).reading == Some(token)
    {
        domain.inbox.window_mut(group).reading = None;
    }
}
