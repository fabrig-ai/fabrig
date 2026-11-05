//! Away's acts and answered toast (temper/docs/design/client/README.md, 3.5, Away).
use super::{Pages, needs::Toasted};
use crate::{Address, Domain, Limits, Request, forge::away, owner::Owner};
use jig_client::{Client, Key, Told};
use skein_lib::{Env, Queue, Wall};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Act {
    Toggle { project: u32 },
    Kinds { project: u32, kinds: u8 },
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
impl Pages {
    /// Consume the section's one-shot focus request.
    pub fn away_focus(&mut self) -> bool {
        core::mem::replace(&mut self.away_focus, false)
    }
    #[must_use]
    pub const fn away_busy(&self) -> bool {
        self.away.is_some()
    }
}
pub(crate) fn act(root: &mut Domain, env: &Env<Limits>, act: Act, out: &mut Queue<Request>) {
    let project = match act {
        Act::Limits { project } => {
            root.pages.away_focus = true;
            out.push(Request::Show { address: Address::Settings { project }, replace: false });
            return;
        }
        Act::Toggle { project } | Act::Kinds { project, .. } | Act::Until { project, .. } => project,
    };
    if root.pages.away.is_some() || !root.forge.away_available() {
        return;
    }
    let row = root.forge.away(project);
    let mut on = root.away_on(project, env.wall);
    let (mut until, mut kinds) = match row {
        Some(row) => (row.until, row.kinds),
        None => (Some(away::next_eight(env.wall, root.zone_minutes())), away::DEFAULT_KINDS),
    };
    match act {
        Act::Toggle { .. } => {
            on = !on;
            if on && until.is_some() {
                until = Some(away::next_eight(env.wall, root.zone_minutes()));
            }
        }
        Act::Kinds { kinds: choice, .. } => kinds = choice,
        Act::Until { eight, .. } => {
            until = if eight { Some(away::next_eight(env.wall, root.zone_minutes())) } else { None };
        }
        Act::Limits { .. } => unreachable!("Limits returned before requesting"),
    }
    let Some(request) = away::ask(project, on, until, kinds, &env.limits.forge) else { return };
    root.pages.serial = root.pages.serial.checked_add(1).expect("page keys fit");
    let key = Key { asker: Owner::Pages.asker(), number: root.pages.serial };
    root.pages.away = Some(Pending { key, project, on, until });
    root.pages.shown = root.pages.shown.checked_add(1).expect("pages shown fits");
    crate::route::client(root, env, jig_client::Event::Send { key, name: None, request }, out);
}
pub(super) fn told(pages: &mut Pages, env: &Env<Limits>, client: &Client, told: &Told) -> bool {
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
