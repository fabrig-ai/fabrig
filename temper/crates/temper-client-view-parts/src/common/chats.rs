//! Shared chats items (temper/docs/design/client/view-crates.md, 3.3).
use crate::words::{Word, Words};
use alloc::boxed::Box;
use jig_client::contract::TaskLine;
use skein_ui::{Builder, phrases::Hole};
use temper_client_domain::{Domain, pages::chats::ended};

pub(crate) fn outcome_text(b: &mut Builder, words: &Words, outcome: &jig_client::Outcome, text_limit: u32) {
    use crate::cards::P;
    use jig_client::{Answered, Outcome};
    match outcome {
        Outcome::Answered(Answered::Jig(jig_client::contract::Answer::Done(value))) => words.outcome(b, value),
        Outcome::Answered(Answered::Jig(jig_client::contract::Answer::Refused { refusal, .. })) => {
            words.refusal(b, refusal, text_limit);
        }
        Outcome::Full { held, room_at } => crate::cards::answers_full(b, words, *held, *room_at),
        Outcome::NotKept | Outcome::Held => words.status(b, P::RequestNotKept, &[]),
        Outcome::Unknown => words.status(b, P::Unknown, &[]),
        Outcome::Limit => words.status(b, P::RequestsInFlight, &[]),
        Outcome::Unconfirmed => words.status(b, P::RequestUnconfirmed, &[]),
        Outcome::Answered(Answered::Family(_)) => {}
        Outcome::Unsent | Outcome::Lost => unreachable!("temper's requests carry no value"),
    }
}

pub(crate) fn status_name(line: &TaskLine, me: &jig_client::contract::Me, words: &Words) -> Box<[u8]> {
    if line.ended_at.is_some() || ended(&line.chip) {
        return words.say(Word::HomeClosed, &[]);
    }
    if let jig_client::facts::ChatStatus::WaitingForYou { .. } =
        jig_client::facts::chat_status(&line.chip, &line.waits_on, None, line.project, me)
    {
        return words.say(Word::HomeWaiting, &[]);
    }
    status_words(&line.chip, (&line.waits_on, None, None), (line.project, line.requester), None, me, words).1
}

pub(crate) fn status_words(
    chip: &jig_client::contract::Chip,
    (waits_on, delegates, run_wait): (
        &jig_client::contract::WaitsOn,
        Option<u32>,
        Option<&jig_client::contract::TaskRunWait>,
    ),
    (project, requester): (u32, jig_client::contract::Requester),
    wake: Option<&jig_client::contract::WakePolicy>,
    me: &jig_client::contract::Me,
    words: &Words,
) -> (skein_ui::Style, Box<[u8]>) {
    use jig_client::contract::{ChipPhaseFacts, HoldCategory, Requester, WaitsOn, WakeRule};
    use jig_client::facts::ChatStatus;
    if ended(chip) {
        return (crate::look::STATUS_QUIET, words.say(Word::ChatClosed, &[]));
    }
    if temper_client_domain::pages::chat::paused(chip, me) {
        return (crate::look::STATUS_QUIET, words.say(Word::ChatPaused, &[]));
    }
    let live = delegates.unwrap_or({
        let ChipPhaseFacts { details, .. } = chip.details.as_ref();
        details.live_delegates
    });
    let waits_on = if live == 0 && *waits_on == WaitsOn::Delegates { &WaitsOn::News } else { waits_on };
    let wake = match wake {
        Some(wake) => Some(wake),
        None => Some(&chip.details.details.wake),
    };
    let narrowed = if (*waits_on == WaitsOn::Delegates || *waits_on == WaitsOn::News) && live > 0 {
        Some(words.say(Word::WaitingForDelegates, &[Hole::Number(u64::from(live))]))
    } else if *waits_on == WaitsOn::News
        && requester == Requester::Person(me.party.number)
        && let Some(wake) = wake
        && match wake.words {
            WakeRule::Never => false,
            WakeRule::Immediate | WakeRule::Batch { .. } | WakeRule::Paced { .. } => true,
        }
    {
        Some(words.say(Word::WaitingForYourWords, &[]))
    } else {
        None
    };
    if let Some(name) = narrowed {
        return (crate::look::STATUS_WAITING, name);
    }
    let status = jig_client::facts::chat_status(chip, waits_on, run_wait, project, me);
    let holes = match &status {
        ChatStatus::Tool { tool, .. } => Some(Hole::Name(tool.clone())),
        ChatStatus::RunWaiting(_)
        | ChatStatus::Starting
        | ChatStatus::Writing
        | ChatStatus::Working { .. }
        | ChatStatus::WaitingForYou { .. }
        | ChatStatus::WaitingFor(_)
        | ChatStatus::Parked
        | ChatStatus::Retrying { .. }
        | ChatStatus::HostRefused(_)
        | ChatStatus::Held(_)
        | ChatStatus::Closing
        | ChatStatus::Ended { .. } => None,
    };
    let token = match status {
        ChatStatus::WaitingForYou { .. } => crate::look::STATUS_YOURS,
        ChatStatus::HostRefused(_) | ChatStatus::Held(_) => crate::look::STATUS_HELD,
        ChatStatus::Retrying { .. } => crate::look::STATUS_RETRYING,
        ChatStatus::Starting | ChatStatus::Writing | ChatStatus::Tool { .. } | ChatStatus::Working { .. } => {
            crate::look::STATUS_ACTIVE
        }
        ChatStatus::RunWaiting(_) | ChatStatus::WaitingFor(_) => crate::look::STATUS_WAITING,
        ChatStatus::Parked | ChatStatus::Closing | ChatStatus::Ended { .. } => crate::look::STATUS_QUIET,
    };
    let name = match status {
        ChatStatus::Held(ref reason @ HoldCategory::Exceeded(_)) => crate::cards::hold_phrase(chip.task, reason, words),
        ChatStatus::WaitingForYou { .. } => words.say(Word::WaitingForYou, &[]),
        ChatStatus::Ended { .. } => words.say(Word::ChatClosed, &[]),
        ChatStatus::RunWaiting(_)
        | ChatStatus::Starting
        | ChatStatus::Writing
        | ChatStatus::Tool { .. }
        | ChatStatus::Working { .. }
        | ChatStatus::WaitingFor(_)
        | ChatStatus::Parked
        | ChatStatus::Retrying { .. }
        | ChatStatus::HostRefused(_)
        | ChatStatus::Held(_)
        | ChatStatus::Closing => words.say_word(jig_client::words::Word::status(&status), holes.as_slice()),
    };
    (token, name)
}

/// Active work follows the bar, including while words stream (client/pages/README.md).
pub(crate) fn active(root: &Domain, words: &Words) -> bool {
    let Some(chat) = root.pages().chat() else { return false };
    let Some(task) = chat.task(root.client()) else { return false };
    let Some(me) = root.pages().me(root.client()) else { return false };
    if task.result.is_some() {
        return false;
    }
    let (token, _) = status_words(
        &task.chip,
        (&task.waits_on, Some(chat.waiting_delegates(root.client())), task.run_wait.as_ref()),
        (task.project, task.requester),
        Some(&task.wake),
        me,
        words,
    );
    token == crate::look::STATUS_ACTIVE
}

/// The conversation draws dots until the reply's words begin (client/pages/README.md).
pub(crate) fn working(root: &Domain, words: &Words, thread: &jig_client::thread::Thread) -> bool {
    active(root, words) && words_not_begun(thread)
}

/// A nonempty provisional line settles the work before it (client/pages/README.md and client/pages/task.md, 2).
pub(crate) fn words_not_begun(thread: &jig_client::thread::Thread) -> bool {
    match thread.lines().into_boxed().last() {
        Some(jig_client::thread::Line::Provisional { text, .. }) => text.is_empty(),
        Some(
            jig_client::thread::Line::Activity { .. }
            | jig_client::thread::Line::Words { .. }
            | jig_client::thread::Line::Turn { .. }
            | jig_client::thread::Line::Undecoded { .. },
        )
        | None => true,
    }
}
