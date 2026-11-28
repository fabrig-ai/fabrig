//! Random byte peers for the batch's client end (jig/docs/design/protocol/client/default-http.md, 6).
#[cfg(test)]
mod fuzzy {
    use core::convert::Infallible;

    use jig_client_protocol_http::{Frames, Heard, Limits};
    use skein_lib::{List, Queue, Rng, Token, Writer};
    use skein_world::{
        sweep::{Case, cases},
        this_test,
    };

    #[test]
    #[expect(clippy::disallowed_macros, reason = "this_test! names the calling package for campaign replay")]
    fn random_frame_bodies_keep_their_parts_across_random_cuts() -> Result<(), Case<Infallible>> {
        const LIMITS: Limits = Limits { record: 64, chunk: 16 };
        let exchange = Token::new(17);
        cases(this_test!("random_frame_bodies_keep_their_parts_across_random_cuts"), 0..128, |seed| {
            let mut rng = Rng::new(seed);
            let mut body = List::with_capacity(4352);
            let mut records = List::with_capacity(64);
            for _ in 0_u32..64 {
                let length = u32::try_from(rng.between(0, 64)).expect("short record");
                let mut record = Writer::new(usize::try_from(length).expect("short record"));
                for _ in 0..length {
                    record.put(&[u8::try_from(rng.below(256)).expect("one byte")]).expect("record sized");
                }
                let record = record.finish();
                for byte in length.to_be_bytes() {
                    body.push(byte).expect("body bounded");
                }
                for byte in &record {
                    body.push(*byte).expect("body bounded");
                }
                records.push(record).expect("64 frames");
            }
            let body = body.into_boxed();
            for random_cuts in [false, true] {
                let mut frames = Frames::new(exchange, &LIMITS);
                let mut out = Queue::uncapped();
                frames.opened(200, &mut out);
                let mut at = 0_usize;
                for _ in 0..body.len() {
                    if at == body.len() {
                        break;
                    }
                    let length = if random_cuts {
                        usize::try_from(rng.between(1, 16)).expect("short piece")
                    } else {
                        body.len()
                    };
                    let end = at.saturating_add(length).min(body.len());
                    frames.body(body.get(at..end).expect("piece within body"), &mut out);
                    at = end;
                }
                frames.ended(&mut out);
                for record in records.as_slice() {
                    assert_eq!(
                        out.pop(),
                        Some(Heard::Part { exchange, message: record.clone() }),
                        "seed {seed}, random {random_cuts}"
                    );
                }
                assert_eq!(out.pop(), Some(Heard::Done { exchange }), "seed {seed}");
                assert!(out.is_empty());
            }
            Ok(())
        })
    }
}
