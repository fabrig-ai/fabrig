//! Visitor stories against the independent forge (temper/docs/design/client/forge/README.md, 9.5).
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
    public::install_landing(&mut world);
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
fn a_one_member_landing_with_a_failed_gate_through_its_change() {
    let mut world = public::world(0);
    public::ready(&mut world, CHANGE, BRANCH);
    public::ready(&mut world, SECOND, OTHER_BRANCH);
    public::install_landing(&mut world);
    let mut failed = false;
    for position in 0..80 {
        let _decision = public::step(&mut world, position, false);
        let a = public::answer(&world, w::PUBLIC_LANDING, Some(CHANGE), &public::handed());
        assert!(world.public(landings::PROJECT, w::PUBLIC_LANDING, Some(SECOND), &public::handed()).is_none());
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
    public::install_landing(&mut world);
    let public = &public::handed()[2..];
    let made = public::answer(&world, w::PUBLIC_LANDINGS, None, public);
    let record = w::PublicLandings::decode(&w::CEILINGS, &mut Reader::new(&made.record)).expect("landings");
    assert!(record.landings().is_empty(), "seed {SEED}");
    for task in [CHANGE, SECOND] {
        assert!(world.public(landings::PROJECT, w::PUBLIC_LANDING, Some(task), public).is_none(), "seed {SEED}");
    }
    assert!(world.public(landings::PROJECT, w::PUBLIC_CHANGE, Some(CHANGE), public).is_none());
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

#[test]
fn a_queue_merge_read_through_a_real_merge() {
    let world = public::queue_merge();
    let public = [top::PublicTask { task: CHANGE, goal: None }];
    let change = public::change(&world);
    assert_eq!(change.step(), &w::PublicStep::Landed, "seed {SEED}");
    assert_eq!(change.way(), &w::PublicWay::Queue, "seed {SEED}");
    assert!(!change.in_landing(), "seed {SEED}");
    let commit = change.merge().expect("queue merge");
    let made = public::answer(&world, w::PUBLIC_LANDINGS, None, &public);
    assert_eq!(made.tasks.as_ref(), &[CHANGE], "seed {SEED}");
    let lines = w::PublicLandings::decode(&w::CEILINGS, &mut Reader::new(&made.record)).expect("landings");
    assert_eq!(lines.turn(), &None, "seed {SEED}");
    assert_eq!(lines.landings().len(), 1, "seed {SEED}");
    let line = lines.landings().get(0).expect("queue line");
    assert_eq!(line.phase(), &w::PublicLandingPhase::Landed, "seed {SEED}");
    assert_eq!(line.members().as_slice(), &[0], "seed {SEED}");
    assert_eq!(line.landed().as_slice(), &[commit], "seed {SEED}");
    let made = public::answer(&world, w::PUBLIC_LANDING, Some(CHANGE), &public);
    assert_eq!(made.tasks.as_ref(), &[CHANGE], "seed {SEED}");
    let landing = w::PublicLanding::decode(&w::CEILINGS, &mut Reader::new(&made.record)).expect("landing");
    assert_eq!(landing.phase(), &w::PublicLandingPhase::Landed, "seed {SEED}");
    assert_eq!(landing.members().len(), 1, "seed {SEED}");
    let member = landing.members().get(0).expect("queue member");
    assert_eq!(member.task(), 0, "seed {SEED}");
    assert_eq!(member.landed(), &Some(commit), "seed {SEED}");
    let made = public::answer(&world, w::PUBLIC_LANDINGS, None, &[]);
    assert!(made.tasks.is_empty(), "seed {SEED}");
    let lines = w::PublicLandings::decode(&w::CEILINGS, &mut Reader::new(&made.record)).expect("private history");
    assert!(lines.landings().is_empty(), "seed {SEED}");
    assert!(world.public(landings::PROJECT, w::PUBLIC_LANDING, Some(CHANGE), &[]).is_none(), "seed {SEED}");
}
