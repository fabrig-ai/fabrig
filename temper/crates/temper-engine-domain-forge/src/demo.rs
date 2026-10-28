//! The forge connector reads the record: with no demo, or one ended, it
//! refuses, *not offered* (protocol/setup.md, 8.4).
use crate::{Connection, Demo, DemoConfig, DemoState, Domain, Request, StartAnswer, Stored};
use alloc::boxed::Box;
use skein_lib::{Queue, Token};

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
    #[must_use]
    pub const fn demo_state(&self) -> Option<DemoState> {
        self.demo_state
    }
    /// Ended state comes from the demo record, independently of routing events.
    #[must_use]
    pub fn forge_ended(&self, forge: u16) -> bool {
        self.demo_state == Some(DemoState::Ended)
            && match &self.demo {
                Some(demo) => demo.adoption.forge == forge,
                None => false,
            }
    }
    #[must_use]
    pub fn demo_adopted(&self) -> bool {
        let Some(demo) = &self.demo else { return false };
        let wanted = &demo.adoption;
        for (_, row) in &self.repositories {
            if row.project == wanted.project
                && row.provider.forge == wanted.forge
                && row.owner == wanted.owner
                && row.name == wanted.name
            {
                return true;
            }
        }
        false
    }
}

fn needs_group(d: &Domain) -> bool {
    let Some(demo) = &d.demo else { return false };
    d.under_box && needs_connection(&d.connection(demo.adoption.forge))
}

fn needs_connection(connection: &Connection) -> bool {
    match connection {
        Connection::Connected { .. } | Connection::Connecting => false,
        Connection::NotConnected
        | Connection::NotSetUp
        | Connection::SettingUp { .. }
        | Connection::SetUpFailed { .. }
        | Connection::Stopped { .. } => true,
    }
}

pub(crate) fn start(d: &mut Domain, request: Token, token: Token, out: &mut Queue<Request>) {
    if d.demo_pending.len() == d.demo_pending.capacity() {
        out.push(Request::ApplicationAgain { request, why: jig_domain_people::Again::Busy });
        return;
    }
    let first = d.demo_pending.is_empty();
    d.demo_pending.insert(request, token).expect("bounded starts");
    if !first {
        return;
    }
    d.demo_answers = skein_lib::Map::with_capacity(d.demo_answers.capacity());
    let Some(demo) = &d.demo else {
        answered(d, StartAnswer::NotOffered, out);
        return;
    };
    if d.demo_state == Some(DemoState::Ended) {
        answered(d, StartAnswer::NotOffered, out);
        return;
    }
    if d.demo_state.is_none() {
        d.demo_state = Some(DemoState::Running);
        out.push(Request::Save {
            read_afresh: None,
            release: None,
            record: Stored::Demo(Demo { forge: demo.adoption.forge, state: DemoState::Running }),
        });
    }
    if needs_group(d) {
        out.push(Request::SetUpGroup { forge: d.demo.as_ref().expect("offered demo").adoption.forge, request });
    }
    out.push(Request::DemoStart { request });
}

pub(crate) fn answered(d: &mut Domain, answer: StartAnswer, out: &mut Queue<Request>) {
    for _ in 0..d.demo_pending.len() {
        let Some((request, token)) = d.demo_pending.first() else { break };
        let (request, token) = (*request, *token);
        d.demo_pending.remove(&request);
        d.demo_answers.insert(token, answer.clone()).expect("one answer per bounded start");
        out.push(Request::ApplicationDone { request, token });
    }
}

/// The account and the adoption are jig's records and stay; no act is made again.
pub(crate) fn restart(d: &Domain, out: &mut Queue<Request>) {
    if d.demo_state == Some(DemoState::Running) && needs_group(d) {
        out.push(Request::SetUpGroup {
            forge: d.demo.as_ref().expect("configured demo").adoption.forge,
            request: Token::new(u64::MAX),
        });
    }
}

/// The first other configured forge ends the playground in this decision
/// (protocol/setup.md, 8.4). Its records remain available to lookups.
pub(crate) fn connected(d: &mut Domain, forge: u16, out: &mut Queue<Request>) {
    let Some(demo) = &d.demo else { return };
    let demo_forge = demo.adoption.forge;
    if forge == demo_forge || !d.configured.contains_key(&forge) || d.demo_state == Some(DemoState::Ended) {
        return;
    }
    let ran = d.demo_state == Some(DemoState::Running);
    d.demo_state = Some(DemoState::Ended);
    out.push(Request::Save {
        read_afresh: None,
        release: None,
        record: Stored::Demo(Demo { forge: demo_forge, state: DemoState::Ended }),
    });
    if ran {
        out.push(Request::ListDemoActs);
    }
    if ran && d.under_box {
        out.push(Request::StopGroup { forge: demo_forge });
    }
    answered(d, StartAnswer::NotOffered, out);
}

pub(crate) fn acts(d: &mut Domain, acts: Box<[crate::DemoAct]>, out: &mut Queue<Request>) {
    for row in acts {
        d.demo_acts.insert(row.key, row.act).expect("end selection fits tasks plus one");
        out.push(Request::Save { read_afresh: None, release: None, record: Stored::DemoAct(row) });
    }
    send_acts(d, out);
}

pub(crate) fn send_acts(d: &Domain, out: &mut Queue<Request>) {
    for (key, act) in &d.demo_acts {
        out.push(Request::SendDemoAct(crate::DemoAct { key: *key, act: *act }));
    }
}

pub(crate) fn act_answered(d: &mut Domain, key: [u8; 16], keep: bool, out: &mut Queue<Request>) {
    if !keep && d.demo_acts.remove(&key).is_some() {
        out.push(Request::Erase { key: crate::Key::DemoAct(key) });
    }
}
