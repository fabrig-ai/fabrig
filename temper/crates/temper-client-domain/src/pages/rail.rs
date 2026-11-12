//! The rail's parts and the phone's tabs (temper/docs/design/client/view.md,
//! 3, The rail's parts and On a phone).
use crate::{Address, Domain};
use skein_lib::Wall;

/// What the rail's parts are decided from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[expect(clippy::struct_excessive_bools, reason = "independent facts, one per rule")]
pub struct Facts {
    pub projects: u32,
    pub needs: u32,
    pub needs_cut: bool,
    pub members: Option<u32>,
    pub standing_roles: Option<u32>,
    pub asking: bool,
    pub away: bool,
    pub drafts: bool,
    pub beginning: bool,
}

/// What the rail and the tab bar show.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[expect(clippy::struct_excessive_bools, reason = "one switch per part of the rail")]
pub struct Parts {
    pub all_projects: bool,
    pub needs_you: bool,
    pub count: u32,
    pub cut: bool,
    pub work: bool,
    pub team: bool,
    pub ask: bool,
    pub how: bool,
    pub away: bool,
    pub new: New,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum New {
    Hidden,
    DraftOrChange,
    Change,
}

#[must_use]
pub fn parts(facts: &Facts) -> Parts {
    let members = match facts.members {
        Some(members) => members >= 2,
        None => false,
    };
    let standing_roles = match facts.standing_roles {
        Some(roles) => roles >= 1,
        None => false,
    };
    Parts {
        all_projects: facts.projects >= 2,
        needs_you: !facts.beginning,
        count: facts.needs,
        cut: facts.needs_cut,
        work: !facts.beginning,
        team: !facts.beginning && (members || standing_roles),
        ask: facts.asking,
        how: true,
        away: facts.away,
        new: if facts.beginning {
            New::Hidden
        } else if facts.drafts {
            New::DraftOrChange
        } else {
            New::Change
        },
    }
}

/// The phone's tab bar, one pure function over the rail's parts
/// (temper/docs/design/client/view.md, 3, On a phone). "Home" always
/// shows, first; the others in this order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[expect(clippy::struct_excessive_bools, reason = "one switch per tab")]
pub struct Tabs {
    /// "Needs", while the rail's "Needs you" shows.
    pub needs: bool,
    /// Its count, drawn after "Needs" while more than zero, and "+" while cut.
    pub count: u32,
    pub cut: bool,
    /// "Work", while the rail's "Work" shows.
    pub work: bool,
    /// "Ask", while the rail's "Ask temper" shows.
    pub ask: bool,
    /// "Deployment", To begin's cut: while the rail's "Work" does not show.
    pub deployment: bool,
}

#[must_use]
pub fn tabs(parts: &Parts) -> Tabs {
    Tabs {
        needs: parts.needs_you,
        count: parts.count,
        cut: parts.cut,
        work: parts.work,
        ask: parts.ask,
        deployment: !parts.work,
    }
}

#[must_use]
#[expect(clippy::useless_conversion, reason = "the rail's contract requires a saturating project-count conversion")]
pub fn facts(root: &Domain, project: u32, wall: Wall) -> Facts {
    let (projects, asking) = match root.pages().me(root.client()) {
        Some(me) => (
            u32::try_from(me.projects.len()).unwrap_or(u32::MAX),
            jig_client::shapes::template(me, project, super::chats::ASKING_TEMPLATE).is_some(),
        ),
        None => (0, false),
    };
    let needs = if across(root) {
        root.pages().needs().count_across(root.client(), root.forge().approvals())
    } else {
        root.pages().needs().count(root.client(), root.forge().approvals(), project)
    };
    Facts {
        projects,
        needs: needs.count,
        needs_cut: needs.cut,
        members: super::team::people(root, project),
        standing_roles: super::team::standing_roles(root, project),
        asking,
        away: if across(root) { super::away_all::across(root, wall).shown } else { root.away_shown(project, wall) },
        drafts: super::drafts_offered(root, project),
        beginning: super::home::beginning(root, project, wall),
    }
}

/// Whether the page is one across projects.
#[must_use]
pub fn across(root: &Domain) -> bool {
    root.pages().address() == &Address::AllHome
        || if let Address::Inbox { filter } = root.pages().address() { filter.project.is_none() } else { false }
}
