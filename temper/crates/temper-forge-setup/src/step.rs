//! The box's ordered commands; each hides whether it is once, long or link.
use crate::{check::joined, setup::Owned};
use alloc::boxed::Box;
use skein_lib::{Decimal, List, bytes};
/// The word carried by a group's step lines (protocol/setup.md, 7.3).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Stage {
    Fetch,
    Prepare,
    Run,
}
impl Stage {
    #[must_use]
    pub fn word(self) -> &'static [u8] {
        match self {
            Self::Fetch => b"fetch",
            Self::Prepare => b"prepare",
            Self::Run => b"run",
        }
    }
}
/// A dormant forge's steps and their place in the always chain.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Group {
    pub forge: u16,
    /// Owned opaque bytes the box uses to name the group on the terminal (protocol/setup.md, 7.3).
    pub label: Box<[u8]>,
    /// Whether its `started` writes the box's record.
    pub recorded: bool,
    pub before: u32,
    pub steps: Box<[(Stage, Step)]>,
}
/// Commands run at startup, and groups run when asked.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Steps {
    pub always: Box<[Step]>,
    pub asked: Box<[Group]>,
}
/// How the box runs a step (protocol/setup.md, 7.2).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    Once,
    Long,
    Link,
}
/// One command of the temper binary that the box runs.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Step {
    name: Box<[u8]>,
    arguments: Box<[Box<[u8]>]>,
    kind: Kind,
    ready: Option<core::net::SocketAddr>,
    listeners: Box<[core::net::SocketAddr]>,
    started_notice: Box<[u8]>,
}
impl Step {
    /// Constructs a step from its display name, arguments and lifetime.
    #[must_use]
    pub fn new(name: Box<[u8]>, arguments: Box<[Box<[u8]>]>, kind: Kind) -> Self {
        Self { name, arguments, kind, ready: None, listeners: Box::new([]), started_notice: Box::new([]) }
    }
    /// Opaque public lines supplied by the binary, printed once this forge starts.
    pub fn set_started_notice(&mut self, notice: Box<[u8]>) {
        self.started_notice = notice;
    }
    #[must_use]
    pub fn started_notice(&self) -> &[u8] {
        &self.started_notice
    }
    /// The listening sockets given to this step, in descriptor order from fd 3.
    /// Port zero stays the step's own bind: its kernel-chosen address cannot
    /// be claimed by a `Listen` naming port zero (protocol/setup.md, 7.1).
    #[must_use]
    pub fn listening(mut self, addrs: Box<[core::net::SocketAddr]>) -> Self {
        let mut listeners = List::with_capacity(u32::try_from(addrs.len()).expect("bounded listeners"));
        for address in addrs {
            if address.port() != 0 {
                listeners.push(address).expect("listener capacity");
            }
        }
        self.listeners = listeners.into_boxed();
        self
    }
    #[must_use]
    pub fn listeners(&self) -> &[core::net::SocketAddr] {
        &self.listeners
    }
    /// Wait for this owned process to answer its version route before continuing.
    #[must_use]
    pub fn with_ready(mut self, address: core::net::SocketAddr) -> Self {
        self.ready = Some(address);
        self
    }
    /// The loopback endpoint whose version proves the process ready.
    #[must_use]
    pub fn ready(&self) -> Option<core::net::SocketAddr> {
        self.ready
    }
    /// How the box runs this step.
    #[must_use]
    pub fn kind(&self) -> Kind {
        self.kind
    }
    /// The step's name for prefixed output.
    #[must_use]
    pub fn name(&self) -> &[u8] {
        &self.name
    }
    /// Arguments passed to temper.
    #[must_use]
    pub fn arguments(&self) -> &[Box<[u8]>] {
        &self.arguments
    }
}
fn forge(action: &[u8], flags: &[Box<[u8]>], number: u16, run: Option<&[u8]>) -> Step {
    let digits = Decimal::of(u64::from(number));
    let n = digits.as_bytes();
    let mut args = List::with_capacity(
        u32::try_from(flags.len()).expect("bounded flags").checked_add(4).expect("bounded arguments"),
    );
    for a in [b"forge".as_slice(), action] {
        args.push(bytes::copy_of(a)).expect("command words");
    }
    for flag in flags {
        args.push(flag.clone()).expect("flags counted");
    }
    args.push(bytes::copy_of(n)).expect("forge number");
    let name = match run {
        Some(which) => {
            args.push(bytes::copy_of(which)).expect("run argument");
            joined(&[which, b" ", n])
        }
        None => joined(&[action, b" ", n]),
    };
    Step::new(name, args.into_boxed(), if run.is_some() { Kind::Long } else { Kind::Once })
}
/// An owned forge's recorded group, before position zero (protocol/setup.md, 7.3; notes contracts T3).
#[must_use]
pub fn owned_group(forge: u16, owned: &Owned, flags: &[Box<[u8]>], forgejo_listens: bool) -> Group {
    let mut group = List::with_capacity(5);
    if owned.fetch.forgejo || owned.fetch.runner {
        group.push((Stage::Fetch, self::forge(b"fetch", flags, forge, None))).expect("fetch");
    }
    group.push((Stage::Prepare, self::forge(b"prepare", flags, forge, None))).expect("prepare");
    // forge run absorbs Forgejo's KillParent (protocol/setup.md, 6.4, 7.1).
    let mut step = self::forge(b"run", flags, forge, Some(b"forgejo")).with_ready(owned.listen);
    if forgejo_listens {
        step = step.listening(Box::new([owned.listen]));
    }
    group.push((Stage::Run, step)).expect("forgejo");
    if owned.runner.is_some() {
        group.push((Stage::Run, self::forge(b"run", flags, forge, Some(b"runner")))).expect("runner");
    }

    let digits = Decimal::of(u64::from(forge));
    Group {
        forge,
        label: joined(&[b"Forge ", digits.as_bytes(), b" (Forgejo)"]),
        recorded: true,
        before: 0,
        steps: group.into_boxed(),
    }
}
/// The service step, with the box's absolute directory flags.
#[must_use]
pub fn steps(flags: &[Box<[u8]>], listeners: Box<[core::net::SocketAddr]>) -> Steps {
    let mut serve = List::with_capacity(
        u32::try_from(flags.len()).expect("bounded flags").checked_add(3).expect("bounded arguments"),
    );
    serve.push(bytes::copy_of(b"internal")).expect("command word");
    serve.push(bytes::copy_of(b"serve")).expect("command word");
    for flag in flags {
        serve.push(flag.clone()).expect("flags counted");
    }
    serve.push(bytes::copy_of(b"--box")).expect("box flag");
    Steps {
        always: Box::new([Step::new(bytes::copy_of(b"serve"), serve.into_boxed(), Kind::Long).listening(listeners)]),
        asked: Box::new([]),
    }
}
