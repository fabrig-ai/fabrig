//! Defaulted declarations must agree field for field, without their provenance.
mod rows;
use super::Declared;
use crate::{
    entries::Claimant,
    errors::{Class, Collector, Collision, Error, Kind, Subject},
    kinds::{
        cmdline::Parameter,
        groups::Group,
        users::{Accounts, User},
    },
    limits::Limits,
    parts::{
        config::{Preset, Tmpfile},
        packages::Preseed,
    },
};
use alloc::boxed::Box;
use rows::Row;
use skein_lib::{List, Map, bytes::copy_of};
/// One agreeing declaration per key, ready for the parts and fixed-id pass.
#[derive(Debug)]
pub struct Survivors {
    pub accounts: Accounts,
    pub presets: Box<[Preset]>,
    pub tmpfiles: Box<[Tmpfile]>,
    pub preseeds: Box<[Preseed]>,
    pub parameters: Box<[Parameter]>,
}
type Kept<'a> = Map<(Subject, Box<[u8]>), (&'a Row, &'a Claimant)>;
type Key = (Box<[u8]>, Box<[u8]>, Subject, u32);
pub(super) struct Rows {
    rows: Map<Key, Row>,
    valid: bool,
}
impl Rows {
    fn new(limits: &Limits) -> Self {
        Self { rows: Map::with_capacity(limits.expansion), valid: true }
    }
    fn insert(&mut self, key: &[u8], row: Row, target: &[u8], errors: &mut Collector) {
        let key = (row.source().task.clone(), copy_of(key), row.subject(), self.rows.len());
        if let Err((_key, row)) = self.rows.insert(key, row) {
            self.valid = false;
            errors.push(error(target, Kind::Evaluation(crate::evaluate::FaultKind::Limit), row.source(), None));
        }
    }
}
/// Returns the survivors only when every declaration agrees and fits the bound.
#[must_use]
pub fn declarations(declared: &Declared<'_>, limits: &Limits, errors: &mut Collector) -> Option<Survivors> {
    let mut rows = Rows::new(limits);
    rows::gather(&mut rows, declared, errors);
    let mut kept: Kept<'_> = Map::with_capacity(limits.expansion);
    for ((_, name, subject, _), row) in &rows.rows {
        let who = row.source();
        let key = (*subject, name.clone());
        if let Some((first, at)) = kept.get_mut(&key) {
            if !first.same(row) {
                errors.push(error(
                    declared.target,
                    Kind::Collision(Collision::Differs {
                        subject: *subject,
                        name: name.clone(),
                        first: at.task.clone(),
                        second: who.task.clone(),
                    }),
                    at,
                    Some(who),
                ));
                rows.valid = false;
            } else if !first.explicit() && row.explicit() {
                // Keep the first claimant for errors, but retain the explicit declaration for the part.
                *first = row;
            }
        } else {
            kept.insert(key, (row, who)).expect("gathered bound");
        }
    }
    if rows.valid { Some(finish(&kept, limits)) } else { None }
}
fn finish(kept: &Kept<'_>, limits: &Limits) -> Survivors {
    let mut users: List<User> = List::with_capacity(kept.len());
    let mut groups: List<Group> = List::with_capacity(kept.len());
    let mut presets = List::with_capacity(kept.len());
    let mut tmpfiles = List::with_capacity(kept.len());
    let mut preseeds = List::with_capacity(kept.len());
    let mut parameters = Map::with_capacity(limits.expansion);
    for ((_, name), (row, _)) in kept {
        match row {
            Row::User(user, _) => users.push(user.clone()).expect("kept bound"),
            Row::Group(group, true) => groups.push(group.clone()).expect("kept bound"),
            Row::Group(_, false) => {}
            Row::Preset(row, _) => presets.push(row.clone()).expect("kept bound"),
            Row::Tmpfile(row, _) => tmpfiles.push(row.clone()).expect("kept bound"),
            Row::Preseed(row, _) => preseeds.push(row.clone()).expect("kept bound"),
            Row::Parameter(row, _) => {
                parameters.insert((row.order, name.clone()), row).expect("kept bound");
            }
        }
    }
    let mut ordered = List::with_capacity(parameters.len());
    for (_, row) in &parameters {
        ordered.push(Parameter { order: row.order, param: row.param.clone() }).expect("kept bound");
    }
    Survivors {
        accounts: Accounts { users: users.into_boxed(), groups: groups.into_boxed() },
        presets: presets.into_boxed(),
        tmpfiles: tmpfiles.into_boxed(),
        preseeds: preseeds.into_boxed(),
        parameters: ordered.into_boxed(),
    }
}
fn error(target: &[u8], kind: Kind, at: &Claimant, also: Option<&Claimant>) -> Error {
    let mut error = Error {
        side: None,
        kind,
        class: Class::Configuration,
        at: Some(at.place()),
        also: None,
        target: Some(copy_of(target)),
        task: None,
    };
    if let Some(also) = also {
        error.also = Some(also.place());
    }
    error
}
