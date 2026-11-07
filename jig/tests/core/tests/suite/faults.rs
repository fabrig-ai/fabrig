//! Chosen crash cuts and their sweep bodies (`jig/docs/design/domain/testing.md`, 5, 9 and 10).

use jig_core_world::Fault as StoreFault;
use jig_core_world::faults;
use jig_core_world::world::Cut;
use jig_core_world::world::Setup;
use jig_core_world::world::World;
use jig_domain_core as core;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use jig_fake_workers::Fault;
use jig_test_connector as connector;
use jig_test_domain as root;
use skein_lib::Duration;

pub(crate) fn effect_scenario(world: &mut World) {
    world.turn(1, 2, None, b"before effect");
    if world.reached_cut.is_some() {
        return;
    }
    world.call(1, 100);
    if world.reached_cut.is_some() {
        return;
    }
    world.end(jig_domain_tasks::End::Parked, 2);
}

/// The first commit of `world` saving a row `chosen` picks.
fn commit_saving(world: &World, chosen: impl Fn(&root::Record) -> bool) -> u64 {
    let from_end = world
        .commits
        .iter()
        .rev()
        .rposition(|writes| writes.iter().any(|write| matches!(write, root::Write::Save(row) if chosen(row))))
        .expect("chosen row is saved");
    // Fixture commits are absent from `commits`; count back from the durable end.
    world.store.applied - u64::try_from(from_end).expect("commit distance")
}

fn outbox(row: &root::Record) -> bool {
    matches!(row, root::Record::Connector { record: connector::Record::Outbox(_), .. })
}

fn terminal(row: &root::Record) -> bool {
    matches!(row, root::Record::Core(core::Record::Core(core::CoreRecord::Terminal(_))))
}

fn sign_in(row: &root::Record) -> bool {
    matches!(row, root::Record::Core(core::Record::People(people::Stored::SignIn { .. })))
}

fn chat(row: &root::Record) -> bool {
    matches!(row, root::Record::Core(core::Record::Tasks(tasks::Stored::Live(_))))
}

fn claim(row: &root::Record) -> bool {
    matches!(row, root::Record::Core(core::Record::Core(core::CoreRecord::RunProof(_))))
}

fn assert_saved_at_cut(world: &World, cut: Cut, chosen: impl Fn(&root::Record) -> bool) {
    assert_eq!(
        world.store.rows.values().any(chosen),
        matches!(cut, Cut::Durable(_)),
        "the named row is durable exactly on the durable side of {cut:?}"
    );
}

pub(crate) fn an_effect_cut_recovers(cut: Cut) {
    an_effect_cut_recovers_checking(cut, |_| {});
}

fn an_effect_cut_recovers_checking(cut: Cut, check: impl FnOnce(&World)) {
    let mut world = World::new(Setup { seed: 300, ..Setup::default() });
    world.cut = Some(cut);
    effect_scenario(&mut world);
    assert_eq!(world.reached_cut, Some(cut));
    check(&world);
    let applied = world.systems[0].observed().iter().filter(|row| row.applied).count();
    world.store.fault(StoreFault::SlowPages { by: 2 });
    world.restart(300, false);
    assert!(world.domain.ready());
    assert!(!world.stopped);
    assert!(world.store.pending.is_empty());
    let recovered = world.systems[0].observed().iter().filter(|row| row.applied).count();
    assert!(recovered >= applied && recovered <= 1, "cut {cut:?}");
    world.restart(300, false);
    assert_eq!(world.systems[0].observed().iter().filter(|row| row.applied).count(), recovered);
}

#[test]
fn an_effect_and_its_terminal_survive_both_crash_cuts() {
    let mut reference = World::new(Setup { seed: 300, ..Setup::default() });
    effect_scenario(&mut reference);
    for chosen in [outbox, terminal] {
        let number = commit_saving(&reference, chosen);
        for cut in [Cut::Submitted(number), Cut::Durable(number)] {
            an_effect_cut_recovers_checking(cut, |world| assert_saved_at_cut(world, cut, chosen));
        }
    }
}

pub(crate) fn a_failed_commit_stops(number: u64) {
    a_failed_commit_stops_checking(number, |_| {});
}

fn a_failed_commit_stops_checking(number: u64, check: impl FnOnce(&World)) {
    let mut world = World::new(Setup { seed: 301, ..Setup::default() });
    world.store.fault(StoreFault::Fail { commit: number });
    effect_scenario(&mut world);
    assert!(world.stopped, "failed commit {number}");
    assert_eq!(world.store.applied, number - 1);
    check(&world);
    let rows = world.store.rows.clone();
    let outputs = world.trace.len();
    world.drain();
    assert_eq!(world.store.rows, rows);
    assert_eq!(world.trace.len(), outputs);
    world.restart(301, false);
    assert!(world.domain.ready());
    assert!(world.systems[0].observed().iter().filter(|row| row.applied).count() <= 1);
}

#[test]
fn a_failed_effect_or_terminal_commit_stops_before_any_later_output() {
    let mut reference = World::new(Setup { seed: 301, ..Setup::default() });
    effect_scenario(&mut reference);
    for chosen in [outbox, terminal] {
        let number = commit_saving(&reference, chosen);
        a_failed_commit_stops_checking(number, |world| {
            assert!(!world.store.rows.values().any(chosen), "the failed commit keeps no named row");
        });
    }
}

#[test]
fn a_slow_worker_keeps_its_assignment_and_resumes_after_the_delay() {
    let mut world = faults::workers(302, 1, false);
    faults::worker_fault(&mut world, 0, Fault::Slow { by: Duration::from_secs(1) });
    faults::say(&mut world, 2);
    assert_eq!(world.peers.as_ref().expect("peers").workers[0].turn_acks, 0);
    world.advance(Duration::from_secs(1));
    let worker = &world.peers.as_ref().expect("peers").workers[0];
    assert_eq!(worker.seen.len(), 1);
    assert_eq!(worker.turn_acks, 1);
    assert_eq!(worker.answer_acks, 1);
}

#[test]
fn a_worker_back_after_its_stop_bound_replays_a_parked_answer() {
    let mut world = faults::workers(303, 1, false);
    faults::worker_fault(&mut world, 0, Fault::DropChannel { for_: Duration::from_secs(1) });
    world.advance(Duration::from_secs(1));
    let worker = &world.peers.as_ref().expect("peers").workers[0];
    assert_eq!(worker.answer_acks, 1);
    assert_eq!(worker.seen.len(), 1);
    faults::say(&mut world, 2);
    assert_eq!(world.peers.as_ref().expect("peers").workers[0].seen.len(), 2);
}

#[test]
fn a_worker_returning_past_grace_cannot_charge_or_finish_its_old_attempt() {
    let mut world = faults::workers(304, 1, false);
    faults::worker_fault(&mut world, 0, Fault::DropChannel { for_: Duration::from_secs(7) });
    world.advance(Duration::from_secs(6));
    world.advance(Duration::from_secs(1));
    world.advance(Duration::from_secs(2));
    let worker = &world.peers.as_ref().expect("peers").workers[0];
    assert_eq!(worker.actual_spent, 0);
    assert_eq!(worker.seen.len(), 2, "the stale listing does not occupy the retry slot");
    assert!(worker.seen[1].attempt > worker.seen[0].attempt);
    assert!(worker.answer_acks >= 1, "the stale parked answer is fenced and acknowledged");
}

#[test]
fn a_vanished_worker_is_lost_and_another_worker_takes_the_retry() {
    let mut world = faults::workers(305, 2, false);
    let owner = world
        .peers
        .as_ref()
        .expect("peers")
        .workers
        .iter()
        .position(|worker| !worker.seen.is_empty())
        .expect("assigned host");
    let first = world.peers.as_ref().expect("peers").workers[owner].seen[0].clone();
    faults::worker_fault(&mut world, owner, Fault::Vanish);
    world.advance(Duration::from_secs(6));
    world.advance(Duration::from_secs(2));
    let peers = world.peers.as_ref().expect("peers");
    assert!(peers.workers[1 - owner].seen.iter().any(|run| run.task == first.task && run.attempt > first.attempt));
    assert_eq!(peers.workers[owner].seen.len(), 1);
}

#[test]
fn a_cold_restart_retries_a_party_request_and_retained_host_traffic() {
    let mut world = faults::workers(306, 1, false);
    let cut = Cut::Durable(world.store.applied + 1);
    world.cut = Some(cut);
    faults::say(&mut world, 2);
    assert_eq!(world.reached_cut, Some(cut));
    faults::restart(&mut world, 306, false);
    assert!(world.domain.ready());
    let peers = world.peers.as_ref().expect("peers");
    assert!(peers.parties[0].quiescent(), "request reply retried");
    assert!(peers.upstream.iter().filter(|up| matches!(up, jig_fake_workers::Up::Hello { .. })).count() >= 2);
    let task = peers.parties[0].last_task.expect("started chat");
    assert!(
        world.store.rows.values().any(|row| matches!(row,
            jig_test_domain::Record::Core(jig_domain_core::Record::Tasks(jig_domain_tasks::Stored::Live(row)))
                if row.number == task && row.inbox.iter().any(|word| word.words.as_ref() == b"continue")
        )),
        "the unanswered relay stays in the durable whole inbox"
    );
}

pub(crate) fn a_startup_cut_restarts(engine: bool, cut: Cut) {
    a_startup_cut_restarts_checking(engine, cut, |_| {});
}

fn a_startup_cut_restarts_checking(engine: bool, cut: Cut, check: impl FnOnce(&World)) {
    let mut world = faults::workers_at(307, 1, engine, Some(cut));
    assert_eq!(world.reached_cut, Some(cut));
    check(&world);
    faults::restart(&mut world, 307, engine);
    world.advance(Duration::from_secs(6));
    world.advance(Duration::from_secs(2));
    assert!(world.domain.ready());
    let peers = world.peers.as_ref().expect("peers");
    assert!(peers.parties[0].person.is_some());
    assert!(peers.parties[0].last_task.is_some());
    assert!(peers.parties[0].quiescent());
    let made = world
        .store
        .rows
        .values()
        .filter(|row| {
            matches!(
                row,
                jig_test_domain::Record::Core(jig_domain_core::Record::Tasks(jig_domain_tasks::Stored::Live(_)))
            )
        })
        .count();
    assert_eq!(made, 1, "one keyed chat at {cut:?}, engine {engine}");
    // Engine 6 step 3 settles the dead engine/in-process claim: phase, retry, reserve and activity change durably.
    if engine {
        world.broken();
    }
}

#[test]
fn the_sign_in_chat_and_claim_commits_restart_on_either_side_of_durability() {
    for engine in [false, true] {
        let mut reference = faults::workers(307, 1, engine);
        // Engine 6 step 3 settles the dead engine/in-process claim: phase, retry, reserve and activity change durably.
        if engine {
            reference.broken();
        }
        let chosen: &[fn(&root::Record) -> bool] = if engine { &[claim] } else { &[sign_in, chat, claim] };
        for &chosen in chosen {
            let number = commit_saving(&reference, chosen);
            for cut in [Cut::Submitted(number), Cut::Durable(number)] {
                a_startup_cut_restarts_checking(engine, cut, |world| assert_saved_at_cut(world, cut, chosen));
            }
        }
    }
}
