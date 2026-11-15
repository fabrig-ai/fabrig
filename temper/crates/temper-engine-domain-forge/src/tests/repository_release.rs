//! Stored seeds exercise release without relying on the lifecycles that normally erase rows.
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_macros,
    reason = "ordinary stored-row fixtures use growable vectors"
)]
use super::*;
use crate::tests::{adopted, domain_with_limits, env, fixtures, outputs as drive, reconcile as links, restore_fit};
use crate::*;
use skein_forge::api;
use temper_engine_domain_forge_landing as policy;

const R: api::Repository = api::Repository { forge: 1, repository: 10 };
const Q: api::Repository = api::Repository { forge: 1, repository: 11 };
const S: api::Repository = api::Repository { forge: 1, repository: 12 };

fn limits() -> Limits {
    Limits {
        repositories: 4,
        client: skein_forge_client::Limits { repositories: 4, resources: 32, ..crate::tests::CLIENT },
        ..crate::tests::LIMITS
    }
}
fn branch(repository: api::Repository) -> Name {
    Name {
        forge: repository.forge,
        repository: repository.repository,
        what: What::Branch(Box::new([Box::from(b"temper".as_slice()), Box::from(b"x".as_slice())])),
    }
}
fn pull(repository: api::Repository, number: u64) -> Name {
    Name { forge: repository.forge, repository: repository.repository, what: What::Pull(number) }
}
fn adoption(repository: api::Repository, project: u32) -> Stored {
    let mut row = adopted();
    row.provider = repository;
    row.project = project;
    Stored::Repository(row)
}

#[expect(clippy::too_many_lines, reason = "each owned family has one independent stored fixture")]
fn families(repository: api::Repository, project: u32, n: u32) -> Vec<(Key, Stored)> {
    let task = u64::from(n);
    let name = branch(repository);
    let mut rows = vec![(Key::Repository(repository), adoption(repository, project))];
    let mut entry = crate::tests::status(task, task);
    entry.repository = repository;
    rows.push((
        Key::ProposedEffect(task),
        Stored::ProposedEffect(ProposedEffect { number: task, entry: entry.clone(), resource: What::Repository }),
    ));
    rows.push((
        Key::Hold(name.clone()),
        Stored::Hold(Hold { name: name.clone(), task, writer: None, drift: None, design_writers: Box::new([]) }),
    ));
    rows.push((Key::Names(task), Stored::Names { task, resources: Box::new([name.clone()]) }));
    let topic = Topic::Ci { repository, head: [3; 32] };
    rows.push((
        Key::Subscription { task, topic: topic.clone() },
        Stored::Subscription(Subscriber {
            task,
            number: 1,
            topic,
            own_change: None,
            goal_tasks: None,
            paths: Box::new([]),
        }),
    ));
    rows.push((
        Key::Ci { repository, head: [3; 32] },
        Stored::Ci(CiState { repository, head: [3; 32], status: api::Ci::Passed }),
    ));
    rows.push((
        Key::BranchHead(name.clone()),
        Stored::BranchHead(BranchHead { name: name.clone(), commit: [3; 32], owned: true }),
    ));
    rows.push((
        Key::PullState(pull(repository, 45)),
        Stored::PullState(PullState {
            name: pull(repository, 45),
            head: [3; 32],
            ci: api::Ci::Passed,
            state: api::State::Open,
        }),
    ));
    rows.push((Key::Entry(task), Stored::Entry { entry, held: true }));
    rows.push((Key::Flight { task, entry: task }, Stored::Flight { task, entry: task }));
    let commit = [u8::try_from(n).expect("fixture byte"); 32];
    rows.push((Key::Landed(commit), Stored::Landed { commit, task }));
    let mut change = fixtures::opening_row();
    change.task = task;
    change.repository = repository;
    let (cold, hot) = change.split();
    rows.push((Key::ChangeCold(task), Stored::ChangeCold(Box::new(cold))));
    rows.push((Key::ChangeHot(task), Stored::ChangeHot(Box::new(hot))));
    rows.push((
        Key::RedPass { change: task, key: b"test a::t x".as_slice().into() },
        restore_fit::row(RecordKind::RedPass, n),
    ));
    let Stored::Issue(mut issue) = restore_fit::row(RecordKind::Issue, n) else { unreachable!("issue fixture") };
    issue.repository = repository;
    rows.push((Key::Issue(task), Stored::Issue(issue)));
    let Stored::Release(mut release) = restore_fit::row(RecordKind::Release, n) else {
        unreachable!("release fixture")
    };
    release.pending = Some((task, Some(name.clone())));
    rows.push((Key::Release(task), Stored::Release(release)));
    let Stored::Landing(mut landing) = restore_fit::row(RecordKind::Landing, n) else {
        unreachable!("landing fixture")
    };
    landing.landing.project = project;
    landing.landing.members = Box::new([policy::Member {
        round: None,
        unit: 0,
        branch: b"temper/x".as_slice().into(),
        combined: None,
        recorded: false,
        told: false,
        judging: false,
        leave_told: false,
        left: None,
        change: task,
        repository: policy::Repository { forge: repository.forge, repository: repository.repository },
        pull: 45,
        head: Some([3; 32]),
        base: None,
        candidate: None,
        gates: Box::new([]),
        asked: None,
        landed: None,
    }]);
    landing.asks = Box::new([None]);
    rows.push((Key::Landing(task), Stored::Landing(landing)));
    let Stored::Receipt { mut key, receipt } = restore_fit::row(RecordKind::Receipt, n) else {
        unreachable!("receipt fixture")
    };
    key.project = 6; // A repository's receipts go under every project.
    key.repository = repository;
    rows.push((Key::Receipt(key.clone()), Stored::Receipt { key, receipt }));
    let Stored::Tier(mut tier) = restore_fit::row(RecordKind::Tier, n) else { unreachable!("tier fixture") };
    tier.repository = repository;
    rows.push((Key::Tier { repository, tier: tier.tier.clone() }, Stored::Tier(tier)));
    rows.push((
        Key::Verification { repository, commit },
        Stored::Verification(Verification { repository, commit, changes: Box::new([task]), tiers: Box::new([]) }),
    ));
    rows.push((Key::LandingLog { repository }, Stored::LandingLog(LandingLog { repository, landings: Box::new([]) })));
    let Stored::LandedDraft(mut draft) = restore_fit::row(RecordKind::LandedDraft, n) else {
        unreachable!("draft fixture")
    };
    draft.repository = repository;
    draft.project = project;
    rows.push((Key::LandedDraft { repository, path: draft.path.clone() }, Stored::LandedDraft(draft)));
    let [_, _, _, mark, live, _, application] = links::repository_rows(repository);
    rows.push((Key::Client(skein_forge_client::Key::Repository(repository)), mark));
    let Stored::Client(skein_forge_client::Stored::Live(mut live)) = live else { unreachable!("live fixture") };
    live.watch.resource.what = skein_forge_client::What::Branch(b"temper/x".as_slice().into());
    rows.push((
        Key::Client(skein_forge_client::Key::Live(live.watch.resource.clone())),
        Stored::Client(skein_forge_client::Stored::Live(live)),
    ));
    let Stored::Application { outcome, .. } = application else { unreachable!("application fixture") };
    rows.push((
        Key::Application(task),
        Stored::Application { token: task, until: skein_lib::Wall::from_nanos(u64::MAX), outcome },
    ));
    rows
}
fn restore(rows: &[(Key, Stored)], l: &Limits) -> Domain {
    let mut d = domain_with_limits(l);
    for (_, row) in rows {
        at(&mut d, l, Event::Restore { record: row.clone() });
    }
    assert_eq!(d.fit_restored(l), Ok(()));
    d
}
fn at(d: &mut Domain, l: &Limits, event: Event) -> Box<[Request]> {
    if let Event::Restore { .. } = &event {
        d.restore_begin(skein_lib::Wall::EPOCH);
    }
    let e = env().with(*l);
    let mut out = Queue::uncapped();
    step(d, &e, event, &mut out);
    let mut requests = Vec::new();
    for _ in 0..out.len() {
        requests.push(out.pop().unwrap());
    }
    requests.into_boxed_slice()
}
fn released(d: &mut Domain, project: u32, request: u64) -> Box<[Request]> {
    drive(d, Event::ReleaseProject { project, request: Token::new(request) })
}
fn erases(requests: &[Request]) -> Vec<Key> {
    let mut keys = Vec::new();
    for request in requests {
        if let Request::Erase { key } = request {
            keys.push(key.clone());
        }
    }
    keys
}
fn position(keys: &[Key], key: &Key) -> usize {
    for (at, found) in keys.iter().enumerate() {
        if found == key {
            return at;
        }
    }
    panic!("missing key: {key:?}");
}
fn assert_empty(d: &Domain, repository: api::Repository, n: u32) {
    let task = u64::from(n);
    assert!(!d.repositories.contains_key(&repository));
    assert!(!d.holds.contains_key(&branch(repository)));
    assert!(!d.heads.contains_key(&branch(repository)));
    assert!(!d.pulls.contains_key(&pull(repository, 45)));
    assert!(!d.ci.contains_key(&(repository, [3; 32])));
    assert!(!d.subscriptions.contains_key(&(task, Topic::Ci { repository, head: [3; 32] })));
    assert!(!d.proposed_effects.contains_key(&task));
    assert!(!d.entries.contains_key(&task));
    assert!(!d.flights.contains_key(&(task, task)));
    assert!(!d.changes.contains_key(&task));
    assert!(!d.change_cold.contains_key(&task));
    assert!(!d.change_hot.contains_key(&task));
    assert!(!d.landed.contains_key(&[u8::try_from(n).unwrap(); 32]));
    assert!(!d.red_passes.contains_key(&(task, b"test a::t x".as_slice().into())));
    assert!(!d.issues.contains_key(&task));
    assert!(!d.releases.contains_key(&task));
    assert!(!d.landing.rows.contains_key(&task));
    for (key, _) in &d.landing.receipts {
        assert_ne!(key.repository, repository);
    }
    for (key, _) in &d.tiers {
        assert_ne!(key.0, repository);
    }
    for (key, _) in &d.verifications {
        assert_ne!(key.0, repository);
    }
    assert!(!d.landing_logs.contains_key(&repository));
    for (key, _) in &d.drafts.as_ref().unwrap().rows {
        assert_ne!(key.0, repository);
    }
    assert!(!d.application.kept.contains_key(&Token::new(task)));
    for (_, names) in d.kept.iter().chain(d.names.iter()) {
        for name in names {
            assert!(!owns(name, repository));
        }
    }
}

#[test]
fn a_project_release_erases_every_owned_family_and_keeps_other_adoptions() {
    let l = limits();
    let owned = families(R, 5, 1);
    let other = families(Q, 6, 2);
    let mut context = families(S, 5, 3);
    for (_, row) in &mut context {
        if let Stored::Repository(row) = row {
            row.role = Role::Context;
            row.home = false;
            row.contexts = Box::new([6]);
        }
        if let Stored::LandedDraft(row) = row {
            row.project = 6;
        }
    }
    let Stored::Receipt { mut key, receipt } = restore_fit::row(RecordKind::Receipt, 3) else {
        unreachable!("receipt fixture")
    };
    key.repository = S;
    key.project = 5;
    let own_receipt = Key::Receipt(key.clone());
    let mut d = restore(&owned, &l);
    d.link_names(&links::empty_tasks(), &l); // R's erased task has kept names; Q and S still have live names.
    for (_, row) in other.iter().chain(&context) {
        at(&mut d, &l, Event::Restore { record: row.clone() });
    }
    at(&mut d, &l, Event::Restore { record: Stored::Receipt { key, receipt } });
    assert_eq!(d.fit_restored(&l), Ok(()));
    at(&mut d, &l, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    let requests = released(&mut d, 5, 100);
    let erased = erases(&requests);
    assert!(erased.contains(&own_receipt), "deleted project receipt of kept context");
    for (key, _) in &owned {
        assert!(erased.contains(key), "not erased: {key:?}");
    }
    for (key, _) in other.iter().chain(&context) {
        assert!(!erased.contains(key), "unrelated erasure: {key:?}");
    }
    let last = position(&erased, &Key::Repository(R));
    assert_eq!(last, erased.len() - 1, "Repository is its last erasure");
    let flight = position(&erased, &Key::Flight { task: 1, entry: 1 });
    let entry = position(&erased, &Key::Entry(1));
    assert!(flight < entry);
    assert_eq!(requests.last(), Some(&Request::ProjectReleased { request: Token::new(100) }));
    assert_empty(&d, R, 1);
    assert_eq!(d.repository(Q).unwrap().project, 6);
    assert_eq!(d.repository(S).unwrap().project, 6);
    let mut rehomed = false;
    for request in &requests {
        if let Request::Save { record: Stored::Repository(row), .. } = request {
            assert_ne!(row.provider, Q);
            rehomed |= row.provider == S && row.project == 6;
        }
    }
    assert!(rehomed);
    for (_, row) in &other {
        assert!(
            !requests.contains(&Request::Save { read_afresh: None, release: None, record: row.clone() }),
            "unrelated save: {row:?}"
        );
    }
    assert_eq!(released(&mut d, 5, 101).as_ref(), &[Request::ProjectReleased { request: Token::new(101) }]);
    let mut empty = domain_with_limits(&l);
    assert_eq!(released(&mut empty, 5, 102).as_ref(), &[Request::ProjectReleased { request: Token::new(102) }]);
}

fn open(d: &mut Domain, l: &Limits) {
    at(d, l, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    at(d, l, Event::ReadAfresh);
}
#[expect(clippy::mem_replace_with_default, reason = "the subset uses an explicit empty terminal value")]
fn apply(rows: &mut Vec<(Key, Stored)>, requests: &[Request]) {
    for request in requests {
        if let Request::Erase { key } = request {
            let mut kept = Vec::new();
            for (held, row) in core::mem::replace(rows, Vec::new()) {
                if held != *key {
                    kept.push((held, row));
                }
            }
            *rows = kept;
        }
        if let Request::Save { record, .. } = request {
            let key = if let Stored::Repository(row) = record {
                Key::Repository(row.provider)
            } else if let Stored::Names { task, .. } = record {
                Key::Names(*task)
            } else {
                panic!("unexpected page save: {record:?}");
            };
            let mut replaced = false;
            for (held, row) in &mut *rows {
                if *held == key {
                    *row = record.clone();
                    replaced = true;
                }
            }
            assert!(replaced, "release saves change an existing row");
        }
    }
}
fn writes(requests: &[Request]) -> usize {
    let mut count = 0_usize;
    for request in requests {
        if matches!(request, Request::Erase { key: Key::Client(skein_forge_client::Key::Live(_)) }) {
            continue; // The watch set erases its live rows in the releasing decision.
        }
        if matches!(request, Request::Erase { .. } | Request::Save { .. }) {
            count = count.checked_add(1).unwrap();
        }
    }
    count
}
fn fire_page(d: &mut Domain, l: &Limits) -> Box<[Request]> {
    let mut out = Queue::uncapped();
    fire(d, &env().with(*l), &mut out);
    let mut requests = Vec::new();
    for _ in 0..out.len() {
        requests.push(out.pop().unwrap());
    }
    requests.into_boxed_slice()
}
fn completed(requests: &[Request], token: u64) -> bool {
    requests.contains(&Request::ProjectReleased { request: Token::new(token) })
}
fn paging_seed() -> (Limits, Vec<(Key, Stored)>) {
    let l =
        Limits { issues: 512, client: skein_forge_client::Limits { resources: 600, ..limits().client }, ..limits() };
    let mut rows = vec![(Key::Repository(R), adoption(R, 5)), (Key::Repository(Q), adoption(Q, 6))];
    let Stored::Repository(mut context) = adoption(S, 5) else { unreachable!("context fixture") };
    context.role = Role::Context;
    context.home = false;
    context.contexts = Box::new([6]);
    rows.push((Key::Repository(S), Stored::Repository(context)));
    rows.push((Key::Names(7), Stored::Names { task: 7, resources: Box::new([branch(R), branch(Q)]) }));
    let [_, _, _, mark, live, _, _] = links::repository_rows(R);
    let Stored::Client(skein_forge_client::Stored::Live(live_row)) = &live else { unreachable!("live fixture") };
    let live_key = Key::Client(skein_forge_client::Key::Live(live_row.watch.resource.clone()));
    rows.push((Key::Client(skein_forge_client::Key::Repository(R)), mark));
    rows.push((live_key, live));
    // Two saves and the client's listing-mark erasure leave 509 slots.
    // The 508 pending rows leave one slot, so the change's parts wait together.
    for n in 1..=508 {
        let Stored::Issue(mut row) = restore_fit::row(RecordKind::Issue, n) else { unreachable!("issue fixture") };
        row.repository = R;
        row.failed = true;
        rows.push((Key::Issue(u64::from(n)), Stored::Issue(row)));
    }
    let mut change = fixtures::opening_row();
    change.repository = R;
    let task = change.task;
    let (cold, hot) = change.split();
    rows.push((Key::ChangeCold(task), Stored::ChangeCold(Box::new(cold))));
    rows.push((Key::ChangeHot(task), Stored::ChangeHot(Box::new(hot))));
    for n in 1_u32..=600 {
        let mut name = branch(R);
        name.what = What::Branch(Box::new([
            Box::from(b"temper".as_slice()),
            skein_lib::Decimal::of(u64::from(n)).as_bytes().into(),
        ]));
        rows.push((
            Key::BranchHead(name.clone()),
            Stored::BranchHead(BranchHead { name, commit: [3; 32], owned: true }),
        ));
        let name = pull(R, u64::from(n));
        rows.push((
            Key::PullState(name.clone()),
            Stored::PullState(PullState { name, head: [3; 32], state: api::State::Open, ci: api::Ci::None }),
        ));
    }
    (l, rows)
}

#[test]
fn release_pages_are_bounded_link_whole_and_restartable_at_every_cut() {
    let (l, mut rows) = paging_seed();
    let mut d = restore(&rows, &l);
    d.link_names(&links::empty_tasks(), &l);
    assert_eq!(d.missing_link(&links::empty_tasks()), None);
    at(&mut d, &l, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    let mut page = at(&mut d, &l, Event::ReleaseProject { project: 5, request: Token::new(100) });
    assert!(!completed(&page, 100));
    let mut pages = 0_u32;
    for _ in 0_u32..10 {
        pages = pages.checked_add(1).unwrap();
        assert!(writes(&page) <= 512, "release page has {} writes", writes(&page));
        let erased = erases(&page);
        assert_eq!(
            erased.contains(&Key::ChangeCold(fixtures::CHANGE)),
            erased.contains(&Key::ChangeHot(fixtures::CHANGE)),
            "a change's parts share a page"
        );
        if pages > 1 {
            for request in &page {
                assert!(!matches!(request, Request::Save { .. }), "all saves belong to the first page");
            }
        }
        apply(&mut rows, &page);
        let mut restarted = restore(&rows, &l);
        restarted.link_names(&links::empty_tasks(), &l);
        assert_eq!(restarted.missing_link(&links::empty_tasks()), None, "cut after page {pages}");
        at(&mut restarted, &l, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
        let mut rest = at(&mut restarted, &l, Event::ReleaseProject { project: 5, request: Token::new(200) });
        let mut answered = completed(&rest, 200);
        let mut repeated = false;
        if !answered {
            rest = at(&mut restarted, &l, Event::ReleaseProject { project: 5, request: Token::new(201) });
            repeated = true;
            answered = completed(&rest, 200);
        }
        for _ in 0_u32..10 {
            assert!(writes(&rest) <= 512);
            for request in &rest {
                assert!(
                    !matches!(request, Request::ResourceRefused { .. }),
                    "the restored client still admits the watch union"
                );
            }
            if answered {
                break;
            }
            assert_eq!(restarted.next_wall_deadline(), Some(skein_lib::Wall::from_nanos(0)));
            rest = fire_page(&mut restarted, &l);
            answered = completed(&rest, 200);
        }
        assert!(answered, "restored release must answer");
        if repeated {
            assert!(completed(&rest, 201), "the repeated release waits for the same last page");
        }
        assert!(!pending(&restarted));
        if completed(&page, 100) {
            break;
        }
        assert_eq!(d.next_wall_deadline(), Some(skein_lib::Wall::from_nanos(0)));
        page = fire_page(&mut d, &l);
    }
    assert!(pages > 1);
    assert!(completed(&page, 100));
    assert!(!pending(&d));
    assert_eq!(
        at(&mut d, &l, Event::ReleaseProject { project: 5, request: Token::new(101) }).as_ref(),
        &[Request::ProjectReleased { request: Token::new(101) }]
    );
}

fn client_output(d: &mut Domain, request: skein_forge_client::Request) -> Queue<Request> {
    d.client_out.push(request);
    let mut out = Queue::uncapped();
    super::super::drain(d, &env(), &mut out);
    out
}
fn no_saves(out: &Queue<Request>) {
    for request in out {
        assert!(!matches!(request, Request::Save { .. }), "late answer saved: {request:?}");
    }
}

#[test]
fn a_released_repository_retires_its_ci_read_and_ignores_the_answer() {
    let l = limits();
    let mut d = restore(&[(Key::Repository(R), adoption(R, 5))], &l);
    open(&mut d, &l);
    drive(
        &mut d,
        Event::Subscribe {
            subscription: Subscriber {
                task: 7,
                number: 1,
                topic: Topic::Ci { repository: R, head: [3; 32] },
                own_change: None,
                goal_tasks: None,
                paths: Box::new([]),
            },
        },
    );
    let (owner, _) = d.pending_ci.first().expect("subscription started a CI read");
    let owner = *owner;
    let (call, op) = fixtures::next_call(&mut d);
    assert_eq!(op, api::Op::Read(api::Read::Checks { commit: [3; 32] }));
    released(&mut d, 5, 100);
    assert!(d.pending_ci.is_empty(), "release retires CI reads in its decision");
    assert!(!d.ci.contains_key(&(R, [3; 32])));
    let result = Ok(api::Answer::Checks(Box::new([])));
    let requests = fixtures::answer(&mut d, call, result.clone());
    for request in requests {
        assert!(!matches!(request, Request::Save { .. }));
    }
    no_saves(&client_output(&mut d, skein_forge_client::Request::Read { owner, result }));
    assert!(d.pending_ci.is_empty());
    assert!(d.ci.is_empty());
}

#[test]
fn late_branch_pull_and_client_rows_do_not_recreate_a_released_repository() {
    let l = limits();
    let mut d = restore(&[(Key::Repository(R), adoption(R, 5))], &l);
    open(&mut d, &l);
    released(&mut d, 5, 100);
    let resource = skein_forge_client::Resource {
        repository: R,
        what: skein_forge_client::What::Branch(b"temper/x".as_slice().into()),
    };
    no_saves(&client_output(
        &mut d,
        skein_forge_client::Request::Changed { resource, result: Ok(api::Answer::Commit([8; 32])) },
    ));
    let resource = skein_forge_client::Resource { repository: R, what: skein_forge_client::What::Pull(45) };
    no_saves(&client_output(
        &mut d,
        skein_forge_client::Request::Changed {
            resource,
            result: Ok(api::Answer::Pull(api::Pull {
                number: 45,
                head: b"temper/x".as_slice().into(),
                base: b"main".as_slice().into(),
                state: api::State::Open,
                commit: [8; 32],
                base_commit: Some([7; 32]),
                merged: None,
                mergeable: true,
                ci: api::Ci::Passed,
                queued: None,
                dequeued: None,
                size: None,
                reviewers: Box::new([]),
            })),
        },
    ));
    let [_, _, _, mark, live, _, _] = links::repository_rows(R);
    for row in [mark, live] {
        let Stored::Client(record) = row else { unreachable!("client fixture") };
        no_saves(&client_output(&mut d, skein_forge_client::Request::Save { record }));
    }
    assert!(d.heads.is_empty());
    assert!(d.pulls.is_empty());
    assert!(d.pull_mergeable.is_empty());
}

#[test]
fn a_sent_entrys_late_outcome_is_dropped_after_its_repository_is_released() {
    let l = limits();
    let mut entry = crate::tests::status(11, 7);
    entry.repository = R;
    let mut d =
        restore(&[(Key::Repository(R), adoption(R, 5)), (Key::Entry(11), Stored::Entry { entry, held: false })], &l);
    open(&mut d, &l);
    drive(&mut d, Event::SettleOutbox);
    let (mut call, mut op) = fixtures::next_call(&mut d);
    for _ in 0_u32..3 {
        let answer = if matches!(op, api::Op::Read(api::Read::Checks { .. })) {
            Some(api::Answer::Checks(Box::new([])))
        } else if matches!(op, api::Op::Read(api::Read::Items { .. })) {
            Some(api::Answer::Items { cut: false, items: Box::new([]), more: false, now: env().now })
        } else {
            None
        };
        let Some(answer) = answer else { break };
        fixtures::answer(&mut d, call, Ok(answer));
        (call, op) = fixtures::next_call(&mut d);
    }
    assert!(matches!(op, api::Op::Write(api::Write::Status { .. })), "entry was sent before release: {op:?}");
    released(&mut d, 5, 100);
    assert!(d.entries.is_empty());
    assert!(d.flights.is_empty());
    let requests = fixtures::answer(&mut d, call, Ok(api::Answer::Done));
    assert_eq!(
        requests.as_ref(),
        &[Request::Erase { key: Key::Entry(11) }],
        "late settlement only repeats the idempotent outbox erasure"
    );
    let mut out = client_output(
        &mut d,
        skein_forge_client::Request::Outcome {
            entry: 11,
            task: 7,
            outcome: skein_forge_client::Outcome::Made {
                made: skein_forge_client::Made::Merged([8; 32]),
                found: false,
            },
        },
    );
    assert_eq!(out.len(), 1, "late outcomes do not reach settlement");
    assert_eq!(out.pop(), Some(Request::Erase { key: Key::Entry(11) }));
    assert!(d.landed.is_empty());
    assert!(d.entries.is_empty());
    assert!(d.verifications.is_empty());
}

#[test]
fn a_rolled_back_adoption_releases_every_repository_family_too() {
    let l = limits();
    let rows = families(R, 5, 1);
    let mut d = restore(&rows, &l);
    open(&mut d, &l);
    let requests = drive(&mut d, Event::ForgetAdoption { repository: R, restore: None });
    let erased = erases(&requests);
    for (key, _) in &rows {
        assert!(erased.contains(key), "rollback left {key:?}");
    }
    assert_eq!(erased.last(), Some(&Key::Repository(R)));
    for request in requests {
        assert!(!matches!(request, Request::ProjectReleased { .. }));
    }
    assert_empty(&d, R, 1);
}

#[test]
fn unnamed_releases_an_ended_forge_in_pages_and_restart_resumes_it() {
    let l =
        Limits { issues: 512, client: skein_forge_client::Limits { resources: 600, ..limits().client }, ..limits() };
    let mut record = crate::tests::records::record(1);
    record.state = ForgeState::Ended { secrets_erased: true, box_removed: true };
    let mut rows = vec![
        (Key::Forge(1), Stored::Forge(record)),
        (Key::Repository(R), adoption(R, 5)),
        (Key::Names(7), Stored::Names { task: 7, resources: Box::new([branch(R)]) }),
        (Key::LandingLog { repository: R }, Stored::LandingLog(LandingLog { repository: R, landings: Box::new([]) })),
    ];
    // Unnamed spends one write on Names; these leave one page slot before the paired change parts.
    for n in 1..=510 {
        let Stored::Issue(mut row) = restore_fit::row(RecordKind::Issue, n) else { unreachable!("issue fixture") };
        row.repository = R;
        row.failed = true;
        rows.push((Key::Issue(u64::from(n)), Stored::Issue(row)));
    }
    let mut change = fixtures::opening_row();
    change.repository = R;
    change.change.state = temper_engine_domain_forge_change::State::Ended;
    change.ending = Some(Ending::Cancelled);
    let task = change.task;
    let (cold, hot) = change.split();
    rows.push((Key::ChangeCold(task), Stored::ChangeCold(Box::new(cold))));
    rows.push((Key::ChangeHot(task), Stored::ChangeHot(Box::new(hot))));
    for n in 1_u32..=600 {
        let name = pull(R, u64::from(n));
        rows.push((
            Key::PullState(name.clone()),
            Stored::PullState(PullState { name, head: [3; 32], ci: api::Ci::Passed, state: api::State::Open }),
        ));
    }
    let mut d = restore(&rows, &l);
    d.link_names(&links::empty_tasks(), &l);
    open(&mut d, &l);
    assert!(d.record(1).is_some(), "kept task names retain the repository and forge");
    let first = at(&mut d, &l, Event::Unnamed { task: 7 });
    assert!(!erases(&first).contains(&Key::Forge(1)), "forge waits for its repositories' last pages");
    assert!(d.record(1).is_some());
    apply(&mut rows, &first);
    let mut restarted = restore(&rows, &l);
    assert_eq!(restarted.missing_link(&links::empty_tasks()), None);
    let restart = at(&mut restarted, &l, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    let mut erased = erases(&first);
    erased.extend(erases(&restart));
    for _ in 0_u32..10 {
        if !pending(&restarted) {
            break;
        }
        // The root opens the restored connector before driving its wall deadline.
        at(&mut restarted, &l, Event::ReadAfresh);
        let page = fire_page(&mut restarted, &l);
        erased.extend(erases(&page));
    }
    assert!(!pending(&restarted));
    assert!(restarted.record(1).is_none());
    assert!(position(&erased, &Key::Repository(R)) < position(&erased, &Key::Forge(1)));
    assert!(restarted.pulls.is_empty());
    assert!(restarted.landing_logs.is_empty());
}
