use super::*;
use crate::payload::{self, Payload};
use core::mem::size_of;

#[test]
fn scripted_owner_hold_moves_return_exact_category_for_release_and_atomic_amend_release() {
    for amendment in [false, true] {
        let mut setup = script();
        let mut row = task();
        let reason = HoldCategory::Failures { class: HoldFailure::Agent };
        row.hold = Some(Box::new(OwnerChipHold {
            words: None,
            placed: None,
            circle: None,
            account: None,
            source: None,
            reason,
            revision: 7,
            before: OwnerPriorPhase::Waiting,
            escalation: None,
            tries: OwnerFailureTries { transient: 1, permanent: 2, run: 3, agent: 4, lost: 5, invalid: 6 },
            hold_wait_since: None,
        }));
        row.acts = list(vec![Offered {
            effect: None,
            act: if amendment { ActKind::Amend } else { ActKind::Release },
            admission: Admission::Allowed,
        }]);
        let spec_revision = row.spec_revision;
        setup.truth.objects.insert(ObjectKey::Task(8), ReadValue::Task(Box::new(row)));
        let mut engine = Engine::new(setup, 1).unwrap();
        let ask = if amendment {
            crate::tests::releasing(
                Ask::Amend {
                    reason: text(b""),
                    project: 1,
                    task: 8,
                    revision: Some(spec_revision),

                    change: Box::new(Amendment {
                        llm: None,
                        addressed: None,
                        spec: None,
                        wake: None,
                        dependencies: None,
                        authority: None,
                        instructions: Some(text(b"")),
                        tracked: None,
                    }),
                },
                Revision { value: 6 },
            )
        } else {
            Ask::Release { project: 1, task: 8, revision: Revision { value: 6 }, again: None }
        };
        send(&mut engine, 0, 11, 1, 1, ask);
        assert_eq!(
            final_answer(&engine.poll(Time::from_nanos(10_000_000))[0]),
            &Answer::Refused {
                refusal: Refusal::Moved {
                    now: Revision { value: 7 },
                    what: Moved::Hold { reason: Some(CurrentHold::Reason(reason)) },
                },
                at: None
            }
        );
    }
}

const UTF8_TEXTS: [&[u8]; 1] = [&[b'!'; 32]];
