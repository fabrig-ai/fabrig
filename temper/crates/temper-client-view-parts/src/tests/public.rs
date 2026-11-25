//! Anonymous view checks (temper/docs/design/client/web/public.md, 4).
use super::fixtures::{
    self,
    public_records::{self as records, PROJECT, TASK, answer, board, reader},
};
use crate::{
    public::{PublicView, build_public},
    sign_in::Ways,
};
use jig_client_protocol_wire::public_v1 as j;
use skein_lib::{Duration, List, Queue, Time, Wall};
use skein_ui::{Node, Role};
use temper_client_domain::{
    address::Public,
    public::{Doc, Heard, PublicEvent, Reader},
};
mod rules;

// The board and card fixtures each carry a goal/task and one delegate with the same title.
const TITLED_LINES: usize = 2;

fn view() -> PublicView {
    PublicView::new(
        &fixtures::tree_limits(),
        fixtures::browser_words(),
        Ways::parse("google", b"https://temper.example"),
    )
}

const FRESH: Duration = Duration::from_secs(5);
const DELEGATE: u64 = 0xfedc_ba98_7654_3210;

fn render(page: Public, documents: &[(Doc, Box<[u8]>)], seconds: u64) -> PublicView {
    let mut reader = reader(page);
    for (doc, body) in documents {
        answer(&mut reader, *doc, Heard::Document { body: body.clone(), fresh: FRESH });
    }
    draw(&reader, Wall::from_nanos(seconds.checked_mul(1_000_000_000).expect("test clock")))
}

fn draw(reader: &Reader, wall: Wall) -> PublicView {
    let mut view = view();
    build_public(&mut view, reader, wall, &mut Queue::uncapped());
    view
}

fn render_answer(page: Public, doc: Doc, heard: Heard) -> PublicView {
    let mut reader = reader(page);
    answer(&mut reader, doc, heard);
    draw(&reader, Wall::EPOCH)
}

#[expect(clippy::disallowed_methods, reason = "the test oracle collects visible runs in tree order")]
fn text_runs(nodes: &[Node]) -> Box<[&[u8]]> {
    nodes.iter().flat_map(|node| node.text.iter().map(|run| run.text.as_ref())).collect()
}

fn assert_link(view: &mut PublicView, name: &[u8], address: &[u8]) {
    let link = fixtures::find(view.tree(), Role::Link, name).expect("public link");
    assert_eq!(crate::public::href(view, link).as_deref(), Some(address), "link {name:?}");
}

#[test]
#[expect(clippy::disallowed_methods, reason = "the test inspects the main subtree by role")]
fn the_projects_page_lists_its_projects_under_its_heading() {
    const SECOND_PROJECT: u64 = 0xabcd_0123_7654_3210;
    const SECOND_NAME: &[u8] = b"Another public project";
    // One link per named project, in the record's order.
    const PROJECT_LINKS: usize = 2;
    let body = records::projects_named(&[(PROJECT, records::PROJECT_NAME), (SECOND_PROJECT, SECOND_NAME)]);
    let mut page = render(Public::Projects, &[(Doc::JigProjects, body)], 0);
    let nodes = page.tree().nodes();
    let (at, main) = nodes.iter().enumerate().find(|(_, node)| node.role == Role::Main).expect("public main");
    let size = usize::try_from(main.size).expect("main size");
    let main = &nodes[at..at.checked_add(size).expect("main end")];
    assert_eq!(text_runs(main).as_ref(), [b"Public projects".as_slice(), records::PROJECT_NAME, SECOND_NAME]);
    assert!(
        main.iter().any(|node| {
            node.role == Role::Heading { level: 1 } && node.name.as_deref() == Some(b"Public projects")
        })
    );
    assert_eq!(main.iter().filter(|node| node.role == Role::Link).count(), PROJECT_LINKS);
    assert_link(&mut page, records::PROJECT_NAME, b"/public/projects/3f2a9c0176543210");
    assert_link(&mut page, SECOND_NAME, b"/public/projects/abcd012376543210");
}

#[test]
#[expect(clippy::disallowed_methods, reason = "the test composes a page oracle from shared row runs")]
fn a_populated_board_and_its_goals_card_show_their_records_alone() {
    const DELEGATE_TITLE: &[u8] = b"Write the parser";
    const GOAL_LINK: &[u8] = b"Goal 9abcdef0";
    const DELEGATE_LINK: &[u8] = b"task fedcba98";
    const DELEGATE_HREF: &[u8] = b"/public/projects/3f2a9c0176543210/tasks/fedcba9876543210";
    const NOW: u64 = 900;
    const SEP: &[u8] = " · ".as_bytes();
    let [(long_title, cut_title), _] = fixtures::row_titles();
    let goal = j::PublicLine {
        title: long_title.into(),
        priority: Some(99),
        executor: j::PublicExecutor::Procedure,
        changed: 60,
        spent: Some(126),
        ..records::line(true)
    };
    let delegate = j::PublicLine {
        task: DELEGATE,
        title: DELEGATE_TITLE.into(),
        parent: Some(TASK),
        phase: j::PublicPhase::Waiting,
        changed: 720,
        ..records::line(false)
    };
    let board = render(
        Public::Board { project: PROJECT },
        &[(Doc::JigBoard, records::board_with(goal.clone(), delegate.clone()))],
        NOW,
    );
    let card = render(
        Public::Task { project: PROJECT, task: TASK },
        &[(Doc::JigCard, records::card_with(goal, List::with_capacity(0), delegate))],
        NOW,
    );
    let top: &[&[u8]] = &[b"temper", b"Sign in"];
    let intro = [
        records::PROJECT_NAME,
        b"What temper is doing on this project, as it happens. Read-only.",
        b"Work that ended before this project went public, or before temper last restarted, isn't shown.",
    ];
    let goal_title = [GOAL_LINK, b" ", cut_title];
    let goal_facts = [b"running", SEP, b"for 14 minutes", SEP, b"126 spent"];
    let delegate_row = [DELEGATE_LINK, b" ", DELEGATE_TITLE, b"waiting", SEP, b"for 3 minutes"];
    let board_words = [top, &intro, &goal_title, &goal_facts, &delegate_row, &[b"2 lines folded"]].concat();
    let card_words = [top, &[GOAL_LINK], &goal_title, &[b"a procedure", SEP], &goal_facts, &delegate_row].concat();
    // Exact runs count the executor (only on the card), while the same allowlists
    // exclude priority and any extra text or accessible name.
    for (kind, mut page, words) in [("board", board, board_words), ("card", card, card_words)] {
        assert_eq!(text_runs(page.tree().nodes()).as_ref(), words.as_slice(), "public {kind}");
        only_words(&page, &words);
        assert_link(&mut page, DELEGATE_LINK, DELEGATE_HREF);
    }
}

#[test]
fn unknown_and_empty_pages_read_their_whole_words() {
    let mut unknown =
        render_answer(Public::Task { project: PROJECT, task: TASK }, Doc::JigCard, Heard::Unknown { fresh: FRESH });
    assert_eq!(
        text_runs(unknown.tree().nodes()).as_ref(),
        [b"temper".as_slice(), b"Sign in", b"Not public, or not here.", b"Read public projects"]
    );
    assert!(fixtures::find(unknown.tree(), Role::Heading { level: 1 }, b"Not public, or not here.").is_some());
    assert_link(&mut unknown, b"Read public projects", b"/public");

    let empty = render(Public::Projects, &[(Doc::JigProjects, records::projects_with(false))], 0);
    assert_eq!(
        text_runs(empty.tree().nodes()).as_ref(),
        [b"temper".as_slice(), b"Sign in", b"Public projects", b"No public projects yet."]
    );
    assert!(fixtures::find(empty.tree(), Role::Heading { level: 1 }, b"Public projects").is_some());
}

#[test]
fn the_view_shows_only_what_the_records_carry() {
    let mut reader = reader(Public::Board { project: PROJECT });
    answer(&mut reader, Doc::JigBoard, Heard::Document { body: board(), fresh: Duration::from_secs(5) });
    let mut view = view();
    build_public(&mut view, &reader, Wall::EPOCH, &mut Queue::uncapped());
    // Every textual node is exactly a public record field or a checked phrase.
    // This catches any extra byte sequence, including every sequence of eight bytes.
    let allowed: &[&[u8]] = &[
        b"temper",
        b"Sign in",
        b"A public project",
        b"What temper is doing on this project, as it happens. Read-only.",
        b"Work that ended before this project went public, or before temper last restarted, isn't shown.",
        b" ",
        b" \xc2\xb7 ",
        b"Goal 9abcdef0",
        b"task 9abcdef0",
        b"A public goal title",
        b"an agent",
        b"running",
        b"for under 1 minute",
        b"12 spent",
        b"2 lines folded",
    ];
    only_words(&view, allowed);
    assert_eq!(title_count(&view), TITLED_LINES, "board shows its goal and delegate titles");
    let goal = fixtures::find(view.tree(), Role::Link, b"Goal 9abcdef0").expect("goal link");
    assert_eq!(
        crate::public::href(&mut view, goal).as_deref(),
        Some(b"/public/projects/3f2a9c0176543210/tasks/9abcdef076543210".as_slice())
    );
    let Some(PublicEvent::Open { page }) = crate::public::act(&mut view, goal, skein_ui::Act::Press) else {
        panic!("public task navigation")
    };
    assert_eq!(page, Public::Task { project: PROJECT, task: TASK });
    assert!(fixtures::find(view.tree(), Role::Heading { level: 1 }, b"A public project").is_some());
}
#[test]
fn unknown_and_unavailable_say_their_words() {
    for (heard, words) in [
        (Heard::Unknown { fresh: Duration::from_secs(5) }, b"Not public, or not here.".as_slice()),
        (Heard::Failed, b"Not available just now."),
    ] {
        let view = render_answer(Public::Projects, Doc::JigProjects, heard);
        assert!(fixtures::find(view.tree(), Role::Heading { level: 1 }, words).is_some());
    }
}

#[test]
fn the_task_and_forge_pages_use_public_links_and_mechanics() {
    let mut task = reader(Public::Task { project: PROJECT, task: TASK });
    answer(&mut task, Doc::JigCard, Heard::Document { body: records::card(), fresh: Duration::from_secs(5) });
    answer(&mut task, Doc::ForgeChange, Heard::Document { body: records::change(), fresh: Duration::from_secs(5) });
    let mut task_view = view();
    build_public(&mut task_view, &task, Wall::EPOCH, &mut Queue::uncapped());
    let allowed: &[&[u8]] = &[
        b"temper",
        b"Sign in",
        b"task 9abcdef0",
        b"Goal 9abcdef0",
        b"task 3f2a9c01",
        b"A public goal title",
        b"an agent",
        b"running",
        b"for under 1 minute",
        b"12 spent",
        b"checking",
        b"CI at its head",
        b"Passed",
        b"Gate 3 \xc2\xb7 review \xc2\xb7 blocking",
        b"Approved",
        b"Its change: 789ada1",
        b" ",
        b" \xc2\xb7 ",
    ];
    only_words(&task_view, allowed);
    assert_eq!(title_count(&task_view), TITLED_LINES, "card shows its task and delegate titles");
    for words in [b"Passed".as_slice(), b"Approved", b"Gate 3 \xc2\xb7 review \xc2\xb7 blocking"] {
        assert!(has_text(&task_view, words), "public change field {words:?}");
    }
}
fn only_words(view: &PublicView, allowed: &[&[u8]]) {
    for node in view.tree().nodes() {
        for run in &node.text {
            assert!(allowed.contains(&run.text.as_ref()), "unexpected public text {:?}", run.text);
        }
        if let Some(name) = &node.name {
            assert!(allowed.contains(&name.as_ref()), "unexpected public name {name:?}");
        }
    }
}
#[test]
fn the_landing_pages_show_the_turn_queues_commits_and_proofs() {
    let mut landings = reader(Public::Landings { project: PROJECT });
    answer(
        &mut landings,
        Doc::ForgeLandings,
        Heard::Document { body: records::landings(), fresh: Duration::from_secs(5) },
    );
    let mut list = view();
    build_public(&mut list, &landings, Wall::EPOCH, &mut Queue::uncapped());
    only_words(
        &list,
        &[
            b"temper",
            b"Sign in",
            b"Landings",
            b"Holds the turn",
            b"Queue",
            b"task 9abcdef0",
            b"Goal 3f2a9c01",
            b"Ready just now",
            b"landed",
            b"789ada1",
            b" \xc2\xb7 ",
        ],
    );
    let mut landing = reader(Public::Landing { project: PROJECT, task: TASK });
    answer(
        &mut landing,
        Doc::ForgeLanding,
        Heard::Document { body: records::landing(), fresh: Duration::from_secs(5) },
    );
    let mut page = view();
    build_public(&mut page, &landing, Wall::EPOCH, &mut Queue::uncapped());
    only_words(
        &page,
        &[
            b"temper",
            b"Sign in",
            b"Landings",
            b"landed",
            b"Holds the turn",
            b"task 9abcdef0",
            b"Its change: 789ada1",
            b"789ada1",
            b" \xc2\xb7 ",
            b"Gate 3 \xc2\xb7 check \xc2\xb7 advisory",
            b"Passed",
            b"2 stale proofs",
        ],
    );
    assert!(has_text(&list, b"landed"), "public landings list keeps its landed phase");
    assert!(has_text(&page, b"landed"), "public landing keeps its landed phase");
    assert!(has_text(&page, b"2 stale proofs"));
}
#[test]
fn projects_are_named_and_the_top_bar_offers_sign_in_from_the_pages_origin() {
    let mut reader = reader(Public::Projects);
    answer(&mut reader, Doc::JigProjects, Heard::Document { body: records::projects(), fresh: Duration::from_secs(5) });
    for (ways, sign_in) in [
        (Ways::NONE, None),
        (Ways::parse("", b"https://temper.example"), Some(b"https://temper.example/".as_slice())),
        (Ways::parse("unknown", b"https://temper.example"), Some(b"https://temper.example/".as_slice())),
        (Ways { google: true, origin: None }, None),
        (Ways::parse("google", b"https://temper.example"), Some(b"https://temper.example/".as_slice())),
    ] {
        let mut view = PublicView::new(&fixtures::tree_limits(), fixtures::browser_words(), ways);
        build_public(&mut view, &reader, Wall::EPOCH, &mut Queue::uncapped());
        let mut nodes = view.tree().nodes().iter();
        assert_eq!(nodes.next().expect("page").role, Role::Page);
        let banner = nodes.next().expect("banner");
        assert_eq!(banner.role, Role::Banner);
        assert_eq!(banner.styles.as_slice(), &[crate::look::TOP]);
        let brand = nodes.next().expect("brand link");
        assert_eq!(brand.role, Role::Link);
        assert_eq!(brand.name.as_deref(), Some(b"temper".as_slice()));
        assert_eq!(brand.styles.as_slice(), &[crate::look::BRAND]);
        let brand_id = brand.id;
        let text = nodes.next().expect("brand words");
        assert_eq!(text.role, Role::Text);
        assert_eq!(text.text.get(0).expect("brand run").text.as_ref(), b"temper");
        if let Some(address) = sign_in {
            let slot = nodes.next().expect("sign-in slot");
            assert_eq!(slot.role, Role::Slot);
            assert_eq!(slot.styles.as_slice(), &[crate::look::TOP_END]);
            let link = nodes.next().expect("sign-in link");
            assert_eq!(link.role, Role::Link);
            assert_eq!(link.name.as_deref(), Some(b"Sign in".as_slice()));
            assert_eq!(link.external.as_deref(), Some(address));
            let text = nodes.next().expect("sign-in words");
            assert_eq!(text.role, Role::Text);
            assert_eq!(text.text.get(0).expect("sign-in run").text.as_ref(), b"Sign in");
            assert_eq!(slot.size, 1_u32.checked_add(link.size).expect("slot holds its link"));
            assert_eq!(
                banner.size,
                1_u32
                    .checked_add(brand.size)
                    .expect("banner holds brand")
                    .checked_add(slot.size)
                    .expect("banner holds slot")
            );
            only_words(&view, &[b"temper", b"Sign in", b"Public projects", b"A public project"]);
        } else {
            assert_eq!(banner.size, 1_u32.checked_add(brand.size).expect("banner holds only its brand"));
            assert!(fixtures::find(view.tree(), Role::Link, b"Sign in").is_none());
            only_words(&view, &[b"temper", b"Public projects", b"A public project"]);
        }
        let main = nodes.next().expect("main follows banner");
        assert_eq!(main.role, Role::Main);
        assert_eq!(main.styles.as_slice(), &[crate::look::MAIN, crate::look::PAGE, crate::look::PUBLIC_PAGE]);
        assert_eq!(crate::public::href(&mut view, brand_id).as_deref(), Some(b"/public".as_slice()));
        let project = fixtures::find(view.tree(), Role::Link, b"A public project").expect("project name");
        assert_eq!(
            crate::public::href(&mut view, project).as_deref(),
            Some(b"/public/projects/3f2a9c0176543210".as_slice())
        );
    }
}

#[test]
fn a_retained_snapshot_has_its_read_time_and_disappears_when_due() {
    let mut reader = reader(Public::Board { project: PROJECT });
    answer(&mut reader, Doc::JigBoard, Heard::Document { body: board(), fresh: Duration::from_secs(5) });
    reader.step(
        Time::ZERO.saturating_add(Duration::from_secs(3)),
        PublicEvent::Heard { doc: Doc::JigBoard, heard: Heard::Busy { retry: Duration::from_secs(3) } },
        &mut Queue::uncapped(),
    );
    let mut view = view();
    build_public(&mut view, &reader, Wall::from_nanos(3_000_000_000), &mut Queue::uncapped());
    assert!(has_text(&view, b"As of 00:00"));
    reader.step(Time::ZERO.saturating_add(Duration::from_secs(5)), PublicEvent::Due, &mut Queue::uncapped());
    build_public(&mut view, &reader, Wall::from_nanos(5_000_000_000), &mut Queue::uncapped());
    assert!(fixtures::find(view.tree(), Role::Heading { level: 1 }, b"Not available just now.").is_some());
    assert!(fixtures::find(view.tree(), Role::Heading { level: 1 }, b"A public project").is_none());
}
#[test]
fn an_unavailable_companion_keeps_the_fresh_board() {
    let mut reader = reader(Public::Board { project: PROJECT });
    answer(&mut reader, Doc::JigBoard, Heard::Document { body: board(), fresh: Duration::from_secs(5) });
    answer(&mut reader, Doc::ForgeLandings, Heard::Failed);
    let mut view = view();
    build_public(&mut view, &reader, Wall::EPOCH, &mut Queue::uncapped());
    assert!(fixtures::find(view.tree(), Role::Heading { level: 1 }, b"A public project").is_some());
    assert!(has_text(&view, b"Not available just now."));
}

fn title_count(view: &PublicView) -> usize {
    let mut count = 0_usize;
    for node in view.tree().nodes() {
        for run in &node.text {
            if run.text.as_ref() == b"A public goal title" {
                count = count.checked_add(1).expect("bounded public text");
            }
        }
    }
    count
}

fn has_text(view: &PublicView, text: &[u8]) -> bool {
    for node in view.tree().nodes() {
        for run in &node.text {
            if run.text.as_ref() == text {
                return true;
            }
        }
    }
    false
}

#[test]
fn a_retained_companion_has_its_own_read_time() {
    let mut reader = reader(Public::Board { project: PROJECT });
    answer(&mut reader, Doc::JigBoard, Heard::Document { body: board(), fresh: Duration::from_secs(10) });
    answer(
        &mut reader,
        Doc::ForgeLandings,
        Heard::Document { body: records::landings(), fresh: Duration::from_secs(5) },
    );
    reader.step(
        Time::ZERO.saturating_add(Duration::from_secs(3)),
        PublicEvent::Heard { doc: Doc::ForgeLandings, heard: Heard::Busy { retry: Duration::from_secs(3) } },
        &mut Queue::uncapped(),
    );
    let mut view = view();
    build_public(&mut view, &reader, Wall::from_nanos(3_000_000_000), &mut Queue::uncapped());
    assert!(has_text(&view, b"As of 00:00"));
    reader.step(Time::ZERO.saturating_add(Duration::from_secs(5)), PublicEvent::Due, &mut Queue::uncapped());
    build_public(&mut view, &reader, Wall::from_nanos(5_000_000_000), &mut Queue::uncapped());
    assert!(fixtures::find(view.tree(), Role::Heading { level: 1 }, b"A public project").is_some());
    assert!(has_text(&view, b"Not available just now."));
    assert!(!has_text(&view, b"As of 00:00"));
}
