use crate::read_support::request;
use crate::row_parts::{ack, close, lines, note, open, query, world};
use jig_core_world::fixtures;
use jig_domain_people as people;
use skein_lib::Rng;
use skein_world::{sweep::cases, this_test};
use std::convert::Infallible;

#[test]
fn random_endings_and_notes_keep_marks_monotone_and_each_ending_visible() {
    cases(this_test!("random_endings_and_notes_keep_marks_monotone_and_each_ending_visible"), 1230..1238, |seed| {
        let mut rng = Rng::new(seed);
        let mut w = world(seed);
        open(&mut w, 600, query(&[1], false));
        let mut greatest = lines(&w, 600).ended_mark;
        for round in 0_u8..6 {
            let task = w.task();
            if rng.below(2) == 0 {
                ack(&mut w, 600);
            }
            if rng.below(2) == 0 {
                note(&mut w, task, 710 + u64::from(round), None);
                assert!(lines(&w, 600).ended_mark >= greatest);
                greatest = lines(&w, 600).ended_mark;
            }
            greatest = greatest.max(lines(&w, 600).ended_mark);
            fixtures::finish_chat(&mut w, 0, b"random ending");
            // Acknowledging releases any coalesced mark from this ending.
            ack(&mut w, 600);
            let after = lines(&w, 600).ended_mark;
            assert!(after > greatest, "every selected ending becomes a greater mark: seed={seed}, round={round}");
            greatest = after;
            if rng.below(2) == 0 {
                note(&mut w, task, 730 + u64::from(round), None);
                assert!(lines(&w, 600).ended_mark >= greatest);
                greatest = lines(&w, 600).ended_mark;
            }
            if round != 5 {
                let people::Ask::Create { project, mut batch } = fixtures::chat(&w, 1, b"next".as_slice().into())
                else {
                    panic!("chat")
                };
                batch[0].label = u32::from(round) + 1;
                request(&mut w, 50 + round, people::Ask::Create { project, batch });
            }
        }
        close(&mut w, 600);
        Ok::<(), Infallible>(())
    })
    .unwrap_or_else(|case| panic!("{case}"));
}
