//! How work runs' parts and What reaches you's columns
//! (temper/docs/design/client/pages/settings.md, 2, *Its parts* and *What reaches you*).
use alloc::boxed::Box;
use jig_client::contract as c;
use skein_lib::List;
use temper_client_protocol_wire::v1 as w;
use temper_protocol_reaches::{Class, Classes, Kind, Subject};

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
#[expect(clippy::struct_excessive_bools, reason = "the page gathers independent facts for its parts")]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Facts {
    /// `me` lists Ask temper's template in the project.
    pub asking: bool,
    pub read: Read,
    /// The rows of the project's shown `ways` read.
    pub rules: List<w::PathRule>,
    /// Kind 7 read, with Goals shown.
    pub goals: bool,
    /// The page has read at least one project role.
    pub roles: bool,
    /// The default goal template's budget, none with no template or kind 7 unread.
    pub budget: Option<c::TemplateBudget>,
    /// Kind 8 read for the project: `root.forge().classes(project)` answers.
    pub classes: bool,
    /// `me` lists two projects or more.
    pub several: bool,
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

/// Rules by path's optional link and column.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rules {
    pub add: bool,
    pub review: bool,
}

/// What How work runs draws (client/pages/settings.md, 2, *Its parts*).
#[expect(clippy::struct_excessive_bools, reason = "each independent part has its own visibility rule")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Parts {
    pub head: Head,
    /// "Ask temper to change this".
    pub ask: bool,
    pub path: Path,
    pub rules: Option<Rules>,
    pub roles: bool,
    /// A goal's amount or cap, while Goals is shown.
    pub budget: Option<Budget>,
    /// What reaches you draws its columns: `facts.classes`.
    pub reaches: bool,
    /// Its head says "each project has its own": `facts.several`.
    pub several: bool,
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
    let mut rules = None;
    if path == Path::Shown && !facts.rules.is_empty() {
        let mut review = false;
        for row in &facts.rules {
            review |= !row.review().is_empty();
        }
        rules = Some(Rules { add: facts.asking, review });
    }
    Parts {
        head,
        ask: facts.asking,
        path,
        rules,
        roles: facts.roles,
        budget,
        reaches: facts.classes,
        several: facts.several,
    }
}

/// Gather from `me`, the forge's kinds 7 and 8, ways and roles held for the address.
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
    let mut path_rules = List::with_capacity(0);
    if (pages.address() == &(Address::Ask { project }) || pages.address() == &(Address::Settings { project }))
        && let Some(ways) = pages.stock_ways()
    {
        read = match ways.state() {
            State::Loading => Read::Reading,
            State::Ended => Read::Ended,
            State::Shown(panel) => {
                path_rules = panel.rules().rows().clone();
                Read::Shown { current: *panel.current() }
            }
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
    let roles = if pages.address() == &(Address::Settings { project }) {
        match pages.roles() {
            Some(roles) => match roles.read() {
                super::roles::Read::Drawn(w::View::Roles(view)) => !view.roles().is_empty(),
                super::roles::Read::Reading
                | super::roles::Read::Unreadable
                | super::roles::Read::Drawn(
                    w::View::Charter(_)
                    | w::View::Rule(_)
                    | w::View::NotDrawn(_)
                    | w::View::Way(_)
                    | w::View::Path(_)
                    | w::View::Proposal(_),
                ) => false,
            },
            None => false,
        }
    } else {
        false
    };
    let classes = root.forge().classes(project).is_some();
    let several = match me {
        Some(me) => me.projects.len() >= 2,
        None => false,
    };
    Facts { asking, read, rules: path_rules, goals, roles, budget, classes, several }
}

/// A subject's line in one of What reaches you's columns.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Line {
    pub subject: Subject,
    /// The item's place in the table, from 1; 0 for a subject with no item
    /// of its own: a movable kind or a set with none, and every fixed kind.
    pub item: u8,
    /// The line's class: its item's, `AskAway` with no item, `Wait` for a fixed kind.
    pub class: Class,
}

/// What reaches you's three columns (client/pages/settings.md, 2, *What reaches you*).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reaches {
    /// "Agents decide, then tell you": the lines at `Decide`, in the settings order.
    pub decide: Box<[Line]>,
    /// "Asks you": the lines at `AskAway`, then those at `Ask`, each in the settings order.
    pub ask: Box<[Line]>,
    /// "Always waits for you": the fixed kinds in `Kind::ALL`'s order, item 0, `Wait`.
    pub wait: Box<[Line]>,
}

#[must_use]
pub fn reaches(classes: &Classes) -> Reaches {
    let mut decide = List::uncapped();
    let mut ask = List::uncapped();
    let mut wait = List::uncapped();
    settings_lines(classes, Class::Decide, &mut decide);
    settings_lines(classes, Class::AskAway, &mut ask);
    settings_lines(classes, Class::Ask, &mut ask);
    for kind in Kind::ALL {
        if kind.fixed() {
            wait.push(Line { subject: Subject::Kind(kind), item: 0, class: Class::Wait }).expect("fixed kinds fit");
        }
    }
    Reaches { decide: decide.into_boxed(), ask: ask.into_boxed(), wait: wait.into_boxed() }
}

fn settings_lines(classes: &Classes, class: Class, lines: &mut List<Line>) {
    for kind in Kind::ALL {
        if kind.fixed() {
            continue;
        }
        let subject = Subject::Kind(kind);
        let mut own = None;
        for (index, item) in classes.items.iter().enumerate() {
            if item.subject == subject && item.words.is_empty() {
                own = Some((index, item));
                break;
            }
        }
        match own {
            Some((index, item)) => {
                if item.class == class {
                    item_line(lines, subject, index, class);
                }
            }
            None => {
                if class == Class::AskAway {
                    lines.push(Line { subject, item: 0, class }).expect("settings lines grow");
                }
            }
        }
    }
    for (index, item) in classes.items.iter().enumerate() {
        let shown = match item.subject {
            Subject::Kind(kind) => !kind.fixed() && !item.words.is_empty(),
            Subject::Set(number) => classes.set(number).is_some(),
        };
        if shown && item.class == class {
            item_line(lines, item.subject, index, class);
        }
    }
    for set in &classes.sets {
        let subject = Subject::Set(set.number);
        let mut own = false;
        for item in &classes.items {
            if item.subject == subject && item.words.is_empty() {
                own = true;
                break;
            }
        }
        if !own && class == Class::AskAway {
            lines.push(Line { subject, item: 0, class }).expect("settings lines grow");
        }
    }
}

fn item_line(lines: &mut List<Line>, subject: Subject, index: usize, class: Class) {
    if let Ok(item) = u8::try_from(index.saturating_add(1)) {
        lines.push(Line { subject, item, class }).expect("settings lines grow");
    }
}
