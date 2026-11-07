//! Model switches ask the newest catalogue (temper/docs/design/client/README.md, 3.5, The `Models` tab).
use super::{Pages, Request, accounts, choice};
use crate::{Address, owner::Owner};
use alloc::boxed::Box;
use jig_client::{Client, Key, Outcome, Told, contract as c};
use skein_lib::Queue;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Act {
    Picker { number: u32, on: bool },
}
#[derive(Debug)]
struct Pending {
    key: Key,
    number: u32,
}
#[derive(Debug)]
pub struct Models {
    pending: Option<Pending>,
    refused: Option<(u32, Outcome)>,
}
impl Models {
    pub(super) const fn new() -> Self {
        Self { pending: None, refused: None }
    }
    #[must_use]
    pub fn refused_at(&self, number: u32) -> Option<&Outcome> {
        let (row, outcome) = self.refused.as_ref()?;
        if *row == number { Some(outcome) } else { None }
    }
    #[must_use]
    pub fn pending(&self, number: u32) -> bool {
        match &self.pending {
            Some(pending) => pending.number == number,
            None => false,
        }
    }
    pub(super) fn act(
        &mut self,
        client: &Client,
        address: &Address,
        act: Act,
        serial: &mut u64,
        out: &mut Queue<Request>,
    ) -> bool {
        if self.pending.is_some() {
            return false;
        }
        let Address::Models { project } = address else { return false };
        let Some(me) = accounts::current_me(client) else { return false };
        if me.deployment_role != Some(c::HeldRole::Owner) {
            return false;
        }
        let Some(models) = choice::offer(me, *project) else { return false };
        let Act::Picker { number, on } = act;
        let mut offered = false;
        for model in models {
            offered |= model.number == number && model.picker != on;
        }
        if !offered {
            return false;
        }
        *serial = serial.checked_add(1).expect("page keys fit");
        let key = Key { asker: Owner::Pages.asker(), number: *serial };
        self.pending = Some(Pending { key, number });
        self.refused = None;
        out.push(Request::Client(jig_client::Event::Send {
            key,
            name: None,
            request: jig_client::Ask::Jig(Box::new(c::Ask::ChangeModel(c::ChangeModel {
                act: c::ModelAct::Picker { number, on },
            }))),
        }));
        true
    }
    pub(super) fn told(&mut self, client: &Client, told: &Told) -> bool {
        let Some(pending) = &self.pending else { return false };
        let outcome = match told {
            Told::Answered { key, outcome } if *key == pending.key => outcome,
            Told::Pending { key } if *key == pending.key => {
                if let Some(request) = client.pending(*key)
                    && request.progress == jig_client::Progress::Unknown
                {
                    &Outcome::Unknown
                } else {
                    return false;
                }
            }
            Told::Answered { .. }
            | Told::Pending { .. }
            | Told::Changed { .. }
            | Told::Ended { .. }
            | Told::Link { .. }
            | Told::Kept { .. }
            | Told::Dropped { .. }
            | Told::Page { .. }
            | Told::Text { .. } => return false,
        };
        if !super::seen_answered(outcome) {
            self.refused = Some((pending.number, outcome.clone()));
        }
        self.pending = None;
        true
    }
}
impl Pages {
    #[must_use]
    pub const fn models(&self) -> &Models {
        &self.models
    }
}
