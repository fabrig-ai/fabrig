//! How work runs' heading, path and budget (temper/docs/design/client/README.md, 5.13).
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

pub(crate) fn head(root: &Domain, w: &Words, project: u32) -> (Box<[u8]>, Box<[u8]>) {
    let change = w.say(Word::HowChange, &[]);
    match how::parts(&how::facts(root, project)).head {
        Head::Reading => (w.say(Word::SettingsHowHeading, &[Hole::Name(project_name(root, w, project))]), change),
        Head::Own => (
            w.say(Word::HowOwn, &[]),
            concat(&[&w.say(Word::HowOwnLine, &[Hole::Name(project_name(root, w, project))]), b" ", &change]),
        ),
        Head::Way(index) => {
            let ways = root.pages().stock_ways().expect("parts names a read way");
            let tabs = ways.tabs();
            let way = tabs.get(index).expect("the current way is in the read");
            (copy_of(&way.name), concat(&[&way.line, b" ", &change]))
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
