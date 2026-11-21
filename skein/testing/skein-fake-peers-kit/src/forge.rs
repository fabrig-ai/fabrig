//! Forge configurations shared by simulated and TLS socket tests.
use skein_fake_forge_domain::{self as domain, api};
use skein_fake_forge_forgejo::{Config, Faults, Repository};
use skein_fake_peers::{Limits, forge::Forge};
use skein_lib::{Duration, List, Secret, bytes};
use std::{
    cell::RefCell,
    net::{Ipv4Addr, SocketAddr},
    rc::Rc,
};

pub const SEED: u64 = 23;
pub const MAX_PASSES: u32 = 1000;
pub const REPOSITORY: &[u8] = b"org/repo";
pub const BRANCH: &[u8] = b"work";
pub const USER: u64 = 1;
pub const ONE_REPLY: usize = 1;
// Setup creates commit 1; the tiny git caller stores and pushes the next commit.
pub const PUSHED_COMMIT: u64 = 2;
pub const WORK_BRANCH_BODY: &[u8] = br#"{"name":"work","commit":{"id":"975835de1c9756ce000000000000000200000000"},"effective_branch_protection_name":""}"#;
#[must_use]
pub fn address() -> SocketAddr {
    (Ipv4Addr::LOCALHOST, 0).into()
}
#[must_use]
pub fn limits() -> Limits {
    Limits {
        io: skein_io::Limits { sockets: 5, retry: Duration::from_millis(1), ..skein_io::Limits::TEST },
        connections: 2,
        queue: 64,
        plaintext: 32_768,
        ciphertext: 32_768,
        observations: 16,
        observation_bytes: 32_768,
    }
}
pub const TOKEN: &[u8] = b"1111111111111111111111111111111111111111";
#[must_use]
pub fn config() -> Config {
    let mut tokens = List::with_capacity(1);
    tokens.push((Secret::new(bytes::copy_of(TOKEN)), 1)).expect("one token");
    let mut users = List::with_capacity(2);
    users.push((1, bytes::copy_of(b"engine"))).expect("two users");
    users.push((2, bytes::copy_of(b"person"))).expect("two users");
    let mut repositories = List::with_capacity(1);
    repositories
        .push(Repository { id: 2, full: bytes::copy_of(b"org/repo"), protected: None })
        .expect("one repository");
    Config {
        base: bytes::copy_of(b"http://127.0.0.1:3000"),
        tokens,
        users,
        repositories,
        max_response_items: 2,
        default_paging_num: 30,
        sources: Box::new([]),
        hooks: None,
        faults: Faults { reset: 0, drop: 0, server_error: 0 },
    }
}
#[must_use]
pub fn protocol_limits() -> skein_fake_forge_forgejo::Limits {
    skein_fake_forge_forgejo::Limits {
        calls: 4,
        connections: 4,
        raw_bytes: 4096,
        http: skein_http::server::Limits {
            head: 4096,
            headers: 16,
            body: 16384,
            read: 1024,
            response: 4096,
            send: 1024,
        },
        documents: skein_forge::forgejo::limits::Limits {
            json: skein_json::tokenizer::Limits { chunk: 1024, string: 16384, number: 32, depth: 16, length: 65536 },
            writer: skein_json::writer::Limits { depth: 16, length: 65536 },
            string_bytes: 512,
            body_bytes: 16384,
            inner: 32,
            target_bytes: 4096,
        },
    }
}
#[must_use]
pub fn boxed(text: &[u8]) -> Box<[u8]> {
    bytes::copy_of(text)
}
#[must_use]
pub fn fake_config() -> domain::Config {
    domain::Config::test()
}

#[must_use]
pub fn shared() -> Rc<RefCell<Forge>> {
    configured_forge(SEED, fake_config(), false)
}
#[must_use]
pub fn configured_forge(seed: u64, config: domain::Config, hooked: bool) -> Rc<RefCell<Forge>> {
    configured_checks_forge(
        seed,
        config,
        hooked,
        api::Checks {
            cues: Box::new([]),
            contexts: Box::new([]),
            latency_min: Duration::ZERO,
            latency_max: Duration::ZERO,
            silent: 0,
            passes: 0,
            reruns: 0,
            cue: None,
        },
    )
}
#[must_use]
pub fn configured_checks_forge(
    seed: u64,
    config: domain::Config,
    hooked: bool,
    checks: api::Checks,
) -> Rc<RefCell<Forge>> {
    let mut domain = domain::Domain::new(&config, seed);
    domain::repository(
        &mut domain,
        &config,
        api::Setup {
            workflows: Box::new([]),
            id: 2,
            name: boxed(REPOSITORY),
            default: boxed(b"main"),
            tree: Box::new([]),
            labels: Box::new([]),
            checks,
            protection: None,
            hooked,
        },
    );
    domain::grant(&mut domain, REPOSITORY, USER, api::Permission::Admin);
    Rc::new(RefCell::new(Forge::new(domain, config)))
}

#[must_use]
pub fn request(path: &[u8]) -> Vec<u8> {
    let mut request = b"GET ".to_vec();
    request.extend_from_slice(path);
    request.extend_from_slice(b" HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: token ");
    request.extend_from_slice(TOKEN);
    request.extend_from_slice(b"\r\nConnection: close\r\n\r\n");
    request
}
