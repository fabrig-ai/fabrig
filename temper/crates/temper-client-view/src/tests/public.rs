//! Anonymous view checks (temper/docs/design/client/browser.md, 8.4).
use super::fixtures;
use crate::{
    public::{PublicView, build_public},
    sign_in::Ways,
};
use jig_client_protocol_wire::public_v1 as j;
use skein_lib::{Duration, List, Queue, Time, Wall, Writer};
use skein_ui::Role;
use temper_client_domain::{
    address::Public,
    public::{Doc, Heard, PublicEvent, PublicLimits, Reader},
};
mod records;
const PROJECT: u64 = 0x3f2a_9c01_7654_3210;
const TASK: u64 = 0x9abc_def0_7654_3210;
// The board and card fixtures each carry a goal/task and one delegate with the same title.
const TITLED_LINES: usize = 2;
fn reader(page: Public) -> Reader {
    let limits = PublicLimits { jig: j::CEILINGS, forge: temper_client_protocol_wire::public_v1::CEILINGS };
    let mut reader = Reader::new(&limits);
    reader.step(Time::ZERO, PublicEvent::Open { page }, &mut Queue::uncapped());
    reader
}
fn view() -> PublicView {
    PublicView::new(
        &fixtures::tree_limits(),
        fixtures::browser_words(),
        Ways::parse("google", b"https://temper.example"),
    )
}
fn answer(reader: &mut Reader, doc: Doc, heard: Heard) {
    reader.step(Time::ZERO, PublicEvent::Heard { doc, heard }, &mut Queue::uncapped());
}
fn board() -> Box<[u8]> {
    let line = records::line(true);
    let goal = j::PublicGoal::new(
        &j::CEILINGS,
        j::PublicGoalParts { line, tree: List::from_boxed(Box::new([records::line(false)])), folded: 2 },
    )
    .expect("goal");
    let board = j::PublicBoard::new(
        &j::CEILINGS,
        j::PublicBoardParts {
            project: PROJECT,
            name: b"A public project".as_slice().into(),
            live: List::from_boxed(Box::new([goal])),
            live_folded: 0,
            ended: List::with_capacity(0),
        },
    )
    .expect("board");
    let mut writer = Writer::new(usize::try_from(board.measure()).expect("small board"));
    board.encode(&mut writer).expect("board");
    writer.finish()
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
        b"Landings",
        b"Goal 9abcdef0",
        b"task 9abcdef0",
        b"A public goal title",
        b"an agent",
        b"running",
        b"for under 1 min",
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
        let mut reader = reader(Public::Projects);
        answer(&mut reader, Doc::JigProjects, heard);
        let mut view = view();
        build_public(&mut view, &reader, Wall::EPOCH, &mut Queue::uncapped());
        assert!(fixtures::find(view.tree(), Role::Heading { level: 1 }, words).is_some());
    }
}
#[test]
fn the_signed_out_page_links_to_public() {
    let root = fixtures::signed_out();
    let view =
        fixtures::sign_in_view(&root, crate::words::Shell::Browser, Ways::parse("google", b"https://temper.example"));
    let link = fixtures::find(view.tree(), Role::Link, b"Read public projects").expect("public reading link");
    assert_eq!(
        view.tree().find(link).expect("link node").external.as_deref(),
        Some(b"https://temper.example/public".as_slice())
    );
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
        b"for under 1 min",
        b"12 spent",
        b"checking",
        b"CI at its head",
        b"Passed",
        b"Gate 3 \xc2\xb7 review \xc2\xb7 blocking",
        b"Approved",
        b"its change on the forge",
        b"12121212",
    ];
    only_words(&task_view, allowed);
    assert_eq!(title_count(&task_view), TITLED_LINES, "card shows its task and delegate titles");
    for words in [b"Passed".as_slice(), b"Approved", b"Gate 3 \xc2\xb7 review \xc2\xb7 blocking"] {
        assert!(has_text(&task_view, words), "public change field {words:?}");
    }
    let pull = fixtures::find(task_view.tree(), Role::Link, b"its change on the forge").expect("public pull link");
    assert_eq!(crate::public::href(&mut task_view, pull).as_deref(), Some(records::PULL));
    let hash = fixtures::find(task_view.tree(), Role::Link, b"12121212").expect("head hash");
    assert_eq!(crate::public::href(&mut task_view, hash).as_deref(), Some(b"https://forge.example/owner/repository/commit/1212121212121212121212121212121212121212121212121212121212121212".as_slice()));
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
            b"12121212",
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
            b"its change on the forge",
            b"12121212",
            b"Gate 3 \xc2\xb7 check \xc2\xb7 advisory",
            b"Passed",
            b"2 stale proofs",
        ],
    );
    assert!(has_text(&list, b"landed"), "public landings list keeps its landed phase");
    assert!(has_text(&page, b"landed"), "public landing keeps its landed phase");
    let proof = fixtures::find(page.tree(), Role::Link, b"Passed").expect("proof run");
    assert_eq!(crate::public::href(&mut page, proof).as_deref(), Some(records::RUN));
}
#[test]
fn projects_are_named_and_the_banner_only_offers_a_known_sign_in() {
    let mut reader = reader(Public::Projects);
    answer(&mut reader, Doc::JigProjects, Heard::Document { body: records::projects(), fresh: Duration::from_secs(5) });
    let mut view = PublicView::new(&fixtures::tree_limits(), fixtures::browser_words(), Ways::NONE);
    build_public(&mut view, &reader, Wall::EPOCH, &mut Queue::uncapped());
    only_words(&view, &[b"temper", b"Public projects", b"A public project"]);
    let project = fixtures::find(view.tree(), Role::Link, b"A public project").expect("project name");
    assert_eq!(
        crate::public::href(&mut view, project).as_deref(),
        Some(b"/public/projects/3f2a9c0176543210".as_slice())
    );
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
