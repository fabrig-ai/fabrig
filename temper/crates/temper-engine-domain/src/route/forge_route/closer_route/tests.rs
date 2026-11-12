use super::*;

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
