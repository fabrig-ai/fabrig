use super::*;
use skein_lib::{Duration, Time};

const LATER: Token = Token::new(246);

fn planned(world: &mut World, section: Token, pull: u64, head: api::Commit) -> Time {
    let start = world.now();
    world.event(top::Event::PlanBrief {
        section,
        partner: None,
        source: top::BriefSource::Pull {
            item: top::BriefItem { repository: REPO.repository, number: pull },
            head: top::BriefCommit(head),
        },
    });
    world.event(top::Event::GatherPlanned {
        task: 9,
        section,
        parts: 1,
        bytes: BYTES,
        ci_budget: BYTES,
        deadline: start.saturating_add(Duration::from_secs(30)),
    });
    start.saturating_add(Duration::from_millis(22_500))
}

fn sizes(world: &World, section: Token) -> Vec<Result<u32, top::BriefUnread>> {
    world
        .seen()
        .iter()
        .filter_map(|request| match request {
            top::Request::BriefSized { section: found, size } if *found == section => Some(size.clone()),
            _ => None,
        })
        .collect()
}

fn until_sized(world: &mut World, section: Token, end: Time) -> Time {
    for _ in 0..20_000 {
        if !sizes(world, section).is_empty() {
            return world.now();
        }
        assert!(world.step_until(end), "section not sized by horizon: {:?}", world.seen());
    }
    panic!("section did not size")
}

fn words(world: &mut World, section: Token) -> String {
    let sizes = sizes(world, section);
    assert_eq!(sizes.len(), 1, "one terminal size");
    assert!(sizes[0].as_ref().expect("pull section kept") <= &BYTES);
    world.event(top::Event::TakeBrief { section });
    let words = world
        .seen()
        .iter()
        .find_map(|request| match request {
            top::Request::BriefTaken { section: found, bytes } if *found == section => bytes.as_deref(),
            _ => None,
        })
        .expect("taken text");
    assert_eq!(words.len(), usize::try_from(*sizes[0].as_ref().expect("successful size")).expect("bounded size"));
    std::str::from_utf8(words).expect("brief text").to_owned()
}

fn refusal(world: &World, stage: &'static str) {
    assert!(
        world.seen().contains(&top::Request::BriefReadRefused {
            section: SECTION,
            stage,
            reason: "not read by the section's cutoff",
            heads: None,
            error: None,
        }),
        "{:?}",
        world.seen()
    );
}

fn files_notice() -> &'static str {
    "[files omitted from page 1: not read by the section's cutoff; the pull changes 2 files; read_pull_files from page 1 lists the rest, and read_file reads a path at the head or the base]"
}

#[test]
fn oversized_files_with_a_deadline_keep_the_count_and_fast_comparison() {
    let (mut world, pull, head, base) = resolver(3, 0, false);
    world.take_seen();
    world.take_calls();
    world.reject_file_sides(api::Excess::at_least(api::Bound::AnswerBytes, 4096, 4097));
    let cutoff = planned(&mut world, SECTION, pull, head);
    assert!(until_sized(&mut world, SECTION, cutoff) < cutoff);
    let calls = world.take_calls();
    assert!(calls.contains(&api::Op::Read(api::Read::Compare { before: head, after: base })));
    assert_eq!(calls.iter().filter(|op| matches!(op, api::Op::Read(api::Read::PullFiles { .. }))).count(), 1);
    let words = words(&mut world, SECTION);
    assert!(words.contains("[files omitted from page 1: past forge.client.answer_bytes (4096): at least 4097 bytes; the pull changes 2 files; read_pull_files from page 2 lists the rest, and read_file reads a path at the head or the base]"), "{words}");
    assert!(words.contains("What landed in the base: 2 commits"), "{words}");
}

#[test]
fn slow_files_are_sized_at_the_cutoff_without_comparing() {
    let (mut world, pull, head, _) = resolver(3, 0, false);
    world.take_seen();
    world.take_calls();
    let cutoff = planned(&mut world, SECTION, pull, head);
    world.run_until_read(&api::Read::PullFiles { number: pull, head, page: 1, skip: 0, sides: true });
    world.slow_calls(Duration::from_secs(40));
    assert_eq!(until_sized(&mut world, SECTION, cutoff), cutoff);
    refusal(&world, "files");
    assert!(!world.take_calls().iter().any(|op| matches!(op, api::Op::Read(api::Read::Compare { .. }))));
    let words = words(&mut world, SECTION);
    assert!(words.contains(files_notice()), "{words}");
}

#[test]
fn slow_comparison_is_sized_at_the_cutoff() {
    let (mut world, pull, head, base) = resolver(3, 0, false);
    world.take_seen();
    world.take_calls();
    let cutoff = planned(&mut world, SECTION, pull, head);
    world.run_until_read(&api::Read::Compare { before: head, after: base });
    world.slow_calls(Duration::from_secs(40));
    assert_eq!(until_sized(&mut world, SECTION, cutoff), cutoff);
    refusal(&world, "compare");
    let words = words(&mut world, SECTION);
    assert!(words.contains("File: shared"), "{words}");
    assert!(words.contains("[comparison omitted: not read by the section's cutoff]"), "{words}");
}

#[test]
fn an_answer_after_the_cutoff_does_not_size_again_or_touch_a_later_section() {
    let (mut world, pull, head, _) = resolver(3, 0, false);
    world.take_seen();
    world.take_calls();
    let cutoff = planned(&mut world, SECTION, pull, head);
    world.run_until_read(&api::Read::PullFiles { number: pull, head, page: 1, skip: 0, sides: true });
    world.slow_calls(Duration::from_secs(40));
    until_sized(&mut world, SECTION, cutoff);
    // The parent's slab gives the next section a distinct generational token.
    world.slow_calls(Duration::from_secs(1));
    let next_cutoff = planned(&mut world, LATER, pull, head);
    until_sized(&mut world, LATER, next_cutoff);
    let later = words(&mut world, LATER);
    assert!(later.contains("What landed in the base: 2 commits"), "{later}");
    world.run_for(60);
    assert_eq!(sizes(&world, SECTION).len(), 1);
    assert_eq!(sizes(&world, LATER).len(), 1);
    let first = words(&mut world, SECTION);
    assert!(first.contains(files_notice()), "{first}");
    assert!(!first.contains("File: shared"), "late files were dropped: {first}");
}

#[test]
fn a_pull_answered_after_the_cutoff_starts_no_files_read() {
    let (mut world, pull, head, _) = resolver(3, 0, false);
    world.take_seen();
    world.take_calls();
    let cutoff = planned(&mut world, SECTION, pull, head);
    world.run_until_read(&api::Read::Pull { number: pull, ci: false });
    world.slow_calls(Duration::from_secs(40));
    world.run_for(23);
    assert!(sizes(&world, SECTION).is_empty(), "identity reads have no cutoff");
    let at = until_sized(&mut world, SECTION, cutoff.saturating_add(Duration::from_secs(40)));
    assert!(at > cutoff);
    refusal(&world, "files");
    let calls = world.take_calls();
    assert!(
        !calls.iter().any(|op| matches!(op, api::Op::Read(api::Read::PullFiles { .. } | api::Read::Compare { .. })))
    );
    let words = words(&mut world, SECTION);
    assert!(words.contains(files_notice()), "{words}");
}
