//! Dropped asks reach the change's own goal (temper/docs/design/domain/landing-members.md, 3.4).
use super::super::{Domain, Work, forge, tasks};
use super::{adopted, change_row, forge_request, procedure_root};

const CHANGE: u64 = 7;
const GOAL: u64 = 9;
const MESSAGE_BYTES: u32 = 16; // "change 7: pre..."
const WORDS: &[u8] = b"change 7: prepare for the old head dropped";

fn route_drop(goal: Option<u64>) -> Domain {
    let (mut domain, mut env) = procedure_root();
    env.limits.tasks.message_bytes = MESSAGE_BYTES;
    let mut row = change_row(None);
    row.repository = adopted(1).provider;
    row.goal = goal;
    super::restore_forge_with_engine_waits(&mut domain, &env, forge::Stored::Change(Box::new(row)));
    forge_request(&mut domain, &env, forge::Request::AskDropped { change: CHANGE, words: WORDS.into() });
    domain
}

#[test]
fn dropped_ask_reaches_the_goal_as_bounded_kept_news() {
    let mut domain = route_drop(Some(GOAL));
    let Some(Work::Tasks(tasks::Event::Notice { task, word })) = domain.work.pop() else { panic!("goal news") };
    assert_eq!(task, GOAL);
    assert_eq!(word.kind, tasks::MessageKind::News { subscription: 0, class: tasks::NewsClass::Kept });
    assert_eq!(word.words.as_ref(), b"change 7: pre...");
}

#[test]
fn a_change_without_a_goal_tells_no_one() {
    let mut domain = route_drop(None);
    assert!(domain.work.pop().is_none());
}
