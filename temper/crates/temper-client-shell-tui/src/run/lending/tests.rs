use super::*;
use crate::{
    journey, limits,
    run::{Front, Settings},
};
use skein_lib::stream;
use skein_ui_screen::Size;
use skein_ui_screen_terminal::Depth;
use temper_client_tui::Scheme;

fn ready(found: bool) -> Run {
    let settings = Settings {
        engine: io::Addr::Ip("127.0.0.1:8367".parse().expect("loopback")),
        directory: b"tui".as_slice().into(),
        address: b"127.0.0.1:8367".as_slice().into(),
        scheme: Some(Scheme::Dark),
        boxed: true,
        link_file: None,
        note: None,
        files: None,
        depth: Depth::Indexed,
        size: Size { rows: 24, columns: 80 },
        limits: limits::tui(),
        opener: None,
        record: Some(Fd::new(8)),
        front: Some(Front {
            executable: b"/bin/temper".as_slice().into(),
            directories: temper_shell::directories::Directories {
                config: b"/config".as_slice().into(),
                data: b"/data".as_slice().into(),
                cache: b"/cache".as_slice().into(),
                origins: [skein_shell::Origin::Flag; 3],
            },
            arguments: Box::new([]),
            environment: Box::new([]),
            directory: Fd::new(5),
            log: b"/data/box/logs/box.log".as_slice().into(),
        }),
    };
    let mut run =
        Run::new(settings, Fd::new(3), 1000, 7, Fd::new(0), Fd::new(1), Fd::new(4)).expect("synthetic descriptors");
    run.started(None, Vec::new());
    if found {
        let front = run.box_child.as_mut().expect("front");
        front.progress();
        front.requests.clear();
        assert!(front.event(&io::Event::Spawned {
            owner: super::super::front::FETCH,
            child: Token::new(11),
            pipes: Box::new([Token::new(12), Token::new(13)])
        }));
        assert!(front.event(&io::Event::Stream {
            owner: Token::new(12),
            up: stream::Up::Bytes(b"found /found/temper-box\n".as_slice().into())
        }));
    }
    run
}

#[test]
fn a_bad_name_or_missing_found_path_reports_not_started_without_spawning() {
    for (name, found) in [
        (b"".as_slice(), true),
        (b"abcdefghijklmnopqrstuvwxyz0123456", true),
        (b"Work", true),
        (b"../work", true),
        (b"work\0", true),
        (b"work --data other", true),
        (b"work", false),
    ] {
        let mut run = ready(found);
        run.lending.begin(host::Lend { number: 1, name: name.into(), line: b"line".as_slice().into() });
        // The terminal and line have completed; these rows pin the next step's refusal.
        run.lending.phase = Phase::Starting;
        assert!(run.lending_progress(), "refusal returns the terminal: {name:?}, found={found}");
        assert!(!run.lending.busy());
        assert!(!run.raw.iter().any(|submit| matches!(submit.kind, Op::Spawn { .. })), "nothing spawned");
        let recorder = run.recording.as_mut().expect("recording").recorder.as_mut().expect("started");
        let bytes = recorder.save(&mut run.host);
        let lines = journey::lines(&bytes).expect("recorded reports");
        assert!(lines.contains(&journey::Line::Lent { number: 1, exit: LoginExit::NotStarted }));
    }
}
