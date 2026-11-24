//! Public row and page contracts (temper/docs/design/client/web/public.md, 3 and 4).
#![expect(clippy::disallowed_types, reason = "test oracles collect complete paragraphs as UTF-8 strings")]
#![expect(clippy::disallowed_methods, reason = "test oracles inspect and collect semantic subtrees")]
#![expect(clippy::disallowed_macros, reason = "page test cases build document lists and compose literal row oracles")]
use super::{DELEGATE, PROJECT, TASK, board, records, render};
use crate::public::PublicView;
use jig_client_protocol_wire::Record as _;
use jig_client_protocol_wire::public_v2 as j;
use skein_lib::{List, Writer};
use skein_ui::{Node, Role};
use temper_client_domain::{address::Public, public::Doc};
use temper_client_protocol_wire::public_v1 as f;

const MINUTE: u64 = 60;
const HOUR: u64 = 60 * MINUTE;
const DAY: u64 = 24 * HOUR;
// Preorder subtrees preserve every text node, including separators between links.
fn text(nodes: &[Node]) -> String {
    let mut text = String::new();
    for node in nodes {
        for run in &node.text {
            text.push_str(core::str::from_utf8(&run.text).expect("public words"));
        }
    }
    text
}
fn paragraphs(view: &PublicView) -> Vec<String> {
    let nodes = view.tree().nodes();
    nodes
        .iter()
        .enumerate()
        .filter(|(_, node)| node.role == Role::Paragraph)
        .map(|(at, node)| {
            text(
                &nodes[at..at.checked_add(usize::try_from(node.size).expect("paragraph size")).expect("paragraph end")],
            )
        })
        .collect()
}
fn facts(view: &PublicView) -> Vec<String> {
    let nodes = view.tree().nodes();
    nodes
        .iter()
        .enumerate()
        .filter(|(_, node)| node.styles.as_slice().contains(&crate::look::PUBLIC_FACTS))
        .map(|(at, node)| {
            text(&nodes[at..at.checked_add(usize::try_from(node.size).expect("facts size")).expect("facts end")])
        })
        .collect()
}
fn card(line: j::PublicLine, seconds: u64) -> PublicView {
    render(Public::Task { project: PROJECT, task: TASK }, &[(Doc::JigCard, records::card_line(line))], seconds)
}
fn line(goal: bool, phase: j::PublicPhase, ended: Option<u64>) -> j::PublicLine {
    let mut parts = records::line(goal);
    parts.phase = phase;
    parts.changed = 60;
    parts.ended = ended;
    parts.spent = if goal { Some(126) } else { None };
    // Priority is deliberately present and must never reach the row.
    parts.priority = Some(99);
    parts
}
fn empty_board() -> Box<[u8]> {
    let board = j::PublicBoard {
        project: PROJECT,
        name: b"A public project".as_slice().into(),
        live: List::with_capacity(0),
        live_folded: 0,
        ended: List::with_capacity(0),
    };
    let mut writer = Writer::new(
        usize::try_from(board.measure(&jig_client_protocol_wire::public_v2::CEILINGS).expect("bounded fixture"))
            .expect("board size"),
    );
    board.write(&jig_client_protocol_wire::public_v2::CEILINGS, &mut writer).expect("board");
    writer.finish()
}
#[test]
fn a_goals_row_shows_phase_time_and_spend_and_a_tasks_no_spend() {
    for (goal, expected) in
        [(true, "an agent · running · for 14 minutes · 126 spent"), (false, "an agent · running · for 14 minutes")]
    {
        let page = card(line(goal, j::PublicPhase::Running, None), 900);
        assert_eq!(facts(&page)[0], expected);
        assert!(!text(page.tree().nodes()).contains("99"), "priority is not drawn");
    }
    let board = render(Public::Board { project: PROJECT }, &[(Doc::JigBoard, board())], 840);
    assert_eq!(facts(&board), ["running · for 14 minutes · 12 spent", "running · for 14 minutes", "2 lines folded"]);
}
#[test]
fn an_ended_goal_reads_landed_failed_or_ended_and_a_task_done_failed_or_ended() {
    for (goal, phase, expected) in [
        (true, j::PublicPhase::Done, "an agent · landed · 2 hours ago · 126 spent"),
        (true, j::PublicPhase::Failed, "an agent · failed · 2 hours ago · 126 spent"),
        (true, j::PublicPhase::Cancelled, "an agent · ended · 2 hours ago · 126 spent"),
        (false, j::PublicPhase::Done, "an agent · done · 2 hours ago"),
        (false, j::PublicPhase::Failed, "an agent · failed · 2 hours ago"),
        (false, j::PublicPhase::Cancelled, "an agent · ended · 2 hours ago"),
    ] {
        let page = card(line(goal, phase, Some(600)), 7800);
        assert_eq!(facts(&page)[0], expected);
        assert!(!text(page.tree().nodes()).contains("cancelled"));
    }
}
#[test]
fn an_ended_row_says_how_long_ago() {
    for (ended, expected) in [
        (Some(600), "an agent · landed · 2 hours ago · 126 spent"),
        (None, "an agent · landed · 2 hours 9 minutes ago · 126 spent"),
    ] {
        assert_eq!(facts(&card(line(true, j::PublicPhase::Done, ended), 7800))[0], expected);
    }
}
#[test]
fn each_rows_phase_and_time_read_at_fixed_clocks() {
    use j::PublicPhase::{
        Active, BackingOff, Cancelled, Claimed, Closing, Done, Due, Failed, Held, Idle, Preparing, Running, Waiting,
    };
    // 10 January 1971 at 17:00 UTC. All live phases began three minutes ago.
    const NOW: u64 = 374 * DAY + 17 * HOUR;
    const CHANGED: u64 = NOW - 3 * MINUTE;
    const DONE: (&str, &str) = ("done", "landed");
    let live = [
        (Waiting, "waiting"),
        (Idle, "waiting for news"),
        (Due, "due to run"),
        (Preparing, "preparing a host"),
        (Claimed, "claimed by a host"),
        (Running, "running"),
        (BackingOff, "retrying"),
        (Active, "executor active"),
        (Closing, "closing"),
        (Held, "held"),
    ]
    .map(|(phase, word)| (phase, None, (word, word), "for 3 minutes", false));
    let ended = [
        (Done, Some(NOW), DONE, "just now", true),
        (Failed, Some(NOW - MINUTE), ("failed", "failed"), "a minute ago", false),
        (Cancelled, Some(NOW - 12 * MINUTE), ("ended", "ended"), "12 minutes ago", false),
        (Done, Some(NOW - HOUR), DONE, "an hour ago", false),
        (Done, Some(NOW - 2 * HOUR), DONE, "2 hours ago", false),
        (Done, Some(NOW - HOUR - MINUTE), DONE, "an hour a minute ago", false),
        (Done, Some(NOW - HOUR - 5 * MINUTE), DONE, "an hour 5 minutes ago", false),
        (Done, Some(NOW - 2 * HOUR - MINUTE), DONE, "2 hours a minute ago", false),
        (Done, Some(NOW - 2 * HOUR - 5 * MINUTE), DONE, "2 hours 5 minutes ago", false),
        (Done, Some(NOW - 12 * HOUR), DONE, "at 05:00", false),
        (Done, Some(NOW - DAY), DONE, "yesterday", false),
        (Done, Some(NOW - 2 * DAY), DONE, "Fri", false),
        (Done, Some(NOW - 9 * DAY), DONE, "1 January", false),
        (Done, Some(NOW - 10 * DAY), DONE, "31 December 1970", false),
    ];
    for (phase, ended_at, (task_word, goal_word), time, merged) in live.into_iter().chain(ended) {
        let task = j::PublicLine { changed: CHANGED, ..line(false, phase.clone(), ended_at) };
        let goal =
            j::PublicLine { task: DELEGATE, parent: Some(TASK), changed: CHANGED, ..line(true, phase, ended_at) };
        let mut documents = vec![(Doc::JigCard, records::card_with(task, List::with_capacity(0), goal))];
        if merged {
            documents
                .push((Doc::ForgeChange, records::change_with_step(f::PublicStep::Landed, None, Some(records::HEAD))));
        }
        let page = render(Public::Task { project: PROJECT, task: TASK }, &documents, NOW);
        assert_eq!(
            facts(&page),
            [format!("an agent · {task_word} · {time}"), format!("{goal_word} · {time} · 126 spent")]
        );
        // The landed change is visible as its own paragraph, not as the task's phase.
        assert_eq!(paragraphs(&page).iter().any(|paragraph| paragraph == "landed"), merged);
    }
}

#[test]
fn the_board_says_what_it_is_and_when_it_has_no_goals() {
    let lede = "What temper is doing on this project, as it happens. Read-only.";
    let held = "Work that ended before this project went public, or before temper last restarted, isn't shown.";
    for (body, empty) in [(empty_board(), true), (board(), false)] {
        let page = render(Public::Board { project: PROJECT }, &[(Doc::JigBoard, body)], 0);
        let lines = paragraphs(&page);
        assert_eq!(&lines[..2], [lede, held], "the history limit follows the lede");
        assert_eq!(lines.contains(&"No goals to show yet.".into()), empty);
        let intro = page
            .tree()
            .nodes()
            .iter()
            .position(|node| node.styles.as_slice().contains(&crate::look::PUBLIC_INTRO))
            .expect("board introduction");
        let end = intro
            .checked_add(usize::try_from(page.tree().nodes()[intro].size).expect("intro size"))
            .expect("intro end");
        let intro = &page.tree().nodes()[intro..end];
        let muted: Vec<_> =
            intro.iter().filter(|node| node.styles.as_slice().contains(&crate::look::PUBLIC_LEDE)).collect();
        assert_eq!(muted.len(), 2, "both introduction paragraphs use the lede style");
        assert!(muted.iter().all(|node| node.role == Role::Paragraph));
    }
    let projects = render(Public::Projects, &[(Doc::JigProjects, records::projects_with(false))], 0);
    assert!(paragraphs(&projects).contains(&"No public projects yet.".into()));
    assert!(!text(projects.tree().nodes()).contains(held));
}
#[test]
fn landings_show_below_the_goals_only_with_a_queue_or_a_landing() {
    for (queue, landed, show) in [(false, false, false), (true, false, true), (false, true, true)] {
        let mut page = render(
            Public::Board { project: PROJECT },
            &[(Doc::JigBoard, board()), (Doc::ForgeLandings, records::landings_with(queue, landed))],
            0,
        );
        let nodes = page.tree().nodes();
        let section = nodes.iter().position(|node| node.role == Role::Heading { level: 2 });
        assert_eq!(section.is_some(), show);
        if let Some(at) = section {
            let goal = nodes.iter().position(|node| node.name.as_deref() == Some(b"Goal 9abcdef0")).expect("goal");
            assert!(at > goal, "Landings follows goals");
            assert_eq!(paragraphs(&page).contains(&"No landings to show yet.".into()), !landed);
            let link = super::fixtures::find(page.tree(), Role::Link, b"Landings").expect("Landings link");
            assert_eq!(
                crate::public::href(&mut page, link).as_deref(),
                Some(b"/public/projects/3f2a9c0176543210/landings".as_slice())
            );
        }
        let list = render(
            Public::Landings { project: PROJECT },
            &[(Doc::ForgeLandings, records::landings_with(queue, landed))],
            0,
        );
        assert_eq!(paragraphs(&list).contains(&"No landings to show yet.".into()), !landed);
    }
}
#[test]
fn a_landings_page_reads_whole_in_each_queue_and_merge_form() {
    for (queue, merged, now, expected) in [
        (false, false, 0, "temperSign inLandingsNo landings to show yet."),
        (
            true,
            false,
            0,
            "temperSign inLandingsQueuetask 9abcdef0 · Goal 3f2a9c01 · Ready just nowNo landings to show yet.",
        ),
        (
            true,
            false,
            MINUTE,
            "temperSign inLandingsQueuetask 9abcdef0 · Goal 3f2a9c01 · Ready a minute agoNo landings to show yet.",
        ),
        (
            true,
            false,
            12 * MINUTE,
            "temperSign inLandingsQueuetask 9abcdef0 · Goal 3f2a9c01 · Ready 12 minutes agoNo landings to show yet.",
        ),
        (false, true, 0, "temperSign inLandingslanded · task 3f2a9c01 · 789ada1"),
        (
            true,
            true,
            2 * MINUTE,
            "temperSign inLandingsQueuetask 9abcdef0 · Goal 3f2a9c01 · Ready 2 minutes agolanded · task 3f2a9c01 · 789ada1",
        ),
    ] {
        let page = render(
            Public::Landings { project: PROJECT },
            &[(Doc::ForgeLandings, records::stock_landings(queue, merged))],
            now,
        );
        assert_eq!(text(page.tree().nodes()), expected);
    }
}

#[test]
fn a_queue_merges_line_and_landing_read_as_a_landing() {
    let (list, landing) = records::queue_merge();
    let mut list = render(Public::Landings { project: PROJECT }, &[(Doc::ForgeLandings, list)], 0);
    assert_eq!(paragraphs(&list), ["landed · task 3f2a9c01 · 789ada1"]);
    for name in [b"task 3f2a9c01".as_slice(), b"789ada1"] {
        let link = super::fixtures::find(list.tree(), Role::Link, name).expect("landing link");
        assert_eq!(
            crate::public::href(&mut list, link).as_deref(),
            Some(b"/public/projects/3f2a9c0176543210/landings/3f2a9c0176543210".as_slice())
        );
        let Some(temper_client_domain::public::PublicEvent::Open { page }) =
            crate::public::act(&mut list, link, skein_ui::Act::Press)
        else {
            panic!("public landing navigation");
        };
        assert_eq!(page, Public::Landing { project: PROJECT, task: PROJECT });
    }
    let page = render(Public::Landing { project: PROJECT, task: PROJECT }, &[(Doc::ForgeLanding, landing)], 0);
    let lines = paragraphs(&page);
    assert!(lines.contains(&"landed".into()));
    assert!(lines.contains(&"task 3f2a9c01 · Its change: 789ada1".into()));
    assert!(!lines.contains(&"0 stale proofs".into()));
    assert!(!lines.iter().any(|line| line.starts_with("Gate ")), "a queue merge has no gate row");
    for view in [&list, &page] {
        assert!(!text(view.tree().nodes()).contains("Holds the turn"));
    }
}
#[test]
fn a_public_landing_shows_the_search_and_blame_by_prefix() {
    let page = render(
        Public::Landing { project: PROJECT, task: TASK },
        &[(Doc::ForgeLanding, records::searching_landing())],
        0,
    );
    let lines = paragraphs(&page);
    for expected in [
        "probing at 1111111",
        "Search: green member 0 at 2222222",
        "Search: red member 3 at 3333333",
        "Left: blamed by gate 3, red at 3333333, green at 2222222",
    ] {
        assert!(lines.iter().any(|line| line == expected), "{expected}: {lines:?}");
    }
}

#[test]
fn a_commit_is_seven_plain_digits_and_no_padding() {
    let other = [0x45; 32];
    for (head, merge, expected) in [
        (Some(records::HEAD), None, Some("Its change: 789ada1")),
        (Some(other), Some(records::HEAD), Some("Its change: 789ada1")),
        (None, None, None),
    ] {
        let page = render(
            Public::Task { project: PROJECT, task: TASK },
            &[(Doc::JigCard, records::card()), (Doc::ForgeChange, records::change_commits(head, merge))],
            0,
        );
        let commits: Vec<_> = paragraphs(&page).into_iter().filter(|p| p.starts_with("Its change:")).collect();
        assert_eq!(commits, expected.into_iter().collect::<Vec<_>>());
        assert!(
            !page.tree().nodes().iter().any(|node| node.role == Role::Link && node.name.as_deref() == Some(b"789ada1"))
        );
    }
    for (candidate, landed, expected) in [
        (Some(records::HEAD), None, "task 9abcdef0 · Its change: 789ada1"),
        (Some(other), Some(records::HEAD), "task 9abcdef0 · Its change: 789ada1"),
        (None, None, "task 9abcdef0"),
    ] {
        let page = render(
            Public::Landing { project: PROJECT, task: TASK },
            &[(Doc::ForgeLanding, records::landing_commits(Some(other), candidate, landed))],
            0,
        );
        assert_eq!(paragraphs(&page)[2], expected);
        assert!(
            !page.tree().nodes().iter().any(|node| node.role == Role::Link && node.name.as_deref() == Some(b"789ada1"))
        );
    }
}
#[test]
fn no_public_page_links_to_a_forge() {
    for (page, documents) in [
        (Public::Projects, vec![(Doc::JigProjects, records::projects())]),
        (Public::Board { project: PROJECT }, vec![(Doc::JigBoard, board()), (Doc::ForgeLandings, records::landings())]),
        (
            Public::Task { project: PROJECT, task: TASK },
            vec![(Doc::JigCard, records::card()), (Doc::ForgeChange, records::change())],
        ),
        (Public::Landings { project: PROJECT }, vec![(Doc::ForgeLandings, records::landings())]),
        (Public::Landing { project: PROJECT, task: TASK }, vec![(Doc::ForgeLanding, records::landing())]),
    ] {
        let page = render(page, &documents, 0);
        let external: Vec<_> = page.tree().nodes().iter().filter(|node| node.external.is_some()).collect();
        assert_eq!(external.len(), 1);
        assert_eq!(external[0].name.as_deref(), Some(b"Sign in".as_slice()));
        assert_eq!(external[0].external.as_deref(), Some(b"https://temper.example/".as_slice()));
    }
}
#[test]
fn a_lines_words_never_touch() {
    let page = render(
        Public::Task { project: PROJECT, task: TASK },
        &[(Doc::JigCard, records::card()), (Doc::ForgeChange, records::change())],
        0,
    );
    let lines = paragraphs(&page);
    assert!(lines.contains(&"task 9abcdef0 A public goal title".into()));
    assert!(lines.contains(&"CI at its head · Passed".into()));
    assert!(lines.contains(&"Gate 3 · review · blocking · Approved".into()));
    let page = render(Public::Landings { project: PROJECT }, &[(Doc::ForgeLandings, records::landings())], 0);
    let lines = paragraphs(&page);
    assert!(lines.contains(&"task 9abcdef0 · Goal 3f2a9c01 · Ready just now".into()));
    assert!(lines.contains(&"landed · task 9abcdef0 · 789ada1".into()));
}

#[test]
fn public_row_titles_cut_locally_and_keep_only_the_engines_needed_cut_word() {
    for (title, cut) in super::fixtures::row_titles() {
        for engine_cut in [false, true] {
            let mut parts = records::line(false);
            parts.title = title.into();
            parts.title_cut = engine_cut;
            let line = parts;
            let page = card(line, 0);
            let expected = String::from_utf8(
                skein_lib::bytes::concat(&[
                    b"task 9abcdef0 ",
                    cut,
                    if engine_cut && title == cut { "…".as_bytes() } else { b"" },
                ])
                .into_vec(),
            )
            .unwrap();
            assert!(paragraphs(&page).contains(&expected), "{expected:?}: {:?}", paragraphs(&page));
        }
    }
}
