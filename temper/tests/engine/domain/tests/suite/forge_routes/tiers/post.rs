//! The forge's own statuses and keyed comments (temper/docs/design/domain/after-landing.md, 3 and 9).
use super::*;
use temper_engine_forge_world::{REPO, translate};

fn comments(world: &World, pull: u64) -> Box<[raw::Comment]> {
    let raw::Answer::Comments(comments) = world.fake.inspect(&world.fake_env.limits, b"org/repo", &raw::Read::Comments { number: pull, since: None }).expect("comments") else { panic!("comments") };
    comments
}
fn raw_tip(commit: skein_forge::api::Commit) -> u64 {
    let value = u64::from_be_bytes(commit[8..16].try_into().expect("fake commit"));
    assert_eq!(translate::commit(value), commit);
    value
}
fn status(world: &World, tip: skein_forge::api::Commit) -> Option<raw::Status> {
    let raw::Answer::Statuses(statuses) = world.fake.inspect(&world.fake_env.limits, b"org/repo", &raw::Read::Statuses { commit: raw_tip(tip) }).expect("statuses") else { panic!("statuses") };
    statuses.into_vec().into_iter().find(|status| status.context.as_ref() == b"temper/verdict/slow")
}
fn comment_body(world: &World, task: u64, run: u64, part: u16, words: &str) -> Box<[u8]> {
    let purpose = forge_top::purpose::EffectPurpose::Tier { repository: REPO, tier: 0, run, operation: 7, part };
    let identity = purpose.identity().expect("fixed tier purpose");
    let pull = change(world, task).pull.expect("member pull");
    let key = world.writes.iter().find_map(|write| match write {
        skein_forge::api::Write::Post { number, key, .. } if *number == pull && key.ends_with(&identity) => Some(key),
        _ => None,
    }).expect("comment's tag 8, run, operation 7 and part");
    translate::keyed(key, words.as_bytes())
}
fn assert_comment(world: &World, change_task: u64, run: u64, part: u16, words: &str) {
    let pull = change(world, change_task).pull.expect("member pull");
    let expected = comment_body(world, change_task, run, part, words);
    assert_eq!(comments(world, pull).iter().filter(|comment| comment.body == expected).count(), 1, "one keyed {words:?}");
}
fn posted(world: &mut World, number: u64) {
    until(world, RUN_TICKS, "verdict posted", |world| tier(world).record.sequence == number && tier(world).posting.is_none());
}

#[test]
fn green_verdict_status_and_member_comments_resume_once() {
    for restart in [false, true] {
        let (mut world, chat) = chat_world(false, Tier::default());
        let a = produce(&mut world, &chat, 93, b"file", b"first fix");
        let b = produce(&mut world, &chat, 94, b"file", b"second fix");
        let first = change(&world, a).pull.expect("member pull");
        let second = change(&world, b).pull.expect("member pull");
        if restart {
            until(&mut world, RUN_TICKS * 2, "between two verdict comments", |world| {
                comments(world, first).iter().any(|comment| String::from_utf8_lossy(&comment.body).contains("Cleared by slow run 2."))
                    && comments(world, second).is_empty() && restart_cut(world)
            });
            world.restart_same();
        }
        posted(&mut world, 2);
        let run = tier_runs::runs(&world).into_iter().find(|run| run.commit == raw_tip(merge(&world, b))).expect("whole run at the tip");
        let url = tier_runs::run_url(&run);
        let verdict = status(&world, merge(&world, b)).expect("tip verdict");
        assert_eq!(verdict.state, raw::Check::Passed);
        assert_eq!(verdict.description.as_ref(), b"run 2 passed");
        assert_eq!(verdict.link.as_ref(), url.as_bytes());
        for (part, member) in [(0, a), (1, b)] {
            assert_comment(&world, member, 2, part, &format!("Cleared by slow run 2.\n\nRun: {url}"));
            assert_eq!(comments(&world, change(&world, member).pull.expect("member pull")).len(), 1);
        }
    }
}

#[test]
fn red_verdict_and_named_repair_reach_each_members_pull() {
    let (mut world, chat) = chat_world(false, Tier::default());
    let clear = produce(&mut world, &chat, 93, b"docs", b"only documentation");
    let red = produce(&mut world, &chat, 94, b"failures", b"suite.alpha\n");
    posted(&mut world, 2);
    let tip = merge(&world, red);
    let run = tier_runs::runs(&world).into_iter().find(|run| run.commit == raw_tip(tip)).expect("whole run at the tip");
    let url = tier_runs::run_url(&run);
    let verdict = status(&world, tip).expect("red status");
    assert_eq!(verdict.state, raw::Check::Failed);
    assert_eq!(verdict.description.as_ref(), b"run 2 failed: 1 blamed, 0 inherited");
    assert_eq!(verdict.link.as_ref(), url.as_bytes());
    assert_comment(&world, clear, 2, 0, &format!("Cleared by slow run 2.\n\nRun: {url}"));
    assert_comment(&world, red, 2, 1, &format!("Red: slow run 2 blames this landing for suite.alpha.\n\nRun: {url}"));
    let answer = world.scripted_call(&chat, 96, engine::Tool::Change { changes: Box::new([engine::ChangeCall {
        repairs: Some(tip), ..engine::ChangeCall::words(b"Repair red".as_slice().into(), None)
    }]) });
    let repair = produce_delegated(&mut world, answer, b"main", b"failures", b"");
    posted(&mut world, 3);
    let verdict = status(&world, merge(&world, repair)).expect("repair status");
    assert_eq!(verdict.state, raw::Check::Passed);
    assert_eq!(verdict.description.as_ref(), b"run 3 passed");
    let url = String::from_utf8(verdict.link.into_vec()).expect("whole run at the tip");
    assert_comment(&world, repair, 3, 0, &format!("Cleared by slow run 3.\n\nRun: {url}"));
    let pull = change(&world, repair).pull.expect("member pull");
    assert_comment(&world, red, 3, 1, &format!("Repaired by #{pull}: slow run 3 passed the failures blamed on this landing.\n\nRun: {url}"));
}

#[test]
fn runner_fault_posts_pending_before_held_then_replaces_it() {
    let (mut world, _) = chat_world(false, Tier::default());
    world.fake_env.limits.setup_faults = 1000;
    let tip = tier(&world).record.run.as_ref().expect("baseline").tip;
    let run_task = tier(&world).task.expect("baseline task");
    until(&mut world, RUN_TICKS * 4, "pending before hold", |world| status(world, tip).is_some_and(|status| status.state == raw::Check::Pending));
    assert!(!matches!(task(&world, run_task).phase, tasks::Phase::Held { .. }), "status made before hold");
    let pending = status(&world, tip).expect("tip status");
    assert_eq!(pending.description.as_ref(), b"held after 3 runner faults, last: set-up");
    assert!(!pending.link.is_empty());
    until(&mut world, LANDING_TICKS, "runner held", |world| matches!(task(world, run_task).phase, tasks::Phase::Held { .. }));
    world.fake_env.limits.setup_faults = 0;
    sign_in(&mut world);
    let revision = task(&world, run_task).revisions.hold;
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]), reply_to: ReplyTo::new(Token::new(111)), sign_in: world.signed_in.expect("owner signed in"), key: [111; 16],
        ask: people::Ask::Release { project: 1, task: run_task, revision, again: None },
    }));
    posted(&mut world, 1);
    let verdict = status(&world, tip).expect("tip status");
    assert_eq!(verdict.state, raw::Check::Passed);
    assert_eq!(verdict.description.as_ref(), b"run 1 passed");
}
