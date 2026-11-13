//! The first standing list (temper/docs/design/client/tui/pages/README.md and temper/docs/design/client/tui/box.md).
use super::{Binds, Event, OptionItem, Options, Words, look, text};
use crate::{Own, Scheme, Tui, WalkStep, theme::ThemeChoice, view::front, words::Word};
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
    if let Some(start) = root.pages().start() {
        return Some(goal(root, start, words));
    }
    if tui.talk_list() {
        return Some(talk(root, words));
    }
    if tui.theme_list() {
        return Some(theme(tui, words));
    }
    if let Some(prompt) = walk(root, tui, words) {
        return Some(prompt);
    }
    if let Some(confirmation) = shared::confirmation(root) {
        return Some(Options {
            style: look::OPTION,
            why: None,
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
            style: look::OPTION,
            why: None,
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
    if root.pages().chat()?.talk().is_some() {
        return None;
    }
    let question = shared::question(root, words.view())?;
    question_list(question, tui.aside(), words)
}

/// A reply may stand behind typed words, but never behind another prompt.
pub(crate) fn reply(root: &Domain, tui: &Tui, words: &Words) -> Option<Options> {
    if shared::running(root, words.view()).is_some() || shared::question(root, words.view()).is_some() {
        return None;
    }
    let (turn, text) = crate::view::chat::read::last_reply(root)?;
    if tui.reply_aside() == Some(turn) {
        return None;
    }
    let choices = crate::reply_choices(&text)?;
    let mut items = List::uncapped();
    for choice in choices.choices {
        let number = u64::from(items.len()).checked_add(1).expect("choice number fits");
        items
            .push(OptionItem {
                name: numbered(&choice.words, number, words),
                words: choice.reason.unwrap_or_default(),
                selected: items.is_empty(),
                event: Event::Answer { words: choice.words },
            })
            .expect("reply choices fit memory");
    }
    let number = u64::from(items.len()).checked_add(1).expect("else number fits");
    items
        .push(OptionItem {
            name: numbered(&words.say(Word::Else, &[]), number, words),
            words: Box::default(),
            selected: false,
            event: Event::ReplyAside { turn },
        })
        .expect("else fits memory");
    Some(Options {
        style: look::OPTION,
        why: None,
        head: None,
        name: words.say(Word::Reply, &[]),
        items: items.into_boxed(),
        escape: Event::ReplyAside { turn },
    })
}

fn goal(root: &Domain, start: &temper_client_domain::pages::start_goal::StartGoal, words: &Words) -> Options {
    let head =
        words.say(Word::GoalHead, &[Hole::Name(copy_of(temper_client_domain::pages::goal::file_of(&start.draft)))]);
    let mut items = List::uncapped();
    if let Some(me) = root.pages().me(root.client()) {
        let ways = temper_client_view::pages::goals::ways(me, start.project, words.view());
        for chosen in [true, false] {
            for way in &ways {
                let selected = start.way == Some(way.number);
                if selected != chosen {
                    continue;
                }
                let number = u64::from(items.len()).checked_add(1).expect("way number fits");
                items
                    .push(OptionItem {
                        name: numbered(&way.name, number, words),
                        words: way.line.clone().unwrap_or_default(),
                        selected,
                        event: Event::Way { template: way.number },
                    })
                    .expect("project ways fit memory");
            }
        }
    }
    items
        .push(OptionItem {
            name: words.view().say(temper_client_view::words::Word::Cancel, &[]),
            words: Box::default(),
            selected: false,
            event: Event::Act { act: Act::StartCancel },
        })
        .expect("cancel fits memory");
    Options {
        style: look::OPTION,
        why: None,
        name: head.clone(),
        head: Some(head),
        items: items.into_boxed(),
        escape: Event::Act { act: Act::StartCancel },
    }
}

fn talk(root: &Domain, words: &Words) -> Options {
    let mut items = List::uncapped();
    let project = root.pages().project();
    let mut name = Box::default();
    if let Some(project) = project {
        name = super::project_name(root, Some(project)).unwrap_or_default();
        let areas = temper_client_domain::pages::team::areas(root, project);
        let shown = temper_client_domain::pages::team::shown_at(root, project, skein_lib::Wall::EPOCH);
        if let Some(facts) = shown.facts() {
            for role in &facts.roles {
                let held = crate::view::talk::areas(areas.as_ref(), role.task);
                items
                    .push(OptionItem {
                        name: role.name.clone(),
                        words: if held.is_empty() { role.now.clone().unwrap_or_default() } else { held },
                        selected: items.is_empty(),
                        event: Event::Act { act: Act::Go { address: Address::Talk { project, task: role.task } } },
                    })
                    .expect("roles fit memory");
            }
        }
    }
    let head = words.say(Word::TalkHead, &[Hole::Name(name)]);
    Options {
        style: look::OPTION,
        why: None,
        name: head.clone(),
        head: Some(head),
        items: items.into_boxed(),
        escape: Event::Dismiss,
    }
}
pub(crate) fn proposal(root: &Domain, tui: &Tui, words: &Words) -> Option<Options> {
    let chat = root.pages().chat()?;
    chat.talk()?;
    for card in &chat.cards {
        if tui.proposal_aside(&card.object) || card.key.is_some() {
            continue;
        }
        let Some(jig_client::contract::ReadValue::Proposal(proposal)) = &card.page else { continue };
        match jig_client::facts::offer(proposal.acts.as_slice(), jig_client::contract::ActKind::Accept) {
            Some(jig_client::contract::Admission::Allowed | jig_client::contract::Admission::Bounded { .. }) => {}
            Some(jig_client::contract::Admission::Refused(_) | jig_client::contract::Admission::Proposed { .. })
            | None => continue,
        }
        let Some(head) = shared::proposal_line(card, words.view()) else { continue };
        let object = card.object;
        return Some(Options {
            style: look::OPTION,
            why: None,
            name: head.clone(),
            head: Some(head),
            items: Box::new([
                OptionItem {
                    name: numbered(&words.say(Word::AcceptIt, &[]), 1, words),
                    words: Box::default(),
                    selected: true,
                    event: Event::Accept { object, revision: None },
                },
                OptionItem {
                    name: numbered(&words.say(Word::NotNow, &[]), 2, words),
                    words: Box::default(),
                    selected: false,
                    event: Event::Aside { object },
                },
            ]),
            escape: Event::Aside { object },
        });
    }
    None
}

fn theme(tui: &Tui, words: &Words) -> Options {
    let mut items = List::with_capacity(3);
    for choice in ThemeChoice::ALL {
        let name = match choice {
            ThemeChoice::Auto => Word::ThemeAuto,
            ThemeChoice::Light => Word::ThemeLight,
            ThemeChoice::Dark => Word::ThemeDark,
        };
        let description = if choice == ThemeChoice::Auto {
            words.say(
                match tui.terminal_scheme() {
                    Some(Scheme::Light) => Word::ThemeAutoLight,
                    Some(Scheme::Dark) => Word::ThemeAutoDark,
                    None => Word::ThemeAutoNone,
                },
                &[],
            )
        } else {
            Box::default()
        };
        items
            .push(OptionItem {
                name: words.say(name, &[]),
                words: description,
                selected: choice == tui.theme_choice(),
                event: Event::Theme { choice: Some(choice) },
            })
            .expect("three theme choices");
    }
    Options {
        style: look::OPTION,
        why: None,
        head: Some(words.say(Word::ThemeHead, &[])),
        name: words.say(Word::ThemeHead, &[]),
        items: items.into_boxed(),
        escape: Event::Theme { choice: None },
    }
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
            style: look::COMMAND,
            why: None,
            head: None,
            name: words.say(Word::Commands, &[]),
            items: items.into_boxed(),
            escape: Event::Dismiss,
        })
    }
}
// Both the list and /help read this order, including front's /log before /quit.
pub(super) fn command_items(root: &Domain, tui: &Tui, words: &Words) -> Box<[OptionItem]> {
    let commands = crate::offered(root, words);
    let own = crate::offered_own(root, tui);
    let capacity = commands.len().checked_add(own.len()).expect("commands fit").checked_add(1).expect("log fits");
    let mut items = List::with_capacity(u32::try_from(capacity).expect("commands fit"));
    for command in &own {
        match command {
            Own::Draft | Own::Change | Own::Talk | Own::Ask => {
                items.push(own_item(*command, words)).expect("own commands counted");
            }
            Own::Brief | Own::Needs | Own::Open | Own::Tell | Own::Theme => {}
        }
    }
    for command in commands {
        if command == Command::Help {
            items.push(own_item(Own::Theme, words)).expect("theme counted");
        }
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
                name: copy_of(crate::command_name(command)),
                words: description(command, words, tui.front()),
                selected: false,
                event: Event::Command { command },
            })
            .expect("root commands counted");
        if command == Command::Chats {
            for command in &own {
                match command {
                    Own::Brief | Own::Needs | Own::Open | Own::Tell => {
                        items.push(own_item(*command, words)).expect("own commands counted");
                    }
                    Own::Draft | Own::Change | Own::Talk | Own::Ask | Own::Theme => {}
                }
            }
        }
    }
    items.into_boxed()
}
fn own_item(command: Own, words: &Words) -> OptionItem {
    let word = match command {
        Own::Draft => Word::DraftCommand,
        Own::Change => Word::ChangeCommand,
        Own::Talk => Word::TalkCommand,
        Own::Ask => Word::AskTemper,
        Own::Brief => Word::BriefCommand,
        Own::Needs => Word::NeedsCommand,
        Own::Open => Word::OpenCommand,
        Own::Tell => Word::TellCommand,
        Own::Theme => Word::ThemeCommand,
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
            shared::NeedAct::Answer(question) => return Some(question_options(question, words, true, prompt.why)),
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
            style: look::OPTION,
            why: prompt.why,
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
    Some(question_options(question, words, false, None))
}
fn question_options(question: shared::Question, words: &Words, walking: bool, why: Option<Box<[u8]>>) -> Options {
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
        style: look::OPTION,
        why,
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
    if let Some(why) = &list.why {
        text(b, Role::Text, why, Some(look::WHY));
    }
    b.open(Role::Menu);
    b.name(&list.name);
    b.state(State::Options);
    for item in &list.items {
        b.open(Role::MenuItem);
        b.style(list.style);
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
