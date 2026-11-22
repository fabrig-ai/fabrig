//! Offline controls for the pair's deciding evidence.
use super::*;

#[test]
fn a_refresh_pair_requires_two_completions_with_a_new_generation() {
    let mut pair = [End::Completed, End::Completed];
    assert_eq!(
        proof(&pair, &Ok(Some(4)), &Ok(Some(5)), Path::new("synthetic-record")),
        Measure::Observed { value: true }
    );
    assert_eq!(
        proof(&pair, &Ok(Some(4)), &Ok(Some(4)), Path::new("synthetic-record")),
        Measure::Observed { value: false }
    );
    pair[1] = End::Refused { setup: "refresh reused".into() };
    assert_eq!(
        proof(&pair, &Ok(Some(4)), &Ok(Some(5)), Path::new("synthetic-record")),
        Measure::Observed { value: false }
    );
}

#[test]
fn an_unreadable_generation_never_passes_the_pair() {
    assert!(matches!(
        proof(
            &[End::Completed, End::Completed],
            &Err(Fault::new(crate::FaultKind::Credential, "cannot inspect generation")),
            &Ok(Some(5)),
            Path::new("synthetic-record")
        ),
        Measure::Unavailable { .. }
    ));
    for (before, after) in [(Ok(None), Ok(Some(5))), (Ok(Some(4)), Ok(None))] {
        assert_eq!(
            proof(&[End::Completed, End::Completed], &before, &after, Path::new("synthetic-record")),
            Measure::unavailable("missing harness sign-in record: synthetic-record"),
        );
    }
}
