//! Playground records and reset (temper/docs/design/protocol/setup.md, 8.4).
use crate::{
    Answered, Bot, BoxKind, Connection, Domain, EndAnswer, ForgeState, Key, Kind, Mode, Playground, PlaygroundConfig,
    PlaygroundState, Request, ResetAct, ResetStep, StartAnswer, StartFailure, Stored,
};
use alloc::boxed::Box;
use skein_lib::{Map, Queue, Time, Token};

impl Domain {
    pub fn configure_playground(&mut self, config: Option<PlaygroundConfig>) {
        self.playground_config = config;
    }

    /// Keep completed start acts in the row that a restart resets (6.1–6.2).
    pub fn playground_progress(&mut self, project: Option<u32>, account: Option<u32>) -> Option<Stored> {
        let mut row = self.playground?;
        if row.state != PlaygroundState::Starting {
            return None;
        }
        row.project = project;
        row.account = account;
        self.playground = Some(row);
        Some(Stored::Playground(row))
    }

    #[must_use]
    pub fn playground(&self) -> Option<&Playground> {
        self.playground.as_ref()
    }
}

fn save(d: &mut Domain, row: Playground, out: &mut Queue<Request>) {
    d.playground = Some(row);
    out.push(Request::Save { record: Stored::Playground(row), release: None, read_afresh: None });
}

fn answer_starts(d: &mut Domain, answer: StartAnswer, out: &mut Queue<Request>) {
    d.playground_due = None;
    d.playground_request = None;
    for _ in 0..d.playground_pending.len() {
        let Some((request, token)) = d.playground_pending.first() else { break };
        let (request, token) = (*request, *token);
        d.playground_pending.remove(&request);
        d.playground_answers.insert(token, Answered::StartPlayground(answer.clone())).expect("bounded starts");
        out.push(Request::ApplicationDone { request, token });
    }
}

fn answer_ends(d: &mut Domain, answer: EndAnswer, out: &mut Queue<Request>) {
    for _ in 0..d.playground_ends.len() {
        let Some((request, token)) = d.playground_ends.first() else { break };
        let (request, token) = (*request, *token);
        d.playground_ends.remove(&request);
        d.playground_answers.insert(token, Answered::EndPlayground(answer)).expect("bounded ends");
        out.push(Request::ApplicationDone { request, token });
    }
}

pub(crate) fn start(d: &mut Domain, now: Time, request: Token, token: Token, out: &mut Queue<Request>) {
    if d.playground_pending.len() == d.playground_pending.capacity() {
        out.push(Request::ApplicationAgain { request, why: jig_domain_people::Again::Busy });
        return;
    }
    let first = d.playground_pending.is_empty();
    d.playground_pending.insert(request, token).expect("bounded starts");
    if !first {
        return;
    }
    d.playground_answers = Map::with_capacity(d.playground_answers.capacity());
    if d.real_forge_kept() {
        answer_starts(d, StartAnswer::NotOffered, out);
        return;
    }
    if let Some(row) = d.playground {
        match row.state {
            PlaygroundState::Running => {
                answer_starts(d, StartAnswer::Running, out);
                return;
            }
            PlaygroundState::Resetting(_) => {
                answer_starts(d, StartAnswer::NotOffered, out);
                return;
            }
            PlaygroundState::Starting => {}
        }
    }
    let Some(config) = &d.playground_config else {
        answer_starts(d, StartAnswer::NotOffered, out);
        return;
    };
    d.playground_due = Some(now.saturating_add(config.deadline));
    if d.playground.is_none() {
        let place = match &d.machine {
            Some(machine) => machine.fake.clone(),
            None => None,
        };
        let Some(place) = place else {
            answer_starts(d, StartAnswer::NotOffered, out);
            return;
        };
        let bot = Bot {
            login: Box::from(b"temper".as_slice()),
            name: Box::from(b"temper".as_slice()),
            email: Box::from(b"temper@localhost".as_slice()),
        };
        let Some(forge) = crate::forges::make(d, Kind::Forgejo, Mode::Fake, place, None, false, bot, false, None, out)
        else {
            answer_starts(d, StartAnswer::Failed(StartFailure::NotConnected), out);
            return;
        };
        save(d, Playground { state: PlaygroundState::Starting, forge, project: None, account: None }, out);
    }
    start_effects(d, request, out);
}

fn needs_group(d: &Domain, forge: u16) -> bool {
    d.under_box
        && match d.connection(forge) {
            Connection::Connected { .. } | Connection::Connecting => false,
            Connection::NotConnected
            | Connection::NotSetUp
            | Connection::SettingUp { .. }
            | Connection::SetUpFailed { .. }
            | Connection::Stopped { .. } => true,
        }
}

fn start_effects(d: &mut Domain, request: Token, out: &mut Queue<Request>) {
    let Some(row) = d.playground else { return };
    if needs_group(d, row.forge) {
        out.push(Request::BoxGroup { forge: row.forge, kind: BoxKind::Fake, request });
    }
    connected(d, out);
}

pub(crate) fn connected(d: &mut Domain, out: &mut Queue<Request>) {
    let Some(row) = d.playground else { return };
    if row.state != PlaygroundState::Starting || d.playground_request.is_some() {
        return;
    }
    match d.connection(row.forge) {
        Connection::Connected { .. } => {
            let request = match d.playground_pending.first() {
                Some((request, _)) => *request,
                None => return,
            };
            d.playground_request = Some(request);
            out.push(Request::PlaygroundActs { request, forge: row.forge });
        }
        Connection::NotConnected
        | Connection::NotSetUp
        | Connection::SettingUp { .. }
        | Connection::SetUpFailed { .. }
        | Connection::Connecting
        | Connection::Stopped { .. } => {}
    }
}

pub(crate) fn made(d: &mut Domain, project: u32, account: u32, out: &mut Queue<Request>) {
    let Some(mut row) = d.playground else { return };
    if row.state != PlaygroundState::Starting {
        return;
    }
    row.state = PlaygroundState::Running;
    row.project = Some(project);
    row.account = Some(account);
    save(d, row, out);
    answer_starts(d, StartAnswer::Running, out);
}

pub(crate) fn failed(d: &mut Domain, why: StartFailure, out: &mut Queue<Request>) {
    answer_starts(d, StartAnswer::Failed(why), out);
}

pub(crate) fn expire(d: &mut Domain, now: Time, out: &mut Queue<Request>) {
    if let Some(due) = d.playground_due
        && due <= now
    {
        failed(d, StartFailure::NotConnected, out);
    }
}

pub(crate) fn end(d: &mut Domain, request: Token, token: Token, out: &mut Queue<Request>) {
    if d.playground_ends.len() == d.playground_ends.capacity() {
        out.push(Request::ApplicationAgain { request, why: jig_domain_people::Again::Busy });
        return;
    }
    let first = d.playground_ends.is_empty();
    d.playground_ends.insert(request, token).expect("bounded ends");
    if !first {
        return;
    }
    d.playground_answers = Map::with_capacity(d.playground_answers.capacity());
    let Some(mut row) = d.playground else {
        answer_ends(d, EndAnswer::NotOffered, out);
        return;
    };
    match row.state {
        PlaygroundState::Starting | PlaygroundState::Running => {
            answer_starts(d, StartAnswer::NotOffered, out);
            row.state = PlaygroundState::Resetting(ResetStep::Project);
            save(d, row, out);
        }
        PlaygroundState::Resetting(_) => {}
    }
    reset(d, out);
}

pub(crate) fn reset_key(forge: u16, step: ResetStep) -> [u8; 16] {
    let mut hash = skein_lib::sha256::Sha256::new();
    hash.update(b"temper playground reset\0");
    hash.update(&forge.to_be_bytes());
    hash.update(match step {
        ResetStep::Project => b"project",
        ResetStep::Forge => b"forge",
        ResetStep::Account => b"account",
    });
    let mut key = [0; 16];
    for (slot, byte) in key.iter_mut().zip(hash.finish()) {
        *slot = byte;
    }
    key
}

fn reset(d: &mut Domain, out: &mut Queue<Request>) {
    // Three ordered steps bound this decision's fall-through.
    for _ in 0_u8..3 {
        let Some(mut row) = d.playground else { return };
        match row.state {
            PlaygroundState::Starting | PlaygroundState::Running => return,
            PlaygroundState::Resetting(ResetStep::Project) => {
                if let Some(project) = row.project {
                    out.push(Request::ResetAct {
                        key: reset_key(row.forge, ResetStep::Project),
                        act: ResetAct::RemoveProject { project },
                    });
                    return;
                }
                row.state = PlaygroundState::Resetting(ResetStep::Forge);
                save(d, row, out);
            }
            PlaygroundState::Resetting(ResetStep::Forge) => {
                if crate::removal::begin(d, row.forge, out) {
                    return;
                }
                // An ended forge can still be waiting for its secrets or box (6.2).
                removed(d, out);
                return;
            }
            PlaygroundState::Resetting(ResetStep::Account) => {
                if let Some(account) = row.account {
                    out.push(Request::ResetAct {
                        key: reset_key(row.forge, ResetStep::Account),
                        act: ResetAct::RemoveAccount { account },
                    });
                    return;
                }
                finish(d, out);
                return;
            }
        }
    }
}

fn finish(d: &mut Domain, out: &mut Queue<Request>) {
    d.playground = None;
    out.push(Request::Erase { key: Key::Playground });
    answer_ends(d, EndAnswer::Reset, out);
}

pub(crate) fn act_answered(d: &mut Domain, key: [u8; 16], keep: bool, out: &mut Queue<Request>) {
    let Some(mut row) = d.playground else { return };
    match row.state {
        PlaygroundState::Starting | PlaygroundState::Running => return,
        PlaygroundState::Resetting(step) => {
            if keep || key != reset_key(row.forge, step) {
                return;
            }
            match step {
                ResetStep::Project => row.state = PlaygroundState::Resetting(ResetStep::Forge),
                ResetStep::Forge => return,
                ResetStep::Account => {
                    finish(d, out);
                    return;
                }
            }
        }
    }
    save(d, row, out);
    reset(d, out);
}

pub(crate) fn removed(d: &mut Domain, out: &mut Queue<Request>) {
    let Some(mut row) = d.playground else { return };
    let complete = match d.record(row.forge) {
        None => true,
        Some(record) => match record.state {
            ForgeState::Kept => false,
            ForgeState::Ended { secrets_erased, box_removed } => secrets_erased && box_removed,
        },
    };
    if row.state == PlaygroundState::Resetting(ResetStep::Forge) && complete {
        row.state = PlaygroundState::Resetting(ResetStep::Account);
        save(d, row, out);
        reset(d, out);
    }
}

pub(crate) fn restart(d: &mut Domain, out: &mut Queue<Request>) {
    let Some(mut row) = d.playground else { return };
    match row.state {
        PlaygroundState::Starting => {
            d.playground_request = None;
            row.state = PlaygroundState::Resetting(ResetStep::Project);
            save(d, row, out);
            reset(d, out);
        }
        PlaygroundState::Running => {
            if needs_group(d, row.forge) {
                out.push(Request::BoxGroup { forge: row.forge, kind: BoxKind::Fake, request: Token::new(u64::MAX) });
            }
        }
        PlaygroundState::Resetting(_) => reset(d, out),
    }
}
