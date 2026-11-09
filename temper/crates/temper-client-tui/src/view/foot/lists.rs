//! The first standing list (temper/docs/design/client/README.md, 6.14 and 6.17).
use super::{Binds, Event, OptionItem, Options, Words, look, text};
use crate::{Own, Tui, WalkStep, view::front, words::Word};
use alloc::boxed::Box;
use skein_lib::{List, bytes::copy_of};
use skein_ui::{Builder, Role, State, phrases::Hole};
use temper_client_domain::{
    Address, Domain,
    address::{self, Filter},
    pages::{
        Act, needs,
        terminal::{Command, Listing},
    },
};
use temper_client_view::shared;

pub(super) fn read(root: &Domain, tui: &Tui, words: &Words, value: &[u8]) -> Option<Options> {
    if let Some(quit) = front::quit(root, tui, words) {
        return Some(quit);
    }
    if let Some(prompt) = walk(root, tui, words) {
        return Some(prompt);
    }
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
    if let Some(commands) = commands(root, tui, words, value) {
        return Some(commands);
    }
    if root.pages().chat().is_none() || !value.is_empty() {
        return None;
    }
    let question = shared::question(root, words.view())?;
    question_list(question, tui.aside(), words)
}

#[expect(clippy::disallowed_methods, reason = "commands reject Unicode whitespace in validated UTF-8 composer words")]
fn commands(root: &Domain, tui: &Tui, words: &Words, value: &[u8]) -> Option<Options> {
    if crate::landing(root) || !value.starts_with(b"/") {
        return None;
    }
    for character in core::str::from_utf8(value).expect("composer words are UTF-8").chars() {
        if character.is_whitespace() {
            return None;
        }
    }
    let commands = command_items(root, tui, words);
    let mut items = List::with_capacity(u32::try_from(commands.len()).expect("commands fit"));
    for item in commands {
        if item.name.starts_with(value) {
            items.push(item).expect("offered commands counted");
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
// Both the list and /help read this order, including front's /log before /exit.
pub(super) fn command_items(root: &Domain, tui: &Tui, words: &Words) -> Box<[OptionItem]> {
    let commands = crate::offered(root, words);
    let own = crate::offered_own(root, tui);
    let capacity = commands.len().checked_add(own.len()).expect("commands fit").checked_add(1).expect("log fits");
    let mut items = List::with_capacity(u32::try_from(capacity).expect("commands fit"));
    for command in &own {
        match command {
            Own::Change | Own::Ask => items.push(own_item(*command, words)).expect("own commands counted"),
            Own::Brief | Own::Needs | Own::Open => {}
        }
    }
    for command in commands {
        if tui.front() && command == Command::Exit {
            items
                .push(OptionItem {
                    name: copy_of(crate::LOG_COMMAND),
                    words: words.say(Word::LogCommand, &[]),
                    selected: false,
                    event: Event::Log,
                })
                .expect("log counted");
        }
        items
            .push(OptionItem {
                name: copy_of(command.name()),
                words: description(command, words, tui.front()),
                selected: false,
                event: Event::Command { command },
            })
            .expect("root commands counted");
        if command == Command::Chats {
            for command in &own {
                match command {
                    Own::Brief | Own::Needs | Own::Open => {
                        items.push(own_item(*command, words)).expect("own commands counted");
                    }
                    Own::Change | Own::Ask => {}
                }
            }
        }
    }
    items.into_boxed()
}
fn own_item(command: Own, words: &Words) -> OptionItem {
    let word = match command {
        Own::Change => Word::ChangeCommand,
        Own::Ask => Word::AskTemper,
        Own::Brief => Word::BriefCommand,
        Own::Needs => Word::NeedsCommand,
        Own::Open => Word::OpenCommand,
    };
    OptionItem {
        name: copy_of(command.name()),
        words: words.say(word, &[]),
        selected: false,
        event: Event::Own { command },
    }
}
pub(crate) fn walk(root: &Domain, tui: &Tui, words: &Words) -> Option<Options> {
    let walk = tui.walk()?;
    for prompt in shared::need_prompts(root, words.view()) {
        if walk.shown(&prompt.object) {
            continue;
        }
        let (name, address) = match prompt.act {
            shared::NeedAct::Answer(question) => return Some(question_options(question, words, true)),
            shared::NeedAct::Review { words, address } => (words, address),
            shared::NeedAct::Web => {
                let path =
                    address::print(&Address::Inbox { filter: Filter { project: prompt.project, ..Filter::ALL } });
                (words.say(Word::OpenOnWeb, &[]), super::notes::join(&[b"http://", tui.address(), &path]))
            }
        };
        let mut items = List::with_capacity(2);
        items
            .push(OptionItem {
                name: numbered(&name, 1, words),
                words: Box::default(),
                selected: false,
                event: Event::Walk { object: prompt.object, step: WalkStep::Open { address } },
            })
            .expect("open counted");
        next(&mut items, prompt.object, words);
        return Some(Options {
            name: prompt.line.words.clone(),
            head: Some(prompt.line.words),
            items: items.into_boxed(),
            escape: Event::EndWalk,
        });
    }
    None
}
fn numbered(name: &[u8], number: u64, words: &Words) -> Box<[u8]> {
    super::notes::join(&[&words.say(Word::Numbered, &[Hole::Number(number)]), name])
}
fn next(items: &mut List<OptionItem>, object: jig_client::contract::ObjectKey, words: &Words) {
    items
        .push(OptionItem {
            name: numbered(
                &words.say(Word::Next, &[]),
                u64::from(items.len()).checked_add(1).expect("next number fits"),
                words,
            ),
            words: Box::default(),
            selected: false,
            event: Event::Walk { object, step: WalkStep::Next },
        })
        .expect("next counted");
}
pub(super) fn description(command: Command, words: &Words, front: bool) -> Box<[u8]> {
    if command == Command::Exit {
        words.say(if front { Word::ExitFront } else { Word::Exit }, &[])
    } else {
        copy_of(shared::description(command))
    }
}
pub(crate) fn question_list(
    question: shared::Question,
    aside: Option<jig_client::contract::ObjectKey>,
    words: &Words,
) -> Option<Options> {
    if Some(question.object) == aside {
        return None;
    }
    Some(question_options(question, words, false))
}
fn question_options(question: shared::Question, words: &Words, walking: bool) -> Options {
    let mut items = List::with_capacity(
        u32::try_from(question.choices.len()).expect("parsed choices fit").checked_add(2).expect("else and next fit"),
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
                    event: if walking {
                        Event::Walk { object: question.object, step: WalkStep::Choose { words: choice.words.clone() } }
                    } else {
                        Event::Act {
                            act: Act::Needs(needs::Act::Choose {
                                object: question.object,
                                words: choice.words.clone(),
                            }),
                        }
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
    if walking {
        next(&mut items, question.object, words);
    }
    Options {
        name: question.head.clone(),
        head: Some(question.head),
        items: items.into_boxed(),
        escape: if walking { Event::EndWalk } else { Event::Aside { object: question.object } },
    }
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
