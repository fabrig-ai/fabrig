//! Kept-row decisions (temper/docs/plan/playground-reset-r4/notes/contracts.md, D4).
use super::*;

fn rows() -> [Kept; 5] {
    [
        Kept { key: crate::kept::key(b"cancel", &[5, 7]), act: KeptAct::Cancel { project: 5, task: 7 } },
        Kept { key: crate::kept::key(b"project", &[5]), act: KeptAct::RemoveProject { project: 5 } },
        Kept { key: crate::kept::key(b"account", &[3]), act: KeptAct::RemoveAccount { account: 3 } },
        Kept { key: crate::kept::key(b"box", &[1]), act: KeptAct::BoxRemove { forge: 1 } },
        Kept { key: crate::kept::key(b"forge", &[1]), act: KeptAct::RemoveForge { forge: 1 } },
    ]
}

#[test]
fn kept_rows_are_saved_and_sent_but_remove_forge() {
    let mut d = domain();
    let mut out = Queue::uncapped();
    crate::kept::keep(&mut d, &rows(), &mut out);
    for row in rows() {
        assert_eq!(out.pop(), Some(Request::Save { read_afresh: None, release: None, record: Stored::Kept(row) }));
        assert_eq!(d.kept.get(&row.key), Some(&row.act));
        if let KeptAct::RemoveForge { .. } = row.act {
            continue;
        }
        assert_eq!(out.pop(), Some(Request::SendKept(row)));
    }
    assert!(out.is_empty());
}

#[test]
fn every_row_is_sent_again_at_a_start() {
    let mut d = domain();
    for row in rows() {
        assert!(outputs(&mut d, Event::Restore { record: Stored::Kept(row) }).is_empty());
    }
    let first = outputs(&mut d, Event::SendKept);
    assert_eq!(first.len(), 4);
    for row in &rows()[..4] {
        assert!(first.contains(&Request::SendKept(*row)));
    }
    assert_eq!(outputs(&mut d, Event::SendKept), first);
}

#[test]
fn a_kept_answer_erases_its_row_and_a_kept_one_stays() {
    let mut d = domain();
    let row = rows()[0];
    outputs(&mut d, Event::Restore { record: Stored::Kept(row) });
    assert!(outputs(&mut d, Event::KeptAnswered { key: row.key, keep: true }).is_empty());
    assert_eq!(d.kept.get(&row.key), Some(&row.act));
    assert_eq!(
        outputs(&mut d, Event::KeptAnswered { key: row.key, keep: false }).as_ref(),
        &[Request::Erase { key: Key::Kept(row.key) }]
    );
    assert!(d.kept.is_empty());
    assert!(outputs(&mut d, Event::KeptAnswered { key: row.key, keep: false }).is_empty());
}

#[test]
fn a_failed_box_remove_stays_kept() {
    let mut d = domain();
    let mut out = Queue::uncapped();
    crate::kept::keep(&mut d, &rows()[3..], &mut out);
    assert!(outputs(&mut d, Event::BoxRemoved { forge: 1, removed: false }).is_empty());
    assert_eq!(outputs(&mut d, Event::SendKept).as_ref(), &[Request::SendKept(rows()[3])]);
    assert!(outputs(&mut d, Event::BoxRemoved { forge: 2, removed: true }).is_empty());
    assert_eq!(
        outputs(&mut d, Event::BoxRemoved { forge: 1, removed: true }).as_ref(),
        &[Request::Erase { key: Key::Kept(rows()[3].key) }]
    );
    assert_eq!(d.kept.len(), 1);
    assert!(d.kept.contains_key(&rows()[4].key));
}

#[test]
fn erased_for_tells_when_a_forges_rows_are_gone() {
    let mut d = domain();
    let mut out = Queue::uncapped();
    let mut change = fixtures::opening_row();
    change.task = 7;
    outputs(&mut d, Event::Restore { record: Stored::Change(change) });
    crate::kept::keep(&mut d, &rows(), &mut out);
    assert!(!crate::kept::erased_for(&d, 1));
    assert!(crate::kept::erased_for(&d, 2));
    crate::kept::erase(&mut d, &env(), rows()[3].key, &mut out);
    assert!(!crate::kept::erased_for(&d, 1), "cancel still names the forge's change");
    crate::kept::erase(&mut d, &env(), rows()[0].key, &mut out);
    assert!(crate::kept::erased_for(&d, 1), "reset and project/account rows do not hold removal");
    crate::kept::erase(&mut d, &env(), rows()[4].key, &mut out);
    assert!(!d.kept.contains_key(&rows()[4].key));
}

#[test]
fn the_same_act_kept_twice_is_one_row() {
    let mut d = domain();
    let row = rows()[0];
    let mut out = Queue::uncapped();
    crate::kept::keep(&mut d, &[row, row], &mut out);
    assert_eq!(d.kept.len(), 1);
    assert_eq!(out.len(), 2, "one save and one send");
    assert_eq!(row.key, crate::kept::key(b"cancel", &[5, 7]));
    assert_ne!(row.key, crate::kept::key(b"cancel", &[7, 5]));
    assert_ne!(row.key, crate::kept::key(b"project", &[5, 7]));
    crate::kept::keep(&mut d, &[row], &mut out);
    assert_eq!(out.len(), 2);
}
