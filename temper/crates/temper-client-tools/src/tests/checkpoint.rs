//! The owner's act's parsing, output, refusals and wait (temper/docs/design/protocol/checkpoints.md, 3).
use super::fixtures as f;
use crate::jig;
use skein_lib::{Duration, Time};
use temper_client_protocol_wire::{self as wire, v1 as w};

fn outcome() -> w::Outcome {
    w::Outcome::Checkpointed(w::Checkpointed {
        name: b"00000000000000000042-00000000000000ab".as_slice().into(),
        store: 42,
        secrets: 7,
        linked: 5,
        copied: 2,
        bytes: 91,
        paused: 1_234_567,
        took: 9_876_543,
        held: 123,
        removed: 1,
        skipped: 3,
    })
}
#[test]
fn checkpoint_parses_an_optional_key_and_timeout_and_prints_the_stores_fields() {
    for (json, expected) in [
        (false, b"checkpoint 00000000000000000042-00000000000000ab at commit 42, secrets at 7: 5 files linked, 2 copied (91 bytes); paused 1 ms, took 9 ms; 123 bytes held; 1 older removed, 3 skipped\n".as_slice()),
        (true, br#"{"name":"00000000000000000042-00000000000000ab","store":42,"secrets":7,"linked":5,"copied":2,"bytes":91,"paused":1234567,"took":9876543,"held":123,"removed":1,"skipped":3}"#),
    ] {
        let mut root = f::signed_in(f::PARTY);
        let call = f::call(&["deployment", "checkpoint", "--timeout", "2m", "--key", "cut"]);
        assert_eq!(crate::limits::call_deadline(&call), Duration::from_secs(120));
        let number = root.submit_prepared(call, json, None);
        let jig_client::Ask::Family { scope, record } = root.adoption_bytes(number) else { panic!("family act") };
        assert_eq!(*scope, jig_client::contract::Scope::Deployment);
        assert_eq!(wire::decode_requesting(&record.bytes, &w::CEILINGS).unwrap().request, w::Request::Checkpoint);
        assert!(root.answer(number).is_none(), "wait for the store");
        root.adoption_answer(number, outcome());
        let printed = root.answer(number).unwrap();
        assert_eq!(printed.status, 0);
        assert_eq!(printed.stdout.as_ref(), expected);
        assert!(printed.stderr.is_empty());
    }
}
#[test]
fn checkpoint_refusals_exit_two_with_the_store_words_or_not_owner() {
    let path = w::CheckpointPath { path: b"store/log-1".as_slice().into() };
    for (why, expected) in [
        (w::CheckpointWhy::NotOwner, "only an owner of the deployment takes a checkpoint"),
        (w::CheckpointWhy::InProgress, "checkpoint in progress"),
        (w::CheckpointWhy::Unavailable, "the store is not serving"),
        (w::CheckpointWhy::Files, "more than 65536 files to take"),
        (w::CheckpointWhy::Unrecognized(path.clone()), "an unrecognized file, store/log-1"),
        (w::CheckpointWhy::Vanished(path), "a selected file vanished, store/log-1"),
        (
            w::CheckpointWhy::Failed(w::CheckpointFailed {
                path: b"store/log-1".as_slice().into(),
                error: b"Permission".as_slice().into(),
            }),
            "a file failed, store/log-1: Permission",
        ),
    ] {
        let mut root = f::signed_in(f::PARTY);
        let call = root.submit(&["deployment", "checkpoint"]);
        root.adoption_answer(call, w::Outcome::CheckpointRefused(w::CheckpointRefused { why }));
        let printed = root.answer(call).unwrap();
        assert_eq!(printed.status, 2);
        assert_eq!(
            printed.stderr.as_ref(),
            alloc::fmt::format(format_args!("temper ctl: checkpoint refused: {expected}\n")).as_bytes()
        );
        assert!(printed.stdout.is_empty());
    }
}
#[test]
fn checkpoint_waits_ten_minutes_by_default_and_times_out_without_cancelling_the_act() {
    for (words, seconds) in [
        (["deployment", "checkpoint"].as_slice(), 600),
        (["deployment", "checkpoint", "--timeout", "1s"].as_slice(), 1),
    ] {
        let mut root = f::signed_in(f::PARTY);
        let call = f::call(words);
        assert_eq!(crate::limits::call_deadline(&call), Duration::from_secs(seconds));
        let number = root.submit_prepared(call, false, None);
        root.now = Time::from_nanos(Duration::from_secs(seconds).as_nanos().checked_sub(1).unwrap());
        root.fire();
        assert!(root.answer(number).is_none());
        root.now = Time::from_nanos(Duration::from_secs(seconds).as_nanos());
        root.fire();
        let printed = root.answer(number).unwrap();
        assert_eq!((printed.status, printed.file), (3, jig::FileEnd::Keep));
        assert_eq!(
            printed.stderr.as_ref(),
            alloc::fmt::format(format_args!(
                "temper ctl: no answer within {seconds}s; the checkpoint goes on in the engine\n"
            ))
            .as_bytes()
        );
        assert!(root.saved.contains_key(&number), "timeout preserves the keyed act");
    }
}
