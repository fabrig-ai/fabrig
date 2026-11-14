use jig_core_world::world::{Scenario, Setup, World};

fn a_chats_run_answers_once_and_its_turn_is_acknowledged_after_its_commit() {
    eprintln!("Chat scenario seed 41");
    let mut world = World::new(Setup {
        seed: 41,
        configuration: Some(jig_test_profile::config(41)),
        limits: jig_test_profile::limits(),
        walking: Some(Scenario::Chat),
        ..Setup::default()
    });
    eprintln!("{:?}", world.trace);
    world.drain();
    assert_eq!(world.walking.as_ref().expect("walking peer").turn_acks, 1);
    assert_eq!(world.walking.as_ref().expect("walking peer").answer_acks, 1);
    assert_eq!(world.walking.as_ref().expect("walking peer").results, 1);
}

fn a_person_sets_a_goal_a_procedure_carries_out_and_hears_its_result() {
    eprintln!("Goal scenario seed 42");
    let mut world = World::new(Setup {
        seed: 42,
        configuration: Some(jig_test_profile::config(42)),
        limits: jig_test_profile::limits(),
        walking: Some(Scenario::Goal),
        ..Setup::default()
    });
    eprintln!("{:?}", world.trace);
    world.drain();
    assert_eq!(world.walking.as_ref().expect("walking peer").results, 1);
    assert!(world.walking.as_ref().expect("walking peer").procedure.is_some());
}

fn a_batch_beyond_the_limits_is_refused_whole_and_the_party_told_why() {
    eprintln!("Batch scenario seed 43");
    let mut world = World::new(Setup {
        seed: 43,
        configuration: Some(jig_test_profile::config(43)),
        limits: jig_test_profile::limits(),
        walking: Some(Scenario::Batch),
        ..Setup::default()
    });
    eprintln!("{:?}", world.trace);
    world.drain();
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
