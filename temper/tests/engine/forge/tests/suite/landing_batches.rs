//! Batch preparation, proof, turn and recovery (domain/landing.md, 16).
use skein_fake_forge_domain::{self as fake, api as raw};
use skein_forge::api;
use temper_engine_domain_forge as top;
use temper_engine_domain_forge_change as change;
use temper_engine_domain_forge_landing as landing;
use temper_engine_forge_world::{World, changes, landings::{self, LANDING, Trains}, translate};

fn batch(count: u64) -> World { batch_root(count).0 }
fn batch_root(count: u64) -> (World, Trains) {
    let mut world = landings::turn_in(World::batch_turn(11, None));
    world.batch(u16::try_from(count).expect("small batch"));
    for index in 0..count {
        let branch = format!("temper/c{}", 51 + index);
        world.produce_file(branch.as_bytes(), format!("file{index}").as_bytes(), b"member");
        landings::ready_change(&mut world, 51 + index, branch.as_bytes());
    }
    let mut trains = Trains::default(); trains.judge(&mut world);
    assert_eq!(world.landing(LANDING).landing.members.len(), usize::try_from(count).expect("count"));
    (world, trains)
}
pub(crate) fn heard(preparation: landing::Preparation, failure: &[u8]) -> top::LandingHeard {
    top::LandingHeard { outcomes: Box::new([]), dependencies: Box::new([]), held: Box::new([]), failure: failure.into(), preparation, released: false, cancelled: false, kept: Box::new([]) }
}
fn start_prepare(world: &mut World) {
    for entry in 100..120 { if matches!(landings::step_landing_raw(world, LANDING, entry), landing::Decision::Prepare { .. }) { return; } }
    panic!("prepare not delegated");
}
fn proving(world: &mut World) {
    for entry in 200..230 { landings::step_landing(world, LANDING, entry); if world.landing(LANDING).landing.phase == landing::Phase::Proving { return; } }
    panic!("not proving");
}
fn number(commit: api::Commit) -> u64 { fake::commit_count(&commit).expect("fake commit") }
#[test]
fn three_prefixes_land_by_one_top_fast_forward() {
    let mut world = batch(3); landings::drive_landing(&mut world, LANDING);
    let row = world.landing(LANDING).clone();
    assert_eq!(row.landing.phase, landing::Phase::Landed);
    let top = row.landing.members[2].candidate.expect("top prefix");
    assert_eq!(translate::commit(world.branch(b"main")), top);
    let mut below = row.landing.members[0].base.expect("base");
    for member in &row.landing.members {
        let prefix = member.candidate.expect("prefix");
        assert_eq!(member.landed, Some(prefix)); assert!(member.told);
        assert!(matches!(world.change(member.change).change.state, change::State::Landed { merge, .. } if merge == prefix));
        let raw::Answer::Pull(pull) = world.inspect(&raw::Read::Pull { number: member.pull }) else { panic!("member pull") };
        assert_eq!(pull.merged.map(translate::commit), Some(prefix));
        let raw::Answer::Commits { commits } = world.inspect(&raw::Read::Commits { before: number(below), after: number(prefix) }) else { panic!("prefix parents") };
        let commit = commits.iter().find(|commit| translate::commit(commit.id) == prefix).expect("prefix commit");
        assert_eq!(commit.parents[0], number(below)); below = prefix;
    }
    let calls = world.take_calls();
    let dispatches: Vec<_> = calls.iter().filter_map(|op| match op { api::Op::Write(api::Write::Dispatch { branch, inputs, .. }) => Some((branch, inputs)), _ => None }).collect();
    assert_eq!(dispatches.len(), 1); assert_eq!(dispatches[0].0, &row.landing.members[2].branch);
    assert!(dispatches[0].1.iter().any(|input| input.name.as_ref() == b"temper_commit" && input.value == top::commit_text(&top)));
    let merges: Vec<_> = calls.iter().filter_map(|op| match op { api::Op::Write(api::Write::Merge { number, head, how }) => Some((*number, *head, *how)), _ => None }).collect();
    assert_eq!(merges, vec![(row.landing.round.pull.expect("round pull"), top, api::MergeHow::FastForwardOnly)]);
    let raw::Answer::Pull(pull) = world.inspect(&raw::Read::Pull { number: row.landing.round.pull.expect("round pull") }) else { panic!("round pull") };
    assert_eq!(pull.head, row.landing.members[2].branch);
    let (title, body) = calls.iter().find_map(|op| match op { api::Op::Write(api::Write::OpenPull { title, body, .. }) => Some((title, body)), _ => None }).expect("round opening");
    assert_eq!(title.as_ref(), b"Land 3 changes: #1, #2, #3");
    for member in &row.landing.members { assert!(String::from_utf8_lossy(body).contains(&format!("Change {}, pull request #{}", member.change, member.pull))); }
    landings::close_landing(&mut world, LANDING);
    for member in &row.landing.members { assert!(!world.has_branch(&member.branch)); }
}
#[test]
fn squashes_record_each_members_prefix_in_stack_order() {
    let mut world = batch(3); start_prepare(&mut world); world.prepare_landing(LANDING, true);
    landings::drive_landing(&mut world, LANDING);
    let row = world.landing(LANDING).clone();
    let expected: Vec<_> = row.landing.members.iter().map(|member| {
        assert_eq!(member.combined, Some(landing::Combine::Squash)); assert!(member.recorded && member.told);
        let prefix = member.landed.expect("landed squash");
        let raw::Answer::Pull(pull) = world.inspect(&raw::Read::Pull { number: member.pull }) else { panic!("member pull") };
        assert_eq!(pull.merged.map(translate::commit), Some(prefix)); (member.pull, prefix)
    }).collect();
    let recorded: Vec<_> = world.take_calls().into_iter().filter_map(|op| match op { api::Op::Write(api::Write::Merge { number, how: api::MergeHow::Recorded { commit }, .. }) => Some((number, commit)), _ => None }).collect();
    assert_eq!(recorded, expected);
}
#[test]
fn a_partial_failed_prepare_takes_two_prefixes_and_stacks_only_the_suffix() {
    let mut world = batch(3); start_prepare(&mut world);
    let p2 = translate::commit(world.prepare_subset(LANDING, false, &[0, 1])); world.restart();
    let decision = landings::step_landing_heard(&mut world, LANDING, 200, heard(landing::Preparation::Failed, b"third push refused"));
    assert!(matches!(decision, landing::Decision::Prepare { member: 2, main, .. } if main == p2), "{decision:?}");
    assert_eq!(world.landing(LANDING).landing.members[1].candidate, Some(p2));
    assert!(world.landing(LANDING).landing.members[2].candidate.is_none());
    world.prepare_landing(LANDING, false); landings::drive_landing(&mut world, LANDING);
    assert!(world.landing(LANDING).landing.members.iter().all(|member| member.landed.is_some()));
}
#[test]
fn a_middle_head_move_keeps_the_lower_prefix_and_rebuilds_only_above_it() {
    let mut world = batch(3); proving(&mut world);
    let first = world.landing(LANDING).landing.members[0].candidate.expect("kept prefix");
    let branches: Vec<_> = world.landing(LANDING).landing.members.iter().map(|m| m.branch.clone()).collect();
    let pull = world.landing(LANDING).landing.round.pull.expect("round pull");
    world.push_file(b"temper/c52", b"moved", b"new head");
    let moved = translate::commit(world.branch(b"temper/c52")); world.take_calls();
    let mut prepared = false;
    for entry in 300..340 {
        if let landing::Decision::Prepare { member, main, .. } = landings::step_landing_raw(&mut world, LANDING, entry) {
            assert_eq!((member, main), (2, first)); prepared = true; break;
        }
    }
    assert!(prepared); assert_eq!(world.landing(LANDING).landing.members[1].left, Some(landing::Leave::HeadMoved { head: moved }));
    let deletes: Vec<_> = world.take_calls().into_iter().filter_map(|op| match op { api::Op::Write(api::Write::DeleteBranch { branch }) => Some(branch), _ => None }).collect();
    assert_eq!(deletes, branches[1..]); assert!(world.has_branch(&branches[0]));
    assert!(!world.has_branch(&branches[1]) && !world.has_branch(&branches[2]));
    let raw::Answer::Pull(old) = world.inspect(&raw::Read::Pull { number: pull }) else { panic!("old pull") };
    assert_eq!((old.state, old.merged), (raw::State::Closed, None));
    world.prepare_landing(LANDING, false); landings::drive_landing(&mut world, LANDING);
    assert_eq!(world.landing(LANDING).landing.members[0].landed, Some(first));
    assert!(world.landing(LANDING).landing.members[2].landed.is_some());
    assert!(world.landing(LANDING).landing.members[1].landed.is_none());
}
#[test]
fn a_prepare_conflict_names_its_member_and_tells_branch_from_member() {
    for (with, why) in [("branch", landing::Leave::Conflict), ("member", landing::Leave::EarlierConflict)] {
        let (mut world, mut trains) = batch_root(3); start_prepare(&mut world); world.prepare_subset(LANDING, false, &[0, 2]);
        let result = format!(r#"{{"combined":[{{"member":1,"by":"merge"}},{{"member":3,"by":"merge"}}],"conflicts":[{{"member":2,"with":"{with}","files":["conflicted"],"cut":false}}]}}"#);
        let mut news = heard(landing::Preparation::Idle, result.as_bytes());
        news.outcomes = top::preparation::outcomes(result.as_bytes(), 3).expect("worker result");
        assert_eq!(landings::step_landing_heard(&mut world, LANDING, 200, news), landing::Decision::Leave { member: 1, why });
        assert_eq!(world.landing(LANDING).asks[1].is_some(), with == "branch"); landings::drive_landing(&mut world, LANDING);
        let row = world.landing(LANDING);
        assert!(row.landing.members[0].landed.is_some() && row.landing.members[2].landed.is_some());
        assert_eq!(row.landing.members[1].left, Some(why));
        assert!(String::from_utf8_lossy(&world.landing_member_leave_words(LANDING, 1).expect("cause")).contains("conflicted"));
        if with == "member" {
            assert!(matches!(world.change(52).change.state, change::State::Ready { .. })); assert!(world.change(52).change.landing.is_none());
            trains.judge(&mut world); let task = world.change(52).change.landing.expect("next landing");
            landings::drive_landing(&mut world, task); assert!(world.landing(task).landing.members[0].landed.is_some());
        }
    }
}
#[test]
fn a_top_head_move_opens_from_the_kept_top_without_another_job() {
    let mut world = batch(3); proving(&mut world); let kept = world.landing(LANDING).landing.members[1].clone();
    world.push_file(b"temper/c53", b"moved", b"new head"); world.take_calls();
    let mut opened = false;
    for entry in 300..340 {
        let decision = landings::step_landing_raw(&mut world, LANDING, entry);
        assert!(!matches!(decision, landing::Decision::Prepare { .. }));
        if decision == landing::Decision::Open { opened = true; break; }
    }
    assert!(opened); let calls = world.take_calls();
    assert!(calls.iter().any(|op| matches!(op, api::Op::Write(api::Write::OpenPull { head, .. }) if *head == kept.branch)), "opens from member two");
    let deletes: Vec<_> = calls.into_iter().filter_map(|op| match op { api::Op::Write(api::Write::DeleteBranch { branch }) => Some(branch), _ => None }).collect();
    assert_eq!(deletes, vec![world.landing(LANDING).landing.members[2].branch.clone()]);
    landings::drive_landing(&mut world, LANDING); assert_eq!(translate::commit(world.branch(b"main")), kept.candidate.expect("kept top"));
}
fn reviewed_members() -> World {
    let mut world = landings::turn_in(World::batch_turn(11, None)); world.batch(3);
    for task in 51_u64..54 {
        let branch = format!("temper/c{task}"); world.produce_file(branch.as_bytes(), format!("file{task}").as_bytes(), b"member");
        let head = translate::commit(world.branch(branch.as_bytes())); let mut row = changes::change_row();
        row.task = task; row.change.task = task; row.branch = branch.into_bytes().into();
        row.pull = Some(world.open_pull(&row.branch, b"main")); row.change.state = change::State::Ready { head, ready_since: world.wall() };
        row.change.last_head = Some(head); row.change.ready_since = Some(world.wall());
        row.change.gates = Box::new([change::Gate { number: 9, kind: change::GateKind::Agent, blocking: true, freshness: change::Freshness::Exact, eager: true }]);
        row.verdicts = Box::new([change::GateReport { number: 9, head, status: change::Status::Passed }]); world.event(top::Event::Change { row });
    }
    Trains::default().judge(&mut world); world
}
#[test]
fn every_exact_judgement_uses_its_members_prefix_branch_and_survives_restart() {
    let mut world = reviewed_members(); let mut asked = Vec::new();
    for entry in 100..160 {
        if let landing::Decision::Judge { member } = landings::step_landing(&mut world, LANDING, entry) {
            let saved = world.landing(LANDING).landing.members[usize::from(member)].clone(); let prefix = saved.candidate.expect("prefix");
            assert_eq!(changes::change_step_task(&mut world, saved.change, entry + 1000, change::Status::Passed, Box::new([]), false, false), change::Decision::Delegate(change::Delegate::Gate { number: 9, head: prefix }));
            let change::State::Judging { target, .. } = &world.change(saved.change).change.state else { panic!("judging member") };
            assert_eq!(target.branch, saved.branch); assert_eq!(target.prefix, prefix);
            world.event(top::Event::Delegated { task: saved.change, child: 71 + u64::from(member), kind: change::Delegate::Gate { number: 9, head: prefix } }); asked.push(member);
        }
        if asked.len() == 3 { break; }
    }
    assert_eq!(asked, [0, 1, 2]); world.restart();
    for entry in 200..220 { assert!(!matches!(landings::step_landing(&mut world, LANDING, entry), landing::Decision::Merge { .. } | landing::Decision::Finish)); }
    for member in 0..3 {
        let task = 51 + member;
        world.event(top::Event::DelegateResult { task, child: 71 + member, status: change::Status::Passed, failed: false, cancelled: false, no_change: false, words: b"prefix reviewed".as_slice().into() });
        changes::change_step_task(&mut world, task, 2000 + member, change::Status::Passed, Box::new([]), false, false);
    }
    landings::drive_landing(&mut world, LANDING); assert!(world.landing(LANDING).landing.members.iter().all(|m| m.landed.is_some()));
}
pub(crate) fn workflow(red: bool) -> raw::Workflow {
    raw::Workflow { file: b"gate.yml".as_slice().into(), jobs: Box::new([b"test".as_slice().into()]), fails: Box::new([]), flaky: Box::new([]), report: None, select: None,
        latency_min: skein_lib::Duration::from_secs(2), latency_max: skein_lib::Duration::from_secs(2), silent: 0, passes: 1000,
        cue: red.then(|| raw::Cue::File { path: b"file".as_slice().into(), green: b"fixed".as_slice().into() }) }
}
#[test]
fn a_red_batch_marks_every_unit_alone_then_clears_each_mark_and_batches_again() {
    let mut world = landings::turn_in(World::batch_turn(31, Some(workflow(true)))); world.batch(2); world.push_file(b"main", b"file", b"fixed");
    for (task, path, bytes) in [(51, b"safe".as_slice(), b"one".as_slice()), (52, b"file", b"broken")] {
        let branch = format!("temper/c{task}"); world.external(raw::Op::Git(raw::Git::Create { branch: branch.as_bytes().into(), commit: world.branch(b"main") }), 2);
        world.push_file(branch.as_bytes(), path, bytes); landings::ready_change(&mut world, task, branch.as_bytes());
    }
    let mut trains = Trains::default(); trains.judge(&mut world); landings::drive_landing(&mut world, LANDING);
    for member in &world.landing(LANDING).landing.members {
        assert!(matches!(member.left, Some(landing::Leave::Fallback { units: 2, .. }))); assert!(world.change(member.change).change.alone);
        assert_eq!(world.change(member.change).change.repairs, 0); assert!(world.landing(LANDING).asks.iter().all(Option::is_none));
        assert!(String::from_utf8_lossy(&world.landing_member_leave_words(LANDING, member.unit).expect("cause")).ends_with(": landing one at a time"));
    }
    landings::close_landing(&mut world, LANDING); trains.judge(&mut world);
    let good = world.change(51).change.landing.expect("first solo"); assert_eq!(world.landing(good).landing.members.len(), 1);
    landings::drive_landing(&mut world, good); assert!(!world.change(51).change.alone); assert!(world.change(52).change.alone);
    landings::close_landing(&mut world, good); trains.judge(&mut world);
    let bad = world.change(52).change.landing.expect("second solo"); assert_eq!(world.landing(bad).landing.members.len(), 1);
    landings::drive_landing(&mut world, bad); assert!(matches!(world.landing(bad).landing.members[0].left, Some(landing::Leave::Failed { .. })));
    assert!(!world.change(52).change.alone); assert_eq!(world.change(52).change.repairs, 1); landings::close_landing(&mut world, bad);
    world.push_file(b"temper/c52", b"file", b"fixed");
    for entry in 3000..3020 { changes::change_step_task(&mut world, 52, entry, change::Status::Passed, Box::new([]), false, false); if matches!(world.change(52).change.state, change::State::Ready { .. }) { break; } }
    assert!(matches!(world.change(52).change.state, change::State::Ready { .. }), "{:?}", world.change(52).change.state);
    world.produce_file(b"temper/c53", b"next", b"next change"); landings::ready_change(&mut world, 53, b"temper/c53"); trains.judge(&mut world);
    let next = world.change(52).change.landing.expect("batch resumed"); assert_eq!(world.landing(next).landing.members.len(), 2);
    landings::drive_landing(&mut world, next); assert!(world.landing(next).landing.members.iter().all(|m| m.landed.is_some()));
}
#[test]
fn an_outside_base_move_rebuilds_every_member_after_restart() {
    let mut world = batch(3); proving(&mut world);
    let before: Vec<_> = world.landing(LANDING).landing.members.iter().map(|m| (m.branch.clone(), m.candidate.expect("prefix"))).collect();
    world.push_file(b"main", b"outside", b"outside push"); world.restart(); world.take_calls(); landings::drive_landing(&mut world, LANDING);
    assert_eq!(world.landing(LANDING).landing.stale_proofs, 1);
    for (member, (branch, old)) in world.landing(LANDING).landing.members.iter().zip(&before) { assert_ne!(member.candidate, Some(*old)); assert!(member.landed.is_some()); assert_eq!(&member.branch, branch); }
    let deletes: Vec<_> = world.take_calls().into_iter().filter_map(|op| match op { api::Op::Write(api::Write::DeleteBranch { branch }) => Some(branch), _ => None }).collect();
    assert_eq!(deletes, before.iter().map(|(branch, _)| branch.clone()).collect::<Vec<_>>()); landings::close_landing(&mut world, LANDING);
    for (branch, _) in before { assert!(!world.has_branch(&branch)); }
}
#[test]
fn batches_restart_preparing_proving_turn_recording_and_ended_without_repeating_writes() {
    for checkpoint in ["preparing", "proving", "turn", "recording", "ended"] {
        let mut world = batch(3); start_prepare(&mut world); let squash = checkpoint == "recording"; world.prepare_landing(LANDING, squash);
        if checkpoint == "ended" { landings::drive_landing(&mut world, LANDING); }
        else if checkpoint != "preparing" {
            let mut reached = false;
            for entry in 500..560 {
                let decision = landings::step_landing_raw(&mut world, LANDING, entry);
                reached = match checkpoint { "proving" => matches!(decision, landing::Decision::Dispatch { .. }), "turn" => matches!(decision, landing::Decision::Merge { .. }), "recording" => matches!(decision, landing::Decision::Record { member: 0, .. }), _ => unreachable!() };
                if reached { break; }
            }
            assert!(reached, "checkpoint {checkpoint}");
        }
        world.restart(); landings::drive_landing(&mut world, LANDING); let row = world.landing(LANDING).clone();
        assert!(row.landing.members.iter().all(|m| m.landed.is_some()), "{checkpoint}"); let calls = world.take_calls();
        assert_eq!(calls.iter().filter(|op| matches!(op, api::Op::Write(api::Write::OpenPull { .. }))).count(), 1, "{checkpoint}");
        assert_eq!(calls.iter().filter(|op| matches!(op, api::Op::Write(api::Write::Dispatch { .. }))).count(), 1, "{checkpoint}");
        assert_eq!(calls.iter().filter(|op| matches!(op, api::Op::Write(api::Write::Merge { how: api::MergeHow::FastForwardOnly, .. }))).count(), 1, "{checkpoint}");
        assert_eq!(calls.iter().filter(|op| matches!(op, api::Op::Write(api::Write::Merge { how: api::MergeHow::Recorded { .. }, .. }))).count(), if squash { 3 } else { 0 }, "{checkpoint}");
        landings::close_landing(&mut world, LANDING); for member in &row.landing.members { assert!(!world.has_branch(&member.branch), "{checkpoint}"); }
    }
}
#[test]
fn a_path_in_the_lower_member_governs_the_turn_for_every_members_evidence() {
    use top::settings;
    let mut world = landings::turn_in(World::batch_turn(11, None)); world.batch(3);
    for task in 51_u64..54 {
        let branch = format!("temper/c{task}"); let path = if task == 51 { "protocol".to_owned() } else { format!("other{task}") };
        world.produce_file(branch.as_bytes(), path.as_bytes(), &task.to_be_bytes()); let head = translate::commit(world.branch(branch.as_bytes()));
        let mut row = changes::change_row(); row.task = task; row.change.task = task; row.branch = branch.into_bytes().into();
        row.pull = Some(world.open_pull(&row.branch, b"main")); row.change.state = change::State::Ready { head, ready_since: world.wall() }; row.change.last_head = Some(head); row.change.ready_since = Some(world.wall());
        if task == 53 { row.change.gates = Box::new([change::Gate { number: 9, kind: change::GateKind::Agent, blocking: true, freshness: change::Freshness::Clean, eager: true }]); row.verdicts = Box::new([change::GateReport { number: 9, head, status: change::Status::Passed }]); }
        world.event(top::Event::Change { row });
    }
    Trains::default().judge(&mut world);
    world.keep_settings(1, &[
        settings::Asked { kind: 8, scope: 5, number: 1, edit: settings::Edit::Set(settings::SizedValue { size: 128, value: settings::Value::Classes(temper_protocol_reaches::Classes { items: Box::new([]), sets: Box::new([temper_protocol_reaches::PathSet { number: 1, name: b"protocol".as_slice().into(), patterns: Box::new([b"protocol".as_slice().into()]) }]) }) }) },
        settings::Asked { kind: 2, scope: 5, number: 1, edit: settings::Edit::Set(settings::SizedValue { size: 128, value: settings::Value::Rule(settings::LandingRule { sets: Box::new([1]), pattern: top::GatePattern { segments: Box::new([b"branch".as_slice().into()]), last: top::GateLast::Exact(b"main".as_slice().into()) }, enforce: true, ci: true, up_to_date: true, gates: Box::new([settings::RuleGate { number: 9, blocking: true, freshness: top::JudgeFreshness::Clean }]), approvals: Box::new([]) }) }) },
    ]);
    let before = world.branch(b"main");
    for entry in 100..160 {
        let step = landings::read_landing_step(&mut world, LANDING, entry, heard(landing::Preparation::Idle, b""));
        if matches!(step.decision, landing::Decision::Prepare { .. }) { world.prepare_landing(LANDING, false); }
        if matches!(step.decision, landing::Decision::Merge { .. }) {
            let evidence = step.evidence.expect("admission evidence"); assert_eq!(evidence.len(), 3);
            assert_eq!(evidence.iter().map(|facts| facts.change).collect::<Vec<_>>(), [51, 52, 53]);
            assert_ne!(world.judge_landing_evidence(LANDING, &evidence[0].facts), Some(top::JudgeVerdict::Met));
            assert_eq!(world.judge_landing_evidence(LANDING, &evidence[2].facts), Some(top::JudgeVerdict::Met));
            assert_eq!(world.branch(b"main"), before); return;
        }
        if let Some(entry) = step.entry { world.event(top::Event::Committed { entry }); world.run_for(1); }
    }
    panic!("merge evidence not offered");
}

#[test]
fn a_prefix_with_a_long_member_history_is_validated_past_the_first_page() {
    let mut world = landings::turn_in(World::batch_turn(11, Some(workflow(false))));
    world.produce_file(b"temper/c51", b"history", b"first");
    for byte in 0..10 { world.push_file(b"temper/c51", b"history", &[byte]); }
    landings::ready_change(&mut world, 51, b"temper/c51"); Trains::default().judge(&mut world);
    start_prepare(&mut world); let prefix = translate::commit(world.prepare_landing(LANDING, false));
    assert_eq!(landings::step_landing_raw(&mut world, LANDING, 200), landing::Decision::Open);
    assert_eq!(world.landing(LANDING).landing.members[0].candidate, Some(prefix));
    assert!(world.take_calls().iter().any(|op| matches!(op, api::Op::Read(api::Read::Commits { after, page: 2, .. }) if *after == prefix)));
}
