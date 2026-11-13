//! Repair causes after a clean update (temper/docs/design/domain/forge/change/repairs.md and temper/docs/design/domain/forge/README.md, 6).
use super::light_rules::{change_row, pushed_change_in};
use super::*;
use temper_engine_domain_forge_change::{Delegate, Repair, State};

const STEPS: u32 = 1000;
const AFTER_UPDATE: &str =
    "CI failed after a clean update from the base, and no merge conflict waits. Repair what the failed check says.";
const PRESERVE: &str = " Preserve the requested change and unrelated base content, including both independent appended lines and existing blank lines; do not remove content to make a check pass.";

fn updated_change() -> (World, u64, u64, u64) {
    let small = temper_engine_service::limits::profile(temper_engine_service::limits::Profile::Small);
    let world = World::configured_limits(
        true,
        true,
        false,
        0,
        Gate::default(),
        None,
        StoryLimits { brief: Some(small.root.brief), result_bytes: Some(512), ..StoryLimits::default() },
    );
    let (mut world, task, old, base) = pushed_change_in(world, false, b"main");
    for commit in [1, base] {
        assert_eq!(
            world.external(
                1,
                raw::Op::Write(raw::Write::Status {
                    commit,
                    context: b"build".as_slice().into(),
                    state: raw::Check::Passed,
                    description: Box::new([]),
                    link: Box::new([]),
                })
            ),
            raw::Answer::Done
        );
    }
    for _ in 0..STEPS {
        world.tick();
        if world.fake.branch(b"org/repo", &change_row(&world, task).branch).is_some_and(|head| head != old) {
            break;
        }
    }
    let row = change_row(&world, task);
    let updated = world.fake.branch(b"org/repo", &row.branch).expect("updated branch");
    assert_ne!(updated, old);
    assert_eq!(row.change.updates, 1);
    (world, task, updated, base)
}

fn repair(world: &mut World, task: u64, head: u64, omit_checks: bool) -> engine::Assignment {
    let mut omitted = 0;
    for _ in 0..STEPS {
        // The pull still reports red, but the detailed checks read has no names.
        if omit_checks && change_row(world, task).delegate.is_none() {
            for event in &mut world.events {
                if let engine::Event::Forge(event) = event
                    && let forge_top::Event::Client(skein_forge_client::Event::Answered { result, .. }) = event.as_mut()
                    && let Ok(skein_forge::api::Answer::Checks(checks)) = result
                    && checks.iter().any(|check| check.job.is_some_and(|job| job.head == translate::commit(head)))
                {
                    *checks = Box::new([]);
                    omitted += 1;
                }
            }
        }
        world.tick();
        if let Some((child, Delegate::Repair(Repair::Semantic))) = change_row(world, task).delegate
            && let Some(run) = world.assigned.iter().find(|run| run.task == child)
        {
            assert!(matches!(change_row(world, task).change.state, State::Repairing { why: Repair::Semantic, .. }));
            assert_eq!(change_row(world, task).change.resolutions, 0);
            assert_eq!(omitted > 0, omit_checks);
            return run.clone();
        }
    }
    panic!("clean update delegates its semantic repair: {:?}", change_row(world, task));
}

fn section(run: &engine::Assignment, kind: engine::ForgeBriefKind) -> &str {
    let wanted = engine::BriefKind::Forge(kind);
    let section = run.sections.iter().find(|section| section.kind == wanted).expect("forge section");
    let engine::BriefBody::Text(words) = &section.body else { panic!("readable forge section") };
    core::str::from_utf8(words).expect("brief text")
}

#[test]
fn a_clean_updates_red_names_its_check_and_no_conflict() {
    let (mut world, task, head, _) = updated_change();
    let run = repair(&mut world, task, head, false);
    let ci = section(&run, engine::ForgeBriefKind::Ci);
    assert!(ci.contains("Failed check: build\n"), "{ci}");
    assert!(ci.contains("Failed step: 1 Run build\nEnd of its log:\n"), "{ci}");
    assert!(ci.contains("failing check build "), "{ci}");
    assert!(ci.ends_with("\nRepair the failed check.\n"), "{ci}");
    let hex = engine::commit_text(&translate::commit(head));
    let short = core::str::from_utf8(&hex[..12]).expect("short head");
    let words = core::str::from_utf8(&live_task(&world, run.task).spec.words).expect("repair words");
    assert_eq!(words, format!("Repair CI at {short}: build\n{AFTER_UPDATE}{PRESERVE}"));
    let pull = section(&run, engine::ForgeBriefKind::Pull);
    assert!(pull.contains("Base: main\n"), "{pull}");
    assert!(pull.contains("What landed in the base: 2 commits\n"), "{pull}");
    assert!(pull.contains("Base file: other\n"), "{pull}");
    let ci_at =
        run.sections.iter().position(|s| s.kind == engine::BriefKind::Forge(engine::ForgeBriefKind::Ci)).expect("CI");
    let pull_at = run
        .sections
        .iter()
        .position(|s| s.kind == engine::BriefKind::Forge(engine::ForgeBriefKind::Pull))
        .expect("pull");
    assert!(ci_at < pull_at, "the reason precedes the base contents");
    world.close();
}

#[test]
fn a_clean_updates_red_with_no_check_read_says_so() {
    let (mut world, task, head, _) = updated_change();
    let run = repair(&mut world, task, head, true);
    let hex = engine::commit_text(&translate::commit(head));
    let short = core::str::from_utf8(&hex[..12]).expect("short head");
    assert_eq!(
        live_task(&world, run.task).spec.words.as_ref(),
        format!("Repair CI at {short}: no failed check named\n{AFTER_UPDATE}{PRESERVE}").as_bytes()
    );
    world.close();
}

#[test]
fn a_resolutions_words_say_conflict() {
    let world = World::configured_limits(true, true, true, 1000, Gate::default(), None, StoryLimits::default());
    let (mut world, task, _, _) = pushed_change_in(world, true, b"main");
    let run = super::reviews::next_assignment(&mut world, 2);
    assert!(matches!(change_row(&world, task).delegate, Some((child, Delegate::Resolve { .. }) ) if child == run.task));
    assert!(live_task(&world, run.task).spec.words.starts_with(b"Resolve the merge conflict."));
    let pull = section(&run, engine::ForgeBriefKind::Pull);
    assert!(pull.contains("\nResolve the conflict from change head "), "{pull}");
    world.close();
}
