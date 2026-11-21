//! Findings blamed on a goal or milestone (domain/red-findings.md, 10).
use super::*;
use forge_top::{FindingState, FindingsFilter};

fn document(world: &mut World, filter: FindingsFilter) -> forge_top::Findings {
    tiers::sign_in(world);
    let watcher = Token::new(world.env.now.as_nanos());
    world.oracle_documents.retain(|(got, _)| *got != watcher);
    world.send(engine::Event::Party(engine::PartyInput::Watch {
        watcher,
        sign_in: world.signed_in.expect("signed in"),
        subject: jig_domain_core::documents::Watched::Application {
            scope: people::ApplicationScope::Project(1),
            connector: 0,
            subject: Token::new(forge_top::findings_subject(1, filter).expect("bounded filter")),
        },
        window: jig_domain_core::documents::Window { rows: 128, after: None },
    }));
    until(world, LANDING_TICKS, "filtered findings document", |world| {
        world.oracle_documents.iter().any(|(got, _)| *got == watcher)
    });
    let (_, forge_top::Document::Findings(document)) =
        world.oracle_documents.iter().find(|(got, _)| *got == watcher).expect("delivered findings")
    else {
        panic!("findings document")
    };
    let document = document.clone();
    world.send(engine::Event::Party(engine::PartyInput::Unwatch { watcher }));
    world.until(Until::Ready);
    document
}

fn numbers(document: &forge_top::Findings) -> Vec<u32> {
    assert!(!document.cut);
    document.rows.iter().map(|row| row.number).collect()
}

pub(super) fn judged(world: &mut World, sequence: u64) {
    until(world, RUN_TICKS * 3, "landing verdict filed", |world| {
        tier(world).record.sequence == sequence && tier(world).posting.is_none()
    });
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one landed story follows the same owned and suspected findings through triage, clearing and reads"
)]
fn filtered_findings_keep_owned_and_suspected_reds_in_every_state_and_their_detail() {
    let (mut world, triager) = world_with_findings(temper_engine_domain_forge_change::Wait::Landed, 8);
    tiers::sign_in(&mut world);
    let milestone = milestones::milestone_with_budget(&mut world, 180, b"Blamed goals", 200);
    let goal = milestones::goal_with_budget(&mut world, 181, milestone, false, b"R2 First goal", 200);
    tiers::produce(&mut world, &goal, 94, b"failures", b"suite.alpha\n");
    judged(&mut world, 2);
    assert_eq!(findings(&world)[0].blame.as_ref().unwrap().strength, forge_top::BlameStrength::Owned);
    let owned = findings(&world)[0].number;

    // Admit the JUnit artifact with more than sixteen cases, as the tier's many story does.
    world.fake_env.limits.limits.body_bytes = 512;
    world.fake_env.limits.limits.content_bytes = 4096;
    // The next enumerated run assigns the many finding its remaining ids as suspected.
    let mut many = b"suite.alpha\n".to_vec();
    for number in 0..17 {
        many.extend_from_slice(format!("suite.beta{number}\n").as_bytes());
    }
    tiers::produce(&mut world, &goal, 95, b"failures", &many);
    judged(&mut world, 3);
    let suspected = findings(&world).last().unwrap().number;
    assert!(findings(&world).last().unwrap().signature.as_ref().unwrap().many);
    fake::advance(
        &mut world.fake,
        &world.fake_env,
        b"org/repo",
        b"main",
        b"failures",
        b"suite.alpha\nsuite.beta0\n",
        1,
    )
    .unwrap();
    judged(&mut world, 4);
    assert_eq!(findings(&world).last().unwrap().blame.as_ref().unwrap().strength, forge_top::BlameStrength::Suspected);

    goal_landing::finish_goal(&mut world, &goal);
    tiers::sign_in(&mut world);
    let other_milestone = milestones::milestone_with_budget(&mut world, 182, b"Other goals", 100);
    let other_goal = milestones::goal_with_budget(&mut world, 183, other_milestone, false, b"R2 Other goal", 100);
    tiers::produce(&mut world, &other_goal, 96, b"failures", b"suite.alpha\nsuite.beta0\nsuite.other\n");
    judged(&mut world, 5);
    let other = findings(&world).last().unwrap().number;
    let triager = world.assigned.iter().find(|run| run.task == triager).unwrap().clone();
    let filed = world.scripted_call(
        &triager,
        501,
        engine::Tool::Finding(engine::FindingCall::File {
            kind: forge_top::FindingKind::Red,
            severity: forge_top::Severity::High,
            area: None,
            title: b"No blame".as_slice().into(),
            body: b"Not an after-landing verdict".as_slice().into(),
        }),
    );
    assert!(
        matches!(filed, engine::CallAnswer::Finding(answer) if matches!(answer.as_ref(), engine::FindingToolAnswer::Write(forge_top::FindingReply { result: Ok(_), .. })))
    );
    let unblamed = findings(&world).last().unwrap().number;
    assert!(findings(&world).last().unwrap().blame.is_none());

    assert_eq!(numbers(&document(&mut world, FindingsFilter::Goal(goal.task))), [suspected, owned]);
    assert_eq!(numbers(&document(&mut world, FindingsFilter::Milestone(milestone))), [suspected, owned]);
    assert_eq!(numbers(&document(&mut world, FindingsFilter::Goal(other_goal.task))), [other]);
    assert_eq!(numbers(&document(&mut world, FindingsFilter::Milestone(other_milestone))), [other]);
    assert!(document(&mut world, FindingsFilter::Goal(u64::from(u32::MAX))).rows.is_empty());
    assert!(document(&mut world, FindingsFilter::Milestone(u64::from(u32::MAX))).rows.is_empty());

    for (call, number, act) in [
        (502, owned, forge_top::Triage::Route(other_goal.task)),
        (503, suspected, forge_top::Triage::Drop(b"Not pursuing".as_slice().into())),
    ] {
        let answer =
            world.scripted_call(&triager, call, engine::Tool::Finding(engine::FindingCall::Triage { number, act }));
        assert!(
            matches!(answer, engine::CallAnswer::Finding(answer) if matches!(answer.as_ref(), engine::FindingToolAnswer::Write(forge_top::FindingReply { result: Ok(_), .. })))
        );
    }
    let routed_and_dropped = document(&mut world, FindingsFilter::Goal(goal.task));
    assert_eq!(numbers(&routed_and_dropped), [suspected, owned]);
    assert_eq!(routed_and_dropped.rows[0].state, FindingState::Dropped);
    assert_eq!(routed_and_dropped.rows[0].reason.as_deref(), Some(b"Not pursuing".as_slice()));
    assert_eq!(routed_and_dropped.rows[1].state, FindingState::Routed);
    assert_eq!(routed_and_dropped.rows[1].goal, Some(other_goal.task));
    assert_eq!(numbers(&document(&mut world, FindingsFilter::Milestone(milestone))), [suspected, owned]);

    fake::advance(&mut world.fake, &world.fake_env, b"org/repo", b"main", b"failures", b"", 1).unwrap();
    judged(&mut world, 6);
    let cleared = document(&mut world, FindingsFilter::Goal(goal.task));
    assert_eq!(numbers(&cleared), [suspected, owned]);
    for row in &cleared.rows {
        let clearing = row.cleared.as_ref().unwrap();
        assert_eq!(clearing.tier.as_ref(), b"slow");
        assert_eq!(clearing.run, 6);
        let stored = findings(&world).into_iter().find(|finding| finding.number == row.number).unwrap();
        assert_eq!(clearing.at, stored.cleared.unwrap().at);
    }
    assert_eq!(numbers(&document(&mut world, FindingsFilter::Milestone(milestone))), [suspected, owned]);
    assert_eq!(
        numbers(&document(&mut world, FindingsFilter::Triage { state: None, mine: false })),
        [unblamed, other, suspected, owned]
    );
    assert_eq!(
        numbers(&document(&mut world, FindingsFilter::Triage { state: Some(FindingState::New), mine: false })),
        [unblamed, other]
    );
    let mine = world.scripted_call(
        &triager,
        504,
        engine::Tool::Finding(engine::FindingCall::List { state: None, mine: true }),
    );
    let engine::CallAnswer::Finding(mine) = mine else { panic!("mine list") };
    let engine::FindingToolAnswer::List(mine) = mine.as_ref() else { panic!("mine rows") };
    assert_eq!(numbers(mine), [unblamed]);

    let shown = world.scripted_call(&triager, 505, engine::Tool::Finding(engine::FindingCall::Show { number: owned }));
    let engine::CallAnswer::Finding(shown) = shown else { panic!("finding read") };
    let engine::FindingToolAnswer::Show { finding: Some(shown), .. } = shown.as_ref() else { panic!("finding detail") };
    assert_eq!(shown.row, cleared.rows[1]);
    assert_eq!(shown.blame, findings(&world)[0].blame);
}
