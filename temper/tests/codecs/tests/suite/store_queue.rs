//! The runtime-30 merge queue is durable at v11 in place (protocol/store.md, 4).
use crate::fixture;
use skein_forge::api::{Answer, Repository, Write};
use skein_forge_client::{Condition, Effect, Entry, Outcome};
use temper_engine_domain_forge::{CallPayload, NamedCall, Stored};
use temper_protocol_store::{STORE_VERSION, records};

fn entry(queued: bool) -> Stored {
    Stored::Entry {
        entry: Entry {
            number: 1,
            task: 2,
            repository: Repository { forge: 1, repository: 3 },
            effect: Effect {
                write: Write::Merge { number: 4, head: [5; 32], how: skein_forge::api::MergeHow::Style },
                condition: Condition::Merge { base: b"main".as_slice().into() },
            },
            start: None,
            attempt: None,
            failures: 0,
            queued,
        },
        held: false,
    }
}
#[test]
fn queued_and_unqueued_entries_have_v11_golden_bytes_and_round_trip() {
    let bounds =
        temper_protocol_store::bounds(&fixture::LIMITS, fixture::LIMITS.client.op_bytes).expect("fixture bounds");
    assert_eq!(STORE_VERSION, 11);
    for queued in [false, true] {
        let value = entry(queued);
        let bytes = records::encode(&value);
        let name = if queued { "merge-queued.bin" } else { "merge-unqueued.bin" };
        let path = skein_scratch::paths::package().join("../../crates/temper-protocol-store/golden/v11").join(name);
        if temper_test_env::rewrite() {
            std::fs::write(&path, &bytes).expect("write queue golden");
        }
        assert_eq!(std::fs::read(path).expect("committed queue golden").as_slice(), bytes.as_ref());
        assert_eq!(records::decode(&bytes, &bounds), Ok(value));
    }
}
#[test]
fn queued_effect_outcomes_and_merge_answers_round_trip_durably() {
    let bounds =
        temper_protocol_store::bounds(&fixture::LIMITS, fixture::LIMITS.client.op_bytes).expect("fixture bounds");
    for answer in [
        CallPayload::Effect { entry: 1, deadline: skein_lib::Wall::EPOCH, outcome: Some(Outcome::Queued) },
        CallPayload::Read(Box::new(Ok(Answer::Queued))),
        CallPayload::Decided { took: None, reach: None },
        CallPayload::Decided {
            took: Some(Box::from(b"Small".as_slice())),
            reach: Some(temper_protocol_reaches::Reach {
                kind: temper_protocol_reaches::Kind::Wording,
                class: temper_protocol_reaches::Class::Decide,
                reason: temper_protocol_reaches::Reason::Kind,
                under: 0,
                sets: 0,
                matched: 0,
                away: false,
            }),
        },
    ] {
        let value = Stored::Call { key: NamedCall { task: 1, attempt: 2, completion: 3, position: 4 }, answer };
        let bytes = records::encode(&value);
        assert_eq!(records::decode(&bytes, &bounds), Ok(value));
    }
}

#[test]
fn not_installed_call_and_application_reasons_round_trip_distinctly_at_v11() {
    use temper_engine_domain_forge::{AdoptionRefusal, ApplicationOutcome};
    let bounds =
        temper_protocol_store::bounds(&fixture::LIMITS, fixture::LIMITS.client.op_bytes).expect("fixture bounds");
    for value in [
        Stored::Call {
            key: NamedCall { task: 1, attempt: 2, completion: 3, position: 4 },
            answer: CallPayload::Read(Box::new(Err(skein_forge::api::Error::NotInstalled))),
        },
        Stored::Application {
            token: 1,
            until: skein_lib::Wall::EPOCH,
            outcome: ApplicationOutcome::ForgeRefused(AdoptionRefusal::NotInstalled),
        },
    ] {
        let bytes = records::encode(&value);
        assert_eq!(STORE_VERSION, 11);
        assert_eq!(records::decode(&bytes, &bounds), Ok(value));
    }
}
