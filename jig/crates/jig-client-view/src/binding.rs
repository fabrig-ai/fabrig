//! Owner tokens carried unchanged by the tree (client/domain/view.md, section 5).

use crate::Owner;
use alloc::boxed::Box;
use skein_lib::Token;
use skein_ui::{Act, Binding, number};

/// A typed meaning belonging to jig's last built tree.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum JigMeaning {
    /// One typed Project settings field, preserving other whole channels.
    ProjectSetting { ask: Box<jig_client::contract::Ask>, field: crate::tree::sections::project::owner::Field },
    /// A new service name, after an explicit supplied role choice.
    ProjectServiceName { project: u32, role: jig_client::contract::HeldRole },
    /// An explicit budget for one supplied party in the configured target.
    ProjectPoolBudget { project: u32, party: u64 },
    /// Pressing, submitting or toggling asks this already typed action.
    Action(crate::state::action::Act),
    /// Whole text edits a domain field; submit names that same field.
    Field(crate::state::fields::Purpose),
    /// Digits are kept in the view and submitting goes to that task.
    TaskNumber,
    /// Digits edit the confirmation's priority without a hidden submit.
    Priority,
    /// Source values and selected replacements kept separately for one named field.
    Amendment {
        before: Box<jig_client::contract::Amendment>,
        change: Box<jig_client::contract::Amendment>,
        field: crate::tree::confirmation::amendment::Field,
    },
    /// A typed project goal form field.
    GoalField {
        form: crate::state::page::project::goals::GoalForm,
        field: crate::tree::sections::project::goals::Field,
    },
    /// Description search changes the current note scope only.
    NoteSearch { filter: crate::state::page::project::NoteFilter },
    /// A connector-owned part referred to by a slot, never an actionable node.
    ApplicationPart(jig_client::contract::ApplicationPart),
    /// A whole exact owner reference, with literal path segment boundaries.
    Part(jig_client::contract::Part),
}
#[derive(Debug)]
struct Entry {
    token: Token,
    meaning: JigMeaning,
}
/// Bounded meanings for one tree; rebuilding retires every old token.
/// Empty + begin -> building; insert -> issued; decode -> typed action or drop.
/// The next token never resets, so a token from any earlier tree stays stale.
#[derive(Debug)]
pub struct JigBindings {
    entries: skein_lib::List<Entry>,
    next: u64,
    text_limit: u32,
    task: Option<u64>,
}
impl JigBindings {
    /// Allocate the most meanings that a tree may carry.
    #[must_use]
    pub fn new(capacity: u32, text_limit: u32) -> Self {
        Self { entries: skein_lib::List::with_capacity(capacity), next: 0, text_limit, task: None }
    }
    /// The meanings carried by the last built tree, slot meanings included.
    #[must_use]
    #[expect(clippy::len_without_is_empty, reason = "the read-only count prices the binding table")]
    pub fn len(&self) -> u32 {
        self.entries.len()
    }
    /// The client's bound on one literal segment's bytes.
    #[must_use]
    pub const fn text_limit(&self) -> u32 {
        self.text_limit
    }
    /// Retire the prior tree's meanings before building its replacement.
    pub fn begin(&mut self) {
        self.entries.clear();
    }
    /// Issue one token; exceeding the derived capacity is a view bug.
    pub fn insert(&mut self, meaning: JigMeaning) -> Binding {
        self.next = self.next.checked_add(1).expect("view token space remains");
        let token = Token::new(self.next);
        self.entries.push(Entry { token, meaning }).expect("bindings fit their derived capacity");
        Binding { owner: Owner::Jig.opaque(), token }
    }
    /// Resolve a part slot's token for its connector's view root.
    #[must_use]
    pub fn part(&self, token: Token) -> Option<&jig_client::contract::ApplicationPart> {
        for entry in &self.entries {
            if entry.token == token {
                match &entry.meaning {
                    JigMeaning::ApplicationPart(part) => return Some(part),
                    JigMeaning::Part(_)
                    | JigMeaning::Action(_)
                    | JigMeaning::Field(_)
                    | JigMeaning::TaskNumber
                    | JigMeaning::Priority
                    | JigMeaning::Amendment { .. }
                    | JigMeaning::ProjectSetting { .. }
                    | JigMeaning::ProjectServiceName { .. }
                    | JigMeaning::ProjectPoolBudget { .. }
                    | JigMeaning::GoalField { .. }
                    | JigMeaning::NoteSearch { .. } => return None,
                }
            }
        }
        None
    }
    /// Resolve an exact reference slot's raw key; scope is supplied by its producer.
    #[must_use]
    pub fn owner_part(&self, token: Token) -> Option<&jig_client::contract::Part> {
        for entry in &self.entries {
            if entry.token == token {
                match &entry.meaning {
                    JigMeaning::Part(part) => return Some(part),
                    JigMeaning::ApplicationPart(_)
                    | JigMeaning::Action(_)
                    | JigMeaning::Field(_)
                    | JigMeaning::TaskNumber
                    | JigMeaning::Priority
                    | JigMeaning::Amendment { .. }
                    | JigMeaning::ProjectSetting { .. }
                    | JigMeaning::ProjectServiceName { .. }
                    | JigMeaning::ProjectPoolBudget { .. }
                    | JigMeaning::GoalField { .. }
                    | JigMeaning::NoteSearch { .. } => return None,
                }
            }
        }
        None
    }
    /// A link's typed address for the platform's address translation.
    #[must_use]
    pub fn address(&self, token: Token) -> Option<crate::state::address::Address> {
        for entry in &self.entries {
            if entry.token == token {
                match &entry.meaning {
                    JigMeaning::Action(action) => match action {
                        crate::state::action::Act::Go { address } => return Some(*address),
                        crate::state::action::Act::OpenChat { task } => {
                            return Some(crate::state::address::Address::Task {
                                task: *task,
                                section: crate::state::address::TaskSection::Activity,
                            });
                        }
                        crate::state::action::Act::Edit { .. }
                        | crate::state::action::Act::Submit { .. }
                        | crate::state::action::Act::AmendRelease { .. }
                        | crate::state::action::Act::CloseChat { .. }
                        | crate::state::action::Act::Intend { .. }
                        | crate::state::action::Act::Choose { .. }
                        | crate::state::action::Act::CannotDo
                        | crate::state::action::Act::Again { .. }
                        | crate::state::action::Act::Dispose { .. }
                        | crate::state::action::Act::Change { .. }
                        | crate::state::action::Act::WriteRelease { .. }
                        | crate::state::action::Act::Priority { .. }
                        | crate::state::action::Act::MoveGoal { .. }
                        | crate::state::action::Act::ProposeInstead { .. }
                        | crate::state::action::Act::ChatAbout { .. }
                        | crate::state::action::Act::LookAgain { .. }
                        | crate::state::action::Act::Confirm
                        | crate::state::action::Act::Dismiss
                        | crate::state::action::Act::Fold { .. }
                        | crate::state::action::Act::FoldCall { .. }
                        | crate::state::action::Act::FoldCurrentCall { .. }
                        | crate::state::action::Act::Filter { .. }
                        | crate::state::action::Act::More { .. }
                        | crate::state::action::Act::FilterInbox { .. }
                        | crate::state::action::Act::OpenUpdates
                        | crate::state::action::Act::BeginConversation { .. }
                        | crate::state::action::Act::ProjectSettings { .. }
                        | crate::state::action::Act::ProjectServiceRole { .. }
                        | crate::state::action::Act::GoalForm { .. }
                        | crate::state::action::Act::FilterTasks { .. }
                        | crate::state::action::Act::FilterNotes { .. }
                        | crate::state::action::Act::FilterPlan { .. }
                        | crate::state::action::Act::HistoryScope { .. }
                        | crate::state::action::Act::OpenEnded { .. }
                        | crate::state::action::Act::SignOut => return None,
                    },
                    JigMeaning::Field(_)
                    | JigMeaning::TaskNumber
                    | JigMeaning::ApplicationPart(_)
                    | JigMeaning::Part(_)
                    | JigMeaning::Priority
                    | JigMeaning::Amendment { .. }
                    | JigMeaning::ProjectSetting { .. }
                    | JigMeaning::ProjectServiceName { .. }
                    | JigMeaning::ProjectPoolBudget { .. }
                    | JigMeaning::GoalField { .. }
                    | JigMeaning::NoteSearch { .. } => return None,
                }
            }
        }
        None
    }
}
/// Decode an act only through a token issued for the last tree.
/// Numeric input is retained here until submit; no navigation happens while typing.
#[must_use]
pub fn decode(view: &mut JigBindings, token: Token, act: Act) -> Option<crate::state::action::Act> {
    for entry in &view.entries {
        if entry.token == token {
            return match &entry.meaning {
                JigMeaning::ApplicationPart(_) | JigMeaning::Part(_) => None,
                JigMeaning::ProjectPoolBudget { project, party } => match act {
                    Act::Input { value } => {
                        Some(crate::state::page::project::owner_form::action(jig_client::contract::Ask::SetPool {
                            project: *project,
                            party: *party,
                            budget: number(&value)?,
                        }))
                    }
                    Act::Press | Act::Submit | Act::Toggle | Act::Drawing { .. } | Act::Near { .. } => None,
                },
                JigMeaning::ProjectServiceName { project, role } => match act {
                    Act::Input { value } => {
                        Some(crate::state::page::project::owner_form::action(jig_client::contract::Ask::MakeService {
                            project: *project,
                            role: *role,
                            name: jig_client::contract::Text::new(value, view.text_limit).ok()?,
                        }))
                    }
                    Act::Press | Act::Submit | Act::Toggle | Act::Drawing { .. } | Act::Near { .. } => None,
                },
                JigMeaning::ProjectSetting { ask, field } => match act {
                    Act::Input { value } => {
                        crate::tree::sections::project::owner::edit(ask, *field, value, view.text_limit)
                    }
                    Act::Press | Act::Submit | Act::Toggle | Act::Drawing { .. } | Act::Near { .. } => None,
                },
                JigMeaning::GoalField { form, field } => match act {
                    Act::Input { value } => crate::tree::sections::project::goals::edit(*form, *field, &value),
                    Act::Press | Act::Submit | Act::Toggle | Act::Drawing { .. } | Act::Near { .. } => None,
                },
                JigMeaning::NoteSearch { filter } => match act {
                    Act::Input { value } => Some(crate::state::action::Act::FilterNotes {
                        filter: crate::state::page::project::NoteFilter {
                            scope: filter.scope.clone(),
                            description: if value.is_empty() {
                                None
                            } else {
                                Some(jig_client::contract::Text::new(value, view.text_limit).ok()?)
                            },
                        },
                    }),
                    Act::Press | Act::Submit | Act::Toggle | Act::Drawing { .. } | Act::Near { .. } => None,
                },
                JigMeaning::Priority => match act {
                    Act::Input { value } => {
                        Some(crate::state::action::Act::Priority { priority: u32::try_from(number(&value)?).ok()? })
                    }
                    Act::Press | Act::Submit | Act::Toggle | Act::Drawing { .. } | Act::Near { .. } => None,
                },
                JigMeaning::Amendment { before, change, field } => match act {
                    Act::Input { value } => {
                        crate::tree::confirmation::amendment::edit(before, change, *field, value, view.text_limit)
                    }
                    Act::Press | Act::Submit | Act::Toggle | Act::Drawing { .. } | Act::Near { .. } => None,
                },
                JigMeaning::Action(action) => match act {
                    Act::Press | Act::Submit | Act::Toggle => Some(action.clone()),
                    Act::Input { .. } | Act::Drawing { .. } | Act::Near { .. } => None,
                },
                JigMeaning::Field(field) => match act {
                    Act::Input { value } => {
                        let text = jig_client::contract::Text::new(value, view.text_limit).ok()?;
                        Some(crate::state::action::Act::Edit { field: *field, text })
                    }
                    Act::Submit => Some(crate::state::action::Act::Submit { field: *field }),
                    Act::Press | Act::Toggle | Act::Drawing { .. } | Act::Near { .. } => None,
                },
                JigMeaning::TaskNumber => match act {
                    Act::Input { value } => {
                        view.task = number(&value);
                        None
                    }
                    Act::Submit => Some(crate::state::action::Act::Go {
                        address: crate::state::address::Address::task(view.task?),
                    }),
                    Act::Press | Act::Toggle | Act::Drawing { .. } | Act::Near { .. } => None,
                },
            };
        }
    }
    None
}
