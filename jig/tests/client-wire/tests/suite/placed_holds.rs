//! Present placement fields round trip at their smallest and full constructor values.
use jig_client_protocol_wire as w;
use skein_lib::{Reader, Writer};

#[test]
fn task_holds_with_a_placed_line_round_trip_their_smallest_and_full_fields() {
    for full in [false, true] {
        let number = if full { u64::MAX } else { 0 };
        let owner = if full {
            w::HoldOwner::Task(
                w::HoldOwnerTask::new(&w::CEILINGS, w::HoldOwnerTaskParts { task: number }).expect("task owner"),
            )
        } else {
            w::HoldOwner::Party(
                w::HoldOwnerParty::new(&w::CEILINGS, w::HoldOwnerPartyParts { person: number }).expect("party owner"),
            )
        };
        let scope = if full {
            w::HoldScope::Project
        } else {
            w::HoldScope::Subtree(
                w::HoldScopeSubtree::new(&w::CEILINGS, w::HoldScopeSubtreeParts { task: number }).expect("subtree"),
            )
        };
        let placed = w::PlacedLine::new(
            &w::CEILINGS,
            w::PlacedLineParts {
                number,
                owner,
                scope,
                mode: if full { w::HoldMode::Drain } else { w::HoldMode::Stop },
                reason: if full {
                    vec![255; w::CEILINGS.placed_line_reason as usize].into_boxed_slice()
                } else {
                    Box::new([])
                },
                placed_at: number,
                revision: number,
                lifted: full,
                may_lift: full,
            },
        )
        .expect("placement");
        let hold = w::ChipHold::new(
            &w::CEILINGS,
            w::ChipHoldParts {
                words: None,
                circle: None,
                account: None,
                source: None,
                reason: w::HoldCategory::Placed(
                    w::HoldCategoryPlaced::new(&w::CEILINGS, w::HoldCategoryPlacedParts { hold: number })
                        .expect("placed category"),
                ),
                revision: number,
                before: w::PriorPhase::Waiting,
                escalation: None,
                tries: w::FailureTries::new(
                    &w::CEILINGS,
                    w::FailureTriesParts {
                        transient: 0,
                        permanent: 0,
                        run: 0,
                        agent: 0,
                        lost: 0,
                        invalid: 0,
                        infrastructure: 0,
                    },
                )
                .expect("tries"),
                hold_wait_since: None,
                placed: Some(placed),
            },
        )
        .expect("held");
        let value = hold;
        let mut out = Writer::new(value.measure() as usize);
        value.encode(&mut out).expect("task hold bytes");
        let bytes = out.finish();
        let mut input = Reader::new(&bytes);
        let decoded = w::ChipHold::decode(&w::CEILINGS, &mut input).expect("placed hold");
        assert!(input.is_empty());
        assert_eq!(decoded, value);
        let mut out = Writer::new(skein_lib::length::usize_of(decoded.measure()));
        decoded.encode(&mut out).expect("measured placed hold");
        assert_eq!(out.finish(), bytes);
    }
}
