//! Playground start (temper/docs/design/draft/playground-reset.md, 6.1;
//! temper/docs/plan/playground-reset-r4/notes/contracts.md, 2).
use crate::{Connection, Domain, Key, Limits, Playground, PlaygroundState, Request, StartAnswer, StartFailure, Stored};
use skein_lib::{Env, Map, Queue, Token};

#[derive(Debug)]
pub(crate) struct State {
    pub(crate) pending: Map<Token, Token>,
    pub(crate) answers: Map<Token, StartAnswer>,
}
impl State {
    pub(crate) fn new(limits: &Limits) -> Self {
        Self { pending: Map::with_capacity(limits.applications), answers: Map::with_capacity(limits.applications) }
    }
}

/// Whether a kept forge other than the playground's stands (draft 6.3).
pub(crate) fn real_forge_kept(d: &Domain) -> bool {
    for row in d.forge_records() {
        if !row.playground && !row.removing {
            return true;
        }
    }
    false
}

pub(crate) fn start(d: &mut Domain, env: &Env<Limits>, request: Token, token: Token, out: &mut Queue<Request>) {
    if d.playground_start.pending.len() == d.playground_start.pending.capacity() {
        out.push(Request::ApplicationAgain { request, why: jig_domain_people::Again::Busy });
        return;
    }
    let first = d.playground_start.pending.is_empty();
    d.playground_start.pending.insert(request, token).expect("bounded starts");
    if !first {
        return;
    }
    d.playground_start.answers = Map::with_capacity(d.playground_start.answers.capacity());
    let resetting = match d.playground() {
        Some(row) => row.state == PlaygroundState::Resetting,
        None => false,
    };
    if d.demo.is_none() || real_forge_kept(d) || resetting {
        answered(d, StartAnswer::NotOffered, out);
        return;
    }
    if d.playground.is_none() {
        d.playground = Some(Playground { state: PlaygroundState::Running, forge: None, project: None, account: None });
        save(d, out);
    }
    if d.playground.as_ref().expect("running playground").forge.is_none() {
        let mut kept = None;
        for row in d.forge_records() {
            if row.playground && !row.removing {
                kept = Some(row.number);
                break;
            }
        }
        let number = if let Some(number) = kept {
            Ok(number)
        } else if let Some(forge) = d.demo.as_ref().expect("offered playground").forge.clone() {
            crate::make::make(d, env, forge, out)
        } else {
            Err(crate::make::Refusal::Invalid)
        };
        match number {
            Ok(number) => {
                d.playground.as_mut().expect("running playground").forge = Some(number);
                save(d, out);
            }
            Err(why) => {
                let why: &[u8] = match why {
                    crate::make::Refusal::Full => b"full",
                    crate::make::Refusal::Busy => b"busy",
                    crate::make::Refusal::Invalid => b"invalid",
                    crate::make::Refusal::Exhausted => b"exhausted",
                };
                d.playground = None;
                out.push(Request::Erase { key: Key::Playground });
                answered(d, StartAnswer::Failed(StartFailure::Group { why: why.into() }), out);
                return;
            }
        }
    }
    let forge = d.playground.as_ref().expect("running playground").forge.expect("made forge");
    if d.under_box {
        match d.connection(forge) {
            Connection::Connected { .. } | Connection::Connecting => {}
            Connection::NotConnected
            | Connection::NotSetUp
            | Connection::SettingUp { .. }
            | Connection::SetUpFailed { .. }
            | Connection::Stopped { .. } => {
                out.push(Request::SetUpGroup { forge, request });
            }
        }
    }
    out.push(Request::PlaygroundStart { request });
}

fn save(d: &Domain, out: &mut Queue<Request>) {
    out.push(Request::Save {
        read_afresh: None,
        release: None,
        record: Stored::Playground(*d.playground.as_ref().expect("running playground")),
    });
}

pub(crate) fn made(d: &mut Domain, project: Option<u32>, account: Option<u32>, out: &mut Queue<Request>) {
    let row = d.playground.as_mut().expect("active playground start");
    if let Some(project) = project {
        row.project = Some(project);
    }
    if let Some(account) = account {
        row.account = Some(account);
    }
    save(d, out);
}

pub(crate) fn answered(d: &mut Domain, answer: StartAnswer, out: &mut Queue<Request>) {
    for _ in 0..d.playground_start.pending.len() {
        let Some((request, token)) = d.playground_start.pending.first() else { break };
        let (request, token) = (*request, *token);
        d.playground_start.pending.remove(&request);
        d.playground_start.answers.insert(token, answer.clone()).expect("one answer per bounded start");
        out.push(Request::ApplicationDone { request, token });
    }
}
