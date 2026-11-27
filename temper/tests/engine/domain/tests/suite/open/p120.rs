//! P120: the latest authorized amendment must reach a released resolution.
//! temper/docs/design/domain/forge/change/questions.md, 1, What every delegate is told.
use super::*;
use crate::forge_routes::{light_rules::pushed_change_in, merge_job, reviews};

const SETTLE_TICKS: u32 = 1000;
const OLD: &[u8] = b"Keep file when resolving this change.";
const NEW: &[u8] = b"Delete file when resolving this change. The former keep ruling is superseded.";

fn amend(world: &mut World, task: u64, key: u8, words: &[u8]) {
    let revision = task_row(world, task).expect("change task").revisions.spec;
    let requester = world.assigned.first().expect("requester").clone();
    let answer = world.scripted_call(
        &requester,
        u64::from(key),
        engine::Tool::Amend {
            target: task,
            revision,
            keep_parameters: true,
            amendment: temper_engine_domain_world::fixtures::task_amendment(Some(words), None, words),
        },
    );
    assert!(matches!(answer, engine::CallAnswer::Amended { .. }), "authorized amendment: {answer:?}");
    world.advance_ticks(20);
    assert_eq!(task_row(world, task).expect("amended task").revisions.spec, revision + 1);
}

#[test]
fn the_newest_ruling_reaches_the_resolution() {
    let (mut world, task, head, base) = pushed_change_in(merge_job::world(false, false, false), true, b"temper/goal");
    amend(&mut world, task, 94, OLD);
    let first = merge_job::next_resolution(&mut world, task);
    assert!(task_row(&world, first.task).expect("first resolution").spec.words.windows(OLD.len()).any(|w| w == OLD));
    reviews::finish_review(
        &mut world,
        &first,
        tasks::TaskResult::Failure {
            reason: b"Cannot resolve until the conflicting keep/delete rulings are settled.".as_slice().into(),
        },
    );
    for _ in 0..SETTLE_TICKS {
        world.tick();
        if matches!(task_row(&world, task).expect("change").phase, tasks::Phase::Held { .. }) {
            break;
        }
    }
    assert!(matches!(
        change_row(&world, task).change.state,
        change::State::Held { hold: change::Hold { cause: change::Cause::Failed, .. }, .. }
    ));
    amend(&mut world, task, 95, NEW);
    let revision = task_row(&world, task).expect("held change").revisions.hold;
    person(&mut world, 96, people::Ask::Release { project: 1, task, revision, again: None });
    for _ in 0..SETTLE_TICKS {
        if world.assigned.len() > 3 {
            break;
        }
        world.tick();
    }
    let next = world.assigned.get(3).cloned().unwrap_or_else(|| {
        panic!(
            "fresh resolution: {:?}; task: {:?}; errors: {:?}",
            change_row(&world, task),
            task_row(&world, task),
            world.call_errors
        )
    });
    assert_ne!(next.task, first.task, "release retires the failed resolution");
    let words = &task_row(&world, next.task).expect("fresh resolution").spec.words;
    assert!(words.ends_with(NEW), "the current spec must be the newest ruling: {}", String::from_utf8_lossy(words));
    // The scripted resolver follows only the current ruling, deleting file.
    let merged = merge_job::push_merge_tree(
        &mut world,
        merge_job::Writer::Run(&next),
        head,
        base,
        Box::new([raw::File {
            path: b"resolution".as_slice().into(),
            content: b"new ruling followed".as_slice().into(),
        }]),
    );
    merge_job::finish_resolution_words(&mut world, &next, b"Deleted file as the newest authorized ruling requires.");
    world.advance_ticks(SETTLE_TICKS);
    assert!(
        matches!(change_row(&world, task).change.state, change::State::Landed { .. }),
        "{:?}",
        change_row(&world, task)
    );
    assert!(
        world.writes.iter().any(
            |write| matches!(write, skein_forge::api::Write::Merge { head, .. } if *head == translate::commit(merged))
        ),
        "the resolution following the new ruling landed"
    );
    world.close();
}
