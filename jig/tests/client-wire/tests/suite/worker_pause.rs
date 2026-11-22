//! Pause records keep their generated fixed-rule values (protocol/channel/pause.md; engine/workers.md, 7).
use jig_client_protocol_wire::{self as w, Record as _};
use skein_lib::Reader;

fn round_trip<T: w::Record<Limits = w::Limits>>(name: &str) {
    let bytes = super::client_values::fixed_value(name);
    let mut input = Reader::new(bytes);
    let value = T::decode(&w::CEILINGS, &mut input).expect("pause fixed-rule value");
    assert!(input.is_empty());
    assert_eq!(value.encode(&w::CEILINGS).expect("pause record fits").as_ref(), bytes);
}

#[test]
fn pause_records_keep_their_tags_and_round_trip_their_fixed_values() {
    for (name, expected) in [
        ("enum_worker_pause_mode_resume_smallest.bin", w::WorkerPauseMode::Resume),
        ("enum_worker_pause_mode_pause_smallest.bin", w::WorkerPauseMode::Pause),
        ("enum_worker_pause_mode_drain_smallest.bin", w::WorkerPauseMode::Drain),
    ] {
        let bytes = super::client_values::fixed_value(name);
        assert_eq!(
            w::WorkerPauseMode::decode(&w::CEILINGS, &mut Reader::new(bytes)).expect("pause record fits"),
            expected
        );
        round_trip::<w::WorkerPauseMode>(name);
    }
    for (name, expected) in [
        ("enum_worker_pause_taking_smallest.bin", w::WorkerPause::Taking),
        ("enum_worker_pause_paused_smallest.bin", w::WorkerPause::Paused),
        ("enum_worker_pause_draining_smallest.bin", w::WorkerPause::Draining),
        ("enum_worker_pause_drained_smallest.bin", w::WorkerPause::Drained),
    ] {
        let bytes = super::client_values::fixed_value(name);
        assert_eq!(w::WorkerPause::decode(&w::CEILINGS, &mut Reader::new(bytes)).expect("pause record fits"), expected);
        round_trip::<w::WorkerPause>(name);
    }
    for name in ["enum_ask_pause_worker_smallest.bin", "enum_ask_pause_worker_full.bin"] {
        round_trip::<w::Ask>(name);
    }
    for name in ["enum_outcome_worker_pause_asked_smallest.bin", "enum_outcome_worker_pause_asked_full.bin"] {
        round_trip::<w::Outcome>(name);
    }
    for name in ["enum_refusal_unlinked_smallest.bin", "enum_refusal_unlinked_full.bin"] {
        round_trip::<w::Refusal>(name);
    }
    for name in ["record_worker_state_smallest.bin", "record_worker_state_full.bin"] {
        round_trip::<w::WorkerState>(name);
    }
}
