//! Writer releases preserve live links (jig/docs/design/domain/engine/restart.md, Load, then link).
use super::*;
use skein_forge::api::{Answer, Error, Op, Read};

const HOLDER: u64 = 3;
const WRITER: u64 = 4;
const ATTEMPT: u64 = 2;

#[test]
fn an_ending_writer_waits_for_its_foreign_hold_to_let_go() {
    for (design, result) in [
        (false, Err(Error::Missing)),
        (true, Ok(Answer::Commit([7; 32]))),
        (false, Err(Error::Refused)),
        (true, Err(Error::Refused)),
    ] {
        let mut d = fixtures::connected_domain();
        outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
        outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
        outputs(&mut d, Event::CatchUp);
        let name = Name {
            forge: 1,
            repository: 2,
            what: What::Branch(Box::new([
                Box::from(b"temper".as_slice()),
                Box::from(b"1".as_slice()),
                Box::from(if design { b"g3".as_slice() } else { b"c3".as_slice() }),
            ])),
        };
        outputs(
            &mut d,
            Event::Restore {
                record: Stored::Hold(Hold {
                    name: name.clone(),
                    task: HOLDER,
                    writer: if design { None } else { Some(Writer::Run { task: WRITER, attempt: ATTEMPT }) },
                    design_writers: if design {
                        Box::new([DesignWriter { task: WRITER, attempt: ATTEMPT, rereading: false }])
                    } else {
                        Box::new([])
                    },
                    beside: None,
                    drift: None,
                }),
            },
        );
        outputs(&mut d, Event::Lost { task: WRITER, attempt: ATTEMPT });
        let (call, op) = fixtures::next_call(&mut d);
        assert!(matches!(op, Op::Read(Read::Branch { .. })));
        let release = outputs(
            &mut d,
            Event::Release { task: WRITER, root: HOLDER, ending: ReleaseEnding::Done, reason: None, entry: 20 },
        );
        assert!(
            !release.contains(&Request::Released { task: WRITER }),
            "writer cannot end before its read: {release:?}"
        );
        assert!(d.releases.contains_key(&WRITER), "the durable release survives a crash with its writer");
        let failed = result == Err(Error::Refused);
        let answered = fixtures::answer(&mut d, call, result);
        if failed {
            assert!(answered.contains(&Request::ReleaseFailed { task: WRITER, account: None }));
            assert!(!answered.contains(&Request::Released { task: WRITER }));
            assert!(d.releases.contains_key(&WRITER));
            continue;
        }
        assert!(
            answered.contains(&Request::ContinueRelease { task: WRITER }),
            "the read wakes its writer's release: {answered:?}"
        );
        let hold = d.hold(&name).expect("the parent still holds its branch");
        assert!(hold.writer.is_none() && hold.design_writers.is_empty());
        assert!(crate::reconcile::references_resolve(&d, &LIMITS, &Stored::Hold(hold.clone()), &[HOLDER]));
        let released = outputs(&mut d, Event::ContinueRelease { task: WRITER, entry: 21 });
        assert!(released.contains(&Request::Released { task: WRITER }));
        for request in &released {
            assert!(
                !matches!(request, Request::Save { record: Stored::Entry { .. }, .. }),
                "no deletion of the parent's branch"
            );
        }
    }
}
