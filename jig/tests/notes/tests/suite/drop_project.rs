use jig_domain_notes::{self as notes, Event, Last, Line, Pattern, Range, Recall, Record, Request, Rows, Scope};
use jig_notes_world::{LIMITS, one_scope, task_entry};
use skein_lib::{Env, List, Queue, Time, Token, Wall};

const OWNER: Token = Token::new(100);

struct Fixture {
    domain: notes::Domain,
    env: Env<notes::Limits>,
}

impl Fixture {
    fn new(scopes: u32) -> Self {
        let limits = notes::Limits { scopes, ..LIMITS };
        Self { domain: notes::Domain::new(&limits), env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits } }
    }

    fn step(&mut self, event: Event) -> Vec<Request> {
        let mut out = Queue::uncapped();
        notes::step(&mut self.domain, &self.env, event, &mut out);
        let mut requests = Vec::new();
        while let Some(request) = out.pop() {
            requests.push(request);
        }
        requests
    }

    fn index(&mut self, scope: &Scope) -> Vec<Request> {
        self.step(Event::Index { owner: OWNER, scopes: one_scope(scope.clone()), most: 3 })
    }

    fn loaded(&mut self, owner: Token, lines: &[Line], more: bool) -> Vec<Request> {
        let mut records = List::with_capacity(self.env.limits.load_rows);
        for line in lines {
            records.push(Record::Line(line.clone())).expect("fixture page fits");
        }
        self.step(Event::Loaded { owner, rows: Rows { records }, more })
    }

    fn cache(&mut self, scope: &Scope) -> Vec<Request> {
        let out = self.index(scope);
        let owner = load(&out, scope, None);
        let out = self.loaded(owner, &[line(scope, 1)], false);
        assert!(!self.domain.busy());
        assert!(self.domain.holds(scope));
        assert_eq!(self.index(scope), out, "subsequent indexes use the cache");
        out
    }
}

fn line(scope: &Scope, name: u64) -> Line {
    Line { scope: scope.clone(), name, description: b"cached note".as_slice().into(), revision: 1 }
}

fn load(out: &[Request], scope: &Scope, after: Option<u64>) -> Token {
    let [Request::Load { owner, range }] = out else { panic!("one store load: {out:?}") };
    assert_eq!(*range, Range::Lines { scope: scope.clone(), after });
    *owner
}

#[test]
fn dropping_a_project_forgets_its_cached_indexes() {
    let removed = [
        Scope::Project { project: 1 },
        Scope::Goal { project: 1, goal: 9 },
        Scope::Resources {
            project: 1,
            connector: 2,
            pattern: Pattern { segments: Box::new([]), last: Last::Open(b"path".as_slice().into()) },
        },
    ];
    let retained = [Scope::Project { project: 2 }, Scope::Deployment];
    let mut fixture = Fixture::new(5);
    for scope in removed.iter().chain(&retained) {
        fixture.cache(scope);
    }
    assert_eq!(fixture.domain.cached_scopes(), 5);
    assert!(fixture.step(Event::DropProject { project: 1 }).is_empty(), "a drop writes nothing");
    assert_eq!(fixture.domain.cached_scopes(), 2);
    for scope in &removed {
        let out = fixture.index(scope);
        let owner = load(&out, scope, None);
        let out = fixture.loaded(owner, &[], false);
        assert_eq!(out, vec![Request::Indexed { owner: OWNER, lines: List::with_capacity(3), more: 0 }]);
    }
    for scope in &retained {
        let mut lines = List::with_capacity(3);
        lines.push(line(scope, 1)).expect("one line fits");
        assert_eq!(fixture.index(scope), vec![Request::Indexed { owner: OWNER, lines, more: 0 }]);
    }
    assert!(fixture.step(Event::DropProject { project: 99 }).is_empty());
    assert_eq!(fixture.domain.cached_scopes(), 5, "an absent project's drop leaves the cache alone");
}

#[test]
fn a_drop_while_busy_is_kept_for_after() {
    let cached = Scope::Goal { project: 1, goal: 9 };
    let pending = Scope::Project { project: 1 };
    let other = Scope::Project { project: 2 };
    let mut fixture = Fixture::new(3);
    fixture.cache(&cached);
    fixture.cache(&other);
    let out = fixture.index(&pending);
    let owner = load(&out, &pending, None);
    for project in [1, 2, 1] {
        assert!(fixture.step(Event::DropProject { project }).is_empty(), "a busy drop is retained without refusal");
    }
    assert!(fixture.domain.busy());
    assert!(fixture.domain.holds(&cached));
    let out = fixture.loaded(owner, &[line(&pending, 1)], true);
    let owner = load(&out, &pending, Some(1));
    assert!(fixture.domain.holds(&cached), "a continued load keeps invalidation deferred");
    let out = fixture.loaded(owner, &[], false);
    let [Request::Indexed { owner: OWNER, lines, more: 0 }] = out.as_slice() else {
        panic!("the active call finishes: {out:?}")
    };
    assert_eq!(lines.as_slice(), &[line(&pending, 1)]);
    assert!(!fixture.domain.busy());
    assert_eq!(fixture.domain.cached_scopes(), 0, "both projects dropped, including the just-loaded scope");
    for scope in [&cached, &pending, &other] {
        let out = fixture.index(scope);
        let owner = load(&out, scope, None);
        fixture.loaded(owner, &[], false);
    }
}

#[test]
fn deferred_drops_apply_when_a_read_or_write_load_fails() {
    for write in [false, true] {
        let scope = Scope::Project { project: 1 };
        let mut fixture = Fixture::new(1);
        fixture.cache(&scope);
        let event = if write {
            Event::Write { owner: OWNER, entry: task_entry(2, scope.clone(), b"new", b"body", 9), recalled: None }
        } else {
            Event::Recall { owner: OWNER, by: Recall::Name { name: 2 }, page: 0 }
        };
        let out = fixture.step(event);
        let [Request::Load { owner, .. }] = out.as_slice() else { panic!("entry load: {out:?}") };
        assert!(fixture.step(Event::DropProject { project: 1 }).is_empty());
        assert!(fixture.step(Event::LoadFailed { owner: Token::new(u64::MAX) }).is_empty());
        assert!(fixture.domain.holds(&scope), "a stale completion leaves the call busy");
        assert_eq!(
            fixture.step(Event::LoadFailed { owner: *owner }),
            vec![Request::Refused { owner: OWNER, why: notes::Refusal::Busy }]
        );
        assert!(!fixture.domain.busy());
        assert!(!fixture.domain.holds(&scope));
    }
}

#[test]
fn a_deferred_drop_applies_after_a_write_completes() {
    let cached = Scope::Goal { project: 1, goal: 9 };
    let written = Scope::Project { project: 1 };
    let other = Scope::Project { project: 2 };
    let mut fixture = Fixture::new(3);
    fixture.cache(&cached);
    let retained = fixture.cache(&other);
    let out = fixture.step(Event::Write {
        owner: OWNER,
        entry: task_entry(2, written.clone(), b"new", b"body", 9),
        recalled: None,
    });
    let [Request::Load { owner, range: Range::Entry { name: 2 } }] = out.as_slice() else {
        panic!("new entry lookup: {out:?}")
    };
    assert!(fixture.step(Event::DropProject { project: 1 }).is_empty());
    let out = fixture.loaded(*owner, &[], false);
    let owner = load(&out, &written, None);
    assert!(fixture.domain.holds(&cached), "the write still has a scope count in flight");
    let out = fixture.loaded(owner, &[], false);
    assert!(matches!(out.as_slice(), [Request::Save { .. }, Request::Save { .. }, Request::Written { .. }]));
    assert!(!fixture.domain.busy());
    assert!(!fixture.domain.holds(&cached));
    assert_eq!(fixture.index(&other), retained);
    let out = fixture.index(&cached);
    load(&out, &cached, None);
}

#[test]
fn more_deferred_projects_than_scopes_coalesce_into_cache_eviction() {
    let mut fixture = Fixture::new(2);
    for project in [1, 2] {
        fixture.cache(&Scope::Project { project });
    }
    let pending = Scope::Project { project: 3 };
    let out = fixture.index(&pending);
    let owner = load(&out, &pending, None);
    for project in [1, 1, 2, 3, 4] {
        assert!(fixture.step(Event::DropProject { project }).is_empty());
    }
    assert_eq!(fixture.domain.cached_scopes(), 2);
    let out = fixture.loaded(owner, &[line(&pending, 1)], false);
    assert!(matches!(out.as_slice(), [Request::Indexed { .. }]));
    assert_eq!(fixture.domain.cached_scopes(), 0, "the drop beyond the pending set's capacity is applied too");
    fixture.cache(&Scope::Project { project: 4 });
    assert_eq!(fixture.domain.cached_scopes(), 1, "the completed eviction does not affect later calls");
}
