//! P53: delivery must account for restoring a base deletion.
//! temper/docs/design/domain/forge/change/repairs.md, A resolution.
use super::*;
use crate::forge_routes::{light_rules::pushed_change_in, merge_job};

const GOAL: &[u8] = b"temper/goal";
const PATH: &[u8] = b"file";
const SETTLE_TICKS: u32 = 200;

#[test]
#[ignore = "a change's reds: P53, unreported restoration of base deletion"]
fn a_resolution_keeping_a_deleted_path_is_refused() {
    let mut world = merge_job::world(false, true, true).with_merge_job(b".temper/merge");
    fake::advance(&mut world.fake, &world.fake_env, b"org/repo", b"main", PATH, b"original", 1)
        .expect("shared ancestor contains the path");
    let (mut world, task, head, before) = pushed_change_in(world, true, GOAL);
    let base = fake::commit(&mut world.fake, &world.fake_env.limits, before, Box::new([]), b"Delete file")
        .expect("base deletion")
        .expect("different tree");
    assert_eq!(
        world.external(1, raw::Op::Git(raw::Git::Push { branch: GOAL.into(), commit: base, expected: Some(before) })),
        raw::Answer::Pushed(raw::Pushed::Pushed)
    );
    let job = merge_job::next_job(&mut world, task, false);
    merge_job::check_job(&world, task, &job, head, base);
    let left = format!(
        "{{\"settled\":[],\"left\":[{{\"path\":\"file\",\"kind\":\"deleted\",\"by\":\"base\",\"commit\":\"{}\"}}]}}",
        String::from_utf8(forge::commit_text(&translate::commit(base)).to_vec()).expect("commit text")
    );
    merge_job::finish_job(&mut world, &job, left.as_bytes());
    let resolution = merge_job::next_resolution(&mut world, task);
    assert!(
        task_row(&world, resolution.task)
            .expect("resolver")
            .spec
            .words
            .windows(left.len())
            .any(|w| w == left.as_bytes()),
        "resolver was told about the deletion"
    );
    let merged = merge_job::push_merge_tree(
        &mut world,
        merge_job::Writer::Run(&resolution),
        head,
        base,
        Box::new([raw::File { path: PATH.into(), content: b"producer".as_slice().into() }]),
    );
    // Deliberately omit the restored path and its justification from delivery.
    merge_job::finish_resolution_words(&mut world, &resolution, b"Resolved the conflict; checks pass.");
    world.advance_ticks(SETTLE_TICKS);
    assert!(
        !matches!(change_row(&world, task).change.state, change::State::Landed { .. }),
        "delivery restored file without an account, but landed at {merged}"
    );
    assert!(
        !world.writes.iter().any(|write| matches!(write,
        skein_forge::api::Write::Merge { head, .. } if *head == translate::commit(merged))),
        "unaccounted restoration must be refused before its merge"
    );
    world.close();
}
