//! The banner's project forms and keyed acts (temper/docs/design/client/view.md, 3).
use super::Request;
use crate::{Address, owner::Owner};
use alloc::boxed::Box;
use jig_client::{Key, Outcome, Told, contract as c};
use skein_lib::Queue;

/// jig's project name bound (jig-domain-people's `NAME_BYTES`).
const NAME_BYTES: u32 = 64;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Act {
    /// Open the rename's form for the open project.
    Rename,
    /// The rename's field.
    RenameWords { words: Box<[u8]> },
    /// Save the trimmed name, or close unchanged, from "Save" or the field's Enter.
    Save,
    /// Close without sending, from "Cancel" or Escape.
    Cancel,
    /// The new project's field.
    NewWords { words: Box<[u8]> },
    /// The new project's "Create", or its field's Enter.
    Create,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Form {
    Rename,
    New,
}
#[derive(Clone, Debug)]
pub struct Switcher {
    renaming: Option<u32>,
    rename_words: Box<[u8]>,
    new_words: Box<[u8]>,
    pending: Option<(Key, Form)>,
    refusal: Option<(Form, Outcome)>,
    pub(super) opening: Option<u32>,
}
#[expect(clippy::derivable_impls, reason = "the programming model uses explicit Default implementations")]
impl Default for Switcher {
    fn default() -> Self {
        Self {
            renaming: None,
            rename_words: Box::default(),
            new_words: Box::default(),
            pending: None,
            refusal: None,
            opening: None,
        }
    }
}
impl Switcher {
    /// The project whose rename's form is open.
    #[must_use]
    pub const fn renaming(&self) -> Option<u32> {
        self.renaming
    }
    #[must_use]
    pub fn rename_words(&self) -> &[u8] {
        &self.rename_words
    }
    #[must_use]
    pub fn new_words(&self) -> &[u8] {
        &self.new_words
    }
    #[must_use]
    pub const fn pending(&self) -> Option<Form> {
        match self.pending {
            Some((_, form)) => Some(form),
            None => None,
        }
    }
    #[must_use]
    pub const fn refusal(&self) -> Option<(Form, &Outcome)> {
        match &self.refusal {
            Some((form, outcome)) => Some((*form, outcome)),
            None => None,
        }
    }
    pub(super) fn went(&mut self, address: &Address) {
        match address {
            Address::Told
            | Address::Remembered
            | Address::Members
            | Address::Limits
            | Address::Accounts
            | Address::Forges
            | Address::ConnectForge { .. }
            | Address::ConnectModel { .. }
            | Address::Welcome { .. }
            | Address::AllHome
            | Address::Inbox { .. } => self.close(),
            Address::Home { .. }
            | Address::Changes { .. }
            | Address::Team { .. }
            | Address::Settings { .. }
            | Address::Models { .. }
            | Address::Builders { .. }
            | Address::Role { .. }
            | Address::Rule { .. }
            | Address::Repositories { .. }
            | Address::Work { .. }
            | Address::Goals { .. }
            | Address::Ask { .. }
            | Address::Chats { .. }
            | Address::NewChat { .. }
            | Address::Chat { .. }
            | Address::Task { .. }
            | Address::Adopt { .. } => {}
        }
    }
    pub(super) fn close(&mut self) {
        self.renaming = None;
        self.rename_words = Box::default();
        self.clear_refusal(Form::Rename);
    }
    fn clear_refusal(&mut self, form: Form) {
        if let Some((refused, _)) = &self.refusal
            && *refused == form
        {
            self.refusal = None;
        }
    }
    pub(super) fn act(
        &mut self,
        me: Option<&c::Me>,
        project: Option<u32>,
        act: Act,
        serial: &mut u64,
        out: &mut Queue<Request>,
    ) {
        match act {
            Act::Rename if self.pending.is_none() => {
                let Some(row) = owned(me, project) else { return };
                let Ok(words) = c::Text::new(row.name.bytes().into(), NAME_BYTES) else { return };
                self.renaming = Some(row.project);
                self.rename_words = words.into_bytes();
                self.clear_refusal(Form::Rename);
            }
            Act::RenameWords { words } if self.renaming.is_some() && self.pending() != Some(Form::Rename) => {
                if let Ok(words) = c::Text::new(words, NAME_BYTES) {
                    self.rename_words = words.into_bytes();
                    self.clear_refusal(Form::Rename);
                }
            }
            Act::NewWords { words } if self.pending() != Some(Form::New) => {
                if let Ok(words) = c::Text::new(words, NAME_BYTES) {
                    self.new_words = words.into_bytes();
                    self.clear_refusal(Form::New);
                }
            }
            Act::Cancel if self.pending() != Some(Form::Rename) => self.close(),
            Act::Save if self.pending.is_none() && self.renaming == project => {
                let Some(row) = owned(me, self.renaming) else { return };
                let words = self.rename_words.trim_ascii();
                if words.is_empty() || words == row.name.bytes() {
                    self.close();
                    return;
                }
                let name = c::Text::new(words.into(), NAME_BYTES).expect("bounded rename words");
                self.send(c::Ask::Rename { project: row.project, name }, Form::Rename, serial, out);
            }
            Act::Create if self.pending.is_none() && me.is_some() && !self.new_words.is_empty() => {
                let name = c::Text::new(self.new_words.clone(), NAME_BYTES).expect("bounded new project words");
                self.send(c::Ask::MakeProject { name, policy: 1 }, Form::New, serial, out);
            }
            Act::Rename | Act::RenameWords { .. } | Act::Save | Act::Cancel | Act::NewWords { .. } | Act::Create => {}
        }
    }
    fn send(&mut self, ask: c::Ask, form: Form, serial: &mut u64, out: &mut Queue<Request>) {
        *serial = serial.checked_add(1).expect("page keys fit");
        let key = Key { asker: Owner::Pages.asker(), number: *serial };
        self.pending = Some((key, form));
        self.refusal = None;
        out.push(Request::Client(jig_client::Event::Send {
            key,
            name: None,
            request: jig_client::Ask::Jig(Box::new(ask)),
        }));
    }
    pub(super) fn told(&mut self, told: &Told) -> bool {
        let Some((pending, form)) = self.pending else { return false };
        match told {
            Told::Pending { key } if *key == pending => true,
            Told::Answered { key, outcome } if *key == pending => {
                self.pending = None;
                match (form, outcome) {
                    (
                        Form::Rename,
                        Outcome::Answered(jig_client::Answered::Jig(c::Answer::Done(c::Outcome::Document(
                            c::DocumentOutcome::Renamed { .. },
                        )))),
                    ) => self.close(),
                    (
                        Form::New,
                        Outcome::Answered(jig_client::Answered::Jig(c::Answer::Done(c::Outcome::Document(
                            c::DocumentOutcome::ProjectMade { project },
                        )))),
                    ) => {
                        self.new_words = Box::default();
                        self.opening = Some(*project);
                    }
                    _ => {
                        if form != Form::Rename || self.renaming.is_some() {
                            self.refusal = Some((form, outcome.clone()));
                        }
                    }
                }
                true
            }
            Told::Link { .. }
            | Told::Pending { .. }
            | Told::Answered { .. }
            | Told::Kept { .. }
            | Told::Dropped { .. }
            | Told::Changed { .. }
            | Told::Text { .. }
            | Told::Ended { .. }
            | Told::Page { .. } => false,
        }
    }
}
#[expect(clippy::manual_find, reason = "the programming model uses a bounded scan without closures")]
fn owned(me: Option<&c::Me>, project: Option<u32>) -> Option<&c::MeProject> {
    let project = project?;
    for row in &me?.projects {
        if row.project == project && row.role == c::HeldRole::Owner {
            return Some(row);
        }
    }
    None
}
