//! History range boundaries through the public domain API.
#![cfg(test)]
use temper_engine_domain::{Key, Range};
use temper_engine_domain_forge::{HistoryRange as H, Key as F};

#[test]
fn history_ranges_admit_only_their_own_family_project_and_subject() {
    let change = Range::History(H::Change { project: 3, change: 9 });
    let numbers = Range::History(H::Numbers { project: 3, numbers: Box::new([4, 7]) });
    let cases = [
        (F::HistoryChange { project: 3, change: 9, number: 4 }, true, false),
        (F::HistoryChange { project: 3, change: 8, number: 4 }, false, false),
        (F::HistoryChange { project: 2, change: 9, number: 4 }, false, false),
        (F::HistoryEvent { project: 3, number: 4 }, false, true),
        (F::HistoryEvent { project: 3, number: 7 }, false, true),
        (F::HistoryEvent { project: 3, number: 5 }, false, false),
        (F::HistoryEvent { project: 2, number: 4 }, false, false),
        (F::HistoryLanding { project: 3, landing: 9, number: 4 }, false, false),
        (F::HistoryLast { project: 3, change: 9, number: 4 }, false, false),
    ];
    for (key, in_change, in_numbers) in cases {
        let key = Key::Forge(key);
        assert_eq!(change.contains(&key), in_change, "{key:?}");
        assert_eq!(numbers.contains(&key), in_numbers, "{key:?}");
        assert!(!Range::Forge.contains(&key));
    }
    for (range, yes, no) in [
        (
            H::Events { project: 3 },
            F::HistoryEvent { project: 3, number: 4 },
            F::HistoryEvent { project: 2, number: 4 },
        ),
        (
            H::Landing { project: 3, landing: 9 },
            F::HistoryLanding { project: 3, landing: 9, number: 4 },
            F::HistoryLanding { project: 3, landing: 8, number: 4 },
        ),
        (
            H::Last { project: 3 },
            F::HistoryLast { project: 3, number: 4, change: 9 },
            F::HistoryLast { project: 2, number: 4, change: 9 },
        ),
    ] {
        assert!(Range::History(range.clone()).contains(&Key::Forge(yes)));
        assert!(!Range::History(range).contains(&Key::Forge(no)));
    }
    assert!(!change.contains(&Key::Wires));
    let invalid = Range::History(H::Numbers { project: 3, numbers: Box::new([4; 33]) });
    assert!(!invalid.contains(&Key::Forge(F::HistoryEvent { project: 3, number: 4 })));
    assert!(Range::Forge.contains(&Key::Forge(F::HistoryProject { project: 3 })));
}
