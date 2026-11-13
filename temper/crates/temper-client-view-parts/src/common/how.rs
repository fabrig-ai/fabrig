//! Shared setup items (temper/docs/design/client/view-crates.md, 3.3).
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
        how::{self, Head},
    },
};

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

#[must_use]
pub fn project_name(root: &Domain, w: &Words, project: u32) -> Box<[u8]> {
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

pub fn text(b: &mut Builder, role: Role, words: &[u8], style: skein_ui::Style) {
    b.open(role);
    b.style(style);
    b.name(words);
    b.text(Run::Plain, words);
    b.close();
}

#[derive(Debug)]
pub enum Who {
    None,
    Code,
    Agent,
    Person,
}

pub fn chip(b: &mut Builder, words: &[u8], who: Who) {
    let style = match who {
        Who::None => look::HOW_WHO_NONE,
        Who::Code => look::HOW_WHO_CODE,
        Who::Agent => look::HOW_WHO_AGENT,
        Who::Person => look::HOW_WHO_PERSON,
    };
    text(b, Role::Text, words, style);
}
