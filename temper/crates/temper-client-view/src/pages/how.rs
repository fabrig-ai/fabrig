//! How work runs' heading, path, reaches, roles and budget (temper/docs/design/client/pages/settings.md, 2).
use crate::{
    frame::Bindings,
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

pub(crate) fn head(root: &Domain, w: &Words, project: u32) -> (Box<[u8]>, Box<[u8]>) {
    let change = w.say(Word::HowChange, &[]);
    match how::parts(&how::facts(root, project)).head {
        Head::Reading => (w.say(Word::SettingsHowHeading, &[Hole::Name(project_name(root, w, project))]), change),
        Head::Own => {
            let ways = root.pages().stock_ways().expect("parts names a shown read");
            let (headline, lede) = match ways.words() {
                Some(words) => (copy_of(words.headline()), copy_of(words.lede())),
                None => {
                    (w.say(Word::HowOwn, &[]), w.say(Word::HowOwnLine, &[Hole::Name(project_name(root, w, project))]))
                }
            };
            (headline, concat(&[&lede, b" ", &change]))
        }
        Head::Way(index) => {
            let ways = root.pages().stock_ways().expect("parts names a read way");
            let tabs = ways.tabs();
            let way = tabs.get(index).expect("the current way is in the read");
            let headline = if way.headline.is_empty() { &way.name } else { &way.headline };
            let lede = w
                .say(Word::HowStockLine, &[Hole::Name(project_name(root, w, project)), Hole::Name(copy_of(&way.name))]);
            (copy_of(headline), concat(&[&lede, b" ", &way.line, b" ", &change]))
        }
    }
}
fn project_name(root: &Domain, w: &Words, project: u32) -> Box<[u8]> {
    match super::project_name(root.pages().me(root.client()), project) {
        Some(name) => name,
        None => w.say(Word::Project, &[Hole::Number(u64::from(project))]),
    }
}
pub(crate) fn ask(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings, project: u32) {
    if how::parts(&how::facts(root, project)).ask {
        super::settings::button_to(
            b,
            &w.say(Word::HowAsk, &[]),
            bindings.insert(Act::Go { address: Address::Ask { project } }),
            true,
        );
    }
}
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
        crate::link(b, &name, bindings.insert(Act::Go { address: Address::Ask { project } }));
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
        crate::link(b, &name, bindings.insert(Act::Go { address: Address::Role { project, charter: role.charter() } }));
        b.text(Run::Plain, &name);
        b.close();
        b.close();
        b.close();
    }
    b.close();
    b.close();
}
pub(crate) fn budgets(b: &mut Builder, root: &Domain, w: &Words, project: u32) {
    let Some(budget) = how::parts(&how::facts(root, project)).budget else { return };
    super::settings::section_head(b, &w.say(Word::HowBudgets, &[]));
    b.close();
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
    b.close();
}
fn text(b: &mut Builder, role: Role, words: &[u8], style: skein_ui::Style) {
    b.open(role);
    b.style(style);
    b.name(words);
    b.text(Run::Plain, words);
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
    let style = if title == Word::HowReachDecide { look::HOW_WHO_AGENT } else { look::HOW_WHO_PERSON };
    text(b, Role::Text, &name, style);
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
