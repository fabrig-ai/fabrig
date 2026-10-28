//! Supplied preferences and next-run waits (jig's client/domain/contract.md, 5.1).
use super::{Cut, Text, accounts};
use jig_client::contract as c;
use skein_json::writer::Encoder;

pub(super) const fn account_why(value: c::AccountWhy) -> &'static [u8] {
    match value {
        c::AccountWhy::Unknown => b"unknown",
        c::AccountWhy::Provider => b"provider",
        c::AccountWhy::Uses => b"uses",
        c::AccountWhy::Limited => b"limited",
    }
}
fn number(value: Option<u32>, out: &mut Encoder) {
    match value {
        Some(value) => out.unsigned(u64::from(value)),
        None => out.null(),
    }
}
pub(super) fn encode_task(value: &c::Task, out: &mut Encoder, cut: &mut Cut) {
    out.key(b"llm");
    match &value.llm {
        None => out.null(),
        Some(choice) => {
            out.object_start();
            out.key(b"account");
            number(choice.account, out);
            out.key(b"main");
            model(&choice.main, out, cut);
            out.key(b"subagents");
            out.array_start();
            for entry in &choice.subagents {
                model(entry, out, cut);
            }
            out.array_end();
            out.object_end();
        }
    }
    out.key(b"kept_account");
    number(value.kept_account, out);
    out.key(b"run_wait");
    match value.run_wait {
        None => out.null(),
        Some(c::TaskRunWait::Credential) => out.string(b"credential"),
        Some(c::TaskRunWait::Refused(value)) => refused(value, out),
        Some(c::TaskRunWait::Host) => out.string(b"host"),
        Some(c::TaskRunWait::Account { account, source, why }) => {
            out.object_start();
            out.key(b"account");
            out.object_start();
            out.key(b"account");
            number(account, out);
            out.key(b"source");
            number(source, out);
            out.key(b"why");
            wait(why, out);
            out.object_end();
            out.object_end();
        }
    }
}
fn model(value: &c::ModelEffort, out: &mut Encoder, cut: &mut Cut) {
    out.object_start();
    out.key(b"model");
    cut.words(value.model.bytes(), out);
    out.key(b"effort");
    match &value.effort {
        Some(effort) => cut.words(effort.bytes(), out),
        None => out.null(),
    }
    out.object_end();
}
fn wait(value: c::TaskWaitWhy, out: &mut Encoder) {
    match value {
        c::TaskWaitWhy::Refreshing => out.string(b"refreshing"),
        c::TaskWaitWhy::Host => out.string(b"host"),
        c::TaskWaitWhy::Empty => out.string(b"empty"),
        c::TaskWaitWhy::Invalid(why) => {
            out.object_start();
            out.key(b"invalid");
            out.string(account_why(why));
            out.object_end();
        }
        c::TaskWaitWhy::Source(state) => source(state, out),
    }
}
fn source(value: c::TaskSourceState, out: &mut Encoder) {
    match value {
        c::TaskSourceState::Moving => out.string(b"moving"),
        c::TaskSourceState::Asking => out.string(b"asking"),
        c::TaskSourceState::Granted => out.string(b"granted"),
        c::TaskSourceState::Refused => out.string(b"refused"),
        c::TaskSourceState::NeedsSignIn => out.string(b"needs_sign_in"),
        c::TaskSourceState::Misconfigured => out.string(b"misconfigured"),
        c::TaskSourceState::Cancelled => out.string(b"cancelled"),
        c::TaskSourceState::Closing => out.string(b"closing"),
        c::TaskSourceState::Unloaded { at, why } => {
            out.object_start();
            out.key(b"unloaded");
            out.object_start();
            accounts::unloaded(at, why, out);
            out.object_end();
            out.object_end();
        }
        c::TaskSourceState::Later { until } => deadline(b"later", until, out),
        c::TaskSourceState::Limited { until } => deadline(b"limited", until, out),
    }
}
fn deadline(label: &[u8], until: Option<skein_lib::Wall>, out: &mut Encoder) {
    out.object_start();
    out.key(label);
    out.object_start();
    out.key(b"until");
    match until {
        Some(wall) => out.unsigned(wall.as_nanos()),
        None => out.null(),
    }
    out.object_end();
    out.object_end();
}
pub(super) fn text_wait(value: Option<c::TaskRunWait>, out: &mut Text) {
    let Some(value) = value else { return };
    out.put(b"\nnext run: ");
    match value {
        c::TaskRunWait::Credential => out.put(b"credential"),
        c::TaskRunWait::Refused(value) => refused_text(value, out),
        c::TaskRunWait::Host => out.put(b"host"),
        c::TaskRunWait::Account { account, source, why } => {
            out.put(b"account ");
            optional_number(account, out);
            out.put(b" source ");
            optional_number(source, out);
            out.put(b": ");
            match why {
                c::TaskWaitWhy::Refreshing => out.put(b"refreshing"),
                c::TaskWaitWhy::Host => out.put(b"host"),
                c::TaskWaitWhy::Empty => out.put(b"empty"),
                c::TaskWaitWhy::Invalid(why) => out.put(account_why(why)),
                c::TaskWaitWhy::Source(value) => source_text(value, out),
            }
        }
    }
}
fn optional_number(value: Option<u32>, out: &mut Text) {
    match value {
        Some(value) => out.number(u64::from(value)),
        None => out.put(b"unknown"),
    }
}
fn source_text(value: c::TaskSourceState, out: &mut Text) {
    match value {
        c::TaskSourceState::Moving => out.put(b"moving"),
        c::TaskSourceState::Later { until } => text_deadline(b"later until ", until, out),
        c::TaskSourceState::Limited { until } => text_deadline(b"limited until ", until, out),
        c::TaskSourceState::Asking => out.put(b"asking"),
        c::TaskSourceState::Granted => out.put(b"granted"),
        c::TaskSourceState::Refused => out.put(b"refused"),
        c::TaskSourceState::NeedsSignIn => out.put(b"needs sign-in"),
        c::TaskSourceState::Misconfigured => out.put(b"misconfigured"),
        c::TaskSourceState::Unloaded { at, why } => {
            out.put(b"unloaded ");
            out.put(match at {
                c::OwnerAccountPlace::Directory => b"directory",
                c::OwnerAccountPlace::File => b"file",
            });
            out.put(b": ");
            out.put(match why {
                c::OwnerAccountUnread::Unreadable => b"unreadable",
                c::OwnerAccountUnread::Refused => b"refused",
                c::OwnerAccountUnread::Failed => b"failed",
                c::OwnerAccountUnread::Stalled => b"stalled",
                c::OwnerAccountUnread::TooLarge => b"too large",
            });
        }
        c::TaskSourceState::Cancelled => out.put(b"cancelled"),
        c::TaskSourceState::Closing => out.put(b"closing"),
    }
}

fn text_deadline(label: &[u8], until: Option<skein_lib::Wall>, out: &mut Text) {
    out.put(label);
    match until {
        Some(wall) => super::utc(wall, out),
        None => out.put(b"unknown"),
    }
}

pub(super) fn refusal(account: u32, why: c::AccountWhy, out: &mut Encoder) {
    out.object_start();
    out.key(b"account");
    out.object_start();
    out.key(b"number");
    out.unsigned(u64::from(account));
    out.key(b"why");
    out.string(account_why(why));
    out.object_end();
    out.object_end();
}

fn host_kind(value: c::OwnerHostKind) -> &'static [u8] {
    match value {
        c::OwnerHostKind::Engine => b"engine",
        c::OwnerHostKind::Worker => b"worker",
    }
}
fn refused(value: c::TaskWaitRefused, out: &mut Encoder) {
    out.object_start();
    out.key(b"refused");
    out.object_start();
    match value {
        c::TaskWaitRefused::Program(value) => {
            out.key(b"program");
            open_program(value, out);
        }
        c::TaskWaitRefused::Hello { kind, at, why } => {
            out.key(b"hello");
            out.object_start();
            out.key(b"kind");
            out.string(host_kind(kind));
            out.key(b"at");
            out.unsigned(at.as_nanos());
            out.key(b"why");
            hello_refusal(why, out);
            out.object_end();
        }
    }
    out.object_end();
    out.object_end();
}
fn open_program(value: c::OpenProgram, out: &mut Encoder) {
    match value {
        c::OpenProgram::Model => out.string(b"model"),
        c::OpenProgram::Source { account, source } => {
            out.object_start();
            out.key(b"source");
            out.object_start();
            out.key(b"account");
            out.unsigned(u64::from(account));
            out.key(b"source");
            out.unsigned(u64::from(source));
            out.object_end();
            out.object_end();
        }
    }
}
fn hello_refusal(value: c::OwnerHelloRefusal, out: &mut Encoder) {
    match value {
        c::OwnerHelloRefusal::ReservedChannel => out.string(b"reserved_channel"),
        c::OwnerHelloRefusal::Agent { why } => {
            out.object_start();
            out.key(b"agent");
            out.object_start();
            out.key(b"why");
            agent_shortfall(why, out);
            out.object_end();
            out.object_end();
        }
        c::OwnerHelloRefusal::Position { worker } => refusal_number(b"position", b"worker", u64::from(worker), out),
        c::OwnerHelloRefusal::Workers { limit } => refusal_number(b"workers", b"limit", u64::from(limit), out),
        c::OwnerHelloRefusal::StopBound { grace } => refusal_number(b"stop_bound", b"grace", grace.as_nanos(), out),
    }
}
fn refusal_number(variant: &[u8], field: &[u8], value: u64, out: &mut Encoder) {
    out.object_start();
    out.key(variant);
    out.object_start();
    out.key(field);
    out.unsigned(value);
    out.object_end();
    out.object_end();
}
fn agent_shortfall(value: c::OwnerAgentShortfall, out: &mut Encoder) {
    match value {
        c::OwnerAgentShortfall::Model { charter, model } => agent_numbers(b"model", charter, model, out),
        c::OwnerAgentShortfall::Window { charter, model } => agent_numbers(b"window", charter, model, out),
        c::OwnerAgentShortfall::Output { charter, model } => agent_numbers(b"output", charter, model, out),
        c::OwnerAgentShortfall::Turns => out.string(b"turns"),
        c::OwnerAgentShortfall::Spend => out.string(b"spend"),
        c::OwnerAgentShortfall::Time => out.string(b"time"),
        c::OwnerAgentShortfall::Waiting => out.string(b"waiting"),
        c::OwnerAgentShortfall::Checks => out.string(b"checks"),
        c::OwnerAgentShortfall::Message => out.string(b"message"),
        c::OwnerAgentShortfall::Inbox => out.string(b"inbox"),
        c::OwnerAgentShortfall::Unacknowledged => out.string(b"unacknowledged"),
    }
}
fn agent_numbers(variant: &[u8], charter: u32, model: u32, out: &mut Encoder) {
    out.object_start();
    out.key(variant);
    out.object_start();
    out.key(b"charter");
    out.unsigned(u64::from(charter));
    out.key(b"model");
    out.unsigned(u64::from(model));
    out.object_end();
    out.object_end();
}
fn refused_text(value: c::TaskWaitRefused, out: &mut Text) {
    out.put(b"refused, ");
    match value {
        c::TaskWaitRefused::Program(value) => open_program_text(value, out),
        c::TaskWaitRefused::Hello { kind, at, why } => {
            out.put(host_kind(kind));
            out.put(b" refused at ");
            super::utc(at, out);
            out.put(b": ");
            hello_refusal_text(why, out);
        }
    }
}
fn open_program_text(value: c::OpenProgram, out: &mut Text) {
    match value {
        c::OpenProgram::Model => out.put(b"model runs through a program"),
        c::OpenProgram::Source { account, source } => {
            out.put(b"source ");
            out.number(u64::from(source));
            out.put(b" of account ");
            out.number(u64::from(account));
            out.put(b" runs through a program");
        }
    }
}
fn hello_refusal_text(value: c::OwnerHelloRefusal, out: &mut Text) {
    match value {
        c::OwnerHelloRefusal::Agent { why } => agent_shortfall_text(why, out),
        c::OwnerHelloRefusal::ReservedChannel => out.put(b"channel zero belongs to the engine's own host"),
        c::OwnerHelloRefusal::Position { worker } => {
            out.put(b"unconfigured or occupied worker position ");
            out.number(u64::from(worker));
        }
        c::OwnerHelloRefusal::Workers { limit } => {
            out.put(b"no room for another worker, limit ");
            out.number(u64::from(limit));
        }
        c::OwnerHelloRefusal::StopBound { grace } => {
            out.put(b"would not stop before the engine's grace ends, allowed grace ");
            out.number(grace.as_nanos());
            out.put(b" ns");
        }
    }
}
fn agent_shortfall_text(value: c::OwnerAgentShortfall, out: &mut Text) {
    match value {
        c::OwnerAgentShortfall::Model { charter, model } => {
            out.put(b"agent model missing for charter ");
            out.number(u64::from(charter));
            out.put(b", model position ");
            out.number(u64::from(model));
        }
        c::OwnerAgentShortfall::Window { charter, model } => {
            out.put(b"agent window too small for charter ");
            out.number(u64::from(charter));
            out.put(b", model position ");
            out.number(u64::from(model));
        }
        c::OwnerAgentShortfall::Output { charter, model } => {
            out.put(b"agent output too small for charter ");
            out.number(u64::from(charter));
            out.put(b", model position ");
            out.number(u64::from(model));
        }
        c::OwnerAgentShortfall::Turns => out.put(b"agent turns too few"),
        c::OwnerAgentShortfall::Spend => out.put(b"agent spend too small"),
        c::OwnerAgentShortfall::Time => out.put(b"agent time too short"),
        c::OwnerAgentShortfall::Waiting => out.put(b"agent waiting too short"),
        c::OwnerAgentShortfall::Checks => out.put(b"agent checks exceed the change reserve"),
        c::OwnerAgentShortfall::Message => out.put(b"agent message bound too small"),
        c::OwnerAgentShortfall::Inbox => out.put(b"agent inbox too small"),
        c::OwnerAgentShortfall::Unacknowledged => out.put(b"agent unacknowledged turns too few"),
    }
}
