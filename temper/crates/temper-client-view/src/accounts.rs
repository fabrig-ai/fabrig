//! Accounts bindings contain row targets and operation names, never a key value.
pub(crate) use crate::words::accounts as words;
use alloc::boxed::Box;
use skein_lib::{List, Secret, Token};
use skein_ui::{Act, Binding};
use temper_client_domain::{
    Action, Event,
    pages::{self, accounts::Act as AccountAct},
};
pub(crate) use words::owner;
#[derive(Clone, Debug)]
pub(crate) enum Meaning {
    Copy { words: Box<[u8]> },
    Action(AccountAct),
    Model(pages::connect_model::Act),
    Try { account: u32 },
    Go(temper_client_domain::Address),
    Key,
    SignInAddress,
    Program,
}
#[derive(Debug)]
pub(crate) struct Bindings {
    entries: List<(Token, Meaning)>,
    next: u64,
    revision: u64,
    counts: jig_client::AccountCounts,
    row_uses: u32,
}
impl Bindings {
    pub fn new(counts: jig_client::AccountCounts, row_uses: u32, providers: u32) -> Self {
        let capacity = binding_capacity(&counts, providers);
        Self { entries: List::with_capacity(capacity), next: 0, revision: 0, counts, row_uses }
    }
    pub const fn counts(&self) -> jig_client::AccountCounts {
        self.counts
    }
    pub const fn row_uses(&self) -> u32 {
        self.row_uses
    }
    pub const fn revision(&self) -> u64 {
        self.revision
    }
    pub fn begin(&mut self, _reset: bool) {
        self.entries.clear();
    }
    pub fn insert(&mut self, meaning: Meaning) -> Option<Binding> {
        self.next = self.next.checked_add(1)?;
        let token = Token::new(self.next);
        self.entries.push((token, meaning)).ok()?;
        Some(Binding { owner: owner(), token })
    }
    pub fn decode(&self, token: Token, act: Act) -> Option<Event> {
        for (key, meaning) in &self.entries {
            if *key != token {
                continue;
            }
            match (meaning, &act) {
                (Meaning::Try { account }, Act::Press) => {
                    return Some(Event::Act {
                        action: Action::Forge(temper_client_domain::forge::Act::TryAccount {
                            account: *account,
                            again: true,
                        }),
                    });
                }
                (Meaning::Copy { words }, Act::Press) => {
                    return Some(Event::Act { action: Action::Copy { words: words.clone() } });
                }
                (Meaning::Model(action), Act::Press) => {
                    return Some(Event::Act { action: Action::Pages(pages::Act::ConnectModel(action.clone())) });
                }
                (Meaning::Go(address), Act::Press) => {
                    return Some(Event::Act { action: Action::Pages(pages::Act::Go { address: address.clone() }) });
                }
                (Meaning::Program, Act::Input { value }) => {
                    return Some(Event::Act {
                        action: Action::Pages(pages::Act::ConnectModel(pages::connect_model::Act::ProgramName {
                            value: value.clone(),
                        })),
                    });
                }
                (Meaning::Try { .. } | Meaning::Copy { .. } | Meaning::Model(_) | Meaning::Go(_), _) => return None,
                _ => {}
            }
            let action = match (meaning, act) {
                (Meaning::SignInAddress, Act::Input { value }) => {
                    return Some(Event::SignInAddress { value: Secret::new(value) });
                }
                (Meaning::Key, Act::Input { value }) => return Some(Event::AccountKey { value: Secret::new(value) }),
                (Meaning::Action(action), Act::Press) => action.clone(),
                (_, Act::Submit | Act::Toggle | Act::Drawing { .. } | Act::Near { .. })
                | (
                    Meaning::Action(_)
                    | Meaning::Try { .. }
                    | Meaning::Copy { .. }
                    | Meaning::Model(_)
                    | Meaning::Go(_)
                    | Meaning::Program,
                    Act::Input { .. },
                )
                | (
                    Meaning::Key
                    | Meaning::SignInAddress
                    | Meaning::Program
                    | Meaning::Try { .. }
                    | Meaning::Copy { .. }
                    | Meaning::Model(_)
                    | Meaning::Go(_),
                    Act::Press,
                ) => return None,
            };
            return Some(Event::Act { action: Action::Pages(pages::Act::Accounts(action)) });
        }
        None
    }
}
/// Each of two account lists has source actions, state actions, ranks and one editor.
pub(crate) fn binding_capacity(counts: &jig_client::AccountCounts, providers: u32) -> u32 {
    let source = counts.sources.checked_mul(6).expect("source controls fit");
    let accounts = source
        .checked_add(16)
        .expect("row controls fit")
        .checked_mul(counts.accounts)
        .expect("account controls fit")
        .checked_mul(2)
        .expect("two holders fit");
    accounts
        .checked_add(providers.checked_mul(3).expect("ways fit"))
        .expect("controls fit")
        .checked_add(16)
        .expect("page controls fit")
}
