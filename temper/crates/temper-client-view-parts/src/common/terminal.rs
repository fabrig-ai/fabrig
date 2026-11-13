//! Shared terminal items (temper/docs/design/client/view-crates.md, 3.3).
pub(crate) mod commands;
use crate::{
    look,
    shared::{TerminalWord as T, say},
    words::{Shell, Word, Words},
};
use alloc::boxed::Box;
use jig_client::contract as c;
use skein_ui::{Builder, Role, Run, Style};
use temper_client_domain::{Domain, pages::terminal};

pub(crate) fn shown(root: &Domain, w: &Words) -> bool {
    w.shell == Shell::Browser && root.pages().look().terminal && terminal::draws(root.pages().address())
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
        && !temper_client_domain::pages::chats::ended(&task.chip)
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

pub(crate) fn details(b: &mut Builder, w: &Words) {
    b.open(Role::Disclosure);
    b.style(look::TERMINAL_DETAILS);
    b.name(&say(w, T::Details));
}

pub(crate) fn kicker(b: &mut Builder, w: &Words) {
    text(b, &w.say(Word::Proposal, &[]), look::PROPOSAL_KICKER);
}

pub(crate) fn text(b: &mut Builder, words: &[u8], style: Style) {
    b.open(Role::Text);
    b.style(style);
    b.name(words);
    b.text(Run::Plain, words);
    b.close();
}
