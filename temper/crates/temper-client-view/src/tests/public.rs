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
        b"an agent (AI work)",
        b"running",
        b"for under 1 min",
        b"12 spent \xc2\xb7 of 50",
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
        b"an agent (AI work)",
        b"running",
        b"for under 1 min",
        b"12 spent \xc2\xb7 of 50",
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
    let checked: &[&[u8]] = &[
        b"temper develops software with agents, directed by people. These public pages can be read without signing in.",
        b"Projects shared for anyone to read.",
        b"Goals and their tasks in this public project.",
        b"This task, its parent tasks and the work it delegates.",
        b"Changes waiting to land and recent landing attempts.",
        b"One landing attempt: its changes, commits and checks.",
        b"Task completion does not mean a forge change has landed. Forge change and landing statuses are shown separately.",
        b"Parent tasks, nearest first", b"Change task", b"For goal", b"Landing details for",
        b"Head commit", b"Merge commit", b"Base commit", b"Candidate commit", b"Landed commit",
        b"Stale proofs checked an older candidate and do not prove the current one.", b" \xc2\xb7 ",
    ];
    for node in view.tree().nodes() {
        for run in &node.text {
            assert!(
                (allowed.contains(&run.text.as_ref()) || checked.contains(&run.text.as_ref())),
                "unexpected public text {:?}",
                run.text
            );
        }
        if let Some(name) = &node.name {
            assert!(
                (allowed.contains(&name.as_ref()) || checked.contains(&name.as_ref())),
                "unexpected public name {name:?}"
            );
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
    text_count(view, b"A public goal title")
}

fn text_count(view: &PublicView, text: &[u8]) -> usize {
    let mut count = 0_usize;
    for node in view.tree().nodes() {
        for run in &node.text {
            if run.text.as_ref() == text {
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

#[test]
fn public_commit_links_use_the_object_id_width() {
    // Equal commit fields keep their roles on one link.
    const CHANGE_COMMITS: usize = 1;
    const LANDING_COMMITS: usize = 1;
    let sha1 = [
        0x78, 0x9a, 0xda, 0x51, 0x5c, 0x1a, 0xf9, 0xd4, 0x49, 0x67, 0xf2, 0x60, 0x49, 0xc4, 0x09, 0x0b, 0x3a, 0xb4,
        0x72, 0xdb, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ];
    let mut sha256_ending_in_zeroes = [0x34; 32];
    sha256_ending_in_zeroes.get_mut(30..).expect("last two bytes").fill(0);
    let cases: [([u8; 32], &[u8]); 3] = [
        (sha1, b"789ada515c1af9d44967f26049c4090b3ab472db"),
        ([0x12; 32], b"1212121212121212121212121212121212121212121212121212121212121212"),
        (sha256_ending_in_zeroes, b"3434343434343434343434343434343434343434343434343434343434340000"),
    ];
    for (hash, digits) in cases {
        commit_links(
            Public::Task { project: PROJECT, task: TASK },
            Doc::ForgeChange,
            records::change_with_commits(hash, Some(hash)),
            digits,
            CHANGE_COMMITS,
        );
        commit_links(
            Public::Landing { project: PROJECT, task: TASK },
            Doc::ForgeLanding,
            records::landing_with_commits(Some(hash), Some(hash), hash),
            digits,
            LANDING_COMMITS,
        );
    }
}

fn commit_links(page: Public, doc: Doc, body: Box<[u8]>, digits: &[u8], count: usize) {
    let mut reader = reader(page);
    answer(&mut reader, Doc::JigCard, Heard::Document { body: records::card(), fresh: Duration::from_secs(5) });
    answer(&mut reader, doc, Heard::Document { body, fresh: Duration::from_secs(5) });
    let mut view = view();
    build_public(&mut view, &reader, Wall::EPOCH, &mut Queue::uncapped());
    let mut linked = 0_usize;
    for node in view.tree().nodes() {
        if let Some(href) = &node.external
            && let Some(path) = href.strip_prefix(records::REPOSITORY)
            && let Some(hash) = path.strip_prefix(b"/commit/")
        {
            assert_eq!(hash, digits, "commit href uses its object id width");
            assert_eq!(node.name.as_deref(), digits.get(..8), "commit labels keep eight digits");
            linked = linked.checked_add(1).expect("bounded public commit links");
        }
    }
    assert_eq!(linked, count, "every commit field is linked");
}

#[test]
fn every_public_route_explains_its_purpose() {
    for (page, doc, body, purpose) in [
        (Public::Projects, Doc::JigProjects, records::projects(), b"Projects shared for anyone to read.".as_slice()),
        (Public::Board { project: PROJECT }, Doc::JigBoard, board(), b"Goals and their tasks in this public project."),
        (
            Public::Task { project: PROJECT, task: TASK },
            Doc::JigCard,
            records::card(),
            b"This task, its parent tasks and the work it delegates.",
        ),
        (
            Public::Landings { project: PROJECT },
            Doc::ForgeLandings,
            records::landings(),
            b"Changes waiting to land and recent landing attempts.",
        ),
        (
            Public::Landing { project: PROJECT, task: TASK },
            Doc::ForgeLanding,
            records::landing(),
            b"One landing attempt: its changes, commits and checks.",
        ),
    ] {
        let rendered = rendered(page, doc, body);
        assert!(has_text(&rendered, b"temper develops software with agents, directed by people. These public pages can be read without signing in."));
        assert!(has_text(&rendered, purpose), "page {page:?}");
    }
}

#[test]
fn empty_public_documents_explain_the_absence_of_work() {
    for (page, doc, body, expected) in [
        (
            Public::Projects,
            Doc::JigProjects,
            records::empty_projects(),
            b"No projects are shared publicly here.".as_slice(),
        ),
        (
            Public::Board { project: PROJECT },
            Doc::JigBoard,
            records::terminal_board(true),
            b"No public goals are shown in this project yet.",
        ),
        (
            Public::Landings { project: PROJECT },
            Doc::ForgeLandings,
            records::empty_landings(),
            b"No public changes are queued and no landing attempts are shown yet.",
        ),
        (
            Public::Task { project: PROJECT, task: TASK },
            Doc::JigCard,
            records::lineage_card(),
            b"No public delegated tasks are shown.",
        ),
        (
            Public::Landing { project: PROJECT, task: TASK },
            Doc::ForgeLanding,
            records::empty_landing(),
            b"No public changes are shown in this landing attempt.",
        ),
    ] {
        let rendered = rendered(page, doc, body);
        assert!(has_text(&rendered, expected), "page {page:?}");
    }
}

#[test]
fn terminal_goals_and_procedures_explain_completion_without_claiming_landing() {
    let board = rendered(Public::Board { project: PROJECT }, Doc::JigBoard, records::terminal_board(false));
    let mut phases = List::with_capacity(4); // The fixture's four ended goals.
    for node in board.tree().nodes() {
        for run in &node.text {
            let text = run.text.as_ref();
            if [b"failed".as_slice(), b"completed successfully", b"cancelled"].contains(&text) {
                phases.push(text).expect("four terminal phases");
            }
        }
    }
    assert_eq!(phases.as_slice(), [b"failed".as_slice(), b"failed", b"completed successfully", b"cancelled"]);
    for kind in [
        b"an agent (AI work)".as_slice(),
        b"a procedure (automated steps)",
        b"a person (human work)",
        b"a job (work on a host)",
    ] {
        assert!(has_text(&board, kind));
    }
    let card = rendered(Public::Task { project: PROJECT, task: TASK }, Doc::JigCard, records::lineage_card());
    assert!(has_text(
        &card,
        b"Task completion does not mean a forge change has landed. Forge change and landing statuses are shown separately."
    ));
    assert!(!has_text(&card, b"landed"));
}

#[test]
fn lineage_and_forge_links_are_grouped_and_commit_roles_share_one_link() {
    let mut task = reader(Public::Task { project: PROJECT, task: TASK });
    answer(&mut task, Doc::JigCard, Heard::Document { body: records::lineage_card(), fresh: Duration::from_secs(5) });
    answer(
        &mut task,
        Doc::ForgeChange,
        Heard::Document {
            body: records::change_with_commits(records::WALK_HEAD, Some(records::WALK_HEAD)),
            fresh: Duration::from_secs(5),
        },
    );
    let mut page = view();
    build_public(&mut page, &task, Wall::EPOCH, &mut Queue::uncapped());
    assert_link_group(&page, b"task 0893ed2e", Role::Item);
    assert_link_group(&page, b"task bd0d98ef", Role::Item);
    assert_link_group(&page, b"its change on the forge", Role::Paragraph);
    assert_link_group(&page, b"789ada51", Role::Paragraph);
    for label in [b"Head commit".as_slice(), b"Merge commit"] {
        assert!(has_text(&page, label));
    }
    assert_eq!(commit_count(&page), 1, "one link keeps both identical commit roles");
    landing_commit_roles();
}

fn landing_commit_roles() {
    let landing = rendered(
        Public::Landing { project: PROJECT, task: TASK },
        Doc::ForgeLanding,
        records::landing_with_commits(Some([0x12; 32]), Some([0x12; 32]), [0x12; 32]),
    );
    for label in [
        b"Base commit".as_slice(),
        b"Candidate commit",
        b"Landed commit",
        b"Stale proofs checked an older candidate and do not prove the current one.",
    ] {
        assert!(has_text(&landing, label), "label {label:?}");
    }
    assert_eq!(commit_count(&landing), 1);
    assert_eq!(text_count(&landing, b"Passed"), 1, "a proof run shows its status once");
    let distinct = rendered(
        Public::Landing { project: PROJECT, task: TASK },
        Doc::ForgeLanding,
        records::landing_with_commits(Some([0x34; 32]), Some([0x56; 32]), [0x12; 32]),
    );
    assert_eq!(commit_count(&distinct), 3, "distinct commit roles keep distinct links");
    let empty = rendered(Public::Landing { project: PROJECT, task: TASK }, Doc::ForgeLanding, records::empty_landing());
    assert!(has_text(&empty, b"0 stale proofs"));
    assert!(has_text(&empty, b"Stale proofs checked an older candidate and do not prove the current one."));
}

fn rendered(page: Public, doc: Doc, body: Box<[u8]>) -> PublicView {
    let mut reader = reader(page);
    answer(&mut reader, doc, Heard::Document { body, fresh: Duration::from_secs(5) });
    let mut view = view();
    build_public(&mut view, &reader, Wall::EPOCH, &mut Queue::uncapped());
    view
}

fn assert_link_group(view: &PublicView, label: &[u8], role: Role) {
    let nodes = view.tree().nodes();
    let target = fixtures::find(view.tree(), Role::Link, label).expect("link");
    let mut location = None;
    for (at, node) in nodes.iter().enumerate() {
        if node.id == target {
            location = Some(at);
        }
    }
    let index = location.expect("link position");
    let mut parent = None;
    for (at, node) in nodes.get(..index).expect("link prefix").iter().enumerate() {
        let end = at.checked_add(usize::try_from(node.size).expect("small subtree")).expect("bounded tree");
        if end > index {
            parent = Some(node.role);
        }
    }
    assert_eq!(parent, Some(role), "{label:?}");
}

fn commit_count(view: &PublicView) -> usize {
    let mut count = 0_usize;
    for node in view.tree().nodes() {
        if let Some(url) = &node.external
            && skein_lib::bytes::find(url, b"/commit/").is_some()
        {
            count = count.checked_add(1).expect("bounded public links");
        }
    }
    count
}
