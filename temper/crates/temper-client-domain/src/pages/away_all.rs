//! Away on All projects (temper/docs/design/client/README.md, 3.5, Away, On All projects).
use super::{Pages, away, needs::Toasted};
use crate::{Domain, Limits, Request, owner::Owner};
use jig_client::{Ask, Client, Key, Told};
use skein_lib::{Env, List, Queue, Wall};

/// The person's projects' Away together.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Across {
    /// M: the projects `me` lists; 0 before `me`.
    pub projects: u32,
    /// N: those whose row is on at the wall (`Domain::away_on`).
    pub on: u32,
    /// The earliest `until` of the rows on; None where none has one.
    pub until: Option<Wall>,
    /// 3.5's Away shows for any of them (`Domain::away_shown`).
    pub shown: bool,
}

#[must_use]
pub fn across(root: &Domain, wall: Wall) -> Across {
    let mut result = Across { projects: 0, on: 0, until: None, shown: false };
    let Some(me) = root.pages().me(root.client()) else { return result };
    result.projects = me.projects.len();
    for project in &me.projects {
        result.shown |= root.away_shown(project.project, wall);
        if root.away_on(project.project, wall) {
            result.on = result.on.checked_add(1).expect("on projects fit their count");
            if let Some(row) = root.forge().away(project.project)
                && let Some(until) = row.until
            {
                result.until = Some(match result.until {
                    Some(held) => held.min(until),
                    None => until,
                });
            }
        }
    }
    result
}
/// Away on All projects' requests, one a project that changes.
#[derive(Debug)]
pub(super) struct Pending {
    on: bool,
    out: List<Key>,
    sent: u32,
    changed: u32,
    /// Requests enter the client's bounded admission one at a time.
    waiting: Queue<(Key, Ask)>,
}

pub(super) fn act(root: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    if root.pages.away_busy() || !root.forge.away_available() {
        return;
    }
    let counts = across(root, env.wall);
    let on = counts.on < counts.projects;
    let Some(me) = root.pages.me(&root.client) else { return };
    let mut pending = Pending { on, out: List::uncapped(), sent: 0, changed: 0, waiting: Queue::uncapped() };
    for project in &me.projects {
        if root.away_on(project.project, env.wall) == on {
            continue;
        }
        let Some((request, _, _)) =
            away::request(root, env, project.project, away::Act::Toggle { project: project.project })
        else {
            continue;
        };
        root.pages.serial = root.pages.serial.checked_add(1).expect("page keys fit");
        let key = Key { asker: Owner::Pages.asker(), number: root.pages.serial };
        pending.waiting.push((key, request));
        pending.sent = pending.sent.checked_add(1).expect("sent projects fit their count");
    }
    let Some((key, request)) = pending.waiting.pop() else { return };
    pending.out.push(key).expect("page keys grow");
    root.pages.shown = root.pages.shown.checked_add(u64::from(pending.sent)).expect("pages shown fits");
    root.pages.away_all = Some(pending);
    crate::route::client(root, env, jig_client::Event::Send { key, name: None, request }, out);
}

pub(super) fn told(
    pages: &mut Pages,
    env: &Env<Limits>,
    client: &Client,
    told: &Told,
    out: &mut Queue<super::Request>,
) -> bool {
    let Some(pending) = &mut pages.away_all else { return false };
    let (key, outcome) = match told {
        Told::Answered { key, outcome } if pending.out.as_slice().contains(key) => (*key, outcome),
        Told::Pending { key } if pending.out.as_slice().contains(key) => {
            if let Some(request) = client.pending(*key)
                && request.progress == jig_client::Progress::Unknown
            {
                (*key, &jig_client::Outcome::Unknown)
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
    assert_eq!(pending.out.swap_remove(0), Some(key), "one Away request out");
    match crate::forge::away::answer(outcome, env.limits.forge) {
        crate::forge::away::Answer::Away => {
            pending.changed = pending.changed.checked_add(1).expect("changed fits sent");
        }
        crate::forge::away::Answer::Unknown | crate::forge::away::Answer::Refused => {}
    }
    if let Some((key, request)) = pending.waiting.pop() {
        pending.out.push(key).expect("page keys grow");
        out.push(super::Request::Client(jig_client::Event::Send { key, name: None, request }));
    } else {
        let toast = if pending.changed == 0 {
            Toasted::AwayRefused
        } else {
            Toasted::AwayAll { on: pending.on, changed: pending.changed, sent: pending.sent }
        };
        pages.away_all = None;
        pages.needs.say(toast, env.now);
    }
    true
}
