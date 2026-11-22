//! Milestone reads and parts count the findings now, never cached reds (red-findings.md, 10).
use super::*;
use jig_domain_core::{documents as d, reads};
use skein_lib::{List, Reader};
use temper_client_protocol_wire::{self as wire, v1 as w};
use w::Record;

fn query(milestone: u64) -> d::TaskQuery {
    let mut query = jig_test_profile::fixtures::own_tasks();
    query.project = Some(1);
    query.requester = None;
    query.root = Some(milestone);
    query.label = Some(4);
    query.parts = List::with_capacity(1);
    query
        .parts
        .push(d::PartAsk { owner: wire::FORGE, kind: wire::MILESTONE_PART, version: wire::CLIENT_VERSION })
        .expect("one part");
    query
}

fn decode(line: &d::TaskLine) -> w::MilestonePart {
    let Some(d::RowPart::Made(bytes)) = line.parts.get(0) else { panic!("milestone part: {:?}", line.parts) };
    w::MilestonePart::decode(&w::CEILINGS, &mut Reader::new(bytes)).expect("milestone part")
}

fn part(world: &mut World, milestone: u64) -> w::MilestonePart {
    tiers::sign_in(world);
    let answer = world.party_read(reads::Read::Tasks { query: query(milestone), live: None, ended: None });
    let reads::Answer::Tasks(page) = answer else { panic!("milestone query: {answer:?}") };
    let ended = match &page.ended {
        reads::EndedGroup::Rows { rows, .. } => rows.iter().find(|line| line.chip.number == milestone),
        reads::EndedGroup::None => None,
    };
    decode(page.live.iter().find(|line| line.chip.number == milestone).or(ended).expect("milestone row"))
}

fn totals(world: &mut World, milestone: u64) -> forge_top::MilestoneTotals {
    tiers::sign_in(world);
    world.milestone_reads.clear();
    world.send(engine::Event::Party(engine::PartyInput::ForgeRead {
        reply: ReplyTo::new(Token::new(850)),
        sign_in: world.signed_in.expect("owner"),
        scope: people::ApplicationScope::Project(1),
        query: engine::ForgeRead::Milestone { project: 1, task: milestone },
    }));
    until(world, LANDING_TICKS, "milestone read", |world| !world.milestone_reads.is_empty());
    world.milestone_reads.last().expect("answered milestone read").clone().expect("milestone totals")
}

fn counts(world: &mut World, milestone: u64, reds: u32) {
    assert_eq!(totals(world, milestone).reds, reds, "Milestone read");
    assert_eq!(part(world, milestone).totals.reds, reds, "milestone part");
}

fn watch(world: &mut World, milestone: u64, goal: u64) {
    world.signed_in = None;
    world.sign_in();
    for (watcher, filter) in
        [(860, forge_top::FindingsFilter::Goal(goal)), (861, forge_top::FindingsFilter::Milestone(milestone))]
    {
        world.send(engine::Event::Party(engine::PartyInput::Watch {
            watcher: Token::new(watcher),
            sign_in: world.signed_in.expect("owner"),
            subject: d::Watched::Application {
                scope: people::ApplicationScope::Project(1),
                connector: 0,
                subject: Token::new(forge_top::findings_subject(1, filter).expect("bounded subject")),
            },
            window: d::Window { rows: 8, after: None },
        }));
    }
    world.send(engine::Event::Party(engine::PartyInput::Watch {
        watcher: Token::new(862),
        sign_in: world.signed_in.expect("owner"),
        subject: d::Watched::Query { query: query(milestone) },
        window: d::Window { rows: 8, after: None },
    }));
    world.until(Until::Ready);
    assert_eq!(observed(world), [1, 1, 1]);
    acknowledge(world);
}

fn acknowledge(world: &mut World) {
    for watcher in [860, 861, 862] {
        world.send(engine::Event::Party(engine::PartyInput::Delivered { watcher: Token::new(watcher) }));
    }
    world.until(Until::Ready);
}

fn observed(world: &World) -> [usize; 3] {
    [
        world.oracle_documents.iter().filter(|(watcher, _)| *watcher == Token::new(860)).count(),
        world.oracle_documents.iter().filter(|(watcher, _)| *watcher == Token::new(861)).count(),
        world.documents.iter().filter(|(watcher, _)| *watcher == Token::new(862)).count(),
    ]
}

#[test]
fn blamed_filing_routing_and_clearing_mark_filtered_watches_and_milestone_parts() {
    let (mut world, triager) =
        world_with_sessions(temper_engine_domain_forge_change::Wait::Landed, 4, Duration::from_secs(1200));
    tiers::sign_in(&mut world);
    let milestone = milestones::milestone(&mut world, b"Watched repairs");
    let goal = milestones::goal_with_words(&mut world, milestone, false, b"R4 Watch a red");
    // Establish the query watch after the landing changes the milestone's last landing.
    tiers::produce(&mut world, &goal, 94, b"failures", b"suite.alpha\n");
    until(&mut world, LANDING_TICKS, "landed before the verdict", |world| {
        tier(world).record.sequence == 1 && tier(world).record.run.is_some()
    });
    watch(&mut world, milestone, goal.task);
    filtered::judged(&mut world, 2);
    world.until(Until::Ready);
    let before = observed(&world);
    assert_eq!(before[0..2], [2, 2], "filing marks both filtered subjects");
    assert!(before[2] >= 2, "filing marks the milestone part");
    acknowledge(&mut world);
    let triager = world.assigned.iter().find(|run| run.task == triager).unwrap().clone();
    let number = findings(&world)[0].number;
    let answer = world.scripted_call(
        &triager,
        401,
        engine::Tool::Finding(engine::FindingCall::Triage { number, act: forge_top::Triage::Route(goal.task) }),
    );
    assert!(
        matches!(answer, engine::CallAnswer::Finding(answer) if matches!(answer.as_ref(), engine::FindingToolAnswer::Write(forge_top::FindingReply { result: Ok(_), .. })))
    );
    let routed = observed(&world);
    assert_eq!(routed, before.map(|n| n + 1), "routing marks the blame, not its repair goal");
    acknowledge(&mut world);
    let routed = observed(&world);
    acknowledge(&mut world);
    let answer = world.scripted_call(
        &triager,
        402,
        engine::Tool::Finding(engine::FindingCall::File {
            kind: forge_top::FindingKind::Qa,
            severity: forge_top::Severity::Low,
            area: None,
            title: b"No blame".as_slice().into(),
            body: b"Not a verdict".as_slice().into(),
        }),
    );
    assert!(
        matches!(answer, engine::CallAnswer::Finding(answer) if matches!(answer.as_ref(), engine::FindingToolAnswer::Write(forge_top::FindingReply { result: Ok(_), .. })))
    );
    assert_eq!(observed(&world), routed, "no blame marks none of these watches");
    fake::advance(&mut world.fake, &world.fake_env, b"org/repo", b"main", b"failures", b"", 1).unwrap();
    filtered::judged(&mut world, 3);
    world.until(Until::Ready);
    assert_eq!(observed(&world), routed.map(|n| n + 1), "judged clearing marks every blamed watch");
}

#[test]
fn a_complete_milestones_reds_follow_drops_and_clearing_in_reads_and_cached_parts() {
    let (mut world, triager) = world_with_wait(temper_engine_domain_forge_change::Wait::Landed);
    tiers::sign_in(&mut world);
    let milestone = milestones::milestone(&mut world, b"Observed repairs");
    let goal = milestones::goal_with_words(&mut world, milestone, false, b"R3 Observe a red");
    tiers::produce(&mut world, &goal, 94, b"failures", b"suite.alpha\n");
    filtered::judged(&mut world, 2);
    counts(&mut world, milestone, 1);
    let number = findings(&world)[0].number;
    let triager = world.assigned.iter().find(|run| run.task == triager).unwrap().clone();
    let answer = world.scripted_call(
        &triager,
        401,
        engine::Tool::Finding(engine::FindingCall::Triage {
            number,
            act: forge_top::Triage::Drop(b"Not pursuing".as_slice().into()),
        }),
    );
    assert!(
        matches!(answer, engine::CallAnswer::Finding(answer) if matches!(answer.as_ref(), engine::FindingToolAnswer::Write(forge_top::FindingReply { result: Ok(_), .. })))
    );
    counts(&mut world, milestone, 1);
    goal_landing::finish_goal(&mut world, &goal);
    milestones::ask(&mut world, 182, people::Ask::Conclude { project: 1, task: milestone });
    let ended = milestones::stored(&world, milestone).ended.clone().expect("milestone ended");
    assert_eq!(ended.outcome, forge_top::MilestoneOutcome::Complete);
    assert_eq!((ended.done, ended.failed), (1, 0));
    counts(&mut world, milestone, 1);
    // Without the forge row, parts fall back to the ended task's cached counts.
    world.store.seed(Box::new([]), Box::new([Key::Forge(forge_top::Key::Milestone(milestone))]));
    world.restart_same();
    world.until(Until::Ready);
    tiers::sign_in(&mut world);
    assert_eq!(part(&mut world, milestone).totals.reds, 1);
    assert_eq!(part(&mut world, milestone).totals.reds, 1);
    fake::advance(&mut world.fake, &world.fake_env, b"org/repo", b"main", b"failures", b"", 1).unwrap();
    filtered::judged(&mut world, 3);
    assert_eq!(part(&mut world, milestone).totals.reds, 0, "cached ended totals are recounted");
    world.send(engine::Event::Forge(Box::new(forge_top::Event::Milestone {
        row: forge_top::MilestoneRow {
            task: milestone,
            project: 1,
            name: b"Observed repairs".as_slice().into(),
            last_landing: None,
            ended: Some(ended.clone()),
        },
    })));
    world.until(Until::Ready);
    counts(&mut world, milestone, 0);
    assert_eq!(milestones::stored(&world, milestone).ended, Some(ended));
}
