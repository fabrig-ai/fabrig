//! Work read for the quit question (temper/docs/design/client/README.md, 6.17).
use crate::Tui;
use alloc::boxed::Box;
use jig_client::{
    contract as c,
    facts::{ChatStatus, chat_status},
};
use skein_lib::{Set, bytes::copy_of};
use temper_client_domain::{Domain, pages::chats};

pub(crate) struct Running {
    pub count: u32,
    pub title: Option<Box<[u8]>>,
}

/// Whether stopping the box would stop work under way, as the root reads it.
#[must_use]
pub fn work_runs(root: &Domain, _tui: &Tui) -> bool {
    running(root).count > 0
}

pub(crate) fn running(root: &Domain) -> Running {
    let mut found = Running { count: 0, title: None };
    let Some(me) = root.pages().me(root.client()) else { return found };
    let Some(project) = root.pages().project() else { return found };
    let open = match root.pages().chat() {
        Some(chat) => chat.task(root.client()),
        None => None,
    };
    let home = match root.pages().chats() {
        Some(chats) => chats.live(root.client()),
        None => None,
    };
    let rows = match home {
        Some(tasks) => tasks.rows.len(),
        None => 0,
    };
    let capacity = rows.checked_add(1).expect("read rows and one open chat fit");
    let mut seen = Set::with_capacity(capacity);
    if let Some(task) = open {
        seen.insert(task.chip.task).expect("room for the open chat");
        if task.project == project
            && task.result.is_none()
            && works(&task.chip, &task.waits_on, task.run_wait.as_ref(), project, me)
        {
            found.count = 1;
            found.title = Some(copy_of(task.chip.title.bytes()));
        }
    }
    let open_works = found.count > 0;
    let mut newest = None;
    if let Some(tasks) = home {
        for line in &tasks.rows {
            if line.project != project || !seen.insert(line.chip.task).expect("one entry per read row") {
                continue;
            }
            if works(&line.chip, &line.waits_on, None, project, me) {
                found.count = found.count.checked_add(1).expect("count fits the read rows and open chat");
                let later = match newest {
                    Some(held) => line.last_activity > held,
                    None => true,
                };
                if !open_works && later {
                    newest = Some(line.last_activity);
                    found.title = Some(copy_of(line.chip.title.bytes()));
                }
            }
        }
    }
    found
}

fn works(chip: &c::Chip, waits_on: &c::WaitsOn, run_wait: Option<&c::TaskRunWait>, project: u32, me: &c::Me) -> bool {
    if chats::ended(chip) {
        return false;
    }
    let working = match chat_status(chip, waits_on, run_wait, project, me) {
        ChatStatus::Starting
        | ChatStatus::Writing
        | ChatStatus::Tool { .. }
        | ChatStatus::Working { .. }
        | ChatStatus::Retrying { .. } => true,
        ChatStatus::RunWaiting(_)
        | ChatStatus::HostRefused(_)
        | ChatStatus::WaitingForYou { .. }
        | ChatStatus::WaitingFor(_)
        | ChatStatus::Parked
        | ChatStatus::Held(_)
        | ChatStatus::Closing
        | ChatStatus::Ended { .. } => false,
    };
    working || chip.details.details.live_delegates > 0
}
