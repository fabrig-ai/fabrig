//! Incoming record laws (domain/host.md, section 6).
//! Size checks precede sequence and terminal checks; no state changes here.
use super::{Agent, Ask, Kind, SendState, Sent, within};
use crate::{Limits, RunResult, Up};
use skein_lib::Env;

pub(super) fn too_large(message: &Up, limits: &Limits) -> bool {
    match message {
        Up::Call { ask, .. } => match ask {
            Ask::Host { tool, body, .. } => {
                !within(tool, u64::from(limits.name_bytes)) || !within(body, limits.call_bytes)
            }
            Ask::Deliver { fields } => fields.owned_bytes() > limits.call_bytes,
        },
        Up::Turn { turn } => turn.body.owned_bytes() > limits.turn_bytes,
        Up::Fact { body } => !within(body, limits.fact_bytes),
        Up::Answer { answer } => match &answer.result {
            RunResult::Accepted { outcome } => outcome.owned_bytes() > limits.outcome_bytes,
            RunResult::Refused { .. } | RunResult::Parked | RunResult::Failed { .. } => false,
        },
        Up::Admitted
        | Up::MessageRefused { .. }
        | Up::Withdraw { .. }
        | Up::Long { .. }
        | Up::LongDone
        | Up::Waiting { .. }
        | Up::Unauthorized { .. }
        | Up::Refused { .. }
        | Up::Limited { .. } => false,
    }
}

pub(super) fn valid_record(agent: &Agent, message: &Up, env: &Env<Limits>) -> bool {
    let limits = &env.limits;
    match message {
        Up::Admitted => !agent.word.admitted() && !agent.word.last_word(),
        Up::MessageRefused { name, .. } => agent.word.admitted() && agent.unread.refusal_known(*name),
        Up::Answer { answer } => valid_answer(agent, answer),
        Up::Call { call, name, deadline, ask } => {
            if !agent.word.admitted()
                || name.activation != agent.activation
                || name.completion == 0
                || *deadline < env.now
            {
                return false;
            }
            let kind = match ask {
                Ask::Host { tool, .. } => {
                    if tool.is_empty() {
                        return false;
                    }
                    Kind::Host
                }
                Ask::Deliver { .. } => Kind::Delivery,
            };
            if agent.calls.contains_key(call) {
                return false;
            }
            for (_, old) in &agent.calls {
                if old.name == *name || (old.kind == Kind::Delivery && kind == Kind::Delivery) {
                    return false;
                }
            }
            for busy in &agent.busy {
                if busy.call == *call || busy.name == *name {
                    return false;
                }
            }
            match agent.sending {
                Some(Sent::Busy { call: old, name: old_name }) => {
                    if old == *call || old_name == *name {
                        return false;
                    }
                }
                Some(Sent::Start | Sent::Message | Sent::Call(_) | Sent::Ack(_) | Sent::Grant | Sent::Cancel)
                | None => {}
            }
            true
        }
        Up::Withdraw { call } => {
            if !agent.word.admitted() {
                return false;
            }
            match agent.calls.get(call) {
                Some(entry) => !entry.withdrawn || entry.stage != SendState::Parent,
                None => true,
            }
        }
        Up::Turn { turn } => {
            agent.word.admitted()
                && agent.number.checked_add(1) == Some(turn.number)
                && turn.spent >= agent.spent
                && agent.unread.known(turn.read)
                && agent.turns.len() < limits.turns
                && match agent.turn_bytes.checked_add(turn.body.owned_bytes()) {
                    Some(bytes) => bytes <= limits.unacknowledged_bytes,
                    None => false,
                }
        }
        Up::Fact { .. } => agent.word.admitted(),
        Up::Long { operation: _, span } => {
            agent.word.admitted() && *span <= limits.long_span && agent.long.open < limits.long_open
        }
        Up::LongDone => agent.word.admitted() && agent.long.open > 0,
        Up::Waiting { read } => agent.word.admitted() && agent.unread.known(*read),
        Up::Unauthorized { account, generation } => match agent.accounts.get(account) {
            Some(account) => agent.word.admitted() && *generation > 0 && *generation <= account.emitted,
            None => false,
        },
        Up::Refused { account, .. } | Up::Limited { account, .. } => {
            agent.word.admitted() && agent.accounts.contains_key(account)
        }
    }
}

fn valid_answer(agent: &Agent, answer: &crate::Answer) -> bool {
    for (_, call) in &agent.calls {
        if call.kind == Kind::Delivery {
            match call.stage {
                SendState::Parent | SendState::Queued => return false,
                SendState::Sending => {}
            }
        }
    }
    if answer.read != agent.unread.read() || answer.turns != agent.number || answer.spent < agent.spent {
        return false;
    }
    match &answer.result {
        RunResult::Refused { .. } => !agent.word.admitted() && answer.turns == 0 && answer.spent == 0,
        RunResult::Accepted { .. } | RunResult::Parked | RunResult::Failed { .. } => agent.word.admitted(),
    }
}
