//! Main and child yield outcomes; only main changes the run lifecycle.
//! Contract: smith/docs/design/domain/run.md, sections 5.3, 6 and 9.1.

use core::mem;

use skein_lib::{Duration, Env, Id, Queue, ReplyTo, Slab, Token};

use super::{
    Conversation, Ending, Phase, Run, State, close, continue_messages, failed_ending, final_remaining, follow, nudge,
    say, wind_down,
};
use crate::call::Work;
use crate::charter::Mode;
use crate::{Domain, Limits, Request, Stop, agent, prompt};

pub(crate) fn yielded(
    domain: &mut Domain,
    env: &Env<Limits>,
    conversation: Token,
    stop: Stop,
    text: &[u8],
    out: &mut Queue<Request>,
) {
    let Domain { rng: _, runs, conversations, calls, alarms, facts, ready } = domain;
    let id = Id::<Conversation>::from_token(conversation);
    let conversation = conversations.get(id).expect("a conversation lives until it has ended");
    facts.about(conversation.run.token());
    let peer = match &conversation.phase {
        Phase::Running { peer } => *peer,
        // The yield crossed the run's close: there is nothing left to decide.
        Phase::Closing => return,
        Phase::Pending | Phase::Opening | Phase::Unwanted | Phase::Closed => {
            unreachable!("a conversation yields only between starting and ending")
        }
    };
    // A sub-agent that yields is done: its last message is its answer.
    if let Some(asker) = conversation.asker {
        let call = calls.get_mut(asker).expect("a sub-agent's call lives until it has ended");
        let close_child = match &mut call.work {
            Work::Child(child) => agent::yielded(child, text, stop, &env.limits),
            Work::Landing(_) | Work::Host(_) => unreachable!("a sub-agent serves a sub-agent's call"),
        };
        if let Some(child) = close_child {
            close(conversations, child, out);
        }
        return;
    }
    let run_id = conversation.run;
    let run = runs.get_mut(run_id).expect("a run lives until its conversations have ended");
    let state = mem::replace(&mut run.state, State::Closed);
    run.state = match state {
        State::Working { reply_to, main } => {
            assert!(main == id, "a conversation with no asker is main");
            assert!(conversation.calls == 0, "a main yield follows every call terminal");
            yielded_working(run, conversations, Yielding { reply_to, main, peer }, stop, text, env, out)
        }

        State::Awaiting { .. }
        | State::Preparing { .. }
        | State::Stopping { .. }
        | State::Listening { .. }
        | State::Waiting { .. }
        | State::Winding { .. }
        | State::Closed => {
            unreachable!("main yields only while its run works: it is closed when the run winds down")
        }
    };
    follow(runs, alarms, facts, ready, run_id);
}

struct Yielding {
    reply_to: ReplyTo,
    main: Id<Conversation>,
    peer: Token,
}

fn yielded_working(
    run: &mut Run,
    conversations: &mut Slab<Conversation>,
    yielding: Yielding,
    stop: Stop,
    text: &[u8],
    env: &Env<Limits>,
    out: &mut Queue<Request>,
) -> State {
    let replied = match run.charter.mode {
        Mode::Finite => false,
        Mode::Open => match stop {
            Stop::EndTurn => !text.is_empty(),
            Stop::Budget | Stop::MaxTokens | Stop::Refusal | Stop::NoCalls => false,
        },
    };
    let final_reply = match &run.wind_down {
        Some(crate::wind_down::WindDown::Finishing { noted: true, .. }) => true,
        Some(
            crate::wind_down::WindDown::Finishing { noted: false, .. } | crate::wind_down::WindDown::Settling { .. },
        )
        | None => false,
    };
    if replied && final_reply {
        yielded_parked(run, conversations, yielding, out)
    } else if run.wind_down.is_some() {
        yielded_finishing(run, conversations, yielding, env, out)
    } else if run.inbox.has_queued() {
        continue_messages(run, yielding.reply_to, yielding.main, yielding.peer, &env.limits, out)
    } else if replied {
        yielded_listening(run, conversations, yielding, env, out)
    } else {
        yielded_nudged(run, conversations, yielding, stop, &env.limits, out)
    }
}

fn yielded_parked(
    run: &Run,
    conversations: &mut Slab<Conversation>,
    yielding: Yielding,
    out: &mut Queue<Request>,
) -> State {
    out.push(Request::Waiting { host_run: run.host_name, read: run.inbox.fence() });
    wind_down(conversations, yielding.reply_to, yielding.main, Ending::Parked, out)
}

fn yielded_finishing(
    run: &mut Run,
    conversations: &mut Slab<Conversation>,
    yielding: Yielding,
    env: &Env<Limits>,
    out: &mut Queue<Request>,
) -> State {
    let Yielding { reply_to, main, peer } = yielding;
    match final_remaining(run, env.now) {
        Ok(()) if run.inbox.has_queued() => continue_messages(run, reply_to, main, peer, &env.limits, out),
        Ok(()) => {
            let note = match run.charter.mode {
                Mode::Finite => prompt::finish_now(),
                Mode::Open => prompt::last_reply(),
            };
            say(reply_to, main, peer, note, out)
        }
        Err(exhausted) => wind_down(conversations, reply_to, main, Ending::Over(exhausted), out),
    }
}

fn yielded_listening(
    run: &Run,
    conversations: &mut Slab<Conversation>,
    yielding: Yielding,
    env: &Env<Limits>,
    out: &mut Queue<Request>,
) -> State {
    if run.charter.waiting == Duration::ZERO {
        yielded_parked(run, conversations, yielding, out)
    } else {
        out.push(Request::Waiting { host_run: run.host_name, read: run.inbox.fence() });
        State::Listening {
            reply_to: yielding.reply_to,
            main: yielding.main,
            until: env.now.saturating_add(run.charter.waiting),
        }
    }
}

fn yielded_nudged(
    run: &mut Run,
    conversations: &mut Slab<Conversation>,
    yielding: Yielding,
    stop: Stop,
    limits: &Limits,
    out: &mut Queue<Request>,
) -> State {
    let Yielding { reply_to, main, peer } = yielding;
    match nudge(run, stop, limits) {
        Ok(()) => say(reply_to, main, peer, prompt::nudge(run.charter.mode, stop, run.nudges, limits.nudges), out),
        Err(failure) => wind_down(conversations, reply_to, main, failed_ending(failure), out),
    }
}
