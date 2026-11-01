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
        && demo.forge == Some(number)
    {
        return None;
    }
    let State::Rows(rows, ..) = rows else { return None };
    for row in rows {
        if row.number == number && row.mode != Mode::Fake {
            return Some(row);
        }
    }
    None
}
#[must_use]
pub fn step(choice: Option<Choice>, rows: &State, demo: Option<&Demo>) -> Shown {
    match selected(choice, rows) {
        None => Shown::Choose,
        Some(choice @ (Choice::Elsewhere | Choice::GitHub | Choice::Owned | Choice::Forgejo)) => Shown::Details(choice),
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
#[must_use]
pub fn choices(rows: &State, demo: Option<&Demo>) -> List<Choice> {
    let mut choices = List::with_capacity(temper_client_protocol_wire::FORGES.checked_add(4).expect("choices fit"));
    let mut github = false;
    let mut owned = false;
    if let State::Rows(rows, ..) = rows {
        for row in rows {
            owned |= row.mode == Mode::Owned && !row.playground;
        }
        for kind in [Kind::GitHub, Kind::Forgejo] {
            // The forges decoder admits rows in increasing number order.
            for row in rows {
                let demo = match demo {
                    Some(demo) => Some(row.number) == demo.forge,
                    None => false,
                };
                if row.kind == kind && row.mode != Mode::Fake && !demo && offered(row) {
                    github |= row.kind == Kind::GitHub;
                    choices.push(Choice::Forge(row.number)).expect("bounded rows");
                }
            }
        }
    }
    if !github {
        choices.push(Choice::GitHub).expect("kind capacity");
    }
    if !owned && owned_offer(rows) != Offer::NotOffered {
        choices.push(Choice::Owned).expect("kind capacity");
    }
    choices.push(Choice::Forgejo).expect("kind capacity");
    choices.push(Choice::Elsewhere).expect("one extra choice");
    choices
}

/// A live step is running in temper; GitHub awaiting installation is idle.
#[must_use]
pub fn live(row: &Row) -> bool {
    if let Connection::SettingUp { .. } | Connection::Connecting = row.connection {
        return true;
    }
    match &row.first {
        Some(first) => first.state == super::forges::FirstState::Making,
        None => false,
    }
}
/// A set up cut short, excluding untouched choices (draft playground-reset.md, 6.4).
#[must_use]
#[expect(clippy::needless_lifetimes, reason = "the signature names the row's borrow of the supplied state")]
#[expect(clippy::manual_find, reason = "step code uses bounded loops without closures")]
pub fn unfinished<'a>(rows: &'a State) -> Option<&'a Row> {
    let State::Rows(rows, ..) = rows else { return None };
    for row in rows {
        if !row.playground && !row.finished && !live(row) && row.connection != Connection::NotSetUp && !offered(row) {
            return Some(row);
        }
    }
    None
}

/// The welcome follows the person's forge once its setup has started.
#[must_use]
pub fn personal<'a>(rows: &'a State, demo: Option<&Demo>) -> Option<&'a Row> {
    if let Some(following) = following(rows)
        && let Some(number) = following.forge
    {
        return row(number, rows, demo);
    }
    let State::Rows(rows, ..) = rows else { return None };
    for row in rows {
        if row.mode == Mode::Fake {
            continue;
        }
        if let Some(demo) = demo
            && Some(row.number) == demo.forge
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
    let State::Rows(rows, ..) = rows else { return None };
    for row in rows {
        if row.mode == Mode::Fake || !underway(row) {
            continue;
        }
        if let Some(demo) = demo
            && Some(row.number) == demo.forge
        {
            continue;
        }
        return Some(row);
    }
    None
}
#[must_use]
pub fn next_number(rows: &State) -> u32 {
    let mut largest = 0;
    if let State::Rows(rows, ..) = rows {
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
pub fn name(row: &Row, demo: Option<&Demo>) -> Name {
    if let Some(demo) = demo
        && demo.forge == Some(row.number)
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

/// A kind request follows the newest forge not listed at dispatch (temper/docs/design/draft/playground-reset.md, 3).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Following {
    pub choice: Choice,
    pub before: u32,
    pub forge: Option<u16>,
}
#[must_use]
pub fn owned_offer(rows: &State) -> Offer {
    match rows {
        State::Rows(_, owned, _) => *owned,
        State::Unknown | State::Ended => Offer::NotOffered,
    }
}
#[must_use]
pub fn following(rows: &State) -> Option<Following> {
    match rows {
        State::Rows(_, _, following) => *following,
        State::Unknown | State::Ended => None,
    }
}
pub(crate) fn begin(rows: &mut State, choice: Choice) {
    let before = next_number(rows);
    if let State::Rows(_, _, following) = rows {
        *following = Some(Following { choice, before, forge: None });
    }
}
pub(crate) fn clear(side: &mut super::Side) {
    if let State::Rows(_, _, following) = &mut side.forges {
        *following = None;
    }
}
pub(crate) fn follow(state: &mut State) {
    let State::Rows(rows, _, Some(following)) = state else { return };
    if following.forge.is_some() {
        return;
    }
    for row in rows {
        if u32::from(row.number) < following.before || row.playground || row.mode == Mode::Fake {
            continue;
        }
        let matches = match following.choice {
            Choice::GitHub => row.kind == Kind::GitHub,
            Choice::Owned => row.kind == Kind::Forgejo && row.mode == Mode::Owned,
            Choice::Forgejo => row.kind == Kind::Forgejo && row.mode == Mode::Managed,
            Choice::Forge(_) | Choice::Elsewhere => false,
        };
        if matches {
            following.forge = Some(row.number);
        }
    }
}
#[must_use]
pub fn selected(choice: Option<Choice>, rows: &State) -> Option<Choice> {
    if let Some(following) = following(rows)
        && choice == Some(following.choice)
        && let Some(number) = following.forge
    {
        return Some(Choice::Forge(number));
    }
    choice
}
