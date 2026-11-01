//! Playground creation and ordered reset (temper/docs/design/draft/playground-reset.md, 6.1–6.3, 9).
use crate::{
    DemoConfig, Domain, EndAnswer, ForgeState, Key, Limits, Mode, Playground, PlaygroundState, Request, ResetAct,
    ResetStep, StartAnswer, Stored,
};
use skein_lib::{Env, Map, Queue, Token};

pub(crate) fn start(d: &mut Domain, env: &Env<Limits>, request: Token, token: Token, out: &mut Queue<Request>) {
    if d.playground_pending.len() == d.playground_pending.capacity() {
        out.push(Request::ApplicationAgain { request, why: jig_domain_people::Again::Busy });
        return;
    }
    let first = d.playground_pending.is_empty();
    d.playground_pending.insert(request, token).expect("bounded playground starts");
    if !first {
        return;
    }
    d.playground_answers = Map::with_capacity(d.playground_answers.capacity());
    if !offered(d) {
        answered(d, StartAnswer::NotOffered, out);
        return;
    }
    let row = match d.playground {
        Some(row) => row,
        None => {
            let mut record = d.machine.playground.clone().expect("offered playground template");
            record.mode = Mode::Fake;
            record.playground = true;
            let Some(forge) = crate::forges::make_record(d, &env.limits, record, out) else {
                answered(d, StartAnswer::NotOffered, out);
                return;
            };
            let row = Playground { state: PlaygroundState::Running, forge, project: None, account: None };
            save(d, row, out);
            row
        }
    };
    if needs_group(d, row.forge) {
        let record = d.records.get(&row.forge).expect("running playground's forge");
        if let Some(entry) = &record.setup {
            out.push(Request::SetUpGroup { forge: row.forge, request, entry: entry.clone() });
        }
    }
    out.push(Request::PlaygroundStart { request });
}

fn offered(d: &Domain) -> bool {
    if d.machine.playground.is_none() {
        return false;
    }
    if let Some(Playground { state: PlaygroundState::Resetting(_), .. }) = d.playground {
        return false;
    }
    for (number, record) in &d.records {
        if record.state == ForgeState::Kept
            && Some(*number)
                != match d.playground {
                    Some(row) => Some(row.forge),
                    None => None,
                }
        {
            return false;
        }
    }
    true
}

fn needs_group(d: &Domain, forge: u16) -> bool {
    d.under_box
        && match d.connection(forge) {
            crate::Connection::Connected { .. } | crate::Connection::Connecting => false,
            crate::Connection::NotConnected
            | crate::Connection::NotSetUp
            | crate::Connection::SettingUp { .. }
            | crate::Connection::SetUpFailed { .. }
            | crate::Connection::Stopped { .. } => true,
        }
}

pub(crate) fn answered(d: &mut Domain, answer: StartAnswer, out: &mut Queue<Request>) {
    for _ in 0..d.playground_pending.len() {
        let Some((request, token)) = d.playground_pending.first() else { break };
        let (request, token) = (*request, *token);
        d.playground_pending.remove(&request);
        d.playground_answers.insert(token, answer.clone()).expect("one answer per bounded start");
        out.push(Request::ApplicationDone { request, token });
    }
}

pub(crate) fn project(d: &mut Domain, project: u32, out: &mut Queue<Request>) {
    if let Some(mut row) = d.playground {
        row.project = Some(project);
        save(d, row, out);
    }
}

pub(crate) fn account(d: &mut Domain, account: u32, out: &mut Queue<Request>) {
    if let Some(mut row) = d.playground {
        row.account = Some(account);
        save(d, row, out);
    }
}

pub(crate) fn end(d: &mut Domain, env: &Env<Limits>, request: Token, token: Token, out: &mut Queue<Request>) {
    if d.playground_ends.len() == d.playground_ends.capacity() {
        out.push(Request::ApplicationAgain { request, why: jig_domain_people::Again::Busy });
        return;
    }
    let first = d.playground_ends.is_empty();
    d.playground_ends.insert(request, token).expect("bounded playground ends");
    if first {
        d.playground_end_answers = Map::with_capacity(d.playground_end_answers.capacity());
    }
    match d.playground {
        None => ended(d, EndAnswer::NotOffered, out),
        Some(Playground { state: PlaygroundState::Running, .. }) => step(d, env, ResetStep::Project, out),
        Some(Playground { state: PlaygroundState::Resetting(_), .. }) => {}
    }
}

pub(crate) fn step(d: &mut Domain, env: &Env<Limits>, step: ResetStep, out: &mut Queue<Request>) {
    let mut next = step;
    for _ in 0_u8..3 {
        let Some(mut row) = d.playground else { return };
        row.state = PlaygroundState::Resetting(next);
        save(d, row, out);
        match next {
            ResetStep::Project => match row.project {
                Some(project) => {
                    out.push(Request::ResetAct(ResetAct::RemoveProject { project }));
                    return;
                }
                None => next = ResetStep::Forge,
            },
            ResetStep::Forge => {
                if d.records.contains_key(&row.forge) {
                    crate::removal::remove(d, env, row.forge, out);
                    return;
                }
                next = ResetStep::Account;
            }
            ResetStep::Account => {
                match row.account {
                    Some(account) => out.push(Request::ResetAct(ResetAct::RemoveAccount { account })),
                    None => erase(d, out),
                }
                return;
            }
        }
    }
}

pub(crate) fn acted(d: &mut Domain, env: &Env<Limits>, act: ResetAct, keep: bool, out: &mut Queue<Request>) {
    if keep {
        return;
    }
    let Some(row) = d.playground else { return };
    match (row.state, act) {
        (PlaygroundState::Resetting(ResetStep::Project), ResetAct::RemoveProject { project })
            if row.project == Some(project) =>
        {
            step(d, env, ResetStep::Forge, out);
        }
        (PlaygroundState::Resetting(ResetStep::Account), ResetAct::RemoveAccount { account })
            if row.account == Some(account) =>
        {
            erase(d, out);
        }
        _ => {}
    }
}

/// Check after a removal answer, in the decision that erases its forge row.
pub(crate) fn forge_removed(d: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    if let Some(row) = d.playground
        && row.state == PlaygroundState::Resetting(ResetStep::Forge)
        && !d.records.contains_key(&row.forge)
    {
        step(d, env, ResetStep::Account, out);
    }
}

pub(crate) fn resume(d: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    let Some(row) = d.playground else { return };
    let next = match row.state {
        PlaygroundState::Running => ResetStep::Project,
        PlaygroundState::Resetting(step) => step,
    };
    step(d, env, next, out);
}

pub(crate) fn restore(d: &mut Domain, row: Playground) {
    d.playground = Some(row);
}

fn save(d: &mut Domain, row: Playground, out: &mut Queue<Request>) {
    d.playground = Some(row);
    out.push(Request::Save { read_afresh: None, release: None, record: Stored::Playground(row) });
}

fn erase(d: &mut Domain, out: &mut Queue<Request>) {
    d.playground = None;
    out.push(Request::Erase { key: Key::Playground });
    ended(d, EndAnswer::Reset, out);
}

fn ended(d: &mut Domain, answer: EndAnswer, out: &mut Queue<Request>) {
    for _ in 0..d.playground_ends.len() {
        let Some((request, token)) = d.playground_ends.first() else { break };
        let (request, token) = (*request, *token);
        d.playground_ends.remove(&request);
        d.playground_end_answers.insert(token, answer).expect("one answer per bounded end");
        out.push(Request::ApplicationDone { request, token });
    }
}

impl DemoConfig {
    #[must_use]
    pub fn within(&self) -> bool {
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
}
