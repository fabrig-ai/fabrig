use hearth_builder_world::fixtures::{collisions, lines_starting};

#[test]
fn a_tree_collision_names_both_tasks_and_exits_2() {
    let rendered = super::render(collisions::TREE_TWICE);
    assert_eq!(
        lines_starting(&rendered.stdout, &["tasks/banner/"]),
        "tasks/banner/task.json:1:12: x1-prod: /etc/motd is given by banner and by motd (tasks/motd/tree/etc/motd)\n"
    );
    assert_eq!(rendered.code, Some(2));
}

#[test]
fn a_file_under_a_data_path_is_refused_and_exits_2() {
    let rendered = super::render(collisions::HIDDEN);
    assert_eq!(
        lines_starting(&rendered.stdout, &["tasks/app/"]),
        "tasks/app/task.json:1:11: x1-prod: /var/lib/app/seed is under the data path /var/lib/app (a mount would hide it) (tasks/dump/tree/var/lib/app/seed)\n"
    );
    assert_eq!(rendered.code, Some(2));
}

#[test]
fn a_differing_user_names_both_tasks_and_exits_2() {
    let rendered = super::render(collisions::CLASH);
    assert_eq!(
        lines_starting(&rendered.stdout, &["tasks/a/"]),
        "tasks/a/task.json:1:12: x1-prod: user \"alice\" differs between a and b (tasks/b/task.json:1:12)\n"
    );
    assert_eq!(rendered.code, Some(2));
}

#[test]
fn a_shared_uid_names_both_users_and_exits_2() {
    let rendered = super::render(collisions::SHARED);
    assert_eq!(
        lines_starting(&rendered.stdout, &["tasks/a/"]),
        "tasks/a/task.json:1:12: x1-prod: uid 1000 is shared by alice and ci (tasks/b/task.json:1:12)\n"
    );
    assert_eq!(rendered.code, Some(2));
}
