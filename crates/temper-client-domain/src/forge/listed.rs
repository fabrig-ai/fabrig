//! The Forges page, navigation dot and setup strip (client/forge.md, 5.4; client/README.md, 5.1, 5.3, 5.6).
use super::{
    Demo, connect,
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
pub fn personal(row: &Row, demo: Option<&Demo>) -> bool {
    if row.mode == Mode::Fake {
        return false;
    }
    if let Some(demo) = demo
        && demo.forge == row.number
    {
        return false;
    }
    match row.connection {
        Connection::NotConnected | Connection::NotSetUp => false,
        Connection::SettingUp { .. }
        | Connection::SetUpFailed { .. }
        | Connection::Connecting
        | Connection::Connected { .. }
        | Connection::Stopped { .. } => true,
    }
}
/// The decoder keeps these rows in increasing forge number order.
#[must_use]
pub fn rows<'a>(state: &'a State, demo: Option<&Demo>) -> List<&'a Row> {
    let mut listed = List::with_capacity(super::forges::ROWS);
    if let State::Rows(rows) = state {
        for row in rows {
            if personal(row, demo) {
                listed.push(row).expect("bounded forge rows");
            }
        }
    }
    listed
}
#[must_use]
pub fn dot(state: &State, demo: Option<&Demo>) -> Dot {
    let mut dot = Dot::None;
    for row in &rows(state, demo) {
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
pub fn playground(root: &Domain) -> Option<(&Demo, &Row)> {
    let (demo, halves) = root.demo()?;
    if !halves.forge {
        return None;
    }
    if let State::Rows(rows) = root.forges() {
        for row in rows {
            if row.number == demo.forge {
                return Some((demo, row));
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
    if !root.demo_answered() || root.forge().demo_running() || welcome::decision(root) != Some(false) {
        return None;
    }
    let owner = accounts::current_me(root.client())?.deployment_role == Some(jig_client::contract::HeldRole::Owner);
    let mut connected = false;
    let mut started = false;
    let mut personal_connected = false;
    for row in &rows(root.forges(), root.forge().demo()) {
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
