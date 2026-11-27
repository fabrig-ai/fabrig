//! Endpoint and retry draws share one stream per case (skein/docs/design/testing/sweeps.md, 4.2 to 4.5).

use core::convert::Infallible;
use skein_lib::Rng;
use skein_push::{Answer, Endpoint, answer};
use skein_world::{
    sweep::{cases, gate},
    this_test,
};

fn draw_endpoint(rng: &mut Rng, round: u32) -> Vec<u8> {
    let mut bytes = if round.is_multiple_of(3) {
        let length = usize::try_from(rng.below(1100)).expect("bounded length");
        let mut bytes = Vec::with_capacity(length);
        for _ in 0..length {
            bytes.push(u8::try_from(rng.below(256)).expect("byte"));
        }
        bytes
    } else {
        format!(
            "{}://push{}.test:{}/push/{}?q={}",
            if rng.chance(500) { "http" } else { "https" },
            rng.below(1000),
            rng.between(1, 65535),
            rng.next_u64(),
            rng.next_u64()
        )
        .into_bytes()
    };
    if round % 3 == 2 {
        let offset = usize::try_from(rng.below(u64::try_from(bytes.len()).expect("length"))).expect("offset");
        bytes[offset] = u8::try_from(rng.below(256)).expect("byte");
    }
    bytes
}

fn draw_retry(rng: &mut Rng) -> u32 {
    u32::try_from(rng.next_u64() & 0xffff_ffff).expect("low 32 bits")
}

#[test]
fn endpoint_and_retry_draws_accept_full_width_seeds() {
    let mut rng = Rng::new(u64::MAX);
    assert_eq!(draw_endpoint(&mut rng, 0).len(), 983);
    assert_eq!(
        draw_endpoint(&mut rng, 1).as_slice(),
        b"https://push231.test:24575/push/16032958732961140416?q=5372385094262928236"
    );
    assert_eq!(
        draw_endpoint(&mut rng, 2).as_slice(),
        b"http://push28.test:38860/push/18263578258851609579?q=1762723\xd7633907938281"
    );
    assert_eq!(draw_retry(&mut rng), 0x30f4_58a8);
}

#[test]
fn endpoints_round_trip_and_answers_never_panic() {
    let test = this_test!("endpoints_round_trip_and_answers_never_panic");
    cases(test, [0x8291], |seed| {
        check_stream(seed);
        Ok::<(), Infallible>(())
    })
    .unwrap_or_else(|case| panic!("{case}"));
}

fn check_stream(seed: u64) {
    let mut rng = Rng::new(seed);
    let mut parsed = 0;
    let mut refused = 0;
    for round in 0..20_000 {
        let bytes = draw_endpoint(&mut rng, round);
        if let Some(endpoint) = Endpoint::parse(&bytes) {
            parsed += 1;
            let mut restored = Vec::from(endpoint.origin());
            restored.extend_from_slice(endpoint.target());
            assert_eq!(restored, bytes, "round {round}");
            assert!(endpoint.target().starts_with(b"/"));
        } else {
            refused += 1;
        }
    }
    if gate() {
        assert!(parsed > 1000 && refused > 1000, "both parser outcomes fell");
    }
    for status in 0..=u16::MAX {
        let retry = Some(draw_retry(&mut rng));
        match answer(status, retry) {
            Answer::Made => assert!((200..300).contains(&status)),
            Answer::Gone => assert!(status == 404 || status == 410),
            Answer::TooLarge => assert_eq!(status, 413),
            Answer::Later(after) => {
                assert!(status == 429 || (500..600).contains(&status));
                assert_eq!(after, retry);
            }
            Answer::Refused(code) => {
                assert_eq!(code, status);
                assert!(status == 400 || status == 401 || status == 403);
            }
            Answer::Failed(code) => assert_eq!(code, status),
        }
        let _ = answer(status, None);
    }
}
