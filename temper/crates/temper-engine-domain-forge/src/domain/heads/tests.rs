//! Heads follow live work; full tables refuse without interrupting the reporting step.
#![expect(clippy::disallowed_methods, reason = "ordinary tests inspect emitted requests with iterator predicates")]
use super::super::*;
use crate::tests::{self, fixtures};

const TASK: u64 = 2;
const DELEGATE: u64 = 90;
const BRANCH: &[u8] = b"temper/2/c2";
const TIP: skein_forge::api::Commit = [9; 32];

fn resource(branch: &[u8]) -> skein_forge_client::Resource {
    skein_forge_client::Resource {
        repository: tests::adopted().provider,
        what: skein_forge_client::What::Branch(branch.into()),
    }
}

fn name(branch: &[u8]) -> Name {
    from_client(&resource(branch), &tests::env().limits).unwrap()
}

fn full() -> (Domain, Env<Limits>) {
    let mut env = tests::env();
    env.limits.client.resources = 2;
    env.limits.holds = 2;
    let mut d = tests::domain_with_limits(&env.limits);
    d.repositories.insert(tests::adopted().provider, Box::new(tests::adopted())).unwrap();
    d.names.insert(1, Box::new([name(b"main"), name(b"temper/1/c1")])).unwrap();
    for branch in [b"main".as_slice(), b"temper/1/c1"] {
        let name = name(branch);
        d.heads.insert(name.clone(), BranchHead { name, commit: fixtures::BASE, owned: true }).unwrap();
    }
    (d, env)
}

fn change() -> ChangeRow {
    let mut row = fixtures::opening_row();
    row.task = TASK;
    row.change.task = TASK;
    row.branch = BRANCH.into();
    row.change.last_head = None;
    row.delegate = Some((DELEGATE, change::Delegate::Produce));
    row
}

fn watch(d: &mut Domain, env: &Env<Limits>) {
    child(
        d,
        env,
        skein_forge_client::Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic },
        &mut Queue::uncapped(),
    );
    child(
        d,
        env,
        skein_forge_client::Event::Keep {
            owner: Token::new(1),
            watches: Box::new([skein_forge_client::Watch { resource: resource(BRANCH), participating: false }]),
        },
        &mut Queue::uncapped(),
    );
    assert!(d.client.cached(&resource(BRANCH)).is_some());
}

fn hold(d: &mut Domain, writer: Option<Writer>) {
    let name = name(BRANCH);
    d.holds.insert(name.clone(), Hold { name, task: 7, writer, drift: None, design_writers: Box::new([]) }).unwrap();
}

fn refused(out: &Queue<Request>, task: Option<u64>, branch: &[u8]) {
    let expected = Request::HeadRefused { task, resource: name(branch), configured: 2, needed: 3 };
    let mut count = 0_u32;
    for request in out {
        if let Request::HeadRefused { .. } = request {
            assert_eq!(request, &expected);
            count = count.saturating_add(1);
        }
    }
    assert_eq!(count, 1);
    for request in out {
        assert!(!matches!(request, Request::Save { record: Stored::BranchHead(_), .. }));
        assert!(!matches!(request, Request::Erase { key: Key::BranchHead(_), .. }));
    }
}

fn assert_saved(out: &Queue<Request>, record: Stored, read_afresh: Option<Name>) {
    let expected = Request::Save { record, release: None, read_afresh };
    let mut saved = false;
    for request in out {
        saved |= *request == expected;
    }
    assert!(saved, "missing {expected:?} in {out:?}");
}

#[test]
fn full_table_push_report_frees_writer_and_pins_delegate_head_on_replay() {
    // Replay begins with the durable state before the answer.
    for _ in 0_u8..2 {
        let (mut d, env) = full();
        d.changes.insert(TASK, change()).unwrap();
        watch(&mut d, &env);
        hold(&mut d, Some(Writer::Run { task: DELEGATE, attempt: 1 }));
        child(
            &mut d,
            &env,
            skein_forge_client::Event::Writer { resource: resource(BRANCH), taken: true },
            &mut Queue::uncapped(),
        );
        let mut out = Queue::uncapped();
        answered(&mut d, &env, DELEGATE, 1, &[(name(BRANCH), TIP, true)], &mut out);
        refused(&out, Some(TASK), BRANCH);
        assert_eq!(d.branch_head(&name(BRANCH)), None);
        let saved = d.hold(&name(BRANCH)).unwrap();
        assert_eq!(saved.writer, None);
        assert_saved(&out, Stored::Hold(saved.clone()), Some(name(BRANCH)));
        assert_eq!(d.change(TASK).unwrap().change.last_head, Some(TIP));
        assert_saved(&out, Stored::Change(Box::new(d.change(TASK).unwrap().clone())), None);
        assert!(d.procedure_wakes.contains_key(&TASK));
        assert_eq!(d.client.cached(&resource(BRANCH)).unwrap().tip, Some(TIP));
        assert_eq!(d.heads.len(), 2);
    }
}

#[test]
fn replacement_at_capacity_is_saved_without_refusal() {
    let (mut d, env) = full();
    let mut out = Queue::uncapped();
    branch_changed(&mut d, &env, resource(b"main"), TIP, &mut out);
    assert_eq!(d.branch_head(&name(b"main")), Some(TIP));
    for request in &out {
        assert!(!matches!(request, Request::HeadRefused { .. }));
    }
    assert_saved(&out, Stored::BranchHead(d.heads.get(&name(b"main")).unwrap().clone()), None);
    assert_eq!(d.heads.len(), 2);
}

#[test]
fn released_moved_branch_clears_drift_and_tells_client_at_capacity() {
    let (mut d, env) = full();
    watch(&mut d, &env);
    hold(&mut d, None);
    d.holds.get_mut(&name(BRANCH)).unwrap().drift = Some(crate::Drift {
        at: env.wall,
        change: crate::DriftChange::Branch { expected: fixtures::HEAD, observed: Some(TIP) },
    });
    let mut row = change();
    row.drift = Some(change::Hold::BranchMoved);
    d.changes.insert(row.task, row.clone()).unwrap();
    let mut out = Queue::uncapped();
    accept_released_branch(&mut d, &env, &mut row, TIP, &mut out);
    refused(&out, Some(TASK), BRANCH);
    assert_eq!(row.drift, None);
    assert_eq!(d.hold(&name(BRANCH)).unwrap().drift, None);
    assert_saved(&out, Stored::Hold(d.hold(&name(BRANCH)).unwrap().clone()), Some(name(BRANCH)));
    assert_eq!(d.client.cached(&resource(BRANCH)).unwrap().tip, Some(TIP));
}

#[test]
fn full_table_observations_name_only_the_live_own_branch_not_a_shared_base() {
    for (branch, ended, concerning) in [
        (b"landing".as_slice(), false, None),
        (b"base", false, None),
        (BRANCH, false, Some(TASK)),
        (BRANCH, true, None),
    ] {
        let (mut d, env) = full();
        let mut row = change();
        row.base = b"base".as_slice().into();
        if ended {
            row.change.state = change::State::Ended;
        }
        d.changes.insert(TASK, row).unwrap();
        let mut out = Queue::uncapped();
        branch_changed(&mut d, &env, resource(branch), TIP, &mut out);
        refused(&out, concerning, branch);
        assert_eq!(out.len(), 1);
        assert_eq!(d.branch_head(&name(branch)), None);
    }
}

#[test]
fn lost_read_at_capacity_still_completes_the_report() {
    let (mut d, env) = full();
    d.changes.insert(TASK, change()).unwrap();
    watch(&mut d, &env);
    hold(&mut d, Some(Writer::Lost { task: DELEGATE, attempt: 1 }));
    let owner = Token::new(10);
    d.lost
        .insert(
            owner,
            PendingLost {
                task: DELEGATE,
                attempt: 1,
                name: name(BRANCH),
                comparison: None,
                baseline: None,
                state: LostReadState::Asked,
                kind: LostWriterKind::Slot,
            },
        )
        .unwrap();
    let mut out = Queue::uncapped();
    lost_read(&mut d, &env, owner, Ok(skein_forge::api::Answer::Commit(TIP)), &mut out);
    refused(&out, Some(TASK), BRANCH);
    assert_eq!(d.hold(&name(BRANCH)).unwrap().writer, None);
    assert_eq!(d.change(TASK).unwrap().change.last_head, Some(TIP));
    assert_eq!(d.client.cached(&resource(BRANCH)).unwrap().tip, Some(TIP));
    assert!(d.lost.is_empty());
    for request in &out {
        assert!(!matches!(request, Request::LostReadFailed { .. }));
    }
}

#[test]
fn stale_full_table_push_report_erases_old_heads_and_saves_the_new_head() {
    let (mut d, env) = full();
    d.names.remove(&1);
    watch(&mut d, &env);
    hold(&mut d, Some(Writer::Run { task: DELEGATE, attempt: 1 }));
    let mut out = Queue::uncapped();
    answered(&mut d, &env, DELEGATE, 1, &[(name(BRANCH), TIP, true)], &mut out);
    assert_eq!(d.heads.len(), 1);
    assert_eq!(d.branch_head(&name(BRANCH)), Some(TIP));
    for branch in [b"main".as_slice(), b"temper/1/c1"] {
        assert!(out.iter().any(|r| *r == Request::Erase { key: Key::BranchHead(name(branch)) }));
    }
    assert!(out.iter().any(|r| matches!(r, Request::Save { record: Stored::BranchHead(row), .. } if row.name == name(BRANCH) && row.commit == TIP)));
    assert!(!out.iter().any(|r| matches!(r, Request::HeadRefused { .. })));
    assert_eq!(d.hold(&name(BRANCH)).unwrap().writer, None);
}

fn subscription(d: &mut Domain, branch: &[u8]) {
    let row = Subscriber {
        task: 8,
        number: 1,
        topic: Topic::Landings { repository: tests::adopted().provider, branch: branch.into() },
        own_change: None,
        goal_tasks: None,
        paths: Box::new([]),
    };
    d.subscriptions.insert((row.task, row.topic.clone()), row).unwrap();
}

#[test]
fn completing_release_erases_deleted_change_head_but_keeps_a_live_base() {
    for subscribed in [false, true] {
        let mut d = tests::domain();
        let env = tests::env();
        d.repositories.insert(tests::adopted().provider, Box::new(tests::adopted())).unwrap();
        d.changes.insert(TASK, change()).unwrap();
        for branch in [BRANCH, b"main".as_slice()] {
            let name = name(branch);
            d.heads.insert(name.clone(), BranchHead { name, commit: TIP, owned: true }).unwrap();
        }
        hold(&mut d, None);
        d.holds.get_mut(&name(BRANCH)).unwrap().task = TASK;
        d.names.insert(TASK, Box::new([name(BRANCH)])).unwrap();
        // The delete's outcome removes the hold; the completion must end the change before sweeping.
        let mut row = release_row(TASK, TASK, ReleaseEnding::Done, None, true, true);
        row.pending = Some((5, Some(name(BRANCH))));
        d.releases.insert(TASK, row).unwrap();
        if subscribed {
            subscription(&mut d, b"main");
        } else {
            let mut other = change();
            other.task = 3;
            other.change.task = 3;
            other.branch = b"temper/3/c3".as_slice().into();
            d.changes.insert(3, other).unwrap();
        }
        let mut out = Queue::uncapped();
        release_outcome(
            &mut d,
            5,
            skein_forge_client::Outcome::Made { made: skein_forge_client::Made::Set, found: false },
            &mut out,
        );
        assert!(d.holds.get(&name(BRANCH)).is_none());
        assert_eq!(d.branch_head(&name(BRANCH)), Some(TIP));
        continue_release(&mut d, &env, TASK, 6, &mut out);
        assert!(out.iter().any(|r| *r == Request::Released { task: TASK }));
        assert!(out.iter().any(|r| *r == Request::Erase { key: Key::BranchHead(name(BRANCH)) }));
        assert_eq!(d.branch_head(&name(BRANCH)), None);
        assert_eq!(d.branch_head(&name(b"main")), Some(TIP));
    }
}

#[test]
fn a_sweep_frees_even_held_heads_of_ended_forges_and_unadopted_repositories() {
    for ended in [false, true] {
        let (mut d, env) = full();
        for branch in [b"main".as_slice(), b"temper/1/c1"] {
            let name = name(branch);
            d.holds
                .insert(
                    name.clone(),
                    Hold { name, task: TASK, writer: None, drift: None, design_writers: Box::new([]) },
                )
                .unwrap();
        }
        if ended {
            d.configured.remove(&1);
        } else {
            d.repositories.remove(&tests::adopted().provider);
        }
        let mut out = Queue::uncapped();
        super::free(&mut d, &env, &mut out);
        assert!(d.heads.is_empty());
        assert_eq!(out.len(), 2);
        for branch in [b"main".as_slice(), b"temper/1/c1"] {
            assert!(out.iter().any(|r| *r == Request::Erase { key: Key::BranchHead(name(branch)) }));
        }
    }
}

#[test]
fn a_sweep_keeps_unsettled_branch_writes_and_pending_lost_reads() {
    use skein_forge::api::Write;
    for effect in [
        skein_forge_client::Effect {
            write: Write::CreateBranch { branch: b"main".as_slice().into(), commit: TIP },
            condition: skein_forge_client::Condition::None,
        },
        skein_forge_client::Effect {
            write: Write::DeleteBranch { branch: b"main".as_slice().into() },
            condition: skein_forge_client::Condition::None,
        },
        skein_forge_client::Effect {
            write: Write::Merge { number: 1, head: TIP, how: skein_forge::api::MergeHow::Style },
            condition: skein_forge_client::Condition::Merge { base: b"main".as_slice().into() },
        },
    ] {
        let (mut d, env) = full();
        d.names.remove(&1);
        let entry = skein_forge_client::Entry {
            number: 1,
            task: TASK,
            repository: tests::adopted().provider,
            effect,
            start: None,
            attempt: None,
            failures: 0,
            queued: false,
        };
        d.entries.insert(1, Outboxed { entry, held: true, handoff: Handoff::Handed }).unwrap();
        d.lost
            .insert(
                Token::new(2),
                PendingLost {
                    task: TASK,
                    attempt: 1,
                    name: name(b"temper/1/c1"),
                    comparison: None,
                    baseline: None,
                    state: LostReadState::Asked,
                    kind: LostWriterKind::Slot,
                },
            )
            .unwrap();
        let mut out = Queue::uncapped();
        super::free(&mut d, &env, &mut out);
        assert_eq!(d.heads.len(), 2);
        assert!(out.is_empty());
        d.entries.remove(&1);
        d.lost.remove(&Token::new(2));
        super::free(&mut d, &env, &mut out);
        assert!(d.heads.is_empty());
        assert_eq!(out.len(), 2);
    }
}
