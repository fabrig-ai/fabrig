//! Visitor stories against the independent forge (temper/docs/design/client/forge.md, 11.5).
use skein_lib::Reader;
use temper_client_protocol_wire::public_v1 as w;
use temper_engine_domain_forge as top;
use temper_engine_domain_forge_landing as policy;
use temper_engine_forge_world::{
    changes, landings,
    public::{self, BRANCH, CHANGE, GOAL, LANDING, OTHER_BRANCH, SECOND, SEED},
};
#[test]
fn a_public_change_read_at_each_step_until_landed() {
    let mut world = public::world(1000);
    let mut row = changes::change_row();
    row.task = CHANGE;
    row.change.task = CHANGE;
    row.branch = Box::from(BRANCH);
    public::mark(&mut row);
    world.event(top::Event::Change { row });
    assert_eq!(public::change(&world).step(), &w::PublicStep::Producing, "seed {SEED}");
    public::ready(&mut world, CHANGE, BRANCH);
    assert_eq!(public::change(&world).step(), &w::PublicStep::Ready, "seed {SEED}");
    public::install_landing(&mut world, false);
    let mut finished = false;
    // The same bound as landings::drive_landing; each iteration reads both public kinds.
    for position in 0..80 {
        let decision = public::step(&mut world, position, false);
        let _change = public::change(&world);
        let _answer = public::answer(&world, w::PUBLIC_LANDING, Some(CHANGE), &public::handed());
        let _answer = public::answer(&world, w::PUBLIC_LANDINGS, None, &public::handed());
        if decision == policy::Decision::Finish {
            finished = true;
            break;
        }
    }
    assert!(finished, "seed {SEED}: landing did not finish");
    assert_eq!(public::change(&world).step(), &w::PublicStep::Landed, "seed {SEED}");
}
#[test]
fn a_two_member_landing_with_a_failed_gate_through_either_member() {
    let mut world = public::world(0);
    public::ready(&mut world, CHANGE, BRANCH);
    public::ready(&mut world, SECOND, OTHER_BRANCH);
    public::install_landing(&mut world, true);
    let mut failed = false;
    for position in 0..80 {
        let _decision = public::step(&mut world, position, true);
        let a = public::answer(&world, w::PUBLIC_LANDING, Some(CHANGE), &public::handed());
        let b = public::answer(&world, w::PUBLIC_LANDING, Some(SECOND), &public::handed());
        assert_eq!(a.tasks, b.tasks, "seed {SEED}");
        assert_eq!(a.record, b.record, "seed {SEED}");
        let record = w::PublicLanding::decode(&w::CEILINGS, &mut Reader::new(&a.record)).expect("landing");
        for member in record.members() {
            for gate in member.gates() {
                if gate.proof() == &w::PublicProof::Failed {
                    assert_eq!(gate.proof(), &w::PublicProof::Failed, "seed {SEED}: failed proof state");
                    failed = true;
                }
            }
        }
        if failed {
            break;
        }
    }
    assert!(failed, "seed {SEED}: no failed proof");
}
#[test]
fn a_private_project_change_an_outside_change_and_a_conversations_change_are_unknown() {
    let mut world = public::world(1000);
    public::ready(&mut world, CHANGE, BRANCH);
    for public in [&[][..], &public::handed()[..1], &public::handed()[2..]] {
        for kind in [w::PUBLIC_CHANGE, w::PUBLIC_LANDING] {
            assert!(world.public(landings::PROJECT, kind, Some(CHANGE), public).is_none(), "seed {SEED}");
        }
    }
}
#[test]
fn a_queued_change_outside_every_goal_is_counted_nowhere() {
    let mut world = public::world(1000);
    public::queued(&mut world);
    let visible = public::answer(&world, w::PUBLIC_LANDINGS, None, &public::handed());
    assert_eq!(visible.tasks.as_ref(), &[CHANGE, GOAL], "seed {SEED}: queued change and goal");
    let public = [top::PublicTask { task: CHANGE, goal: None }];
    let made = public::answer(&world, w::PUBLIC_LANDINGS, None, &public);
    assert!(made.tasks.is_empty(), "seed {SEED}");
    let record = w::PublicLandings::decode(&w::CEILINGS, &mut Reader::new(&made.record)).expect("landings");
    assert!(record.queues().is_empty(), "seed {SEED}");
}
#[test]
fn a_landing_with_a_member_not_public_is_absent_and_unknown() {
    let mut world = public::world(1000);
    public::ready(&mut world, CHANGE, BRANCH);
    public::ready(&mut world, SECOND, OTHER_BRANCH);
    public::install_landing(&mut world, true);
    let public = &public::handed()[..2];
    let made = public::answer(&world, w::PUBLIC_LANDINGS, None, public);
    let record = w::PublicLandings::decode(&w::CEILINGS, &mut Reader::new(&made.record)).expect("landings");
    assert!(record.landings().is_empty(), "seed {SEED}");
    for task in [CHANGE, SECOND] {
        assert!(world.public(landings::PROJECT, w::PUBLIC_LANDING, Some(task), public).is_none(), "seed {SEED}");
    }
    let made = public::answer(&world, w::PUBLIC_CHANGE, Some(CHANGE), public);
    let record = w::PublicChange::decode(&w::CEILINGS, &mut Reader::new(&made.record)).expect("change");
    assert!(!record.in_landing(), "seed {SEED}");
}
#[test]
fn a_task_jig_did_not_hand_is_never_named() {
    let mut world = public::world(1000);
    public::queued(&mut world);
    let visible = public::answer(&world, w::PUBLIC_LANDINGS, None, &public::handed());
    assert_eq!(visible.tasks.as_ref(), &[CHANGE, GOAL], "seed {SEED}: handed goal");
    let public = [top::PublicTask { task: CHANGE, goal: Some(GOAL) }];
    let made = public::answer(&world, w::PUBLIC_LANDINGS, None, &public);
    assert!(made.tasks.is_empty(), "seed {SEED}");
    assert!(world.public(landings::PROJECT, w::PUBLIC_CHANGE, Some(SECOND), &public).is_none(), "seed {SEED}");
    assert!(!made.tasks.contains(&LANDING), "seed {SEED}");
}
