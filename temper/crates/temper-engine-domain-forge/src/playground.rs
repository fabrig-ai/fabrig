//! Playground start and reset (temper/docs/design/draft/playground-reset.md, 6.1, 6.2;
//! temper/docs/plan/playground-reset-r4/notes/contracts.md, 2).
use crate::{
    Connection, DemoConfig, Domain, Key, Limits, Playground, PlaygroundState, Request, StartAnswer, StartFailure,
    Stored,
};
use skein_lib::{Env, Map, Queue, Token};

#[derive(Debug)]
pub(crate) struct State {
    pub(crate) ends: Map<Token, Token>,
    pub(crate) pending: Map<Token, Token>,
    pub(crate) answers: Map<Token, StartAnswer>,
}
impl State {
    pub(crate) fn new(limits: &Limits) -> Self {
        Self {
            ends: Map::with_capacity(limits.applications),
            pending: Map::with_capacity(limits.applications),
            answers: Map::with_capacity(limits.applications),
        }
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
        record: Stored::Playground(*d.playground.as_ref().expect("playground row")),
    });
}

pub(crate) fn made(d: &mut Domain, project: Option<u32>, account: Option<u32>, out: &mut Queue<Request>) {
    let Some(row) = d.playground.as_mut() else { return };
    if row.state == PlaygroundState::Resetting {
        return;
    }
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

pub(crate) fn end(d: &mut Domain, env: &Env<Limits>, request: Token, token: Token, out: &mut Queue<Request>) {
    if d.demo.is_none() || d.playground.is_none() {
        let answer = if d.demo.is_none() { crate::EndAnswer::NotOffered } else { crate::EndAnswer::Reset };
        crate::asks::refuse(d, request, token, crate::Answered::EndPlayground(answer), out);
        return;
    }
    if d.playground_start.ends.len() == d.playground_start.ends.capacity() {
        out.push(Request::ApplicationAgain { request, why: jig_domain_people::Again::Busy });
        return;
    }
    d.playground_start.ends.insert(request, token).expect("bounded ends");
    if d.playground.as_ref().expect("playground").state == PlaygroundState::Running {
        reset(d, env, out);
    }
}

fn reset(d: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    d.playground.as_mut().expect("playground").state = PlaygroundState::Resetting;
    save(d, out);
    answered(d, StartAnswer::NotOffered, out);
    next_step(d, env, out);
}

/// A kept act of the reset was erased: the next step (draft 6.2).
pub(crate) fn kept_erased(d: &mut Domain, env: &Env<Limits>, act: crate::KeptAct, out: &mut Queue<Request>) {
    let Some(row) = d.playground.as_mut() else { return };
    if row.state != PlaygroundState::Resetting {
        return;
    }
    match act {
        crate::KeptAct::RemoveProject { project } if row.project == Some(project) => row.project = None,
        crate::KeptAct::RemoveForge { forge } if row.forge == Some(forge) => row.forge = None,
        crate::KeptAct::RemoveAccount { account } if row.account == Some(account) => row.account = None,
        crate::KeptAct::Cancel { .. }
        | crate::KeptAct::BoxRemove { .. }
        | crate::KeptAct::RemoveProject { .. }
        | crate::KeptAct::RemoveForge { .. }
        | crate::KeptAct::RemoveAccount { .. } => return,
    }
    save(d, out);
    next_step(d, env, out);
}

/// At each start, once the core is ready: a running playground is reset (D5).
pub(crate) fn restarted(d: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    let Some(row) = d.playground else { return };
    match row.state {
        PlaygroundState::Running => reset(d, env, out),
        PlaygroundState::Resetting => {
            for number in 1..=3 {
                if d.kept.contains_key(&crate::kept::key(b"playground reset", &[number])) {
                    return;
                }
            }
            next_step(d, env, out);
        }
    }
}

fn next_step(d: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    let row = d.playground.expect("resetting playground");
    let (number, act) = if let Some(project) = row.project {
        (1, crate::KeptAct::RemoveProject { project })
    } else if let Some(forge) = row.forge {
        (2, crate::KeptAct::RemoveForge { forge })
    } else if let Some(account) = row.account {
        (3, crate::KeptAct::RemoveAccount { account })
    } else {
        d.playground = None;
        out.push(Request::Erase { key: Key::Playground });
        for _ in 0..d.playground_start.ends.len() {
            let Some((&request, &token)) = d.playground_start.ends.first() else { break };
            d.playground_start.ends.remove(&request);
            d.connect.answer(request, token, crate::Answered::EndPlayground(crate::EndAnswer::Reset), out);
        }
        return;
    };
    let key = crate::kept::key(b"playground reset", &[number]);
    crate::kept::keep(d, &[crate::Kept { key, act }], out);
    if let crate::KeptAct::RemoveForge { forge } = act {
        match crate::removal::remove(d, env, forge, out) {
            Ok(()) => {}
            Err(crate::RemoveRefusal::NoSuchForge) => crate::kept::erase(d, env, key, out),
            Err(crate::RemoveRefusal::NotOffered | crate::RemoveRefusal::Role) => {
                unreachable!("internal removal has no admission refusal")
            }
        }
    }
}

/// Playground configuration bounds (temper/docs/design/draft/playground-reset.md, 6.1).
impl DemoConfig {
    #[must_use]
    pub fn within(&self) -> bool {
        if let Some(forge) = &self.forge
            && !forge.fits()
        {
            return false;
        }
        let adoption = &self.adoption;
        for bytes in [
            adoption.host.len(),
            adoption.owner.len(),
            adoption.name.len(),
            adoption.prefix.len(),
            adoption.landing.len(),
        ] {
            if bytes > 1024 {
                return false;
            }
        }
        adoption.checks.is_empty()
    }
}
/// Playground configuration and forge availability (temper/docs/design/draft/playground-reset.md, 6.1).
impl Domain {
    pub fn configure_demo(&mut self, demo: Option<DemoConfig>) {
        if let Some(demo) = &demo {
            assert!(demo.within(), "bounded demo configuration");
        }
        self.demo = demo;
    }
    #[must_use]
    pub fn demo(&self) -> Option<&DemoConfig> {
        self.demo.as_ref()
    }
    /// Removing and removed forges stay ended (temper/docs/design/draft/playground-reset.md, 4, 6.1).
    #[must_use]
    pub fn forge_ended(&self, forge: u16) -> bool {
        if crate::removal::removing(self, forge) {
            return true;
        }
        if let Some(next) = self.next_forge
            && forge < next
            && !self.configured.contains_key(&forge)
        {
            return true;
        }
        false
    }
}
