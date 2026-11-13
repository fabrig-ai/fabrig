//! How work runs' heading, path, reaches, roles and budget (temper/docs/design/client/pages/settings.md, 2).
pub(crate) use crate::common::how::{Who, chip, project_name, text};
use crate::{
    bindings::Bindings,
    look,
    words::{Word, Words},
};
use alloc::boxed::Box;
use skein_lib::bytes::{concat, copy_of};
use skein_ui::{Builder, Role, Run, phrases::Hole};
use temper_client_domain::{
    Address, Domain,
    pages::{
        Act,
        how::{self, Budget, Head, Path},
    },
};
use temper_client_protocol_wire::v1 as wire;

pub(crate) fn path(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings, project: u32) {
    let parts = how::parts(&how::facts(root, project));
    super::settings::section_head(b, &w.say(Word::HowPath, &[]));
    match parts.path {
        Path::Reading => {
            b.close();
            b.close();
            return;
        }
        Path::Ended => {
            b.close();
            text(b, Role::Paragraph, &w.say(Word::WaysEnded, &[]), look::WAYS_SAYS);
            b.close();
            return;
        }
        Path::Shown => {}
    }
    let ways = root.pages().stock_ways().expect("parts has a shown read");
    let tabs = ways.tabs();
    let name = match parts.head {
        Head::Way(index) => copy_of(&tabs.get(index).expect("current way is in the read").name),
        Head::Own => w.say(Word::HowItsOwn, &[]),
        Head::Reading => unreachable!("shown parts has a current or own way"),
    };
    text(b, Role::Text, &w.say(Word::HowWay, &[Hole::Name(name)]), look::HOW_WAY);
    b.close();
    super::ways::tabs(b, ways, w, bindings, project);
    if let Some(index) = ways.shown()
        && let Some(tab) = tabs.get(index)
    {
        b.open(Role::Group);
        b.name(&tab.name);
        b.style(look::WAYS_PANEL);
        if !tab.line.is_empty() {
            text(b, Role::Paragraph, &tab.line, look::WAYS_SAYS);
        }
        super::ways::drawing(b, &tab.path, ways.open(), w, bindings, project);
        if let Some(open) = ways.open()
            && let Some(stop) = tab.path.stops().get(open)
        {
            super::ways::card(b, stop, w, bindings, project);
        }
        b.open(Role::Slot);
        b.style(look::WAYS_FOOT);
        super::ways::legend(b, w);
        super::ways::act(b, ways, index, w, bindings, project);
        b.close();
        super::ways::refused(b, ways, w);
        b.close();
    }
    b.close();
}
pub(crate) fn rules(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings, project: u32) {
    let Some(parts) = how::parts(&how::facts(root, project)).rules else { return };
    let ways = root.pages().stock_ways().expect("parts names a shown read");
    let rules = ways.rules().expect("parts names shown rules");
    super::settings::section_head(b, &w.say(Word::HowRules, &[]));
    if parts.add {
        let name = w.say(Word::HowAddRule, &[]);
        crate::bindings::link(b, &name, bindings.insert(Act::Go { address: Address::Ask { project } }));
        b.style(look::HOW_ADD);
        b.text(Run::Plain, &name);
        b.close();
    }
    b.close();
    b.open(Role::Slot);
    b.style(look::HOW_RULES);
    b.open(Role::Table);
    b.style(look::HOW_TABLE);
    b.open(Role::Row);
    for word in [Word::HowPaths, Word::HowReview, Word::HowSignOff, Word::HowLands] {
        if word != Word::HowReview || parts.review {
            crate::cards::text(b, Role::ColumnHeader, &w.say(word, &[]));
        }
    }
    b.close();
    for row in rules.rows() {
        b.open(Role::Row);
        text(b, Role::Cell, row.paths(), look::HOW_PATHS);
        if parts.review {
            rule_cell(b, w, row.review(), false);
        }
        rule_cell(b, w, row.sign_off(), true);
        crate::cards::text(b, Role::Cell, row.lands());
        b.close();
    }
    b.close();
    b.close();
    if !rules.note().is_empty() {
        text(b, Role::Paragraph, rules.note(), look::HOW_RULES_NOTE);
    }
    b.close();
}
fn rule_cell(b: &mut Builder, w: &Words, words: &[u8], person: bool) {
    b.open(Role::Cell);
    if words.is_empty() {
        text(b, Role::Text, &w.say(Word::HowEmpty, &[]), look::HOW_EMPTY);
    } else if person {
        text(b, Role::Text, words, look::HOW_WHO_PERSON);
    } else {
        b.text(Run::Plain, words);
    }
    b.close();
}
pub(crate) fn roles(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings, project: u32) {
    if !how::parts(&how::facts(root, project)).roles {
        return;
    }
    let Some(page) = root.pages().roles() else { return };
    let temper_client_domain::pages::roles::Read::Drawn(wire::View::Roles(view)) = page.read() else {
        return;
    };
    super::settings::section_head(b, &w.say(Word::HowRoles, &[]));
    text(b, Role::Text, &w.say(Word::HowRolesLine, &[Hole::Name(project_name(root, w, project))]), look::HOW_WAY);
    b.close();
    b.open(Role::Slot);
    b.style(look::HOW_ROLES);
    for role in view.roles() {
        let card = role.card();
        b.open(Role::Disclosure);
        b.style(look::HOW_ROLE);
        b.name(card.name());
        b.description(card.who());
        crate::views::facts(b, card);
        b.open(Role::Paragraph);
        b.style(look::HOW_ROLE_MORE);
        let name = w.say(Word::HowRoleInstructions, &[]);
        crate::bindings::link(
            b,
            &name,
            bindings.insert(Act::Go { address: Address::Role { project, charter: role.charter() } }),
        );
        b.text(Run::Plain, &name);
        b.close();
        b.close();
        b.close();
    }
    b.close();
    b.close();
}
pub(crate) fn budgets(b: &mut Builder, root: &Domain, w: &Words, project: u32) {
    draw_budgets(b, w, &how::parts(&how::facts(root, project)));
}
pub(crate) fn draw_budgets(b: &mut Builder, w: &Words, parts: &how::Parts) {
    if parts.budget.is_none() && parts.standing.is_empty() {
        return;
    }
    super::settings::section_head(b, &w.say(Word::HowBudgets, &[]));
    if let Some(period) = parts.period {
        text(b, Role::Text, &period_words(w, period), look::HOW_WAY);
    }
    b.close();
    for row in &parts.standing {
        b.open(Role::Slot);
        b.style(look::HOW_ROW);
        crate::cards::text(b, Role::Text, &row.title);
        text(b, Role::Text, &w.say(Word::HowStanding, &[Hole::Number(row.left)]), look::SETTINGS_DETAIL);
        crate::cards::text(b, Role::Text, &w.say(Word::HowAmount, &[Hole::Number(row.budget)]));
        b.close();
    }
    if let Some(budget) = parts.budget {
        b.open(Role::Slot);
        b.style(look::HOW_ROW);
        crate::cards::text(b, Role::Text, &w.say(Word::SettingsGoals, &[]));
        text(b, Role::Text, &w.say(Word::HowBudgetLine, &[]), look::SETTINGS_DETAIL);
        let (word, number) = match budget {
            Budget::Amount(number) => (Word::HowAmount, number),
            Budget::UpTo(number) => (Word::HowUpTo, number),
        };
        crate::cards::text(b, Role::Text, &w.say(word, &[Hole::Number(number)]));
        b.close();
    }
    b.close();
}
/// The project's classes and live rows (client/pages/settings.md, 2, *What reaches you* and *Its rows*).
pub(crate) fn reaches(
    b: &mut Builder,
    root: &Domain,
    w: &Words,
    bindings: &mut Bindings,
    forge: &mut crate::forge::bindings::Bindings,
    project: u32,
    wall: skein_lib::Wall,
) {
    let parts = how::parts(&how::facts(root, project));
    b.open(Role::Slot);
    b.style(look::SETTINGS_SECTION);
    b.open(Role::Slot);
    b.style(look::SETTINGS_SECTION_HEAD);
    b.open(Role::Heading { level: 2 });
    b.style(look::SETTINGS_SECTION_TITLE);
    let title = w.say(Word::HowReaches, &[]);
    b.name(&title);
    b.text(Run::Plain, &title);
    let count = root.pages().away_focus();
    if count > 0 && root.pages().switcher().renaming().is_none() {
        b.focus(count);
    }
    b.close();
    let on = if parts.several { Word::HowReachesEach } else { Word::HowReachesOn };
    text(b, Role::Text, &w.say(on, &[Hole::Name(project_name(root, w, project))]), look::HOW_WAY);
    b.close();
    if parts.reaches {
        let classes = root.forge().classes(project).expect("parts names read classes");
        let columns = how::reaches(classes);
        b.open(Role::Slot);
        b.style(look::HOW_REACHES);
        column(b, w, classes, &columns.decide, Word::HowReachDecide, &w.say(Word::HowReachDecideNote, &[]));
        column(b, w, classes, &columns.ask, Word::HowReachAsk, &ask_note(w, classes, &columns.ask));
        column(b, w, classes, &columns.wait, Word::HowReachWait, &w.say(Word::HowReachWaitNote, &[]));
        b.close();
    }
    super::settings::questions(b, root, w, forge, project);
    super::settings::away_until(b, root, w, bindings, project, wall);
    b.close();
}

fn column(
    b: &mut Builder,
    w: &Words,
    classes: &temper_protocol_reaches::Classes,
    lines: &[how::Line],
    title: Word,
    note: &[u8],
) {
    if lines.is_empty() {
        return;
    }
    let name = w.say(title, &[]);
    b.open(Role::Group);
    b.style(look::HOW_REACH);
    b.name(&name);
    let mark = if title == Word::HowReachDecide { Who::Agent } else { Who::Person };
    chip(b, &name, mark);
    b.open(Role::List);
    for line in lines {
        crate::cards::text(b, Role::Item, &line_words(w, classes, *line));
    }
    b.close();
    text(b, Role::Paragraph, note, look::HOW_REACH_NOTE);
    b.close();
}

fn line_words(w: &Words, classes: &temper_protocol_reaches::Classes, line: how::Line) -> Box<[u8]> {
    use temper_protocol_reaches::{Kind, Subject};
    let word = match line.subject {
        Subject::Kind(kind) => match kind {
            Kind::Wording => Word::HowLineWording,
            Kind::Retry => Word::HowLineRetry,
            Kind::Implementation => Word::HowLineImplementation,
            Kind::Tests => Word::HowLineTests,
            Kind::Dependency => Word::HowLineDependency,
            Kind::Departure => Word::HowLineDeparture,
            Kind::Money => Word::HowLineMoney,
            Kind::Irreversible => Word::HowLineIrreversible,
            Kind::Rules => Word::HowLineRules,
            Kind::Other => Word::HowLineOther,
        },
        Subject::Set(number) => {
            let set = classes.set(number).expect("model names a present set");
            return line_under(w, classes, line.item, w.say(Word::HowLineSet, &[Hole::Name(set.name.clone())]));
        }
    };
    line_under(w, classes, line.item, w.say(word, &[]))
}

fn line_under(w: &Words, classes: &temper_protocol_reaches::Classes, item: u8, words: Box<[u8]>) -> Box<[u8]> {
    if let Some(index) = item.checked_sub(1)
        && let Some(item) = classes.items.get(usize::from(index))
        && !item.words.is_empty()
    {
        return w.say(Word::ReachUnder, &[Hole::Name(words), Hole::Name(item.words.clone())]);
    }
    words
}

fn ask_note(w: &Words, classes: &temper_protocol_reaches::Classes, lines: &[how::Line]) -> Box<[u8]> {
    use temper_protocol_reaches::Class;
    let mut objects = skein_lib::List::uncapped();
    let mut away = false;
    for line in lines {
        match line.class {
            Class::AskAway => {
                objects.push(crate::shared::object(w, line.subject, line.item, Some(classes))).expect("subjects grow");
            }
            Class::Ask => away = true,
            Class::Decide | Class::Wait => {}
        }
    }
    if objects.is_empty() {
        w.say(Word::HowReachAwayAll, &[])
    } else if !away {
        w.say(Word::HowReachAwayNone, &[])
    } else {
        w.say(Word::HowReachAwayBut, &[Hole::Name(crate::cards::proposal::comma_join(objects.as_slice()))])
    }
}

fn period_words(w: &Words, period: how::Period) -> Box<[u8]> {
    let word = w.say(if period.week { Word::HowWeek } else { Word::HowPeriod }, &[]);
    match period.next {
        Some(next) => w.say(
            Word::HowRenews,
            &[Hole::Name(word), Hole::Name(w.time(crate::words::TimeForm::Weekday, next, next).0)],
        ),
        None => word,
    }
}
pub(crate) fn follows(b: &mut Builder, root: &Domain, w: &Words, project: u32) {
    draw_follows(b, w, &how::landed(root, project));
}
/// Shared boxes (client/pages/settings.md, 2, *What follows a landing*).
pub fn draw_follows(b: &mut Builder, w: &Words, landed: &how::Landed) {
    if landed.follows.is_empty() && landed.tiers.is_none() {
        return;
    }
    super::settings::section_head(b, &w.say(Word::HowFollows, &[]));
    text(b, Role::Text, &w.say(Word::HowFollowsLine, &[]), look::HOW_WAY);
    b.close();
    b.open(Role::Slot);
    b.style(look::HOW_FOLLOWS);
    for follow in &landed.follows {
        box_head(
            b,
            &w.say(Word::HowFollowTitle, &[Hole::Name(follow.target.clone()), Hole::Name(follow.source.clone())]),
        );
        text(b, Role::Paragraph, &follow_words(w, follow), look::SETTINGS_DETAIL);
        let lag = temper_client_domain::forge::follows::lag(follow.lag);
        text(
            b,
            Role::Paragraph,
            &w.say(Word::HowFollowStatus, &[Hole::Name(lag), Hole::Name(follow.parity.clone())]),
            look::HOW_REACH_NOTE,
        );
        if let Some(stopped) = &follow.stopped {
            text(
                b,
                Role::Paragraph,
                &w.say(Word::HowFollowStopped, &[Hole::Name(stopped.clone())]),
                look::SETTINGS_DETAIL,
            );
        }
        b.open(Role::Slot);
        b.style(look::HOW_CHIPS);
        chip(b, &w.say(Word::HowHearsLanding, &[]), Who::Code);
        chip(b, &w.say(Word::HowAgentPorts, &[]), Who::Agent);
        chip(b, &w.say(Word::HowCodeChecks, &[]), Who::Code);
        b.close();
        b.close();
    }
    if let Some(tiers) = &landed.tiers {
        box_head(b, &w.say(Word::HowTiersTitle, &[Hole::Name(tiers.branch.clone())]));
        let mut names = skein_lib::List::uncapped();
        for tier in &tiers.tiers {
            let name = match &tier.checks {
                Some(checks) => {
                    w.say(Word::HowTierChecks, &[Hole::Name(tier.name.clone()), Hole::Name(checks.clone())])
                }
                None => tier.name.clone(),
            };
            names.push(name).expect("tiers grow");
        }
        let joined = join(names.as_slice(), b"; ", b"; ");
        text(
            b,
            Role::Paragraph,
            &w.say(Word::HowTiersWords, &[Hole::Name(tiers.branch.clone()), Hole::Name(joined)]),
            look::SETTINGS_DETAIL,
        );
        b.open(Role::Slot);
        b.style(look::HOW_CHIPS);
        chip(b, &w.say(Word::HowCodeRuns, &[]), Who::Code);
        chip(b, &w.say(Word::HowAgentRepairs, &[]), Who::Agent);
        b.close();
        b.close();
    }
    b.close();
    b.close();
}
fn box_head(b: &mut Builder, title: &[u8]) {
    b.open(Role::Group);
    b.style(look::HOW_FOLLOW);
    b.name(title);
    crate::cards::text(b, Role::Heading { level: 3 }, title);
}
fn join(items: &[Box<[u8]>], middle: &[u8], last: &[u8]) -> Box<[u8]> {
    let mut joined = Box::default();
    for (index, item) in items.iter().enumerate() {
        let separator = if index == 0 {
            b"".as_slice()
        } else if index.checked_add(1) == Some(items.len()) {
            last
        } else {
            middle
        };
        joined = concat(&[&joined, separator, item]);
    }
    joined
}
fn follow_words(w: &Words, follow: &how::Follow) -> Box<[u8]> {
    let touches = if follow.paths.is_empty() {
        Box::default()
    } else {
        w.say(Word::HowFollowTouches, &[Hole::Name(join(&follow.paths, b", ", b" or "))])
    };
    let owned = match &follow.owner {
        how::Owner::You => w.say(Word::HowFollowOwned, &[Hole::Name(w.say(Word::HowFollowYou, &[]))]),
        how::Owner::Named(name) => w.say(Word::HowFollowOwned, &[Hole::Name(name.clone())]),
        how::Owner::Unknown => Box::default(),
    };
    let source =
        w.say(Word::HowFollowBranch, &[Hole::Name(follow.source.clone()), Hole::Name(follow.source_branch.clone())]);
    let target =
        w.say(Word::HowFollowBranch, &[Hole::Name(follow.target.clone()), Hole::Name(follow.target_branch.clone())]);
    let main =
        w.say(Word::HowFollowMain, &[Hole::Name(source), Hole::Name(touches), Hole::Name(target), Hole::Name(owned)]);
    let start = if follow.code_starts {
        w.say(Word::HowFollowCode, &[])
    } else {
        match &follow.owner {
            how::Owner::You => w.say(Word::HowFollowYourWord, &[]),
            how::Owner::Named(name) => w.say(Word::HowFollowOwnerWord, &[Hole::Name(name.clone())]),
            how::Owner::Unknown => w.say(Word::HowFollowUnknownWord, &[]),
        }
    };
    let check = w.say(Word::HowFollowCheck, &[Hole::Name(follow.workflow.clone())]);
    concat(&[&main, b" ", &start, b" ", &check])
}
