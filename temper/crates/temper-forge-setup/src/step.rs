//! The box's ordered commands and lifetimes (temper/docs/design/protocol/setup/box.md, 2).
//! Source retries follow temper/docs/design/protocol/openid-provider.md, 9.
use crate::machine::{BoxKind, Machine};
use alloc::boxed::Box;
use skein_lib::{List, bytes};
/// The word carried by a group's step lines (protocol/setup/box.md, 3).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Stage {
    Fetch,
    Prepare,
    Run,
    Source,
}
impl Stage {
    #[must_use]
    pub fn word(self) -> &'static [u8] {
        match self {
            Self::Fetch => b"fetch",
            Self::Prepare => b"prepare",
            Self::Run => b"run",
            Self::Source => b"source",
        }
    }
}
/// A dormant forge's steps and their place in the always chain.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Group {
    pub kind: Option<BoxKind>,
    pub remover: Option<Step>,
    pub forge: u16,
    /// Owned opaque bytes the box uses to name the group on the terminal (protocol/setup/box.md, 3).
    pub label: Box<[u8]>,
    /// `recorded` whether its `started` writes the box's record: an owned forge's does, the demo's does not.
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
/// How the box runs a step (protocol/setup/box.md, 2).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    Once,
    Long,
    Link,
    /// Ends the group's start once, then retries until the source serves.
    Source,
}
/// One command of the temper binary that the box runs.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Step {
    name: Box<[u8]>,
    arguments: Box<[Box<[u8]>]>,
    kind: Kind,
    ignore_ending: bool,
    ready: Option<core::net::SocketAddr>,
    listeners: Box<[core::net::SocketAddr]>,
    started_notice: Box<[u8]>,
}
impl Step {
    /// Constructs a step from its display name, arguments and lifetime.
    #[must_use]
    pub fn new(name: Box<[u8]>, arguments: Box<[Box<[u8]>]>, kind: Kind) -> Self {
        Self {
            name,
            arguments,
            kind,
            ignore_ending: false,
            ready: None,
            listeners: Box::new([]),
            started_notice: Box::new([]),
        }
    }
    /// Its group goes on however it ends (protocol/setup/box.md, 2).
    #[must_use]
    pub fn ending_ignored(mut self) -> Self {
        self.ignore_ending = true;
        self
    }
    #[must_use]
    pub fn ignores_ending(&self) -> bool {
        self.ignore_ending || self.kind == Kind::Source
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
    /// be claimed by a `Listen` naming port zero (protocol/setup/box.md, 1).
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
fn serve(flags: &[Box<[u8]>], listeners: Box<[core::net::SocketAddr]>, record: bool) -> Step {
    let mut arguments = List::with_capacity(
        u32::try_from(flags.len()).expect("bounded flags").checked_add(4).expect("bounded arguments"),
    );
    arguments.push(bytes::copy_of(b"internal")).expect("command");
    arguments.push(bytes::copy_of(b"serve")).expect("command");
    for flag in flags {
        arguments.push(flag.clone()).expect("flags counted");
    }
    arguments.push(bytes::copy_of(b"--box")).expect("box flag");
    if record {
        arguments.push(bytes::copy_of(b"--record")).expect("record flag");
    }
    Step::new(bytes::copy_of(b"serve"), arguments.into_boxed(), Kind::Long).listening(listeners)
}

/// The argument a slot names its forge by; the box puts the bound number in its place (temper/docs/design/protocol/setup/box.md, 1).
pub const FORGE_ARGUMENT: &[u8] = b"$forge";

/// The owned kind's slot and serve (temper/docs/design/protocol/setup/box.md, 1).
#[must_use]
pub fn kinds(
    machine: &Machine,
    flags: &[Box<[u8]>],
    listeners: Box<[core::net::SocketAddr]>,
    forgejo_listens: bool,
    record: bool,
) -> Steps {
    let mut result = Steps { always: Box::new([serve(flags, listeners, record)]), asked: Box::new([]) };
    if let Some(owned) = &machine.owned {
        let mut group = List::with_capacity(5);
        if owned.fetch.forgejo || owned.fetch.runner {
            group.push((Stage::Fetch, slot(b"fetch", flags, None))).expect("fetch");
        }
        group.push((Stage::Prepare, slot(b"prepare", flags, None))).expect("prepare");
        let mut forgejo = slot(b"run", flags, Some(b"forgejo")).with_ready(owned.listen);
        if forgejo_listens {
            forgejo = forgejo.listening(Box::new([owned.listen]));
        }
        group.push((Stage::Run, forgejo)).expect("forgejo");
        if owned.runner.is_some() {
            group.push((Stage::Run, slot(b"run", flags, Some(b"runner")))).expect("runner");
        }
        if owned.sign_in.is_some() {
            group.push((Stage::Source, slot(b"source", flags, None).ending_ignored())).expect("source");
        }
        result.asked = Box::new([Group {
            kind: Some(BoxKind::Owned),
            remover: Some(slot(b"remove", flags, None)),
            forge: 0,
            label: bytes::copy_of(b"Forge $forge (Forgejo)"),
            recorded: true,
            // A restored source fetches discovery while Forgejo starts.
            before: u32::from(owned.sign_in.is_some()),
            steps: group.into_boxed(),
        }]);
    }
    result
}
fn slot(action: &[u8], flags: &[Box<[u8]>], run: Option<&[u8]>) -> Step {
    let mut args = List::with_capacity(
        u32::try_from(flags.len()).expect("bounded flags").checked_add(4).expect("bounded arguments"),
    );
    args.push(bytes::copy_of(b"forge")).expect("command");
    args.push(bytes::copy_of(action)).expect("action");
    for flag in flags {
        args.push(flag.clone()).expect("flags counted");
    }
    args.push(bytes::copy_of(FORGE_ARGUMENT)).expect("forge argument");
    let name = match run {
        Some(which) => {
            args.push(bytes::copy_of(which)).expect("run argument");
            which
        }
        None => action,
    };
    let kind = if run.is_some() {
        Kind::Long
    } else if action == b"source" {
        Kind::Source
    } else {
        Kind::Once
    };
    Step::new(bytes::copy_of(name), args.into_boxed(), kind)
}
