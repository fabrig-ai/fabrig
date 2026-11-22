//! Restart progress is a per-forge document, not the room's readiness.
use super::*;
use skein_forge::api::{Answer, Op, Read, Write};
use skein_forge_client::{Attempt, RecoveryClock};

fn restored() -> Domain {
    let mut d = fixtures::connected_domain();
    let second = configured().get(1).unwrap().clone();
    d.configured.insert(second.number, second).unwrap();
    outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
    outputs(
        &mut d,
        Event::Restore {
            record: Stored::Hold(Hold {
                name: goal_branch::name(b"temper/7/g7"),
                task: 7,
                writer: None,
                beside: None,
                design_writers: Box::new([]),
                drift: None,
            }),
        },
    );
    let mut entry = status(1, 7);
    entry.effect.write = Write::Post {
        number: 9,
        key: Box::from(b"\x01\x00\x04testcomment".as_slice()),
        body: b"saved comment".as_slice().into(),
    };
    entry.start = Some(skein_forge_client::Position { at: Time::ZERO, comment: 0, review: 0, review_page: 1 });
    entry.attempt = Some(Attempt {
        sent: env().now,
        deadline: env().now.saturating_add(CLIENT.lifetime),
        wall: env().wall,
        expires: Wall::from_nanos(env().wall.as_nanos().checked_add(CLIENT.lifetime.as_nanos()).unwrap()),
    });
    outputs(&mut d, Event::Restore { record: Stored::Entry { entry, held: false } });
    assert_eq!(d.catching_up(1), None, "restored records alone do not announce progress");
    outputs(&mut d, Event::Restored { clock: RecoveryClock::Monotonic });
    d
}

#[test]
fn catching_up_tracks_fresh_reads_and_uncertain_attempts_until_both_halves_end() {
    assert_eq!(domain().catching_up(1), None, "no restart");
    let mut d = restored();
    assert_eq!(d.catching_up(1).unwrap().uncertain, 1, "not yet handed to client");
    outputs(&mut d, Event::CatchUp);
    let state = d.catching_up(1).unwrap();
    assert_eq!(state.goal_branches, 1);
    assert_eq!(state.uncertain, 1);
    assert_eq!(
        d.catching_up(2),
        Some(CatchingUp {
            fresh: 0,
            gathers: 0,
            ci: 0,
            goal_files: 0,
            lost_writers: 0,
            goal_branches: 0,
            uncertain: 0,
        })
    );
    let (call, op) = fixtures::next_call(&mut d);
    assert_eq!(op, Op::Read(Read::Branch { branch: b"temper/7/g7".as_slice().into() }));
    fixtures::answer(&mut d, call, Ok(Answer::Commit(fixtures::HEAD)));
    assert_eq!(d.catching_up(1).unwrap().goal_branches, 0);
    assert_eq!(d.catching_up(1).unwrap().uncertain, 1);
    assert_eq!(d.catching_up(1).unwrap().uncertain, 1, "handoff does not count as a find");
    let (call, op) = fixtures::next_call(&mut d);
    assert_eq!(op, Op::Read(Read::Item { number: 9, after: 0, since: Time::ZERO }));
    fixtures::answer(&mut d, call, Ok(empty_item()));
    assert_eq!(d.catching_up(1), None);
    assert_eq!(d.catching_up(2), None);
}

#[test]
fn progress_and_its_end_mark_both_the_forges_and_project_changes_documents() {
    let mut d = restored();
    let began = outputs(&mut d, Event::CatchUp);
    assert_marks(&began);
    assert_eq!(d.forges().rows.first().unwrap().catching_up, d.catching_up(1));
    let Document::Changes(panel) = d
        .document(changes_subject(5), jig_domain_people::ApplicationScope::Project(5), 1, env().wall, &LIMITS)
        .unwrap()
    else {
        panic!("project changes");
    };
    assert_eq!(panel.catching_up, d.catching_up(1));
    let (call, _) = fixtures::next_call(&mut d);
    let changed = fixtures::answer(&mut d, call, Ok(Answer::Commit(fixtures::HEAD)));
    assert_marks(&changed);
    let (call, _) = fixtures::next_call(&mut d);
    let ended = fixtures::answer(&mut d, call, Ok(empty_item()));
    assert_eq!(d.catching_up(1), None);
    assert_marks(&ended);
}

fn empty_item() -> Answer {
    Answer::Item {
        item: skein_forge::api::Summary {
            number: 9,
            kind: skein_forge::api::Kind::Issue,
            state: skein_forge::api::State::Open,
            author: 7,
            key: None,
            labels: Box::new([]),
            title: Box::new([]),
            body: Box::new([]),
            updated: env().now,
        },
        comments: Box::new([]),
        more: false,
    }
}

fn assert_marks(requests: &[Request]) {
    assert!(requests.contains(&Request::MarkedForges), "{requests:?}");
    assert!(requests.contains(&Request::Marked { tasks: Box::new([changes_subject(5)]) }), "{requests:?}");
}
