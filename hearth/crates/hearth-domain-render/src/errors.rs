//! Decode errors retain their revision places (hearth/docs/design/domain/render/errors.md, 1).
use crate::revision::{self, Position};
pub use crate::vocabulary::Class;
use alloc::boxed::Box;
pub use hearth_domain_render_vocabulary::errors::{
    Cause, Collision, Command, Composition, Declared, Decode, Ended, Error, Expected, ExportFault, Exported, Field,
    Format, Guard, Host, Id, Ids, Kind, Limit, Object, Owns, Place, Repository, Required, Resolution, Root, Schema,
    Step, StoreError, StoreFault, StoreLimit, Subject, Takes, Tool, TypeName, Why,
};

pub(crate) fn refused(path: &[u8], position: Option<Position>, fault: Decode, also: Option<Position>) -> Error {
    let task = match revision::class(path) {
        revision::Class::Declaration { task }
        | revision::Class::Tree { task, .. }
        | revision::Class::Template { task, .. }
        | revision::Class::Script { task, .. }
        | revision::Class::Readme { task }
        | revision::Class::Refused { task: Some(task), .. } => Some(skein_lib::bytes::copy_of(task)),
        revision::Class::Fleet
        | revision::Class::Environment { .. }
        | revision::Class::Host { .. }
        | revision::Class::Refused { task: None, .. }
        | revision::Class::Other => None,
    };
    Error {
        side: None,
        kind: Kind::Decode(fault),
        class: Class::Configuration,
        at: Some(Place { path: skein_lib::bytes::copy_of(path), position }),
        also: second(path, also),
        target: None,
        task,
    }
}
fn second(path: &[u8], position: Option<Position>) -> Option<Place> {
    Some(Place { path: skein_lib::bytes::copy_of(path), position: Some(position?) })
}
mod held;

/// The key of the first error excluded from the ordered prefix (7.1).
#[derive(Debug, Hash)]
struct Order {
    unplaced: bool,
    path: Box<[u8]>,
    position: (u32, u32),
    target: Option<Box<[u8]>>,
}
/// The longest ordered prefix within both of hearth/docs/design/domain/render/errors.md, 1's bounds.
#[derive(Debug, Hash)]
pub struct Collector {
    kept: skein_lib::List<Error>,
    cut: Option<Order>,
    bytes: u64,
    left_out: u32,
    outcome: crate::vocabulary::Outcome,
    side: Option<crate::diff::Side>,
}
impl Collector {
    #[must_use]
    pub fn new(limits: &crate::limits::Limits) -> Self {
        Self {
            kept: skein_lib::List::with_capacity(limits.errors),
            cut: None,
            bytes: limits.error_bytes,
            left_out: 0,
            outcome: crate::vocabulary::Outcome::Done,
            side: None,
        }
    }
    pub(crate) fn side(&mut self, side: crate::diff::Side) {
        self.side = Some(side);
    }
    /// Counts every class, including errors beyond either prefix bound.
    pub fn push(&mut self, mut error: Error) {
        use crate::vocabulary::Outcome;
        error.side = self.side;
        self.outcome = self.outcome.worst(match error.class {
            Class::Configuration => Outcome::Failed,
            Class::Operational => Outcome::TryAgain,
        });
        if let Some(cut) = &self.cut
            && held::past(&error, cut)
        {
            self.left_out = self.left_out.saturating_add(1);
            return;
        }
        // Insertion is stable: equal places retain their arrival order.
        for index in 0..self.kept.len() {
            let kept = self.kept.get_mut(index).expect("kept index");
            if held::compare(&error, kept).is_lt() {
                core::mem::swap(&mut error, kept);
            }
        }
        if let Err(error) = self.kept.push(error) {
            self.cut = Some(held::order(&error));
            self.left_out = self.left_out.saturating_add(1);
        }
        let mut bytes = 0_u64;
        let mut count = 0_u32;
        for error in &self.kept {
            bytes = bytes.saturating_add(held::bytes(error));
            if bytes > self.bytes {
                self.cut = Some(held::order(error));
                break;
            }
            count = count.saturating_add(1);
        }
        self.left_out = self.left_out.saturating_add(self.kept.len().saturating_sub(count));
        self.kept.truncate(count);
    }
    pub(crate) fn extend(&mut self, errors: Box<[Error]>) {
        for error in errors {
            self.push(error);
        }
    }
    #[must_use]
    pub fn finish(self) -> (Box<[Error]>, u32) {
        (self.kept.into_boxed(), self.left_out)
    }
    #[cfg(test)]
    pub(crate) fn errors(&self) -> Box<[Error]> {
        self.kept.to_boxed()
    }
    pub(crate) fn get(&self, index: u32) -> Option<&Error> {
        self.kept.get(index)
    }
    pub(crate) const fn left_out(&self) -> u32 {
        self.left_out
    }
    pub(crate) const fn outcome(&self) -> crate::vocabulary::Outcome {
        self.outcome
    }
}
pub(crate) fn push(errors: &mut Collector, error: Error) {
    errors.push(error);
}

pub(crate) fn repository(path: &[u8], task: Option<&[u8]>, why: Repository) -> Error {
    repository_at(Place { path: skein_lib::bytes::copy_of(path), position: None }, task, why)
}

pub(crate) fn repository_at(at: Place, task: Option<&[u8]>, why: Repository) -> Error {
    Error {
        side: None,
        kind: Kind::Repository(why),
        class: Class::Configuration,
        at: Some(at),
        also: None,
        target: None,
        task: copy_task(task),
    }
}

fn copy_task(task: Option<&[u8]>) -> Option<Box<[u8]>> {
    Some(skein_lib::bytes::copy_of(task?))
}

pub mod syntax;
pub use syntax::Syntax;

mod reserved;
pub use reserved::{Reserved, Rule};

pub(crate) fn target_limit(target: &[u8], task: Option<&[u8]>) -> Error {
    let mut scope = None;
    if let Some(task) = task {
        scope = Some(skein_lib::bytes::copy_of(task));
    }
    Error {
        side: None,
        kind: Kind::Evaluation(crate::evaluate::FaultKind::Limit),
        class: Class::Configuration,
        at: None,
        also: None,
        target: Some(skein_lib::bytes::copy_of(target)),
        task: scope,
    }
}
