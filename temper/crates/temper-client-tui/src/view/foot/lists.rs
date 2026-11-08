//! The first standing list (client/README.md, 6.14, The commands and The chat's question).
use super::{Binds, Event, OptionItem, Options, Words, look, text};
use crate::{Tui, words::Word};
use alloc::boxed::Box;
use skein_lib::{List, bytes::copy_of};
use skein_ui::{Builder, Role, State, phrases::Hole};
use temper_client_domain::{
    Domain,
    pages::{
        Act, needs,
        terminal::{Command, Listing},
    },
};
use temper_client_view::shared;

pub(super) fn read(root: &Domain, tui: &Tui, words: &Words, value: &[u8]) -> Option<Options> {
    if let Some(confirmation) = shared::confirmation(root) {
        return Some(Options {
            head: Some(copy_of(confirmation.head)),
            name: copy_of(confirmation.head),
            items: Box::new([
                OptionItem {
                    name: copy_of(confirmation.take),
                    words: Box::default(),
                    selected: false,
                    event: Event::Act { act: Act::Confirm },
                },
                OptionItem {
                    name: copy_of(confirmation.keep),
                    words: Box::default(),
                    selected: false,
                    event: Event::Act { act: Act::Cancel },
                },
            ]),
            escape: Event::Act { act: Act::Cancel },
        });
    }
    if let Some(listing) = root.pages().listing() {
        let task = match root.pages().chat() {
            Some(chat) => chat.task(root.client()),
            None => None,
        };
        let choices = match listing {
            Listing::Model => shared::models(root, task, words.view()),
            Listing::Effort => shared::efforts(root, task, words.view()),
            Listing::Chats => shared::chats(root, words.view()),
        };
        let mut items = List::with_capacity(u32::try_from(choices.len()).expect("choices fit"));
        for choice in choices {
            items
                .push(OptionItem {
                    name: choice.name,
                    words: choice.words,
                    selected: choice.selected,
                    event: Event::Act { act: choice.act },
                })
                .expect("choices fit");
        }
        return Some(Options {
            head: Some(copy_of(shared::list_head(listing))),
            name: copy_of(shared::list_head(listing)),
            items: items.into_boxed(),
            escape: Event::Dismiss,
        });
    }
    if let Some(commands) = commands(root, words, value) {
        return Some(commands);
    }
    if root.pages().chat().is_none() || !value.is_empty() {
        return None;
    }
    let question = shared::question(root, words.view())?;
    question_list(question, tui.aside(), words)
}

#[expect(clippy::disallowed_methods, reason = "commands reject Unicode whitespace in validated UTF-8 composer words")]
fn commands(root: &Domain, words: &Words, value: &[u8]) -> Option<Options> {
    if !value.starts_with(b"/") {
        return None;
    }
    for character in core::str::from_utf8(value).expect("composer words are UTF-8").chars() {
        if character.is_whitespace() {
            return None;
        }
    }
    let mut items = List::with_capacity(13);
    for command in crate::offered(root, words) {
        if command.name().starts_with(value) {
            items
                .push(OptionItem {
                    name: copy_of(command.name()),
                    words: description(command, words),
                    selected: false,
                    event: Event::Command { command },
                })
                .expect("at most Command::ALL commands");
        }
    }
    if items.is_empty() {
        None
    } else {
        Some(Options {
            head: None,
            name: words.say(Word::Commands, &[]),
            items: items.into_boxed(),
            escape: Event::Dismiss,
        })
    }
}
pub(super) fn description(command: Command, words: &Words) -> Box<[u8]> {
    if command == Command::Exit { words.say(Word::Exit, &[]) } else { copy_of(shared::description(command)) }
}
pub(crate) fn question_list(
    question: shared::Question,
    aside: Option<jig_client::contract::ObjectKey>,
    words: &Words,
) -> Option<Options> {
    if Some(question.object) == aside {
        return None;
    }
    let mut items = List::with_capacity(
        u32::try_from(question.choices.len()).expect("parsed choices fit").checked_add(1).expect("else fits"),
    );
    for primary in [true, false] {
        for (index, choice) in question.choices.iter().enumerate() {
            let selected = Some(index) == question.primary;
            if selected != primary {
                continue;
            }
            let numbered = words.say(
                Word::Numbered,
                &[Hole::Number(u64::from(items.len()).checked_add(1).expect("choice number fits"))],
            );
            items
                .push(OptionItem {
                    name: super::notes::join(&[&numbered, &choice.words]),
                    words: Box::default(),
                    selected,
                    event: Event::Act {
                        act: Act::Needs(needs::Act::Choose { object: question.object, words: choice.words.clone() }),
                    },
                })
                .expect("parsed choices fit");
        }
    }
    let numbered =
        words.say(Word::Numbered, &[Hole::Number(u64::from(items.len()).checked_add(1).expect("else number fits"))]);
    items
        .push(OptionItem {
            name: super::notes::join(&[&numbered, &words.say(Word::Else, &[])]),
            words: Box::default(),
            selected: false,
            event: Event::Aside { object: question.object },
        })
        .expect("else fits");
    Some(Options {
        name: question.head.clone(),
        head: Some(question.head),
        items: items.into_boxed(),
        escape: Event::Aside { object: question.object },
    })
}
pub(super) fn draw(b: &mut Builder, list: &Options, binds: &mut Binds) {
    b.open(Role::Region);
    b.region(look::LIST);
    if let Some(head) = &list.head {
        text(b, Role::Text, head, Some(look::LIST_HEAD));
    }
    b.open(Role::Menu);
    b.name(&list.name);
    b.state(State::Options);
    for item in &list.items {
        b.open(Role::MenuItem);
        b.style(if list.head.is_none() { look::COMMAND } else { look::OPTION });
        b.name(&item.name);
        b.description(&item.words);
        if item.selected {
            b.state(State::Selected);
        }
        b.bind(binds.bind(item.event.clone()));
        b.close();
    }
    b.close();
    b.close();
}
