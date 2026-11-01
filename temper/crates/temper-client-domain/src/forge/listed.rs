//! The Forges page, navigation dot (client/forge.md, 5.4; client/README.md, 5.1, 5.3, 5.6).
use super::{
    Demo, connect,
    forges::{Connection, Mode, Row, State},
};
use crate::Domain;
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
    true
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
    let demo = root.demo()?;
    if !root.forge().playground_running() {
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
