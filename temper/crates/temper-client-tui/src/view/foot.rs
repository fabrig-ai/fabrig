//! The composer, quiet line and lists (temper/docs/design/client/tui/pages/README.md, 1 and temper/docs/design/client/tui/box.md, The foot).
use super::{Binds, chat as conversation, front, text};
use crate::{
    Event, Local, Tui, look,
    words::{Word, Words},
};
use alloc::boxed::Box;
use jig_client::Link;
use skein_lib::{
    List, Wall,
    bytes::{concat, copy_of},
};
use skein_ui::{Binding, Builder, Role, State, Style, phrases::Hole};
use temper_client_domain::{
    Domain,
    pages::{chat, chats, needs::Count},
};
use temper_client_view::{
    shared::{self, TerminalWord},
    words::Word as Shared,
};

pub(crate) mod lists;
mod notes;
pub(crate) use notes::{Notes, draw_notes, read_notes};

#[derive(Debug)]
#[expect(clippy::struct_excessive_bools, reason = "independent view facts read from the root, not a lifecycle")]
pub(crate) struct Foot {
    pub rule: Option<Rule>,
    pub state: Box<[u8]>,
    pub style: Style,
    pub composer: Option<Composer>,
    pub running: Option<Box<[u8]>>,
    pub project: Option<Box<[u8]>>,
    pub paused: bool,
    pub session: bool,
    pub launchpad: bool,
    pub needs: Count,
    pub notice: Option<Box<[u8]>>,
    pub box_state: Option<(Box<[u8]>, Style)>,
    pub playground: bool,
    pub closed: Option<Box<[u8]>>,
    pub columns: u16,
    pub pane: Option<chat::document::Mode>,
    pub goal: Option<Box<[u8]>>,
    pub list: Option<Options>,
    pub reply: Box<[Event]>,
}
#[derive(Debug)]
pub(crate) struct Rule {
    pub project: Box<[u8]>,
    pub title: Box<[u8]>,
    pub task: u64,
    pub place: Box<[u8]>,
}
#[derive(Debug)]
pub(crate) struct Composer {
    pub name: Box<[u8]>,
    pub placeholder: Box<[u8]>,
    pub value: Box<[u8]>,
    pub written: u64,
    pub focus: u64,
    pub refusal: Option<Box<[u8]>>,
}
#[derive(Debug)]
pub(crate) struct Options {
    pub style: Style,
    pub head: Option<Box<[u8]>>,
    pub name: Box<[u8]>,
    pub items: Box<[OptionItem]>,
    pub escape: Event,
}
#[derive(Debug)]
pub(crate) struct OptionItem {
    pub name: Box<[u8]>,
    pub words: Box<[u8]>,
    pub selected: bool,
    pub event: Event,
}
fn state(tui: &Tui, root: &Domain) -> (Shared, Style) {
    match root.client().link().state {
        Link::Live => {
            if root.pages().me(root.client()).is_some() {
                (Shared::Live, look::LIVE)
            } else {
                (Shared::Connecting, look::WAITING)
            }
        }
        Link::Starting => (Shared::Connecting, look::WAITING),
        Link::Offline => (Shared::Offline, look::DOWN),
        Link::OutOfDate => (Shared::OutOfDate, look::DOWN),
        Link::SignedOut => {
            if tui.local() == Local::Signing {
                (Shared::Connecting, look::WAITING)
            } else {
                (Shared::SignedOut, look::DOWN)
            }
        }
    }
}

#[expect(clippy::manual_map, reason = "the step subset spells out optional wording without closures")]
#[expect(clippy::too_many_lines, reason = "the foot and launchpad retain their existing section layout")]
pub(crate) fn read(root: &Domain, tui: &Tui, words: &Words, _wall: Wall) -> Foot {
    let (state, style) = state(tui, root);
    let chat = root.pages().chat();
    let task = match chat {
        Some(chat) => chat.task(root.client()),
        None => None,
    };
    let project = match task {
        Some(task) => Some(task.project),
        None => root.pages().project(),
    };
    let value = match chat {
        Some(chat) => copy_of(&chat.words),
        None => match root.pages().chats() {
            Some(chats) => copy_of(&chats.words),
            None => copy_of(tui.command_words()),
        },
    };
    let paused = match (task, root.pages().me(root.client())) {
        (Some(task), Some(me)) => chat::paused(&task.chip, me),
        (Some(_), None) | (None, Some(_) | None) => false,
    };
    let session = match (task, root.pages().me(root.client())) {
        (Some(task), Some(me)) => chats::drafting(me, task.project, &task.chip.executor),
        (Some(_), None) | (None, Some(_) | None) => false,
    };
    let ended = match task {
        Some(task) => task.result.is_some() || chats::ended(&task.chip),
        None => false,
    };
    let placeholder = if chat.is_some() {
        words.say(if ended { Word::Ended } else { Word::Reply }, &[])
    } else if project.is_some() {
        words.say(Word::BoxStart, &[])
    } else {
        shared::say(words.view(), TerminalWord::Commands)
    };
    let signing_in = root.client().link().state == Link::SignedOut && tui.local() != Local::Signing;
    let mut list = if signing_in { None } else { lists::read(root, tui, words, &value) };
    let pane = match chat {
        Some(chat) => chat.draft.is_some() || chat.writing,
        None => false,
    };
    let standing =
        if signing_in || list.is_some() || !(session || pane) { None } else { lists::reply(root, tui, words) };
    let mut reply = List::uncapped();
    if let Some(standing) = standing {
        for item in &standing.items {
            reply.push(item.event.clone()).expect("reply events fit memory");
        }
        if value.is_empty() {
            list = Some(standing);
        }
    }
    let rule = match (chat, task) {
        (Some(chat), Some(task)) => {
            let mut project = Box::default();
            if let Some(me) = root.pages().me(root.client()) {
                for item in &me.projects {
                    if item.project == task.project {
                        project = copy_of(item.name.bytes());
                        break;
                    }
                }
            }
            Some(Rule {
                project,
                title: conversation::cut(&shared::heading(task.chip.title.bytes()), words),
                task: chat.task,
                place: crate::place(tui.address(), tui.front() || tui.boxed()),
            })
        }
        (Some(_), None) | (None, Some(_) | None) => None,
    };
    Foot {
        rule,
        state: if session && root.client().link().state == Link::Live {
            Box::default()
        } else {
            words.say(Word::State, &[Hole::Name(words.view().say(state, &[]))])
        },
        style,
        composer: if signing_in {
            None
        } else {
            Some(Composer {
                name: words.say(if chat.is_some() { Word::Message } else { Word::AskTemper }, &[]),
                placeholder,
                value,
                written: tui.box_written(),
                focus: tui.focus(),
                refusal: if root.pages().start().is_some() {
                    shared::start_goal_refusal(root, words.view())
                } else {
                    shared::refusal(root, words.view())
                },
            })
        },
        running: if chat.is_some() { shared::running(root, words.view()) } else { None },
        notice: match tui.notice() {
            Some(notice) => Some(words.notice(notice)),
            None => None,
        },
        box_state: front::box_state(tui, words),
        playground: front::playground(root, project),
        project: project_name(root, project),
        paused,
        session,
        launchpad: chat.is_none(),
        needs: match project {
            Some(project) => root.pages().needs().count(root.client(), root.forge().approvals(), project),
            None => Count::default(),
        },
        closed: if project.is_some() && root.pages().choice().context.is_some() {
            Some(shared::closed_line(root, task, words.view()))
        } else {
            None
        },
        columns: tui.columns(),
        pane: match chat {
            Some(chat) => super::pane::shown(chat, tui.columns()),
            None => None,
        },
        goal: if super::goal_offered(root) {
            Some(words.say(Word::PaneGoal, &[]))
        } else {
            match chat {
                Some(chat) => match chat.goal_chip(root) {
                    Some(chip) => Some(
                        words.say(Word::PaneLinkedGoal, &[Hole::Name(conversation::cut(chip.title.bytes(), words))]),
                    ),
                    None => None,
                },
                None => None,
            }
        },
        list,
        reply: reply.into_boxed(),
    }
}

fn project_name(root: &Domain, number: Option<u32>) -> Option<Box<[u8]>> {
    let me = root.pages().me(root.client())?;
    let number = match number {
        Some(number) => number,
        None => me.projects.get(0)?.project,
    };
    for project in &me.projects {
        if project.project == number {
            return Some(copy_of(project.name.bytes()));
        }
    }
    None
}
#[expect(clippy::disallowed_methods, reason = "display widths read characters from validated UTF-8 view words")]
fn width(bytes: &[u8]) -> u32 {
    let mut columns = 0_u32;
    for character in core::str::from_utf8(bytes).expect("view words are UTF-8").chars() {
        columns = columns.saturating_add(u32::from(skein_ui_screen::width(character)));
    }
    columns
}
fn rule(b: &mut Builder, part: &Foot, words: &Words) {
    let Some(rule) = &part.rule else { return };
    let number = concat(&[b" ", &words.view().say(Shared::Chat, &[Hole::Number(rule.task)])]);
    let between = words.say(Word::Between, &[]);
    let on = words.say(Word::On, &[]);
    let title_width = width(&rule.title).saturating_add(width(&number)).saturating_add(6);
    let project_width = title_width.saturating_add(width(&rule.project)).saturating_add(width(&between));
    let full_width = project_width.saturating_add(width(&on)).saturating_add(width(&rule.place));
    let place = part.columns == 0 || full_width <= u32::from(part.columns);
    let project = part.columns == 0 || project_width <= u32::from(part.columns);
    b.open(Role::Heading { level: 1 });
    if project {
        rule_part(b, &rule.project, look::NAME);
        rule_part(b, &between, look::FAINT);
    }
    rule_part(b, &rule.title, look::NAME);
    rule_part(b, &number, look::HINT);
    if place {
        rule_part(b, &on, look::HINT);
        rule_part(b, &rule.place, look::ADDRESS);
    }
    b.close();
}
fn rule_part(b: &mut Builder, bytes: &[u8], paint: Style) {
    b.open(Role::Text);
    b.style(paint);
    b.style(look::RULE_TITLE);
    b.text(skein_ui::Run::Plain, bytes);
    b.close();
}
#[expect(clippy::too_many_lines, reason = "the quiet line measures its ordered parts before dropping them by priority")]
fn quiet(b: &mut Builder, part: &Foot, words: &Words) {
    let separator = words.say(Word::Separator, &[]);
    let needing = if part.needs.count > 0 {
        words.say(
            if part.needs.cut { Word::NeedingCut } else { Word::Needing },
            &[Hole::Number(u64::from(part.needs.count))],
        )
    } else {
        Box::default()
    };
    let mut right = List::with_capacity(7);
    let mut shortcut = None;
    let mut goal = None;
    if let Some(notice) = &part.notice {
        right.push((copy_of(notice), look::SIGNAL)).expect("one notice fits");
    } else {
        if let Some((state, style)) = &part.box_state {
            right.push((copy_of(state), *style)).expect("seven quiet line parts");
        }
        if part.playground {
            right.push((words.say(Word::Playground, &[]), look::HINT)).expect("seven quiet line parts");
        }
        if part.paused {
            right.push((shared::say(words.view(), TerminalWord::Paused), look::HINT)).expect("seven quiet line parts");
        }
        if let Some(mode) = part.pane {
            let (state, key) = match mode {
                chat::document::Mode::Chat => (Word::PaneFolded, Word::PaneOpens),
                chat::document::Mode::Split => (Word::PaneHalf, Word::PaneWidens),
                chat::document::Mode::Draft => (Word::PaneAll, Word::PaneFolds),
            };
            right.push((words.say(state, &[]), look::HINT)).expect("seven quiet line parts");
            shortcut = Some(usize::try_from(right.len()).expect("quiet parts fit usize"));
            right.push((words.say(key, &[]), look::HINT)).expect("seven quiet line parts");
            if let Some(line) = &part.goal {
                goal = Some(usize::try_from(right.len()).expect("quiet parts fit usize"));
                right.push((copy_of(line), look::HINT)).expect("seven quiet line parts");
            }
        }
        if !needing.is_empty() {
            right.push((needing, look::SIGNAL)).expect("seven quiet line parts");
        }
    }
    let mut right_width = 0_u32;
    for (index, (line, _style)) in right.iter().enumerate() {
        if index > 0 {
            right_width = right_width.saturating_add(width(&separator));
        }
        right_width = right_width.saturating_add(width(line));
    }
    let drafter = words.say(Word::Drafter, &[]);
    let mut left_width = width(&part.state);
    if part.session {
        left_width = left_width.saturating_add(width(&drafter));
        if !part.state.is_empty() {
            left_width = left_width.saturating_add(width(&separator));
        }
    }
    if let Some(project) = &part.project {
        left_width = left_width.saturating_add(width(project)).saturating_add(width(&separator));
    }
    let closed_width = match &part.closed {
        Some(closed) => width(closed).saturating_add(width(&separator)),
        None => 0,
    };
    let walk = words.say(Word::ToWalk, &[]);
    let walk_width = width(&walk).saturating_add(width(&separator));
    let room = if part.columns == 0 { u32::MAX } else { u32::from(part.columns) };
    let mut total = left_width.saturating_add(closed_width).saturating_add(right_width).saturating_add(4);
    let mut omit_goal = false;
    let mut omit_shortcut = false;
    for candidate in [goal, shortcut] {
        if total > room
            && let Some(index) = candidate
        {
            let (line, _style) = right.as_slice().get(index).expect("quiet part is present");
            total = total.saturating_sub(width(line).saturating_add(width(&separator)));
            if candidate == goal {
                omit_goal = true;
            } else {
                omit_shortcut = true;
            }
        }
    }
    let narrow_notice = part.notice.is_some() && total > room;
    let walking =
        part.notice.is_none() && part.launchpad && part.needs.count > 0 && total.saturating_add(walk_width) <= room;
    let closed = part.closed.is_some() && (total <= room || narrow_notice);
    b.open(Role::ContentInfo);
    b.region(look::BOTTOM);
    b.open(Role::Status);
    if narrow_notice {
        text(b, Role::Text, part.notice.as_deref().expect("narrow notice is present"), Some(look::SIGNAL));
        text(b, Role::Text, &separator, Some(look::FAINT));
    }
    if let Some(project) = &part.project {
        text(b, Role::Text, project, Some(look::HINT));
        text(b, Role::Text, &separator, Some(look::FAINT));
    }
    if part.session {
        text(b, Role::Text, &drafter, Some(look::HINT));
        if closed || !part.state.is_empty() {
            text(b, Role::Text, &separator, Some(look::FAINT));
        }
    }
    if closed {
        text(b, Role::Text, part.closed.as_deref().expect("closed line is present"), Some(look::HINT));
        if !part.state.is_empty() {
            text(b, Role::Text, &separator, Some(look::FAINT));
        }
    }
    if !part.state.is_empty() {
        text(b, Role::Text, &part.state, Some(part.style));
    }
    b.close();
    b.open(Role::Paragraph);
    if !narrow_notice {
        let mut drawn = false;
        for (index, (line, style)) in right.iter().enumerate() {
            if (omit_goal && goal == Some(index)) || (omit_shortcut && shortcut == Some(index)) {
                continue;
            }
            if drawn {
                let style = if Some(index) == shortcut || Some(index) == goal { look::HINT } else { look::FAINT };
                text(b, Role::Text, &separator, Some(style));
            }
            text(b, Role::Text, line, Some(*style));
            drawn = true;
        }
    }
    if walking {
        text(b, Role::Text, &separator, Some(look::FAINT));
        text(b, Role::Text, &walk, Some(look::HINT));
    }
    b.close();
    b.close();
}

pub(crate) fn draw(b: &mut Builder, part: &Foot, words: &Words, binds: &mut Binds) -> Option<Event> {
    let Some(composer) = &part.composer else {
        b.open(Role::ContentInfo);
        b.region(look::BOTTOM);
        text(b, Role::Paragraph, &words.say(Word::QuitKeys, &[]), Some(look::HINT));
        text(b, Role::Status, &part.state, Some(part.style));
        b.close();
        return None;
    };
    if let Some(running) = &part.running {
        b.open(Role::Region);
        b.region(look::WORKING);
        b.open(Role::List);
        text(b, Role::Item, running, Some(look::RUNNING));
        b.close();
        b.close();
    }
    b.open(Role::Form);
    b.open(Role::Region);
    b.region(look::BOX);
    rule(b, part, words);
    b.open(Role::TextArea);
    b.style(look::FIELD);
    b.name(&composer.name);
    b.state(State::SubmitsOnEnter);
    b.value(&composer.value, composer.written);
    b.focus(composer.focus);
    b.bind(Binding { owner: look::OWNER, token: super::BOX });
    if part.list.is_none() {
        b.placeholder(&composer.placeholder);
    }
    b.close();
    if let Some(refusal) = &composer.refusal {
        text(b, Role::Paragraph, refusal, Some(look::REFUSED));
    }
    b.close();
    let escape = if let Some(list) = &part.list {
        lists::draw(b, list, binds);
        Some(list.escape.clone())
    } else {
        quiet(b, part, words);
        None
    };
    b.close();
    escape
}
