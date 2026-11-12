use super::*;
use skein_io::kernel::Fd;

const PROCESS: Token = Token::new(9);
const LIFE: Token = Token::new(10);
const OUT: Token = Token::new(11);
const ERR: Token = Token::new(12);
const HANDED: Token = Token::new(13);
const LINES: Token = Token::new(14);
const URL: &[u8] = b"http://[::1]:9400/jig/v1/sign-in/link?secret=AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";

fn child() -> Child {
    Child::new(Front {
        executable: b"/bin/temper".as_slice().into(),
        cache: b"/cache".as_slice().into(),
        arguments: vec![
            Box::from(b"--fake-backends".as_slice()),
            Box::from(b"--data".as_slice()),
            Box::from(b"a data dir".as_slice()),
        ]
        .into(),
        environment: vec![
            Box::from(b"HOME=/home/person".as_slice()),
            Box::from(b"TEMPER_INTERNAL_SITE=https://localhost:9443".as_slice()),
        ]
        .into(),
        directory: Fd::new(7),
        log: b"/data/box/logs/box.log".as_slice().into(),
    })
}
fn boxed() -> Child {
    let mut child = child();
    child.wanted = Some(Kind::Box);
    child.found = Some(b"/cache/temper-box".as_slice().into());
    child.progress();
    child.requests.clear();
    assert!(child.event(&io::Event::Spawned {
        owner: BOX,
        child: PROCESS,
        pipes: Box::new([LIFE, OUT, ERR, HANDED, LINES])
    }));
    child.reports.clear();
    child
}
fn bytes(child: &mut Child, owner: Token, bytes: &[u8]) {
    assert!(child.event(&io::Event::Stream { owner, up: stream::Up::Bytes(bytes.into()) }));
}
fn end(child: &mut Child, owner: Token) {
    assert!(child.event(&io::Event::Stream { owner, up: stream::Up::End }));
    assert!(child.event(&io::Event::Closed { owner }));
}
fn stages(child: &mut Child) -> Vec<BoxStage> {
    let mut stages = Vec::new();
    while let Some(report) = child.reports.pop() {
        if let Report::Stage(stage) = report {
            stages.push(stage);
        }
    }
    stages
}

#[test]
fn the_fetchs_lines_and_exit_start_the_box_with_its_captured_inputs() {
    let mut child = child();
    child.progress();
    let Some(io::Request::SpawnAlone { owner: FETCH, spawn }) = child.requests.pop() else { panic!("fetch spawn") };
    assert_eq!(spawn.program.as_ref(), b"/bin/temper");
    assert_eq!(
        spawn.args.as_ref(),
        &[
            Box::from(b"internal".as_slice()),
            Box::from(b"fetch-box".as_slice()),
            Box::from(b"--cache".as_slice()),
            Box::from(b"/cache".as_slice())
        ]
    );
    assert_eq!(spawn.env, child.settings.environment);
    assert_eq!(spawn.root, Fd::new(7));
    assert_eq!(spawn.dir.as_ref(), b".");
    assert_eq!(
        spawn.pipes.iter().map(|pipe| (pipe.child, pipe.way, pipe.parent)).collect::<Vec<_>>(),
        vec![(0, Way::Null, None), (1, Way::Out, None), (2, Way::Out, None)]
    );
    assert!(child.event(&io::Event::Spawned { owner: FETCH, child: PROCESS, pipes: Box::new([OUT, ERR]) }));
    bytes(&mut child, OUT, b"fetched temper-box 7 of 10 bytes\n");
    bytes(&mut child, OUT, b"fetched temper-box 9 bytes\n");
    bytes(&mut child, OUT, b"found /cache/box with spaces\n");
    assert_eq!(
        stages(&mut child),
        vec![BoxStage::Fetching { done: 7, total: Some(10) }, BoxStage::Fetching { done: 9, total: None }]
    );
    end(&mut child, OUT);
    end(&mut child, ERR);
    child.progress();
    assert!(
        !child.requests.iter().any(|request| matches!(request, io::Request::SpawnAlone { .. })),
        "found is not an exit"
    );
    child.event(&io::Event::Exited { owner: FETCH, exit: Exit::Code(0) });
    child.progress();
    assert!(
        child.requests.iter().any(|request| matches!(request, io::Request::Abort { entity: PROCESS })),
        "reap the fetch"
    );
    child.event(&io::Event::Closed { owner: PROCESS });
    child.requests.clear();
    child.progress();
    let Some(io::Request::SpawnAlone { owner: BOX, spawn }) = child.requests.pop() else { panic!("box spawn") };
    assert_eq!(spawn.program.as_ref(), b"/cache/box with spaces");
    let mut expected = vec![Box::from(b"--front".as_slice())];
    expected.extend(child.settings.arguments.iter().cloned());
    assert_eq!(spawn.args.as_ref(), expected);
    assert_eq!(spawn.env, child.settings.environment);
    assert_eq!(spawn.root, Fd::new(7));
    assert_eq!(
        spawn.pipes.iter().map(|pipe| (pipe.child, pipe.way, pipe.parent)).collect::<Vec<_>>(),
        vec![(0, Way::In, None), (1, Way::Out, None), (2, Way::Out, None), (3, Way::Out, None), (4, Way::Out, None)]
    );
    assert_eq!(stages(&mut child), vec![BoxStage::Starting]);
}

#[test]
fn a_failed_fetch_keeps_its_stderr_and_tells_only_the_first_line() {
    for exit in [Exit::Code(0), Exit::Code(1), Exit::Signal(15)] {
        let mut child = child();
        child.progress();
        child.requests.clear();
        child.event(&io::Event::Spawned { owner: FETCH, child: PROCESS, pipes: Box::new([OUT, ERR]) });
        bytes(&mut child, ERR, b"temper: download refused\nFetch it by hand\n");
        end(&mut child, OUT);
        end(&mut child, ERR);
        child.event(&io::Event::Exited { owner: FETCH, exit });
        child.progress();
        child.event(&io::Event::Closed { owner: PROCESS });
        assert_eq!(stages(&mut child), vec![BoxStage::Failed { why: b"temper: download refused".as_slice().into() }]);
        assert_eq!(child.exit(), (1, Box::from(b"temper: download refused\nFetch it by hand\n".as_slice())));
        assert!(child.wanted.is_none(), "no found path, even on exit 0");
    }
}

#[test]
fn a_handed_link_waits_for_its_end_and_gives_the_engine_its_authority() {
    for size in [URL.len(), 4096] {
        let mut child = boxed();
        let mut url = URL.to_vec();
        url.resize(size, b'a');
        for part in url.chunks(19) {
            bytes(&mut child, HANDED, part);
        }
        assert!(child.reports.is_empty(), "no Running or link before EOF");
        end(&mut child, HANDED);
        assert!(matches!(child.reports.pop(), Some(Report::Stage(BoxStage::Running))));
        let Some(Report::Link { engine, address, link }) = child.reports.pop() else { panic!("handed link") };
        assert_eq!(engine, io::Addr::Ip("[::1]:9400".parse().expect("link address")));
        assert_eq!(address.as_ref(), b"[::1]:9400");
        assert_eq!(link.expose(), url);
    }
}

#[test]
fn a_missing_or_past_bound_link_fails_and_stops_the_box() {
    let mut past = URL.to_vec();
    past.resize(4097, b'a');
    for link in [b"".as_slice(), b"https://127.0.0.1:9400/link", b"http://127.0.0.1/link", b"http://:9400/link", &past]
    {
        let mut child = boxed();
        bytes(&mut child, HANDED, link);
        end(&mut child, HANDED);
        assert_eq!(stages(&mut child), vec![BoxStage::Failed { why: b"temper-box handed no link".as_slice().into() }]);
        assert!(
            child.requests.iter().any(|request| matches!(request, io::Request::Abort { entity: LIFE })),
            "a failed link stops the box"
        );
        assert!(child.link.is_empty(), "bounded, forgotten handed bytes");
    }
}

#[test]
fn the_first_stop_closes_life_and_the_second_signals_the_child() {
    let mut child = boxed();
    child.stop(Stop::First);
    assert!(matches!(child.requests.pop(), Some(io::Request::Abort { entity: LIFE })));
    assert!(child.requests.is_empty(), "no signal at the first stop");
    child.stop(Stop::Second);
    assert!(matches!(
        child.requests.pop(),
        Some(io::Request::Signal { child: PROCESS, signal: Signal::Terminate, to: Target::Child })
    ));
    assert!(child.requests.is_empty());
    assert!(child.ended.is_none(), "both stops wait on the box");
}

#[test]
fn box_lines_are_parsed_and_an_end_waits_for_the_lines_and_output() {
    let mut child = boxed();
    bytes(&mut child, LINES, b"log recent\n");
    bytes(&mut child, LINES, b"unknown ignored\n");
    bytes(&mut child, LINES, &vec![b'x'; BoxLine::MOST]);
    bytes(&mut child, LINES, b"log forged suffix\n");
    bytes(&mut child, LINES, b"running serve\n");
    assert!(matches!(child.reports.pop(), Some(Report::Line(BoxLine::Log { line })) if line.as_ref() == b"recent"));
    assert!(matches!(child.reports.pop(), Some(Report::Line(BoxLine::Running { step })) if step.as_ref() == b"serve"));
    assert!(child.reports.is_empty());
    bytes(&mut child, HANDED, URL);
    end(&mut child, HANDED);
    child.reports.clear();
    child.event(&io::Event::Exited { owner: BOX, exit: Exit::Code(2) });
    end(&mut child, OUT);
    end(&mut child, ERR);
    child.progress();
    assert!(!child.reaping, "lines must be drained before Ended");
    end(&mut child, LINES);
    child.progress();
    assert!(child.reaping);
    child.event(&io::Event::Closed { owner: PROCESS });
    assert_eq!(stages(&mut child), vec![BoxStage::Ended { end: BoxEnd::Exited { code: 2 } }]);
}

#[test]
fn the_boxs_exit_keeps_one_eight_kib_tail_in_read_order() {
    const KEPT: usize = 8 * 1024;
    for exit in [Exit::Code(0), Exit::Code(2), Exit::Signal(9)] {
        let mut child = boxed();
        bytes(&mut child, OUT, &vec![b'a'; KEPT]);
        bytes(&mut child, ERR, &vec![b'b'; KEPT - 3]);
        bytes(&mut child, OUT, b"end");
        assert_eq!(child.tail.len(), KEPT);
        assert_eq!(child.tail, [vec![b'b'; KEPT - 3], b"end".to_vec()].concat());
        child.box_exit = Some(exit);
        let (code, words) = child.exit();
        let (head, expected) = match exit {
            Exit::Code(code) => (format!("temper: temper-box exited {code}\n"), code),
            Exit::Signal(signal) => (format!("temper: temper-box killed by signal {signal}\n"), 1),
        };
        assert_eq!(code, expected);
        assert_eq!(
            words.as_ref(),
            [head.as_bytes(), child.tail.as_slice(), b"\nLog: /data/box/logs/box.log\n"].concat()
        );
        child.stop = Some(Stop::First);
        if exit == Exit::Code(0) {
            assert_eq!(child.exit(), (0, Box::<[u8]>::default()));
        }
    }
    let mut child = boxed();
    bytes(&mut child, ERR, b"arguments refused\n");
    end(&mut child, HANDED);
    child.box_exit = Some(Exit::Code(2));
    assert_eq!(
        child.exit(),
        (2, Box::from(b"temper: temper-box exited 2\narguments refused\nLog: /data/box/logs/box.log\n".as_slice()))
    );
}
