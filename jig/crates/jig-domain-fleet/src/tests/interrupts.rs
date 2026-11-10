//! A stopped engine forgets an attempt without waiting for fleet resumption (jig/docs/design/domain/engine/restart.md, 1).
use super::{A1, C1, FAILED, Harness, LIMITS, R1, drop, hosted, turn};
use crate::{Event, Phase};

#[derive(Clone, Copy, Debug)]
enum Placement {
    Waiting,
    Hosted,
    Adrift,
    Answered,
    Stray,
}

#[test]
fn interrupt_forgets_placements_and_turns_without_telling_the_host() {
    for placement in [Placement::Waiting, Placement::Hosted, Placement::Adrift, Placement::Answered, Placement::Stray] {
        let mut h = Harness::new(crate::Limits { turns: 2, ..LIMITS });
        let dropped: Box<[crate::Request]> = match placement {
            Placement::Waiting => {
                h.start(R1, A1, 1);
                Box::new([])
            }
            Placement::Hosted | Placement::Adrift | Placement::Answered => {
                h.hello(C1, 2, &[], &[]);
                h.place(R1, A1, 1);
                turn(&mut h, C1, 1, 1);
                match placement {
                    Placement::Adrift => {
                        h.step(Event::Lost { channel: C1 });
                    }
                    Placement::Answered => {
                        h.answer(C1, R1, A1, FAILED, 2);
                    }
                    Placement::Hosted => {}
                    Placement::Waiting | Placement::Stray => unreachable!("placed row"),
                }
                Box::new([])
            }
            Placement::Stray => {
                h.hello(C1, 2, &[], &[hosted(R1, A1, Phase::Active)]);
                turn(&mut h, C1, 1, 1);
                Box::new([drop(1)])
            }
        };
        assert_eq!(
            &*h.step(Event::Interrupt { run: R1, attempt: A1 }),
            &*dropped,
            "{placement:?}: only held payloads leave"
        );
        assert_eq!(h.domain.turns(), 0, "{placement:?}: pending turns forgotten without resume");
        assert_eq!(h.domain.attempts(), 0, "{placement:?}");
        assert_eq!(h.domain.waiting(), 0, "{placement:?}");
        assert_eq!(h.domain.claims, 0, "{placement:?}");
        assert!(h.domain.runs.is_empty(), "{placement:?}");
        assert!(h.domain.names.is_empty(), "{placement:?}");
        assert_eq!(h.domain.next_deadline(), None, "{placement:?}");
        h.domain.reclaim();
        assert!(h.step(Event::Interrupt { run: R1, attempt: A1 }).is_empty(), "{placement:?}: idempotent");
        assert!(h.settle().is_empty(), "{placement:?}: no delayed host notifications");
    }
}
