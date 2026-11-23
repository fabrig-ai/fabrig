//! The synthetic billing envelope must mark the account, not the task.
//! Contract: temper/docs/design/draft/failure-worlds.md, section 7 (P72).
use smith_agent_process_world::{
    charter, program,
    program_world::{self, Case},
};
use smith_channel::{ModelFault, Record as _, RunFailure, RunResult};
use smith_fake_claude::{End, Story, Turn};

#[test]
#[ignore = "provider failures: classify the program billing envelope as an account refusal"]
fn a_billing_refusal_marks_the_account() {
    let charter = charter();
    let decoded = smith_charter::Charter::decode(&smith_charter::CEILINGS, &mut skein_lib::Reader::new(&charter))
        .expect("fixture charter");
    let account = decoded.main.account.expect("run account");
    let story = Story { turns: vec![Turn { steps: vec![], end: End::billing_refusal() }], ..Story::default() };
    let run = program_world::run(Case::Events, &story, Some(b"claude.ai"), &charter, &program(b"source"), None);
    assert_eq!(run.invocations.len(), 2, "one login check and one program, no restart");
    assert_eq!(run.received.len(), 1, "no call retry");
    let answer = run.answer().expect("run terminal");
    let notices: Vec<_> =
        run.frames.iter().filter(|frame| frame.kind == smith_channel::kind::ACCOUNT_REFUSED).collect();
    assert!(
        matches!(&answer.result, RunResult::Failed(failed) if matches!(&failed.reason,
            RunFailure::Model(model) if matches!(&model.value, ModelFault::Refused(refused) if refused.account == account))),
        "billing must refuse account {account}, answer={answer:?}, refused notices={}",
        notices.len(),
    );
    assert_eq!(notices.len(), 1, "one account-refused notice reaches the host");
    let notice =
        smith_channel::AccountRefused::decode(&smith_channel::CEILINGS, &mut skein_lib::Reader::new(&notices[0].body))
            .expect("account refusal notice");
    assert_eq!(notice.account, account);
}
