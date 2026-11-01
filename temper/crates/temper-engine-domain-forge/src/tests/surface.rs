//! Connector surface stubs (temper/docs/design/draft/playground-reset.md, 4).
use super::*;

#[test]
fn release_answers_project_released_at_once() {
    let mut d = domain();
    let request = Token::new(71);
    assert_eq!(
        outputs(&mut d, Event::ReleaseProject { request, project: 5 }).as_ref(),
        &[Request::ProjectReleased { request, project: 5 }],
    );
}

#[test]
fn kept_events_change_nothing_yet() {
    let mut d = domain();
    let key = [7; 16];
    let act = KeptAct::BoxRemove { forge: 1 };
    assert!(outputs(&mut d, Event::Restore { record: Stored::Kept(Kept { key, act }) }).is_empty());
    for event in [
        Event::SendKept,
        Event::KeptAnswered { key, keep: true },
        Event::KeptAnswered { key, keep: false },
        Event::BoxRemoved { forge: 1, removed: false },
        Event::BoxRemoved { forge: 1, removed: true },
    ] {
        assert!(outputs(&mut d, event).is_empty());
        assert_eq!(d.kept.len(), 1);
        assert_eq!(d.kept.get(&key), Some(&act));
    }
}
