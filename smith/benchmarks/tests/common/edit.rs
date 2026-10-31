//! Typed edits fail when a fixture record changes kind.

use smith_events::{Event, Record};

pub fn budget(record: &mut Record) -> &mut smith_events::BudgetStanding {
    let Event::Budget(value) = &mut record.event else { panic!("expected Budget fixture") };
    value
}

pub fn conversation_opened(record: &mut Record) -> &mut smith_events::ConversationOpened {
    let Event::ConversationOpened(value) = &mut record.event else { panic!("expected ConversationOpened fixture") };
    value
}

pub fn response_completed(record: &mut Record) -> &mut smith_events::ResponseCompleted {
    let Event::ResponseCompleted(value) = &mut record.event else { panic!("expected ResponseCompleted fixture") };
    value
}

pub fn response_started(record: &mut Record) -> &mut smith_events::ResponseStarted {
    let Event::ResponseStarted(value) = &mut record.event else { panic!("expected ResponseStarted fixture") };
    value
}

pub fn run_completed(record: &mut Record) -> &mut smith_events::RunCompleted {
    let Event::RunCompleted(value) = &mut record.event else { panic!("expected RunCompleted fixture") };
    value
}

pub fn run_started(record: &mut Record) -> &mut smith_events::RunStarted {
    let Event::RunStarted(value) = &mut record.event else { panic!("expected RunStarted fixture") };
    value
}

pub fn session_ended(record: &mut Record) -> &mut smith_events::SessionEnded {
    let Event::SessionEnded(value) = &mut record.event else { panic!("expected SessionEnded fixture") };
    value
}

pub fn session_started(record: &mut Record) -> &mut smith_events::SessionStarted {
    let Event::SessionStarted(value) = &mut record.event else { panic!("expected SessionStarted fixture") };
    value
}

pub fn tool_completed(record: &mut Record) -> &mut smith_events::ToolCompleted {
    let Event::ToolCompleted(value) = &mut record.event else { panic!("expected ToolCompleted fixture") };
    value
}

pub fn tool_started(record: &mut Record) -> &mut smith_events::ToolStarted {
    let Event::ToolStarted(value) = &mut record.event else { panic!("expected ToolStarted fixture") };
    value
}

pub fn conversation_closed(record: &mut Record) -> &mut smith_events::ConversationClosed {
    let Event::ConversationClosed(value) = &mut record.event else { panic!("expected ConversationClosed fixture") };
    value
}

pub fn message_received(record: &mut Record) -> &mut smith_events::MessageReceived {
    let Event::MessageReceived(value) = &mut record.event else { panic!("expected MessageReceived fixture") };
    value
}

pub fn notice(record: &mut Record) -> &mut smith_events::Notice {
    let Event::Notice(value) = &mut record.event else { panic!("expected Notice fixture") };
    value
}
