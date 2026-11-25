//! Scenario configuration and observations of Skein's independent LLM peer.
//! Only real macOS peers reach the table of listeners held until adoption.
//! The peer owns byte routing and its script. Contract: smith/docs/design/testing.md, sections 4 and 5.
use skein_fake_llm_domain::{self as domain, api};
use skein_fake_llm_protocol::{documents, provider};
use skein_fake_peers::{Limits, Transport, llm};
use skein_lib::Duration;

use skein_io::kernel::Fd;
use std::{
    collections::BTreeMap,
    net::{SocketAddr, SocketAddrV4},
    sync::Mutex,
};

static REAL: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static MACOS: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

#[cfg(not(test))]
static LISTENERS: Mutex<BTreeMap<u16, HeldListener>> = Mutex::new(BTreeMap::new());

#[cfg(test)]
use tests::LISTENERS;

struct HeldListener {
    fd: Fd,
    address: SocketAddr,
    adopted: bool,
}

impl Drop for HeldListener {
    fn drop(&mut self) {
        if !self.adopted {
            skein_shell::close_startup_fd(self.fd);
        }
    }
}

/// A scratch scope restores its peers' previous loopback mode when it ends.
pub struct RealLoopback {
    previous: bool,
    previous_macos: bool,
    previous_listeners: BTreeMap<u16, HeldListener>,
}

impl Drop for RealLoopback {
    fn drop(&mut self) {
        REAL.store(self.previous, std::sync::atomic::Ordering::Relaxed);
        MACOS.store(self.previous_macos, std::sync::atomic::Ordering::Relaxed);
        *LISTENERS.lock().expect("fixture listeners") = std::mem::take(&mut self.previous_listeners);
    }
}

/// Select real-host peer addresses until the scratch scope ends.
#[must_use]
pub fn real_loopback() -> RealLoopback {
    real_loopback_on(cfg!(target_os = "macos"))
}

/// Select a real host's address policy, including macOS's held path on Linux.
#[must_use]
pub fn real_loopback_on(macos: bool) -> RealLoopback {
    RealLoopback {
        previous: REAL.swap(true, std::sync::atomic::Ordering::Relaxed),
        previous_macos: MACOS.swap(macos, std::sync::atomic::Ordering::Relaxed),
        previous_listeners: std::mem::take(&mut *LISTENERS.lock().expect("fixture listeners")),
    }
}

/// Pure bind choice: macOS real peers request a held kernel port; others keep theirs.
#[must_use]
pub fn listen_on(real: bool, macos: bool, process_id: u32, port: u16) -> SocketAddrV4 {
    let host = if real && !macos { loopback_for(process_id) } else { std::net::Ipv4Addr::LOCALHOST };
    SocketAddrV4::new(host, if real && macos { 0 } else { port })
}

fn selected(port: u16) -> SocketAddrV4 {
    listen_on(
        REAL.load(std::sync::atomic::Ordering::Relaxed),
        MACOS.load(std::sync::atomic::Ordering::Relaxed),
        std::process::id(),
        port,
    )
}

/// The address shared by settings and the peer, reserved until adoption.
#[must_use]
pub fn address(port: u16) -> SocketAddr {
    let selected = selected(port);
    if selected.port() != 0 {
        return selected.into();
    }
    LISTENERS
        .lock()
        .expect("fixture listeners")
        .entry(port)
        .or_insert_with(|| {
            let (fd, address) = skein_shell::open_listener(selected.into(), limits().io.backlog)
                .expect("held fake listener on localhost");
            HeldListener { fd, address, adopted: false }
        })
        .address
}

/// Startup descriptors to declare before constructing a hosted real peer.
#[must_use]
pub fn listener_fds(port: u16) -> Vec<Fd> {
    if selected(port).port() != 0 {
        return Vec::new();
    }
    let _address = address(port);
    LISTENERS.lock().expect("fixture listeners").get(&port).map_or_else(Vec::new, |held| vec![held.fd])
}

/// Transfer the held listener to its peer once, before its first io pass.
#[must_use]
pub fn take_listener(port: u16) -> Option<(Fd, SocketAddr)> {
    if selected(port).port() != 0 {
        return None;
    }
    let _address = address(port);
    LISTENERS.lock().expect("fixture listeners").get_mut(&port).map(|held| {
        assert!(!held.adopted, "one fake per reserved port in a scratch scope");
        held.adopted = true;
        (held.fd, held.address)
    })
}

/// A Linux process id maps to one distinct address within 127/8.
#[must_use]
pub fn loopback_for(process_id: u32) -> std::net::Ipv4Addr {
    let address = std::net::Ipv4Addr::from(0x7f01_0000_u32.checked_add(process_id).expect("Linux process id fits"));
    assert!(address.is_loopback(), "Linux process id stays within loopback");
    address
}

/// Real Linux peers use this process's address; macOS and simulation use localhost.
#[must_use]
pub fn loopback() -> std::net::Ipv4Addr {
    *selected(443).ip()
}

/// Real sign-in takes one free localhost port per process; simulation keeps 2345.
#[must_use]
pub fn redirect_port() -> u16 {
    static PORT: std::sync::OnceLock<u16> = std::sync::OnceLock::new();
    if !REAL.load(std::sync::atomic::Ordering::Relaxed) {
        return 2345;
    }
    *PORT.get_or_init(|| {
        std::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))
            .expect("free sign-in redirect port")
            .local_addr()
            .expect("redirect listener address")
            .port()
    })
}

/// Bounded loopback transport and observation settings for Smith's peers.
#[must_use]
pub fn limits() -> Limits {
    Limits {
        io: skein_io::Limits {
            sockets: 8,
            intake: 32_768,
            output: 65_536,
            close_timeout: Duration::from_millis(50),
            ..super::limits().io
        },
        connections: 4,
        queue: 256,
        plaintext: 32_768,
        ciphertext: 65_536,
        observations: 256,
        observation_bytes: 4_194_304,
    }
}

/// The one-turn agent fixture's provider response, sent on plaintext loopback.
#[must_use]
pub fn peer() -> llm::Peer {
    peer_for(false)
}

/// The same one-turn text response in the selected provider dialect.
#[must_use]
pub fn peer_for(anthropic: bool) -> llm::Peer {
    let scripts = one_turn();
    let credential = credential(anthropic);
    if anthropic { configured_anthropic(scripts, credential) } else { configured(scripts, credential, Duration::ZERO) }
}

/// The one-turn product fixture on the public Responses route.
#[must_use]
pub fn peer_openai() -> llm::Peer {
    Options { openai: true, ..Options::default() }
        .make(one_turn(), skein_llm::Credential::bearer(b"token".as_slice().into()))
}

fn one_turn() -> Box<[api::Script]> {
    let turns = Box::new([smith_agent_world::fixture::says(
        Box::new([api::Line::Text { text: b"Hello".as_slice().into() }]),
        1,
    )]);
    Box::new([api::Script { cue: Box::new([]), turns }])
}

/// The shared scenario credential for the selected provider dialect.
#[must_use]
pub fn credential(anthropic: bool) -> skein_llm::Credential {
    let value = b"token".as_slice().into();
    if anthropic { skein_llm::Credential::key(value) } else { skein_llm::Credential::bearer(value) }
}

/// Script a reply with the shared scenario's standard credential.
#[must_use]
pub fn scripted_reply(turns: Box<[api::Turn]>, anthropic: bool) -> llm::Peer {
    scripted(turns, credential(anthropic), anthropic)
}

/// Write the supplied turns through the scenario's provider dialect.
#[must_use]
pub fn scripted(turns: Box<[api::Turn]>, credential: skein_llm::Credential, anthropic: bool) -> llm::Peer {
    let scripts = Box::new([api::Script { cue: Box::new([]), turns }]);
    // Live pressure stories pass 16 KiB by one byte; script and document room
    // admit those turns and their replay within 128 KiB.
    Options { anthropic, request_bytes: 131_072, document_bytes: 131_072, ..Options::default() }
        .make(scripts, credential)
}

/// Build a seeded plaintext loopback peer for a local scenario.
#[must_use]
pub fn configured(scripts: Box<[api::Script]>, credential: skein_llm::Credential, latency: Duration) -> llm::Peer {
    Options { latency, ..Options::default() }.make(scripts, credential)
}

/// Configure the same fake domain for plaintext replay or TLS in a real loop.
#[must_use]
pub fn configured_transport(
    scripts: Box<[api::Script]>,
    credential: skein_llm::Credential,
    latency: Duration,
    transport: Transport,
) -> llm::Peer {
    Options { latency, transport, ..Options::default() }.make(scripts, credential)
}

/// Script the public Responses route for product worlds on either transport.
#[must_use]
pub fn configured_openai(
    scripts: Box<[api::Script]>,
    credential: skein_llm::Credential,
    latency: Duration,
    transport: Transport,
) -> llm::Peer {
    Options { openai: true, latency, transport, ..Options::default() }.make(scripts, credential)
}

/// Select Anthropic's real byte codec for the native live-text scenario.
#[must_use]
pub fn configured_anthropic(scripts: Box<[api::Script]>, credential: skein_llm::Credential) -> llm::Peer {
    Options { anthropic: true, ..Options::default() }.make(scripts, credential)
}

/// An independent HTTP provider rejection, with no application decoder substitute.
#[must_use]
pub fn rejected() -> llm::Peer {
    Options { rejected: true, ..Options::default() }.make(Box::new([]), credential(false))
}

struct Options {
    latency: Duration,
    transport: Transport,
    anthropic: bool,
    openai: bool,
    rejected: bool,
    request_bytes: u32,
    document_bytes: u32,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            latency: Duration::ZERO,
            transport: Transport::Plaintext,
            anthropic: false,
            openai: false,
            rejected: false,
            request_bytes: 32_768,
            document_bytes: 16_384,
        }
    }
}

impl Options {
    fn make(self, scripts: Box<[api::Script]>, credential: skein_llm::Credential) -> llm::Peer {
        let Self { latency, transport, anthropic, openai, rejected, request_bytes, document_bytes } = self;
        let mut config = skein_llm_kit::fake::config();
        config.latency_min = latency;
        config.latency_max = latency;
        config.script_bytes = config.script_bytes.max(document_bytes);
        config.answer_bytes = config.answer_bytes.max(document_bytes);
        config.query_bytes = config.query_bytes.max(document_bytes);
        if rejected {
            config.too_long = 1000;
        }
        let mut protocol = skein_llm_kit::fake::limits(&skein_llm_kit::limits());
        protocol.documents.openai.request = request_bytes;
        protocol.documents.openai.strings = document_bytes;
        protocol.documents.openai.answer = protocol.documents.openai.answer.max(document_bytes);
        protocol.http.body = protocol.http.body.max(u64::from(request_bytes));
        protocol.sse.event = protocol.sse.event.max(document_bytes);
        protocol.documents.openai.tokens = 4096;
        protocol.documents.openai.output_items = 256;
        protocol.documents.anthropic.output_items = 256;
        let port = if transport == Transport::Tls { 34_443 } else { 443 };
        let mut peer = llm::Peer::new(
            address(port),
            transport,
            limits(),
            provider::Config {
                list: None,
                list_status: None,
                echo: skein_llm::openai::Echo::NONE,
                usage_fields: documents::UsageFields::ALL,
                provider: if anthropic { documents::Provider::Anthropic } else { documents::Provider::OpenAi },
                path: if anthropic {
                    skein_llm::Endpoint::anthropic().target
                } else if openai {
                    skein_llm::Endpoint::openai().target
                } else {
                    skein_llm_kit::public_call(1).endpoint.target
                },
                headers: Box::new([]),
                without_credential: false,
            },
            credential,
            protocol,
            domain::Domain::try_scripted(&config, 17, scripts).expect("bounded scenario scripts"),
            config,
        )
        .expect("configured independent peer");
        if let Some((fd, address)) = take_listener(port) {
            peer.adopt_listener(fd, address).expect("provider adopts held listener");
        }
        peer
    }
}

/// Decoded outside requests emitted by the independent byte peer.
pub fn queries(peer: &llm::Peer) -> impl Iterator<Item = &api::Query> {
    peer.observations().iter().filter_map(|observation| match observation {
        llm::Observation::Query { query, .. } => Some(query),
        llm::Observation::Head { .. }
        | llm::Observation::Accepted { .. }
        | llm::Observation::Answered { .. }
        | llm::Observation::Closed { .. }
        | llm::Observation::Credential { .. }
        | llm::Observation::List { .. } => None,
    })
}

/// Whether the script domain emitted a successful terminal.
#[must_use]
pub fn replied(peer: &llm::Peer) -> bool {
    peer.observations()
        .iter()
        .any(|observation| matches!(observation, llm::Observation::Answered { result: Ok(()), .. }))
}

/// Whether the independent provider terminated with the configured context rejection.
#[must_use]
pub fn context_rejected(peer: &llm::Peer) -> bool {
    peer.observations().iter().any(|observation| {
        matches!(observation, llm::Observation::Answered { result: Err(api::Error::ContextTooLong), .. })
    })
}

/// Admit the observed reasoning turn when it is replayed in the next request.
pub(crate) fn reasoning(scripts: Box<[api::Script]>) -> llm::Peer {
    Options::default().make(scripts, credential(false))
}

#[cfg(test)]
mod tests {
    use super::{BTreeMap, Fd, HeldListener, Mutex, address, listener_fds, real_loopback_on, take_listener};
    use std::{
        cell::Cell,
        net::{Ipv4Addr, SocketAddr},
        sync::{LockResult, MutexGuard},
    };

    // Provider ports for plaintext and TLS respectively.
    const PORTS: [u16; 2] = [443, 34_443];

    thread_local! {
        static LOCK_FORBIDDEN: Cell<bool> = const { Cell::new(false) };
    }

    // Check every attempted lock before it can block.
    pub(super) static LISTENERS: ListenerTable = ListenerTable(Mutex::new(BTreeMap::new()));

    pub(super) struct ListenerTable(Mutex<BTreeMap<u16, HeldListener>>);

    impl ListenerTable {
        pub(super) fn lock(&self) -> LockResult<MutexGuard<'_, BTreeMap<u16, HeldListener>>> {
            assert!(!LOCK_FORBIDDEN.get(), "unheld peer reached the fixture listener lock");
            self.0.lock()
        }
    }

    struct ForbidListenerLock;

    impl ForbidListenerLock {
        fn new() -> Self {
            LOCK_FORBIDDEN.set(true);
            Self
        }
    }

    impl Drop for ForbidListenerLock {
        fn drop(&mut self) {
            LOCK_FORBIDDEN.set(false);
        }
    }

    fn assert_no_held_listener(port: u16) {
        assert_eq!(listener_fds(port), Vec::<Fd>::new());
        assert_eq!(take_listener(port), None);
    }

    #[test]
    fn peers_without_held_listeners_do_not_lock_the_table() {
        {
            let _no_lock = ForbidListenerLock::new();
            for port in PORTS {
                assert_eq!(address(port), SocketAddr::from((Ipv4Addr::LOCALHOST, port)));
                assert_no_held_listener(port);
            }
        }

        let _linux = real_loopback_on(false);
        let _no_lock = ForbidListenerLock::new();
        for port in PORTS {
            assert_no_held_listener(port);
        }
    }
}
