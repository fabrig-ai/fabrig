//! Spawn and process-tree proofs (domain/host.md, sections 4 and 6).
//! A pending spawn owns Start; a spawned tree owns its process token. Wait
//! supplies exit status and Reap supplies the final tail. `domain::follow`
//! derives demands and retirement after every entrance.
use super::{
    Agent, Down, End, Env, Exit, Limits, Output, Queue, Sent, Start, Token, draining, parent, process, report_unread,
    tail,
};
use alloc::boxed::Box;
use core::mem;

#[derive(Debug)]
#[expect(clippy::large_enum_variant, reason = "the bounded Start moves inline until its spawn terminal")]
pub(super) enum State {
    Spawning { start: Start },
    Running { process: Token },
    Exited { process: Token, exit: Option<Exit> },
    Reaped { process: Token, exit: Option<Exit>, detail: Box<[u8]> },
    Closed,
}

impl State {
    pub(super) fn process(&self) -> Token {
        match self {
            Self::Running { process } | Self::Exited { process, .. } | Self::Reaped { process, .. } => *process,
            Self::Spawning { .. } | Self::Closed => unreachable!("a spawned tree owns the process operation"),
        }
    }

    pub(super) fn exited(&self) -> bool {
        match self {
            Self::Exited { .. } | Self::Reaped { .. } => true,
            Self::Spawning { .. } | Self::Running { .. } | Self::Closed => false,
        }
    }

    pub(super) fn reaped(&self) -> bool {
        match self {
            Self::Reaped { .. } => true,
            Self::Spawning { .. } | Self::Running { .. } | Self::Exited { .. } | Self::Closed => false,
        }
    }
}

pub(super) fn spawned(agent: &mut Agent, owner: Token, process: Token, env: &Env<Limits>, out: &mut Queue<Output>) {
    let state = mem::replace(&mut agent.tree, State::Closed);
    agent.tree = match state {
        State::Spawning { start } => {
            agent.supervision.start(env.now, env.limits.wall_time);
            out.push(Output::Parent(parent::Request::Started { client: agent.client, agent: owner }));
            for message in &start.messages {
                agent.unread.sent(message.name);
            }
            for grant in &start.grants {
                agent.accounts.get_mut(&grant.account).expect("validated start grant").emitted = grant.generation;
            }
            out.push(Output::Process(process::Request::Send {
                owner,
                process,
                message: Down::Start {
                    start,
                    window: crate::Window { turns: env.limits.turns, bytes: env.limits.unacknowledged_bytes },
                },
            }));
            agent.sending = Some(Sent::Start);
            out.push(Output::Process(process::Request::Wait { owner, process }));
            out.push(Output::Process(process::Request::Reap { owner, process }));
            State::Running { process }
        }
        State::Running { .. } | State::Exited { .. } | State::Reaped { .. } | State::Closed => {
            unreachable!("one terminal per process spawn")
        }
    };
}

pub(super) fn unspawned(agent: &mut Agent, detail: &[u8], env: &Env<Limits>, out: &mut Queue<Output>) {
    let state = mem::replace(&mut agent.tree, State::Closed);
    agent.tree = match state {
        State::Spawning { .. } => {
            report_unread(agent, out);
            agent.supervision = super::State::Closed;
            out.push(Output::Parent(parent::Request::Gone {
                client: agent.client,
                end: End::Unspawned,
                detail: tail(detail, env.limits.detail_bytes),
            }));
            State::Closed
        }
        State::Running { .. } | State::Exited { .. } | State::Reaped { .. } | State::Closed => {
            unreachable!("one terminal per process spawn")
        }
    };
}

pub(super) fn exited(agent: &mut Agent, exit: Option<Exit>, env: &Env<Limits>) {
    let state = mem::replace(&mut agent.tree, State::Closed);
    agent.tree = match state {
        State::Running { process } => State::Exited { process, exit },
        State::Spawning { .. } | State::Exited { .. } | State::Reaped { .. } | State::Closed => {
            unreachable!("one terminal per Wait")
        }
    };
    if !agent.word.last_word() {
        draining(agent, env);
    }
}

pub(super) fn reaped(agent: &mut Agent, detail: &[u8], env: &Env<Limits>) {
    let state = mem::replace(&mut agent.tree, State::Closed);
    agent.tree = match state {
        State::Exited { process, exit } => {
            State::Reaped { process, exit, detail: tail(detail, env.limits.detail_bytes) }
        }
        State::Spawning { .. } | State::Running { .. } | State::Reaped { .. } | State::Closed => {
            unreachable!("Reap follows Exited once")
        }
    };
}

pub(super) fn finish(agent: &mut Agent, out: &mut Queue<Output>) {
    let state = mem::replace(&mut agent.tree, State::Closed);
    agent.tree = match state {
        State::Reaped { exit, detail, .. } => {
            out.push(Output::Parent(parent::Request::Gone {
                client: agent.client,
                end: End::Exited { exit, forced: agent.forced },
                detail,
            }));
            State::Closed
        }
        State::Spawning { .. } | State::Running { .. } | State::Exited { .. } | State::Closed => {
            unreachable!("empty tree proof precedes Gone")
        }
    };
}
