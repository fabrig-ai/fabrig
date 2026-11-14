//! Tier acts round trip under client wire 22 (client/forge/records.md).
use crate::{self as wire, v1 as w};
use skein_lib::Reader;
#[test]
fn tier_requests_and_outcomes_round_trip() {
    for repository in [
        None,
        Some(
            w::DraftRepository::new(
                &w::CEILINGS,
                w::DraftRepositoryParts { owner: b"acme".as_slice().into(), name: b"site".as_slice().into() },
            )
            .unwrap(),
        ),
    ] {
        for act in [
            w::TierAct::Hold(
                w::TierHoldAct::new(&w::CEILINGS, w::TierHoldActParts { reason: b"maintenance".as_slice().into() })
                    .unwrap(),
            ),
            w::TierAct::Release,
        ] {
            let request = w::TierActRequest::new(
                &w::CEILINGS,
                w::TierActRequestParts {
                    project: 2,
                    repository: repository.clone(),
                    tier: b"browser".as_slice().into(),
                    act,
                },
            )
            .unwrap();
            let requesting =
                w::Requesting::new(&w::CEILINGS, w::RequestingParts { request: w::Request::TierAct(request) }).unwrap();
            let bytes = wire::requesting(&requesting);
            assert_eq!(wire::decode_requesting(&bytes, &w::CEILINGS), Some(requesting));
            assert_eq!(bytes.get(..2), Some(22_u16.to_be_bytes().as_slice()));
        }
    }
    for refusal in [
        None,
        Some(w::TierRefusal::NoRepository),
        Some(w::TierRefusal::NoTier),
        Some(w::TierRefusal::Held(
            w::TierHeldBy::new(
                &w::CEILINGS,
                w::TierHeldByParts { party: 19, reason: b"maintenance".as_slice().into() },
            )
            .unwrap(),
        )),
        Some(w::TierRefusal::NotHeld),
        Some(w::TierRefusal::Words),
    ] {
        let answer = w::Answer::new(
            &w::CEILINGS,
            w::AnswerParts {
                outcome: w::Outcome::Tier(w::TierOutcome::new(&w::CEILINGS, w::TierOutcomeParts { refusal }).unwrap()),
            },
        )
        .unwrap();
        assert_eq!(w::Answer::decode(&w::CEILINGS, &mut Reader::new(&wire::answer(&answer))).unwrap(), answer);
    }
}
