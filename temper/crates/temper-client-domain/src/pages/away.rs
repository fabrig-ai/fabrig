//! Away's acts and answered toast; the project's classes decide what Away may decide (temper/docs/design/client/pages/README.md, Away).
use super::{Pages, needs::Toasted};
use crate::{
    Address, Domain, Limits, Request,
    forge::{away, decided},
    owner::Owner,
};
use alloc::boxed::Box;
use jig_client::{Client, Key, Told};
use skein_lib::{Env, List, Queue, Wall};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Act {
    Overturn { project: u32, number: u32 },
    Toggle { project: u32 },
    ToggleAll,
    Until { project: u32, eight: bool },
    Limits { project: u32 },
}
#[derive(Debug)]
pub(super) struct Pending {
    key: Key,
    project: u32,
    on: bool,
    until: Option<Wall>,
}
/// Session ownership keeps the message alive across page navigation (client/pages/README.md, Overturn).
#[derive(Debug)]
pub(super) struct Overturns {
    pending: Option<Overturn>,
    marked: List<(u32, u32)>,
}
#[derive(Debug)]
struct Overturn {
    key: Key,
    project: u32,
    number: u32,
    task: u64,
    words: Option<Box<[u8]>>,
}
impl Overturns {
    pub(super) fn new() -> Self {
        Self { pending: None, marked: List::uncapped() }
    }
    pub(super) fn marked(&self, project: u32, number: u32) -> bool {
        self.marked.as_slice().contains(&(project, number))
    }
}
impl Pages {
    #[must_use]
    pub const fn overturn_busy(&self) -> bool {
        self.overturns.pending.is_some()
    }
    /// The section’s focus count, raised by each Limits act.
    #[must_use]
    pub const fn away_focus(&self) -> u64 {
        self.away_focus
    }
    #[must_use]
    pub const fn away_busy(&self) -> bool {
        self.away.is_some() || self.away_all.is_some()
    }
}
pub(crate) fn act(root: &mut Domain, env: &Env<Limits>, act: Act, out: &mut Queue<Request>) {
    let project = match act {
        Act::ToggleAll => {
            super::away_all::act(root, env, out);
            return;
        }
        Act::Overturn { project, number } => {
            overturn(root, env, project, number, out);
            return;
        }
        Act::Limits { project } => {
            root.pages.away_focus = root.pages.away_focus.checked_add(1).expect("focus count fits");
            out.push(Request::Show { address: Address::Settings { project }, replace: false });
            return;
        }
        Act::Toggle { project } | Act::Until { project, .. } => project,
    };
    if root.pages.away_busy() || !root.forge.away_available() {
        return;
    }
    let Some((request, on, until)) = request(root, env, project, act) else { return };
    root.pages.serial = root.pages.serial.checked_add(1).expect("page keys fit");
    let key = Key { asker: Owner::Pages.asker(), number: root.pages.serial };
    root.pages.away = Some(Pending { key, project, on, until });
    root.pages.shown = root.pages.shown.checked_add(1).expect("pages shown fits");
    crate::route::client(root, env, jig_client::Event::Send { key, name: None, request }, out);
}
/// The project's request, with the state its answer will toast.
pub(super) fn request(
    root: &Domain,
    env: &Env<Limits>,
    project: u32,
    act: Act,
) -> Option<(jig_client::Ask, bool, Option<Wall>)> {
    let row = root.forge.away(project);
    let mut on = root.away_on(project, env.wall);
    let mut until = match row {
        Some(row) => row.until,
        None => Some(away::next_eight(env.wall, root.zone_minutes())),
    };
    match act {
        Act::Toggle { .. } => {
            on = !on;
            if on && until.is_some() {
                until = Some(away::next_eight(env.wall, root.zone_minutes()));
            }
        }
        Act::Until { eight, .. } => {
            until = if eight { Some(away::next_eight(env.wall, root.zone_minutes())) } else { None };
        }
        Act::ToggleAll | Act::Overturn { .. } | Act::Limits { .. } => {
            unreachable!("only project settings build an Away request")
        }
    }
    let request = away::ask(project, on, until, &env.limits.forge)?;
    Some((request, on, until))
}
pub(super) fn told(
    pages: &mut Pages,
    env: &Env<Limits>,
    client: &Client,
    told: &Told,
    out: &mut Queue<super::Request>,
) -> bool {
    if super::away_all::told(pages, env, client, told, out) || overturn_told(pages, env, client, told, out) {
        return true;
    }
    let Some(pending) = &pages.away else { return false };
    let outcome = match told {
        Told::Answered { key, outcome } if *key == pending.key => outcome,
        Told::Pending { key } if *key == pending.key => {
            if let Some(request) = client.pending(*key)
                && request.progress == jig_client::Progress::Unknown
            {
                &jig_client::Outcome::Unknown
            } else {
                return true;
            }
        }
        Told::Answered { .. }
        | Told::Pending { .. }
        | Told::Changed { .. }
        | Told::Ended { .. }
        | Told::Link { .. }
        | Told::Kept { .. }
        | Told::Dropped { .. }
        | Told::Page { .. }
        | Told::Text { .. } => return false,
    };
    let toast = match away::answer(outcome, env.limits.forge) {
        away::Answer::Away => {
            if pending.on {
                Toasted::AwayOn { project: pending.project, until: pending.until }
            } else {
                Toasted::AwayOff
            }
        }
        away::Answer::Unknown | away::Answer::Refused => Toasted::AwayRefused,
    };
    pages.away = None;
    pages.needs.say(toast, env.now);
    true
}

fn overturn(root: &mut Domain, env: &Env<Limits>, project: u32, number: u32, out: &mut Queue<Request>) {
    let shown = match root.pages.address {
        Address::Home { project: home } if home == project => super::home::facts(root, project, env.wall).decided,
        Address::AllHome => root.pages.decided(None, env.wall),
        Address::Inbox { filter } if filter.project.is_none() || filter.project == Some(project) => {
            root.pages.decided(filter.project, env.wall)
        }
        Address::Home { .. }
        | Address::Inbox { .. }
        | Address::Team { .. }
        | Address::NewChat { .. }
        | Address::Chats { .. }
        | Address::Accounts
        | Address::Forges
        | Address::ConnectForge { .. }
        | Address::ConnectModel { .. }
        | Address::Welcome { .. }
        | Address::Chat { .. }
        | Address::Talk { .. }
        | Address::Task { .. }
        | Address::Changes { .. }
        | Address::Settings { .. }
        | Address::Repositories { .. }
        | Address::Models { .. }
        | Address::Builders { .. }
        | Address::Role { .. }
        | Address::Rule { .. }
        | Address::Goals { .. }
        | Address::Ask { .. }
        | Address::Work { .. }
        | Address::Told
        | Address::Remembered
        | Address::Members
        | Address::Limits
        | Address::Adopt { .. } => None,
    };
    if root.pages.overturn_busy() {
        return;
    }
    let Some(form) = shown else { return };
    for row in form.rows {
        if row.project != project
            || row.number != number
            || row.overturned
            || row.state != super::decided::DecisionTask::Live
        {
            continue;
        }
        let Some(request) = decided::overturn(project, number, &env.limits.forge) else { return };
        let words = decided::overturn_words(&row.question, &row.took);
        if jig_client::contract::Text::new(words.clone(), env.limits.client.domain.text).is_err() {
            return;
        }
        root.pages.serial = root.pages.serial.checked_add(1).expect("page keys fit");
        let key = Key { asker: Owner::Pages.asker(), number: root.pages.serial };
        root.pages.overturns.pending = Some(Overturn { key, project, number, task: row.task, words: Some(words) });
        root.pages.shown = root.pages.shown.checked_add(1).expect("pages shown fits");
        crate::route::client(root, env, jig_client::Event::Send { key, name: None, request }, out);
        return;
    }
}
fn overturn_told(
    pages: &mut Pages,
    env: &Env<Limits>,
    client: &Client,
    told: &Told,
    out: &mut Queue<super::Request>,
) -> bool {
    let Some(pending) = &pages.overturns.pending else { return false };
    let outcome = match told {
        Told::Answered { key, outcome } if *key == pending.key => outcome,
        Told::Pending { key } if *key == pending.key => {
            if let Some(request) = client.pending(*key)
                && request.progress == jig_client::Progress::Unknown
            {
                &jig_client::Outcome::Unknown
            } else {
                return true;
            }
        }
        Told::Answered { .. }
        | Told::Pending { .. }
        | Told::Changed { .. }
        | Told::Ended { .. }
        | Told::Link { .. }
        | Told::Kept { .. }
        | Told::Dropped { .. }
        | Told::Page { .. }
        | Told::Text { .. } => return false,
    };
    let mut pending = pages.overturns.pending.take().expect("matched overturn");
    let toast = if let Some(words) = pending.words.take() {
        if decided::overturned(outcome, env.limits.forge) {
            pages.overturns.marked.push((pending.project, pending.number)).expect("session marks grow");
            pages.serial = pages.serial.checked_add(1).expect("page keys fit");
            pending.key = Key { asker: Owner::Pages.asker(), number: pages.serial };
            let request = jig_client::Ask::Jig(Box::new(jig_client::contract::Ask::Message {
                project: pending.project,
                task: pending.task,
                words: jig_client::contract::Text::new(words, env.limits.client.domain.text)
                    .expect("overturn words bounded at act"),
            }));
            out.push(super::Request::Client(jig_client::Event::Send { key: pending.key, name: None, request }));
            pages.overturns.pending = Some(pending);
            return true;
        }
        Toasted::OverturnAlready
    } else {
        match outcome {
            jig_client::Outcome::Answered(jig_client::Answered::Jig(jig_client::contract::Answer::Done(_))) => {
                Toasted::Overturned
            }
            jig_client::Outcome::Answered(_)
            | jig_client::Outcome::Unknown
            | jig_client::Outcome::NotKept
            | jig_client::Outcome::Limit
            | jig_client::Outcome::Unconfirmed
            | jig_client::Outcome::Held
            | jig_client::Outcome::Unsent
            | jig_client::Outcome::Full { .. }
            | jig_client::Outcome::Lost => Toasted::OverturnEnded,
        }
    };
    pages.needs.say(toast, env.now);
    true
}
