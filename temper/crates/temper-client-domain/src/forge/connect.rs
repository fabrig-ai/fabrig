//! Pure facts for connecting a forge (client/forge.md, 5.4).
use super::{
    Demo,
    forges::{Connection, Kind, Mode, Offer, Row, State},
};
use crate::Choice;
use skein_lib::List;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shown {
    Choose,
    Details(Choice),
    SettingUp(u16),
    Ready(u16),
}
#[must_use]
#[expect(clippy::manual_find, reason = "step code uses bounded loops without closures")]
pub fn row<'a>(number: u16, rows: &'a State, demo: Option<&Demo>) -> Option<&'a Row> {
    if let Some(demo) = demo
        && demo.forge == number
    {
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
pub fn step(choice: Option<Choice>, rows: &State, demo: Option<&Demo>) -> Shown {
    let choice = target(choice, rows, demo);
    match choice {
        Some(Choice::Here) => Shown::Details(Choice::Here),
        Some(Choice::GitHub) | None => Shown::Choose,
        Some(Choice::Elsewhere) => Shown::Details(Choice::Elsewhere),
        Some(Choice::Forge(number)) => {
            let Some(row) = row(number, rows, demo) else { return Shown::Choose };
            if row.admin && admin_stopped(row) {
                return Shown::Details(Choice::Forge(number));
            }
            if failed(row) {
                return Shown::SettingUp(number);
            }
            match &row.connection {
                Connection::Stopped { .. } if row.kind == Kind::GitHub => Shown::Details(Choice::Forge(number)),
                Connection::NotSetUp | Connection::NotConnected => {
                    if offered(row) {
                        Shown::Details(Choice::Forge(number))
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
pub fn welcome_step(choice: Option<Choice>, rows: &State, demo: Option<&Demo>) -> Shown {
    let choice = target(choice, rows, demo);
    if let Some(Choice::Forge(number)) = choice
        && let Some(row) = row(number, rows, demo)
        && at_once(row)
    {
        return Shown::Choose;
    }
    step(choice, rows, demo)
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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Card {
    GitHub { forge: Option<u16> },
    Here { forge: Option<u16> },
    Admin { forge: u16 },
    Elsewhere,
}
fn visible(row: &Row, demo: Option<&Demo>) -> bool {
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
/// A kept real row has begun once it leaves its untouched choice.
#[must_use]
pub fn begun(row: &Row) -> bool {
    if let Connection::NotSetUp | Connection::NotConnected = row.connection {
        row.first.is_some() || row.app.is_some() || !row.installations.is_empty() || !row.repositories.is_empty()
    } else {
        true
    }
}
/// Connected with an adopted repository and, on GitHub, an accepted installation.
#[must_use]
pub fn whole(row: &Row) -> bool {
    if row.mode == Mode::Fake {
        return false;
    }
    let Connection::Connected { .. } = row.connection else { return false };
    if row.kind == Kind::GitHub && !super::forges::accepted(row) {
        return false;
    }
    if !row.repositories.is_empty() {
        return true;
    }
    if let Some(first) = &row.first
        && let super::forges::FirstState::Made { .. } = first.state
    {
        return true;
    }
    false
}
#[must_use]
pub fn unfinished<'a>(rows: &'a State, demo: Option<&Demo>) -> Option<&'a Row> {
    if let State::Rows(rows) = rows {
        for row in rows {
            if visible(row, demo) && begun(row) && !whole(row) {
                return Some(row);
            }
        }
    }
    None
}
#[must_use]
pub fn cards(rows: &State, demo: Option<&Demo>) -> List<Card> {
    let mut cards = List::with_capacity(temper_client_protocol_wire::FORGES.checked_add(3).expect("cards fit"));
    let mut github_seen = false;
    let mut here_seen = false;
    let mut github = None;
    let mut here = None;
    if let State::Rows(rows) = rows {
        for row in rows {
            if !visible(row, demo) {
                continue;
            }
            if row.kind == Kind::GitHub {
                github_seen = true;
                if github.is_none() && offered(row) && !begun(row) {
                    github = Some(row.number);
                }
            } else if row.mode == Mode::Owned {
                here_seen = true;
                if here.is_none() && offered(row) && !begun(row) {
                    here = Some(row.number);
                }
            }
        }
    }
    if github.is_some() || !github_seen {
        cards.push(Card::GitHub { forge: github }).expect("kind card fits");
    }
    if here.is_some() || !here_seen {
        cards.push(Card::Here { forge: here }).expect("kind card fits");
    }
    if let State::Rows(rows) = rows {
        for row in rows {
            if row.kind == Kind::Forgejo && visible(row, demo) && row.admin && offered(row) {
                cards.push(Card::Admin { forge: row.number }).expect("bounded rows");
            }
        }
    }
    cards.push(Card::Elsewhere).expect("extra card fits");
    cards
}
fn target(choice: Option<Choice>, rows: &State, demo: Option<&Demo>) -> Option<Choice> {
    for card in &cards(rows, demo) {
        match (*card, choice) {
            (Card::GitHub { forge: Some(number) }, Some(Choice::GitHub))
            | (Card::Here { forge: Some(number) }, Some(Choice::Here)) => return Some(Choice::Forge(number)),
            _ => {}
        }
    }
    choice
}

/// The welcome follows the person's forge once its setup has started.
#[must_use]
pub fn personal<'a>(rows: &'a State, demo: Option<&Demo>) -> Option<&'a Row> {
    let State::Rows(rows) = rows else { return None };
    for row in rows {
        if row.mode == Mode::Fake {
            continue;
        }
        if let Some(demo) = demo
            && row.number == demo.forge
        {
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
pub fn underway_forge<'a>(rows: &'a State, demo: Option<&Demo>) -> Option<&'a Row> {
    let State::Rows(rows) = rows else { return None };
    for row in rows {
        if row.mode == Mode::Fake || !underway(row) {
            continue;
        }
        if let Some(demo) = demo
            && row.number == demo.forge
        {
            continue;
        }
        return Some(row);
    }
    None
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Name {
    Here,
    GitHub,
    Forgejo,
    Playground,
}
#[must_use]
pub fn name(row: &Row, demo: Option<&Demo>) -> Name {
    if let Some(demo) = demo
        && demo.forge == row.number
    {
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
