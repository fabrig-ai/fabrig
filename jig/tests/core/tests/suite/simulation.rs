use jig_core_world::effects::{Scenario, Setup, World};

fn a_chats_run_answers_once_and_its_turn_is_acknowledged_after_its_commit() {
    let mut world = World::new(Setup {
        seed: 41,
        configuration: Some(jig_core_world::world::config(41)),
        limits: jig_core_world::world::limits(),
        walking: Some(Scenario::Chat),
        ..Setup::default()
    });
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| world.drain()));
    assert!(result.is_ok(), "{:?}", world.trace);
    assert_eq!(world.walking.as_ref().expect("walking peer").turn_acks, 1);
    assert_eq!(world.walking.as_ref().expect("walking peer").answer_acks, 1);
    assert_eq!(world.walking.as_ref().expect("walking peer").results, 1);
}

fn a_person_sets_a_goal_a_procedure_carries_out_and_hears_its_result() {
    let mut world = World::new(Setup {
        seed: 42,
        configuration: Some(jig_core_world::world::config(42)),
        limits: jig_core_world::world::limits(),
        walking: Some(Scenario::Goal),
        ..Setup::default()
    });
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| world.drain()));
    assert!(result.is_ok(), "{:?}", world.trace);
    assert_eq!(world.walking.as_ref().expect("walking peer").results, 1);
    assert!(world.walking.as_ref().expect("walking peer").procedure.is_some());
}

fn a_batch_beyond_the_limits_is_refused_whole_and_the_party_told_why() {
    let mut world = World::new(Setup {
        seed: 43,
        configuration: Some(jig_core_world::world::config(43)),
        limits: jig_core_world::world::limits(),
        walking: Some(Scenario::Batch),
        ..Setup::default()
    });
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| world.drain()));
    assert!(result.is_ok(), "{:?}", world.trace);
    assert_eq!(world.walking.as_ref().expect("walking peer").batch_refusals, 1);
    assert!(
        world.walking.as_ref().expect("walking peer").procedure.is_none(),
        "no member of the refused batch received an ID"
    );
    assert!(
        !world.store.rows.values().any(|row| matches!(row,
            jig_test_domain::Record::Core(jig_domain_core::Record::Tasks(jig_domain_tasks::Stored::Live(task)))
                if task.requester == jig_domain_tasks::Party::Task(world.walking.as_ref().expect("walking peer").chat.expect("parent goal"))
        )),
        "the batch left no child row"
    );
}

// Each named scenario preserves its own fixture and original assertions.
#[test]
fn simulation_scenario_family() {
    a_chats_run_answers_once_and_its_turn_is_acknowledged_after_its_commit();
    a_person_sets_a_goal_a_procedure_carries_out_and_hears_its_result();
    a_batch_beyond_the_limits_is_refused_whole_and_the_party_told_why();
}
