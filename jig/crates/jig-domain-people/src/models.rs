//! A request's canonical original name context and tagged selectors (protocol/store/records.md, 1).
use crate::{Action, Amendment, Ask, Authority, Member, Operation, PolicyChange};
use alloc::boxed::Box;

use crate::MODELS;
pub const MODEL_NAME_BYTES: u32 = 4_096;
/// The most bytes in a configured model's description (protocol/limits.md).
pub const MODEL_DESCRIPTION_BYTES: u32 = 512;

/// A literal ALL or a frozen mask into its enclosing record's model-name context.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Models {
    All,
    Named(u64),
}
impl Models {
    pub const ALL: Self = Self::All;
    pub const NONE: Self = Self::Named(0);
    #[must_use]
    pub const fn bits(self) -> u64 {
        match self {
            Self::All => u64::MAX,
            Self::Named(bits) => bits,
        }
    }
    #[must_use]
    pub const fn is_all(self) -> bool {
        match self {
            Self::All => true,
            Self::Named(_) => false,
        }
    }
    #[must_use]
    pub const fn from_parts(bits: u64, all: bool) -> Self {
        if all { Self::All } else { Self::Named(bits) }
    }
    /// Reordering and removal affect effective authority, never a retained ask's identity.
    #[must_use]
    pub fn remap(self, moved: &[u64; 64]) -> Self {
        match self {
            Self::All => Self::All,
            Self::Named(bits) => {
                let mut result = 0;
                for (bit, today) in moved.iter().enumerate() {
                    if bits.checked_shr(u32::try_from(bit).expect("bounded bit")).unwrap_or(0) & 1 == 1 {
                        result |= today;
                    }
                }
                Self::Named(result)
            }
        }
    }
}

#[must_use]
/// Protocols validate UTF-8; the domain checks count, lengths and canonical byte order.
pub fn valid_model_names(names: &[Box<[u8]>]) -> bool {
    if names.len() > skein_lib::length::usize_of(MODELS) {
        return false;
    }
    for (at, name) in names.iter().enumerate() {
        if name.is_empty() || name.len() > skein_lib::length::usize_of(MODEL_NAME_BYTES) {
            return false;
        }
        for earlier in names.iter().take(at) {
            if earlier >= name {
                return false;
            }
        }
    }
    true
}

fn authority(value: &mut Authority, moved: &[u64; 64]) {
    if let Some(models) = value.models {
        value.models = Some(models.remap(moved));
    }
}
fn amendment(value: &mut Amendment, moved: &[u64; 64]) {
    if let Some(value) = &mut value.authority {
        authority(value, moved);
    }
}
fn members(values: &mut [Member], moved: &[u64; 64]) {
    for value in values {
        authority(&mut value.authority, moved);
    }
}

/// Exhaustive traversal of every authority-bearing request, shared by canonicalization and
/// fresh-request admission. Stored asks keep their original context and are never remapped.
pub fn remap_models(ask: &mut Ask, moved: &[u64; 64]) {
    match ask {
        Ask::Create { batch, .. } => members(batch, moved),
        Ask::Amend { amendment: value, .. } => amendment(value, moved),
        Ask::Propose { action, .. } => match action {
            Action::Batch(batch) => members(batch, moved),
            Action::Amend { amendment: value, .. } => amendment(value, moved),
            Action::Settings { .. } => {}
        },
        Ask::Compound { operations, .. } => {
            for at in 0..operations.len() {
                match operations.get_mut(at).expect("bounded operation") {
                    Operation::Amend { amendment: value, .. } => amendment(value, moved),
                    Operation::Cancel { .. }
                    | Operation::Move { .. }
                    | Operation::Release { .. }
                    | Operation::Message { .. }
                    | Operation::Stop { .. } => {}
                }
            }
        }
        Ask::ChangePolicy { change, .. } => match change {
            PolicyChange::Role(role) => authority(&mut role.authority, moved),
            PolicyChange::RoleSpend { .. }
            | PolicyChange::ProjectSpend { .. }
            | PolicyChange::Requirements { .. }
            | PolicyChange::Projections { .. }
            | PolicyChange::Permissions { .. }
            | PolicyChange::Templates { .. } => {}
        },
        Ask::RemoveProject { .. }
        | Ask::SetVisibility { .. }
        | Ask::Restore { .. }
        | Ask::Rename { .. }
        | Ask::MakeProject { .. }
        | Ask::ChangeSettings { .. }
        | Ask::MoveToNewest { .. }
        | Ask::MarkSeen { .. }
        | Ask::Application { .. }
        | Ask::EditNote { .. }
        | Ask::MakeService { .. }
        | Ask::Adopt { .. }
        | Ask::Withdraw { .. }
        | Ask::Stop { .. }
        | Ask::Cancel { .. }
        | Ask::Lift { .. }
        | Ask::Release { .. }
        | Ask::TakePerson { .. }
        | Ask::HandBackPerson { .. }
        | Ask::AnswerPerson { .. }
        | Ask::Move { .. }
        | Ask::DecideProposal { .. }
        | Ask::Talk { .. }
        | Ask::Leave { .. }
        | Ask::Say { .. }
        | Ask::AnswerQuestion { .. }
        | Ask::SetRoles { .. }
        | Ask::SetPool { .. }
        | Ask::DecideEscalation { .. }
        | Ask::AccountSignIn { .. }
        | Ask::AccountSignOut { .. }
        | Ask::AccountChange { .. } => {}
    }
}

/// Named references and explicit presence are independent: ALL and NONE both select explicitly.
struct Selectors {
    bits: u64,
    any: bool,
}

impl Selectors {
    fn authority(&mut self, value: &Authority) {
        match value.models {
            Some(Models::Named(bits)) => {
                self.bits |= bits;
                self.any = true;
            }
            Some(Models::All) => self.any = true,
            None => {}
        }
    }
    fn amendment(&mut self, value: &Amendment) {
        if let Some(authority) = &value.authority {
            self.authority(authority);
        }
    }
    fn members(&mut self, values: &[Member]) {
        for value in values {
            self.authority(&value.authority);
        }
    }
}

fn selectors(ask: &Ask) -> Selectors {
    let mut selected = Selectors { bits: 0, any: false };
    match ask {
        Ask::Create { batch, .. } => selected.members(batch),
        Ask::Amend { amendment, .. } => selected.amendment(amendment),
        Ask::Propose { action, .. } => match action {
            Action::Batch(batch) => selected.members(batch),
            Action::Amend { amendment, .. } => selected.amendment(amendment),
            Action::Settings { .. } => {}
        },
        Ask::Compound { operations, .. } => {
            for operation in operations {
                match operation {
                    Operation::Amend { amendment, .. } => selected.amendment(amendment),
                    Operation::Cancel { .. }
                    | Operation::Move { .. }
                    | Operation::Release { .. }
                    | Operation::Message { .. }
                    | Operation::Stop { .. } => {}
                }
            }
        }
        Ask::ChangePolicy { change, .. } => match change {
            PolicyChange::Role(role) => selected.authority(&role.authority),
            PolicyChange::RoleSpend { .. }
            | PolicyChange::ProjectSpend { .. }
            | PolicyChange::Requirements { .. }
            | PolicyChange::Projections { .. }
            | PolicyChange::Permissions { .. }
            | PolicyChange::Templates { .. } => {}
        },
        Ask::RemoveProject { .. }
        | Ask::SetVisibility { .. }
        | Ask::Restore { .. }
        | Ask::Rename { .. }
        | Ask::MakeProject { .. }
        | Ask::ChangeSettings { .. }
        | Ask::MoveToNewest { .. }
        | Ask::MarkSeen { .. }
        | Ask::Application { .. }
        | Ask::EditNote { .. }
        | Ask::MakeService { .. }
        | Ask::Adopt { .. }
        | Ask::Withdraw { .. }
        | Ask::Stop { .. }
        | Ask::Cancel { .. }
        | Ask::Lift { .. }
        | Ask::Release { .. }
        | Ask::TakePerson { .. }
        | Ask::HandBackPerson { .. }
        | Ask::AnswerPerson { .. }
        | Ask::Move { .. }
        | Ask::DecideProposal { .. }
        | Ask::Talk { .. }
        | Ask::Leave { .. }
        | Ask::Say { .. }
        | Ask::AnswerQuestion { .. }
        | Ask::SetRoles { .. }
        | Ask::SetPool { .. }
        | Ask::DecideEscalation { .. }
        | Ask::AccountSignIn { .. }
        | Ask::AccountSignOut { .. }
        | Ask::AccountChange { .. } => {}
    }
    selected
}

/// The original context contains exactly the names referenced by frozen selectors.
#[must_use]
pub fn valid_model_context(ask: &Ask, names: &[Box<[u8]>]) -> bool {
    if !valid_model_names(names) {
        return false;
    }
    let mask = u64::MAX
        .checked_shr(u32::try_from(64_usize.checked_sub(names.len()).expect("bounded context")).expect("bounded names"))
        .unwrap_or(0);
    selectors(ask).bits == mask
}

/// Application parts have no model-name carrier; explicit selectors need that capability.
pub(crate) fn has_model_selector(ask: &Ask) -> bool {
    selectors(ask).any
}
