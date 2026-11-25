use hearth_builder_world::{fixtures, protocol};

#[test]
fn a_directory_that_is_no_repository_exits_75() {
    let scenario = fixtures::faulty(fixtures::Fault::NoRepository);
    let run = protocol::run(&scenario, fixtures::SIMULATION_SEED);
    assert_eq!(fixtures::lines_starting(&run.stdout, &["git "]), "git rev-parse exited 128\n");
    assert_eq!(run.code, Some(75), "git failed operationally");
}

#[test]
fn a_tree_git_cannot_read_prints_git_s_tail_and_exits_75() {
    let scenario = fixtures::faulty(fixtures::Fault::UnreadableTree);
    let run = protocol::run(&scenario, fixtures::SIMULATION_SEED);
    assert!(run.vectors.iter().flatten().any(|arg| arg.as_ref() == b"ls-tree"), "git reached ls-tree");
    assert_eq!(
        std::str::from_utf8(&run.stdout).expect("git error text"),
        "git ls-tree exited 128\n  fatal: not a tree object\n"
    );
    assert_eq!(run.code, Some(75));
}

#[test]
fn head_renders_the_later_tree_without_its_removed_paths() {
    let later = fixtures::edited(fixtures::THIN, &[], &[fixtures::MOTD_KIND.path]);
    let scenario = fixtures::Scenario { later: Some(&later), ..fixtures::thin() };
    let run = protocol::run(&scenario, fixtures::SIMULATION_SEED);
    assert_eq!(
        fixtures::lines_starting(&run.stdout, &["  /etc/motd", "  /usr/local/bin/motd"]),
        "  /usr/local/bin/motd      file 0755 root:root f487088ec8b1\n"
    );
}
