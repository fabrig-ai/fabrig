//! Forge choices and unfinished records (temper/docs/design/draft/playground-reset.md, 3, 6.4).
use super::forges::{Connection, Kind, Mode, Offer, Row, State};
use crate::Choice as AddressChoice;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Choice {
    GitHub,
    Owned,
    Managed,
}
#[must_use]
pub fn playground(side: &super::Side) -> Option<u16> {
    Some(side.playground()?.forge)
}
use alloc::boxed::Box;
use skein_lib::List;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shown {
    Choose,
    Unfinished(u16),
    Details(AddressChoice),
    SettingUp(u16),
    Ready(u16),
}
#[must_use]
#[expect(clippy::manual_find, reason = "step code uses bounded loops without closures")]
pub fn row(number: u16, rows: &State, playground: Option<u16>) -> Option<&Row> {
    if playground == Some(number) {
        return None;
    }
    let State::Rows(rows) = rows else { return None };
    for row in rows {
        if row.number == number && row.mode != Mode::Fake {
            return Some(row);
        }
    }
    None
}
#[must_use]
pub fn step(choice: Option<AddressChoice>, rows: &State, playground: Option<u16>) -> Shown {
    match choice {
        None => Shown::Choose,
        Some(
            choice @ (AddressChoice::GitHub | AddressChoice::Owned | AddressChoice::Managed | AddressChoice::Elsewhere),
        ) => Shown::Details(choice),
        Some(AddressChoice::Forge(number)) => {
            let Some(row) = row(number, rows, playground) else { return Shown::Choose };
            if row.admin && admin_stopped(row) {
                return Shown::Details(AddressChoice::Forge(number));
            }
            if failed(row) {
                return Shown::SettingUp(number);
            }
            match &row.connection {
                Connection::Stopped { .. } if row.kind == Kind::GitHub => Shown::Details(AddressChoice::Forge(number)),
                Connection::NotSetUp | Connection::NotConnected => {
                    if offered(row) {
                        Shown::Details(AddressChoice::Forge(number))
                    } else {
                        Shown::Choose
                    }
                }
                Connection::SettingUp { .. } | Connection::SetUpFailed { .. } | Connection::Connecting => {
                    Shown::SettingUp(number)
                }
                Connection::Connected { .. } => {
                    if underway(row) || failed(row) {
                        Shown::SettingUp(number)
                    } else {
                        Shown::Ready(number)
                    }
                }
                Connection::Stopped { .. } => Shown::Choose,
            }
        }
    }
}
/// The welcome skips Details only for an offered owned Forgejo under the box.
#[must_use]
pub fn at_once(row: &Row) -> bool {
    row.kind == Kind::Forgejo
        && row.mode == Mode::Owned
        && !row.admin
        && row.connection == Connection::NotSetUp
        && row.offer == Offer::Offered
}
#[must_use]
pub fn welcome_step(choice: Option<AddressChoice>, rows: &State, playground: Option<u16>) -> Shown {
    if choice.is_none()
        && let Some(row) = unfinished(records(rows), playground)
    {
        return Shown::Unfinished(row.number);
    }
    if let Some(AddressChoice::Forge(number)) = choice
        && let Some(row) = row(number, rows, playground)
        && at_once(row)
    {
        return Shown::Choose;
    }
    step(choice, rows, playground)
}
#[must_use]
pub(crate) fn admin_stopped(row: &Row) -> bool {
    if let Connection::Stopped { .. } = row.connection {
        return true;
    }
    row.ended_stop.is_some() || failed(row)
}
fn disconnected(row: &Row) -> bool {
    match row.connection {
        Connection::NotConnected | Connection::Stopped { .. } => true,
        Connection::NotSetUp
        | Connection::SettingUp { .. }
        | Connection::SetUpFailed { .. }
        | Connection::Connecting
        | Connection::Connected { .. } => false,
    }
}
fn offered(row: &Row) -> bool {
    match row.kind {
        Kind::GitHub => match row.connection {
            Connection::NotConnected | Connection::Stopped { .. } => true,
            Connection::NotSetUp
            | Connection::SettingUp { .. }
            | Connection::SetUpFailed { .. }
            | Connection::Connecting
            | Connection::Connected { .. } => false,
        },
        Kind::Forgejo => {
            (row.admin && disconnected(row))
                || (row.mode == Mode::Owned && row.connection == Connection::NotSetUp && row.offer != Offer::NotOffered)
        }
    }
}
/// Unknown and ended watches have no records to offer to the pure facts.
#[must_use]
pub fn records(state: &State) -> &[Row] {
    match state {
        State::Rows(rows) => rows,
        State::Unknown | State::Ended => &[],
    }
}
/// The deployment fact supplies availability before any owned record exists.
#[must_use]
pub fn choices(rows: &[Row], playground: Option<u16>, under_box: bool) -> Box<[Choice]> {
    let mut choices = List::with_capacity(3);
    if playground.is_some() {
        return choices.into_boxed();
    }
    let mut owned = under_box;
    for row in rows {
        if row.mode == Mode::Owned {
            owned = row.offer == Offer::Offered;
            break;
        }
    }
    choices.push(Choice::GitHub).expect("three choices");
    if owned {
        choices.push(Choice::Owned).expect("three choices");
    }
    choices.push(Choice::Managed).expect("three choices");
    choices.into_boxed()
}

/// A real record is unfinished until connected with its first repository adopted.
#[must_use]
pub fn unfinished(rows: &[Row], playground: Option<u16>) -> Option<&Row> {
    for row in rows {
        if row.mode == Mode::Fake || playground == Some(row.number) {
            continue;
        }
        if !complete(row) {
            return Some(row);
        }
    }
    None
}
#[must_use]
pub fn complete(row: &Row) -> bool {
    if let Connection::Connected { .. } = row.connection {
        return !row.repositories.is_empty();
    }
    false
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnfinishedAct {
    GoOn,
    StartOver,
}
#[must_use]
pub fn unfinished_acts(row: &Row) -> List<UnfinishedAct> {
    let mut acts = List::with_capacity(2);
    if row.kind == Kind::GitHub && row.app.is_some() {
        acts.push(UnfinishedAct::GoOn).expect("two acts");
    }
    acts.push(UnfinishedAct::StartOver).expect("two acts");
    acts
}

/// The welcome follows the person's forge once its setup has started.
#[must_use]
pub fn personal(rows: &State, playground: Option<u16>) -> Option<&Row> {
    let State::Rows(rows) = rows else { return None };
    for row in rows {
        if row.mode == Mode::Fake {
            continue;
        }
        if playground == Some(row.number) {
            continue;
        }
        if row.admin && admin_stopped(row) {
            return Some(row);
        }
        match row.connection {
            Connection::SettingUp { .. }
            | Connection::Connecting
            | Connection::Connected { .. }
            | Connection::SetUpFailed { .. } => return Some(row),
            Connection::NotConnected | Connection::NotSetUp | Connection::Stopped { .. } => {}
        }
    }
    None
}
#[must_use]
pub fn underway(row: &Row) -> bool {
    match row.connection {
        Connection::SettingUp { .. } | Connection::Connecting => true,
        Connection::Connected { .. } if row.kind == Kind::GitHub && !super::forges::accepted(row) => true,
        Connection::Connected { .. } => match &row.first {
            Some(first) => first.state == super::forges::FirstState::Making,
            None => false,
        },
        Connection::NotConnected
        | Connection::NotSetUp
        | Connection::SetUpFailed { .. }
        | Connection::Stopped { .. } => false,
    }
}
#[must_use]
pub fn failed(row: &Row) -> bool {
    if let Some(first) = &row.first {
        match first.state {
            super::forges::FirstState::Failed { .. } => return true,
            super::forges::FirstState::Making | super::forges::FirstState::Made { .. } => {}
        }
    }
    match row.connection {
        Connection::SetUpFailed { .. } => true,
        Connection::NotConnected
        | Connection::NotSetUp
        | Connection::SettingUp { .. }
        | Connection::Connecting
        | Connection::Connected { .. }
        | Connection::Stopped { .. } => false,
    }
}
/// A setup line remains visible even when another forge is already connected.
#[must_use]
pub fn underway_forge(rows: &State, playground: Option<u16>) -> Option<&Row> {
    let State::Rows(rows) = rows else { return None };
    for row in rows {
        if row.mode == Mode::Fake || !underway(row) {
            continue;
        }
        if playground == Some(row.number) {
            continue;
        }
        return Some(row);
    }
    None
}
#[must_use]
pub fn next_number(rows: &State) -> u32 {
    let mut largest = 0;
    if let State::Rows(rows) = rows {
        for row in rows {
            largest = largest.max(u32::from(row.number));
        }
    }
    largest.checked_add(1).expect("u16 forge numbers fit u32")
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Name {
    Here,
    GitHub,
    Forgejo,
    Playground,
}
#[must_use]
pub fn name(row: &Row, playground: Option<u16>) -> Name {
    if playground == Some(row.number) {
        return Name::Playground;
    }
    match row.kind {
        Kind::GitHub => Name::GitHub,
        Kind::Forgejo => {
            if row.mode == Mode::Owned {
                Name::Here
            } else {
                Name::Forgejo
            }
        }
    }
}

/// The five visible setup steps; repository creation has its own failure facts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FailedStep {
    Fetch,
    Prepare,
    Run,
    Connect,
    Repository,
}
#[must_use]
pub fn failed_step(row: &Row) -> Option<FailedStep> {
    if let Some(first) = &row.first {
        match first.state {
            super::forges::FirstState::Failed { .. } => return Some(FailedStep::Repository),
            super::forges::FirstState::Making | super::forges::FirstState::Made { .. } => {}
        }
    }
    match &row.connection {
        Connection::SetUpFailed { at, .. } => match at {
            Some(super::forges::FailedStep::Fetch) => Some(FailedStep::Fetch),
            Some(super::forges::FailedStep::Prepare) => Some(FailedStep::Prepare),
            Some(super::forges::FailedStep::Run) => Some(FailedStep::Run),
            Some(super::forges::FailedStep::Connect) => Some(FailedStep::Connect),
            None => None,
        },
        Connection::NotConnected
        | Connection::NotSetUp
        | Connection::SettingUp { .. }
        | Connection::Connecting
        | Connection::Connected { .. }
        | Connection::Stopped { .. } => None,
    }
}
