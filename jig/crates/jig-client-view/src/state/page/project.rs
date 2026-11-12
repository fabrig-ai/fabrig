//! Project documents, board and scoped notes belong to the page generation.
//! Cards remain in the shared object set; connector values stay in root slots.
//! Goal creation has one page confirmation and one keyed intent. See
//! client/domain/pages.md, 6, and client/domain/root.md, 6.
//!
//! | Page | Event | Next | Emits |
//! |---|---|---|---|
//! | loading | project document | current facts | section parts |
//! | board | goals | ordered board | shared task names |
//! | notes | scope/search | reading | scoped notes read |
//! | any | more | reading | opaque page cursor |
//! | any | leave | empty context | parts leave through routing |
use crate::{
    Request, Ui,
    state::objects::{Object, Source},
};
use alloc::boxed::Box;
use jig_client::contract::{self, ApplicationPart, Cursor, NoteScope, Read, TaskLine, TaskQuery, Text};
use jig_client::thread::window::End;
use skein_lib::{Env, Id, List, Queue, Token};
pub(crate) mod goals;
pub(crate) mod moving;
pub(crate) mod notes;
pub(crate) mod owner_form;
/// Selected scope and description search, passed to the notes owner unchanged.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct NoteFilter {
    pub scope: NoteScope,
    pub description: Option<Text>,
}
/// Only current project context; pending goal requests survive leaving the page.
#[derive(Debug)]
pub struct ProjectPage {
    /// Retained funding rows, independent of live project spend.
    pub owner_spend: Option<Box<contract::SpendPage>>,
    /// Whole owner facts, with no fabricated earlier document.
    pub owner: Option<Box<contract::Project>>,
    /// The current read window, which may include both live and ended goals.
    pub ended: List<TaskLine>,
    /// Keyed display merge; the board's current row wins over a read row.
    pub ended_cut: bool,
    pub ended_before: Option<Cursor>,
    pub ended_after: Option<Cursor>,
    pub task_filter: TaskQuery,
    pub notes: List<Id<Object>>,
    pub note_filter: Option<NoteFilter>,
    /// Ui writes of the selected note scope and description.
    pub note_writes: u64,
    pub notes_cut: bool,
    pub notes_before: Option<Cursor>,
    pub notes_after: Option<Cursor>,
    pub spend_cut: bool,
    pub spend_before: Option<Cursor>,
    pub spend_after: Option<Cursor>,
    pub form: goals::FormState,
    pub settings: owner_form::Form,
}
impl ProjectPage {
    pub(crate) fn new(rows: u32) -> Self {
        Self {
            owner_spend: None,
            owner: None,
            ended: List::with_capacity(rows),
            ended_cut: false,
            ended_before: None,
            ended_after: None,
            task_filter: TaskQuery {
                project: None,
                requester: None,
                root: None,
                parent: None,
                label: None,
                phase: None,
                executor: None,
                tracked: None,
                order: contract::QueryOrder::Changed,
            },
            notes: List::with_capacity(rows),
            note_filter: None,
            note_writes: 0,
            notes_cut: false,
            notes_before: None,
            notes_after: None,
            spend_cut: false,
            spend_before: None,
            spend_after: None,
            form: goals::FormState::new(),
            settings: owner_form::Form::new(),
        }
    }
    pub(crate) fn clear(&mut self) {
        self.owner_spend = None;
        self.owner = None;
        self.ended.clear();
        self.ended_cut = false;
        self.ended_before = None;
        self.ended_after = None;
        self.task_filter = TaskQuery {
            project: None,
            requester: None,
            root: None,
            parent: None,
            label: None,
            phase: None,
            executor: None,
            tracked: None,
            order: contract::QueryOrder::Changed,
        };
        self.notes.clear();
        self.note_filter = None;
        self.note_writes = self.note_writes.checked_add(1).expect("note writes remain");
        self.notes_cut = false;
        self.notes_before = None;
        self.notes_after = None;
        self.spend_cut = false;
        self.spend_before = None;
        self.spend_after = None;
        self.form.confirmation = None;
        self.settings.review = None;
        self.settings.service_role = None;
        self.settings.pool_period = None;
        self.settings.before = None;
        self.settings.refused = None;
    }
}
/// Number of the currently shown project, never inferred from another page.
#[must_use]
pub(crate) fn number(domain: &Ui) -> Option<u32> {
    domain.shown_project()
}
fn open(domain: &mut Ui, subject: contract::Subject, out: &mut Queue<Request>) {
    super::routing::open_once(domain, subject, crate::state::page::shown::owner(domain), out);
}
pub(crate) fn ask(domain: &mut Ui, number: u16, what: Read, out: &mut Queue<Request>) {
    crate::state::page::shown::context_read(domain, &what);
    if domain.handles.pending(&what) {
        return;
    }
    if domain.handles.read(Source::Window { number }, what, out).is_none() {
        crate::state::page::routing::refuse(domain, contract::Limit::Reads);
    }
}

pub(crate) fn show(
    domain: &mut Ui,
    _env: &Env<crate::Config>,
    parts: List<ApplicationPart>,
    out: &mut Queue<Request>,
    lent: &jig_client::Client,
) {
    let mut count = 0_u32;
    for part in parts.into_boxed() {
        if count == lent.domain_limits().parts {
            crate::state::page::routing::refuse(domain, contract::Limit::Objects);
            break;
        }
        let scope = match domain.shown_project() {
            Some(project) => contract::PartScope::Project(project),
            None => contract::PartScope::Deployment,
        };
        let scope = crate::state::application::owner_scope(&part, scope);
        out.push(Request::ShowPart { part, scope });
        count = count.saturating_add(1);
    }
}
/// Replaces the live task watch so every filter is authoritative at its source.
pub(crate) fn tasks(
    domain: &mut Ui,
    _env: &Env<crate::Config>,
    filter: TaskQuery,
    out: &mut Queue<Request>,
    lent: &jig_client::Client,
) {
    let Some(project) = number(domain) else {
        return;
    };
    let mut old = List::with_capacity(lent.domain_limits().watches);
    for (id, watch) in &domain.handles.watches {
        match watch.subject {
            contract::Subject::Tasks { .. } => {
                if watch.owner == (crate::state::page::shown::owner(domain))
                    && watch.subject
                        == (contract::Subject::Tasks {
                            query: TaskQuery { project: Some(project), ..domain.project_page.task_filter },
                            window: lent.domain_limits().rows,
                        })
                {
                    old.push(*id).expect("watch IDs fit");
                }
            }
            contract::Subject::Me
            | contract::Subject::Task { .. }
            | contract::Subject::Tree { .. }
            | contract::Subject::Chips { .. }
            | contract::Subject::Inbox { .. }
            | contract::Subject::Run { .. }
            | contract::Subject::Terminal { .. }
        | contract::Subject::Accounts
            | contract::Subject::Settings
            | contract::Subject::Deployment => {}
        }
    }
    for id in old.into_boxed() {
        domain.release(id, out);
    }
    domain.project_page.task_filter = filter;
    open(
        domain,
        contract::Subject::Tasks {
            query: TaskQuery { project: Some(project), ..filter },
            window: lent.domain_limits().rows,
        },
        out,
    );
    domain.changed();
}
/// Opaque windows exposed by the board, notes and spend views.
pub const GOALS_WINDOW: u64 = 20;
/// The goals read source.
pub(crate) const GOALS_READ: u16 = 20;
/// Scoped notes window.
pub const NOTES_WINDOW: u64 = 21;
/// The notes read source.
pub(crate) const NOTES_READ: u16 = 3;
/// Past spend periods window.
pub const SPEND_WINDOW: u64 = 22;
/// The spend read source.
pub(crate) const SPEND_READ: u16 = 23;
/// The project document read source.
pub(crate) const PROJECT_READ: u16 = 0;
pub(crate) fn more(domain: &mut Ui, window: Token, end: End, out: &mut Queue<Request>, lent: &jig_client::Client) {
    let Some(project) = number(domain) else {
        return;
    };
    if window == Token::new(GOALS_WINDOW) {
        let cursor = match end {
            End::Before => domain.project_page.ended_before.clone(),
            End::After => domain.project_page.ended_after.clone(),
        };
        if let Some(cursor) = cursor
            && let Some(me) = domain.frame(lent).document
            && let Some(template) = crate::state::queries::goal_template(me, project)
        {
            ask(
                domain,
                GOALS_READ,
                Read::Tasks {
                    query: crate::state::queries::ended_goals(project, template.label),
                    live: None,
                    ended: Some(cursor),
                },
                out,
            );
        }
    } else if window == Token::new(NOTES_WINDOW) {
        let cursor = match end {
            End::Before => domain.project_page.notes_before.clone(),
            End::After => domain.project_page.notes_after.clone(),
        };
        if let Some(cursor) = cursor
            && let Some(filter) = domain.project_page.note_filter.clone()
        {
            ask(
                domain,
                NOTES_READ,
                Read::Notes { scope: filter.scope, description: filter.description, after: Some(cursor) },
                out,
            );
        }
    } else if window == Token::new(SPEND_WINDOW) {
        let cursor = match end {
            End::Before => domain.project_page.spend_before.clone(),
            End::After => domain.project_page.spend_after.clone(),
        };
        if let Some(cursor) = cursor {
            ask(domain, SPEND_READ, Read::Spend { project, after: Some(cursor) }, out);
        }
    }
}
/// Whether these limits are representable: every edge bound they size
/// is finite and every capacity they derive fits (programming-model.md, 6.3).
#[must_use]
pub(crate) fn valid(limits: &jig_client::DomainLimits) -> bool {
    limits.rows.checked_mul(2).is_some()
}

pub(crate) fn ended(
    domain: &mut Ui,
    env: &Env<crate::Config>,
    source: Source,
    page: contract::TaskPage,
    out: &mut Queue<Request>,
    lent: &jig_client::Client,
) {
    let contract::EndedGroup::Rows { rows, after, cut } = page.ended else { return };
    domain.project_page.ended_cut = cut;
    domain.project_page.ended_before = None;
    domain.project_page.ended_after = after;
    domain.project_page.ended.clear();
    for row in rows.into_boxed() {
        if domain.project_page.ended.room() == 0 {
            break;
        }
        domain.project_page.ended.push(row).expect("ended row fits");
    }
    let mut keys = List::with_capacity(lent.domain_limits().rows);
    for goal in &domain.project_page.ended {
        keys.push(contract::ObjectKey::Task(goal.chip.task)).expect("keys fit row window");
    }
    crate::state::page::routing::names(domain, env, source, keys, out, lent);
    for goal in &domain.project_page.ended {
        if !domain
            .objects
            .put(contract::ObjectKey::Task(goal.chip.task), crate::state::objects::Body::Chip(goal.chip.clone()))
        {
            break;
        }
    }
    domain.changed();
}
pub(crate) fn task_rows(
    domain: &mut Ui,
    env: &Env<crate::Config>,
    source: Source,
    tasks: contract::Tasks,
    out: &mut Queue<Request>,
    lent: &jig_client::Client,
) {
    let mut keys = List::with_capacity(lent.domain_limits().rows);
    for task in &tasks.rows {
        keys.push(contract::ObjectKey::Task(task.chip.task)).expect("keys fit rows");
    }
    crate::state::page::routing::names(domain, env, source, keys, out, lent);
    domain.changed();
}

/// Install the complete retained source page in supplied order.
pub(crate) fn owner_spend(domain: &mut Ui, page: Box<contract::SpendPage>) {
    domain.project_page.spend_cut = page.cut;
    domain.project_page.spend_before.clone_from(&page.before);
    domain.project_page.spend_after.clone_from(&page.after);
    domain.project_page.owner_spend = Some(page);
    domain.changed();
}

/// Receive only the original project's whole owner value; order stays supplied.
pub(crate) fn owner_document(domain: &mut Ui, value: Box<contract::Project>) {
    if number(domain) != Some(value.number) {
        return;
    }
    domain.project_page.owner = Some(value);
    domain.changed();
}
