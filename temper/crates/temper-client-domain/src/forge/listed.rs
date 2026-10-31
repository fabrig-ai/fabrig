//! Kept forges and their navigation dot (temper/docs/design/draft/playground-reset.md, 3, 6.1).
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
