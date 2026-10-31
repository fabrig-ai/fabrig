//! One pending local command, applied between activations (shell.md, section 4.1).
//! It owns only the bounded entered value; the store and run keep their existing owners.
use crate::chat::State;
use crate::{Domain, Limits, Notice, Request};
use alloc::boxed::Box;
use skein_lib::{Env, Queue};

/// A person's local command, applied from the next activation.
#[derive(Debug, PartialEq, Eq)]
pub enum Slash {
    /// Select a main model by its endpoint and model name.
    Model {
        model: Box<[u8]>,
        /// Position in the startup model inventory, restricted to main's endpoint.
        selected: Option<usize>,
    },
    /// Select the main model's reasoning effort.
    Effort { level: Box<[u8]> },
    /// Start a new named chat instead of resuming the current one.
    New,
    /// Reopen a chat by its single directory name.
    Resume { name: Box<[u8]> },
    /// End input while allowing received lines to drain.
    Quit,
}

/// Typed local choice feedback, rendered by the terminal protocol.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ChoiceNotice {
    /// A command is already held until the current activation ends.
    Pending,
    /// A model belongs to another startup endpoint inventory.
    ModelElsewhere { model: Box<[u8]> },
    /// An effort is refused or cleared when the main model changes.
    Effort { effort: Box<[u8]>, choices: Box<[Box<[u8]>]>, dropped: bool },
}

pub(crate) fn receive(domain: &mut Domain, choice: Slash, env: &Env<Limits>, out: &mut Queue<Request>) {
    match choice {
        Slash::Quit => crate::domain::closed(domain, out),
        choice @ (Slash::Model { .. } | Slash::Effort { .. } | Slash::New | Slash::Resume { .. }) => {
            if domain.choice.is_some() {
                out.push(Request::Notice { notice: Notice::Choice(ChoiceNotice::Pending) });
                return;
            }
            domain.choice = Some(choice);
            apply(domain, env, out);
        }
    }
}

pub(crate) fn apply(domain: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    if domain.chat.phase != State::Idle || domain.run.is_some() || domain.chat.state_pending {
        return;
    }
    let Some(choice) = domain.choice.take() else { return };
    match choice {
        Slash::New => {
            let name = domain.new_chat_name();
            crate::domain::select_chat(domain, env, name, true, out);
        }
        Slash::Resume { name } => crate::domain::select_chat(domain, env, name, false, out),
        Slash::Quit => crate::domain::closed(domain, out),
        Slash::Model { model, selected } => match selected {
            Some(index) if main_endpoint(domain, index) => {
                domain.main_model = index;
                let efforts = domain.config.efforts.get(index).expect("selected startup efforts");
                if let Some(effort) = &domain.effort
                    && !smith_domain::ConfiguredModel::accepts_effort(efforts, Some(effort))
                {
                    let notice =
                        ChoiceNotice::Effort { effort: effort.clone(), choices: efforts.clone(), dropped: true };
                    domain.effort = None;
                    out.push(Request::Notice { notice: Notice::Choice(notice) });
                }
            }
            Some(_) | None => {
                out.push(Request::Notice { notice: Notice::Choice(ChoiceNotice::ModelElsewhere { model }) });
            }
        },
        Slash::Effort { level } => {
            let efforts = domain.config.efforts.get(domain.main_model).expect("selected startup efforts");
            if smith_domain::ConfiguredModel::accepts_effort(efforts, Some(&level)) {
                domain.effort = Some(level);
            } else {
                out.push(Request::Notice {
                    notice: Notice::Choice(ChoiceNotice::Effort {
                        effort: level,
                        choices: efforts.clone(),
                        dropped: false,
                    }),
                });
            }
        }
    }
}

fn main_endpoint(domain: &Domain, index: usize) -> bool {
    match domain.config.models.get(index) {
        Some(model) => model.endpoint == domain.config.models.first().expect("startup main model").endpoint,
        None => false,
    }
}
