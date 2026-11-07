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
/// Print the supplied main and ordered sub-agent choices, with their optional efforts.
pub(super) fn text_choice(choice: Option<&c::LlmChoice>, out: &mut Text) {
    let Some(choice) = choice else { return };
    out.put(b"\nmain model: ");
    text_model(&choice.main, out);
    for entry in &choice.subagents {
        out.put(b"\nsub-agent model: ");
        text_model(entry, out);
    }
}
fn text_model(value: &c::ModelEffort, out: &mut Text) {
    out.words(value.model.bytes());
    if let Some(effort) = &value.effort {
        out.put(b"/");
        out.words(effort.bytes());
    }
}
/// Whether no account serves the next run (client/tools.md, 6.1).
pub(super) fn no_account(task: &c::Task) -> bool {
    let Some(c::TaskRunWait::Account { account: None, source: None, why: c::TaskWaitWhy::Empty }) = task.run_wait
    else {
        return false;
    };
    true
}
/// Whether the task waits for an account and nothing else (client/tools.md, 6.1).
pub(super) fn waits_for_account(task: &c::Task) -> bool {
    task.waits_on == c::WaitsOn::Nothing && task.hold.is_none() && no_account(task)
}
pub(super) fn text_wait(task: &c::Task, limits: &crate::jig::Limits, out: &mut Text) {
    let Some(value) = task.run_wait else { return };
    out.put(b"\nnext run:");
    if !no_account(task) {
        out.put(b" ");
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
    wait_explanation(task, value, limits, out);
}

fn wait_explanation(task: &c::Task, wait: c::TaskRunWait, limits: &crate::jig::Limits, out: &mut Text) {
    use c::{TaskRunWait as W, TaskSourceState as S, TaskWaitWhy as R};
    match wait {
        W::Credential => out.put(b"\n  it needs a credential from its forge: forge list"),
        W::Host => out.put(b"\n  no host has room, or none is connected; it starts when one does"),
        W::Refused(why) => {
            out.put(b"\n  ");
            refused_explanation(why, out);
        }
        W::Account { account, why, .. } => match why {
            R::Refreshing | R::Source(S::Asking | S::Granted) => {}
            R::Empty => empty_account_explanation(task, limits, out),
            R::Host => {
                out.put(b"\n  its login isn't signed in on a host: ");
                out.put(limits.program_login);
                out.put(b" NAME, then ");
                account_command(b"verify", account, out);
                out.put(b" --source program");
            }
            R::Invalid(why) => {
                out.put(b"\n  its account cannot serve this work: ");
                out.put(account_why(why));
            }
            R::Source(state) => source_explanation(state, account, out),
        },
    }
}
fn empty_account_explanation(task: &c::Task, limits: &crate::jig::Limits, out: &mut Text) {
    out.put(b"\n  no account serving this work yet");
    if let Some(choice) = &task.llm {
        out.put(b" (shown model: ");
        out.words(choice.main.model.bytes());
        out.put(b")");
    }
    out.put(b"\n  without an explicit model, a usable account may select another offered model");
    out.put(b"\n  provider sign-in, where offered: account connect PROVIDER --key KEY");
    out.put(b"\n  API key, where offered: account add-key PROVIDER FILE --key KEY");
    out.put(b"\n  Claude Code: account declare PROVIDER claude_code NAME --key KEY, then ");
    out.put(limits.program_login);
    out.put(b" NAME");
    out.put(b"\n  skip program-login if NAME's directory is already signed in; then account list and account verify ACCOUNT --source program");
}
fn account_command(verb: &[u8], account: Option<u32>, out: &mut Text) {
    out.put(b"account ");
    out.put(verb);
    out.put(b" ");
    match account {
        Some(number) => out.number(u64::from(number)),
        None => out.put(b"ACCOUNT"),
    }
}
fn source_explanation(state: c::TaskSourceState, account: Option<u32>, out: &mut Text) {
    use c::TaskSourceState as S;
    match state {
        S::Asking | S::Granted => {}
        S::Limited { .. } => {
            out.put(b"\n  its account is limited: ");
            account_command(b"retry", account, out);
        }
        S::Refused => {
            out.put(b"\n  its account's source was refused: ");
            account_command(b"replace-key", account, out);
            out.put(b" FILE");
        }
        S::NeedsSignIn => out.put(b"\n  its account needs sign-in: account connect PROVIDER --key KEY"),
        S::Moving => out.put(b"\n  its account is moving out"),
        S::Later { .. } => out.put(b"\n  its account will be retried later"),
        S::Misconfigured => out.put(b"\n  its account's source is misconfigured"),
        S::Unloaded { .. } => out.put(b"\n  its account's source could not be loaded"),
        S::Cancelled => out.put(b"\n  its account's source was cancelled"),
        S::Closing => out.put(b"\n  its account's source is closing"),
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
                c::AccountPlace::Directory => b"directory",
                c::AccountPlace::File => b"file",
            });
            out.put(b": ");
            out.put(match why {
                c::AccountUnread::Unreadable => b"unreadable",
                c::AccountUnread::Refused => b"refused",
                c::AccountUnread::Failed => b"failed",
                c::AccountUnread::Stalled => b"stalled",
                c::AccountUnread::TooLarge => b"too large",
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

fn host_kind(value: c::HostKind) -> &'static [u8] {
    match value {
        c::HostKind::Engine => b"engine",
        c::HostKind::Worker => b"worker",
    }
}
fn refused(value: c::TaskWaitRefused, out: &mut Encoder) {
    out.object_start();
    out.key(b"refused");
    out.object_start();
    match value {
        c::TaskWaitRefused::Bounds { kind, why, figures: amounts } => {
            out.key(b"bounds");
            out.object_start();
            out.key(b"kind");
            out.string(host_kind(kind));
            out.key(b"why");
            match why {
                Some(why) => agent_shortfall(why, out),
                None => out.null(),
            }
            figures(amounts, out);
            out.object_end();
        }
        c::TaskWaitRefused::Hello { kind, at, why, figures: amounts } => {
            out.key(b"hello");
            out.object_start();
            out.key(b"kind");
            out.string(host_kind(kind));
            out.key(b"at");
            out.unsigned(at.as_nanos());
            out.key(b"why");
            hello_refusal(why, out);
            figures(amounts, out);
            out.object_end();
        }
    }
    out.object_end();
    out.object_end();
}
fn figures(value: Option<c::Figures>, out: &mut Encoder) {
    out.key(b"figures");
    match value {
        Some(value) => {
            out.object_start();
            out.key(b"required");
            out.unsigned(value.required);
            out.key(b"offered");
            out.unsigned(value.offered);
            out.object_end();
        }
        None => out.null(),
    }
}
fn hello_refusal(value: c::HelloRefusal, out: &mut Encoder) {
    match value {
        c::HelloRefusal::JobKinds => out.string(b"job_kinds"),
        c::HelloRefusal::ReservedChannel => out.string(b"reserved_channel"),
        c::HelloRefusal::Agent { why } => {
            out.object_start();
            out.key(b"agent");
            out.object_start();
            out.key(b"why");
            agent_shortfall(why, out);
            out.object_end();
            out.object_end();
        }
        c::HelloRefusal::Position { worker } => refusal_number(b"position", b"worker", u64::from(worker), out),
        c::HelloRefusal::Workers { limit } => refusal_number(b"workers", b"limit", u64::from(limit), out),
        c::HelloRefusal::StopBound { grace } => refusal_number(b"stop_bound", b"grace", grace.as_nanos(), out),
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
fn agent_shortfall(value: c::AgentShortfall, out: &mut Encoder) {
    match value {
        c::AgentShortfall::Model { charter, model } => agent_numbers(b"model", charter, model, out),
        c::AgentShortfall::Window { charter, model } => agent_numbers(b"window", charter, model, out),
        c::AgentShortfall::Output { charter, model } => agent_numbers(b"output", charter, model, out),
        c::AgentShortfall::Turns { charter } => refusal_number(b"turns", b"charter", u64::from(charter), out),
        c::AgentShortfall::Spend => out.string(b"spend"),
        c::AgentShortfall::Time { charter } => refusal_number(b"time", b"charter", u64::from(charter), out),
        c::AgentShortfall::Waiting { charter } => refusal_number(b"waiting", b"charter", u64::from(charter), out),
        c::AgentShortfall::Checks { charter } => {
            out.object_start();
            out.key(b"checks");
            out.object_start();
            out.key(b"charter");
            out.unsigned(u64::from(charter));
            out.key(b"member");
            out.string(b"reserve_time");
            out.object_end();
            out.object_end();
        }
        c::AgentShortfall::Message => out.string(b"message"),
        c::AgentShortfall::Inbox => out.string(b"inbox"),
        c::AgentShortfall::Unacknowledged => out.string(b"unacknowledged"),
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
        c::TaskWaitRefused::Bounds { kind, why, .. } => {
            out.put(match kind {
                c::HostKind::Worker => b"no worker takes it",
                c::HostKind::Engine => b"the engine's own host can't take it",
            });
            if let Some(why) = why {
                out.put(b": ");
                agent_shortfall_text(why, out);
            }
        }
        c::TaskWaitRefused::Hello { kind, at, why, .. } => {
            out.put(host_kind(kind));
            out.put(b" refused at ");
            super::utc(at, out);
            out.put(b": ");
            hello_refusal_text(why, out);
        }
    }
}
fn hello_refusal_text(value: c::HelloRefusal, out: &mut Text) {
    match value {
        c::HelloRefusal::Agent { why } => agent_shortfall_text(why, out),
        c::HelloRefusal::JobKinds => out.put(b"its job kinds are not ascending and distinct"),
        c::HelloRefusal::ReservedChannel => out.put(b"channel zero belongs to the engine's own host"),
        c::HelloRefusal::Position { worker } => {
            out.put(b"unconfigured or occupied worker position ");
            out.number(u64::from(worker));
        }
        c::HelloRefusal::Workers { limit } => {
            out.put(b"no room for another worker, limit ");
            out.number(u64::from(limit));
        }
        c::HelloRefusal::StopBound { grace } => {
            out.put(b"would not stop before the engine's grace ends, allowed grace ");
            out.number(grace.as_nanos());
            out.put(b" ns");
        }
    }
}
fn agent_shortfall_text(value: c::AgentShortfall, out: &mut Text) {
    match value {
        c::AgentShortfall::Model { charter, model } => {
            out.put(b"agent model missing for charter ");
            out.number(u64::from(charter));
            out.put(b", model position ");
            out.number(u64::from(model));
        }
        c::AgentShortfall::Window { charter, model } => {
            out.put(b"agent window too small for charter ");
            out.number(u64::from(charter));
            out.put(b", model position ");
            out.number(u64::from(model));
        }
        c::AgentShortfall::Output { charter, model } => {
            out.put(b"agent output too small for charter ");
            out.number(u64::from(charter));
            out.put(b", model position ");
            out.number(u64::from(model));
        }
        c::AgentShortfall::Turns { charter } => {
            out.put(b"agent turns too few for charter ");
            out.number(u64::from(charter));
        }
        c::AgentShortfall::Spend => out.put(b"agent spend too small"),
        c::AgentShortfall::Time { charter } => {
            out.put(b"agent time too short for charter ");
            out.number(u64::from(charter));
        }
        c::AgentShortfall::Waiting { charter } => {
            out.put(b"agent waiting too short for charter ");
            out.number(u64::from(charter));
        }
        c::AgentShortfall::Checks { charter } => {
            out.put(b"agent check timeout exceeds charter ");
            out.number(u64::from(charter));
            out.put(b" reserve_time; see temper help settings for held revisions");
        }
        c::AgentShortfall::Message => out.put(b"agent message bound too small"),
        c::AgentShortfall::Inbox => out.put(b"agent inbox too small"),
        c::AgentShortfall::Unacknowledged => out.put(b"agent unacknowledged turns too few"),
    }
}

/// The client's bound clauses in ctl's units (temper/docs/design/client/tools.md, 6.1).
fn refused_explanation(value: c::TaskWaitRefused, out: &mut Text) {
    match value {
        c::TaskWaitRefused::Bounds { kind, why, figures } => bound_explanation(kind, why, figures, None, out),
        c::TaskWaitRefused::Hello { kind, at, why, figures } => match why {
            c::HelloRefusal::Agent { why } if figures.is_some() => {
                bound_explanation(kind, Some(why), figures, Some(at), out);
            }
            c::HelloRefusal::Agent { .. }
            | c::HelloRefusal::JobKinds
            | c::HelloRefusal::ReservedChannel
            | c::HelloRefusal::Position { .. }
            | c::HelloRefusal::Workers { .. }
            | c::HelloRefusal::StopBound { .. } => {
                out.put(b"its host refused this start: ");
                refused_text(value, out);
            }
        },
    }
}
fn bound_explanation(
    kind: c::HostKind,
    why: Option<c::AgentShortfall>,
    figures: Option<c::Figures>,
    at: Option<skein_lib::Wall>,
    out: &mut Text,
) {
    use c::AgentShortfall as S;
    out.put(match kind {
        c::HostKind::Worker => b"no worker can take this",
        c::HostKind::Engine => b"the engine's own host can't take this",
    });
    let Some(why) = why else {
        out.put(b": each falls short of another bound its charter asks");
        return;
    };
    let Some(figures) = figures else {
        out.put(b". ");
        agent_shortfall_text(why, out);
        return;
    };
    out.put(b": ");
    bound_clause(why, figures.required, at, out);
    out.put(b", ");
    match kind {
        c::HostKind::Engine => out.put(b"and it takes "),
        c::HostKind::Worker => match at {
            Some(at) => {
                out.put(b"and the worker refused at ");
                super::utc(at, out);
                out.put(b" takes ");
            }
            None => out.put(match why {
                S::Checks { .. } => b"and the quickest worker's checks take ",
                S::Model { .. }
                | S::Window { .. }
                | S::Output { .. }
                | S::Turns { .. }
                | S::Spend
                | S::Time { .. }
                | S::Waiting { .. }
                | S::Message
                | S::Inbox
                | S::Unacknowledged => b"and the largest worker takes ",
            }),
        },
    }
    bound_amount(why, figures.offered, out);
}
fn bound_clause(why: c::AgentShortfall, required: u64, at: Option<skein_lib::Wall>, out: &mut Text) {
    use c::AgentShortfall as S;
    match why {
        S::Model { .. } => agent_shortfall_text(why, out),
        S::Window { charter, model } | S::Output { charter, model } => {
            bound_charter(charter, at, out);
            out.put(b"'s model at position ");
            out.number(u64::from(model));
            out.put(match why {
                S::Window { .. } => b" asks a window of ",
                S::Output { .. } => b" asks ",
                S::Model { .. }
                | S::Turns { .. }
                | S::Spend
                | S::Time { .. }
                | S::Waiting { .. }
                | S::Checks { .. }
                | S::Message
                | S::Inbox
                | S::Unacknowledged => unreachable!("window or output"),
            });
            out.number(required);
            out.put(match why {
                S::Output { .. } => b" output tokens",
                S::Model { .. }
                | S::Window { .. }
                | S::Turns { .. }
                | S::Spend
                | S::Time { .. }
                | S::Waiting { .. }
                | S::Checks { .. }
                | S::Message
                | S::Inbox
                | S::Unacknowledged => b" tokens",
            });
        }
        S::Turns { charter } | S::Time { charter } | S::Waiting { charter } | S::Checks { charter } => {
            bound_charter(charter, at, out);
            out.put(match why {
                S::Checks { .. } => b" keeps ",
                S::Model { .. }
                | S::Window { .. }
                | S::Output { .. }
                | S::Turns { .. }
                | S::Spend
                | S::Time { .. }
                | S::Waiting { .. }
                | S::Message
                | S::Inbox
                | S::Unacknowledged => b" asks ",
            });
            bound_amount(why, required, out);
            out.put(match why {
                S::Turns { .. } => b" turns",
                S::Time { .. } => b" a run",
                S::Waiting { .. } => b" of waiting",
                S::Checks { .. } => b" for checks",
                S::Model { .. }
                | S::Window { .. }
                | S::Output { .. }
                | S::Spend
                | S::Message
                | S::Inbox
                | S::Unacknowledged => unreachable!("charter bound"),
            });
        }
        S::Spend => {
            out.put(b"a run here may spend ");
            out.number(required);
        }
        S::Message => {
            out.put(b"a message here may be ");
            out.number(required);
            out.put(b" bytes");
        }
        S::Inbox => {
            out.put(b"a start here sends ");
            out.number(required);
            out.put(b" messages");
        }
        S::Unacknowledged => {
            out.put(b"a run here keeps ");
            out.number(required);
            out.put(b" turns unacknowledged");
        }
    }
}
fn bound_charter(charter: u32, at: Option<skein_lib::Wall>, out: &mut Text) {
    match at {
        None => out.put(b"its charter"),
        Some(_) => {
            out.put(b"charter ");
            out.number(u64::from(charter));
        }
    }
}
fn bound_amount(why: c::AgentShortfall, value: u64, out: &mut Text) {
    out.number(value);
    match why {
        c::AgentShortfall::Time { .. } | c::AgentShortfall::Waiting { .. } | c::AgentShortfall::Checks { .. } => {
            out.put(b" ms");
        }
        c::AgentShortfall::Model { .. }
        | c::AgentShortfall::Window { .. }
        | c::AgentShortfall::Output { .. }
        | c::AgentShortfall::Turns { .. }
        | c::AgentShortfall::Spend
        | c::AgentShortfall::Message
        | c::AgentShortfall::Inbox
        | c::AgentShortfall::Unacknowledged => {}
    }
}
