//! How work runs' parts (temper/docs/design/client/README.md, 5.13).
use jig_client::contract as c;

/// The project's `ways` read, as the page sees it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Read {
    Reading,
    Ended,
    /// The stock way the project runs, none for its own.
    Shown {
        current: Option<u32>,
    },
}

/// What How work runs' parts are decided from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Facts {
    /// `me` lists Ask temper's template in the project.
    pub asking: bool,
    pub read: Read,
    /// Kind 7 read, with Goals shown.
    pub goals: bool,
    /// The default goal template's budget, none with no template or kind 7 unread.
    pub budget: Option<c::TemplateBudget>,
}

/// The heading's way.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Head {
    Reading,
    Way(u32),
    Own,
}

/// The path section's state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Path {
    Reading,
    Ended,
    Shown,
}

/// Budgets' one row's figure.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Budget {
    Amount(u64),
    UpTo(u64),
}

/// What How work runs draws (README.md, 5.13, *Its parts*).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Parts {
    pub head: Head,
    /// "Ask temper to change this".
    pub ask: bool,
    pub path: Path,
    /// A goal's amount or cap, while Goals is shown.
    pub budget: Option<Budget>,
}

#[must_use]
pub fn parts(facts: &Facts) -> Parts {
    let (head, path) = match facts.read {
        Read::Reading => (Head::Reading, Path::Reading),
        Read::Ended => (Head::Reading, Path::Ended),
        Read::Shown { current: Some(index) } => (Head::Way(index), Path::Shown),
        Read::Shown { current: None } => (Head::Own, Path::Shown),
    };
    let budget = if facts.goals {
        match facts.budget {
            Some(c::TemplateBudget::Amount(amount)) => Some(Budget::Amount(amount)),
            Some(c::TemplateBudget::AsAllowed { cap: Some(amount) }) => Some(Budget::UpTo(amount)),
            Some(c::TemplateBudget::Given | c::TemplateBudget::AsAllowed { cap: None }) | None => None,
        }
    } else {
        None
    };
    Parts { head, ask: facts.asking, path, budget }
}

/// Gather from `me`, the forge's kind 7 and the ways held for the address.
#[must_use]
pub fn facts(root: &crate::Domain, project: u32) -> Facts {
    use crate::{Address, forge::stock_ways::State};
    let pages = root.pages();
    let me = pages.me(root.client());
    let asking = match me {
        Some(me) => jig_client::shapes::template(me, project, super::chats::ASKING_TEMPLATE).is_some(),
        None => false,
    };
    let mut read = Read::Reading;
    if (pages.address() == &(Address::Ask { project }) || pages.address() == &(Address::Settings { project }))
        && let Some(ways) = pages.stock_ways()
    {
        read = match ways.state() {
            State::Loading => Read::Reading,
            State::Ended => Read::Ended,
            State::Shown(panel) => Read::Shown { current: *panel.current() },
        };
    }
    let mut goals = false;
    let mut budget = None;
    if let Some(ways) = root.forge().ways(project) {
        goals = ways.goals;
        if let Some(me) = me
            && let Some(number) = crate::forge::ways::default_way(me, project, &ways)
            && let Some(template) = jig_client::shapes::template(me, project, number)
        {
            budget = Some(template.budget);
        }
    }
    Facts { asking, read, goals, budget }
}
