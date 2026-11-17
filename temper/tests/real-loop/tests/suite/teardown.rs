//! Teardown names an unrequested kill and the service notes around it.
#[test]
fn a_cancelled_stubborn_agent_names_its_killer_and_grace_with_recent_notes() {
    use temper_real_loop_world::{
        End, Exit, Scratch, argv,
        calls::{D1, call, release_hold, task, until},
        check, fixtures, llm, run_stubborn_on, scripts,
    };
    const CHAT: usize = 1;
    let scratch = Scratch::new("stubborn-cancel");
    fixtures::standing_chats(scratch.path()).expect("standing chat fixture");
    let ran = run_stubborn_on(
        scratch,
        || llm::scripted(scripts::live_scripts()),
        || {
            let mut steps = fixtures::signed_chat("stubborn");
            steps.extend([
                call(D1, argv!["task", "show", task(CHAT), "--until", "phase:running"]),
                until(D1, argv!["chat", "show", task(CHAT)], scripts::FIRST, scripts::FIRST),
                call(D1, argv!["task", "cancel", task(CHAT), "Stop the stubborn run"]),
                call(D1, argv!["task", "show", task(CHAT), "--until", "ended"]),
                release_hold(scripts::LIVE),
            ]);
            steps
        },
    );
    for called in ran.calls() {
        assert_eq!(called.exit, Some(Exit::Code(0)), "{called:?}");
    }
    assert_eq!(ran.served(), Some(&End::Engine(temper_service::EngineEnd::Drained)));
    let finding = check(&ran).expect_err("smith must force the stubborn agent after its grace");
    assert!(finding.contains("only requested kills"), "{finding}");
    assert_eq!(ran.killed().len(), 1, "{finding}");
    let killed = &ran.killed()[0];
    assert!(killed.program.ends_with("/temper-agent"), "{finding}");
    let mut lines = finding.lines();
    lines.next().expect("kill check heading");
    let kill = lines.next().expect("kill line");
    assert!(kill.starts_with(&killed.to_string()), "{finding}");
    assert!(kill.contains("(a root): forced after its grace; exit Signal(9)"), "{finding}");
    assert_eq!(killed.requester, 1, "the service root: {finding}");
    let (_, millis) = kill.rsplit_once("; ").expect("relative kill time");
    millis.strip_suffix(" ms from run start").expect("milliseconds").parse::<u64>().expect("relative time");
    assert_eq!(lines.next(), Some("service notes (last 5 s, latest 100):"), "{finding}");
    let note = lines.next().expect("cancellation leaves worker notes after the kills");
    let (millis, debug) = note.split_once(" ms: ").expect("note time then Debug");
    millis.parse::<u64>().expect("relative note time");
    assert!(!debug.is_empty(), "{finding}");
    assert!(lines.count() < 100, "at most 100 notes: {finding}");
}
