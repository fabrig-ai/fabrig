//! Kept forges and their navigation dot (temper/docs/design/client/forge.md, 5.4; temper/docs/design/client/README.md, 3.5, 4.4 and 4.8).
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
