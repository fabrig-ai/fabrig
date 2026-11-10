//! Kept forges and their navigation dot (temper/docs/design/client/forge/deployment.md; temper/docs/design/client/pages/README.md and temper/docs/design/client/forge/family.md, 4 and 7).
use super::{
    Playground, connect,
    forges::{Connection, Mode, Row, State},
};
use crate::{
    Domain, Step,
    pages::{accounts, welcome},
};
use skein_lib::List;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dot {
    None,
    Amber,
    Red,
}
#[must_use]
pub fn personal(row: &Row, playground: Option<u16>) -> bool {
    row.mode != Mode::Fake || playground == Some(row.number)
}
/// The decoder keeps these rows in increasing forge number order.
#[must_use]
pub fn rows(state: &State, playground: Option<u16>) -> List<&Row> {
    let mut listed = List::with_capacity(super::forges::ROWS);
    if let State::Rows(rows) = state {
        for row in rows {
            if personal(row, playground) {
                listed.push(row).expect("bounded forge rows");
            }
        }
    }
    listed
}
#[must_use]
pub fn dot(state: &State, playground: Option<u16>) -> Dot {
    let mut dot = Dot::None;
    for row in &rows(state, playground) {
        if row.ended.is_some() {
            return Dot::Red;
        }
        if connect::underway(row) {
            dot = Dot::Amber;
        }
        if let Some(first) = &row.first {
            match first.state {
                super::forges::FirstState::Failed { .. } => return Dot::Red,
                super::forges::FirstState::Making => dot = Dot::Amber,
                super::forges::FirstState::Made { .. } => {}
            }
        }
        match row.connection {
            Connection::SetUpFailed { .. } | Connection::Stopped { .. } => return Dot::Red,
            Connection::SettingUp { .. } | Connection::Connecting => dot = Dot::Amber,
            Connection::NotConnected | Connection::NotSetUp | Connection::Connected { .. } => {}
        }
    }
    dot
}
#[must_use]
pub fn playground(root: &Domain) -> Option<(&Playground, &Row)> {
    let playground = root.forge().playground()?;
    if let State::Rows(rows) = root.forges() {
        for row in rows {
            if row.number == playground.forge {
                return Some((playground, row));
            }
        }
    }
    None
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Setup {
    pub playground: bool,
    pub forge: bool,
    pub model: bool,
    pub step: Step,
}
#[must_use]
pub fn setup(root: &Domain) -> Option<Setup> {
    if !root.forge().playground_answered()
        || root.forge().playground_running()
        || match welcome::decision(root) {
            Some(welcome::FirstRun::Empty | welcome::FirstRun::Unfinished { .. }) => false,
            Some(welcome::FirstRun::Whole) | None => true,
        }
    {
        return None;
    }
    let owner = accounts::current_me(root.client())?.deployment_role == Some(jig_client::contract::HeldRole::Owner);
    let mut connected = false;
    let mut started = false;
    let mut personal_connected = false;
    for row in &rows(root.forges(), connect::playground(root.forge())) {
        if let Connection::Connected { .. } = row.connection {
            personal_connected = true;
            connected |= !connect::underway(row) && !connect::failed(row);
            started = true;
        }
        started |= connect::underway(row);
    }
    let model = !welcome::model_usable(root)?;
    let step = if owner && !started {
        Step::Forge { forge: None }
    } else if model {
        Step::Model { way: None }
    } else {
        Step::Ready
    };
    Some(Setup {
        playground: !personal_connected && root.forge().playground().is_some(),
        forge: owner && !connected,
        model,
        step,
    })
}

/// K4: Some(true) once the rows are delivered and none names `project`.
#[must_use]
pub fn without_repository(forges: &State, playground: Option<&Playground>, project: u32) -> Option<bool> {
    if let Some(playground) = playground
        && playground.project == Some(project)
    {
        return None;
    }
    let State::Rows(rows) = forges else { return None };
    for row in rows {
        for repository in &row.repositories {
            if repository.project == project {
                return Some(false);
            }
        }
    }
    Some(true)
}
/// 4.4: the first repository made and adopted nowhere, on the lowest-numbered real forge.
#[must_use]
pub fn unadopted_first(forges: &State) -> Option<(u16, &[u8], &[u8])> {
    let State::Rows(rows) = forges else { return None };
    let mut first = None;
    for row in rows {
        if row.mode == Mode::Fake {
            continue;
        }
        let Some(candidate) = &row.first else { continue };
        match candidate.state {
            super::forges::FirstState::Made { .. } => {}
            super::forges::FirstState::Making | super::forges::FirstState::Failed { .. } => continue,
        }
        let mut adopted = false;
        for other in rows {
            for repository in &other.repositories {
                adopted |= repository.owner == candidate.org && repository.name == candidate.repository;
            }
        }
        let lower = match first {
            Some((number, _, _)) => row.number < number,
            None => true,
        };
        if !adopted && lower {
            first = Some((row.number, candidate.org.as_ref(), candidate.repository.as_ref()));
        }
    }
    first
}

/// A project's repositories (temper/docs/design/client/view.md, 3).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Held<'a> {
    Unknown,
    None,
    One { forge: Option<&'a Row>, owner: &'a [u8], name: &'a [u8] },
    Several { forge: Option<&'a Row>, owner: &'a [u8], name: &'a [u8], more: u32 },
}
#[must_use]
pub fn held<'a>(forges: &'a State, playground: Option<&'a Playground>, project: u32) -> Held<'a> {
    if let Some(playground) = playground
        && playground.project == Some(project)
    {
        return Held::One { forge: None, owner: &playground.owner, name: &playground.name };
    }
    let State::Rows(rows) = forges else { return Held::Unknown };
    let mut first: Option<(&Row, &[u8], &[u8])> = None;
    let mut count = 0_u32;
    for row in rows {
        for repository in &row.repositories {
            if repository.project == project {
                count = count.checked_add(1).expect("listed repositories fit");
                let lower = match first {
                    Some((forge, _, _)) => row.number < forge.number,
                    None => true,
                };
                if lower {
                    first = Some((row, repository.owner.as_ref(), repository.name.as_ref()));
                }
            }
        }
    }
    match first {
        Some((forge, owner, name)) if count == 1 => Held::One { forge: Some(forge), owner, name },
        Some((forge, owner, name)) => {
            Held::Several { forge: Some(forge), owner, name, more: count.checked_sub(1).expect("first counted") }
        }
        None => Held::None,
    }
}

#[cfg(test)]
#[path = "../tests/listed.rs"]
mod tests;
