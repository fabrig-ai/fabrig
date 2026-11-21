//! Seeded argument and loopback request bytes, with no resolver or network.
use skein_lib::{
    Rng, Writer,
    base64::{self, Alphabet},
};
use temper_client_shell_tui::{
    loopback::{self, Returned},
    start::{self, Refused},
};
use temper_client_tui::Origin;

fn check(bytes: &[u8], seed: u64) {
    match start::address(bytes) {
        Ok(origin) => {
            assert_eq!(Origin::parse(&origin.print()), Ok(origin.clone()), "seed {seed}, {bytes:?}");
            if let Origin::Https { name, .. } = origin {
                let name = std::str::from_utf8(&name).expect("ASCII name");
                assert!(skein_tls::Name::new(name).is_some(), "seed {seed}, TLS name {name}");
            }
        }
        Err(refused) => {
            let expected = match &refused {
                Refused::NotAnAddress(argument) => {
                    assert_eq!(argument.as_ref(), bytes, "seed {seed}");
                    format!("temper: not an address: {}\n", String::from_utf8_lossy(bytes))
                }
                Refused::HttpName => {
                    "temper: http:// reaches an engine on this machine alone: use https://NAME\n".into()
                }
                Refused::HttpsAddress => "temper: https:// needs the deployment's name\n".into(),
                Refused::Link => {
                    "temper: a sign-in link is never an argument: run temper http://HOST:PORT and paste it\n".into()
                }
                Refused::NotLoopback => "temper: the engine's address must be on loopback\n".into(),
                Refused::NotATerminal | Refused::Directory(_) | Refused::Resolve { .. } | Refused::Roots(_) => {
                    panic!("seed {seed}: not an argument refusal: {refused:?}")
                }
            };
            assert_eq!(refused.line("temper"), expected, "seed {seed}, {bytes:?}");
        }
    }
}

#[test]
fn argument_bytes_and_mutated_origins_never_panic_and_keep_their_meaning() {
    let origins = [
        b"http://127.0.0.1:8367".as_slice(),
        b"http://[::1]:9000/",
        b"https://dev.fabrig.ai",
        b"https://Dev.Fabrig.AI:8443/",
        b"https://a.123a:65535",
        b"https://box.123",
        b"https://127.0.0.1",
        b"http://dev.fabrig.ai",
        b"http://10.0.0.1:8367",
        b"http://127.0.0.1:8367/jig/v1/sign-in/link?secret=AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
    ];
    for seed in 1..=16 {
        let mut rng = Rng::new(seed);
        for origin in origins {
            check(origin, seed);
        }
        for _ in 0..512 {
            let size = usize::try_from(rng.below(320)).expect("argument size");
            let bytes: Vec<u8> = (0..size).map(|_| u8::try_from(rng.below(256)).expect("byte")).collect();
            check(&bytes, seed);
            let origin = origins
                .get(usize::try_from(rng.below(u64::try_from(origins.len()).expect("origins"))).expect("index"))
                .expect("origin");
            let mut bytes = origin.to_vec();
            let at = usize::try_from(rng.below(u64::try_from(bytes.len()).expect("length"))).expect("offset");
            let byte = u8::try_from(rng.below(256)).expect("mutation byte");
            match rng.below(3) {
                0 => {
                    bytes.remove(at);
                }
                1 => {
                    bytes.insert(at, byte);
                }
                _ => {
                    *bytes.get_mut(at).expect("mutation position") = byte;
                }
            }
            check(&bytes, seed);
        }
    }
}

const RETURN_HOST: &[u8] = b"127.0.0.1:43210";

fn naive_grant(request: &Returned) -> Option<Box<[u8]>> {
    if !request.get || request.host.as_deref() != Some(RETURN_HOST) {
        return None;
    }
    let (prefix, text) = request.target.split_at_checked(16)?;
    if prefix != b"/approved?grant=" {
        return None;
    }
    let bytes = base64::decode(Alphabet::Url, text, 32).ok()?;
    (bytes.len() == 32).then_some(bytes)
}

fn random_bytes(rng: &mut Rng, len: usize) -> Box<[u8]> {
    (0..len).map(|_| u8::try_from(rng.below(256)).expect("byte")).collect()
}

fn mutate(rng: &mut Rng, bytes: &mut [u8]) {
    let len = u64::try_from(bytes.len()).expect("small fixture");
    let at = usize::try_from(rng.below(len)).expect("offset");
    let flips = u8::try_from(1 + rng.below(255)).expect("nonzero byte");
    *bytes.get_mut(at).expect("mutation position") ^= flips;
}

#[test]
fn loopback_grants_agree_with_the_naive_rule_for_bytes_and_mutations() {
    for seed in 1..=16 {
        let mut rng = Rng::new(seed);
        for _ in 0..128 {
            let mut encoded = Writer::new(43);
            base64::encode(Alphabet::Url, &random_bytes(&mut rng, 32), &mut encoded).expect("43 bytes");
            let good = Returned {
                get: true,
                target: [b"/approved?grant=".as_slice(), &encoded.finish()].concat().into(),
                host: Some(RETURN_HOST.into()),
            };
            let mut target_mutation = good.clone();
            mutate(&mut rng, &mut target_mutation.target);
            let mut host_mutation = good.clone();
            mutate(&mut rng, host_mutation.host.as_deref_mut().expect("Host"));
            let target_len = usize::try_from(rng.below(96)).expect("size");
            let host_len = usize::try_from(rng.below(32)).expect("size");
            let random_host = Some(random_bytes(&mut rng, host_len));
            for request in [
                target_mutation,
                host_mutation,
                Returned { target: random_bytes(&mut rng, target_len), ..good.clone() },
                Returned { host: random_host, ..good.clone() },
                Returned { get: false, ..good.clone() },
                good,
            ] {
                assert_eq!(
                    loopback::grant(&request, 43210).map(skein_lib::Secret::into_exposed),
                    naive_grant(&request),
                    "seed {seed}, method get={}, target={:?}, Host={:?}",
                    request.get,
                    request.target,
                    request.host,
                );
            }
        }
    }
}
