//! A ctl answer releases a latched report batch; the setup's scripts prove the red top.
//! testing/real-loop.md, the turn-landing setup's stories; testing/qa/setups.md, 6.
use crate::cpu;
use serde_json::Value;
use skein_fake_forge_domain::{api, object_bytes};
use temper_real_loop_world::{Exit, Ran, turn_landing as story};

const REPOSITORY: &[u8] = b"acme/app";

#[test]
fn one_ctl_answer_releases_the_latched_batch() {
    use story::latched;
    let start = cpu();
    let ran = latched::run();
    for (at, call) in ran.calls().iter().enumerate() {
        assert_eq!(call.exit, Some(Exit::Code(0)), "call {at}: {call:?}");
    }
    assert_eq!(ran.number(1), latched::GOAL);
    let tree = story::document(&ran, 3);
    let delegates = tree["delegates"].as_array().expect("goal delegates");
    let numbers: Vec<_> = delegates.iter().map(|task| task["chip"]["task"].as_u64().expect("delegate")).collect();
    assert_eq!(numbers, [latched::LATCH, latched::REPORTS[0], latched::REPORTS[1]], "{tree}");
    for (at, task) in latched::BEFORE.into_iter().zip(latched::REPORTS) {
        let pending = story::document(&ran, at);
        assert_eq!(pending["chip"]["task"], task);
        assert_eq!(
            pending["chip"]["details"]["owner"]["phase"]["waiting"]["dependencies"],
            serde_json::json!([latched::LATCH]),
            "{pending}"
        );
        let [fact] = pending["dependency_facts"].as_array().expect("dependency facts").as_slice() else {
            panic!("one pending latch: {pending}")
        };
        assert_eq!(fact["number"], latched::LATCH);
        assert_eq!(fact["pending"], true);
        let details = &pending["chip"]["details"]["owner"]["details"];
        assert!(details["run_since"].is_null(), "not run before release: {pending}");
        assert!(details["attempt"].is_null(), "no attempt before release: {pending}");
    }
    let latch = story::document(&ran, latched::PERSON);
    assert_eq!(latch["chip"]["task"], latched::LATCH);
    assert_eq!(latch["chip"]["executor"], serde_json::json!({"party":{"party":1}}));
    assert_eq!(latch["result_contract"], serde_json::json!({"report":{"words":64}}));
    assert_eq!(latch["person_revision"]["value"], 0, "revision used by the answer");
    assert_eq!(ran.calls()[latched::RELEASE].exit, Some(Exit::Code(0)));
    let latch = story::document(&ran, latched::LATCH_ENDED);
    let ended = |task: &Value| task["chip"]["details"]["owner"]["details"]["ended_at"].as_u64().expect("ended wall");
    assert_eq!(latch["chip"]["phase"], "ended");
    let released = ended(&latch);
    assert_eq!(latch["result"]["value"]["report"]["bytes"], "Ready together");
    for ((at, task), report) in latched::AFTER.into_iter().zip(latched::REPORTS).zip(["X ended.", "Y ended."]) {
        let done = story::document(&ran, at);
        assert_eq!(done["chip"]["task"], task);
        assert_eq!(done["chip"]["phase"], "ended");
        assert!(ended(&done) >= released);
        let [fact] = done["dependency_facts"].as_array().expect("dependency facts").as_slice() else {
            panic!("one met latch: {done}")
        };
        assert_eq!(fact["number"], latched::LATCH);
        assert_eq!(fact["pending"], false);
        let started = done["chip"]["details"]["owner"]["details"]["run_since"].as_u64().expect("report run start");
        assert!(started >= released, "report started after latch ended: {done}; latch {latch}");
        assert_eq!(done["result"]["value"]["report"]["bytes"], report);
    }
    let goal = story::document(&ran, latched::ENDED);
    assert_eq!(goal["chip"]["phase"], "ended");
    assert_eq!(goal["result"]["value"]["report"]["bytes"], "Both reports ended.");
    eprintln!("latched story CPU: {} ns (user + system, children included)", cpu() - start);
}

#[test]
fn a_failing_member_turns_the_dispatched_top_red() {
    let start = cpu();
    let ran = story::run();
    before_release(&ran);
    let failed = ran
        .forge()
        .dispatched()
        .find(|seen| seen.run.conclusion == Some(api::Conclusion::Failure))
        .expect("top gate failed");
    assert_eq!(failed.run.event, api::RunEvent::Dispatch);
    assert_eq!(failed.jobs[0].conclusion, Some(api::Conclusion::Failure));
    let tree = &failed.tree;
    assert_eq!(
        tree.iter().find(|file| file.path.as_ref() == b".fake-forge/gate").expect("red member at top").content.as_ref(),
        b"fail\n"
    );
    assert!(tree.iter().any(|file| file.path.as_ref() == b"turn-b.txt"), "the failed top contains both members");
    let history = story::document(&ran, story::RELEASE + 1);
    let events = history["events"].as_array().expect("history events");
    assert!(
        events.iter().any(|event| event["what"]["kind"] == "formed"
            && event["what"]["members"].as_array().is_some_and(|members| members.len() == 2)),
        "one forming decision of two: {history}"
    );
    let failed_head = sha(failed.run.commit);
    assert!(
        events.iter().any(|event| event["what"]["kind"] == "gate-ended"
            && event["what"]["verdict"] == "failed"
            && event["head"].as_str().is_some_and(|head| head.starts_with(failed_head.as_str().expect("head")))),
        "failed receipt for the dispatched top: {history}"
    );
    assert_ne!(main(&ran), failed.run.commit);
    let red = story::document(&ran, story::BEFORE[0]);
    let api::Answer::Pull(pull) =
        ran.forge().read(REPOSITORY, &api::Read::Pull { number: red["pull"]["number"].as_u64().expect("red pull") })
    else {
        panic!("red pull")
    };
    let api::Answer::Comparison { contains_base, .. } =
        ran.forge().read(REPOSITORY, &api::Read::Compare { base: pull.commit, head: main(&ran), page: 1, limit: 32 })
    else {
        panic!("red ancestry")
    };
    assert!(!contains_base, "the red member's head is not an ancestor of main");
    let api::Answer::Tree(main) = ran.forge().read(REPOSITORY, &api::Read::Tree { commit: main(&ran) }) else {
        panic!("main tree")
    };
    assert!(
        !main.iter().any(|file| file.path.as_ref() == b".fake-forge/gate" && file.content.as_ref() == b"fail\n"),
        "the failing member is not on main"
    );
    eprintln!("red story CPU: {} ns (user + system, children included)", cpu() - start);
}

fn before_release(ran: &Ran) {
    for (at, call) in ran.calls().iter().enumerate() {
        if [story::RELEASE + 2, story::RELEASE + 4, story::RELEASE + 6].contains(&at)
            && call.exit == Some(Exit::Code(2))
            && String::from_utf8_lossy(&call.stderr).contains("ended")
        {
            continue;
        }
        assert_eq!(call.exit, Some(Exit::Code(0)), "call {at}: {call:?}");
    }
    assert_eq!(ran.number(2), 1, "first goal");
    assert_eq!(ran.number(8), 6, "second goal");
    for (at, change) in story::BEFORE.into_iter().zip(story::CHANGES) {
        let panel = story::document(ran, at);
        assert_eq!(panel["task"], change);
        assert_eq!(panel["step"], "gating", "reviewed but not ready: {panel}");
        assert_eq!(panel["gates"][0]["verdict"], "approved");
    }
    for (at, change) in [19, 20].into_iter().zip(story::CHANGES) {
        let task = story::document(ran, at);
        assert_eq!(task["chip"]["task"], change);
        let [dependency] = task["dependencies"].as_array().expect("dependencies").as_slice() else {
            panic!("one shared dependency")
        };
        assert_eq!(dependency["live"]["task"], story::LATCH);
    }
    let list = story::document(ran, story::RELEASE - 1);
    assert_eq!(list["landings"], serde_json::json!([]), "no earlier forming decision");
    let latch = story::document(ran, 6);
    assert_eq!(latch["chip"]["task"], story::LATCH);
    assert!(latch["person_revision"].is_object(), "a ctl-answerable person task: {latch}");
}

fn main(ran: &Ran) -> u64 {
    let api::Answer::Commit(commit) =
        ran.forge().read(REPOSITORY, &api::Read::Branch { branch: b"main".as_slice().into() })
    else {
        panic!("main")
    };
    commit
}
fn sha(commit: u64) -> Value {
    use std::fmt::Write as _;
    let mut head = String::new();
    for byte in object_bytes(commit) {
        write!(head, "{byte:02x}").expect("head hex");
    }
    Value::String(head)
}
