//! What QA cannot judge is recorded and filed without a goal (qa/gaps.md, 8).
use super::passes::{build, start};
use super::{case, session};
use temper_qa_tool::{Answer, CaseState, Exit, Ledger};

use super::world::{Agent, Git, Loop, World, id};
use std::fs;
use temper_qa_tool::faces::CommandAnswer;

fn running_triage(world: &World) -> Loop {
    let mut git = Git::default();
    git.commit(0, &[], "baseline");
    let mut run = world.start(git, &[Agent::Never]);
    run.tick();
    run.git.commit(1, &[0], "change");
    run.git.merge(2, &[0, 1], 11);
    run.tick();
    run
}
fn record_gaps(world: &World) {
    world
        .home
        .transact(|ledger| {
            ledger.triage_gap(1, "ctl", "no runner tool", "tools", 1)?;
            ledger.add_case(1, &case(), 1)?;
            ledger.triage_gap(1, "fake", "no fake runner", "fakes", 2)?;
            Ok(())
        })
        .expect("two gaps");
}
fn answer(run: &mut Loop, exit: Exit) {
    run.processes.answers.push_back(CommandAnswer { exit, last_stdout: None });
}
fn gap_request(world: &World) -> std::path::PathBuf {
    world.home.directory.join("gaps/requests/1")
}

#[test]
fn one_request_files_every_open_gap_once_and_has_no_goal() {
    let world = World::new();
    let mut run = running_triage(&world);
    record_gaps(&world);
    answer(&mut run, Exit::Code(0));
    run.tick();
    let request = gap_request(&world);
    assert!(!request.join("goal").exists());
    assert_eq!(
        fs::read_to_string(request.join("message")).expect("message"),
        "QA gaps 1, 2: 2 open\n\nQA-gaps: 1, 2\n"
    );
    for (number, owner, why) in [(1, "tools", "no runner tool"), (2, "fakes", "no fake runner")] {
        let text = fs::read_to_string(request.join(format!("tree/temper/docs/development/qa/gaps/gap-{number}.md")))
            .expect("gap document");
        for words in [
            format!("Owner: {owner}"),
            format!("Why: {why}"),
            "Landing 1: goal 11, Goal 11's title".into(),
            format!("Commit: {}", id(2)),
            format!("QA-gap: {number}"),
        ] {
            assert!(text.contains(&words), "{words}");
        }
    }
    assert!(world.ledger().gaps.iter().all(|gap| gap.state == temper_qa_tool::GapState::Filed && gap.filed == Some(0)));
    run.tick();
    assert_eq!(run.processes.bounded.len(), 1);
    let (command, bound) = &run.processes.bounded[0];
    assert_eq!(*bound, 600);
    assert_eq!(command.directory, request);
    assert_eq!(command.stdin, std::path::Path::new("/dev/null"));
}

#[test]
fn failed_gap_filing_retries_and_removes_a_stale_goal() {
    let world = World::new();
    let mut run = running_triage(&world);
    record_gaps(&world);
    answer(&mut run, Exit::Code(1));
    run.tick();
    assert!(world.ledger().gaps.iter().all(|gap| gap.state == temper_qa_tool::GapState::Open && gap.filed.is_none()));
    fs::write(gap_request(&world).join("goal"), "stale").expect("stale file");
    answer(&mut run, Exit::Code(0));
    run.tick();
    assert_eq!(run.processes.bounded.len(), 2);
    assert!(!gap_request(&world).join("goal").exists());
    assert!(world.ledger().gaps.iter().all(|gap| gap.state == temper_qa_tool::GapState::Filed));
}

fn closing_git() -> Git {
    let mut git = Git::default();
    git.commit(0, &[], "baseline");
    git.commit(1, &[0], "first change");
    git.merge(2, &[0, 1], 11);
    git.commit(3, &[2], "the tool is added\n\nQA-gap: 1\nQA-gap: 999\nQA-gap: bad\n");
    git.merge(4, &[2, 3], 12);
    git
}

#[test]
fn closing_trailer_waits_for_a_kept_build_and_requeues_on_that_build() {
    let world = World::new();
    let mut ledger = Ledger::default();
    ledger.record_landing(&id(2), 11, None, "first", 0);
    let triage = session(&mut ledger, 1);
    ledger.add_case(triage, &case(), 1).expect("case");
    ledger.end_session(triage, Exit::Code(0), None, 2, "qa").expect("triage end");
    build(&mut ledger, &id(2), &[1], 1);
    let pass = start(&mut ledger);
    ledger.pass_gap(pass.session, 1, "ctl", "no runner tool", "tools", 5).expect("gap");
    ledger.end_pass_session(pass.session, Exit::Code(0), None, 6, "qa", &[]).expect("pass end");
    ledger.record_head(&id(2));
    let mut git = closing_git();
    temper_qa_tool::watch::watch(&mut git, &world.home.configuration.git, &mut ledger, 7).expect("watch without build");
    assert_eq!(ledger.gaps[0].state, temper_qa_tool::GapState::Open);
    ledger.keep_build(&id(4), 8).expect("closing build");
    temper_qa_tool::watch::watch(&mut git, &world.home.configuration.git, &mut ledger, 9).expect("watch kept build");
    assert_eq!(ledger.gaps[0].closed, Some(temper_qa_tool::GapClosed { landing: 2, time: 9 }));
    assert_eq!(ledger.cases[0].state, CaseState::Queued);
    assert_eq!(ledger.cases[0].outcomes.len(), 0);
    assert_eq!(Ledger::decode(&ledger.encode().expect("encode")).expect("closed roundtrip"), ledger);
    let triage = session(&mut ledger, 2);
    ledger.answer_none(triage, "covered by the gap's case").expect("answer");
    ledger.end_session(triage, Exit::Code(0), None, 10, "qa").expect("end");
    ledger.builds[1].held = Some(99);
    assert!(
        ledger.start_pass("places/1", temper_qa_tool::Kind::Box, &["web".into()], "high", 11).is_none(),
        "old build lacks the closing landing"
    );
    ledger.builds[1].held = None;
    assert_eq!(start(&mut ledger).build, id(4));
    temper_qa_tool::watch::watch(&mut git, &world.home.configuration.git, &mut ledger, 12).expect("idempotent");
    assert_eq!(ledger.cases[0].state, CaseState::Assigned);
}

#[test]
fn gap_closures_read_only_the_landing_side_of_either_parent_order() {
    for parents in [[3, 4], [4, 3]] {
        let world = World::new();
        let mut git = Git::default();
        git.commit(0, &[], "baseline");
        git.commit(1, &[0], "earlier change");
        git.merge(2, &[0, 1], 11);
        git.commit(3, &[0], "branch closes its gap\n\nQA-gap: 1\n");
        git.commit(4, &[2], "main closes another gap\n\nQA-gap: 2\n");
        git.merge(5, &parents, 12);
        let mut ledger = Ledger::default();
        ledger.record_landing(&id(2), 11, None, "earlier landing", 0);
        let session = session(&mut ledger, 1);
        ledger.triage_gap(session, "ctl", "branch's gap", "tools", 1).expect("branch gap");
        ledger.triage_gap(session, "ctl", "main's gap", "tools", 1).expect("main gap");
        ledger.end_session(session, Exit::Code(0), None, 2, "qa").expect("end triage");
        ledger.record_head(&id(4));
        ledger.keep_build(&id(5), 3).expect("kept build");
        temper_qa_tool::watch::watch(&mut git, &world.home.configuration.git, &mut ledger, 4).expect("watch");
        assert_eq!(ledger.gaps[0].closed, Some(temper_qa_tool::GapClosed { landing: 2, time: 4 }));
        assert_eq!(ledger.gaps[1].state, temper_qa_tool::GapState::Open);
        assert!(git.calls.contains(&format!("gaps {} {}", id(4), id(5))));
    }
}

#[test]
fn closed_triage_gap_retriages_with_the_kept_build_and_earlier_gap() {
    let world = World::new();
    let mut run = running_triage(&world);
    world.home.transact(|ledger| ledger.triage_gap(1, "ctl", "no runner tool", "tools", 1)).expect("gap");
    answer(&mut run, Exit::Code(0));
    run.processes.finish(1, Agent::Empty);
    run.tick();
    run.git = closing_git();
    run.tick();
    assert_eq!(world.ledger().landings[0].triaged, Some(0));
    run.processes.finish(2, Agent::None);
    run.tick();
    run.git.commit(5, &[4], "kept later tip");
    run.git.file_at(5, "temper/docs/help/reference/ctl-runner.md", b"# Runner");
    world.build(5);
    run.tick();
    let ledger = world.ledger();
    let triage = ledger.sessions.iter().rev().find(|session| session.landing == Some(1)).expect("new triage");
    assert!(triage.ended.is_none());
    assert!(triage.pages["ctl"].contains(&"runner".into()));
    assert_eq!(ledger.gaps[0].state, temper_qa_tool::GapState::Closed);
    assert!(run.git.calls.contains(&format!("checkout {}", id(5))));
    let brief = fs::read_to_string(world.home.session_directory(triage.number).join("brief")).expect("brief");
    assert!(brief.contains("Earlier gap 1 (ctl; closed): no runner tool"));
    assert!(brief.contains("is a gap: `temper-qa triage gap KIND WHY`, never none"));
}

#[test]
fn a_pass_gap_request_keeps_the_case_setup_and_steps() {
    let world = World::new();
    let mut run = running_triage(&world);
    world.home.transact(|ledger| ledger.add_case(1, &case(), 1)).expect("case");
    run.processes.finish(1, Agent::Empty);
    run.tick();
    world
        .home
        .transact(|ledger| {
            build(ledger, &id(2), &[1], 1);
            let pass = start(ledger);
            ledger.pass_gap(pass.session, 1, "qa", "no pushed branch", "qa", 5)?;
            ledger.end_pass_session(pass.session, Exit::Code(0), None, 6, "qa", &[])
        })
        .expect("pass gap");
    answer(&mut run, Exit::Code(0));
    run.tick();
    let text = fs::read_to_string(gap_request(&world).join("tree/temper/docs/development/qa/gaps/gap-1.md"))
        .expect("case gap");
    for words in [
        "Case 1: web / work",
        "Landing 1; setup fresh; adds:",
        "- do: open Work",
        "expect: Drafts lists the draft",
        "Why: no pushed branch",
        "QA-gap: 1",
    ] {
        assert!(text.contains(words), "{words}");
    }
    assert!(!gap_request(&world).join("goal").exists());
}

#[test]
fn closing_a_triage_gap_preserves_the_running_later_triage() {
    let world = World::new();
    let mut run = running_triage(&world);
    world.home.transact(|ledger| ledger.triage_gap(1, "ctl", "no runner tool", "tools", 1)).expect("gap");
    answer(&mut run, Exit::Code(0));
    run.processes.finish(1, Agent::Empty);
    run.tick();
    run.git = closing_git();
    run.processes.next.push_back(Agent::None);
    run.tick(); // The closing landing's triage starts before its build is kept.
    run.git.commit(5, &[4], "later build");
    world.build(5);
    run.tick();
    let ledger = world.ledger();
    assert_eq!(ledger.sessions[1].exit, Some(Exit::Code(0)));
    assert_eq!(ledger.landings[1].answer, Some(Answer::None { why: "internal refactor".into() }));
    assert_eq!(ledger.gaps[0].state, temper_qa_tool::GapState::Closed);
    assert_eq!(ledger.sessions[2].landing, Some(1));
    assert!(ledger.sessions[2].ended.is_none());
}
