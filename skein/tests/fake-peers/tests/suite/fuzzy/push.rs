//! Bounded generated and mutated decoders, seeded as the other fuzzy suites are.
use skein_lib::Rng;
use skein_push::{Answer, Endpoint, answer};
#[test]
fn endpoints_round_trip_and_answers_never_panic() {
    let mut rng = Rng::new(0x8291);
    let mut parsed = 0;
    let mut refused = 0;
    for round in 0..20_000 {
        let mut bytes = if round % 3 == 0 {
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
    assert!(parsed > 1000 && refused > 1000, "both parser outcomes fell");
    for status in 0..=u16::MAX {
        let retry = Some(u32::try_from(rng.next_u64() & 0xffff_ffff).expect("u32"));
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
