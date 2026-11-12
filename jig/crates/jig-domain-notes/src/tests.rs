//! Unit tests exercise the boundary and the requests it emits.

use alloc::boxed::Box;

use skein_lib::{Env, List, Queue, Time, Token, Wall};

use crate::{
    Author, Change, Domain, Entry, Event, Limits, Line, New, Range, Recall, Record, Refusal, Request, Rows, Scope, step,
};

const LIMITS: Limits = Limits {
    scopes: 2,
    entries_per_scope: 1,
    party_entries: 1,
    party_bytes: 16,
    pattern_bytes: 16,
    description_bytes: 16,
    body_bytes: 32,
    references: 2,
    load_rows: 2,
    lines: 2,
    recalled: 2,
};

fn new(name: u64, description: &[u8]) -> New {
    New {
        name,
        scope: Scope::Project { project: 7 },
        description: Box::from(description),
        body: Box::from(&b"body"[..]),
        references: List::with_capacity(2),
        author: Author::Task { task: 9, attempt: 1 },
    }
}

fn rows(records: &[Record]) -> Rows {
    rows_with_capacity(records, 2)
}

fn rows_with_capacity(records: &[Record], capacity: u32) -> Rows {
    let mut list = List::with_capacity(capacity);
    for record in records {
        list.push(record.clone()).expect("test page fits");
    }
    Rows { records: list }
}

struct Harness {
    domain: Domain,
    env: Env<Limits>,
    out: Queue<Request>,
}

impl Harness {
    fn new() -> Harness {
        Harness::with_limits(LIMITS)
    }

    fn with_limits(limits: Limits) -> Harness {
        Harness {
            domain: Domain::new(&limits),
            env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits },
            out: Queue::uncapped(),
        }
    }

    fn step(&mut self, event: Event) -> Box<[Request]> {
        step(&mut self.domain, &self.env, event, &mut self.out);
        let mut got = List::with_capacity(self.out.len());
        while let Some(request) = self.out.pop() {
            got.push(request).expect("test output fits");
        }
        got.into_boxed()
    }
}

fn scopes(projects: &[u32], capacity: u32) -> List<Scope> {
    let mut scopes = List::with_capacity(capacity);
    for project in projects {
        scopes.push(Scope::Project { project: *project }).expect("test scopes fit");
    }
    scopes
}

fn line(project: u32, name: u64, description: &[u8]) -> Line {
    Line { scope: Scope::Project { project }, name, description: Box::from(description), revision: 1 }
}

fn entry(project: u32, name: u64, description: &[u8]) -> Entry {
    Entry {
        name,
        scope: Scope::Project { project },
        description: Box::from(description),
        body: Box::from(&b"body"[..]),
        references: List::with_capacity(2),
        author: Author::Task { task: 9, attempt: 1 },
        revision: 1,
        changed: Wall::from_nanos(100),
    }
}

#[test]
fn a_new_entry_is_saved_with_its_scope_line_in_one_decision() {
    let mut h = Harness::new();
    let asked = h.step(Event::Write { owner: Token::new(4), entry: new(20, b"warmup"), recalled: None });
    let [Request::Load { owner, range: Range::Entry { name: 20 } }] = &*asked else {
        panic!("entry lookup: {asked:?}");
    };
    let owner = *owner;
    let asked = h.step(Event::Loaded { owner, rows: rows(&[]), more: false });
    let [Request::Load { owner, range: Range::Lines { .. } }] = &*asked else {
        panic!("scope lookup: {asked:?}");
    };
    h.env.wall = Wall::from_nanos(900);
    let got = h.step(Event::Loaded { owner: *owner, rows: rows(&[]), more: false });
    let [
        Request::Save { record: Record::Entry(saved) },
        Request::Save { record: Record::Line(_) },
        Request::Written {
            owner,
            name: 20,
            revision: 1,
            change: crate::LineChange { lines: 1, added: 1, removed: 0, first: 1 },
        },
    ] = &*got
    else {
        panic!("new entry writes: {got:?}");
    };
    assert_eq!(*owner, Token::new(4));
    assert_eq!(saved.changed, Wall::from_nanos(900), "time belongs to the writing decision after store loads");
    assert!(!h.domain.busy());
}

#[test]
fn a_revision_must_name_the_entry_the_writer_recalled() {
    let mut h = Harness::new();
    let asked = h.step(Event::Write { owner: Token::new(5), entry: new(20, b"revised"), recalled: Some(1) });
    let [Request::Load { owner, .. }] = &*asked else {
        panic!("entry lookup");
    };
    let old = Entry {
        name: 20,
        scope: Scope::Project { project: 7 },
        description: Box::from(&b"old"[..]),
        body: Box::from(&b"body"[..]),
        references: List::with_capacity(2),
        author: Author::Party { party: 2 },
        revision: 2,
        changed: Wall::from_nanos(100),
    };
    let got = h.step(Event::Loaded { owner: *owner, rows: rows(&[Record::Entry(old)]), more: false });
    assert_eq!(&*got, &[Request::Refused { owner: Token::new(5), why: Refusal::Moved { revision: 2 } }]);
}

#[test]
fn a_party_deletes_only_the_revision_it_saw() {
    let mut h = Harness::new();
    let asked = h.step(Event::Edit {
        owner: Token::new(6),
        party: 2,
        name: 20,
        scope: Scope::Project { project: 7 },
        change: Change::Delete { recalled: 2 },
    });
    let [Request::Load { owner, .. }] = &*asked else {
        panic!("entry lookup");
    };
    let old = Entry {
        name: 20,
        scope: Scope::Project { project: 7 },
        description: Box::from(&b"old"[..]),
        body: Box::from(&b"body"[..]),
        references: List::with_capacity(2),
        author: Author::Task { task: 1, attempt: 1 },
        revision: 2,
        changed: Wall::from_nanos(100),
    };
    let got = h.step(Event::Loaded { owner: *owner, rows: rows(&[Record::Entry(old)]), more: false });
    let [Request::Erase { .. }, Request::Erase { .. }, Request::Deleted { owner, name: 20 }] = &*got else {
        panic!("delete writes: {got:?}");
    };
    assert_eq!(*owner, Token::new(6));
}

#[test]
fn a_full_scope_refuses_a_new_entry() {
    for (scope, entries_per_scope) in [(Scope::Project { project: 7 }, 1), (Scope::Party { party: 2 }, 2)] {
        let mut limits = LIMITS;
        limits.entries_per_scope = entries_per_scope;
        let mut h = Harness::with_limits(limits);
        let mut entry = new(21, b"new");
        entry.scope = scope.clone();
        entry.body = Box::new([]);
        let asked = h.step(Event::Write { owner: Token::new(7), entry, recalled: None });
        let [Request::Load { owner, .. }] = &*asked else { panic!("entry lookup") };
        let asked = h.step(Event::Loaded { owner: *owner, rows: rows(&[]), more: false });
        let [Request::Load { owner, range: Range::Lines { .. } }] = &*asked else { panic!("scope lookup") };
        let line = Line { scope, name: 20, description: Box::from(&b"old"[..]), revision: 1 };
        let got = h.step(Event::Loaded { owner: *owner, rows: rows(&[Record::Line(line)]), more: false });
        assert_eq!(&*got, &[Request::Refused { owner: Token::new(7), why: Refusal::Full }]);
    }
}

#[test]
fn an_index_pages_scope_lines_and_counts_what_does_not_fit() {
    let mut limits = LIMITS;
    limits.entries_per_scope = 2;
    limits.load_rows = 1;
    limits.lines = 1;
    let mut h = Harness::with_limits(limits);
    let asked = h.step(Event::Index { owner: Token::new(30), scopes: scopes(&[7], 1), most: 1 });
    let [Request::Load { owner, range: Range::Lines { scope: Scope::Project { project: 7 }, after: None } }] = &*asked
    else {
        panic!("first index page: {asked:?}");
    };
    let asked = h.step(Event::Loaded {
        owner: *owner,
        rows: rows_with_capacity(&[Record::Line(line(7, 20, b"first"))], 1),
        more: true,
    });
    let [Request::Load { owner, range: Range::Lines { scope: Scope::Project { project: 7 }, after: Some(20) } }] =
        &*asked
    else {
        panic!("second index page: {asked:?}");
    };
    let got = h.step(Event::Loaded {
        owner: *owner,
        rows: rows_with_capacity(&[Record::Line(line(7, 21, b"second"))], 1),
        more: false,
    });
    let [Request::Indexed { owner, lines, more: 1 }] = &*got else {
        panic!("index answer: {got:?}");
    };
    assert_eq!(*owner, Token::new(30));
    assert_eq!(lines.get(0).expect("one line").name, 20);
    assert_eq!(h.domain.cached_scopes(), 1);
}

#[test]
fn a_description_search_recalls_one_page_of_bodies_at_a_time() {
    let mut limits = LIMITS;
    limits.entries_per_scope = 2;
    limits.recalled = 1;
    let mut h = Harness::with_limits(limits);
    let asked = h.step(Event::Index { owner: Token::new(30), scopes: scopes(&[7], 1), most: 2 });
    let [Request::Load { owner, .. }] = &*asked else {
        panic!("index load");
    };
    let _indexed = h.step(Event::Loaded {
        owner: *owner,
        rows: rows(&[Record::Line(line(7, 20, b"warm first")), Record::Line(line(7, 21, b"warm second"))]),
        more: false,
    });
    let by = Recall::Search { scopes: scopes(&[7], 1), query: Box::from(&b"warm"[..]) };
    let asked = h.step(Event::Recall { owner: Token::new(31), by, page: 0 });
    let [Request::Load { owner, range: Range::Entry { name: 20 } }] = &*asked else {
        panic!("first recall: {asked:?}");
    };
    let got =
        h.step(Event::Loaded { owner: *owner, rows: rows(&[Record::Entry(entry(7, 20, b"warm first"))]), more: false });
    let [Request::Recalled { owner, entries, more: true }] = &*got else {
        panic!("first page: {got:?}");
    };
    assert_eq!(*owner, Token::new(31));
    assert_eq!(entries.get(0).expect("one entry").name, 20);
    let by = Recall::Search { scopes: scopes(&[7], 1), query: Box::from(&b"warm"[..]) };
    let asked = h.step(Event::Recall { owner: Token::new(32), by, page: 1 });
    let [Request::Load { owner, range: Range::Entry { name: 21 } }] = &*asked else {
        panic!("second recall: {asked:?}");
    };
    let got = h.step(Event::Loaded {
        owner: *owner,
        rows: rows(&[Record::Entry(entry(7, 21, b"warm second"))]),
        more: false,
    });
    let [Request::Recalled { owner, entries, more: false }] = &*got else {
        panic!("second page: {got:?}");
    };
    assert_eq!(*owner, Token::new(32));
    assert_eq!(entries.get(0).expect("one entry").name, 21);
}

#[test]
fn a_new_scope_evicts_the_least_recently_used_index() {
    let mut h = Harness::new();
    for project in [7, 8] {
        let asked =
            h.step(Event::Index { owner: Token::new(u64::from(project)), scopes: scopes(&[project], 1), most: 1 });
        let [Request::Load { owner, .. }] = &*asked else {
            panic!("scope load");
        };
        let _answer = h.step(Event::Loaded { owner: *owner, rows: rows(&[]), more: false });
    }
    let got = h.step(Event::Index { owner: Token::new(70), scopes: scopes(&[7], 1), most: 1 });
    let [Request::Indexed { .. }] = &*got else {
        panic!("cached index");
    };
    let asked = h.step(Event::Index { owner: Token::new(9), scopes: scopes(&[9], 1), most: 1 });
    let [Request::Load { owner, .. }] = &*asked else {
        panic!("new scope load");
    };
    let _answer = h.step(Event::Loaded { owner: *owner, rows: rows(&[]), more: false });
    assert!(h.domain.holds(&Scope::Project { project: 7 }));
    assert!(!h.domain.holds(&Scope::Project { project: 8 }));
    assert!(h.domain.holds(&Scope::Project { project: 9 }));
}

#[test]
fn a_write_invalidates_the_cached_scope_index() {
    let mut h = Harness::new();
    let asked = h.step(Event::Index { owner: Token::new(30), scopes: scopes(&[7], 1), most: 1 });
    let [Request::Load { owner, .. }] = &*asked else {
        panic!("scope load");
    };
    let _answer =
        h.step(Event::Loaded { owner: *owner, rows: rows(&[Record::Line(line(7, 20, b"old"))]), more: false });
    assert!(h.domain.holds(&Scope::Project { project: 7 }));
    let asked = h.step(Event::Write { owner: Token::new(31), entry: new(20, b"new"), recalled: Some(1) });
    let [Request::Load { owner, range: Range::Entry { name: 20 } }] = &*asked else {
        panic!("entry load");
    };
    let got = h.step(Event::Loaded { owner: *owner, rows: rows(&[Record::Entry(entry(7, 20, b"old"))]), more: false });
    let [Request::Save { .. }, Request::Save { .. }, Request::Written { revision: 2, .. }] = &*got else {
        panic!("entry revision: {got:?}");
    };
    assert!(!h.domain.holds(&Scope::Project { project: 7 }));
}

#[test]
fn a_named_recall_loads_its_entry_without_caching_the_body() {
    let mut h = Harness::new();
    let asked = h.step(Event::Recall { owner: Token::new(50), by: Recall::Name { name: 20 }, page: 0 });
    let [Request::Load { owner, range: Range::Entry { name: 20 } }] = &*asked else {
        panic!("entry lookup: {asked:?}");
    };
    let got = h.step(Event::Loaded { owner: *owner, rows: rows(&[Record::Entry(entry(7, 20, b"warm"))]), more: false });
    let [Request::Recalled { owner, entries, more: false }] = &*got else {
        panic!("entry answer: {got:?}");
    };
    assert_eq!(*owner, Token::new(50));
    assert_eq!(entries.get(0).expect("one entry").name, 20);
    assert_eq!(h.domain.cached_scopes(), 0);
}

#[test]
fn a_project_drop_retries_busy_loads_and_answers_only_after_an_empty_page() {
    let mut h = Harness::new();
    let caller = Token::new(71);
    let first = h.step(Event::DropProject { owner: caller, project: 7 });
    let [Request::Load { owner, range: Range::Project { project: 7, after: None } }] = &*first else {
        panic!("project load: {first:?}");
    };
    let first_owner = *owner;
    assert!(h.domain.busy());
    assert_eq!(
        &*h.step(Event::DropProject { owner: Token::new(72), project: 8 }),
        &[Request::Refused { owner: Token::new(72), why: Refusal::Busy }]
    );
    assert_eq!(
        &*h.step(Event::Write { owner: Token::new(73), entry: new(21, b"new"), recalled: None }),
        &[Request::Refused { owner: Token::new(73), why: Refusal::Busy }]
    );
    let retry = h.step(Event::LoadFailed { owner: first_owner });
    let [Request::Load { owner, range: Range::Project { project: 7, after: None } }] = &*retry else {
        panic!("retry project load: {retry:?}");
    };
    assert_ne!(*owner, first_owner, "retry has a fresh load token");
    assert!(h.step(Event::LoadFailed { owner: first_owner }).is_empty(), "stale failure is ignored");
    let next = h.step(Event::Loaded { owner: *owner, rows: rows(&[Record::Line(line(7, 20, b"note"))]), more: false });
    let [
        Request::Erase { key },
        Request::Erase { key: entry },
        Request::Load { owner, range: Range::Project { project: 7, after: Some(after) } },
    ] = &*next
    else {
        panic!("erase and next page: {next:?}");
    };
    assert_eq!(key, &crate::Key::Line { scope: Scope::Project { project: 7 }, name: 20 });
    assert_eq!(entry, &crate::Key::Entry { name: 20 });
    assert_eq!(key, after);
    let retry = h.step(Event::LoadFailed { owner: *owner });
    let [Request::Load { owner, range: Range::Project { project: 7, after: Some(cursor) } }] = &*retry else {
        panic!("retry continuation: {retry:?}");
    };
    assert_eq!(cursor, key, "busy continuation keeps its cursor");
    assert_eq!(
        &*h.step(Event::Loaded { owner: *owner, rows: rows(&[]), more: false }),
        &[Request::ProjectDropped { owner: caller }]
    );
    assert!(!h.domain.busy());
}

#[test]
fn a_first_write_counts_every_line() {
    assert_eq!(crate::line_change(None, b"a\nb\nc"), crate::LineChange { lines: 3, added: 3, removed: 0, first: 1 });
}

#[test]
fn an_empty_body_has_no_lines() {
    assert_eq!(crate::line_change(None, b""), crate::LineChange { lines: 0, added: 0, removed: 0, first: 1 });
    assert_eq!(crate::line_change(Some(b"a\nb"), b""), crate::LineChange { lines: 0, added: 0, removed: 2, first: 1 });
}

#[test]
fn a_trailing_newline_adds_no_line() {
    assert_eq!(crate::line_change(None, b"a\nb\n"), crate::LineChange { lines: 2, added: 2, removed: 0, first: 1 });
    assert_eq!(crate::line_change(None, b"\n"), crate::LineChange { lines: 1, added: 1, removed: 0, first: 1 });
    assert_eq!(crate::line_change(Some(b"a"), b"a\n"), crate::LineChange { lines: 1, added: 0, removed: 0, first: 1 });
}

#[test]
fn a_middle_change_trims_both_ends() {
    assert_eq!(
        crate::line_change(Some(b"a\nx\ny\nz"), b"a\nb\nz"),
        crate::LineChange { lines: 3, added: 1, removed: 2, first: 2 }
    );
}

#[test]
fn an_append_starts_after_the_old_body() {
    assert_eq!(
        crate::line_change(Some(b"a\nb\n"), b"a\nb\nc\nd"),
        crate::LineChange { lines: 4, added: 2, removed: 0, first: 3 }
    );
}

#[test]
fn a_removal_at_the_end_points_at_the_last_line() {
    assert_eq!(
        crate::line_change(Some(b"a\nb\nc"), b"a\nb"),
        crate::LineChange { lines: 2, added: 0, removed: 1, first: 2 }
    );
}

#[test]
fn an_unchanged_body_changes_nothing() {
    assert_eq!(
        crate::line_change(Some(b"a\nb"), b"a\nb"),
        crate::LineChange { lines: 2, added: 0, removed: 0, first: 2 }
    );
    assert_eq!(crate::line_change(Some(b""), b""), crate::LineChange { lines: 0, added: 0, removed: 0, first: 1 });
}

#[test]
fn prefix_and_suffix_never_overlap() {
    assert_eq!(
        crate::line_change(Some(b"a\nX\na"), b"a\nX\nX\na"),
        crate::LineChange { lines: 4, added: 1, removed: 0, first: 3 }
    );
}

fn generated_body(mut value: u32, trailing: bool) -> Box<[u8]> {
    let mut writer = List::with_capacity(8);
    while value > 1 {
        if value & 1 == 0 {
            writer.push(b'a').expect("four generated lines fit");
        }
        value >>= 1_u32;
        if value > 1 || trailing {
            writer.push(b'\n').expect("four generated lines fit");
        }
    }
    writer.into_boxed()
}

fn logical_lines(body: &[u8]) -> List<&[u8]> {
    let mut lines = List::with_capacity(8);
    let mut start = 0;
    for (at, byte) in body.iter().enumerate() {
        if *byte == b'\n' {
            lines.push(&body[start..at]).expect("generated lines fit");
            start = at.checked_add(1).expect("delimiter within generated body");
        }
    }
    if start < body.len() {
        lines.push(&body[start..]).expect("generated lines fit");
    }
    lines
}

#[test]
fn generated_changes_rebuild_the_new_body() {
    for old_id in 1..32 {
        for new_id in 1..32 {
            for trailing in [false, true] {
                let old = generated_body(old_id, trailing);
                let new = generated_body(new_id, !trailing);
                let old_lines = logical_lines(&old);
                let new_lines = logical_lines(&new);
                let change = crate::line_change(Some(&old), &new);
                assert_eq!(change.lines, new_lines.len());
                let added = usize::try_from(change.added).unwrap();
                let removed = usize::try_from(change.removed).unwrap();
                let old_lines = old_lines.as_slice();
                let new_lines = new_lines.as_slice();
                let mut prefix = 0;
                for (old, new) in old_lines.iter().zip(new_lines) {
                    if old != new {
                        break;
                    }
                    prefix += 1;
                }
                assert_eq!(usize::try_from(change.first).unwrap(), (prefix + 1).min(new_lines.len().max(1)));
                let suffix = new_lines.len() - prefix - added;
                assert_eq!(old_lines.len() - prefix - removed, suffix);
                let mut rebuilt = List::with_capacity(8);
                for line in &old_lines[..prefix] {
                    rebuilt.push(*line).expect("generated prefix fits");
                }
                for line in &new_lines[prefix..prefix + added] {
                    rebuilt.push(*line).expect("generated replacement fits");
                }
                for line in &old_lines[prefix + removed..] {
                    rebuilt.push(*line).expect("generated suffix fits");
                }
                assert_eq!(rebuilt.as_slice(), new_lines, "old={old:?}, new={new:?}, change={change:?}");
            }
        }
    }
}

#[test]
fn revisions_and_corrections_answer_from_the_loaded_body() {
    for correction in [false, true] {
        let mut h = Harness::new();
        let mut updated = new(20, b"updated");
        updated.body = b"body\nnew".as_slice().into();
        let event = if correction {
            Event::Edit {
                owner: Token::new(41),
                party: 2,
                name: 20,
                scope: updated.scope,
                change: Change::Correct {
                    description: updated.description,
                    body: updated.body,
                    references: updated.references,
                    recalled: 1,
                },
            }
        } else {
            Event::Write { owner: Token::new(41), entry: updated, recalled: Some(1) }
        };
        let asked = h.step(event);
        let [Request::Load { owner, .. }] = &*asked else {
            panic!("old entry lookup");
        };
        let got =
            h.step(Event::Loaded { owner: *owner, rows: rows(&[Record::Entry(entry(7, 20, b"old"))]), more: false });
        let [Request::Save { .. }, Request::Save { .. }, Request::Written { owner, name: 20, revision: 2, change }] =
            &*got
        else {
            panic!("revision writes: {got:?}");
        };
        assert_eq!(*owner, Token::new(41));
        assert_eq!(*change, crate::LineChange { lines: 2, added: 1, removed: 0, first: 2 });
    }
}

fn spliced(old: &[u8], first: u32, remove: u32, text: &[u8]) -> Box<[u8]> {
    crate::splice(old, crate::LineRange { first, remove, join: false }, text, 64).expect("valid range within bound")
}

#[test]
fn a_range_replaces_its_lines() {
    assert_eq!(&*spliced(b"a\nb\n", 2, 1, b"x\n"), b"a\nx\n");
}
#[test]
fn a_range_inserts_when_it_removes_nothing() {
    assert_eq!(&*spliced(b"a\nb\n", 2, 0, b"x"), b"a\nx\nb\n");
}
#[test]
fn an_append_after_a_final_newline_keeps_the_texts_ending() {
    assert_eq!(&*spliced(b"a\nb\n", 3, 0, b"c"), b"a\nb\nc");
    assert_eq!(&*spliced(b"a\n", 7, 0, b"b"), b"a\nb");
}
#[test]
fn an_append_to_a_body_with_no_final_newline_joins_no_lines() {
    assert_eq!(&*spliced(b"a\nb", 3, 0, b"c\n"), b"a\nb\nc\n");
}
#[test]
fn text_with_no_final_newline_joins_no_lines_before_what_follows() {
    assert_eq!(&*spliced(b"a\nb\nc", 2, 1, b"x"), b"a\nx\nc");
}
#[test]
fn removing_the_last_lines_keeps_the_rest() {
    assert_eq!(&*spliced(b"a\nb\nc", 3, 1, b""), b"a\nb\n");
}
#[test]
fn a_blank_line_counts_as_a_line() {
    assert_eq!(&*spliced(b"a\n\nb", 2, 1, b"x"), b"a\nx\nb");
}
#[test]
fn an_empty_body_takes_an_append_and_nothing_else() {
    for first in [1, 3] {
        assert_eq!(&*spliced(b"", first, 0, b"x"), b"x");
    }
    for (first, remove) in [(0, 0), (0, 1), (1, 1), (3, 1)] {
        let range = crate::LineRange { first, remove, join: false };
        assert_eq!(crate::splice(b"", range, b"x", 64), Err(Refusal::PastEnd { lines: 0 }));
    }
}
#[test]
fn first_zero_and_removals_past_the_end_say_the_lines() {
    for (first, remove) in [(0, 0), (0, 1), (2, 1), (1, 2)] {
        assert_eq!(
            crate::splice(b"a\n", crate::LineRange { first, remove, join: false }, b"x", 64),
            Err(Refusal::PastEnd { lines: 1 })
        );
    }
}
#[test]
fn a_range_on_u32_max_does_not_overflow() {
    assert_eq!(&*spliced(b"a", u32::MAX, 0, b"x"), b"a\nx");
    for (first, remove) in [(0, u32::MAX), (1, u32::MAX), (u32::MAX, u32::MAX)] {
        assert_eq!(
            crate::splice(b"a", crate::LineRange { first, remove, join: false }, b"x", 64),
            Err(Refusal::PastEnd { lines: 1 })
        );
    }
}
#[test]
fn a_result_at_the_bound_fits_and_one_past_says_its_bytes() {
    let range = crate::LineRange { first: 2, remove: 0, join: false };
    assert_eq!(&*crate::splice(b"a", range, b"b", 3).unwrap(), b"a\nb");
    assert_eq!(crate::splice(b"a", range, b"bc", 3), Err(Refusal::PastBound { bytes: 4, bound: 3 }));
}

#[test]
fn a_range_matches_a_list_of_lines() {
    for old_id in 1..16 {
        for text_id in 1..8 {
            for trailing in [false, true] {
                let old = generated_body(old_id, trailing);
                let text = generated_body(text_id, !trailing);
                let mut old_lines = List::with_capacity(8);
                for line in old.split_inclusive(|byte| *byte == b'\n') {
                    old_lines.push(line).expect("generated lines fit");
                }
                let old_lines = old_lines.as_slice();
                for first in 1..=old_lines.len() + 1 {
                    for remove in 0..=old_lines.len() + 1 - first {
                        let mut parts = List::with_capacity(9);
                        for line in &old_lines[..first - 1] {
                            parts.push(*line).expect("generated prefix fits");
                        }
                        parts.push(text.as_ref()).expect("generated text fits");
                        for line in &old_lines[first - 1 + remove..] {
                            parts.push(*line).expect("generated suffix fits");
                        }
                        let mut model = List::with_capacity(32);
                        for part in &parts {
                            if !model.is_empty() && !model.as_slice().ends_with(b"\n") && !part.is_empty() {
                                model.push(b'\n').expect("generated separator fits");
                            }
                            for byte in *part {
                                model.push(*byte).expect("generated bytes fit");
                            }
                        }
                        let model = model.into_boxed();
                        let result =
                            spliced(&old, u32::try_from(first).unwrap(), u32::try_from(remove).unwrap(), &text);
                        assert_eq!(&*result, &*model, "old={old:?} first={first} remove={remove} text={text:?}");
                        let change = crate::line_change(Some(&old), &result);
                        let old_lines = logical_lines(&old);
                        let new_lines = logical_lines(&model);
                        let mut prefix = 0;
                        for (old, new) in old_lines.as_slice().iter().zip(new_lines.as_slice()) {
                            if old != new {
                                break;
                            }
                            prefix += 1;
                        }
                        let mut suffix = 0;
                        for (old, new) in
                            old_lines.as_slice()[prefix..].iter().rev().zip(new_lines.as_slice()[prefix..].iter().rev())
                        {
                            if old != new {
                                break;
                            }
                            suffix += 1;
                        }
                        assert_eq!(
                            change,
                            crate::LineChange {
                                lines: new_lines.len(),
                                added: u32::try_from(new_lines.as_slice().len() - prefix - suffix).unwrap(),
                                removed: u32::try_from(old_lines.as_slice().len() - prefix - suffix).unwrap(),
                                first: u32::try_from((prefix + 1).min(new_lines.as_slice().len().max(1))).unwrap(),
                            }
                        );
                    }
                }
            }
        }
    }
}

fn range_write(
    h: &mut Harness,
    old: Option<Entry>,
    updated: New,
    recalled: u32,
    first: u32,
    remove: u32,
) -> Box<[Request]> {
    range_write_with(h, old, updated, recalled, crate::LineRange { first, remove, join: false })
}

fn range_write_with(
    h: &mut Harness,
    old: Option<Entry>,
    updated: New,
    recalled: u32,
    range: crate::LineRange,
) -> Box<[Request]> {
    let asked = h.step(Event::WriteLines { owner: Token::new(51), entry: updated, recalled, range });
    let [Request::Load { owner, range: Range::Entry { name: 20 } }] = &*asked else {
        panic!("range entry lookup: {asked:?}")
    };
    let rows = match old {
        Some(old) => rows(&[Record::Entry(old)]),
        None => rows(&[]),
    };
    h.step(Event::Loaded { owner: *owner, rows, more: false })
}

fn range_new(body: &[u8]) -> New {
    let mut updated = new(20, b"range");
    updated.body = body.into();
    updated
}

#[test]
fn a_range_write_saves_the_whole_body_at_the_next_revision() {
    for (join, expected) in [(false, b"body\nnext".as_slice()), (true, b"bodynext".as_slice())] {
        let mut h = Harness::new();
        h.env.wall = Wall::from_nanos(900);
        let got = range_write_with(
            &mut h,
            Some(entry(7, 20, b"old")),
            range_new(b"next"),
            1,
            crate::LineRange { first: 2, remove: 0, join },
        );
        let [
            Request::Save { record: Record::Entry(saved) },
            Request::Save { record: Record::Line(line) },
            Request::Written { revision: 2, .. },
        ] = &*got
        else {
            panic!("range saves: {got:?}")
        };
        assert_eq!(&*saved.body, expected);
        assert_eq!(saved.revision, 2);
        assert_eq!(saved.changed, h.env.wall);
        assert_eq!(saved.author, Author::Task { task: 9, attempt: 1 });
        assert_eq!(line.revision, saved.revision);
    }
}
#[test]
fn a_range_answers_as_line_change_does() {
    let mut h = Harness::new();
    let mut old = entry(7, 20, b"old");
    old.body = b"a\nb\nc".as_slice().into();
    let got = range_write(&mut h, Some(old), range_new(b"b\nx"), 1, 2, 1);
    let [Request::Save { .. }, Request::Save { .. }, Request::Written { change, .. }] = &*got else {
        panic!("answer: {got:?}")
    };
    assert_eq!(*change, crate::line_change(Some(b"a\nb\nc"), b"a\nb\nx\nc"));
    assert_eq!(change.first, 3, "common lines precede the differing span");
}
#[test]
fn an_overshooting_append_saves_and_answers_the_changed_line() {
    for (body, expected, expected_change) in [
        (b"".as_slice(), b"x".as_slice(), crate::LineChange { lines: 1, added: 1, removed: 0, first: 1 }),
        (b"old".as_slice(), b"old\nx".as_slice(), crate::LineChange { lines: 2, added: 1, removed: 0, first: 2 }),
    ] {
        let mut h = Harness::new();
        let mut old = entry(7, 20, b"old");
        old.body = body.into();
        let got = range_write(&mut h, Some(old), range_new(b"x"), 1, 5, 0);
        let [
            Request::Save { record: Record::Entry(saved) },
            Request::Save { record: Record::Line(_) },
            Request::Written { revision: 2, change, .. },
        ] = &*got
        else {
            panic!("append saves and answer: {got:?}")
        };
        assert_eq!(&*saved.body, expected);
        assert_eq!(*change, expected_change);
        assert_eq!(*change, crate::line_change(Some(body), expected));
    }
}
#[test]
fn a_stale_range_says_the_revision() {
    let mut old = entry(7, 20, b"old");
    old.revision = 7;
    let got = range_write(&mut Harness::new(), Some(old), range_new(b"x"), 1, u32::MAX, 0);
    assert_eq!(&*got, &[Request::Refused { owner: Token::new(51), why: Refusal::Moved { revision: 7 } }]);
}
#[test]
fn a_range_on_another_scope_says_the_revision() {
    let got = range_write(&mut Harness::new(), Some(entry(8, 20, b"old")), range_new(b"x"), 1, 1, 0);
    assert_eq!(&*got, &[Request::Refused { owner: Token::new(51), why: Refusal::Moved { revision: 1 } }]);
}
#[test]
fn a_range_on_no_entry_is_missing() {
    let got = range_write(&mut Harness::new(), None, range_new(b"x"), 1, 1, 0);
    assert_eq!(&*got, &[Request::Refused { owner: Token::new(51), why: Refusal::Missing }]);
}
#[test]
fn a_refused_range_saves_nothing() {
    let mut h = Harness::new();
    for (old_body, first, remove, body, why) in [
        (b"a\n".as_slice(), 0, 0, b"x".as_slice(), Refusal::PastEnd { lines: 1 }),
        (b"a\n", 0, 1, b"x", Refusal::PastEnd { lines: 1 }),
        (b"a\n", 2, 1, b"x", Refusal::PastEnd { lines: 1 }),
        (b"a\n", 1, 2, b"x", Refusal::PastEnd { lines: 1 }),
        (b"a", 1, u32::MAX, b"x", Refusal::PastEnd { lines: 1 }),
        (b"a", u32::MAX, u32::MAX, b"x", Refusal::PastEnd { lines: 1 }),
        (b"body", 5, 0, b"012345678901234567890123456789", Refusal::PastBound { bytes: 35, bound: 32 }),
        (b"0123456789012345678901234567890\n", 5, 0, b"x", Refusal::PastBound { bytes: 33, bound: 32 }),
    ] {
        let mut old = entry(7, 20, b"old");
        old.body = old_body.into();
        let got = range_write(&mut h, Some(old), range_new(body), 1, first, remove);
        assert_eq!(&*got, &[Request::Refused { owner: Token::new(51), why }]);
        assert!(!h.domain.busy());
    }
}
#[test]
fn a_range_keeps_references_unless_it_names_some() {
    for replace in [false, true] {
        let mut old = entry(7, 20, b"old");
        old.references.push(3).unwrap();
        let mut updated = range_new(b"x");
        if replace {
            updated.references.push(4).unwrap();
        }
        let got = range_write(&mut Harness::new(), Some(old), updated, 1, 1, 1);
        let [Request::Save { record: Record::Entry(saved) }, ..] = &*got else { panic!("saved: {got:?}") };
        assert_eq!(saved.references.as_slice(), &[if replace { 4 } else { 3 }]);
    }
}
#[test]
fn a_range_writes_the_description_it_names() {
    let got = range_write(&mut Harness::new(), Some(entry(7, 20, b"old")), range_new(b"x"), 1, 1, 1);
    let [Request::Save { record: Record::Entry(saved) }, Request::Save { record: Record::Line(line) }, ..] = &*got
    else {
        panic!("saved: {got:?}")
    };
    assert_eq!(&*saved.description, b"range");
    assert_eq!(line.description, saved.description);
}
#[test]
fn a_range_at_the_last_revision_is_exhausted() {
    let mut old = entry(7, 20, b"old");
    old.revision = u32::MAX;
    for (first, remove, why) in [
        (1, 1, Refusal::RevisionExhausted),
        (2, 0, Refusal::RevisionExhausted),
        (u32::MAX, 0, Refusal::RevisionExhausted),
        (0, 0, Refusal::PastEnd { lines: 1 }),
        (0, 1, Refusal::PastEnd { lines: 1 }),
        (2, 1, Refusal::PastEnd { lines: 1 }),
    ] {
        let got = range_write(&mut Harness::new(), Some(old.clone()), range_new(b"x"), u32::MAX, first, remove);
        assert_eq!(&*got, &[Request::Refused { owner: Token::new(51), why }]);
    }
}
#[test]
fn a_body_past_its_bound_says_so_in_either_form() {
    for range in [false, true] {
        let mut h = Harness::new();
        let updated = range_new(&[b'x'; 33]);
        let event = if range {
            Event::WriteLines {
                owner: Token::new(51),
                entry: updated,
                recalled: 1,
                range: crate::LineRange { first: 1, remove: 1, join: false },
            }
        } else {
            Event::Write { owner: Token::new(51), entry: updated, recalled: None }
        };
        assert_eq!(
            &*h.step(event),
            &[Request::Refused { owner: Token::new(51), why: Refusal::PastBound { bytes: 33, bound: 32 } }]
        );
        assert!(!h.domain.busy());
    }
}

#[test]
fn join_continues_the_line_before_first() {
    for (old, first, remove, text, expected) in [
        (b"a".as_slice(), 2, 0, b"b".as_slice(), b"ab".as_slice()),
        (b"a\n", 2, 0, b"b", b"ab"),
        (b"a\nb\nc", 2, 1, b"x", b"ax\nc"),
        (b"a\n\n", 3, 0, b"x", b"a\nx"),
        (b"", 1, 0, b"x", b"x"),
        (b"a\nb", 1, 1, b"x", b"x\nb"),
    ] {
        let range = crate::LineRange { first, remove, join: true };
        assert_eq!(&*crate::splice(old, range, text, 64).expect("joined range"), expected);
    }
}

#[test]
fn join_keeps_range_and_body_bounds() {
    let append = crate::LineRange { first: 2, remove: 0, join: true };
    assert_eq!(&*crate::splice(b"a\n", append, b"bc", 3).expect("joined bound"), b"abc");
    assert_eq!(crate::splice(b"a\n", append, b"bcd", 3), Err(Refusal::PastBound { bytes: 4, bound: 3 }));
    let past = crate::LineRange { first: 3, remove: 0, join: true };
    assert_eq!(&*crate::splice(b"a\n", past, b"b", 3).expect("joined overshooting append"), b"ab");
    assert_eq!(crate::splice(b"a\n", past, b"bcd", 3), Err(Refusal::PastBound { bytes: 4, bound: 3 }));
    for (first, remove) in [(0, 0), (0, 1), (2, 1), (1, 2)] {
        let range = crate::LineRange { first, remove, join: true };
        assert_eq!(crate::splice(b"a\n", range, b"b", 3), Err(Refusal::PastEnd { lines: 1 }));
    }
}

#[test]
fn a_partys_line_and_body_are_bounded_for_every_writer() {
    let scope = Scope::Party { party: 2 };
    let mut limits = LIMITS;
    limits.party_bytes = 4;
    for (description, body, refusal) in [
        (b"line".as_slice(), b"".as_slice(), None),
        (b"lines".as_slice(), b"".as_slice(), Some(Refusal::PastBound { bytes: 5, bound: 4 })),
        (b"line".as_slice(), b"x".as_slice(), Some(Refusal::PastBound { bytes: 1, bound: 0 })),
    ] {
        let mut entry = new(20, description);
        entry.scope = scope.clone();
        entry.body = body.into();
        let durable = Entry {
            name: entry.name,
            scope: scope.clone(),
            description: entry.description.clone(),
            body: entry.body.clone(),
            references: entry.references.clone(),
            author: entry.author,
            revision: 1,
            changed: Wall::EPOCH,
        };
        assert_eq!(crate::valid_new(&entry, &limits), refusal.is_none());
        assert_eq!(crate::valid_entry(&durable, &limits), refusal.is_none());
        let line = Line { scope: scope.clone(), name: 20, description: description.into(), revision: 1 };
        assert_eq!(crate::valid_line(&line, &limits), description.len() <= 4);
        for event in [
            Event::Write { owner: Token::new(51), entry: entry.clone(), recalled: None },
            Event::Write { owner: Token::new(51), entry: entry.clone(), recalled: Some(1) },
            Event::Edit {
                owner: Token::new(51),
                party: 2,
                name: 20,
                scope: scope.clone(),
                change: Change::Correct {
                    description: description.into(),
                    body: body.into(),
                    references: List::with_capacity(0),
                    recalled: 1,
                },
            },
        ] {
            let mut h = Harness::with_limits(limits);
            let got = h.step(event);
            if let Some(why) = &refusal {
                assert_eq!(&*got, &[Request::Refused { owner: Token::new(51), why: *why }]);
                assert!(!h.domain.busy());
            } else {
                let [Request::Load { .. }] = &*got else { panic!("at the bound: {got:?}") };
            }
        }
        let mut h = Harness::with_limits(limits);
        let got = h.step(Event::WriteLines {
            owner: Token::new(51),
            entry,
            recalled: 1,
            range: crate::LineRange { first: 1, remove: 0, join: false },
        });
        assert_eq!(
            &*got,
            &[Request::Refused {
                owner: Token::new(51),
                why: refusal.unwrap_or(Refusal::PastBound { bytes: 0, bound: 0 })
            }]
        );
    }
}
