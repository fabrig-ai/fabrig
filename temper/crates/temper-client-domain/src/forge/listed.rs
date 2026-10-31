//! Kept forges and their navigation dot (temper/docs/design/draft/playground-reset.md, 3, 6.1).
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
    if !root.demo_answered()
        || root.forge().demo_running()
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
        playground: !personal_connected && root.forge().demo().is_some(),
        forge: owner && !connected,
        model,
        step,
    })
}
