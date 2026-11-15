//! Four units, a red third, and its later repaired landing (domain/landing.md, 16;
//! domain/landing-search.md, 3–7, 10).
use skein_fake_forge_domain::api as raw;
use skein_forge::api;
use temper_engine_domain_forge as top;
use temper_engine_domain_forge_change as change;
use temper_engine_domain_forge_landing as landing;
use temper_engine_forge_world::{
    World, changes,
    landings::{self, LANDING, Trains},
    translate,
};

fn step_members(world: &mut World, entry: u64) {
    for task in 51..=54 {
        if !matches!(world.change(task).change.state, change::State::Repairing { requested: true, .. }) {
            changes::change_step_task(
                world,
                task,
                entry * 10 + task,
                change::Status::Passed,
                Box::new([]),
                false,
                false,
            );
        }
    }
}

fn assert_landed(world: &World, member: &landing::Member, prefix: api::Commit) {
    assert_eq!(member.landed, Some(prefix));
    assert!(member.told);
    assert!(matches!(world.change(member.change).change.state, change::State::Landed { merge, .. } if merge == prefix));
    let raw::Answer::Pull(pull) = world.inspect(&raw::Read::Pull { number: member.pull }) else {
        panic!("member pull")
    };
    assert_eq!(pull.state, raw::State::Closed);
    assert_eq!(pull.merged.map(translate::commit), Some(prefix));
}

fn assert_parent(world: &World, prefix: api::Commit, below: api::Commit) {
    let raw::Answer::Commits { commits } = world.inspect(&raw::Read::Commits {
        before: skein_fake_forge_domain::commit_count(&below).expect("fake base"),
        after: skein_fake_forge_domain::commit_count(&prefix).expect("fake prefix"),
    }) else {
        panic!("prefix parents")
    };
    let commit = commits.iter().find(|commit| translate::commit(commit.id) == prefix).expect("prefix commit");
    assert_eq!(commit.parents.first().copied().map(translate::commit), Some(below));
}

fn assert_run(world: &World, prefix: api::Commit, conclusion: raw::Conclusion) {
    let raw::Answer::Runs(runs) = world.inspect(&raw::Read::Runs {
        workflow: Some(b"gate.yml".as_slice().into()),
        commit: Some(skein_fake_forge_domain::commit_count(&prefix).expect("fake prefix")),
        number: None,
        page: 1,
        limit: 8,
    }) else {
        panic!("prefix run")
    };
    assert_eq!(runs.len(), 1, "one run at the prefix");
    assert_eq!(runs[0].conclusion, Some(conclusion));
}

#[test]
#[expect(clippy::too_many_lines, reason = "one scenario follows the blamed unit through search, restart and repair")]
fn a_red_third_of_four_is_narrowed_across_restart_and_its_repair_lands_later() {
    let mut world = landings::turn_in(World::search_turn(184, super::landing_batches::workflow(true)));
    world.batch(4);
    world.push_file(b"main", b"file", b"fixed");
    let base = translate::commit(world.branch(b"main"));
    for task in 51..=54 {
        let branch = format!("temper/c{task}");
        assert_eq!(
            world.external(
                raw::Op::Git(raw::Git::Create { branch: branch.as_bytes().into(), commit: world.branch(b"main") }),
                1,
            ),
            raw::Answer::Branch(raw::Created::Created)
        );
        world.push_file(branch.as_bytes(), format!("safe{task}").as_bytes(), b"safe");
        if task == 53 {
            world.push_file(branch.as_bytes(), b"file", b"broken");
        }
        landings::ready_change(&mut world, task, branch.as_bytes());
        assert_eq!(world.change(task).change.landing, None);
    }
    let ready_since = world.change(53).change.ready_since;
    let mut trains = Trains::default();
    trains.judge(&mut world);
    assert_eq!(
        world.landing(LANDING).landing.members.iter().map(|member| (member.change, member.unit)).collect::<Vec<_>>(),
        [(51, 0), (52, 1), (53, 2), (54, 3)]
    );

    let mut prefixes = Vec::new();
    let mut probes = Vec::new();
    let mut first_pull = None;
    let mut green_pull = None;
    let mut restarted = false;
    let mut reached_second = false;
    for entry in 1000..1200 {
        let decision = landings::step_landing(&mut world, LANDING, entry);
        step_members(&mut world, entry);
        let row = &world.landing(LANDING).landing;
        if let landing::Decision::Dispatch { member, .. } = decision {
            if row.round.search.is_some() {
                probes.push(member + 1);
                if !restarted {
                    assert_eq!(probes, [2], "four units first probe boundary 2");
                    world.restart();
                    restarted = true;
                }
            } else {
                assert_eq!(member, 3, "the initial proof is at the top");
                prefixes = row.members.iter().map(|member| member.candidate.expect("round one prefix")).collect();
                first_pull = row.round.pull;
            }
        }
        if matches!(decision, landing::Decision::Merge { .. }) {
            assert_eq!(probes, [2, 3], "green 2 then red 3 bound the blamed third");
            let row = &world.landing(LANDING).landing;
            assert_eq!(row.preparation, 1);
            green_pull = row.round.pull;
            assert_ne!(green_pull, first_pull);
            assert_eq!(translate::commit(world.branch(b"main")), prefixes[1]);
        }
        if world.landing(LANDING).landing.preparation == 2 {
            reached_second = true;
            break;
        }
    }
    assert!(restarted && reached_second, "restart during probing and finish round one's turn");
    assert_eq!(probes, [2, 3]);
    assert_run(&world, prefixes[3], raw::Conclusion::Failure);
    assert_run(&world, prefixes[1], raw::Conclusion::Success);
    assert_run(&world, prefixes[2], raw::Conclusion::Failure);
    let row = world.landing(LANDING).clone();
    for (member, prefix) in row.landing.members.iter().zip(&prefixes).take(2) {
        assert_landed(&world, member, *prefix);
    }
    assert_eq!(row.landing.members[3].landed, None, "fourth has not landed in round one");
    let Some(landing::Leave::Blamed { gate, run, red, green }) = row.landing.members[2].left else {
        panic!("third blamed")
    };
    assert_eq!((gate, red, green), (0, prefixes[2], prefixes[1]));
    assert_eq!(row.landing.members[2].landed, None);
    assert_eq!(world.change(53).change.repairs, 1);
    assert_eq!(
        world.change(53).change.state,
        change::State::Repairing {
            requested: true,
            why: change::Repair::LandingAt {
                gate,
                candidate: red,
                below: Some(change::Below {
                    green,
                    changes: [
                        Some(change::Landed { change: 51, pull: row.landing.members[0].pull }),
                        Some(change::Landed { change: 52, pull: row.landing.members[1].pull }),
                        None,
                        None,
                    ],
                }),
            },
        }
    );
    let head = world.change(53).change.last_head.expect("blamed head");
    let brief = String::from_utf8(world.ci_brief(53, head).into_vec()).expect("CI brief");
    for words in [
        format!("Candidate: {}", String::from_utf8_lossy(&top::commit_text(&red))),
        format!("Base: {}", String::from_utf8_lossy(&top::commit_text(&base))),
        format!("Run: http://forge/org/repo/actions/runs/{run}"),
        "Gate: gate".to_owned(),
        "Failed job: test".to_owned(),
        "Failed step: ".to_owned(),
        "End of its log:".to_owned(),
    ] {
        assert!(brief.contains(&words), "missing {words}: {brief}");
    }

    landings::drive_landing(&mut world, LANDING);
    let row = world.landing(LANDING).clone();
    assert_eq!(row.landing.phase, landing::Phase::Landed);
    assert_eq!(row.landing.preparation, 2);
    let fourth = &row.landing.members[3];
    let fourth_prefix = fourth.candidate.expect("restacked fourth");
    assert_ne!(fourth_prefix, prefixes[3]);
    assert_eq!(fourth.base, Some(green));
    assert_parent(&world, fourth_prefix, green);
    assert_run(&world, fourth_prefix, raw::Conclusion::Success);
    assert_landed(&world, fourth, fourth_prefix);
    assert_eq!(translate::commit(world.branch(b"main")), fourth_prefix);
    let second_pull = row.landing.round.pull.expect("second round pull");
    landings::close_landing(&mut world, LANDING);

    // The worker delivers on the blamed change's own branch; its procedure, not
    // a replacement ChangeRow, brings that repaired head back to Ready.
    world.push(b"temper/c53", b"fixed");
    let repaired_head = translate::commit(world.branch(b"temper/c53"));
    assert_ne!(repaired_head, head);
    for entry in 3000..3010 {
        changes::change_step_task(&mut world, 53, entry, change::Status::Passed, Box::new([]), false, false);
        if matches!(world.change(53).change.state, change::State::Ready { .. }) {
            break;
        }
    }
    assert!(matches!(world.change(53).change.state, change::State::Ready { head, .. } if head == repaired_head));
    assert_eq!(world.change(53).change.ready_since, ready_since);
    trains.judge(&mut world);
    let repaired_landing = world.change(53).change.landing.expect("later landing formed");
    assert_ne!(repaired_landing, LANDING);
    assert_eq!(world.landing(repaired_landing).landing.members.len(), 1);
    landings::drive_landing(&mut world, repaired_landing);
    let repaired = world.landing(repaired_landing).clone();
    assert_eq!(repaired.landing.phase, landing::Phase::Landed);
    let member = &repaired.landing.members[0];
    let repaired_prefix = member.candidate.expect("repaired prefix");
    assert_eq!(member.head, Some(repaired_head));
    assert_eq!(member.base, Some(fourth_prefix));
    assert_parent(&world, repaired_prefix, fourth_prefix);
    assert_run(&world, repaired_prefix, raw::Conclusion::Success);
    assert_landed(&world, member, repaired_prefix);
    assert_eq!(translate::commit(world.branch(b"main")), repaired_prefix);
    assert!(trains.held.is_empty(), "no train held: {:?}", trains.held);
    assert_eq!(world.landing_hold_words(LANDING), None);
    assert_eq!(world.landing_hold_words(repaired_landing), None);

    let calls = world.take_calls();
    let dispatches: Vec<_> = calls
        .iter()
        .filter_map(|op| {
            if let api::Op::Write(api::Write::Dispatch { branch, inputs, .. }) = op {
                Some((
                    branch.clone(),
                    inputs
                        .iter()
                        .find(|input| input.name.as_ref() == b"temper_commit")
                        .expect("dispatched commit")
                        .value
                        .clone(),
                ))
            } else {
                None
            }
        })
        .collect();
    assert_eq!(
        dispatches,
        [
            (row.landing.members[3].branch.clone(), top::commit_text(&prefixes[3])),
            (row.landing.members[1].branch.clone(), top::commit_text(&green)),
            (row.landing.members[2].branch.clone(), top::commit_text(&red)),
            (fourth.branch.clone(), top::commit_text(&fourth_prefix)),
            (member.branch.clone(), top::commit_text(&repaired_prefix)),
        ],
        "exactly five dispatches: top, probes 2 and 3, round two, repair"
    );
    let opens: Vec<_> =
        calls
            .iter()
            .filter_map(|op| {
                if let api::Op::Write(api::Write::OpenPull { head, .. }) = op { Some(head.as_ref()) } else { None }
            })
            .collect();
    assert_eq!(
        opens,
        [
            row.landing.members[3].branch.as_ref(),
            row.landing.members[1].branch.as_ref(),
            fourth.branch.as_ref(),
            member.branch.as_ref(),
        ],
        "one opening per round pull, including the cut's green pull"
    );
    let merges: Vec<_> = calls
        .iter()
        .filter_map(|op| {
            if let api::Op::Write(api::Write::Merge { number, head, how }) = op {
                Some((*number, *head, *how))
            } else {
                None
            }
        })
        .collect();
    assert_eq!(
        merges,
        [
            (green_pull.expect("green pull"), green, api::MergeHow::FastForwardOnly),
            (second_pull, fourth_prefix, api::MergeHow::FastForwardOnly),
            (repaired.landing.round.pull.expect("repair pull"), repaired_prefix, api::MergeHow::FastForwardOnly),
        ],
        "one fast-forward per turn, none repeated after restart"
    );
    let raw::Answer::Pull(pull) = world.inspect(&raw::Read::Pull { number: first_pull.expect("initial top pull") })
    else {
        panic!("cut round pull")
    };
    assert_eq!((pull.state, pull.merged), (raw::State::Closed, None));
    landings::close_landing(&mut world, repaired_landing);
}
