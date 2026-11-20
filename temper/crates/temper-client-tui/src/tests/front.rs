//! The front's views (temper/docs/design/client/tui/box.md).
use super::{fixtures as f, goldens, launchpad};
use crate::{BoxLine, Scheme, Told, Tui, look, view};
use jig_client::contract as c;
use skein_lib::{List, Queue};
use skein_ui::Role;

// The request's review size and section 6.17's retained front limits.
const FULL: skein_ui_screen::Size = skein_ui_screen::Size { columns: 80, rows: 24 };
const BOX_LINES: u32 = 50;
const LOG_BYTES: usize = 512;
const STEP_BYTES: usize = 64;
// Twenty short rows overflow the body at FULL with its header and composer.
const OVERFLOW_LINES: u32 = 20;

pub(super) fn working(task: u64, title: &[u8], activity: u64) -> c::TaskLine {
    let mut row = launchpad::line(task, title, activity, false);
    row.chip.details.phase = c::ChipPhase::Active(c::RunStage::Running { attempt: 1 });
    row.chip.details.details.observation = None;
    row.waits_on = c::WaitsOn::Nothing;
    row.chip.details.details.live_delegates = 0;
    row
}

#[test]
fn web_is_offered_only_on_a_running_fronts_pages() {
    use crate::{BoxEnd, BoxStage, Own};
    let home = launchpad::home_read(Some(true), &[], true, None);
    let welcome = f::live(Some(f::me(&[]))).0;
    let chat = launchpad::question_root();
    let talk = super::chat::talk_root("open");
    let signed_out = f::signed_out();
    let connecting = f::live(None).0;
    for (name, root, command, expected) in [
        ("launchpad", &home, None, true),
        ("welcome", &welcome, None, true),
        ("chat", &chat, None, true),
        ("talk", &talk, None, true),
        ("place over home", &home, Some(Own::Change), true),
        ("place over chat", &chat, Some(Own::Tell), true),
        ("place over talk", &talk, Some(Own::Tell), true),
        ("sign in", &signed_out, None, false),
    ] {
        let mut tui = front();
        tui.step(Told::Place { command });
        let commands = crate::offered_own(root, &tui);
        assert_eq!(commands.contains(&Own::Web), expected, "{name}");
        for (at, command) in commands.iter().enumerate() {
            if *command == Own::Open {
                assert_eq!(commands.get(at + 1), Some(&Own::Web), "{name}: after Open");
            }
        }
    }
    assert_eq!(crate::offered_own(&welcome, &front()).first(), Some(&Own::Web));
    let mut starting = front();
    starting.step(Told::Front { set_up: true, box_log: Box::default() });
    assert!(!crate::offered_own(&connecting, &starting).contains(&Own::Web), "starting screen");
    let mut ordinary = Tui::new(f::ADDRESS);
    ordinary.step(Told::Stage { stage: BoxStage::Running });
    assert!(!crate::offered_own(&home, &ordinary).contains(&Own::Web), "out of front");
    for stage in [
        BoxStage::Finding,
        BoxStage::Fetching { done: 1, total: None },
        BoxStage::Starting,
        BoxStage::Stopping,
        BoxStage::Ended { end: BoxEnd::Exited { code: 0 } },
        BoxStage::Failed { why: b"failed".as_slice().into() },
    ] {
        let mut tui = front();
        tui.step(Told::Stage { stage: stage.clone() });
        assert!(!crate::offered_own(&home, &tui).contains(&Own::Web), "{stage:?}");
    }
    let mut gave_up = front();
    gave_up.step(Told::BoxLine { line: BoxLine::GaveUp { step: b"serve".as_slice().into() } });
    assert!(!crate::offered_own(&home, &gave_up).contains(&Own::Web));
}

#[test]
#[expect(clippy::disallowed_macros, reason = "the golden table embeds hand-written frames")]
fn the_front_views_are_their_goldens() {
    let rows = [
        (
            "front-playground-80x24-dark",
            true,
            false,
            include_bytes!("golden/front-playground-80x24-dark.txt").as_slice(),
        ),
        ("front-quit-80x24-dark", false, true, include_bytes!("golden/front-quit-80x24-dark.txt").as_slice()),
        ("front-log-80x24-dark", false, false, include_bytes!("golden/front-log-80x24-dark.txt").as_slice()),
    ];
    let mut views = List::with_capacity(3);
    for (_, playground, quitting, _) in &rows {
        let chats = if *playground {
            List::with_capacity(0)
        } else {
            List::from_boxed(Box::new([working(9, b"csv-export", 1)]))
        };
        let root = launchpad::home_read(
            Some(true),
            chats.as_slice(),
            false,
            if *playground { Some((Some(1), temper_client_protocol_wire::v1::PlaygroundState::Running)) } else { None },
        );
        let mut tui = front();
        if *quitting {
            tui.step(Told::Quitting);
        }
        if !*playground && !*quitting {
            tui.step(Told::BoxLine { line: BoxLine::Restarting { step: b"forgejo-3".as_slice().into() } });
            for line in [
                b"10:41:02 forgejo exited 1: database is locked".as_slice(),
                b"10:41:03 forgejo restarting, 2 of 5 within 10 min",
                b"10:41:05 forgejo listening on 127.0.0.1:8368",
            ] {
                tui.step(Told::BoxLine { line: BoxLine::Log { line: line.into() } });
            }
            tui.step(Told::Log);
        }
        let words = f::words();
        let mut part = view::launchpad::read(&root, &tui, &words, f::NOW);
        if *playground {
            part.project = Some(b"Demo".as_slice().into());
        } else {
            part.lines = List::from_boxed(Box::new([
                view::launchpad::Line {
                    role: Role::Paragraph,
                    words: b"csv-export is two of three tasks in; its last task is running.".as_slice().into(),
                    style: None,
                },
                view::launchpad::Line {
                    role: Role::Paragraph,
                    words: b"written by the status agent 10:12".as_slice().into(),
                    style: Some(look::BYLINE),
                },
            ]));
        }
        let mut foot = view::foot::read(&root, &tui, &words, f::NOW);
        foot.project.clone_from(&part.project);
        foot.closed =
            Some(if *playground { b"Scripted model".as_slice() } else { "Claude Opus 5.5 · high".as_bytes() }.into());
        let mut view = f::view();
        view::signed(&mut view, &view::Signed { body: view::Body::Launchpad(part), foot }, &mut Queue::uncapped());
        views.push(view).unwrap();
    }
    let mut checks = List::with_capacity(3);
    for ((name, _, _, want), view) in rows.iter().zip(&views) {
        checks
            .push(goldens::Golden {
                name,
                tree: view.tree(),
                scheme: Scheme::Dark,
                background: Some(crate::theme::palette(Scheme::Dark)[0].1),
                size: FULL,
                focus: skein_ui_screen::Focus::default(),
                want,
            })
            .unwrap();
    }
    goldens::check(checks.as_slice());
    log_follows_end();
}

#[test]
#[expect(clippy::disallowed_macros, reason = "the lending golden table embeds its fixed frames")]
fn the_welcome_back_notices_and_model_details_are_their_goldens() {
    use crate::Notice;
    let rows = [
        (
            "front-login-signed-in-80x24-dark",
            Some(Notice::LoginSignedIn),
            None,
            include_bytes!("golden/front-login-signed-in-80x24-dark.txt").as_slice(),
        ),
        (
            "front-login-cancelled-80x24-dark",
            Some(Notice::LoginCancelled),
            None,
            include_bytes!("golden/front-login-cancelled-80x24-dark.txt").as_slice(),
        ),
        (
            "front-login-not-finished-80x24-dark",
            Some(Notice::LoginNotFinished),
            None,
            include_bytes!("golden/front-login-not-finished-80x24-dark.txt").as_slice(),
        ),
        (
            "front-model-program-80x24-dark",
            None,
            Some(c::AccountKind::Program),
            include_bytes!("golden/front-model-program-80x24-dark.txt").as_slice(),
        ),
        (
            "front-model-plan-80x24-dark",
            None,
            Some(c::AccountKind::SignIn),
            include_bytes!("golden/front-model-plan-80x24-dark.txt").as_slice(),
        ),
    ];
    let mut views = List::with_capacity(5);
    for (_, notice, kind, _) in &rows {
        let root = super::welcome::lending_root(*kind);
        let mut tui = front();
        tui.step(Told::Notice { notice: notice.clone() });
        let mut view = f::view();
        f::build(&mut view, &tui, &root);
        super::limits::fits(view.tree(), &crate::limits(&f::limits(), &skein_ui::Limits::EMPTY, 512).unwrap());
        views.push(view).unwrap();
    }
    let mut checks = List::with_capacity(5);
    for ((name, _, _, want), view) in rows.iter().zip(&views) {
        checks
            .push(goldens::Golden {
                name,
                tree: view.tree(),
                scheme: Scheme::Dark,
                background: Some(crate::theme::palette(Scheme::Dark)[0].1),
                size: FULL,
                focus: skein_ui_screen::Focus::default(),
                want,
            })
            .unwrap();
    }
    goldens::check(checks.as_slice());
}

#[test]
fn the_front_tree_fits_its_limits() {
    let root = launchpad::home(None, &[working(9, b"csv-export", 1)]);
    let mut tui = front();
    tui.step(Told::Log);
    tui.step(Told::Quitting);
    tui.step(Told::BoxLine { line: BoxLine::GaveUp { step: Box::new([b's'; STEP_BYTES]) } });
    for _ in 0..BOX_LINES {
        tui.step(Told::BoxLine { line: BoxLine::Log { line: Box::new([b'x'; LOG_BYTES]) } });
    }
    let mut view = f::view();
    f::build(&mut view, &tui, &root);
    assert_eq!(f::count(view.tree(), Role::Log), 1);
    f::find(view.tree(), Role::MenuItem, b"1. Stop the box");
    let notes = view::foot::read_notes(&root, &tui, &f::words());
    let log = notes.log.as_ref().unwrap();
    assert_eq!(log.lines.len(), usize::try_from(BOX_LINES).unwrap());
    for line in &log.lines {
        assert_eq!(line.len(), LOG_BYTES);
    }
    assert!(notes.gave_up.is_some());
    super::limits::fits(view.tree(), &crate::limits(&f::limits(), &skein_ui::Limits::EMPTY, 512).unwrap());
}

fn log_follows_end() {
    // Twenty short lines overflow the body's available rows at the review size.
    let root = launchpad::home(None, &[working(9, b"csv-export", 1)]);
    let mut tui = front();
    tui.step(Told::Log);
    for index in 0..OVERFLOW_LINES {
        let line = match index {
            0 => b"oldest".as_slice(),
            19 => b"newest",
            1..19 => b"middle",
            20.. => unreachable!(),
        };
        tui.step(Told::BoxLine { line: BoxLine::Log { line: line.into() } });
    }
    let mut view = f::view();
    f::build(&mut view, &tui, &root);
    let size = FULL;
    let mut frame = skein_ui_screen::Frame::new(size);
    skein_ui_screen::render(
        view.tree(),
        &crate::theme(Scheme::Dark, None),
        size,
        &mut skein_ui_screen::Focus::default(),
        &mut frame,
    );
    let printed = frame.print();
    assert!(skein_lib::bytes::find(&printed, b"newest").is_some(), "the log follows its newest line");
    assert!(skein_lib::bytes::find(&printed, b"oldest").is_none(), "older rows can be read back");
}

#[test]
#[expect(clippy::disallowed_macros, reason = "the web note golden embeds its generated frame")]
fn the_web_note_wraps_the_latest_link_at_its_own_column() {
    let root = launchpad::home_read(None, &[], false, None);
    let mut tui = front();
    tui.step(Told::BoxLine {
        line: BoxLine::Link {
            url: skein_lib::Secret::new(b"http://127.0.0.1:8367/sign-in?secret=old".as_slice().into()),
        },
    });
    tui.step(Told::LinkNotOpened);
    let link = b"http://127.0.0.1:8367/sign-in?secret=AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".as_slice();
    tui.step(Told::BoxLine { line: BoxLine::Link { url: skein_lib::Secret::new(link.into()) } });
    let mut view = f::view();
    f::build(&mut view, &tui, &root);
    let head = f::find(view.tree(), Role::Paragraph, b"Open this link in a browser that reaches temper:");
    assert!(head.styles.as_slice().contains(&look::NOTE));
    f::find(view.tree(), Role::Paragraph, link);
    for hint in [
        b"It signs you in as this temper's owner, once, within 2 hours.".as_slice(),
        b"Opened after that, temper makes a fresh one: /web again opens the fresh one.",
        b"From another machine, tunnel port 8367 to this one first.",
    ] {
        assert!(f::find(view.tree(), Role::Paragraph, hint).styles.as_slice().contains(&look::HINT));
    }
    super::limits::fits(view.tree(), &crate::limits(&f::limits(), &skein_ui::Limits::EMPTY, 512).unwrap());
    goldens::check(&[goldens::Golden {
        name: "front-web-note-80x24-dark",
        tree: view.tree(),
        scheme: Scheme::Dark,
        background: Some(crate::theme::palette(Scheme::Dark)[0].1),
        size: FULL,
        focus: skein_ui_screen::Focus::default(),
        want: include_bytes!("golden/front-web-note-80x24-dark.txt"),
    }]);
    let root = launchpad::home_read(Some(true), &[], false, None);
    tui.step(Told::Brief);
    tui.step(Told::BoxLine {
        line: BoxLine::Link {
            url: skein_lib::Secret::new(b"http://127.0.0.1:9367/sign-in?secret=fresh".as_slice().into()),
        },
    });
    f::build(&mut view, &tui, &root);
    f::find(view.tree(), Role::Paragraph, b"From another machine, tunnel port 9367 to this one first.");
    let part = view::launchpad::read(&root, &tui, &f::words(), f::NOW);
    assert!(!part.brief.is_empty(), "fixture has the brief's note");
    for line in &part.brief {
        let mut before_web = false;
        for node in view.tree().nodes() {
            if node.role == Role::Paragraph {
                let text = f::text(view.tree(), node);
                if text.as_ref() == b"Open this link in a browser that reaches temper:" {
                    assert!(before_web, "the brief's note precedes the web note");
                    break;
                }
                before_web |= text == line.words;
            }
        }
    }
}

fn front() -> Tui {
    let mut tui = Tui::new(f::ADDRESS);
    tui.step(Told::Front { set_up: false, box_log: Box::default() });
    tui.step(Told::Stage { stage: crate::BoxStage::Running });
    tui.step(Told::Size { columns: FULL.columns });
    tui
}

#[test]
#[expect(clippy::disallowed_macros, reason = "the stop and end golden table embeds its frames")]
fn the_stopping_and_ended_box_views_are_their_goldens() {
    use crate::{BoxEnd, BoxStage};
    let rows = [
        (
            "front-stopping-80x24-dark",
            BoxStage::Stopping,
            true,
            b"stopping".as_slice(),
            look::SIGNAL,
            include_bytes!("golden/front-stopping-80x24-dark.txt").as_slice(),
        ),
        (
            "front-ended-80x24-dark",
            BoxStage::Ended { end: BoxEnd::Exited { code: 1 } },
            false,
            b"box stopped".as_slice(),
            look::DOWN,
            include_bytes!("golden/front-ended-80x24-dark.txt").as_slice(),
        ),
        (
            "front-stopped-asked-80x24-dark",
            BoxStage::Ended { end: BoxEnd::Exited { code: 0 } },
            true,
            b"box stopped".as_slice(),
            look::DOWN,
            include_bytes!("golden/front-stopped-asked-80x24-dark.txt").as_slice(),
        ),
    ];
    let root = launchpad::home(None, &[working(9, b"csv-export", 1)]);
    let mut views = List::with_capacity(u32::try_from(rows.len()).unwrap());
    for (_, stage, asked, quiet, style, _) in &rows {
        let mut tui = front();
        if *asked {
            tui.step(Told::Stage { stage: BoxStage::Stopping });
        }
        tui.step(Told::Stage { stage: stage.clone() });
        for line in [b"10:52:01 Stopping".as_slice(), b"10:52:02 serve exited 0"] {
            tui.step(Told::BoxLine { line: BoxLine::Log { line: line.into() } });
        }
        assert_eq!(view::front::box_state(&tui, &f::words()), Some(((*quiet).into(), *style)));
        let mut view = f::view();
        f::build(&mut view, &tui, &root);
        if *asked {
            assert!(
                f::find(view.tree(), Role::Paragraph, "Stopping the box…".as_bytes())
                    .styles
                    .as_slice()
                    .contains(&look::HEAD)
            );
            assert!(
                f::find(view.tree(), Role::Paragraph, b"Ctrl+C again ends its steps at once.")
                    .styles
                    .as_slice()
                    .contains(&look::HINT)
            );
            assert!(
                f::find(view.tree(), Role::Paragraph, b"10:52:02 serve exited 0")
                    .styles
                    .as_slice()
                    .contains(&look::NOTE)
            );
        } else {
            assert!(
                f::find(view.tree(), Role::Paragraph, b"The box stopped: exited 1. /log shows why; Ctrl+C closes.")
                    .styles
                    .as_slice()
                    .contains(&look::REFUSED)
            );
        }
        super::limits::fits(view.tree(), &crate::limits(&f::limits(), &skein_ui::Limits::EMPTY, 512).unwrap());
        views.push(view).unwrap();
    }
    let mut checks = List::with_capacity(u32::try_from(rows.len()).unwrap());
    for ((name, _, _, _, _, want), view) in rows.iter().zip(&views) {
        checks
            .push(goldens::Golden {
                name,
                tree: view.tree(),
                scheme: Scheme::Dark,
                background: Some(crate::theme::palette(Scheme::Dark)[0].1),
                size: FULL,
                focus: skein_ui_screen::Focus::default(),
                want,
            })
            .unwrap();
    }
    goldens::check(checks.as_slice());
    let mut tui = front();
    tui.step(Told::Stage { stage: BoxStage::Ended { end: BoxEnd::Killed { signal: 15 } } });
    assert_eq!(
        view::front::gave_up_note(&tui, &f::words()).as_deref(),
        Some(b"The box stopped: killed by signal 15. /log shows why; Ctrl+C closes.".as_slice())
    );
    stopping_log_bounds();
}

fn stopping_log_bounds() {
    use crate::{BoxEnd, BoxStage};
    let root = launchpad::home(None, &[working(9, b"csv-export", 1)]);
    let mut tui = front();
    tui.step(Told::Stage { stage: BoxStage::Stopping });
    tui.step(Told::Log);
    for _ in 0..BOX_LINES {
        tui.step(Told::BoxLine { line: BoxLine::Log { line: Box::new([b'x'; LOG_BYTES]) } });
    }
    for stage in [BoxStage::Stopping, BoxStage::Ended { end: BoxEnd::Killed { signal: 15 } }] {
        tui.step(Told::Stage { stage });
        let mut view = f::view();
        f::build(&mut view, &tui, &root);
        assert_eq!(f::count(view.tree(), Role::Log), 2, "/log keeps its own view while stopping");
        let notes = view::foot::read_notes(&root, &tui, &f::words());
        assert_eq!(notes.log.as_ref().unwrap().lines.len(), usize::try_from(BOX_LINES).unwrap());
        assert_eq!(notes.stopping.as_ref().unwrap().log.lines.len(), usize::try_from(BOX_LINES).unwrap());
        super::limits::fits(view.tree(), &crate::limits(&f::limits(), &skein_ui::Limits::EMPTY, 512).unwrap());
    }
}

#[test]
fn closed_notices_and_quit_questions_cut_only_the_chat_title() {
    for (title, cut) in [
        (
            b"abcd abcd abcd abcd abcd abcd abcd abcd abcd abcd abcd abcd abcd abcd abcd abcd abcd abcd abcd abcd"
                .as_slice(),
            "abcd abcd abcd abcd abcd abcd abcd abcd abcd abcd abcd abcd abcd abcd abcd abcd…".as_bytes(),
        ),
        (
            b"xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx".as_slice(),
            b"xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx".as_slice(),
        ),
    ] {
        let mut limits = f::limits();
        limits.client.domain.text = 256;
        limits.client.domain.long_text = 256;
        limits.client.domain.turn_text = 256;
        let shared = temper_client_view_words::words::Words::english(
            temper_client_protocol_wire::number(temper_client_protocol_wire::Owner::Forge),
            256,
            temper_client_view_words::words::Shell::Terminal,
        )
        .unwrap();
        let words = crate::words::Words::english(shared, &limits).unwrap();
        let notice = crate::Notice::Closed { task: 12, title: Some(title.into()) };
        let expected = skein_lib::bytes::concat(&[b"Closed T12", " · ".as_bytes(), cut]);
        assert_eq!(words.notice(&notice), expected);
        let mut row = working(12, b"chat", 1);
        row.chip.title = c::DocumentText::complete(c::Text::new(title.into(), 256).unwrap());
        let root = launchpad::home(None, &[row]);
        let mut tui = Tui::new(f::ADDRESS);
        tui.step(Told::Quitting);
        let options = view::front::quit(&root, &tui, &words).unwrap();
        let head = skein_lib::bytes::concat(&[b"Stop the box? ", cut, b"'s running task stops with it."]);
        assert_eq!(options.head.as_deref(), Some(head.as_ref()));
        assert_eq!(options.name, head);
    }
}
