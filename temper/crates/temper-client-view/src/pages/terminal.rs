//! A chat's pages as a terminal (temper/docs/design/client/pages/README.md, A chat's terminal;
//! temper/docs/design/client/web/look.md, A chat's terminal).
pub(crate) mod commands;
mod question;
pub(crate) mod words;
use crate::shared::{TerminalWord as T, say};
use crate::{
    frame::Bindings,
    look,
    words::{Shell, Word, Words},
};
use alloc::boxed::Box;
use jig_client::contract as c;
use skein_ui::{Builder, Modifier, Role, Run, Shortcut, State, Style, phrases::Hole};
use temper_client_domain::{
    Address, Domain,
    pages::{
        Act,
        chat::document::Mode,
        terminal::{self, Font, Look, Theme},
    },
};

pub(crate) fn shown(root: &Domain, w: &Words) -> bool {
    w.shell == Shell::Browser && root.pages().look().terminal && terminal::draws(root.pages().address())
}
pub(crate) fn style(b: &mut Builder, look: Look) {
    b.style(look::TERMINAL);
    if look.theme == Theme::Light {
        b.style(look::TERMINAL_LIGHT);
    }
    if look.font == Font::Plex {
        b.style(look::TERMINAL_PLEX);
    }
}
pub(crate) struct Top<'a> {
    pub project: Option<u32>,
    pub heading: &'a [u8],
    pub name: &'a [u8],
    pub hint: Option<&'a [u8]>,
    pub number: Option<u64>,
    pub draft: Draft,
}
pub(crate) enum Draft {
    Shown,
    Hidden,
    Add,
    None,
}
pub(crate) fn top(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings, at: Top<'_>) {
    b.open(Role::Slot);
    b.style(look::TERMINAL_TOP);
    b.open(Role::Navigation);
    b.name(&w.say(Word::AskWhere, &[]));
    b.style(look::TERMINAL_WHERE);
    text(b, &say(w, T::Star), look::TERMINAL_STAR);
    if let Some(project) = at.project {
        let name = match super::project_name(root.pages().me(root.client()), project) {
            Some(name) => name,
            None => w.say(Word::Project, &[Hole::Number(u64::from(project))]),
        };
        text(b, &name, look::TERMINAL_PLACE);
        text(b, &say(w, T::Slash), look::TERMINAL_SEP);
    }
    b.open(Role::Heading { level: 1 });
    b.name(at.name);
    if let Some(hint) = at.hint {
        b.hint(hint);
    }
    b.text(Run::Plain, at.heading);
    b.close();
    if let Some(number) = at.number {
        text(b, &w.say(Word::Chat, &[Hole::Number(number)]), look::TERMINAL_TASK);
        if super::chat::asking(root) {
            text(b, &w.say(Word::ToolSeparator, &[]), look::TERMINAL_SEP);
            text(b, &w.say(Word::Asking, &[]), look::TERMINAL_TASK);
        }
    }
    b.close();
    b.open(Role::Slot);
    b.style(look::TERMINAL_ACTS);
    match at.draft {
        Draft::Shown | Draft::Hidden => {
            let pressed = match at.draft {
                Draft::Shown => true,
                Draft::Hidden | Draft::Add | Draft::None => false,
            };
            button(b, bindings, &say(w, T::Draft), Act::Mode { mode: if pressed { Mode::Chat } else { Mode::Split } });
            if pressed {
                b.state(State::Pressed);
            }
            b.shortcut(Shortcut { key: b'\\', modifier: Modifier::Primary });
            b.close();
        }
        Draft::Add => {
            let act = match root.pages().address() {
                Address::NewChat { .. } => Act::Go { address: Address::NewChat { draft: true } },
                Address::Chat { .. }
                | Address::Talk { .. }
                | Address::Ask { .. }
                | Address::AllHome
                | Address::Inbox { .. }
                | Address::Chats { .. }
                // page-view: draw Remembered.
                | Address::Told
                | Address::Remembered
                | Address::Members
                | Address::Limits
                | Address::Accounts
                | Address::Forges
                | Address::ConnectForge { .. }
                | Address::ConnectModel { .. }
                | Address::Welcome { .. }
                | Address::Task { .. }
                | Address::Work { .. }
                | Address::Goals { .. }
                | Address::Home { .. }
                | Address::Changes { .. }
                | Address::Team { .. }
                | Address::Settings { .. }
                | Address::Models { .. }
                | Address::Builders { .. }
                | Address::Role { .. }
                | Address::Rule { .. }
                | Address::Repositories { .. }
                | Address::Adopt { .. } => Act::AddDraft,
            };
            button(b, bindings, &w.say(Word::PlusDraft, &[]), act);
            b.close();
        }
        Draft::None => {}
    }
    button(b, bindings, &say(w, T::Exit), Act::Terminal(terminal::Act::Leave));
    b.close();
    b.close();
    b.close();
}
fn button(b: &mut Builder, bindings: &mut Bindings, name: &[u8], act: Act) {
    b.open(Role::Button);
    b.style(look::TERMINAL_BTN);
    b.name(name);
    b.bind(bindings.insert(act));
}
fn text(b: &mut Builder, words: &[u8], style: Style) {
    b.open(Role::Text);
    b.style(style);
    b.name(words);
    b.text(Run::Plain, words);
    b.close();
}
pub(crate) fn running(b: &mut Builder, words: Option<&[u8]>) {
    if let Some(words) = words {
        b.open(Role::Status);
        b.style(look::TERMINAL_RUN);
        b.name(words);
        b.text(Run::Plain, words);
        b.close();
    }
}
pub(crate) enum Hint {
    Commands,
    Paused,
}
pub(crate) fn under(
    b: &mut Builder,
    root: &Domain,
    task: Option<&c::Task>,
    w: &Words,
    bindings: &mut Bindings,
    hint: Hint,
) {
    if crate::confirm::terminal_list(b, root, w, bindings)
        || commands::under(b, root, task, w, bindings)
        || question::under(b, root, w, bindings)
    {
        return;
    }
    b.open(Role::Slot);
    b.style(look::TERMINAL_LINE);
    let hint = say(
        w,
        match hint {
            Hint::Commands => T::Commands,
            Hint::Paused => T::Paused,
        },
    );
    b.open(Role::Paragraph);
    b.style(look::TERMINAL_HINT);
    b.name(&hint);
    b.text(Run::Plain, &hint);
    b.close();
    b.open(Role::Slot);
    let line = crate::shared::closed_line(root, task, w);
    b.open(Role::Button);
    b.name(&line);
    b.bind(bindings.insert(Act::Terminal(terminal::Act::Run { command: terminal::Command::Model })));
    b.close();
    if let Some(task) = task {
        let spend = w.say(
            Word::ChatbarSpend,
            &[Hole::Number(task.budget.numbers.spent), Hole::Number(task.budget.numbers.budget)],
        );
        b.open(Role::Paragraph);
        b.name(&spend);
        b.text(Run::Plain, &spend);
        b.close();
    }
    b.close();
    b.close();
}
pub(crate) fn notes(b: &mut Builder, root: &Domain, _w: &Words) {
    commands::notes(b, root);
}
pub(crate) fn prompt(b: &mut Builder, w: &Words) {
    text(b, &say(w, T::Prompt), look::TERMINAL_STAR);
}
pub(crate) fn chat_running(b: &mut Builder, root: &Domain, w: &Words) {
    running(b, crate::shared::running(root, w).as_deref());
}
/// The running line (temper/docs/design/client/pages/README.md, A chat's terminal).
pub(crate) fn running_words(root: &Domain, w: &Words) -> Option<Box<[u8]>> {
    let chat = root.pages().chat()?;
    let task = chat.task(root.client())?;
    let me = root.pages().me(root.client())?;
    let delegates = chat.waiting_delegates(root.client());
    let account = match task.run_wait {
        Some(c::TaskRunWait::Account { .. }) => true,
        Some(c::TaskRunWait::Credential | c::TaskRunWait::Host | c::TaskRunWait::Refused(_)) | None => false,
    };
    if task.result.is_none()
        && !super::chats::ended(&task.chip)
        && !temper_client_domain::pages::chat::paused(&task.chip, me)
        && (super::chats::active(root, w)
            || (delegates > 0 && (task.waits_on == c::WaitsOn::Delegates || task.waits_on == c::WaitsOn::News))
            || account)
    {
        let (token, words) = super::chats::status_words(
            &task.chip,
            (&task.waits_on, Some(delegates), task.run_wait.as_ref()),
            (task.project, task.requester),
            Some(&task.wake),
            me,
            w,
        );
        let writing =
            jig_client::facts::chat_status(&task.chip, &task.waits_on, task.run_wait.as_ref(), task.project, me)
                == jig_client::facts::ChatStatus::Writing;
        Some(if token == look::STATUS_ACTIVE && writing { w.say(Word::RunningWork, &[]) } else { words })
    } else {
        None
    }
}

pub(crate) fn confirms(root: &Domain) -> bool {
    commands::confirms(root) || crate::confirm::terminal_confirms(root)
}
pub(crate) fn details(b: &mut Builder, w: &Words) {
    b.open(Role::Disclosure);
    b.style(look::TERMINAL_DETAILS);
    b.name(&say(w, T::Details));
}
pub(crate) fn kicker(b: &mut Builder, w: &Words) {
    text(b, &w.say(Word::Proposal, &[]), look::PROPOSAL_KICKER);
}
