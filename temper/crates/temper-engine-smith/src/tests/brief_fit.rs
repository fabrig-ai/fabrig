//! The brief shares smith's host bound with the rest of its charter (agent.md, 4.3).
#![expect(clippy::disallowed_methods, reason = "fixtures validate the retained UTF-8 prefix")]

use super::{assignment, run};
use crate::{charter, fit_brief};
use skein_lib::{Env, Queue, ReplyTo, Time, Token, Wall};

fn selected() -> run::Charter {
    let mut selected = charter(&assignment());
    selected.brief.sections = Box::new([
        run::Section { title: b"Task".as_slice().into(), text: b"Keep this task".as_slice().into() },
        run::Section { title: b"CI".as_slice().into(), text: "λ".repeat(300).into_bytes().into_boxed_slice() },
    ]);
    selected
}

fn limits(selected: &run::Charter, room: u32) -> run::Limits {
    let base = smith_agent_world::LIMITS.run;
    let non_brief = base.host_part.checked_sub(u64::from(run::brief_room(selected, None, &base).unwrap())).unwrap();
    run::Limits { host_part: non_brief.checked_add(u64::from(room)).unwrap(), host_tools: 40, ..base }
}

fn admitted(selected: run::Charter, limits: &run::Limits) {
    assert!(run::brief_bytes(&selected.brief).unwrap() <= u64::from(run::brief_room(&selected, None, limits).unwrap()));
    let mut domain = run::Domain::new(limits, 1);
    let mut out = Queue::uncapped();
    run::step(
        &mut domain,
        &Env { now: Time::ZERO, wall: Wall::EPOCH, limits: *limits },
        run::Event::Start {
            reply_to: ReplyTo::new(Token::new(1)),
            host_run: Token::new(1),
            activation: 1,
            identity: None,
            window: run::Window { turns: 1, bytes: 65_536, largest_turn: 65_536 },
            messages: Box::default(),
            charter: selected,
            workspace: None,
            transcript: None,
            resumed: false,
        },
        &mut out,
    );
    let mut admitted = false;
    while let Some(request) = out.pop() {
        if let run::Request::Admitted { .. } = request {
            admitted = true;
        }
    }
    assert!(admitted, "smith's start check admits the fitted brief");
}

#[test]
fn a_brief_at_its_room_is_unchanged() {
    let mut selected = selected();
    let original = self::selected().brief;
    let limits = limits(&selected, u32::try_from(run::brief_bytes(&original).unwrap()).unwrap());
    fit_brief(&mut selected, None, &limits);
    assert_eq!(selected.brief, original);
    admitted(selected, &limits);
}

#[test]
fn a_brief_one_byte_past_its_room_cuts_only_the_last_section() {
    let mut selected = selected();
    let original = self::selected().brief;
    let room = u32::try_from(run::brief_bytes(&original).unwrap()).unwrap().checked_sub(1).unwrap();
    let limits = limits(&selected, room);
    fit_brief(&mut selected, None, &limits);
    assert_eq!(selected.brief.sections[0], original.sections[0]);
    assert_ne!(selected.brief.sections[1].text, original.sections[1].text);
    assert!(selected.brief.sections[1].text.ends_with(b" bytes; whole: read_job/read_checks]\n"));
    admitted(selected, &limits);
}

#[test]
fn a_brief_past_its_room_cuts_the_last_section_at_a_utf8_boundary() {
    let mut selected = selected();
    let task = self::selected().brief.sections.into_iter().next().unwrap();
    let limits = limits(&selected, 181);
    fit_brief(&mut selected, None, &limits);
    assert_eq!(selected.brief.sections[0], task);
    let ci = &selected.brief.sections[1];
    assert_eq!(ci.title.as_ref(), b"CI");
    let text = core::str::from_utf8(&ci.text).unwrap();
    let prefix = text.trim_start_matches('λ');
    let kept = text.len().checked_sub(prefix.len()).unwrap();
    let lost = 600_usize.checked_sub(kept).unwrap();
    assert_eq!(prefix, format!("\n[cut {lost} bytes; whole: read_job/read_checks]\n"));
    let once = selected.brief.sections[1].text.clone();
    fit_brief(&mut selected, None, &limits);
    assert_eq!(selected.brief.sections[1].text, once, "a fitted brief is stable");
    admitted(selected, &limits);
}

#[test]
fn a_last_section_without_room_for_its_cut_line_is_omitted() {
    let mut selected = selected();
    selected.brief.sections[0].text = b"t".repeat(600).into_boxed_slice();
    selected.brief.sections[1].text = b"x".as_slice().into();
    let limits = limits(&selected, 150);
    fit_brief(&mut selected, None, &limits);
    assert_eq!(selected.brief.sections[0].title.as_ref(), b"Task");
    assert!(selected.brief.sections[0].text.ends_with(b" bytes; whole: the task's activity]\n"));
    assert_eq!(selected.brief.sections.len(), 1);
    admitted(selected, &limits);
}

#[test]
fn cut_lines_do_not_increase_the_charters_admitted_ownership() {
    let mut selected = selected();
    selected.brief.sections[1].text = b"x".as_slice().into();
    let owned = selected.owned_bytes().unwrap();
    let room = u32::try_from(run::brief_bytes(&selected.brief).unwrap()).unwrap().checked_sub(1).unwrap();
    let limits = limits(&selected, room);
    fit_brief(&mut selected, None, &limits);
    assert_eq!(selected.brief.sections.len(), 1);
    assert_eq!(selected.brief.sections[0].text.as_ref(), b"Keep this task");
    assert!(selected.owned_bytes().unwrap() <= owned);
    admitted(selected, &limits);
}

#[test]
fn zero_room_omits_every_section_and_smith_admits_the_empty_brief() {
    let mut selected = selected();
    let limits = limits(&selected, 0);
    fit_brief(&mut selected, None, &limits);
    assert!(selected.brief.sections.is_empty());
    assert_eq!(run::brief_bytes(&selected.brief), Some(0));
    admitted(selected, &limits);
}

#[test]
fn an_omitted_last_section_is_named_when_the_line_fits_after_the_previous_section() {
    let mut selected = selected();
    selected.brief.sections[1].title = b"Pull request".as_slice().into();
    let limits = limits(&selected, 88);
    fit_brief(&mut selected, None, &limits);
    assert_eq!(selected.brief.sections.len(), 1);
    assert_eq!(selected.brief.sections[0].title.as_ref(), b"Task");
    assert_eq!(
        selected.brief.sections[0].text.as_ref(),
        b"Keep this task\n[1 sections omitted: Pull request; whole: the task's activity]\n"
    );
    admitted(selected, &limits);
}

#[test]
fn an_omission_line_that_does_not_fit_leaves_the_previous_section_whole() {
    let mut selected = selected();
    let limits = limits(&selected, 30);
    fit_brief(&mut selected, None, &limits);
    assert_eq!(selected.brief.sections.len(), 1);
    assert_eq!(selected.brief.sections[0].title.as_ref(), b"Task");
    assert_eq!(selected.brief.sections[0].text.as_ref(), b"Keep this task");
    admitted(selected, &limits);
}

#[test]
fn several_omitted_titles_are_listed_in_their_original_order() {
    let mut selected = selected();
    selected.brief.sections = Box::new([
        run::Section { title: b"Task".as_slice().into(), text: b"Keep this task".as_slice().into() },
        run::Section { title: b"A".as_slice().into(), text: b"a".repeat(600).into_boxed_slice() },
        run::Section { title: b"B".as_slice().into(), text: b"b".repeat(600).into_boxed_slice() },
    ]);
    let limits = limits(&selected, 82);
    fit_brief(&mut selected, None, &limits);
    assert_eq!(selected.brief.sections.len(), 1);
    assert_eq!(
        selected.brief.sections[0].text.as_ref(),
        b"Keep this task\n[2 sections omitted: A, B; whole: the task's activity]\n"
    );
    admitted(selected, &limits);
}

#[test]
fn an_omission_line_can_stand_alone_when_no_section_is_kept() {
    let mut selected = selected();
    selected.brief.sections =
        Box::new([run::Section { title: b"Appendix".as_slice().into(), text: b"a".repeat(600).into_boxed_slice() }]);
    let limits = limits(&selected, 65);
    fit_brief(&mut selected, None, &limits);
    assert_eq!(selected.brief.sections.len(), 1);
    assert!(selected.brief.sections[0].title.is_empty());
    assert_eq!(
        selected.brief.sections[0].text.as_ref(),
        b"[1 sections omitted: Appendix; whole: the task's activity]\n"
    );
    assert_eq!(run::brief_bytes(&selected.brief), Some(65));
    admitted(selected, &limits);
}

#[test]
fn every_room_from_zero_through_the_whole_brief_respects_both_bounds() {
    let full = u32::try_from(run::brief_bytes(&selected().brief).unwrap()).unwrap();
    for room in 0..=full {
        let mut selected = selected();
        let owned = selected.owned_bytes().unwrap();
        let limits = limits(&selected, room);
        fit_brief(&mut selected, None, &limits);
        assert!(run::brief_bytes(&selected.brief).unwrap() <= u64::from(room), "room={room}");
        assert!(selected.owned_bytes().unwrap() <= owned, "room={room}");
    }
}

#[test]
fn declarations_past_the_host_bound_leave_the_brief_unchanged() {
    let mut selected = selected();
    let original = self::selected().brief;
    let mut limits = limits(&selected, 0);
    limits.host_part = limits.host_part.checked_sub(1).unwrap();
    assert_eq!(run::brief_room(&selected, None, &limits), None);
    fit_brief(&mut selected, None, &limits);
    assert_eq!(selected.brief, original);
}
