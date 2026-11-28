//! Each sweep keeps one mutation stream across its fixtures (skein/docs/design/testing/sweeps.md, 4.2).

#![cfg(test)]
use crate::common;
use common::{forgejo, generated, mutated, mutated_into};
use skein_forge::forgejo::{client, server};
use skein_lib::Rng;
use skein_world::{
    sweep::{Case, cases, gate},
    this_test,
};
use std::{convert::Infallible, ops::Range};

#[test]
fn forgejo_decoders_answer_every_mutated_transcript_without_panicking() -> Result<(), Case<Infallible>> {
    let test = this_test!("forgejo_decoders_answer_every_mutated_transcript_without_panicking");
    cases(test, [0x06_05], |seed| {
        let mut rng = Rng::new(seed);
        let mut mutations = common::MutationBatch::default();
        for (case, request) in forgejo::json_cases() {
            let body = forgejo::transcript(&case).body;
            let mut bytes = Vec::new();
            for _ in 0..32 {
                let mutated = mutated_into(&mut rng, &body, &mut bytes);
                mutations.judge(mutated, |bytes| {
                    let decoded = match &request {
                        Some(request) => forgejo::decode(request, bytes),
                        None => forgejo::decode_error(bytes),
                    };
                    assert!(decoded.steps <= forgejo::STEP_BOUND, "{case}: decode terminates within its bound");
                });
            }
        }
        mutations.finish();
        Ok(())
    })
}

#[test]
fn forgejo_server_routes_answer_every_mutated_request_without_panicking() -> Result<(), Case<Infallible>> {
    let test = this_test!("forgejo_server_routes_answer_every_mutated_request_without_panicking");
    cases(test, [0x06_05_02], |seed| {
        let mut rng = Rng::new(seed);
        let limits = forgejo::limits();
        let mut mutations = common::MutationBatch::default();
        for case in forgejo::transcript_cases() {
            let transcript = forgejo::transcript(&case);
            for _ in 0..200 {
                let target = mutated(&mut rng, &transcript.target);
                let body = mutated(&mut rng, &transcript.request_body);
                mutations.judge(target.zip(body), |(target, body)| {
                    if let Ok((_, steps)) = forgejo::decode_route(transcript.method, &target, &body, &limits) {
                        assert!(steps <= forgejo::STEP_BOUND, "{case}: route terminates within its bound");
                    }
                });
            }
        }
        mutations.finish();
        Ok(())
    })
}

#[test]
fn forgejo_delivery_decoders_answer_every_mutated_delivery_without_panicking() -> Result<(), Case<Infallible>> {
    let test = this_test!("forgejo_delivery_decoders_answer_every_mutated_delivery_without_panicking");
    cases(test, [0x04_02], |seed| {
        let mut rng = Rng::new(seed);
        let limits = forgejo::profile_limits();
        let mut mutations = common::MutationBatch::default();
        for (case, _) in forgejo::delivery_cases() {
            let sent = forgejo::delivery(case);
            let mut bytes = Vec::new();
            for _ in 0..16 {
                let mutated = mutated_into(&mut rng, &sent.body, &mut bytes);
                mutations.judge(mutated, |bytes| {
                    let _decoded = forgejo::decode_delivery(&sent.event, bytes, &limits);
                });
            }
        }
        mutations.finish();
        Ok(())
    })
}

const GENERATED_STREAM_SEED: u64 = 0x06_03;
const GENERATED_ROUNDS: u64 = 32;

fn fixture_variants(seed: u64, at_gate: bool) -> Range<u64> {
    // Gate variants run in order from zero. Fresh cases shift the same number
    // of rounds, bounding the shift in u64 before the fixtures add or narrow it.
    let first = if at_gate { 0 } else { seed % GENERATED_ROUNDS };
    first..first + GENERATED_ROUNDS
}

#[test]
fn generated_fixture_variants_keep_gate_order_and_accept_full_width_seeds() {
    assert_eq!(fixture_variants(GENERATED_STREAM_SEED, true), 0..32);

    let variants = fixture_variants(u64::MAX, false);
    assert_eq!(variants, 31..63);
    let largest = variants.end - 1;
    assert!(!generated::documents(largest).is_empty());
    assert_ne!(generated::deliveries(largest), []);
}

#[test]
fn generated_documents_and_their_mutations_end_within_the_step_bound() -> Result<(), Case<Infallible>> {
    let test = this_test!("generated_documents_and_their_mutations_end_within_the_step_bound");
    cases(test, [GENERATED_STREAM_SEED], |seed| {
        check_generated_documents(seed, fixture_variants(seed, gate()));
        Ok(())
    })
}

#[expect(clippy::similar_names, reason = "the request and reply names follow the dialect vocabulary")]
fn check_generated_documents(stream_seed: u64, variants: Range<u64>) {
    const MUTATIONS: u32 = 8;
    let limits = generated::limits();
    let mut rng = Rng::new(stream_seed);
    let mut bytes = Vec::new();
    let mut mutations = common::MutationBatch::default();
    for seed in variants {
        for (name, request) in generated::request_documents() {
            let sent = client::encode(&request, &limits).expect("generated request");
            let Some(body) = sent.body.as_deref() else {
                continue;
            };
            assert_eq!(
                forgejo::decode_route(sent.method, &sent.target, body, &limits).expect("valid request").0,
                request,
                "{seed}/{name}"
            );
            for _ in 0..MUTATIONS {
                let mutated = mutated_into(&mut rng, body, &mut bytes);
                mutations.judge(mutated, |bytes| {
                    if let Ok((_, steps)) = forgejo::decode_route(sent.method, &sent.target, bytes, &limits) {
                        assert!(steps <= forgejo::STEP_BOUND, "{seed}/{name}: bounded request");
                    }
                });
            }
        }
        for reply in generated::documents(seed) {
            let body = server::encode(&reply.answer, &limits).expect("generated answer");
            let decoder = || match &reply.request {
                Some(request) => client::decoder(request, &limits).expect("answer decoder"),
                None => client::error_decoder(&limits),
            };
            let decoded = forgejo::decode_answer(decoder(), &body, &limits);
            assert_eq!(decoded.document, Ok(reply.document), "{seed}/{}", reply.name);
            assert_eq!(decoded.rows, reply.rows, "{seed}/{}", reply.name);
            for _ in 0..MUTATIONS {
                let mutated = mutated_into(&mut rng, &body, &mut bytes);
                mutations.judge(mutated, |bytes| {
                    let decoded = forgejo::decode_answer(decoder(), bytes, &limits);
                    assert!(decoded.steps <= forgejo::STEP_BOUND, "{seed}/{}: bounded answer", reply.name);
                });
            }
        }
        for (name, delivery) in generated::deliveries(seed) {
            let sent = server::delivery::encode(&delivery, &limits).expect("generated delivery");
            assert_eq!(forgejo::decode_delivery(sent.event, &sent.body, &limits), Ok(delivery), "{seed}/{name}");
            for _ in 0..MUTATIONS {
                let mutated = mutated_into(&mut rng, &sent.body, &mut bytes);
                mutations.judge(mutated, |bytes| {
                    let (_, steps) = forgejo::decode_delivery_steps(sent.event, bytes, &limits);
                    assert!(steps <= forgejo::STEP_BOUND, "{seed}/{name}: bounded delivery");
                });
            }
        }
    }
    mutations.finish();
}
