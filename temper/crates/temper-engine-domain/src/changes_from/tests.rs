use super::{Entry, KEPT, Memory};

#[test]
fn provenance_evicts_the_least_recent_document_use_at_4096_changes() {
    let mut memory = Memory::new();
    for task in 1..=u64::from(KEPT) {
        memory.touch(task);
        let mut entry = Entry::new(1, task);
        entry.used = memory.clock;
        memory.rows.insert(task, entry).unwrap();
    }
    assert_eq!(memory.rows.len(), KEPT);
    memory.touch(1);
    let next = u64::from(KEPT) + 1;
    memory.touch(next);
    let mut entry = Entry::new(1, next);
    entry.used = memory.clock;
    memory.rows.insert(next, entry).unwrap();
    assert_eq!(memory.rows.len(), KEPT);
    assert!(memory.rows.contains_key(&1), "a document refreshed the oldest change");
    assert!(!memory.rows.contains_key(&2), "the next least-recently-used change left");
    assert!(memory.rows.contains_key(&next));
}
