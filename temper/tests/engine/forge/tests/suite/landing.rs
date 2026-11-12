//! A landing of one and its documentation successor (domain/landing.md, 16).
use skein_fake_forge_domain::api as raw;
use skein_forge::api;
use temper_engine_domain_forge as top;
use temper_engine_domain_forge_change as change;
use temper_engine_domain_forge_landing as landing;
use temper_engine_forge_world::{
    REPO, World,
    landings::{self, CHANGE, LANDING},
    translate,
};

const DOC_CHANGE: u64 = 52;
const DOC_LANDING: u64 = 62;
const NEXT_CHANGE: u64 = 53;
const NEXT_LANDING: u64 = 63;

fn runs(world: &World) -> Box<[raw::Run]> {
    let raw::Answer::Runs(runs) = world.inspect(&raw::Read::Runs {
        workflow: Some(Box::from(b"gate.yml".as_slice())),
        commit: None,
        number: None,
        page: 1,
        limit: 16,
    }) else {
        panic!("workflow runs")
    };
    runs
}
fn receipt(world: &World) -> (&top::ReceiptKey, &top::Receipt) {
    world
        .stored()
        .iter()
        .find_map(|(_, row)| if let top::Stored::Receipt { key, receipt } = row { Some((key, receipt)) } else { None })
        .expect("saved receipt")
}
fn documents(world: &World) -> [top::Document; 3] {
    [LANDING, top::landings_subject(landings::PROJECT, true), CHANGE]
        .map(|subject| world.document(landings::PROJECT, subject).expect("settled document"))
}
fn hex(bytes: &[u8]) -> Box<[u8]> {
    bytes.iter().flat_map(|byte| top::hex(*byte)).collect()
}
#[test]
fn a_landing_of_one_is_prepared_on_its_branch_proved_and_fast_forwarded() {
    let mut world = landings::first_landing();
    let candidate = world.landing(LANDING).landing.members[0].candidate.expect("candidate");
    let row = world.change(CHANGE).clone();
    assert!(matches!(row.change.state, change::State::Landed { merge, .. } if merge == candidate));
    assert_eq!(translate::commit(world.branch(b"main")), candidate);
    assert_eq!(world.landing(LANDING).landing.phase, landing::Phase::Landed);
    let raw::Answer::Pull(pull) = world.inspect(&raw::Read::Pull { number: row.pull.expect("opened pull") }) else {
        panic!("pull")
    };
    assert_eq!(pull.merged, Some(world.branch(b"main")));
    let (key, receipt) = receipt(&world);
    assert_eq!(key.repository, REPO);
    assert!(receipt.passed);
    assert_eq!(receipt.candidate, candidate);
    let runs = runs(&world);
    assert_eq!(runs.len(), 1);
    assert_eq!(
        runs[0].inputs.as_ref(),
        &[
            (Box::from(b"temper_commit".as_slice()), hex(&candidate[..20])),
            (Box::from(b"temper_key".as_slice()), hex(&key.key))
        ]
    );
    let link = receipt.run.url.clone();
    let short = String::from_utf8(hex(&key.key[..8]).into_vec()).expect("hex");
    let writes: Vec<_> = world
        .take_calls()
        .into_iter()
        .filter_map(|op| match op {
            api::Op::Write(write) => Some(write),
            api::Op::Read(_) => None,
        })
        .collect();
    assert_eq!(writes.len(), 5);
    let round = &world.landing(LANDING).landing.round;
    let title = format!("Land #{}: {}", row.pull.expect("member pull"), String::from_utf8_lossy(&row.title));
    assert!(
        matches!(&writes[0], api::Write::OpenPull { title: actual, head, base, .. } if actual.as_ref() == title.as_bytes() && *head == round.branch && base.as_ref() == b"main")
    );
    assert!(matches!(&writes[1], api::Write::Dispatch { branch, .. } if *branch == round.branch));
    assert_ne!(translate::commit(world.branch(b"temper/51")), candidate, "the member's branch is untouched");
    for (index, check, description, expected_link) in [
        (2, api::Check::Pending, format!("key {short}"), Box::<[u8]>::from([])),
        (3, api::Check::Passed, format!("key {short} passed"), link),
    ] {
        assert_eq!(
            writes[index],
            api::Write::Status {
                commit: candidate,
                context: Box::from(b"temper/gate/gate".as_slice()),
                check,
                description: description.into_bytes().into_boxed_slice(),
                link: expected_link
            }
        );
    }
    assert!(
        matches!(writes[4], api::Write::Merge { number, head, how: api::MergeHow::FastForwardOnly } if head == candidate && Some(number) == round.pull)
    );
}
#[test]
fn a_documentation_landing_lands_on_its_receipt_with_no_run() {
    let mut world = landings::first_landing();
    let first_key = receipt(&world).0.key;
    let first_link = receipt(&world).1.run.url.clone();
    world.take_calls();
    landings::documentation_landing(&mut world, DOC_CHANGE, DOC_LANDING, b"temper/52");
    let candidate = world.landing(DOC_LANDING).landing.members[0].candidate.expect("candidate");
    assert!(matches!(world.change(DOC_CHANGE).change.state, change::State::Landed { merge, .. } if merge == candidate));
    assert_eq!(translate::commit(world.branch(b"main")), candidate);
    assert_eq!(world.landing(DOC_LANDING).landing.members[0].gates[0].proof, landing::Proof::Passed { key: first_key });
    assert_eq!(runs(&world).len(), 1);
    let writes: Vec<_> = world
        .take_calls()
        .into_iter()
        .filter_map(|op| match op {
            api::Op::Write(write) => Some(write),
            api::Op::Read(_) => None,
        })
        .collect();
    assert_eq!(writes.len(), 3);
    assert!(matches!(writes[0], api::Write::OpenPull { .. }));
    let short = String::from_utf8(hex(&first_key[..8]).into_vec()).expect("hex");
    assert_eq!(
        writes[1],
        api::Write::Status {
            commit: candidate,
            context: Box::from(b"temper/gate/gate".as_slice()),
            check: api::Check::Passed,
            description: format!("key {short} passed by receipt").into_bytes().into_boxed_slice(),
            link: first_link
        }
    );
    assert!(
        matches!(writes[2], api::Write::Merge { head, how: api::MergeHow::FastForwardOnly, .. } if head == candidate)
    );
}

#[test]
fn a_restart_finds_the_dispatched_run_without_dispatching_again() {
    const BEFORE_DISPATCH_STEPS: u64 = 20;
    let mut world = landings::turn_world();
    world.produce(b"temper/51");
    world.push_file(b"main", b"base", b"main moved");
    landings::ready_change(&mut world, CHANGE, b"temper/51");
    let row = landings::landing_row(world.change(CHANGE));
    world.event(top::Event::Landing { row });
    let mut dispatched = false;
    for position in 0..BEFORE_DISPATCH_STEPS {
        let entry = 100 + position * 2;
        let decision = landings::step_landing(&mut world, LANDING, entry);
        temper_engine_forge_world::changes::change_step_task(
            &mut world,
            CHANGE,
            entry + 1,
            change::Status::Passed,
            Box::new([]),
            false,
            false,
        );
        if matches!(decision, landing::Decision::Dispatch { .. }) {
            dispatched = true;
            break;
        }
    }
    assert!(dispatched);
    world.restart();
    world.run_for(1);
    landings::drive_landing(&mut world, LANDING);
    assert_eq!(runs(&world).len(), 1);
    assert_eq!(world.landing(LANDING).landing.phase, landing::Phase::Landed);
    world.run_for(1);
    let landing = world.landing(LANDING).clone();
    let before = documents(&world);
    let saved = (receipt(&world).0.clone(), receipt(&world).1.clone());
    let main = world.branch(b"main");
    let count = runs(&world).len();
    world.take_calls();
    world.restart();
    world.run_for(1);
    assert_eq!(world.landing(LANDING), &landing);
    assert_eq!(documents(&world), before, "settled cold documents are unchanged");
    assert_eq!(receipt(&world), (&saved.0, &saved.1));
    assert_eq!(world.branch(b"main"), main);
    assert_eq!(runs(&world).len(), count);
    assert!(
        world
            .take_calls()
            .iter()
            .all(|op| !matches!(op, api::Op::Write(api::Write::Dispatch { .. } | api::Write::Merge { .. }))),
        "settled restart dispatches and merges nothing"
    );
}

#[test]
fn a_finished_landing_surrenders_its_live_slot_to_the_next() {
    let mut world = landings::first_landing();
    landings::documentation_landing(&mut world, DOC_CHANGE, DOC_LANDING, b"temper/52");
    landings::documentation_landing(&mut world, NEXT_CHANGE, NEXT_LANDING, b"temper/53");
    assert_eq!(world.landing(NEXT_LANDING).landing.phase, landing::Phase::Landed);
    assert!(!world.stored().contains_key(&top::Key::Landing(LANDING)));
}

#[test]
fn a_failed_dispatch_still_expires_after_restart() {
    let mut world = landings::turn_world();
    assert!(world.project_landing(
        landings::PROJECT,
        change::Way::Turn,
        Box::new([top::RepositoryGates {
            tiers: Box::new([]),
            repository: REPO,
            gates: Box::new([landing::Gate {
                name: Box::from(b"gate".as_slice()),
                workflow: Box::from(b"missing.yml".as_slice()),
                ignored: Box::new([]),
                blocking: true,
                base: false,
            }]),
        }]),
    ));
    world.produce(b"temper/51");
    landings::ready_change(&mut world, CHANGE, b"temper/51");
    let row = landings::landing_row(world.change(CHANGE));
    world.event(top::Event::Landing { row });
    let mut dispatched = false;
    for position in 0..20 {
        let entry = 100 + position * 2;
        let decision = landings::step_landing(&mut world, LANDING, entry);
        temper_engine_forge_world::changes::change_step_task(
            &mut world,
            CHANGE,
            entry + 1,
            change::Status::Passed,
            Box::new([]),
            false,
            false,
        );
        if matches!(decision, landing::Decision::Dispatch { .. }) {
            dispatched = true;
            break;
        }
    }
    assert!(dispatched);
    world.restart();
    world.run_for(11);
    let mut held = false;
    for position in 0..20 {
        if matches!(landings::step_landing(&mut world, LANDING, 1000 + position), landing::Decision::Hold(_)) {
            held = true;
            break;
        }
    }
    assert!(held, "failed dispatch must reach its bounded retry limit after restart");
}

#[test]
fn a_landing_is_read_through_its_watch_while_it_proves_and_after_it_lands() {
    // The scripted root asks the same document function as the engine's watch route.
    let mut world = landings::turn_world();
    world.produce(b"temper/51");
    world.push_file(b"main", b"base", b"main moved");
    landings::ready_change(&mut world, CHANGE, b"temper/51");
    let row = landings::landing_row(world.change(CHANGE));
    world.event(top::Event::Landing { row });
    let mut proving = false;
    for position in 0..20 {
        let entry = 100 + position * 2;
        let decision = landings::step_landing(&mut world, LANDING, entry);
        temper_engine_forge_world::changes::change_step_task(
            &mut world,
            CHANGE,
            entry + 1,
            change::Status::Passed,
            Box::new([]),
            false,
            false,
        );
        if matches!(decision, landing::Decision::Dispatch { .. }) {
            let top::Document::Landing(view) = world.document(landings::PROJECT, LANDING).expect("landing watch")
            else {
                panic!("landing document")
            };
            assert_eq!((view.task, view.phase, view.turn), (LANDING, landing::Phase::Proving, None));
            let member = &view.members[0];
            assert_eq!(member.change, CHANGE);
            assert_eq!(member.candidate, Some(world.landing(LANDING).landing.members[0].candidate.expect("candidate")));
            assert!(member.base.is_some());
            assert!(matches!(member.gates[0].proof, top::GateProof::Dispatched { attempt: 1, .. }));
            assert!(member.gates[0].key.is_some());
            proving = true;
            break;
        }
    }
    assert!(proving, "the watch saw its dispatched gate before completion");
    landings::drive_landing(&mut world, LANDING);
    let top::Document::Landing(view) = world.document(landings::PROJECT, LANDING).expect("ended landing watch") else {
        panic!("landing document")
    };
    assert_eq!(view.phase, landing::Phase::Landed);
    let member = &view.members[0];
    assert_eq!(member.landed, member.candidate);
    assert_eq!(member.landed, Some(translate::commit(world.branch(b"main"))));
    let top::GateProof::Passed(run) = &member.gates[0].proof else { panic!("receipt run") };
    assert_eq!(run, &receipt(&world).1.run);
    for ended in [false, true] {
        let top::Document::Landings(view) = world
            .document(landings::PROJECT, top::landings_subject(landings::PROJECT, ended))
            .expect("project list watch")
        else {
            panic!("list document")
        };
        assert_eq!(view.turn, None);
        assert_eq!(view.landings.len(), usize::from(ended));
        if ended {
            assert_eq!(view.landings[0].task, LANDING);
        }
    }
}

#[test]
fn a_moving_base_reruns_only_the_gate_that_reads_it() {
    let mut world = landings::turn_world();
    let gate = landing::Gate {
        name: b"gate".as_slice().into(),
        workflow: b"gate.yml".as_slice().into(),
        ignored: Box::new([b"docs".as_slice().into()]),
        blocking: true,
        base: false,
    };
    assert!(world.project_landing(
        landings::PROJECT,
        change::Way::Turn,
        Box::new([top::RepositoryGates {
            repository: REPO,
            gates: Box::new([gate.clone(), landing::Gate { name: b"base".as_slice().into(), base: true, ..gate }]),
            tiers: Box::new([]),
        }])
    ));
    world.produce(b"temper/51");
    let base = translate::commit(world.branch(b"main"));
    landings::ready_change(&mut world, CHANGE, b"temper/51");
    let mut row = landings::landing_row(world.change(CHANGE));
    let proof = row.landing.members[0].gates[0];
    row.landing.members[0].gates = Box::new([proof, proof]);
    world.event(top::Event::Landing { row });
    for position in 0..40 {
        let entry = 100 + position * 2;
        landings::step_landing(&mut world, LANDING, entry);
        temper_engine_forge_world::changes::change_step_task(
            &mut world,
            CHANGE,
            entry + 1,
            change::Status::Passed,
            Box::new([]),
            false,
            false,
        );
        if world.landing(LANDING).landing.members[0]
            .gates
            .iter()
            .all(|gate| matches!(gate.proof, landing::Proof::Passed { .. }))
        {
            break;
        }
    }
    let first = world.landing(LANDING).landing.members[0].gates.clone();
    assert!(first.iter().all(|gate| matches!(gate.proof, landing::Proof::Passed { .. })));
    let initial = runs(&world);
    assert_eq!(initial.len(), 2);
    assert_eq!(
        initial
            .iter()
            .filter(|run| run
                .inputs
                .iter()
                .any(|(name, value)| name.as_ref() == b"temper_base" && *value == hex(&base[..20])))
            .count(),
        1
    );
    assert_eq!(initial.iter().filter(|run| run.inputs.len() == 2).count(), 1);
    // Main takes the member's content independently: preparing it again changes
    // its ancestry and base, but not its filtered candidate tree.
    world.push_file(b"main", b"file", b"change");
    let next_base = translate::commit(world.branch(b"main"));
    assert_ne!(base, next_base);
    world.restart();
    landings::drive_landing(&mut world, LANDING);
    let member = &world.landing(LANDING).landing.members[0];
    assert_eq!(world.landing(LANDING).landing.phase, landing::Phase::Landed);
    assert_eq!(member.base, Some(next_base));
    assert_eq!(member.gates[0].proof, first[0].proof, "the non-base receipt holds");
    assert_ne!(member.gates[1].proof, first[1].proof, "the base key moves");
    let final_runs = runs(&world);
    let calls = world.take_calls();
    assert_eq!(world.landing(LANDING).landing.stale_proofs, 1);
    assert_eq!(world.landing(LANDING).landing.preparation, 2);
    let close =
        calls.iter().position(|op| matches!(op, api::Op::Write(api::Write::Close { .. }))).expect("round closed");
    let delete = calls
        .iter()
        .position(|op| matches!(op, api::Op::Write(api::Write::DeleteBranch { .. })))
        .expect("branch deleted");
    let open = calls
        .iter()
        .enumerate()
        .filter_map(|(at, op)| matches!(op, api::Op::Write(api::Write::OpenPull { .. })).then_some(at))
        .nth(1)
        .expect("next round opened");
    assert!(close < delete && delete < open, "the old round ends before the next prepare");
    assert_eq!(calls.iter().filter(|op| matches!(op, api::Op::Write(api::Write::Dispatch { .. }))).count(), 3);
    assert!(final_runs.iter().any(|run| !initial.iter().any(|old| old.id == run.id)), "a new run was made");
    assert!(final_runs.iter().any(|run| {
        run.inputs.iter().any(|(name, value)| name.as_ref() == b"temper_base" && *value == hex(&next_base[..20]))
    }));
}
