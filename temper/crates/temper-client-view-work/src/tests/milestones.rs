#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros,
    reason = "test fixtures generate numbered goal names and inspect bounded semantic trees"
)]
use super::fixtures as f;
use crate::{
    look,
    pages::{goals::Names, milestones},
    words::Words,
};
use jig_client::contract as c;
use skein_lib::{List, Wall};
use skein_ui::{Builder, Role};
use temper_client_domain::pages::{
    milestones::{self as pure, Count, Headline, Mark, State},
    report::Report,
};
use temper_client_protocol_wire::v1 as w;

fn line(task: u64, title: &[u8]) -> c::TaskLine {
    let mut line = f::goals::line();
    line.chip.task = task;
    line.chip.title = f::goal::text(title);
    line
}
fn done(task: u64, title: &[u8]) -> c::TaskLine {
    let mut line = line(task, title);
    line.chip.phase = c::Phase::Ended;
    line.chip.details.phase = c::ChipPhase::Ended;
    line.chip.details.details.ending = Some(c::ChipEnding::Done(c::ResultValue::Report(f::goal::text(b"done"))));
    line
}
fn totals(done: u32, failed: u32, live: u32) -> w::MilestoneTotals {
    w::MilestoneTotals { task: 99, done, failed, live, closed: 0, ended: None, last_landing: None, spent: 0 }
}
fn report() -> Report {
    Report {
        headline: b"5 of its 7 changes landed; the other 2 are building.".as_slice().into(),
        paragraphs: List::uncapped(),
        left: None,
        risk: None,
        decided: List::uncapped(),
        at: None,
    }
}
fn word(words: &Words, headline: Headline, lines: &[c::TaskLine], report: Option<&Report>) -> Box<[u8]> {
    milestones::headline(words, headline, lines, report.map(|report| report.headline.as_ref()), &Names::new())
}

#[test]
fn a_strip_is_a_named_figure_with_goal_ids_and_a_terminal_line() {
    let mut goals: Vec<_> = (1..=8).map(|i| done(i, format!("S{i} step").as_bytes())).collect();
    goals[6] = line(7, b"S7 running");
    let count = Count { done: 7, total: 8 };
    let mut b = Builder::new(&f::tree_limits());
    milestones::strip(&mut b, &f::browser_words(), &goals, &[], count);
    let tree = b.finish();
    let nodes = tree.nodes();
    assert_eq!(nodes[0].role, Role::Figure);
    assert_eq!(nodes[0].name.as_deref(), Some(b"7 done of 8".as_slice()));
    assert_eq!(nodes[0].styles.as_slice(), [look::MS_STRIP]);
    let mut segments = Vec::new();
    for (index, node) in nodes.iter().enumerate() {
        if node.styles.as_slice().contains(&look::MS_DONE) || node.styles.as_slice().contains(&look::MS_LIVE) {
            segments.push((node.styles.as_slice(), f::node_text(&tree, index)));
        }
    }
    assert_eq!(segments.len(), 8);
    for (index, (style, title)) in segments.into_iter().enumerate() {
        assert_eq!(style, [if index == 6 { look::MS_LIVE } else { look::MS_DONE }]);
        assert_eq!(title.as_ref(), format!("S{}", index + 1).as_bytes());
    }
    let mut b = Builder::new(&f::tree_limits());
    milestones::strip(&mut b, &f::words(), &goals, &[], count);
    let tree = b.finish();
    assert_eq!(f::text(&tree).as_ref(), "■■■■■■▒■ 7 done of 8".as_bytes());
    let mut print = skein_ui_text::Print::new(&f::tree_limits()).unwrap();
    skein_ui_text::render(&tree, b"temper", None, &mut print);
    assert_eq!(print.text.as_slice(), "== temper == \n■■■■■■▒■ 7 done of 8\n".as_bytes());
    let mut waiting = line(9, b"K9 waits");
    waiting.chip.details.phase = c::ChipPhase::Waiting { dependencies: List::uncapped(), more: false, holds: false };
    let mut failed = done(10, b"K10 failed");
    failed.chip.details.details.ending = Some(c::ChipEnding::Failed(f::goal::text(b"failed")));
    let goals = [line(7, b"S7 needs"), waiting, failed];
    assert_eq!(
        pure::strip(&goals, &[7]).as_slice().iter().map(|s| s.mark).collect::<Vec<_>>(),
        [Mark::Needs, Mark::Waits, Mark::Failed]
    );
    for words in [f::browser_words(), f::words()] {
        let mut b = Builder::new(&f::tree_limits());
        milestones::strip(&mut b, &words, &goals, &[7], Count { done: 0, total: 3 });
        let tree = b.finish();
        if words.shell == crate::words::Shell::Terminal {
            assert_eq!(f::text(&tree).as_ref(), "▒□× 0 done of 3".as_bytes());
        } else {
            for style in [look::MS_NEEDS, look::MS_WAITS, look::MS_FAILED] {
                assert!(tree.nodes().iter().any(|n| n.styles.as_slice().contains(&style)));
            }
        }
    }
}

#[test]
fn the_milestone_headline_words_the_held_case_in_both_shells() {
    let mut milestone = line(99, b"Store shape");
    let mut goals: Vec<_> = (1..=8).map(|i| done(i, format!("S{i} step").as_bytes())).collect();
    goals[6] = line(7, b"S7 logs");
    goals[6].last_activity = Wall::from_nanos(3);
    for words in [f::browser_words(), f::words()] {
        let headline = pure::headline(&milestone, &goals, Some(&totals(7, 0, 1)), false);
        assert_eq!(
            word(&words, headline, &goals, Some(&report())).as_ref(),
            b"7 of 8 goals done. S7: 5 of its 7 changes landed; the other 2 are building."
        );
        goals[7] = line(8, b"S8 another");
        let headline = pure::headline(&milestone, &goals, Some(&totals(7, 0, 1)), false);
        assert_eq!(
            word(&words, headline, &goals, Some(&report())).as_ref(),
            b"7 of 8 goals done. S7: 5 of its 7 changes landed; the other 2 are building, and one more running."
        );
        assert_eq!(
            word(
                &words,
                Headline::Running { count: Count { done: 7, total: 8 }, goal: 7, more: 2 },
                &goals,
                Some(&report())
            )
            .as_ref(),
            b"7 of 8 goals done. S7: 5 of its 7 changes landed; the other 2 are building, and 2 more running."
        );
        assert_eq!(
            word(&words, Headline::Running { count: Count { done: 7, total: 8 }, goal: 7, more: 0 }, &goals, None)
                .as_ref(),
            b"7 of 8 goals done. S7: Six of nine tasks done."
        );
        let mut failed = done(5, b"S5 tests");
        failed.chip.details.details.ending = Some(c::ChipEnding::Failed(f::goal::text(b"failed")));
        for (headline, expected) in [
            (
                Headline::Ended { count: Count { done: 7, total: 8 }, failed: 1, first: Some(5) },
                "7 of 8 goals done; S5 failed.",
            ),
            (
                Headline::Ended { count: Count { done: 7, total: 8 }, failed: 1, first: None },
                "7 of 8 goals done; 1 failed.",
            ),
            (
                Headline::Ended { count: Count { done: 7, total: 9 }, failed: 2, first: Some(5) },
                "7 of 9 goals done; 2 failed.",
            ),
            (Headline::Complete(Count { done: 8, total: 8 }), "All 8 goals done."),
            (Headline::Complete(Count { done: 1, total: 1 }), "All 1 goal done."),
            (Headline::Complete(Count { done: 0, total: 0 }), "Complete, with no goals."),
            (Headline::Open(Count { done: 6, total: 6 }), "6 goals done so far; more may join."),
            (Headline::Open(Count { done: 1, total: 1 }), "1 goal done so far; more may join."),
            (Headline::Open(Count { done: 0, total: 0 }), "No goals yet."),
            (Headline::Closed(Count { done: 3, total: 8 }), "Closed by you; 3 of 8 goals done."),
            (Headline::Closed(Count { done: 0, total: 8 }), "Closed by you; None of 8 goals done."),
            (Headline::Count(Count { done: 0, total: 15 }), "None of 15 goals done."),
            (Headline::Count(Count { done: 1, total: 1 }), "1 of 1 goal done."),
        ] {
            assert_eq!(word(&words, headline, core::slice::from_ref(&failed), None).as_ref(), expected.as_bytes());
        }
        goals[7] = done(8, b"S8 done");
        // Done is not complete before totals have answered.
        milestone.chip.phase = c::Phase::Ended;
        milestone.chip.details.phase = c::ChipPhase::Ended;
        milestone.chip.details.details.ending =
            Some(c::ChipEnding::Done(c::ResultValue::Report(f::goal::text(b"done"))));
        assert_eq!(pure::headline(&milestone, &goals, None, false), Headline::Count(Count { done: 7, total: 8 }));
        milestone = line(99, b"Store shape");
    }
}

#[test]
fn a_milestone_row_links_its_top_strip_and_headline_in_both_shells() {
    use crate::bindings::Bindings;
    use temper_client_domain::{
        Action, Address,
        pages::{Act, goals::Goals},
    };
    let mut goals = Vec::new();
    for task in 1..=8 {
        let mut goal = if task == 7 { line(task, b"S7 logs") } else { done(task, b"S1 done") };
        if task != 7 {
            goal.ended_at = Some(Wall::EPOCH);
        }
        goals.push((goal, None));
    }
    let mut listed = line(99, b"Store shape");
    listed.parts = List::from_boxed(Box::new([f::milestones::part(None, totals(7, 0, 1), &goals, 0)]));
    let limits = f::domain_limits();
    let link = jig_client_protocol_link::Limits::new(limits.client.domain, limits.client.exchanges).unwrap();
    let milestone = pure::Milestone::from_line(&listed, &link);
    for ended in [false, true] {
        let listed = if ended { done(99, b"Store shape") } else { line(99, b"Store shape") };
        for words in [f::browser_words(), f::words()] {
            let mut b = Builder::new(&f::tree_limits());
            let mut bindings = Bindings::new(f::tree_limits().bindings);
            milestones::row(&mut b, &words, &mut bindings, &listed, &milestone, &Goals::new(1), &Names::new());
            let tree = b.finish();
            let links: Vec<_> = tree.nodes().iter().filter(|n| n.role == Role::Link).collect();
            assert_eq!(links.len(), 1);
            let link = links[0];
            let expected = if ended {
                "milestone · Store shape · 7 / 8 done · complete"
            } else {
                "milestone · Store shape · 7 / 8 done · running"
            };
            assert_eq!(link.name.as_deref(), Some(expected.as_bytes()));
            assert!(link.styles.as_slice().contains(&look::GOALS_ROW));
            assert!(link.styles.as_slice().contains(&look::MS_ROW));
            assert_eq!(link.styles.as_slice().contains(&look::GOALS_ROW_ENDED), ended);
            assert_eq!(
                bindings.decode(link.binding.unwrap().token, skein_ui::Act::Press),
                Some(Action::Pages(Act::Go { address: Address::Task { task: 99 } }))
            );
            let text = f::text(&tree);
            let headline = if ended { "All 8 goals done." } else { "7 of 8 goals done. S7: Six of nine tasks done." };
            assert!(skein_lib::bytes::find(&text, headline.as_bytes()).is_some());
            if words.shell == crate::words::Shell::Browser {
                assert!(
                    tree.nodes().iter().any(|n| n.role == Role::Figure && n.name.as_deref() == Some(b"7 done of 8"))
                );
                let index = tree.nodes().iter().position(|n| n.styles.as_slice().contains(&look::GOALS_TOP)).unwrap();
                assert!(f::node_text(&tree, index).starts_with(b"milestoneStore shape"));
            } else {
                let mut print = skein_ui_text::Print::new(&f::tree_limits()).unwrap();
                skein_ui_text::render(&tree, b"temper", None, &mut print);
                let text = core::str::from_utf8(print.text.as_slice()).unwrap();
                assert!(text.contains(expected));
                assert!(text.contains("■■■■■■▒■ 7 done of 8\n"));
                assert!(text.contains(headline));
            }
        }
    }
}

#[test]
fn milestone_tag_count_and_state_words_keep_their_styles() {
    for words in [f::browser_words(), f::words()] {
        assert_eq!(milestones::count(&words, Count { done: 7, total: 8 }).as_ref(), b"7 / 8 done");
        let mut b = Builder::new(&f::tree_limits());
        milestones::tag(&mut b, &words);
        let tree = b.finish();
        assert_eq!(tree.nodes()[0].styles.as_slice(), [look::MS_TAG]);
        assert_eq!(
            f::text(&tree).as_ref(),
            if words.shell == crate::words::Shell::Browser {
                b"milestone".as_slice()
            } else {
                "milestone · ".as_bytes()
            }
        );
        for (state, text, style) in [
            (State::Closed, "closed", look::GOALS_STATE_CLOSED),
            (State::Complete, "complete", look::GOALS_STATE_LANDED),
            (State::Ended, "ended", look::GOALS_STATE_CLOSED),
            (State::Done, "done", look::GOALS_STATE_LANDED),
            (State::Needs, "needs you", look::GOALS_STATE_NEEDS),
            (State::Paused, "paused by you", look::GOALS_STATE_PAUSED),
            (State::Running, "running", look::GOALS_STATE_RUNNING),
            (State::Waits, "waits", look::GOALS_STATE_WAITS),
            (State::Open, "open", look::GOALS_STATE_PLANNING),
        ] {
            let (word, token) = milestones::state_word(&words, state);
            assert_eq!(word.as_ref(), text.as_bytes());
            assert_eq!(token, style);
        }
    }
}

#[test]
fn milestone_rows_cut_the_title_at_eighty_but_goal_names_at_forty() {
    for (title, cut) in f::row_titles() {
        let mut listed = line(99, title);
        listed.parts =
            List::from_boxed(Box::new([f::milestones::part(None, totals(0, 0, 1), &[(line(7, title), None)], 0)]));
        // The part's goal line carries the whole title, past the kit's text bound.
        let mut limits = f::domain_limits();
        limits.client.domain.text = 1024;
        limits.client.domain.long_text = 1024;
        limits.client.domain.turn_text = 1024;
        let link = jig_client_protocol_link::Limits::new(limits.client.domain, limits.client.exchanges).unwrap();
        let milestone = pure::Milestone::from_line(&listed, &link);
        let goal_name = crate::common::chat::cut_title(title, 40, false);
        for words in [f::browser_words(), f::words()] {
            let mut b = Builder::new(&f::tree_limits());
            let mut bindings = crate::bindings::Bindings::new(f::tree_limits().bindings);
            milestones::row(
                &mut b,
                &words,
                &mut bindings,
                &listed,
                &milestone,
                &temper_client_domain::pages::goals::Goals::new(1),
                &Names::new(),
            );
            let tree = b.finish();
            let link = tree.nodes().iter().find(|node| node.role == Role::Link).unwrap();
            assert!(skein_lib::bytes::find(link.name.as_deref().unwrap(), cut).is_some());
            assert!(skein_lib::bytes::find(&f::text(&tree), cut).is_some());
            let headline =
                tree.nodes().iter().position(|node| node.styles.as_slice().contains(&look::GOALS_LINE)).unwrap();
            assert!(skein_lib::bytes::find(&f::node_text(&tree, headline), &goal_name).is_some());
            assert!(goal_name.len() < cut.len());
        }
    }
}
