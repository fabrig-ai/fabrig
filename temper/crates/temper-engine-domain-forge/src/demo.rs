//! Demo configuration and legacy acts (temper/docs/design/protocol/setup.md, 8.4).
use crate::{DemoConfig, DemoState, Domain, Request, Stored};
use alloc::boxed::Box;
use skein_lib::Queue;

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
