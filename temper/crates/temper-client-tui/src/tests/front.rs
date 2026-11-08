//! The front's views (temper/docs/design/client/README.md, 6.17).
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

fn front() -> Tui {
    let mut tui = Tui::new(f::ADDRESS);
    tui.step(Told::Front);
    tui.step(Told::Size { columns: FULL.columns });
    tui
}
