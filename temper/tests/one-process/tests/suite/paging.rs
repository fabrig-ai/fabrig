#[test]
fn small_template_pages_keep_every_activity_and_live_delegate() {
    let seen = temper_one_process_world::lower::paging(7);
    assert!(seen.pages >= 2);
    assert!(seen.positions.len() >= 3);
    let tree = seen.tree.split_once("tree:\n").expect("tree").1.split_once("digest:").expect("digest").0;
    assert_eq!(tree.lines().filter(|line| line.contains(" · ")).count(), 3);
    assert!(tree.starts_with("1 · "));
    assert!(tree.contains("cut: 1 more not shown (1 Waiting); read on with: task list --project 1 --parent 1\n"));
    assert_eq!(seen.delegates, [4, 5, 6]);
}
