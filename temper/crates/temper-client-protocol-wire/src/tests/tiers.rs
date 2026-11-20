//! Tier acts round trip under the current client wire (client/forge/records.md).
use crate::v1::Record;
use crate::{self as wire, v1 as w};
use skein_lib::Reader;
#[test]
fn tier_requests_and_outcomes_round_trip() {
    for repository in
        [None, Some(w::DraftRepository { owner: b"acme".as_slice().into(), name: b"site".as_slice().into() })]
    {
        for act in [w::TierAct::Hold(w::TierHoldAct { reason: b"maintenance".as_slice().into() }), w::TierAct::Release]
        {
            let request = w::TierActRequest {
                project: 2,
                repository: repository.clone(),
                tier: b"browser".as_slice().into(),
                act,
            };
            let requesting = w::Requesting { request: w::Request::TierAct(request) };
            let bytes = wire::requesting(&requesting);
            assert_eq!(wire::decode_requesting(&bytes, &w::CEILINGS), Some(requesting));
            assert_eq!(bytes.get(..2), Some(u16::try_from(wire::CLIENT_VERSION).unwrap().to_be_bytes().as_slice()));
        }
    }
    for refusal in [
        None,
        Some(w::TierRefusal::NoRepository),
        Some(w::TierRefusal::NoTier),
        Some(w::TierRefusal::Held(w::TierHeldBy { party: 19, reason: b"maintenance".as_slice().into() })),
        Some(w::TierRefusal::NotHeld),
        Some(w::TierRefusal::Words),
    ] {
        let answer = w::Answer { outcome: w::Outcome::Tier(w::TierOutcome { refusal }) };
        assert_eq!(w::Answer::decode(&w::CEILINGS, &mut Reader::new(&wire::answer(&answer))).unwrap(), answer);
    }
}
