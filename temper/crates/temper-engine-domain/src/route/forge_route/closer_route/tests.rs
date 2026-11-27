use super::super::forge_change::{CloseAt, CloseOutdated};
use super::super::tests::{adopted, change_choice_world, change_start_fixture};
use super::*;

const GOAL: u64 = 8;
const GOAL_BRANCH: &[u8] = b"temper/8/g8";
const OUTDATED_CLOSE: u64 = 20;
const AT: CloseAt = CloseAt {
    branch: [0x12; 32],
    landing: [0xab; 32],
    asked: true,
    refused: None,
    outdated: Some(CloseOutdated::Ancestry),
};

#[test]
fn a_catch_up_change_uses_the_outdated_closes_pinned_main() {
    const HALF_SPEND: u64 = 50; // Half of the test delegation's budget.
    const PARAMETERS: u32 = 4; // Repository, goal branch, questions and merge commit.
    let (mut fixture, fixture_env, context) = change_start_fixture();
    assert!(super::super::start_change(&mut fixture, &fixture_env, &context));
    let mut close = fixture.forge.change(context.task).unwrap().clone();
    close.base = GOAL_BRANCH.into();
    close.close_at = Some(AT);
    let (domain, mut env, delegation) = change_choice_world();
    env.limits.tasks.parameters = PARAMETERS;
    let mut origin =
        super::super::ChangeOrigin { authority: delegation.authority, tasks_left: delegation.tasks_left, source: None };
    let merge = tasks::AuthorityExecutor::Job(forge::MERGE_KIND);
    origin.authority.delegation.kinds = Box::new([merge]);
    let step = catch_up::delegate(&domain, &env, &origin, &adopted(1), &close, GOAL, HALF_SPEND).unwrap();
    assert_eq!(step.spec.words.as_ref(), b"Catch goal 8 up from main at abababababab.");
    assert_eq!(
        step.spec.parameters.as_ref(),
        &[
            tasks::Parameter::Resource {
                name: 1,
                connector: domain.config.forge_connector,
                resource: forge::tag(1, 1)
            },
            tasks::Parameter::Bytes { name: 2, value: GOAL_BRANCH.into() },
            tasks::Parameter::Number { name: 14, value: 2 },
            tasks::Parameter::Bytes {
                name: 16,
                value: b"abababababababababababababababababababababababababababababababab".as_slice().into()
            },
        ]
    );
    assert!(step.authority.delegation.kinds.contains(&merge));
}

#[test]
fn an_unavailable_catch_up_explains_what_it_lacks() {
    let merge = [tasks::AuthorityExecutor::Job(forge::MERGE_KIND)];
    for (hooks, job, kinds, expected) in [
        (forge::Hooks::Legacy, false, merge.as_slice(), "the project names no merge_job"),
        (forge::Hooks::Legacy, true, &[], "its authority does not delegate to the merge job"),
        (forge::Hooks::Directory { merge: false, close: true }, true, &merge, "`.temper/hooks/` names no `merge`"),
    ] {
        let lack = catch_up::lack(hooks, job, kinds).expect("catch-up is unavailable");
        let words = catch_up::outdated_words(OUTDATED_CLOSE, AT, CloseOutdated::Ancestry, Err(lack));
        let expected = skein_lib::bytes::concat(&[
            b"the close step's change T20 found the goal's branch at 121212121212 outdated: its ancestry was incomplete; the closer can't catch it up from main: ",
            expected.as_bytes(),
            b". Merge main into the goal's branch and release the closer to close again.",
        ]);
        assert_eq!(words, expected);
    }
    assert_eq!(catch_up::lack(forge::Hooks::Directory { merge: true, close: true }, false, &merge), None);
}

#[test]
fn a_closers_landing_uses_the_goal_title_and_draft() {
    assert_eq!(landing_words(b"TITLE\n\nBuild PATH.\nDraft: PATH").as_ref(), b"TITLE\n\nDraft: PATH");
    assert_eq!(landing_words(b"TITLE\n\nOther words").as_ref(), b"TITLE");
}

#[test]
fn a_changes_task_share_leaves_room_for_the_attempts() {
    for (given, room, bound, ended, expected) in [
        (80, 83, 3, 0, 19),
        (80, 2, 3, 0, 1),
        (8, 83, 3, 0, 8),
        (80, 83, 3, 2, 40),
        (80, 83, 3, 4, 40),
        (80, 83, 255, 0, 1),
    ] {
        assert_eq!(change_tasks(given, room, bound, ended), expected);
    }
}
