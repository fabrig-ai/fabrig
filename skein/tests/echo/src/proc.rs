//! The processes of the echo's worlds: the echo, as its shell runs it, and
//! the fake clients, as theirs would; one type, so that a world hosts both.

use skein_echo_client::{self as client, Client, Plan};
use skein_echo_service::{self as service, Service};
use skein_echo_shell::Echo;
use skein_io::Addr;
use skein_io::kernel::{Complete, Submit};
use skein_lib::{Queue, Time, Wall};
use skein_world::{Dispatch, Host};

/// A process of an echo world.
#[derive(Debug)]
pub enum Proc {
    Echo { svc: Box<Echo> },
    Client { client: Box<ClientHost> },
}

impl Proc {
    /// The echo under `limits`, to listen at `addr`.
    #[must_use]
    pub fn echo(limits: service::Limits, addr: Addr, seed: u64) -> Proc {
        Proc::Echo { svc: Box::new(Echo::new(limits, addr, seed)) }
    }

    /// A fake client under `limits`, its connections following `plans`.
    #[must_use]
    pub fn client(limits: client::Limits, plans: &[Plan]) -> Proc {
        Proc::Client { client: Box::new(ClientHost { client: Client::new(&limits, plans), limits }) }
    }

    /// The echo, if this is it.
    #[must_use]
    pub fn as_echo(&self) -> Option<&Service> {
        match self {
            Proc::Echo { svc, .. } => Some(&svc.svc),
            Proc::Client { .. } => None,
        }
    }

    /// The fake client, if this is one.
    #[must_use]
    pub fn as_client(&self) -> Option<&Client> {
        match self {
            Proc::Client { client, .. } => Some(&client.client),
            Proc::Echo { .. } => None,
        }
    }
}

impl Dispatch for Proc {
    fn host(&self) -> &dyn Host {
        match self {
            Self::Echo { svc } => svc.as_ref(),
            Self::Client { client } => client.as_ref(),
        }
    }
    fn host_mut(&mut self) -> &mut dyn Host {
        match self {
            Self::Echo { svc } => svc.as_mut(),
            Self::Client { client } => client.as_mut(),
        }
    }
}

/// The scripted client's host face (examples.md, 6), with its ring price.
#[derive(Debug)]
pub struct ClientHost {
    client: Client,
    limits: client::Limits,
}

impl ClientHost {
    /// Supplies the address learned by the referee.
    pub fn dial(&mut self, addr: Addr) {
        self.client.dial(addr);
    }

    /// Whether every scripted connection has finished.
    #[must_use]
    pub fn done(&self) -> bool {
        self.client.done()
    }
}

impl Host for ClientHost {
    fn iterate(&mut self, now: Time, wall: Wall) {
        client::iterate(&mut self.client, now, wall);
    }
    fn completions(&mut self) -> &mut Queue<Complete> {
        self.client.completions()
    }
    fn submissions(&mut self) -> &mut Queue<Submit> {
        self.client.submissions()
    }
    fn work_pending(&self, now: Time) -> bool {
        self.client.work_pending(now)
    }
    fn next_deadline(&self) -> Option<Time> {
        self.client.next_deadline()
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        self.client.next_policy_deadline()
    }
    fn is_empty(&self) -> bool {
        self.client.is_empty()
    }
    fn operations(&self) -> u32 {
        client::operations(&self.limits).expect("a small ring")
    }
}
