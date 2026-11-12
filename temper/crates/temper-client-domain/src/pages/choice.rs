//! Next-run drafts take their models and account preferences from the offer (client/pages/README.md).
use super::{Pages, Request, accounts};
use crate::Address;
use crate::owner::Owner;
use alloc::boxed::Box;
use jig_client::{Client, Key, Outcome, Told, contract as c};
use skein_lib::{List, Queue};

/// Catalogue ceilings enforced by the received client wire.
pub const MODELS: u32 = jig_client_protocol_wire::v20::CEILINGS.me_project_models;
pub const MODEL_ACCOUNTS: u32 = jig_client_protocol_wire::v20::CEILINGS.me_model_accounts;
pub const EFFORTS: u32 = jig_client_protocol_wire::v20::CEILINGS.me_model_efforts;

/// The search admits at most this many UTF-8 bytes.
pub const FIND: u32 = 256;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Context {
    Template {
        project: u32,
        template: u32,
        executor: c::TemplateExecutor,
        model_index: Option<u8>,
    },
    /// The Ask template for an Ask temper chat, otherwise CHAT.
    Task {
        project: u32,
        task: u64,
        template: u32,
        revision: c::Revision,
        model_index: Option<u8>,
    },
}
impl Context {
    // The row fits the Task variant's existing alignment space: no extra draft bytes.
    // None resolves a supplied choice by name until a person picks a row.
    // A byte index covers the wire catalogue ceiling (64 rows).
    fn model_index(self) -> Option<u32> {
        let index = match self {
            Self::Template { model_index, .. } | Self::Task { model_index, .. } => model_index,
        };
        Some(u32::from(index?))
    }
    fn select(&mut self, index: Option<u8>) {
        match self {
            Self::Template { model_index, .. } | Self::Task { model_index, .. } => *model_index = index,
        }
    }
    fn unselected(context: Option<Self>) -> Option<Self> {
        match context {
            Some(mut context) => {
                context.select(None);
                Some(context)
            }
            None => None,
        }
    }
}
#[derive(Debug)]
pub struct Draft {
    pub context: Option<Context>,
    pub original: Option<c::LlmChoice>,
    pub requested: Option<c::LlmChoice>,
    pub pending: Option<Key>,
    pub outcome: Option<Outcome>,
    /// The menu is on All models.
    pub all: bool,
    /// Find a model, at most FIND bytes.
    pub find: Box<[u8]>,
}
#[expect(
    clippy::derivable_impls,
    reason = "the programming model permits explicit Default implementations, not its derive"
)]
impl Default for Draft {
    fn default() -> Self {
        Self {
            context: None,
            original: None,
            requested: None,
            pending: None,
            outcome: None,
            all: false,
            find: Box::default(),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Act {
    Model { index: u32, account: Option<u32> },
    Effort { index: u32 },
    All,
    Back,
    Find { words: Box<[u8]> },
}
#[must_use]
pub fn offer(me: &c::Me, project: u32) -> Option<&List<c::MeModel>> {
    for row in &me.projects {
        if row.project == project {
            return Some(&row.models);
        }
    }
    None
}
impl Draft {
    pub(crate) fn reset(&mut self, context: Option<Context>, original: Option<&c::LlmChoice>) {
        let context = Context::unselected(context);
        let same_menu = match (self.context, context) {
            (
                Some(Context::Template { project: a, template: b, .. }),
                Some(Context::Template { project: x, template: y, .. }),
            ) => a == x && b == y,
            (Some(Context::Task { project: a, task: b, .. }), Some(Context::Task { project: x, task: y, .. })) => {
                a == x && b == y
            }
            (Some(Context::Template { .. }), Some(Context::Task { .. }))
            | (Some(Context::Task { .. }), Some(Context::Template { .. }))
            | (None, None | Some(Context::Template { .. } | Context::Task { .. }))
            | (Some(Context::Template { .. } | Context::Task { .. }), None) => false,
        };
        if !same_menu {
            self.all = false;
            self.find = Box::default();
        }
        if Context::unselected(self.context) != context || self.original.as_ref() != original {
            self.context = context;
            self.original = original.cloned();
            self.requested = original.cloned();
            self.outcome = None;
        }
    }
    #[must_use]
    pub fn project(&self) -> Option<u32> {
        match self.context? {
            Context::Template { project, .. } | Context::Task { project, .. } => Some(project),
        }
    }
    #[must_use]
    pub fn chosen<'a>(&'a self, me: &'a c::Me) -> Option<&'a c::LlmChoice> {
        self.requested.as_ref().or(self.resolved(me))
    }
    fn resolved<'a>(&self, me: &'a c::Me) -> Option<&'a c::LlmChoice> {
        let template = match self.context? {
            Context::Template { template, .. } | Context::Task { template, .. } => template,
        };
        jig_client::shapes::resolved_default(me, self.project()?, template)
    }
    #[must_use]
    pub fn default_row(&self, me: &c::Me) -> Option<u32> {
        let choice = self.resolved(me)?;
        for (model, index) in offer(me, self.project()?)?.iter().zip(0u32..) {
            if model.name == choice.main.model {
                return Some(index);
            }
        }
        None
    }
    #[must_use]
    #[expect(
        clippy::disallowed_methods,
        reason = "Unicode case matching reads the validated text as str; it does not validate protocol text"
    )]
    pub fn finds(&self, model: &c::MeModel) -> bool {
        let Ok(words) = core::str::from_utf8(&self.find) else { return false };
        let words = words.to_lowercase();
        if core::str::from_utf8(model.name.bytes()).expect("model name is text").to_lowercase().contains(&words) {
            return true;
        }
        match &model.shown {
            Some(shown) => {
                core::str::from_utf8(shown.bytes()).expect("shown name is text").to_lowercase().contains(&words)
            }
            None => false,
        }
    }
    /// Exactly one offered row supplies the draft's effort controls and validation.
    #[must_use]
    pub fn selected(&self, me: &c::Me) -> Option<u32> {
        let choice = self.chosen(me)?;
        let models = offer(me, self.project()?)?;
        if let Some(index) = self.context?.model_index() {
            let model = models.get(index)?;
            return if model.name == choice.main.model { Some(index) } else { None };
        }
        for (model, index) in models.iter().zip(0u32..) {
            if model.name == choice.main.model {
                return Some(index);
            }
        }
        None
    }
    #[must_use]
    pub fn valid(&self, me: &c::Me) -> bool {
        if self.requested == self.original {
            return true;
        }
        let Some(choice) = &self.requested else { return true };
        let Some(index) = self.selected(me) else { return false };
        let Some(project) = self.project() else { return false };
        let Some(models) = offer(me, project) else { return false };
        let Some(model) = models.get(index) else { return false };
        if model.efforts.is_empty() {
            choice.main.effort.is_none()
        } else if let Some(effort) = &choice.main.effort {
            model.efforts.as_slice().contains(effort)
        } else {
            false
        }
    }
    pub(crate) fn edit(&mut self, me: &c::Me, act: Act) {
        if self.pending.is_some() {
            return;
        }
        let Some(project) = self.project() else { return };
        let Some(models) = offer(me, project) else { return };
        match act {
            Act::Model { index, account } => {
                let Some(model) = models.get(index) else { return };
                let first = model.accounts.get(0).copied();
                let default_account = account.is_none() || account == first;
                let original_model = match &self.original {
                    Some(choice) => choice.main.model == model.name,
                    None => true,
                };
                if self.default_row(me) == Some(index) && default_account && original_model {
                    self.context = Context::unselected(self.context);
                    self.requested = self.original.clone();
                    self.outcome = None;
                    return;
                }
                let Some(context) = &mut self.context else { return };
                let Ok(index) = u8::try_from(index) else { return };
                context.select(Some(index));
                if let Some(choice) = &mut self.requested {
                    choice.account = account;
                    choice.main = c::ModelEffort { model: model.name.clone(), effort: model.effort.clone() };
                } else {
                    self.requested = Some(c::LlmChoice {
                        later: None,
                        account,
                        main: c::ModelEffort { model: model.name.clone(), effort: model.effort.clone() },
                        subagents: List::with_capacity(0),
                    });
                }
            }
            Act::Effort { index } => {
                let Some(selected) = self.selected(me) else { return };
                let Some(model) = models.get(selected) else { return };
                let Some(effort) = model.efforts.get(index) else { return };
                if self.requested.is_none() {
                    self.requested = self.chosen(me).cloned();
                }
                let Some(choice) = &mut self.requested else { return };
                choice.main.effort = Some(effort.clone());
            }
            Act::All => self.all = true,
            Act::Back => self.all = false,
            Act::Find { words } => {
                if words.len() <= usize::try_from(FIND).expect("find bound fits") && skein_lib::bytes::is_utf8(&words) {
                    self.find = words;
                }
            }
        }
    }
    pub(crate) fn amend(&mut self, me: &c::Me, task: &c::Task, serial: &mut u64, out: &mut Queue<Request>) {
        let Some(Context::Task { project, task: number, revision, .. }) = self.context else { return };
        if self.pending.is_some() || self.requested == self.original || !self.valid(me) || number != task.chip.task {
            return;
        }
        let admission = jig_client::facts::offer(task.acts.as_slice(), c::ActKind::Amend);
        let change = Box::new(c::Amendment {
            addressed: None,
            spec: None,
            wake: None,
            dependencies: None,
            authority: None,
            instructions: None,
            tracked: None,
            llm: self.requested.clone(),
        });
        let Ok(reason) = c::Text::new(Box::from(*b"Change next run"), u32::MAX) else { return };
        let ask = match admission {
            Some(c::Admission::Allowed | c::Admission::Bounded { .. }) => {
                c::Ask::Amend { project, task: number, revision: Some(revision), change, reason }
            }
            Some(c::Admission::Proposed { .. }) => c::Ask::Propose {
                project,
                action: c::ProposeAction::Amendment { task: number, revision, change },
                reason,
            },
            Some(c::Admission::Refused(_)) | None => return,
        };
        *serial = serial.checked_add(1).expect("page keys fit");
        let key = Key { asker: Owner::Pages.asker(), number: *serial };
        self.pending = Some(key);
        self.outcome = None;
        out.push(Request::Client(jig_client::Event::Send {
            key,
            name: None,
            request: jig_client::Ask::Jig(Box::new(ask)),
        }));
    }
    pub(crate) fn told(&mut self, told: &Told) -> bool {
        if let Told::Answered { key, outcome } = told
            && self.pending == Some(*key)
        {
            self.pending = None;
            if !keeps_requested(outcome) {
                self.requested = self.original.clone();
                self.context = Context::unselected(self.context);
            }
            self.outcome = Some(outcome.clone());
            return true;
        }
        false
    }
}
pub(crate) fn refresh(pages: &mut Pages, client: &Client) {
    let Some(me) = accounts::current_me(client) else {
        pages.choice.reset(None, None);
        return;
    };
    let task = if let Some(chat) = &pages.chat {
        chat.task(client)
    } else if let Some(task) = &pages.task {
        task.document(client)
    } else {
        None
    };
    if let Some(task) = task {
        pages.choice.reset(
            Some(Context::Task {
                project: task.project,
                task: task.chip.task,
                template: if super::chats::asking(me, task.project, &task.chip.executor) {
                    super::chats::ASKING_TEMPLATE
                } else {
                    jig_client::shapes::CHAT
                },
                revision: task.spec_revision,
                model_index: None,
            }),
            task.llm.as_ref(),
        );
    } else if pages.chats.is_some() {
        let template = if let Address::Ask { .. } | Address::Home { .. } = pages.address() {
            super::chats::ASKING_TEMPLATE
        } else {
            jig_client::shapes::CHAT
        };
        refresh_template(pages, me, template);
    } else {
        pages.choice.reset(None, None);
    }
}
fn refresh_template(pages: &mut Pages, me: &c::Me, number: u32) {
    if let Some(project) = pages.project
        && let Some(template) = jig_client::shapes::template(me, project, number)
    {
        pages.choice.reset(
            Some(Context::Template { project, template: number, executor: template.executor, model_index: None }),
            template.llm.as_ref(),
        );
    } else {
        pages.choice.reset(None, None);
    }
}
pub(crate) fn act(pages: &mut Pages, client: &Client, act: Act, out: &mut Queue<Request>) {
    let Some(me) = accounts::current_me(client) else { return };
    let sends = match &act {
        Act::Model { .. } | Act::Effort { .. } => true,
        Act::All | Act::Back | Act::Find { .. } => false,
    };
    pages.choice.edit(me, act);
    if sends {
        let task = if let Some(chat) = &pages.chat {
            chat.task(client)
        } else if let Some(task) = &pages.task {
            task.document(client)
        } else {
            None
        };
        if let Some(task) = task {
            pages.choice.amend(me, task, &mut pages.serial, out);
        }
    }
}

fn keeps_requested(outcome: &Outcome) -> bool {
    match outcome {
        Outcome::Answered(jig_client::Answered::Jig(c::Answer::Done(value))) => match value {
            c::Outcome::Document(value) => match value {
                c::DocumentOutcome::Proposed { .. } => false,
                c::DocumentOutcome::Lifted { .. }
                | c::DocumentOutcome::AccountFinishing { .. }
                | c::DocumentOutcome::Created { .. }
                | c::DocumentOutcome::Compound { .. }
                | c::DocumentOutcome::MarkedSeen { .. }
                | c::DocumentOutcome::NoteEdited { .. }
                | c::DocumentOutcome::ServiceMade { .. }
                | c::DocumentOutcome::Adopted { .. }
                | c::DocumentOutcome::Left { .. }
                | c::DocumentOutcome::Stopped { .. }
                | c::DocumentOutcome::Cancelled { .. }
                | c::DocumentOutcome::Released { .. }
                | c::DocumentOutcome::PersonTaken { .. }
                | c::DocumentOutcome::PersonHandedBack { .. }
                | c::DocumentOutcome::PersonAnswered { .. }
                | c::DocumentOutcome::Moved { .. }
                | c::DocumentOutcome::ProposalDecided { .. }
                | c::DocumentOutcome::Said { .. }
                | c::DocumentOutcome::QuestionAnswered { .. }
                | c::DocumentOutcome::Amended { .. }
                | c::DocumentOutcome::Withdrawn { .. }
                | c::DocumentOutcome::RolesSet { .. }
                | c::DocumentOutcome::PolicyChanged { .. }
                | c::DocumentOutcome::Renamed { .. }
                | c::DocumentOutcome::ProjectRemoved { .. }
                | c::DocumentOutcome::ProjectMade { .. }
                | c::DocumentOutcome::PoolSet { .. }
                | c::DocumentOutcome::EscalationDecided { .. }
                | c::DocumentOutcome::AccountKeyKept { .. }
                | c::DocumentOutcome::AccountProgramDeclared { .. }
                | c::DocumentOutcome::AccountMovedIn { .. }
                | c::DocumentOutcome::AccountRemoved { .. }
                | c::DocumentOutcome::AccountSignInStarted { .. }
                | c::DocumentOutcome::AccountSignedOut { .. }
                | c::DocumentOutcome::VisibilitySet { .. }
                | c::DocumentOutcome::AccountChanged => true,
            },
            c::Outcome::Restored { .. }
            | c::Outcome::Account(_)
            | c::Outcome::SettingsChanged(_)
            | c::Outcome::MovedToNewest(_)
            | c::Outcome::Made { .. } => true,
        },
        Outcome::Answered(jig_client::Answered::Jig(c::Answer::Refused { .. }) | jig_client::Answered::Family(_))
        | Outcome::NotKept
        | Outcome::Unknown
        | Outcome::Limit
        | Outcome::Unconfirmed
        | Outcome::Held
        | Outcome::Unsent
        | Outcome::Full { .. }
        | Outcome::Lost => false,
    }
}
