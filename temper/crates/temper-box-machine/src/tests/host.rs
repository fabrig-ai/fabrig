use super::fixtures::{Harness, grouped_steps, long, once};
use crate::host::{LINE_BYTES, io_limits};
use alloc::boxed::Box;
use skein_io::kernel::{Done, Exit, Target};
use skein_lib::{Duration, List, bytes};
use skein_sim::{Event, Summary};
use temper_forge_setup::Step;

const FIRST_URL: &[u8] = b"http://local/first";
const FIRST_FOOTER: &[u8] = b"\nIt signs you in as this temper's owner, once, within 2 hours.\nOpened after that, it prints a fresh one here.\nIn a terminal: temper http://local\n";
const FIRST_SECRET: &[u8] = &[0x83; 32];
const AFTER_BLOCK: &[u8] = b"Additional startup information.\n";
fn first_link_harness(steps: &[Step], items: &[skein_fake_machine::Item]) -> Harness {
    Harness::with_terminal(
        temper_forge_setup::Steps { always: Box::from(steps), asked: Box::new([]) },
        items,
        true,
        16_384,
        &crate::limits::small(),
        crate::host::Echo::None,
        super::fixtures::Opening {
            parent: None,
            link_form: crate::terminal::LinkForm::Bare,
            log_lines: Box::new([]),
            front: None,
            first_link: Some(crate::host::FirstLink {
                url: bytes::copy_of(FIRST_URL),
                local: None,
                secret: skein_lib::Secret::new(bytes::copy_of(FIRST_SECRET)),
            }),
            after_block: bytes::copy_of(AFTER_BLOCK),
        },
    )
}
const LOCAL_URL: &[u8] = b"http://local/local";
const LOCAL_SECRET: &[u8] = &[0x5c; 32];
fn front_harness(steps: &[Step], parent: Option<u32>) -> Harness {
    front_harness_buffer(steps, parent, 16_384)
}
fn front_harness_buffer(steps: &[Step], parent: Option<u32>, buffer: u32) -> Harness {
    Harness::with_terminal(
        temper_forge_setup::Steps { always: Box::from(steps), asked: Box::new([]) },
        &[],
        true,
        buffer,
        &crate::limits::small(),
        crate::host::Echo::None,
        super::fixtures::Opening {
            parent,
            link_form: crate::terminal::LinkForm::Bare,
            log_lines: Box::new([]),
            first_link: Some(crate::host::FirstLink {
                url: bytes::copy_of(FIRST_URL),
                secret: skein_lib::Secret::new(bytes::copy_of(FIRST_SECRET)),
                local: Some(skein_lib::Secret::new(bytes::copy_of(LOCAL_SECRET))),
            }),
            front: Some(skein_lib::Secret::new(bytes::copy_of(LOCAL_URL))),
            after_block: bytes::copy_of(AFTER_BLOCK),
        },
    )
}
fn child_processes(h: &Harness) -> List<skein_io::kernel::Fd> {
    let mut children = List::with_capacity(64);
    for entry in h.sim.trace() {
        if entry.pid == h.pid
            && let Event::Complete { result: Ok(Done::Spawned { pidfd, .. }), .. } = &entry.event
        {
            children.push(*pidfd).unwrap();
        }
    }
    children
}
fn front_signals(h: &Harness) -> List<(skein_io::kernel::Fd, skein_lib::Time)> {
    let mut signals = List::with_capacity(128);
    for entry in h.sim.trace() {
        if entry.pid == h.pid
            && let Event::Submit { kind: Summary::Signal { pidfd, signal: 15, to: Target::Child }, .. } = &entry.event
        {
            signals.push((*pidfd, entry.at)).unwrap();
        }
    }
    signals
}
fn process_closed_at(h: &Harness, process: skein_io::kernel::Fd) -> Option<skein_lib::Time> {
    for entry in h.sim.trace() {
        if entry.pid == h.pid
            && let Event::Submit { kind: Summary::Close { fd }, .. } = entry.event
            && fd == process
        {
            return Some(entry.at);
        }
    }
    None
}
fn assert_front_settled(h: &Harness) {
    h.sim.assert_quiescent(h.pid);
    h.sim.assert_no_open_fds(h.pid);
    assert_eq!(h.machine.open_handles(), 0);
}
fn named_log(h: &mut Harness, name: &[u8]) -> Box<[u8]> {
    let root = h.machine.reopen_root(0);
    let fd = h
        .machine
        .open(root, &crate::lines::joined(&[b"box/logs/", name]), skein_fake_machine::How::ReadNoFollow)
        .unwrap();
    assert_eq!(h.machine.stat(fd).mode, 0o600);
    let words = h.machine.read(fd, 0, 65_536).unwrap().into_boxed_slice();
    h.machine.close(fd);
    h.machine.close(root);
    words
}
#[test]
fn the_front_writes_the_local_link_before_any_step() {
    for parent in [None, Some(4321)] {
        let mut h = front_harness_buffer(&[long(b"serve", &[b"never"])], parent, 8);
        h.run_until_idle();
        assert!(h.launches.is_empty(), "a held local link holds every step");
        let [_, link, _] = h.front_pipes.unwrap();
        let mut url = List::uncapped();
        for _ in 0..LOCAL_URL.len() {
            for byte in h.sim.peer_drain(h.pid, link, 4096) {
                url.push(byte).unwrap();
            }
            h.run_until_idle();
            if h.sim.peer_eof(h.pid, link) {
                break;
            }
        }
        assert_eq!(url.as_slice(), LOCAL_URL, "the local URL alone, without LF");
        assert!(h.sim.peer_eof(h.pid, link), "the local link is closed");
        assert_eq!(h.launches.len(), 1, "no front child");
        let launch = h.launches.get(0).unwrap();
        assert_eq!(launch.name.as_ref(), b"serve");
        assert_spawn_carries_no_link(launch);
        for pipe in &launch.pipes {
            assert!(pipe.parent.is_none(), "no parent's descriptors passed");
        }
        let serve = h.child_index(b"serve");
        let mut secret = List::uncapped();
        for _ in 0_u32..8 {
            for byte in &h.read_stdin(serve, 8) {
                secret.push(*byte).unwrap();
            }
        }
        assert_eq!(secret.as_slice(), crate::lines::joined(&[FIRST_SECRET, LOCAL_SECRET]).as_ref());
        assert!(h.read_stdin(serve, 8).is_empty());
        let logged = h.log();
        let words = bytes::find(&logged, b"Open this link in a browser on this machine.\n").unwrap();
        let start = bytes::find(&logged, b"Starting serve\n").unwrap();
        assert!(words < start);
        assert!(Harness::contains(&logged, b"Additional startup information.\n"));
        for hidden in [FIRST_URL, LOCAL_URL, FIRST_SECRET, LOCAL_SECRET] {
            assert!(!Harness::contains(&logged, hidden));
        }
        assert!(h.stdout().is_empty());
        assert!(h.stderr().is_empty());
        end_front(&mut h);
        assert_eq!(h.run_to_end(), Exit::Code(0));
        assert_front_settled(&h);
    }
}
fn end_front(h: &mut Harness) {
    h.sim.peer_close(h.pid, h.front_pipes.unwrap()[0]);
}
fn front_bytes(h: &mut Harness, length: usize) -> Box<[u8]> {
    let fd = h.front_pipes.unwrap()[2];
    let mut got = List::uncapped();
    for _ in 0..length {
        if usize::try_from(got.len()).unwrap() >= length {
            break;
        }
        let left = length.checked_sub(usize::try_from(got.len()).unwrap()).unwrap();
        let chunk = h.sim.peer_drain(h.pid, fd, left);
        for byte in &chunk {
            got.push(*byte).unwrap();
        }
        h.run_until_idle();
    }
    got.into_boxed()
}
fn front_log_lines(log: &[u8]) -> Box<[u8]> {
    let mut parts = List::uncapped();
    for line in log.split_inclusive(|byte| *byte == b'\n') {
        parts.push(b"log ".as_slice()).unwrap();
        parts.push(line).unwrap();
    }
    crate::lines::joined(parts.as_slice())
}
fn read_front_start(h: &mut Harness) {
    let first = crate::lines::joined(&[b"link ", FIRST_URL, b"\n"]);
    assert!(front_bytes(h, first.len()) == first, "the printed link is first");
    let expected = front_log_lines(&h.log());
    assert_eq!(front_bytes(h, expected.len()), expected);
}
#[test]
fn the_front_is_told_the_link_first_then_each_box_log_line() {
    let mut h = front_harness(&[long(b"serve", &[b"never"])], None);
    h.run_until_idle();
    read_front_start(&mut h);
    let serve = h.child_index(b"serve");
    h.write_stdout(serve, b"http://next\n");
    let expected = b"link http://next\n";
    assert!(front_bytes(&mut h, expected.len()).as_ref() == expected, "the fresh link goes to the front");
    assert!(h.stdout().is_empty());
    assert!(h.stderr().is_empty());
    for log in [h.log(), named_log(&mut h, b"serve.log")] {
        for url in [FIRST_URL, b"http://next".as_slice()] {
            assert!(!Harness::contains(&log, url), "logs hold no printed link");
        }
    }
    end_front(&mut h);
    assert_eq!(h.run_to_end(), Exit::Code(0));
    assert_front_settled(&h);
}
#[test]
#[expect(clippy::too_many_lines, reason = "Each row exercises a terminal route on the parent pipes.")]
fn the_terminals_lines_go_to_box_log_and_the_front_while_it_runs() {
    for row in ["restart", "unknown", "pipe too long", "link too long", "step log", "box log", "give up"] {
        let mut h = front_harness(&[long(b"forgejo", &[b"never"]), long(b"serve", &[b"never"])], None);
        if row == "step log" {
            h.refuse_log = Some(bytes::copy_of(b"forgejo.log"));
        }
        if row == "box log" {
            h.refuse_log = Some(bytes::copy_of(b"box.log"));
        }
        h.run_until_idle();
        if row == "box log" {
            // No file holds these lines; the front is still their sink.
            let expected = b"log 00:00:00Z box: cannot write /data/box/logs/box.log";
            let first = crate::lines::joined(&[b"link ", FIRST_URL, b"\n"]);
            assert!(front_bytes(&mut h, first.len()) == first, "the printed link is first");
            let mut got = List::uncapped();
            for _ in 0_u32..10 {
                for _ in 0_u32..1024 {
                    let byte = front_bytes(&mut h, 1);
                    got.push(*byte.first().unwrap()).unwrap();
                    if byte.as_ref() == b"\n" {
                        break;
                    }
                }
            }
            assert!(Harness::contains(got.as_slice(), expected), "failed box log notice reaches the front");
        } else {
            read_front_start(&mut h);
        }
        match row {
            "restart" | "give up" => {
                let delays = if row == "restart" { &[500][..] } else { &[500, 1_000, 2_000, 4_000, 8_000][..] };
                for delay in delays {
                    let before = h.log().len();
                    let forge = h.child_index(b"forgejo");
                    h.write_child(forge, 2, b"boom\n");
                    finish_child(&mut h, b"forgejo", 1);
                    h.run_until_idle();
                    let block = b"00:00:00Z forgejo exited 1; restarting in ";
                    assert!(Harness::contains(&h.log(), block));
                    assert!(Harness::contains(
                        &h.log(),
                        b"00:00:00Z     forgejo: boom\n00:00:00Z     Log: /data/box/logs/forgejo.log\n"
                    ));
                    let logged = h.log();
                    let lines = front_log_lines(logged.get(before..).unwrap());
                    let want = crate::lines::joined(&[b"restarting forgejo\n", &lines]);
                    assert_eq!(front_bytes(&mut h, want.len()), want);
                    h.sim.advance_to(h.sim.now().saturating_add(Duration::from_millis(*delay)));
                    h.run_until_idle();
                    let expected = b"log 00:00:00Z Starting forgejo\nrunning forgejo\n";
                    assert_eq!(front_bytes(&mut h, expected.len()).as_ref(), expected);
                }
                if row == "give up" {
                    finish_child(&mut h, b"forgejo", 1);
                    h.run_until_idle();
                    let expected = b"gave-up forgejo\nlog 00:00:00Z forgejo exited 1; giving up\n";
                    assert_eq!(front_bytes(&mut h, expected.len()).as_ref(), expected);
                }
            }
            "pipe too long" | "link too long" => {
                let message = if row == "pipe too long" {
                    h.ask(&[b'x'; 300]);
                    h.run_until_idle();
                    h.ask(b"\n");
                    b"box: serve: pipe line too long\n".as_slice()
                } else {
                    let serve = h.child_index(b"serve");
                    let line = crate::lines::joined(&[&[b'x'; 4097], b"\n"]);
                    h.write_stdout(serve, &line);
                    b"box: serve: sign-in link too long\n".as_slice()
                };
                h.run_until_idle();
                let expected = crate::lines::joined(&[b"log 00:00:00Z ", message]);
                assert_eq!(front_bytes(&mut h, expected.len()), expected);
                assert!(Harness::contains(&h.log(), &crate::lines::joined(&[b"00:00:00Z ", message])));
            }
            "unknown" | "box log" => {
                h.ask(b"bogus\n");
                h.run_until_idle();
                let expected = b"log 00:00:00Z box: serve: unknown pipe line\n";
                assert_eq!(front_bytes(&mut h, expected.len()).as_ref(), expected);
                if row == "unknown" {
                    assert!(Harness::contains(&h.log(), b"00:00:00Z box: serve: unknown pipe line\n"));
                }
            }
            "step log" => {
                assert!(Harness::contains(&h.log(), b"00:00:00Z box: cannot write /data/box/logs/forgejo.log"));
                let forge = h.child_index(b"forgejo");
                h.write_child(forge, 2, b"boom\n");
                h.run_until_idle();
                let expected = b"log 00:00:00Z forgejo: boom\n";
                assert_eq!(front_bytes(&mut h, expected.len()).as_ref(), expected);
                assert!(Harness::contains(&h.log(), b"00:00:00Z forgejo: boom\n"));
            }
            _ => unreachable!(),
        }
        assert!(h.stdout().is_empty(), "{row}");
        assert!(h.stderr().is_empty(), "{row}");
        end_front(&mut h);
        h.run_until_idle();
        assert_eq!(h.run_to_end(), Exit::Code(u8::from(row == "give up")), "{row}");
        assert!(h.stderr().is_empty());
        assert_front_settled(&h);
    }
}
#[test]
fn a_front_that_reads_nothing_holds_256_lines() {
    let mut h = front_harness_buffer(&[long(b"serve", &[b"never"])], None, 8);
    h.run_until_idle();
    let link = h.front_pipes.unwrap()[1];
    for _ in 0..LOCAL_URL.len() {
        h.sim.peer_drain(h.pid, link, 4096);
        h.run_until_idle();
    }
    for _ in 0_u32..300 {
        h.ask(b"bogus\n");
        h.run_until_idle();
        assert!(crate::host::pressure::front_held(&h.service) <= crate::front::HELD);
        assert!(crate::host::pressure::bounded(&h.service));
    }
    assert_eq!(crate::host::pressure::front_held(&h.service), crate::front::HELD);
    assert_eq!(bytes::count(&h.log(), b"box: serve: unknown pipe line\n", u32::MAX), 300);
    h.ask(b"start 3\n");
    h.run_until_idle();
    assert!(Harness::contains(&h.reply_bytes(), b"failed 3 not asked\n"));
    finish_child(&mut h, b"serve", 1);
    h.run_until_idle();
    h.sim.advance_to(h.sim.now().saturating_add(Duration::from_millis(500)));
    h.run_until_idle();
    assert_eq!(h.spawn_times(b"serve").len(), 2);
    let serve = h.child_index(b"serve");
    h.write_stdout(serve, b"http://dropped\n");
    let mut received = List::uncapped();
    for _ in 0_u32..4096 {
        h.run_until_idle();
        let chunk = h.sim.peer_drain(h.pid, h.front_pipes.unwrap()[2], 8192);
        if chunk.is_empty() && crate::host::pressure::front_held(&h.service) == 0 {
            break;
        }
        for byte in chunk {
            received.push(byte).unwrap();
        }
    }
    assert!(!Harness::contains(received.as_slice(), b"http://dropped"), "a line past 256 held is dropped");
    assert!(h.stdout().is_empty());
    assert!(h.stderr().is_empty());
    end_front(&mut h);
    assert_eq!(h.run_to_end(), Exit::Code(0));
    assert_front_settled(&h);
}
#[test]
fn the_fronts_input_end_stops_and_drains_the_box() {
    let mut h = front_harness(&[long(b"serve", &[b"never"])], None);
    h.run_until_idle();
    read_front_start(&mut h);
    end_front(&mut h);
    h.run_until_idle();
    let child = *child_processes(&h).get(0).unwrap();
    assert_eq!(front_signals(&h).len(), 1, "EOF is the first signal");
    assert!(process_closed_at(&h, child).is_none(), "first stop drains the step");
    finish_child(&mut h, b"serve", 0);
    assert_eq!(h.run_to_end(), Exit::Code(0));
    assert!(Harness::contains(&h.log(), b"Stopping\n00:00:00Z Stopped\n"));
    assert_eq!(front_bytes(&mut h, 60).as_ref(), b"log 00:00:00Z Stopping\nlog 00:00:00Z Stopped\n");
    assert!(h.stdout().is_empty());
    assert!(h.stderr().is_empty());
    assert_front_settled(&h);
}
#[test]
fn a_signal_after_the_fronts_input_end_closes_the_steps() {
    let mut h = front_harness(&[long(b"serve", &[b"never"])], None);
    h.run_until_idle();
    end_front(&mut h);
    h.run_until_idle();
    let child = *child_processes(&h).get(0).unwrap();
    let now = h.sim.now();
    h.terminate();
    assert_eq!(h.run_to_end(), Exit::Code(0));
    assert_eq!(front_signals(&h).len(), 1);
    assert_eq!(process_closed_at(&h, child), Some(now));
    assert_front_settled(&h);
}
#[test]
fn a_give_up_writes_its_lines_and_exits_without_waiting_for_the_parent() {
    for buffer in [8, 16_384] {
        let mut h = front_harness_buffer(&[once(b"prepare", &[b"exit", b"1"])], None, buffer);
        h.run_until_idle();
        let link = h.front_pipes.unwrap()[1];
        for _ in 0..LOCAL_URL.len() {
            h.sim.peer_drain(h.pid, link, 4096);
            h.run_until_idle();
        }
        assert_eq!(h.run_to_end(), Exit::Code(1));
        assert!(Harness::contains(&h.log(), b"prepare exited 1; giving up\n"));
        if buffer == 16_384 {
            let lines = h.sim.peer_drain(h.pid, h.front_pipes.unwrap()[2], 16_384);
            assert!(Harness::contains(&lines, b"gave-up prepare\n"));
            assert!(Harness::contains(&lines, b"log 00:00:00Z prepare exited 1; giving up\n"));
        }
        assert!(h.stdout().is_empty());
        assert!(h.stderr().is_empty());
        assert_front_settled(&h);
    }
}
fn stop_first_link(h: &mut Harness) {
    h.terminate();
    assert_eq!(h.run_to_end(), Exit::Code(0));
    h.sim.assert_quiescent(h.pid);
    h.sim.assert_no_open_fds(h.pid);
    assert_eq!(h.machine.open_handles(), 0);
}
#[test]
fn the_block_is_the_terminal_before_any_spawn() {
    use skein_fake_machine::Item;
    let mut h = first_link_harness(
        &[long(b"serve", &[b"never"])],
        &[Item::directory(b"box"), Item::directory(b"box/logs"), Item::file(b"box/logs/serve.log", b"last run\n")],
    );
    h.hold_terminal_ack = true;
    h.run_until_idle();
    assert!(h.launches.is_empty());
    assert_eq!(h.stderr().as_ref(), crate::terminal::PRELUDE);
    assert!(h.stdout().is_empty());
    let root = h.machine.reopen_root(0);
    let old = h.machine.open(root, b"box/logs.previous/serve.log", skein_fake_machine::How::ReadNoFollow).unwrap();
    assert_eq!(h.machine.read(old, 0, 128).unwrap().as_slice(), b"last run\n");
    h.machine.close(old);
    h.machine.close(root);
    h.release_terminal_ack();
    h.run_until_idle();
    assert_eq!(h.stdout().as_ref(), b"http://local/first\n");
    assert_eq!(h.stderr().as_ref(), crate::lines::joined(&[FIRST_FOOTER, AFTER_BLOCK]).as_ref());
    assert_eq!(h.launches.len(), 1);
    stop_first_link(&mut h);
    assert!(!Harness::contains(&h.terminal_bytes(), crate::terminal::CTRL_C));
    assert!(Harness::contains(&h.log(), b"Logs: /data/box/logs/\n00:00:00Z Ctrl-C stops temper.\n"));
}
#[test]
fn a_held_block_holds_the_first_spawn() {
    use skein_shell::Host;
    let mut h = first_link_harness(&[long(b"serve", &[b"never"])], &[]);
    h.hold_terminal_ack = true;
    h.run_until_idle();
    assert!(!h.held_terminal_ack.is_empty());
    assert!(h.launches.is_empty(), "even a written prelude still awaits its acknowledgement");
    assert!(!h.service.work_pending(h.sim.now()), "a held write lets the host sleep");
    for (target, expected) in
        [(h.stdout, b"http://local/first\n".as_slice()), (h.stderr, FIRST_FOOTER), (h.stderr, AFTER_BLOCK)]
    {
        h.release_terminal_ack();
        h.turn();
        h.hold_terminal_ack = true;
        h.run_until_idle();
        assert!(h.launches.is_empty(), "each item holds startup until its write is acknowledged");
        assert!(!h.held_terminal_ack.is_empty());
        assert!(!h.service.work_pending(h.sim.now()));
        let got = if target == h.stdout { h.stdout() } else { h.stderr() };
        let expected = if expected == FIRST_FOOTER {
            crate::lines::joined(&[crate::terminal::PRELUDE, expected])
        } else {
            bytes::copy_of(expected)
        };
        assert_eq!(got, expected);
    }
    h.release_terminal_ack();
    h.run_until_idle();
    assert_eq!(
        h.terminal_bytes(),
        crate::lines::joined(&[crate::terminal::PRELUDE, FIRST_URL, b"\n", FIRST_FOOTER, AFTER_BLOCK,])
    );
    assert_eq!(h.launches.len(), 1);
    stop_first_link(&mut h);
}
fn stdin_pipes(launch: &super::fixtures::Launch) -> u32 {
    let mut count = 0_u32;
    for pipe in &launch.pipes {
        if pipe.child == 0 {
            count = count.checked_add(1).unwrap();
        }
    }
    count
}
fn assert_spawn_carries_no_link(launch: &super::fixtures::Launch) {
    for value in launch.args.iter().chain(launch.environment.iter()) {
        assert!(!Harness::contains(value, FIRST_SECRET), "spawn metadata contains secret bytes");
        assert!(!Harness::contains(value, FIRST_URL), "spawn metadata contains the link");
        assert!(!Harness::contains(value, LOCAL_SECRET), "spawn metadata contains local secret bytes");
        assert!(!Harness::contains(value, LOCAL_URL), "spawn metadata contains the local link");
    }
}
#[test]
fn the_secret_is_on_serves_first_stdin_alone() {
    use skein_io::kernel::Way;
    let mut h = first_link_harness(
        &[
            long(b"other", &[b"never"]),
            long(b"serve", &[b"never"]).listening(Box::new([([127, 0, 0, 1], 8124).into()])),
        ],
        &[],
    );
    h.run_until_idle();
    for launch in &h.launches {
        assert_spawn_carries_no_link(launch);
        let mut stdin = List::with_capacity(1);
        for pipe in &launch.pipes {
            if pipe.child == 0 {
                stdin.push(pipe).expect("one stdin pipe");
            }
        }
        if launch.name.as_ref() == b"serve" {
            assert_eq!(stdin.len(), 1);
            assert_eq!(stdin.get(0).unwrap().way, Way::In);
            assert_eq!(stdin.get(0).unwrap().parent, None);
            assert_eq!(launch.pipes.last().unwrap().child, 0);
        } else {
            assert!(stdin.is_empty(), "other steps receive no secret pipe");
        }
    }
    let serve = h.child_index(b"serve");
    let got = h.read_stdin(serve, 64);
    assert!(got.as_ref() == FIRST_SECRET, "stdin holds exactly the secret bytes");
    assert!(h.read_stdin(serve, 64).is_empty(), "the secret ends with EOF");
    assert!(!Harness::contains(&h.log(), FIRST_SECRET));
    assert!(!Harness::contains(&h.terminal_bytes(), FIRST_SECRET));
    stop_first_link(&mut h);
}
#[test]
fn a_restarted_serve_has_no_stdin_and_its_link_is_fresh() {
    let mut h = first_link_harness(&[long(b"serve", &[b"never"])], &[]);
    h.run_until_idle();
    h.stdout();
    h.stderr();
    for url in [b"http://next\n".as_slice(), b"http://another\n"] {
        h.write_child(h.child_index(b"serve"), 1, url);
        h.run_until_idle();
        assert_eq!(h.stdout().as_ref(), url);
        assert_eq!(h.stderr().as_ref(), b"\n");
    }
    let pid = h.hosted.get(h.child_index(b"serve")).unwrap().pid;
    h.sim.finish_service(pid, Exit::Code(1));
    h.run_until_idle();
    h.sim.advance_to(h.sim.now().saturating_add(Duration::from_millis(500)));
    h.run_until_idle();
    assert_eq!(h.launches.len(), 2);
    let second = h.launches.get(1).unwrap();
    assert_spawn_carries_no_link(second);
    assert_eq!(stdin_pipes(second), 0);
    h.stderr();
    h.write_child(h.child_index(b"serve"), 1, b"http://fresh\n");
    h.run_until_idle();
    assert_eq!(h.stdout().as_ref(), b"http://fresh\n");
    assert_eq!(h.stderr().as_ref(), b"\n");
    stop_first_link(&mut h);
}
#[test]
fn a_termination_before_the_block_is_written_starts_no_step() {
    for before_rotation in [true, false] {
        let mut h = first_link_harness(&[long(b"serve", &[b"never"])], &[]);
        h.hold_terminal_ack = true;
        if !before_rotation {
            h.run_until_idle();
        }
        h.terminate();
        h.run_until_idle();
        assert!(h.launches.is_empty());
        h.release_terminal_ack();
        assert_eq!(h.run_to_end(), Exit::Code(0));
        assert!(h.launches.is_empty(), "stopping never starts the held step");
        h.sim.assert_no_open_fds(h.pid);
        assert_eq!(h.machine.open_handles(), 0);
    }
}

#[test]
fn a_failed_first_spawn_spends_the_secret() {
    let mut h = first_link_harness(&[long(b"serve", &[b"never"])], &[]);
    h.hold_spawn = Some(bytes::copy_of(b"serve"));
    h.run_until_idle();
    assert!(h.held_spawn.is_some());
    assert_eq!(stdin_pipes(h.launches.get(0).unwrap()), 1);
    h.fail_spawn();
    h.run_until_idle();
    h.sim.advance_to(h.sim.now().saturating_add(Duration::from_millis(500)));
    h.run_until_idle();
    assert_eq!(h.launches.len(), 2);
    assert_eq!(stdin_pipes(h.launches.get(1).unwrap()), 0, "a refused first spawn spends the secret");
    stop_first_link(&mut h);
}
#[test]
fn startup_log_and_record_diagnostics_follow_the_block() {
    use skein_fake_machine::Item;
    let mut h = Harness::with_terminal(
        grouped_steps(),
        &[Item::directory(b"box"), Item::directory(b"box/started"), Item::file(b"box/started/3", b"").mode(0)],
        true,
        16_384,
        &crate::limits::small(),
        crate::host::Echo::None,
        super::fixtures::Opening {
            parent: None,
            link_form: crate::terminal::LinkForm::Bare,
            log_lines: Box::new([]),
            front: None,
            first_link: Some(crate::host::FirstLink {
                url: bytes::copy_of(FIRST_URL),
                local: None,
                secret: skein_lib::Secret::new(bytes::copy_of(FIRST_SECRET)),
            }),
            after_block: bytes::copy_of(AFTER_BLOCK),
        },
    );
    h.refuse_log = Some(bytes::copy_of(b"serve.log"));
    h.hold_terminal_ack = true;
    h.run_until_idle();
    assert_eq!(h.stderr().as_ref(), crate::terminal::PRELUDE);
    assert!(h.stdout().is_empty());
    assert!(h.launches.is_empty());
    h.release_terminal_ack();
    h.run_until_idle();
    assert_eq!(h.stdout().as_ref(), b"http://local/first\n");
    let stderr = h.stderr();
    assert!(stderr.starts_with(&crate::lines::joined(&[FIRST_FOOTER, AFTER_BLOCK])));
    assert!(Harness::contains(&stderr, b"box: cannot write /data/box/logs/serve.log"));
    assert!(Harness::contains(&stderr, b"box: open box/started/3: permission denied"));
    stop_first_link(&mut h);
}
#[test]
fn with_no_first_link_after_block_is_at_the_start_and_the_first_serve_link_gets_the_block() {
    let mut h = Harness::with_terminal(
        temper_forge_setup::Steps { always: Box::new([long(b"serve", &[b"never"])]), asked: Box::new([]) },
        &[],
        true,
        16_384,
        &crate::limits::small(),
        crate::host::Echo::None,
        super::fixtures::Opening {
            parent: None,
            link_form: crate::terminal::LinkForm::Bare,
            log_lines: Box::new([]),
            front: None,
            first_link: None,
            after_block: bytes::copy_of(AFTER_BLOCK),
        },
    );
    h.run_until_idle();
    assert_eq!(h.stderr().as_ref(), AFTER_BLOCK);
    assert!(h.stdout().is_empty());
    assert_eq!(stdin_pipes(h.launches.get(0).unwrap()), 0);
    h.write_child(h.child_index(b"serve"), 1, b"http://first\n");
    h.run_until_idle();
    assert_eq!(h.stdout().as_ref(), b"http://first\n");
    assert_eq!(
        h.stderr(),
        crate::lines::joined(&[
            crate::terminal::PRELUDE,
            crate::terminal::FOOTER,
            b"In a terminal: temper http://first\n"
        ])
    );
    h.write_child(h.child_index(b"serve"), 1, b"http://later\n");
    h.run_until_idle();
    assert_eq!(h.stdout().as_ref(), b"http://later\n");
    assert_eq!(h.stderr().as_ref(), b"\n");
    stop_first_link(&mut h);
}

#[test]
fn the_terminal_keeps_errors_alone_and_box_log_the_rest() {
    for echo in [crate::host::Echo::None, crate::host::Echo::Steps] {
        let mut h = Harness::configured_echo(grouped_steps(), &[], true, 16_384, &crate::limits::small(), echo);
        h.wall = skein_lib::Wall::from_nanos(3_723_000_000_000);
        h.run_until_idle();
        h.ask(b"start 3\n");
        h.run_until_idle();
        h.ask(b"stop 3\n");
        h.run_until_idle();
        let forge = h.hosted.get(h.child_index(b"forgejo 3")).unwrap().pid;
        h.sim.finish_service(forge, Exit::Code(0));
        h.run_until_idle();
        h.ask(b"unknown\n");
        h.run_until_idle();
        h.terminate();
        assert_eq!(h.run_to_end(), Exit::Code(0));
        assert_eq!(h.stdout().as_ref(), b"");
        assert_eq!(h.stderr().as_ref(), b"01:02:03Z box: serve: unknown pipe line\n");
        assert_eq!(
            h.log().as_ref(),
            concat!(
                "01:02:03Z Logs: /data/box/logs/\n",
                "01:02:03Z Ctrl-C stops temper.\n",
                "01:02:03Z Starting serve\n",
                "01:02:03Z Forge 3 (Forgejo): starting\n",
                "01:02:03Z Forge 3 (Forgejo): started\n",
                "01:02:03Z Forge 3 (Forgejo): stopped\n",
                "01:02:03Z Stopping\n01:02:03Z Stopped\n",
            )
            .as_bytes()
        );
    }
}

#[test]
fn box_log_ends_with_stopped() {
    let mut h = Harness::with_stubborn(&[long(b"serve", &[b"never"])], true);
    h.run_until_idle();
    h.hold_log_write = true;
    h.terminate();
    h.run_until_idle();
    assert!(h.held_log_write.is_some(), "shutdown waits on its log write");
    assert!(!h.log().ends_with(b"Stopped\n"));
    h.release_log_write();
    assert_eq!(h.run_to_end(), Exit::Code(0));
    assert!(h.log().ends_with(b"00:00:00Z Stopped\n"));
    h.sim.assert_quiescent(h.pid);
    h.sim.assert_no_open_fds(h.pid);
    assert_eq!(h.machine.open_handles(), 0);
}

#[test]
fn a_blocked_terminal_holds_no_log_line_and_no_spawn() {
    let mut h = Harness::with_stubborn(&[long(b"serve", &[b"never"])], true);
    h.hold_terminal_ack = true;
    crate::host::pressure::fill_terminal(&mut h.service);
    h.run_until_idle();
    assert!(!h.held_terminal_ack.is_empty(), "terminal write stays held");
    assert!(Harness::contains(&h.log(), b"00:00:00Z Starting serve\n"));
    assert_eq!(h.spawn_times(b"serve").len(), 1, "log placement lets the step spawn");
    h.release_terminal_ack();
    h.terminate();
    assert_eq!(h.run_to_end(), Exit::Code(0));
}

#[test]
fn an_unwritable_box_log_is_named_once_and_its_lines_follow() {
    for create in [true, false] {
        let mut h = Harness::with_stubborn(&[long(b"serve", &[b"never"])], true);
        if create {
            h.refuse_log = Some(bytes::copy_of(b"box.log"));
        } else {
            h.hold_log_write = true;
        }
        h.wall = skein_lib::Wall::from_nanos(3_723_000_000_000);
        h.run_until_idle();
        if !create {
            assert!(h.held_log_write.is_some(), "first box line waits on its write");
            h.fail_log_write();
            h.run_until_idle();
        }
        h.terminate();
        assert_eq!(h.run_to_end(), Exit::Code(0));
        assert_eq!(h.stdout().as_ref(), b"");
        assert_eq!(
            h.stderr().as_ref(),
            concat!(
                "01:02:03Z box: cannot write /data/box/logs/box.log: NoSpace; the step's lines follow here\n",
                "01:02:03Z Logs: /data/box/logs/\n",
                "01:02:03Z Starting serve\n",
                "01:02:03Z Stopping\n01:02:03Z Stopped\n",
            )
            .as_bytes()
        );
        h.sim.assert_no_open_fds(h.pid);
        assert_eq!(h.machine.open_handles(), 0);
    }
}

#[test]
fn the_steps_start_in_order_and_a_stop_leaves_no_child() {
    let steps = [
        once(b"prepare", &[b"exit", b"0"]),
        long(b"forgejo", &[b"never"]),
        long(b"runner", &[b"never"]),
        once(b"once-1", &[b"exit", b"0"]),
        once(b"once-2", &[b"exit", b"0"]),
        long(b"serve", &[b"never"]),
    ];
    let mut harness = Harness::new(&steps);
    harness.run_until_idle();
    let mut names = List::with_capacity(6);
    for launch in &harness.launches {
        names.push(launch.name.clone()).expect("six launches");
    }
    let expected: Box<[Box<[u8]>]> = Box::new([
        bytes::copy_of(b"prepare"),
        bytes::copy_of(b"forgejo"),
        bytes::copy_of(b"runner"),
        bytes::copy_of(b"once-1"),
        bytes::copy_of(b"once-2"),
        bytes::copy_of(b"serve"),
    ]);
    assert_eq!(names.as_slice(), expected.as_ref(), "the setup's step order");
    harness.terminate();
    assert_eq!(harness.run_to_end(), Exit::Code(0), "a requested stop succeeds");
    harness.sim.assert_quiescent(harness.pid);
    harness.sim.assert_no_open_fds(harness.pid);
}

#[test]
fn an_owned_version_probe_stops_or_times_out_without_starting_the_runner() {
    for stop in [false, true] {
        let steps =
            [long(b"forgejo", &[b"never"]).with_ready(([127, 0, 0, 1], 3000).into()), long(b"runner", &[b"never"])];
        let mut harness = Harness::new(&steps);
        harness.run_until_idle();
        assert_eq!(harness.spawn_times(b"forgejo").len(), 1);
        assert_eq!(harness.spawn_times(b"runner").len(), 0, "no version endpoint answers");
        if stop {
            harness.terminate();
        }
        assert_eq!(harness.run_to_end(), Exit::Code(u8::from(!stop)));
        assert_eq!(harness.spawn_times(b"runner").len(), 0, "the barrier never passed");
        harness.sim.assert_quiescent(harness.pid);
        harness.sim.assert_no_open_fds(harness.pid);
    }
}
#[test]
fn a_stop_signals_each_long_step_twice_last_first() {
    let steps = [long(b"forgejo", &[b"never"]), long(b"runner", &[b"never"]), long(b"serve", &[b"never"])];
    let mut harness = Harness::with_stubborn(&steps, true);
    harness.run_until_idle();
    harness.terminate();
    assert_eq!(harness.run_to_end(), Exit::Code(0), "a stop ultimately closes stubborn children");
    assert_stop_order(&harness);
    harness.sim.assert_quiescent(harness.pid);
    harness.sim.assert_no_open_fds(harness.pid);
}
fn assert_stop_order(harness: &Harness) {
    let mut children = List::with_capacity(3);
    let mut signals = List::with_capacity(6);
    for entry in harness.sim.trace() {
        if entry.pid == harness.pid {
            if let Event::Complete { result: Ok(Done::Spawned { pidfd, .. }), .. } = &entry.event {
                children.push(*pidfd).expect("three children");
            }
            if let Event::Submit { kind: Summary::Signal { pidfd, signal: 15, to: Target::Child }, .. } = &entry.event {
                signals.push((*pidfd, entry.at)).expect("two termination signals per child");
            }
        }
    }
    assert_eq!(signals.len(), 6, "each stubborn child is signalled twice");
    for (index, child) in children.iter().rev().enumerate() {
        let start = u32::try_from(index).expect("three children").saturating_mul(2);
        let first = signals.get(start).expect("first signal");
        let second = signals.get(start.saturating_add(1)).expect("second signal");
        assert_eq!((first.0, second.0), (*child, *child), "last child first");
        assert!(second.1.saturating_since(first.1) >= Duration::from_secs(10), "a grace between signals");
    }
}
fn parent_harness(parent: Option<u32>) -> Harness {
    Harness::with_terminal(
        temper_forge_setup::Steps {
            always: Box::new([once(b"prepare", &[b"exit", b"0"]), long(b"serve", &[b"never"])]),
            asked: Box::new([]),
        },
        &[],
        true,
        16_384,
        &crate::limits::small(),
        crate::host::Echo::None,
        super::fixtures::Opening {
            parent,
            link_form: crate::terminal::LinkForm::Bare,
            log_lines: Box::new([]),
            front: None,
            first_link: None,
            after_block: Box::new([]),
        },
    )
}
#[test]
fn each_step_starts_through_forge_child_with_the_boxs_id() {
    let mut h = parent_harness(Some(4321));
    h.run_until_idle();
    assert_eq!(h.launches.len(), 2);
    for (launch, arguments) in h
        .launches
        .iter()
        .zip([[b"exit".as_slice(), b"0", b"prepare"].as_slice(), [b"never".as_slice(), b"serve"].as_slice()])
    {
        let mut expected = List::with_capacity(11);
        for argument in [
            b"internal".as_slice(),
            b"forge-child",
            b"--parent",
            b"4321",
            b"--signal",
            b"terminate",
            b"--",
            b"process_fixture",
        ]
        .iter()
        .chain(arguments.iter())
        {
            expected.push(bytes::copy_of(argument)).expect("wrapper and step arguments");
        }
        assert_eq!(
            launch.args.as_ref(),
            expected.as_slice(),
            "each once and long step execs with the box's parent-death signal"
        );
    }
    h.terminate();
    assert_eq!(h.run_to_end(), Exit::Code(0));
    h.sim.assert_no_open_fds(h.pid);
}
#[test]
fn with_no_parent_a_step_starts_directly() {
    let mut h = parent_harness(None);
    h.run_until_idle();
    for (launch, expected) in h
        .launches
        .iter()
        .zip([[b"exit".as_slice(), b"0", b"prepare"].as_slice(), [b"never".as_slice(), b"serve"].as_slice()])
    {
        let mut args = List::with_capacity(3);
        for argument in expected {
            args.push(bytes::copy_of(argument)).expect("step arguments");
        }
        assert_eq!(launch.args.as_ref(), args.as_slice(), "no wrapper without a parent");
    }
    assert_eq!(h.launches.len(), 2);
    h.terminate();
    assert_eq!(h.run_to_end(), Exit::Code(0));
    h.sim.assert_no_open_fds(h.pid);
}
fn hangup(h: &mut Harness) {
    h.sim.deliver_service_signal(h.pid, h.signals, skein_io::kernel::ServiceSignal::Hangup, None);
}
fn stopping_harness() -> Harness {
    let mut h = Harness::with_stubborn(
        &[long(b"forgejo", &[b"never"]), long(b"runner", &[b"never"]), long(b"serve", &[b"never"])],
        true,
    );
    h.run_until_idle();
    h
}
#[test]
fn a_hangup_stops_in_the_usual_order_and_a_second_changes_nothing() {
    let mut h = stopping_harness();
    hangup(&mut h);
    h.run_until_idle();
    let deadline = skein_shell::Host::next_deadline(&h.service);
    assert!(deadline.is_some(), "hangup starts the first child's grace");
    hangup(&mut h);
    h.run_until_idle();
    assert_eq!(skein_shell::Host::next_deadline(&h.service), deadline, "another hangup keeps the grace and order");
    assert_eq!(h.run_to_end(), Exit::Code(0));
    assert_stop_order(&h);
    let log = h.log();
    assert_eq!(bytes::count(&log, b"Stopping\n", 2), 1);
    assert!(log.ends_with(b"Stopped\n"));
    h.sim.assert_quiescent(h.pid);
    h.sim.assert_no_open_fds(h.pid);
}
#[test]
fn a_hangup_during_a_termination_changes_nothing() {
    let mut h = stopping_harness();
    h.terminate();
    h.run_until_idle();
    let deadline = skein_shell::Host::next_deadline(&h.service);
    hangup(&mut h);
    h.run_until_idle();
    assert_eq!(
        skein_shell::Host::next_deadline(&h.service),
        deadline,
        "hangup cannot escalate an existing termination"
    );
    assert_eq!(h.run_to_end(), Exit::Code(0));
    assert_stop_order(&h);
    h.sim.assert_no_open_fds(h.pid);
}
#[test]
fn a_termination_after_a_hangup_closes_the_steps() {
    for signal in [skein_io::kernel::ServiceSignal::Interrupt, skein_io::kernel::ServiceSignal::Terminate] {
        let mut h = stopping_harness();
        hangup(&mut h);
        h.run_until_idle();
        let now = h.sim.now();
        h.sim.deliver_service_signal(h.pid, h.signals, signal, None);
        h.run_until_idle();
        assert_eq!(h.run_to_end(), Exit::Code(0));
        assert_eq!(h.sim.now(), now, "explicit second termination closes without waiting for grace");
        h.sim.assert_quiescent(h.pid);
        h.sim.assert_no_open_fds(h.pid);
    }
}
#[test]
fn a_long_step_that_keeps_exiting_is_given_up_naming_it() {
    let steps = [once(b"prepare", &[b"exit", b"0"]), long(b"forgejo", &[b"exit", b"1"])];
    let mut harness = Harness::new(&steps);
    assert_eq!(harness.run_to_end(), Exit::Code(1));
    let spawns = harness.spawn_times(b"forgejo");
    assert_eq!(spawns.len(), 6, "the first run and five restarts");
    for (pair, gap) in spawns.windows(2).zip([500, 1_000, 2_000, 4_000, 8_000]) {
        assert!(pair[1].saturating_since(pair[0]) >= Duration::from_millis(gap), "the backoff doubles");
    }
    assert_eq!(harness.spawn_times(b"prepare").len(), 6, "each restart runs prepare again");
    assert!(Harness::contains(&harness.stderr(), b"forgejo exited 1; giving up\n"), "give-up names step");
}
#[test]
fn a_once_step_that_fails_ends_the_box_with_exit_one() {
    let steps = [once(b"prepare", &[b"exit", b"1"]), long(b"forgejo", &[b"never"])];
    let mut harness = Harness::new(&steps);
    assert_eq!(harness.run_to_end(), Exit::Code(1));
    assert_eq!(&*harness.stderr(), b"00:00:00Z prepare exited 1; giving up\n");
    assert_eq!(harness.spawn_times(b"forgejo").len(), 0, "failure never starts the next step");
}
#[test]
fn every_step_keeps_the_environment_except_inherited_activation_variables() {
    let steps = [once(b"prepare", &[b"exit", b"0"]), long(b"serve", &[b"never"])];
    let mut harness = Harness::new(&steps);
    harness.run_until_idle();
    let expected: Box<[Box<[u8]>]> = Box::new([bytes::copy_of(b"HOME=/box"), bytes::copy_of(b"TOKEN=whole=value")]);
    assert_eq!(harness.launches.len(), 2, "both steps ran");
    for launch in &harness.launches {
        assert_eq!(launch.environment, expected, "activation variables are removed and other entries stay whole");
    }
    harness.terminate();
    assert_eq!(harness.run_to_end(), Exit::Code(0));
}
#[test]
fn the_io_limits_are_usable_for_one_six_and_sixty_four_steps() {
    for count in [1_u32, 6, 64] {
        let limits = io_limits(count);
        assert!(limits.is_usable(), "usable for {count} steps");
        assert_eq!(limits.sockets, count.saturating_mul(5).saturating_add(5));
        assert_eq!(limits.intake, limits.receive, limits.names), (LINE_BYTES, 1, 0);
        assert_eq!((limits.sends, limits.accepts, limits.backlog), (1, 1, 1));
        assert_eq!(limits.close_timeout, Duration::from_secs(5));
        assert_eq!(limits.retry, Duration::from_millis(10));
        let mut steps = List::with_capacity(count);
        steps.push(long(b"serve", &[b"never"])).unwrap();
        if count > 1 {
            steps.push(super::fixtures::link(b"seed", &[b"exit", b"0"])).unwrap();
        }
        for _ in steps.len()..count {
            steps.push(long(b"worker", &[b"never"])).unwrap();
        }
        let mut harness = Harness::with_stubborn(steps.as_slice(), true);
        harness.run_until_idle();
        if count > 1 {
            harness.write_child(harness.serve_index(), 1, b"link\n");
            harness.run_until_idle();
        }
        assert_eq!(harness.launches.len(), count);
        harness.terminate();
        assert_eq!(harness.run_to_end(), Exit::Code(0));
        harness.sim.assert_quiescent(harness.pid);
        harness.sim.assert_no_open_fds(harness.pid);
    }
}

#[test]
fn serves_pipe_lines_are_bounded_named_and_answered() {
    let mut too_long = List::with_capacity(257);
    for _ in 0_u32..256 {
        too_long.push(b'x').unwrap();
    }
    too_long.push(b'\n').unwrap();
    for (name, input, expected, diagnostic) in [
        ("ask", b"start 3\n".as_slice(), b"step 3 prepare\nstep 3 run\nstarted 3\n".as_slice(), b"".as_slice()),
        ("unknown forge", b"start 99\n", b"failed 99 not asked\n", b""),
        ("unknown line", b"secret unknown\n", b"", b"box: serve: unknown pipe line\n"),
        ("too long", too_long.as_slice(), b"", b"box: serve: pipe line too long\n"),
    ] {
        let mut harness = Harness::grouped(grouped_steps(), &[], true);
        harness.run_until_idle();
        assert_eq!(harness.launches.len(), 1, "{name}: dormant until asked");
        assert_eq!(
            harness.launches.get(0).unwrap().pipes.as_ref(),
            &[
                skein_io::kernel::Pipe { child: 1, way: skein_io::kernel::Way::Out, parent: None },
                skein_io::kernel::Pipe { child: 2, way: skein_io::kernel::Way::Out, parent: None },
                skein_io::kernel::Pipe { child: 3, way: skein_io::kernel::Way::Out, parent: None },
                skein_io::kernel::Pipe { child: 4, way: skein_io::kernel::Way::In, parent: None },
            ]
        );
        harness.ask(input);
        harness.run_until_idle();
        assert_eq!(&*harness.reply_bytes(), expected, "{name}");
        let timed = if diagnostic.is_empty() {
            bytes::copy_of(b"")
        } else {
            crate::lines::joined(&[b"00:00:00Z ", diagnostic])
        };
        assert_eq!(harness.stderr(), timed, "{name}");
        if name == "ask" {
            assert!(Harness::contains(
                &harness.log(),
                b"00:00:00Z Forge 3 (Forgejo): starting\n00:00:00Z Forge 3 (Forgejo): started\n"
            ));
        }
        if !diagnostic.is_empty() {
            harness.ask(b"start 99\n");
            harness.run_until_idle();
            assert_eq!(&*harness.reply_bytes(), b"failed 99 not asked\n", "{name}: next line recovered");
        }
        harness.terminate();
        assert_eq!(harness.run_to_end(), Exit::Code(0));
        harness.sim.assert_quiescent(harness.pid);
        harness.sim.assert_no_open_fds(harness.pid);
    }
}
#[test]
fn records_are_probed_before_start_and_written_private_or_reported() {
    use skein_fake_machine::{How, Item};
    let present =
        [Item::directory(b"box"), Item::directory(b"box/started"), Item::file(b"box/started/3", b"").mode(0o600)];
    let unreadable =
        [Item::directory(b"box"), Item::directory(b"box/started"), Item::file(b"box/started/3", b"").mode(0)];
    let blocked = [Item::file(b"box", b"kept")];
    let denied = [Item::directory(b"box"), Item::directory(b"box/started").mode(0o500)];
    for (name, items, recorded, writable, why) in [
        ("absent", &[][..], false, true, b"".as_slice()),
        ("present", present.as_slice(), true, true, b""),
        ("unreadable", unreadable.as_slice(), false, true, b"permission denied"),
        ("directory blocked", blocked.as_slice(), false, false, b"not a directory"),
        ("create denied", denied.as_slice(), false, false, b""),
    ] {
        let mut harness = Harness::grouped(grouped_steps(), items, true);
        harness.run_until_idle();
        assert_eq!(harness.spawn_times(b"prepare 3").len(), usize::from(recorded), "{name}: startup record");
        let diagnostics = harness.stderr();
        if !why.is_empty() {
            assert!(Harness::contains(&diagnostics, b"box/started/3"), "{name}");
            assert!(Harness::contains(&diagnostics, why), "{name}");
        }
        harness.ask(b"start 3\n");
        harness.run_until_idle();
        let replies = harness.reply_bytes();
        if recorded {
            assert_eq!(&*replies, b"started 3\n");
        } else if writable {
            assert_eq!(&*replies, b"step 3 prepare\nstep 3 run\nstarted 3\n");
        } else {
            assert_eq!(&*replies, b"step 3 prepare\nstep 3 run\nfailed 3 record not written\n");
            assert!(Harness::contains(&harness.stderr(), b"box/started/3"));
        }
        if writable && name != "unreadable" {
            let root = harness.machine.reopen_root(0);
            let file = harness.machine.open(root, b"box/started/3", How::Read).unwrap();
            assert_eq!(harness.machine.stat(file).mode & 0o777, 0o600, "{name}: private record");
            assert_eq!(harness.machine.read(file, 0, 1).unwrap().len(), 0);
            harness.machine.close(file);
            harness.machine.close(root);
        }
        harness.terminate();
        assert_eq!(harness.run_to_end(), Exit::Code(0));
        harness.sim.assert_quiescent(harness.pid);
        harness.sim.assert_no_open_fds(harness.pid);
        assert_eq!(harness.machine.open_handles(), 0, "{name}: every opened record closed");
    }
}
#[test]
fn an_exited_serve_loses_its_answers_and_gets_fresh_pipes() {
    let mut harness = Harness::grouped(grouped_steps(), &[], true);
    harness.run_until_idle();
    let old = harness.serve_index();
    let child = harness.hosted.get(old).unwrap();
    let old_output = child.output.unwrap();
    let old_input = child.input.unwrap();
    let pid = child.pid;
    harness.hold_spawn = Some(bytes::copy_of(b"prepare 3"));
    harness.ask(b"start 3\n");
    harness.run_until_idle();
    assert!(harness.held_spawn.is_some());
    harness.sim.finish_service(pid, Exit::Code(1));
    harness.run_until_idle();
    harness.replies.clear();
    harness.release_spawn();
    harness.run_until_idle();
    assert!(harness.replies.is_empty(), "answers to the exited serve are dropped");
    harness.sim.advance_to(harness.sim.now().saturating_add(Duration::from_millis(500)));
    harness.run_until_idle();
    assert_eq!(harness.spawn_times(b"serve").len(), 2);
    let fresh = harness.hosted.get(harness.serve_index()).unwrap();
    assert_ne!(fresh.pid, pid);
    assert!(fresh.output.is_some() && fresh.input.is_some());
    assert_ne!(harness.control_fds.get(0), harness.control_fds.get(1));
    assert!(old_output.raw() > 0_i32 && old_input.raw() > 0_i32);
    assert!(harness.replies.is_empty(), "old pending answers were dropped");
    harness.ask(b"start 3\n");
    harness.run_until_idle();
    assert_eq!(&*harness.reply_bytes(), b"started 3\n");
    harness.terminate();
    assert_eq!(harness.run_to_end(), Exit::Code(0));
    harness.sim.assert_quiescent(harness.pid);
    harness.sim.assert_no_open_fds(harness.pid);
}

#[test]
fn a_serve_restarts_when_its_requests_pipe_closes_before_its_exit() {
    let mut harness = Harness::with_stubborn(&[long(b"serve", &[b"never"])], true);
    harness.run_until_idle();
    let pid = harness.hosted.get(harness.serve_index()).unwrap().pid;
    harness.close_requests();
    harness.run_until_idle();
    harness.sim.finish_service(pid, Exit::Signal(6));
    harness.run_until_idle();
    harness.sim.advance_to(harness.sim.now().saturating_add(Duration::from_millis(500)));
    harness.run_until_idle();
    assert_eq!(
        harness.spawn_times(b"serve").len(),
        2,
        "the closed old requests pipe cannot hold a fresh spawn: {:?}",
        crate::host::pressure::restart_gate(&harness.service)
    );
    harness.terminate();
    assert_eq!(harness.run_to_end(), Exit::Code(0));
    harness.sim.assert_quiescent(harness.pid);
    harness.sim.assert_no_open_fds(harness.pid);
}

#[test]
fn a_serve_that_does_not_read_holds_the_next_ask_until_replies_are_written() {
    for buffer in [8, 16] {
        let mut harness = Harness::buffered(grouped_steps(), &[], true, buffer);
        harness.run_until_idle();
        harness.ask(b"start 3\n");
        harness.run_until_idle();
        harness.ask(b"start 4\n");
        harness.run_until_idle();
        assert!(!Harness::contains(&harness.reply_bytes(), b"failed 4"), "buffer {buffer}: next ask waits");
        let expected = b"step 3 prepare\nstep 3 run\nstarted 3\nfailed 4 not asked\n";
        let mut received = List::with_capacity(u32::try_from(expected.len()).unwrap());
        for _ in 0..expected.len() {
            if usize::try_from(received.len()).unwrap() == expected.len() {
                break;
            }
            let remaining = u32::try_from(expected.len()).unwrap().saturating_sub(received.len());
            for byte in harness.read_reply(remaining) {
                received.push(byte).unwrap();
            }
            harness.run_until_idle();
        }
        assert_eq!(received.as_slice(), expected, "buffer {buffer}: replies stay in order");
        harness.terminate();
        assert_eq!(harness.run_to_end(), Exit::Code(0));
        harness.sim.assert_quiescent(harness.pid);
        harness.sim.assert_no_open_fds(harness.pid);
    }
}

#[test]
#[expect(clippy::too_many_lines, reason = "the link's byte and lifecycle cases share the hosted-child table")]
fn a_link_receives_only_the_first_line_and_its_end() {
    use super::fixtures::link;
    use skein_shell::Host;
    enum Action {
        Write(&'static [u8], usize, Box<[u8]>),
        Read(Box<[u8]>),
        Exit(&'static [u8]),
        WriteExit(&'static [u8], Box<[u8]>),
        BrokenPipe,
        Advance,
        Stdout(Box<[u8]>),
        Stderr(&'static [u8]),
        Spawns(&'static [u8], usize),
    }
    type Case<'a> = (&'a str, &'a [&'a [u8]], Duration, Box<[Action]>);
    use Action::{Advance, Exit as End, Read, Spawns, Stderr, Stdout, Write};
    fn write(name: &'static [u8], fd: usize, data: &[u8]) -> Action {
        Write(name, fd, bytes::copy_of(data))
    }
    fn read(data: &[u8]) -> Action {
        Read(bytes::copy_of(data))
    }
    fn stdout(data: &[u8]) -> Action {
        Stdout(bytes::copy_of(data))
    }
    let mut full = List::with_capacity(LINE_BYTES);
    for _ in 0..LINE_BYTES {
        full.push(b'x').unwrap();
    }
    let full = full.into_boxed();
    let past = crate::lines::joined(&[full.as_ref(), b"tail\nlater\n"]);
    let cases: [Case<'_>; 9] = [
        (
            "whole",
            &[b"never".as_slice()][..],
            Duration::from_millis(500),
            Box::new([write(b"serve", 1, b"link\nlater\n"), read(b"link\n"), read(b""), stdout(b"later\n")]),
        ),
        (
            "two chunks",
            &[b"never".as_slice()][..],
            Duration::from_millis(500),
            Box::new([
                write(b"serve", 1, b"li"),
                Spawns(b"seed", 0),
                stdout(b""),
                write(b"serve", 1, b"nk\nlater\n"),
                read(b"link\n"),
                read(b""),
                stdout(b"later\n"),
            ]),
        ),
        (
            "at cap",
            &[b"never".as_slice()][..],
            Duration::from_millis(500),
            Box::new([write(b"serve", 1, &full), read(&full), read(b""), stdout(b"")]),
        ),
        (
            "past cap",
            &[b"never".as_slice()][..],
            Duration::from_millis(500),
            Box::new([write(b"serve", 1, &past), read(&full), read(b""), stdout(b"tail\nlater\n")]),
        ),
        (
            "incomplete exit",
            &[b"never".as_slice()][..],
            Duration::from_millis(500),
            Box::new([
                write(b"serve", 1, b"half"),
                End(b"serve"),
                stdout(b"half"),
                Spawns(b"seed", 0),
                Advance,
                write(b"serve", 1, b"restart\nlater\n"),
                read(b"restart\n"),
                read(b""),
                stdout(b"later\n"),
            ]),
        ),
        (
            "seed exits unread",
            &[b"exit".as_slice(), b"0"][..],
            Duration::from_millis(500),
            Box::new([
                write(b"serve", 1, b"link\nlater\n"),
                Spawns(b"seed", 1),
                Action::BrokenPipe,
                stdout(b"later\n"),
            ]),
        ),
        (
            "seed output",
            &[b"never".as_slice()][..],
            Duration::from_millis(500),
            Box::new([
                write(b"serve", 1, b"link\nlater\n"),
                read(b"link\n"),
                read(b""),
                write(b"seed", 1, b"out\n"),
                write(b"seed", 2, b"err\n"),
                Stderr(b"seed: out\nseed: err\n"),
                stdout(b"later\n"),
            ]),
        ),
        (
            "restart after seed",
            &[b"never".as_slice()][..],
            Duration::from_millis(500),
            Box::new([
                write(b"serve", 1, b"link\n"),
                read(b"link\n"),
                read(b""),
                End(b"serve"),
                Advance,
                write(b"serve", 1, b"fresh\nlater\n"),
                Spawns(b"seed", 1),
                stdout(b"fresh\nlater\n"),
            ]),
        ),
        (
            "line with immediate restart",
            &[b"never".as_slice()][..],
            Duration::from_millis(0),
            Box::new([
                Action::WriteExit(b"serve", bytes::copy_of(b"link\n")),
                read(b"link\n"),
                read(b""),
                Spawns(b"serve", 2),
                Spawns(b"seed", 1),
                stdout(b""),
            ]),
        ),
    ];
    for (name, seed_args, backoff, actions) in cases {
        let steps = temper_forge_setup::Steps {
            always: Box::new([long(b"serve", &[b"never"]), link(b"seed", seed_args)]),
            asked: Box::new([]),
        };
        let limits = crate::limits::Limits { backoff, ..crate::limits::small() };
        let mut harness = Harness::configured(steps, &[], true, skein_sim::Config::calm().buffer, &limits);
        harness.run_until_idle();
        for action in actions {
            match action {
                Write(child, fd, data) => harness.write_child(harness.child_index(child), fd, &data),
                Read(expected) => {
                    let seed = harness.child_index(b"seed");
                    let mut received = List::with_capacity(LINE_BYTES);
                    // A pipe may return a short read; read again until the expected line is whole.
                    for _ in 0..expected.len().max(1) {
                        let most = u32::try_from(
                            expected.len().saturating_sub(usize::try_from(received.len()).unwrap()).max(1),
                        )
                        .unwrap();
                        let piece = harness.read_stdin(seed, most);
                        for byte in &piece {
                            received.push(*byte).unwrap();
                        }
                        if piece.is_empty() || usize::try_from(received.len()).unwrap() == expected.len() {
                            break;
                        }
                    }
                    assert_eq!(received.as_slice(), expected.as_ref(), "{name}: stdin");
                }
                End(child) => {
                    let pid = harness.hosted.get(harness.child_index(child)).unwrap().pid;
                    harness.sim.finish_service(pid, Exit::Code(0));
                }
                Action::WriteExit(child, data) => {
                    let index = harness.child_index(child);
                    harness.write_child(index, 1, &data);
                    let pid = harness.hosted.get(index).unwrap().pid;
                    harness.sim.finish_service(pid, Exit::Code(0));
                }
                Action::BrokenPipe => {
                    let mut found = false;
                    for entry in harness.sim.trace() {
                        if let Event::Complete { result: Err(skein_io::kernel::Error::BrokenPipe), .. } = &entry.event {
                            found = true;
                        }
                    }
                    assert!(found, "{name}: the stdin write failed");
                }
                Advance => harness.sim.advance_to(harness.sim.now().saturating_add(Duration::from_millis(500))),
                Stdout(expected) => assert_eq!(harness.stdout().as_ref(), expected.as_ref(), "{name}: stdout"),
                Stderr(expected) => assert!(
                    Harness::contains(&harness.stderr(), expected),
                    "{name}: child stderr remains visible alongside lifecycle and block notices"
                ),
                Spawns(child, expected) => assert_eq!(harness.spawn_times(child).len(), expected, "{name}: launches"),
            }
            harness.run_until_idle();
        }
        assert_eq!(harness.service.exit(), None, "{name}: link stdin alone can fail");
        harness.terminate();
        assert_eq!(harness.run_to_end(), Exit::Code(0), "{name}: stop");
        harness.sim.assert_quiescent(harness.pid);
        harness.sim.assert_no_open_fds(harness.pid);
    }
}

#[test]
fn an_unrecorded_group_neither_probes_nor_writes_and_waits_at_every_box_start() {
    use skein_fake_machine::{How, Item};
    // A non-directory would expose either an accidental probe or a write.
    for _restart in 0_u8..2 {
        let mut steps = grouped_steps();
        steps.asked[0].recorded = false;
        let mut harness = Harness::grouped(steps, &[Item::file(b"box", b"kept")], true);
        harness.run_until_idle();
        assert_eq!(harness.launches.len(), 1, "only serve starts");
        assert_eq!(
            harness.stderr().as_ref(),
            b"00:00:00Z box: cannot write /data/box/logs/: NotADirectory; the steps' lines follow here\n00:00:00Z Logs: /data/box/logs/\n00:00:00Z Starting serve\n",
            "no probe of box/started/3"
        );
        harness.ask(b"start 3\n");
        harness.run_until_idle();
        assert_eq!(&*harness.reply_bytes(), b"step 3 prepare\nstep 3 run\nstarted 3\n");
        assert_eq!(
            harness.stderr().as_ref(),
            b"00:00:00Z Forge 3 (Forgejo): starting\n00:00:00Z Forge 3 (Forgejo): started\n",
            "unrecorded group emits its lifecycle without a record write"
        );
        let root = harness.machine.reopen_root(0);
        let file = harness.machine.open(root, b"box", How::Read).unwrap();
        assert_eq!(harness.machine.read(file, 0, 4).unwrap().as_slice(), b"kept");
        harness.machine.close(file);
        harness.machine.close(root);
        harness.terminate();
        assert_eq!(harness.run_to_end(), Exit::Code(0));
        harness.sim.assert_quiescent(harness.pid);
        harness.sim.assert_no_open_fds(harness.pid);
        assert_eq!(harness.machine.open_handles(), 0);
    }
}

#[test]
fn a_pipe_stop_closes_only_its_group_keeps_the_record_and_can_be_started_again() {
    use skein_fake_machine::How;
    let mut harness = Harness::grouped(grouped_steps(), &[], true);
    harness.run_until_idle();
    harness.ask(b"start 3\n");
    harness.run_until_idle();
    assert_eq!(&*harness.reply_bytes(), b"step 3 prepare\nstep 3 run\nstarted 3\n");
    harness.replies.clear();
    harness.ask(b"stop 3\n");
    harness.run_until_idle();
    assert!(harness.reply_bytes().is_empty(), "stubborn group has not closed yet");
    for _ in 0_u32..2 {
        harness.sim.advance_to(harness.sim.now().saturating_add(Duration::from_secs(10)));
        harness.run_until_idle();
    }
    assert_eq!(&*harness.reply_bytes(), b"stopped 3\n");
    assert!(Harness::contains(&harness.log(), b"Forge 3 (Forgejo): stopped\n"));
    assert_eq!(harness.spawn_times(b"serve").len(), 1);
    assert_eq!(harness.spawn_times(b"forgejo 3").len(), 1);
    let root = harness.machine.reopen_root(0);
    let record = harness.machine.open(root, b"box/started/3", How::Read).expect("stop keeps the record");
    harness.machine.close(record);
    harness.machine.close(root);
    harness.replies.clear();
    harness.ask(b"start 3\n");
    harness.run_until_idle();
    assert_eq!(&*harness.reply_bytes(), b"step 3 prepare\nstep 3 run\nstarted 3\n");
    assert_eq!(harness.spawn_times(b"forgejo 3").len(), 2);
    harness.terminate();
    assert_eq!(harness.run_to_end(), Exit::Code(0));
    harness.sim.assert_quiescent(harness.pid);
    harness.sim.assert_no_open_fds(harness.pid);
    assert_eq!(harness.machine.open_handles(), 0);
}

#[test]
fn a_group_stop_is_logged_after_the_control_pipe_closes() {
    let mut steps = grouped_steps();
    steps.asked.get_mut(0).unwrap().forge = 1;
    let mut harness = Harness::grouped(steps, &[], true);
    harness.run_until_idle();
    harness.ask(b"start 1\n");
    harness.run_until_idle();
    harness.replies.clear();
    harness.ask(b"stop 1\n");
    harness.run_until_idle();
    assert_eq!(
        harness.log().as_ref(),
        b"00:00:00Z Logs: /data/box/logs/\n00:00:00Z Ctrl-C stops temper.\n00:00:00Z Starting serve\n00:00:00Z Forge 3 (Forgejo): starting\n00:00:00Z Forge 3 (Forgejo): started\n",
        "the group has not closed"
    );
    let serve = harness.hosted.get(harness.serve_index()).unwrap().pid;
    let forge = harness.hosted.get(harness.child_index(b"forgejo 3")).unwrap().pid;
    harness.sim.finish_service(serve, Exit::Code(1));
    harness.sim.finish_service(forge, Exit::Code(0));
    harness.run_until_idle();
    assert!(harness.reply_bytes().is_empty(), "the old control pipe has closed");
    assert_eq!(&*harness.stderr(), b"00:00:00Z serve exited 1; restarting in 0.5 s\n");
    assert!(harness.log().ends_with(b"00:00:00Z Forge 3 (Forgejo): stopped\n"));
    harness.terminate();
    assert_eq!(harness.run_to_end(), Exit::Code(0));
    harness.sim.assert_quiescent(harness.pid);
    harness.sim.assert_no_open_fds(harness.pid);
}

#[test]
fn each_child_gets_only_its_listeners_and_serve_pipes_follow_them() {
    use skein_io::kernel::Way;
    let forge = ([127, 0, 0, 1], 8100).into();
    let clients = ([127, 0, 0, 1], 8101).into();
    let webhooks = ([127, 0, 0, 1], 8102).into();
    let steps = [
        once(b"prepare", &[b"exit", b"0"]),
        long(b"fake forge", &[b"never"]).listening(Box::new([forge])),
        long(b"serve", &[b"never"]).listening(Box::new([([127, 0, 0, 1], 0).into(), clients, webhooks])),
    ];
    let mut harness = Harness::with_stubborn(&steps, true);
    harness.run_until_idle();
    assert_eq!(harness.launches.len(), 3);
    for (launch, count) in harness.launches.iter().zip([0, 1, 2]) {
        let mut given = 0_u32;
        for pipe in &launch.pipes {
            match pipe.way {
                Way::Given(_) => {
                    assert_eq!(pipe.child, given.checked_add(3).unwrap());
                    given = given.checked_add(1).unwrap();
                }
                Way::In | Way::Out | Way::Null => {}
            }
        }
        assert_eq!(given, count);
        let expected = match count {
            0 => None,
            1 => Some(b"LISTEN_FDS=1".as_slice()),
            _ => Some(b"LISTEN_FDS=2".as_slice()),
        };
        let mut activation = None;
        for entry in &launch.environment {
            if entry.starts_with(b"LISTEN_") {
                assert!(activation.is_none(), "at most one activation variable");
                activation = Some(entry.as_ref());
            }
        }
        assert_eq!(activation, expected);
    }
    let serve = harness.launches.get(2).unwrap();
    assert_eq!((serve.pipes[4].child, serve.pipes[4].way), (5, Way::Out));
    assert_eq!((serve.pipes[5].child, serve.pipes[5].way), (6, Way::In));
    // Receiving a control answer also proves Spawned.pipes still indexes real pipes only.
    harness.ask(b"start 99\n");
    harness.run_until_idle();
    assert_eq!(harness.reply_bytes().as_ref(), b"failed 99 not asked\n");
    harness.terminate();
    assert_eq!(harness.run_to_end(), Exit::Code(0));
    harness.sim.assert_quiescent(harness.pid);
    harness.sim.assert_no_open_fds(harness.pid);
}

#[test]
fn a_document_port_zero_step_is_given_no_listener_or_activation_environment() {
    use skein_io::kernel::Way;
    let steps = [long(b"serve", &[b"never"]).listening(Box::new([([127, 0, 0, 1], 0).into()]))];
    let mut harness = Harness::with_stubborn(&steps, true);
    harness.run_until_idle();
    let launch = harness.launches.get(0).unwrap();
    for pipe in &launch.pipes {
        match pipe.way {
            Way::Given(_) => panic!("port zero is not held or given"),
            Way::In | Way::Out | Way::Null => {}
        }
    }
    for entry in &launch.environment {
        assert!(!entry.starts_with(b"LISTEN_"));
    }
    assert_eq!((launch.pipes[2].child, launch.pipes[3].child), (3, 4));
    assert!(harness.hosted.get(0).unwrap().listeners.is_empty());
    harness.terminate();
    assert_eq!(harness.run_to_end(), Exit::Code(0));
    harness.sim.assert_quiescent(harness.pid);
    harness.sim.assert_no_open_fds(harness.pid);
}

#[test]
fn a_dormant_listener_queues_connections_and_is_given_again_after_restart() {
    use super::fixtures::{call, socket};
    use skein_io::kernel::{Fd, Op, Way};
    let address = ([127, 0, 0, 1], 8200).into();
    let mut steps = grouped_steps();
    steps.asked[0].steps[1].1 = long(b"fake forge 3", &[b"never"]).listening(Box::new([address]));
    let mut harness = Harness::grouped(steps, &[], true);
    harness.run_until_idle();
    assert_eq!(harness.launches.len(), 1, "forge remains dormant");
    let client = harness.sim.spawn_process();
    let connection = socket(&mut harness.sim, client);
    assert_eq!(call(&mut harness.sim, client, Op::Connect { fd: connection, addr: address }).result, Ok(Done::Nothing));
    harness.ask(b"start 3\n");
    harness.run_until_idle();
    let index = harness.child_index(b"fake forge 3");
    let child = harness.hosted.get(index).unwrap();
    let pid = child.pid;
    assert_eq!(child.listeners.as_ref(), &[Fd::new(3)]);
    let accepted = call(&mut harness.sim, pid, Op::Accept { fd: Fd::new(3) });
    let Ok(Done::Accepted { fd, .. }) = accepted.result else { panic!("connection queued before ask is accepted") };
    assert_eq!(call(&mut harness.sim, pid, Op::Close { fd }).result, Ok(Done::Nothing));
    assert_eq!(call(&mut harness.sim, client, Op::Close { fd: connection }).result, Ok(Done::Nothing));
    let mut first = None;
    for launch in &harness.launches {
        if launch.name.as_ref() == b"fake forge 3" {
            assert!(first.is_none(), "one launch before restart");
            first = Some(launch.pipes[2]);
        }
    }
    let first = first.expect("forge was launched");
    let Way::Given(_) = first.way else { panic!("forge is given its listener") };
    harness.sim.finish_service(pid, Exit::Code(1));
    harness.run_until_idle();
    // The parent still holds the socket between children, so this new connection queues too.
    let connection = socket(&mut harness.sim, client);
    assert_eq!(call(&mut harness.sim, client, Op::Connect { fd: connection, addr: address }).result, Ok(Done::Nothing));
    harness.sim.advance_to(harness.sim.now().saturating_add(Duration::from_millis(500)));
    harness.run_until_idle();
    let mut launches = List::with_capacity(2);
    for launch in &harness.launches {
        if launch.name.as_ref() == b"fake forge 3" {
            launches.push(launch).unwrap();
        }
    }
    assert_eq!(launches.len(), 2);
    assert_eq!(launches.get(1).unwrap().pipes[2], first, "restart gets the parent's same held listener");
    let child = harness.hosted.get(harness.child_index(b"fake forge 3")).unwrap();
    assert_ne!(child.pid, pid);
    let pid = child.pid;
    let Ok(Done::Accepted { fd, .. }) = call(&mut harness.sim, pid, Op::Accept { fd: Fd::new(3) }).result else {
        panic!("restart accepts the connection queued between children")
    };
    call(&mut harness.sim, pid, Op::Close { fd });
    call(&mut harness.sim, client, Op::Close { fd: connection });
    harness.terminate();
    assert_eq!(harness.run_to_end(), Exit::Code(0));
    harness.sim.assert_quiescent(harness.pid);
    harness.sim.assert_no_open_fds(harness.pid);
    harness.sim.assert_no_open_fds(client);
}

fn terminal_harness() -> Harness {
    Harness::with_stubborn(&[long(b"serve", &[b"never"]), long(b"diagnostic", &[b"never"])], true)
}
#[test]
fn split_links_and_diagnostics_form_ordered_first_fresh_and_restarted_blocks() {
    let mut harness = terminal_harness();
    harness.run_until_idle();
    let serve = harness.child_index(b"serve");
    let diagnostic = harness.child_index(b"diagnostic");
    harness.write_child(serve, 1, b"http://first");
    harness.run_until_idle();
    harness.write_child(diagnostic, 2, b"interposed\n");
    harness.run_until_idle();
    assert_eq!(harness.stdout().as_ref(), b"", "split URL stays in assembly");
    harness.write_child(serve, 1, b"/link\n");
    harness.run_until_idle();
    harness.wall = skein_lib::Wall::from_nanos(86_399_000_000_000);
    harness.write_child(serve, 1, b"http://fre");
    harness.run_until_idle();
    harness.write_child(diagnostic, 1, b"between refresh pieces\n");
    harness.run_until_idle();
    harness.write_child(serve, 1, b"sh\n");
    harness.run_until_idle();
    let pid = harness.hosted.get(serve).unwrap().pid;
    harness.sim.finish_service(pid, Exit::Code(0));
    harness.run_until_idle();
    harness.wall = skein_lib::Wall::from_nanos(86_400_000_000_000);
    harness.sim.advance_to(harness.sim.now().saturating_add(Duration::from_millis(500)));
    harness.run_until_idle();
    harness.write_child(harness.child_index(b"serve"), 1, b"http://restart\n");
    harness.run_until_idle();
    harness.terminate();
    assert_eq!(harness.run_to_end(), Exit::Code(0));
    assert_eq!(
        harness.terminal_bytes().as_ref(),
        concat!(
            "diagnostic: interposed\n",
            "Open this link in a browser on this machine.\n\n",
            "http://first/link\n",
            "\nIt signs you in as this temper's owner, once, within 2 hours.\n",
            "Opened after that, it prints a fresh one here.\n",
            "In a terminal: temper http://first\n",
            "diagnostic: between refresh pieces\n\nhttp://fresh\n",
            "23:59:59Z serve exited 0; restarting in 0.5 s\n",
            "\nhttp://restart\n",
        )
        .as_bytes()
    );
    harness.sim.assert_quiescent(harness.pid);
    harness.sim.assert_no_open_fds(harness.pid);
}
#[test]
fn held_output_preserves_completion_wall_and_blocks_both_descriptors() {
    use skein_shell::Host;
    let mut harness = terminal_harness();
    harness.run_until_idle();
    harness.write_child(harness.child_index(b"serve"), 1, b"first\n");
    harness.run_until_idle();
    harness.hold_terminal_ack = true;
    harness.write_child(harness.child_index(b"diagnostic"), 2, b"held diagnostic\n");
    harness.run_until_idle();
    let before = harness.terminal_writes.len();
    harness.wall = skein_lib::Wall::from_nanos(86_399_000_000_000);
    // The existing io profile reads one byte at a time. Assemble the prefix
    // while room exists, then retain the final LF after diagnostics use that
    // room. Its completion is old even though extraction must wait.
    harness.write_child(harness.child_index(b"serve"), 1, b"fresh");
    harness.run_until_idle();
    for words in [b"queued one\n".as_slice(), b"queued two\n", b"later diagnostic\n"] {
        harness.write_child(harness.child_index(b"diagnostic"), 1, words);
        harness.run_until_idle();
    }
    harness.write_child(harness.child_index(b"serve"), 1, b"\n");
    harness.run_until_idle();
    assert_eq!(harness.terminal_writes.len(), before, "small Room and Send never release the active item");
    assert!(!harness.service.work_pending(harness.sim.now()), "blocked output lets the loop sleep");
    harness.wall = skein_lib::Wall::from_nanos(172_800_000_000_000);
    harness.release_terminal_ack();
    harness.run_until_idle();
    let all = harness.terminal_bytes();
    assert!(Harness::contains(&all, b"\nfresh\n"), "a fresh link stays untimed under admission pressure");
    assert!(Harness::contains(&all, b"diagnostic: later diagnostic\n"));
    harness.terminate();
    assert_eq!(harness.run_to_end(), Exit::Code(0));
    harness.sim.assert_quiescent(harness.pid);
    harness.sim.assert_no_open_fds(harness.pid);
}
#[test]
fn serve_overflow_is_refused_without_bytes_then_eof_link_and_diagnostic_drain() {
    let mut harness = terminal_harness();
    harness.run_until_idle();
    let serve = harness.child_index(b"serve");
    let cap = super::fixtures::line_cap();
    harness.write_child(serve, 1, &cap);
    harness.run_until_idle();
    harness.write_child(serve, 1, b"x\ngood\nlast");
    harness.run_until_idle();
    harness.write_child(harness.child_index(b"diagnostic"), 2, b"EOF fragment");
    harness.run_until_idle();
    harness.terminate();
    assert_eq!(harness.run_to_end(), Exit::Code(0));
    assert_eq!(harness.stdout().as_ref(), b"good\nlast\n");
    let output = harness.terminal_bytes();
    assert!(!Harness::contains(&output, b"xxxx"), "no refused URL prefix or tail is written");
    assert!(Harness::contains(&output, b"box: serve: sign-in link too long\nOpen this link"));
    assert!(Harness::contains(&output, b"diagnostic: EOF fragment\n"));
    harness.sim.assert_quiescent(harness.pid);
    harness.sim.assert_no_open_fds(harness.pid);
}

#[test]
fn output_ack_drains_loaded_orders_without_spinning_or_losing_items() {
    use skein_shell::Host;
    let mut harness = Harness::with_stubborn(&[long(b"serve", &[b"never"])], true);
    harness.hold_terminal_ack = true;
    harness.run_until_idle();
    let (queued, orders) = crate::host::pressure::saturate(&mut harness.service);
    harness.run_until_idle();
    assert!(crate::host::pressure::blocked(&harness.service));
    assert!(!harness.service.work_pending(harness.sim.now()), "full queues without a credit are not runnable work");
    assert!(crate::host::pressure::bounded(&harness.service));
    harness.release_terminal_ack(); // Its full-budget Room advances the blocked policy orders.
    harness.run_until_idle();
    let output = harness.terminal_bytes();
    assert_eq!(bytes::count(&output, b"queued\n", queued.saturating_add(1)), queued);
    assert_eq!(bytes::count(&output, b"busy; giving up\n", orders.saturating_add(1)), orders);
    assert!(crate::host::pressure::bounded(&harness.service));
    harness.terminate();
    assert_eq!(harness.run_to_end(), Exit::Code(0));
    harness.sim.assert_quiescent(harness.pid);
    harness.sim.assert_no_open_fds(harness.pid);
}

#[test]
fn failed_terminal_output_still_drains_retained_child_reads_and_closes() {
    use skein_io::kernel::Error;
    let mut harness = Harness::with_stubborn(&[long(b"serve", &[b"never"])], true);
    harness.hold_terminal_ack = true;
    harness.run_until_idle();
    // A first block cannot fit beside the active Starting notice. This leaves
    // admitted bytes in the pipe and io intake when the output breaks.
    harness.write_child(harness.child_index(b"serve"), 1, b"retained link\n");
    harness.run_until_idle();
    let mut failed = harness.held_terminal_ack.pop().expect("active terminal write");
    failed.result = Err(Error::BrokenPipe);
    harness.held_terminal_ack.push(failed);
    harness.release_terminal_ack();
    assert_eq!(harness.run_to_end(), Exit::Code(1));
    assert!(crate::host::pressure::bounded(&harness.service));
    harness.sim.assert_quiescent(harness.pid);
    harness.sim.assert_no_open_fds(harness.pid);
}
#[test]
fn a_legacy_handoff_does_not_print_or_mark_the_first_block() {
    use super::fixtures::link;
    let mut harness = Harness::with_stubborn(&[long(b"serve", &[b"never"]), link(b"seed", &[b"never"])], true);
    harness.run_until_idle();
    harness.write_child(harness.child_index(b"serve"), 1, b"held\n");
    harness.run_until_idle();
    assert!(harness.terminal_bytes().is_empty());
    assert!(Harness::contains(&harness.log(), b"00:00:00Z Starting serve\n00:00:00Z Starting seed\n"));
    assert_eq!(harness.read_stdin(harness.child_index(b"seed"), LINE_BYTES).as_ref(), b"held\n");
    harness.write_child(harness.child_index(b"serve"), 1, b"visible\n");
    harness.run_until_idle();
    assert!(Harness::contains(&harness.terminal_bytes(), b"Open this link in a browser on this machine."));
    assert_eq!(harness.stdout().as_ref(), b"visible\n");
    harness.terminate();
    assert_eq!(harness.run_to_end(), Exit::Code(0));
    harness.sim.assert_quiescent(harness.pid);
    harness.sim.assert_no_open_fds(harness.pid);
}

#[test]
fn labelled_group_failure_keeps_serve_usable_and_wire_replies_numeric() {
    let mut steps = grouped_steps();
    steps.asked[0].label = bytes::copy_of(b"other label\xff");
    steps.asked[0].steps[0].1 = once(b"prepare 3", &[b"exit", b"7"]);
    let mut harness = Harness::grouped(steps, &[], true);
    harness.run_until_idle();
    harness.write_child(harness.serve_index(), 1, b"http://first\n");
    harness.run_until_idle();
    harness.stderr();
    for _attempt in 0_u32..2 {
        harness.ask(b"start 3\n");
        harness.run_until_idle();
        assert_eq!(harness.reply_bytes().as_ref(), b"step 3 prepare\nfailed 3 prepare 3 exited 7\n");
        assert_eq!(harness.stderr().as_ref(), b"00:00:00Z other label\xff: failed, prepare 3 exited 7\n");
        harness.read_reply(256);
        harness.replies.clear();
        harness.ask(b"start 99\n");
        harness.run_until_idle();
        assert_eq!(harness.reply_bytes().as_ref(), b"failed 99 not asked\n");
        assert!(harness.stderr().is_empty(), "a refusal is a wire reply only");
        harness.read_reply(256);
        harness.replies.clear();
    }
    assert_eq!(harness.spawn_times(b"serve").len(), 1, "failure leaves the same serve usable");
    assert_eq!(harness.spawn_times(b"prepare 3").len(), 2);
    assert_eq!(harness.spawn_times(b"forgejo 3").len(), 0);
    harness.terminate();
    assert_eq!(harness.run_to_end(), Exit::Code(0));
    assert!(!Harness::contains(&harness.stderr(), b"giving up"), "group failure never becomes box failure");
    harness.sim.assert_quiescent(harness.pid);
    harness.sim.assert_no_open_fds(harness.pid);
    assert_eq!(harness.machine.open_handles(), 0);
}

#[test]
fn lifecycle_facts_keep_source_wall_and_order_under_terminal_backpressure() {
    use skein_lib::Wall;
    use skein_shell::Host;
    let mut limits = crate::limits::small();
    limits.restarts = 1;
    let mut harness = Harness::configured(
        temper_forge_setup::Steps {
            always: Box::new([long(b"serve", &[b"never"]), long(b"other", &[b"never"])]),
            asked: Box::new([]),
        },
        &[],
        true,
        skein_sim::Config::calm().buffer,
        &limits,
    );
    harness.run_until_idle();
    harness.write_child(harness.serve_index(), 1, b"http://first\n");
    harness.run_until_idle();
    harness.hold_terminal_ack = true;
    harness.write_child(harness.child_index(b"other"), 2, b"held\n");
    harness.run_until_idle();
    for words in [b"queued one\n".as_slice(), b"queued two\n", b"queued three\n"] {
        harness.write_child(harness.child_index(b"other"), 2, words);
        harness.run_until_idle();
    }
    harness.wall = Wall::from_nanos(86_399_000_000_000);
    let serve = harness.hosted.get(harness.serve_index()).unwrap().pid;
    harness.sim.finish_service(serve, Exit::Signal(9));
    harness.run_until_idle();
    // Both the restart decision and its monotonic timer progress while output
    // waits for its ACK. The Starting fact reaches the box log independently.
    harness.wall = Wall::from_nanos(86_400_000_000_000);
    harness.sim.advance_to(harness.sim.now().saturating_add(limits.backoff));
    harness.run_until_idle();
    assert_eq!(harness.spawn_times(b"serve").len(), 2);
    let serve = harness.hosted.get(harness.serve_index()).unwrap().pid;
    harness.sim.finish_service(serve, Exit::Code(0));
    harness.run_until_idle();
    assert_eq!(
        crate::host::pressure::outcome(&harness.service),
        Some(crate::machine::Outcome::GaveUp { step: 0, restarts: 1 })
    );
    assert_eq!(harness.service.exit(), None, "another child and terminal ACK remain outstanding");
    assert!(!Harness::contains(&harness.terminal_bytes(), b"giving up"));
    assert!(
        crate::host::pressure::fact_waits(
            &harness.service,
            crate::terminal::Fact::GivingUp {
                step: 0,
                why: crate::machine::Why::Exited { step: 0, exit: Exit::Code(0) }
            },
            Wall::from_nanos(86_400_000_000_000),
        ),
        "the give-up fact must wait unformatted in the policy queue"
    );
    assert!(crate::host::pressure::bounded(&harness.service));
    harness.wall = Wall::from_nanos(172_861_000_000_000);
    harness.release_terminal_ack();
    harness.run_until_idle();
    let output = harness.terminal_bytes();
    assert!(Harness::contains(&output, b"other: held\nother: queued one\nother: queued two\nother: queued three\n23:59:59Z serve killed by signal 9; restarting in 0.5 s\n00:00:00Z serve exited 0; giving up\n"), "source walls and decision order survive output delay: {output:?}");
    assert!(Harness::contains(&harness.log(), b"00:00:00Z Starting serve\n"));
    assert_eq!(bytes::count(&output, b"giving up\n", 2), 1);
    assert_eq!(harness.service.exit(), None, "giving up is visible before the stubborn other child closes");
    assert_eq!(harness.run_to_end(), Exit::Code(1));
    assert_eq!(bytes::count(&harness.terminal_bytes(), b"giving up\n", 2), 1, "no final duplicate");
    harness.sim.assert_quiescent(harness.pid);
    harness.sim.assert_no_open_fds(harness.pid);
}

#[test]
fn long_labels_pass_actual_output_admission_with_tiny_control_and_write_caps() {
    use skein_lib::Wall;
    for buffer in [8, 16] {
        let label: Box<[u8]> = Box::new([b'L'; 4352]);
        let name: Box<[u8]> = Box::new([b'N'; 80]);
        let mut steps = grouped_steps();
        steps.asked[0].label = label.clone();
        steps.asked[0].steps[0].1 = once(&name, &[b"exit", b"255"]);
        let limits = crate::limits::Limits { line_bytes: 64, ..crate::limits::small() };
        let mut harness = Harness::configured(steps, &[], true, buffer, &limits);
        harness.run_until_idle();
        harness.write_child(harness.serve_index(), 1, b"link\n");
        harness.run_until_idle();
        harness.wall = Wall::from_nanos(86_399_000_000_000);
        harness.ask(b"start 3\n");
        harness.run_until_idle();
        let expected_reply = crate::lines::joined(&[b"step 3 prepare\nfailed 3 ", name.get(..54).unwrap(), b"\n"]);
        let mut received = List::with_capacity(u32::try_from(expected_reply.len()).unwrap());
        for _ in 0..expected_reply.len() {
            if received.len() == u32::try_from(expected_reply.len()).unwrap() {
                break;
            }
            for byte in harness.read_reply(u32::try_from(expected_reply.len()).unwrap().saturating_sub(received.len()))
            {
                received.push(byte).unwrap();
            }
            harness.run_until_idle();
        }
        assert_eq!(received.as_slice(), expected_reply.as_ref(), "wire WHY is still truncated to the 64-byte line cap");
        let expected = crate::lines::joined(&[b"23:59:59Z ", &label, b": failed, ", &name, b" exited 255\n"]);
        assert!(
            Harness::contains(&harness.terminal_bytes(), &expected),
            "long lifecycle lines must actually write with buffer {buffer}"
        );
        assert!(crate::host::pressure::bounded(&harness.service));
        harness.terminate();
        assert_eq!(harness.run_to_end(), Exit::Code(0));
        harness.sim.assert_quiescent(harness.pid);
        harness.sim.assert_no_open_fds(harness.pid);
        assert_eq!(harness.machine.open_handles(), 0);
    }
}

#[test]
fn an_unsent_restart_is_withdrawn_while_old_output_waits_on_the_terminal() {
    let mut harness = Harness::with_stubborn(&[long(b"serve", &[b"never"])], true);
    harness.hold_terminal_ack = true;
    harness.run_until_idle();
    crate::host::pressure::fill_terminal(&mut harness.service);
    let serve = harness.child_index(b"serve");
    harness.write_child(serve, 1, b"x");
    harness.run_until_idle();
    let pid = harness.hosted.get(serve).unwrap().pid;
    harness.sim.finish_service(pid, Exit::Code(0));
    harness.run_until_idle();
    harness.sim.advance_to(harness.sim.now().saturating_add(Duration::from_millis(500)));
    harness.run_until_idle();
    assert_eq!(harness.spawn_times(b"serve").len(), 1, "old EOF fragment holds the restart unsent");
    harness.terminate();
    harness.run_until_idle();
    harness.release_terminal_ack();
    assert_eq!(harness.run_to_end(), Exit::Code(0));
    assert_eq!(harness.spawn_times(b"serve").len(), 1, "withdrawn restart never launches");
    assert_eq!(harness.stdout().as_ref(), b"x\n", "old output is preserved");
    harness.sim.assert_quiescent(harness.pid);
    harness.sim.assert_no_open_fds(harness.pid);
}
#[test]
fn a_held_spawn_starts_late_and_is_stopped_or_closed_after_second_termination() {
    for forced in [false, true] {
        let mut harness = Harness::with_stubborn(
            &[long(b"forgejo", &[b"never"]).with_ready(([127, 0, 0, 1], 3000).into()), long(b"runner", &[b"never"])],
            true,
        );
        harness.hold_spawn = Some(bytes::copy_of(b"forgejo"));
        harness.run_until_idle();
        assert!(harness.held_spawn.is_some());
        harness.terminate();
        harness.run_until_idle();
        if forced {
            harness.terminate();
            harness.run_until_idle();
        }
        harness.release_spawn();
        assert_eq!(harness.run_to_end(), Exit::Code(0));
        let mut signals = 0_u32;
        for entry in harness.sim.trace() {
            if entry.pid == harness.pid
                && let Event::Submit { kind: Summary::Signal { signal: 15, to: Target::Child, .. }, .. } = entry.event
            {
                signals += 1;
            }
        }
        assert_eq!(signals, if forced { 0 } else { 2 });
        assert_eq!(harness.spawn_times(b"forgejo").len(), 1);
        assert_eq!(harness.spawn_times(b"runner").len(), 0, "late child is never readied or continued");
        harness.sim.assert_quiescent(harness.pid);
        harness.sim.assert_no_open_fds(harness.pid);
    }
}
#[test]
fn a_held_spawn_failing_during_stop_ends_without_retry_or_failure_notice() {
    let mut harness = Harness::with_stubborn(&[long(b"forgejo", &[b"never"]), long(b"runner", &[b"never"])], true);
    harness.hold_spawn = Some(bytes::copy_of(b"forgejo"));
    harness.run_until_idle();
    assert!(harness.held_spawn.is_some());
    harness.terminate();
    harness.run_until_idle();
    harness.fail_spawn();
    assert_eq!(harness.run_to_end(), Exit::Code(0));
    assert_eq!(harness.spawn_times(b"forgejo").len(), 1, "counts the failed call too");
    assert_eq!(harness.spawn_times(b"runner").len(), 0);
    assert!(!Harness::contains(&harness.stderr(), b"restarting"));
    assert!(!Harness::contains(&harness.stderr(), b"giving up"));
    harness.sim.assert_quiescent(harness.pid);
    harness.sim.assert_no_open_fds(harness.pid);
}
#[test]
fn a_groups_late_restart_preparation_is_stopped_then_the_group_can_start_again() {
    let mut harness = Harness::grouped(grouped_steps(), &[], true);
    harness.run_until_idle();
    harness.ask(b"start 3\n");
    harness.run_until_idle();
    harness.read_reply(256);
    harness.replies.clear();
    harness.hold_spawn = Some(bytes::copy_of(b"prepare 3"));
    let forge = harness.child_index(b"forgejo 3");
    let pid = harness.hosted.get(forge).unwrap().pid;
    harness.sim.finish_service(pid, Exit::Code(1));
    harness.run_until_idle();
    harness.sim.advance_to(harness.sim.now().saturating_add(Duration::from_millis(500)));
    harness.run_until_idle();
    assert!(harness.held_spawn.is_some());
    harness.ask(b"stop 3\n");
    harness.run_until_idle();
    assert!(harness.reply_bytes().is_empty(), "stop awaits the admitted preparation");
    harness.release_spawn();
    harness.run_until_idle();
    assert_eq!(harness.reply_bytes().as_ref(), b"stopped 3\n");
    assert_eq!(harness.spawn_times(b"forgejo 3").len(), 1, "late once step never continues");
    harness.read_reply(256);
    harness.replies.clear();
    harness.ask(b"start 3\n");
    harness.run_until_idle();
    assert_eq!(harness.reply_bytes().as_ref(), b"step 3 prepare\nstep 3 run\nstarted 3\n");
    assert_eq!(harness.spawn_times(b"forgejo 3").len(), 2);
    harness.terminate();
    assert_eq!(harness.run_to_end(), Exit::Code(0));
    harness.sim.assert_quiescent(harness.pid);
    harness.sim.assert_no_open_fds(harness.pid);
}

#[test]
fn a_spawn_order_ahead_of_a_blocked_stop_notice_still_never_launches() {
    let mut harness = Harness::with_stubborn(&[once(b"prepare", &[b"never"]), long(b"serve", &[b"never"])], true);
    harness.hold_terminal_ack = true;
    harness.run_until_idle();
    crate::host::pressure::fill_terminal(&mut harness.service);
    let prepare = harness.child_index(b"prepare");
    for line in [b"one\n".as_slice(), b"two\n", b"three\n", b"four\n"] {
        harness.write_child(prepare, 1, line);
        harness.run_until_idle();
    }
    let pid = harness.hosted.get(prepare).unwrap().pid;
    harness.sim.finish_service(pid, Exit::Code(0));
    harness.run_until_idle();
    assert_eq!(harness.spawn_times(b"serve").len(), 0);
    harness.terminate();
    harness.run_until_idle();
    harness.release_terminal_ack();
    assert_eq!(harness.run_to_end(), Exit::Code(0));
    assert_eq!(harness.spawn_times(b"serve").len(), 0, "queued withdrawal prevents admission behind a full terminal");
    harness.sim.assert_quiescent(harness.pid);
    harness.sim.assert_no_open_fds(harness.pid);
}

#[test]
fn pipelined_commands_keep_reply_fifo_under_load() {
    let mut harness = Harness::grouped(grouped_steps(), &[], true);
    harness.run_until_idle();
    let mut commands = List::uncapped();
    let mut expected = List::uncapped();
    for forge in 100_u64..164 {
        for byte in crate::lines::joined(&[b"start ", skein_lib::Decimal::of(forge).as_bytes(), b"\n"]) {
            commands.push(byte).unwrap();
        }
        for byte in crate::lines::joined(&[b"failed ", skein_lib::Decimal::of(forge).as_bytes(), b" not asked\n"]) {
            expected.push(byte).unwrap();
        }
    }
    harness.ask(commands.as_slice());
    harness.run_until_idle();
    assert_eq!(harness.reply_bytes().as_ref(), expected.as_slice(), "each command's replies precede the next");
    assert!(crate::host::pressure::bounded(&harness.service));
    harness.terminate();
    assert_eq!(harness.run_to_end(), Exit::Code(0));
    harness.sim.assert_quiescent(harness.pid);
    harness.sim.assert_no_open_fds(harness.pid);
}

#[test]
fn admitted_termination_load_precedes_a_due_restart_timer() {
    let mut harness = Harness::with_stubborn(&[long(b"serve", &[b"never"])], true);
    harness.run_until_idle();
    let serve = harness.hosted.get(harness.serve_index()).unwrap().pid;
    harness.sim.finish_service(serve, Exit::Code(0));
    harness.run_until_idle();
    assert_eq!(harness.spawn_times(b"serve").len(), 1);
    crate::host::pressure::terminate_load(&mut harness.service);
    harness.sim.advance_to(harness.sim.now().saturating_add(Duration::from_millis(500)));
    assert_eq!(harness.run_to_end(), Exit::Code(0));
    assert_eq!(harness.spawn_times(b"serve").len(), 1, "admitted stop wins over the expired retry");
    assert_eq!(bytes::count(&harness.log(), b"Stopping\n", 2), 1);
    assert_eq!(bytes::count(&harness.log(), b"Stopped\n", 2), 1);
    harness.sim.assert_quiescent(harness.pid);
    harness.sim.assert_no_open_fds(harness.pid);
}

#[test]
fn record_diagnostics_precede_group_failure_notices_under_command_load() {
    use skein_fake_machine::Item;
    let denied = [Item::directory(b"box"), Item::directory(b"box/started").mode(0o500)];
    let mut harness = Harness::grouped(grouped_steps(), &denied, true);
    harness.run_until_idle();
    let mut commands = List::uncapped();
    for _ in 0_u32..64 {
        for byte in b"start 3\n" {
            commands.push(*byte).unwrap();
        }
    }
    harness.ask(commands.as_slice());
    harness.run_until_idle();
    let output = harness.terminal_bytes();
    let notice = b"box: create box/started/3: permission denied\n";
    let failed = b"Forge 3 (Forgejo): failed, record not written\n";
    let attempts = bytes::count(&output, notice, 65);
    assert!(attempts > 1, "the command load retries the record");
    assert_eq!(bytes::count(&output, failed, 65), attempts);
    let mut remaining = output.as_ref();
    for _ in 0..attempts {
        let before = bytes::find(remaining, notice).unwrap();
        let after = bytes::find(remaining, failed).unwrap();
        assert!(before < after, "the record error is admitted before its failure fact");
        remaining = &remaining[after + failed.len()..];
    }
    assert_eq!(bytes::count(&harness.reply_bytes(), b"failed 3 record not written\n", 65), attempts);
    harness.terminate();
    assert_eq!(harness.run_to_end(), Exit::Code(0));
    harness.sim.assert_quiescent(harness.pid);
    harness.sim.assert_no_open_fds(harness.pid);
    assert_eq!(harness.machine.open_handles(), 0);
}

#[test]
fn output_ack_bypasses_request_blocked_ordinary_fifo_under_load() {
    use crate::host::pressure;
    let mut harness = Harness::with_stubborn(&[long(b"serve", &[b"never"])], true);
    harness.hold_terminal_ack = true;
    harness.run_until_idle();
    pressure::saturate(&mut harness.service);
    harness.run_until_idle();
    pressure::ordinary_load(&mut harness.service);
    harness.release_terminal_ack();
    for _ in 0_u32..16 {
        pressure::reserve_requests(&mut harness.service);
        harness.turn();
        assert!(pressure::ordinary_load_waits(&harness.service), "ordinary source walls stay in FIFO order");
    }
    assert!(Harness::contains(&harness.terminal_bytes(), b"queued\n"), "ACKs bypass the retained ordinary FIFO");
    harness.run_until_idle();
    assert!(!pressure::ordinary_load_waits(&harness.service));
    assert!(pressure::bounded(&harness.service));
    harness.terminate();
    assert_eq!(harness.run_to_end(), Exit::Code(0));
    harness.sim.assert_quiescent(harness.pid);
    harness.sim.assert_no_open_fds(harness.pid);
}

#[test]
fn a_ready_probe_deadline_waits_for_request_credit() {
    use skein_shell::Host;
    let mut harness =
        Harness::with_stubborn(&[long(b"forgejo", &[b"never"]).with_ready(([127, 0, 0, 1], 3000).into())], true);
    harness.run_until_idle();
    assert!(harness.service.next_policy_deadline().is_some());
    crate::host::pressure::reserve_requests(&mut harness.service);
    assert_eq!(harness.service.next_policy_deadline(), None, "a blocked probe timer cannot keep waking the loop");
    harness.terminate();
    assert_eq!(harness.run_to_end(), Exit::Code(0));
    harness.sim.assert_quiescent(harness.pid);
    harness.sim.assert_no_open_fds(harness.pid);
}

#[test]
fn a_kind_start_writes_then_runs_and_removal_unlinks_both_records() {
    use skein_fake_machine::{How, Refusal};
    let mut h = Harness::grouped(super::fixtures::kind_steps(), &[], true);
    h.run_until_idle();
    h.ask(b"start 17 fake\n");
    h.run_until_idle();
    assert_eq!(h.reply_bytes().as_ref(), b"step 17 run\nstarted 17\n");
    let root = h.machine.reopen_root(0);
    let group = h.machine.open(root, b"box/groups/17", How::Read).unwrap();
    assert_eq!(h.machine.read(group, 0, 8).unwrap(), b"fake\n");
    h.machine.close(group);
    let record = h.machine.open(root, b"box/started/17", How::Read).unwrap();
    h.machine.close(record);
    let log = h.machine.open(root, b"box/logs/fake-forge-17.log", How::Read).unwrap();
    h.machine.close(log);
    h.ask(b"start 18 fake\n");
    h.run_until_idle();
    assert!(h.reply_bytes().ends_with(b"failed 18 busy\n"));
    h.replies.clear();
    h.ask(b"remove 17\n");
    h.run_until_idle();
    let pid = h.hosted.get(h.child_index(b"fake-forge")).unwrap().pid;
    h.sim.finish_service(pid, Exit::Code(0));
    h.run_until_idle();
    assert_eq!(h.reply_bytes().as_ref(), b"removed 17\n");
    assert_eq!(h.machine.open(root, b"box/groups/17", How::Read), Err(Refusal::NotFound));
    assert_eq!(h.machine.open(root, b"box/started/17", How::Read), Err(Refusal::NotFound));
    h.ask(b"start 18 fake\n");
    h.run_until_idle();
    assert!(h.reply_bytes().ends_with(b"started 18\n"));
    let log = h.machine.open(root, b"box/logs/fake-forge-18.log", How::Read).unwrap();
    h.machine.close(log);
    h.machine.close(root);
    h.terminate();
    assert_eq!(h.run_to_end(), Exit::Code(0));
    h.sim.assert_quiescent(h.pid);
    h.sim.assert_no_open_fds(h.pid);
    assert_eq!(h.machine.open_handles(), 0);
}
#[test]
fn a_failed_remover_reports_its_exit_and_preserves_both_records_for_retry() {
    use skein_fake_machine::How;
    use temper_forge_setup::FORGE_ARGUMENT;
    let mut steps = super::fixtures::kind_steps();
    steps.asked.get_mut(0).unwrap().remover = Some(once(b"remove", &[b"exit", b"7", FORGE_ARGUMENT]));
    let mut h = Harness::grouped(steps, &[], true);
    h.run_until_idle();
    h.ask(b"start 17 fake\n");
    h.run_until_idle();
    h.replies.clear();
    for attempt in 1..=2 {
        h.ask(b"remove 17\n");
        h.run_until_idle();
        if attempt == 1 {
            let pid = h.hosted.get(h.child_index(b"fake-forge")).unwrap().pid;
            h.sim.finish_service(pid, Exit::Code(0));
            h.run_until_idle();
        }
        assert!(h.reply_bytes().ends_with(b"failed 17 remove: exited 7\n"));
        assert_eq!(h.spawn_times(b"remove").len(), attempt);
        let root = h.machine.reopen_root(0);
        for path in [b"box/groups/17".as_slice(), b"box/started/17"] {
            let file = h.machine.open(root, path, How::Read).expect("retry keeps both files");
            h.machine.close(file);
        }
        h.machine.close(root);
    }
    h.terminate();
    assert_eq!(h.run_to_end(), Exit::Code(0));
    h.sim.assert_quiescent(h.pid);
    h.sim.assert_no_open_fds(h.pid);
    assert_eq!(h.machine.open_handles(), 0);
}

#[test]
fn an_unknown_remove_unlinks_an_orphan_started_record() {
    use skein_fake_machine::{How, Item, Refusal};
    let items = [Item::directory(b"box"), Item::directory(b"box/started"), Item::file(b"box/started/99", b"")];
    let mut h = Harness::grouped(super::fixtures::kind_steps(), &items, true);
    h.run_until_idle();
    h.ask(b"remove 99\n");
    h.run_until_idle();
    assert_eq!(h.reply_bytes().as_ref(), b"removed 99\n");
    assert_eq!(h.spawn_times(b"remove").len(), 0);
    let root = h.machine.reopen_root(0);
    assert_eq!(h.machine.open(root, b"box/started/99", How::Read), Err(Refusal::NotFound));
    h.machine.close(root);
    h.terminate();
    assert_eq!(h.run_to_end(), Exit::Code(0));
    h.sim.assert_quiescent(h.pid);
    h.sim.assert_no_open_fds(h.pid);
    assert_eq!(h.machine.open_handles(), 0);
}
#[test]
fn a_failed_group_file_write_starts_no_group_step() {
    use skein_fake_machine::Item;
    let items = [Item::directory(b"box"), Item::directory(b"box/groups").mode(0o500)];
    let mut h = Harness::grouped(super::fixtures::kind_steps(), &items, true);
    h.run_until_idle();
    h.ask(b"start 17 fake\n");
    h.run_until_idle();
    assert_eq!(h.reply_bytes().as_ref(), b"failed 17 record not written\n");
    assert_eq!(h.spawn_times(b"fake-forge").len(), 0);
    assert_eq!(h.spawn_times(b"remove").len(), 0);
    h.terminate();
    assert_eq!(h.run_to_end(), Exit::Code(0));
    h.sim.assert_quiescent(h.pid);
    h.sim.assert_no_open_fds(h.pid);
    assert_eq!(h.machine.open_handles(), 0);
}
#[test]
fn an_unlink_failure_reports_failure_and_keeps_the_started_record() {
    use skein_fake_machine::{How, Item};
    let items = [
        Item::directory(b"box"),
        Item::directory(b"box/groups").mode(0o500),
        Item::directory(b"box/started"),
        Item::file(b"box/groups/17", b"fake\n"),
        Item::file(b"box/started/17", b""),
    ];
    let mut h = Harness::grouped(super::fixtures::kind_steps(), &items, true);
    h.run_until_idle();
    h.ask(b"remove 17\n");
    h.run_until_idle();
    let pid = h.hosted.get(h.child_index(b"fake-forge")).unwrap().pid;
    h.sim.finish_service(pid, Exit::Code(0));
    h.run_until_idle();
    assert_eq!(h.reply_bytes().as_ref(), b"failed 17 remove: files not removed\n");
    let root = h.machine.reopen_root(0);
    for path in [b"box/groups/17".as_slice(), b"box/started/17"] {
        let file = h.machine.open(root, path, How::Read).unwrap();
        h.machine.close(file);
    }
    h.machine.close(root);
    h.terminate();
    assert_eq!(h.run_to_end(), Exit::Code(0));
    h.sim.assert_quiescent(h.pid);
    h.sim.assert_no_open_fds(h.pid);
    assert_eq!(h.machine.open_handles(), 0);
}
#[test]
fn a_kind_can_run_when_its_log_directory_is_unavailable() {
    use skein_fake_machine::Item;
    let items = [Item::directory(b"box"), Item::file(b"box/logs", b"kept")];
    let mut h = Harness::grouped(super::fixtures::kind_steps(), &items, true);
    h.run_until_idle();
    h.ask(b"start 17 fake\n");
    h.run_until_idle();
    assert_eq!(h.reply_bytes().as_ref(), b"step 17 run\nstarted 17\n");
    let child = h.child_index(b"fake-forge");
    h.write_child(child, 2, b"diagnostic\n");
    h.run_until_idle();
    assert!(Harness::contains(&h.stderr(), b"fake-forge-17: diagnostic\n"));
    h.terminate();
    assert_eq!(h.run_to_end(), Exit::Code(0));
    h.sim.assert_quiescent(h.pid);
    h.sim.assert_no_open_fds(h.pid);
    assert_eq!(h.machine.open_handles(), 0);
}

#[test]
fn an_absent_group_file_does_not_reset_a_numbered_groups_live_state() {
    let mut h = Harness::grouped(grouped_steps(), &[], true);
    h.run_until_idle();
    h.ask(b"start 3\n");
    h.run_until_idle();
    h.replies.clear();
    h.ask(b"remove 3\n");
    h.run_until_idle();
    assert_eq!(h.reply_bytes().as_ref(), b"removed 3\n");
    h.replies.clear();
    h.ask(b"start 3\n");
    h.run_until_idle();
    assert_eq!(h.reply_bytes().as_ref(), b"started 3\n");
    assert_eq!(h.spawn_times(b"forgejo 3").len(), 1);
    h.terminate();
    assert_eq!(h.run_to_end(), Exit::Code(0));
    h.sim.assert_quiescent(h.pid);
    h.sim.assert_no_open_fds(h.pid);
    assert_eq!(h.machine.open_handles(), 0);
}

#[test]
fn debug_of_settings_and_first_link_hides_credentials_and_the_url() {
    use crate::host::{Echo, Service, Settings};
    use skein_io::kernel::Fd;

    for with_front in [false, true] {
        let canary = b"TOKEN=canary-7f3a9c";
        let link = crate::host::FirstLink {
            url: bytes::copy_of(FIRST_URL),
            local: if with_front { Some(skein_lib::Secret::new(bytes::copy_of(LOCAL_SECRET))) } else { None },
            secret: skein_lib::Secret::new(bytes::copy_of(FIRST_SECRET)),
        };
        let link_debug = alloc::fmt::format(format_args!("{link:?}"));
        assert!(link_debug == "FirstLink { .. }", "FirstLink Debug hides both fields");
        let settings = Settings {
            word: b"temper".as_slice().into(),
            link_form: crate::terminal::LinkForm::Bare,
            front: if with_front {
                Some(crate::host::Front {
                    input: Fd::new(5),
                    link_output: Fd::new(6),
                    lines: Fd::new(7),
                    link: skein_lib::Secret::new(bytes::copy_of(LOCAL_URL)),
                })
            } else {
                None
            },
            first_link: Some(link),
            after_block: Box::new([]),
            log_lines: Box::new([]),
            parent: None,
            program: bytes::copy_of(b"process_fixture"),
            environment: Box::new([bytes::copy_of(b"HOME=/box"), bytes::copy_of(canary)]),
            serve: bytes::copy_of(b"serve"),
            listeners: Box::new([]),
            root: Fd::new(3),
            echo: Echo::Steps,
            data: bytes::copy_of(b"/data"),
            signals: Fd::new(4),
            stdout: Fd::new(1),
            stderr: Fd::new(2),
        };
        let expected = alloc::fmt::format(format_args!(
            "Settings {{ program: {:?}, parent: None, environment_entries: 2, serve: {:?}, listeners: {:?}, root: {:?}, echo: {:?}, data: {:?}, log_lines: [], first_link: Some(FirstLink {{ .. }}), front: {}, link_form: Bare, word: {:?}, after_block_bytes: 0, signals: {:?}, stdout: {:?}, stderr: {:?} }}",
            settings.program,
            settings.serve,
            settings.listeners,
            settings.root,
            settings.echo,
            settings.data,
            if with_front { "Some(Front { input: Fd(5), .. })" } else { "None" },
            settings.word,
            settings.signals,
            settings.stdout,
            settings.stderr,
        ));
        let settings_debug = alloc::fmt::format(format_args!("{settings:?}"));
        let service = Service::new(
            &crate::limits::small(),
            temper_forge_setup::Steps { always: Box::new([long(b"serve", &[b"never"])]), asked: Box::new([]) },
            settings,
        )
        .expect("valid limits");
        let service_debug = alloc::fmt::format(format_args!("{service:?}"));
        let canary_debug = alloc::fmt::format(format_args!("{canary:?}"));
        let home_debug = alloc::fmt::format(format_args!("{:?}", b"HOME=/box"));
        for debug in [&settings_debug, &service_debug] {
            assert!(!debug.contains(&alloc::fmt::format(format_args!("{FIRST_URL:?}"))));
            assert!(!debug.contains(&alloc::fmt::format(format_args!("{FIRST_SECRET:?}"))));
            assert!(!debug.contains(&alloc::fmt::format(format_args!("{LOCAL_URL:?}"))));
            assert!(!debug.contains(&alloc::fmt::format(format_args!("{LOCAL_SECRET:?}"))));
            assert!(!debug.contains("canary-7f3a9c"));
            assert!(!debug.contains(&canary_debug));
            assert!(!debug.contains("HOME=/box"));
            assert!(!debug.contains(&home_debug));
            assert!(debug.contains("environment_entries: 2"));
        }
        assert!(settings_debug == expected, "Settings Debug exposes only safe fields");
    }
}

// protocol/setup/box.md, 2: fetch progress belongs to the pipe and private log.
const FETCH_LINE: &[u8] = b"fetched forgejo 1 of 2 bytes\n";
const FETCH_STDERR: &[u8] = b"download refused\n";
fn fetch_harness(echo: crate::host::Echo, buffer: u32, items: &[skein_fake_machine::Item]) -> Harness {
    Harness::configured_echo(super::fixtures::fetching_steps(), items, true, buffer, &crate::limits::small(), echo)
}
fn asked_fetch() -> Harness {
    let mut h = fetch_harness(crate::host::Echo::None, skein_sim::Config::calm().buffer, &[]);
    h.run_until_idle();
    h.ask(b"start 3\n");
    h.run_until_idle();
    h
}
fn finish_child(h: &mut Harness, name: &[u8], code: u8) {
    let pid = h.hosted.get(h.child_index(name)).unwrap().pid;
    h.sim.finish_service(pid, Exit::Code(code));
    h.run_until_idle();
}
fn fetch_log(h: &mut Harness) -> Box<[u8]> {
    let root = h.machine.reopen_root(0);
    let fd = h.machine.open(root, b"box/logs/fetch-3.log", skein_fake_machine::How::Read).unwrap();
    let log = h.machine.read(fd, 0, 65_536).unwrap().into_boxed_slice();
    h.machine.close(fd);
    h.machine.close(root);
    log
}
fn stop_fetch_box(h: &mut Harness) {
    h.terminate();
    assert_eq!(h.run_to_end(), Exit::Code(0));
    h.sim.assert_quiescent(h.pid);
    h.sim.assert_no_open_fds(h.pid);
    assert_eq!(h.machine.open_handles(), 0);
}
fn receive_replies(h: &mut Harness, expected: &[u8]) {
    let mut received = List::with_capacity(u32::try_from(expected.len()).unwrap());
    for _ in 0..expected.len() {
        if usize::try_from(received.len()).unwrap() == expected.len() {
            break;
        }
        let remaining = u32::try_from(expected.len()).unwrap().checked_sub(received.len()).unwrap();
        for byte in h.read_reply(remaining) {
            received.push(byte).unwrap();
        }
        assert_eq!(received.as_slice(), expected.get(..usize::try_from(received.len()).unwrap()).unwrap());
        h.run_until_idle();
    }
    assert_eq!(received.as_slice(), expected);
}
#[test]
fn an_asked_fetch_relays_its_lines_between_its_steps() {
    let mut h = asked_fetch();
    let child = h.child_index(b"fetch 3");
    h.write_stdout(child, FETCH_LINE);
    h.write_stdout(child, b"fetched runner 7 bytes\n");
    finish_child(&mut h, b"fetch 3", 0);
    assert_eq!(
        h.reply_bytes().as_ref(),
        b"step 3 fetch\nfetched 3 forgejo 1 2\nfetched 3 runner 7 none\nstep 3 prepare\nstep 3 run\nstarted 3\n"
    );
    let log = fetch_log(&mut h);
    assert!(Harness::contains(&log, FETCH_LINE));
    assert!(Harness::contains(&log, b"fetched runner 7 bytes\n"));
    stop_fetch_box(&mut h);
}
#[test]
fn no_terminal_shows_a_fetch_line() {
    use crate::host::Echo;
    // A normal log and a refused log each run with the default and --echo steps.
    for (echo, refused) in [(Echo::None, false), (Echo::Steps, false), (Echo::None, true), (Echo::Steps, true)] {
        let mut h = fetch_harness(echo, skein_sim::Config::calm().buffer, &[]);
        if refused {
            h.refuse_log = Some(bytes::copy_of(b"fetch-3.log"));
        }
        h.run_until_idle();
        h.ask(b"start 3\n");
        h.run_until_idle();
        let child = h.child_index(b"fetch 3");
        h.write_stdout(child, FETCH_LINE);
        h.write_child(child, 2, FETCH_STDERR);
        h.run_until_idle();
        finish_child(&mut h, b"fetch 3", 7);
        let terminal = h.terminal_bytes();
        assert!(!Harness::contains(&terminal, b"fetched"), "{echo:?}, refused {refused}");
        assert!(Harness::contains(&terminal, FETCH_STDERR));
        if echo == Echo::None && !refused {
            assert!(Harness::contains(
                &terminal,
                b"00:00:00Z     fetch 3: download refused\n00:00:00Z     Log: /data/box/logs/fetch-3.log\n"
            ));
        }
        if !refused {
            assert!(Harness::contains(&fetch_log(&mut h), FETCH_LINE));
        }
        stop_fetch_box(&mut h);
    }
}
#[test]
fn a_fetch_line_left_in_the_pipe_goes_no_further() {
    let mut h = asked_fetch();
    let child = h.child_index(b"fetch 3");
    h.write_child(child, 1, FETCH_LINE);
    finish_child(&mut h, b"fetch 3", 0);
    assert_eq!(h.reply_bytes().as_ref(), b"step 3 fetch\nstep 3 prepare\nstep 3 run\nstarted 3\n");
    assert!(!Harness::contains(&h.terminal_bytes(), b"fetched"));
    assert!(!Harness::contains(&h.stderr(), b"unknown line"));
    assert!(Harness::contains(&fetch_log(&mut h), FETCH_LINE));
    stop_fetch_box(&mut h);
}
#[test]
fn a_line_it_cannot_read_is_named() {
    let mut h = asked_fetch();
    let child = h.child_index(b"fetch 3");
    let unknown = b"fetched forgejo 3 of 2 bytes\n";
    h.write_stdout(child, unknown);
    assert!(Harness::contains(&h.stderr(), b"box: fetch: unknown line\n"));
    assert_eq!(h.reply_bytes().as_ref(), b"step 3 fetch\n");
    assert!(Harness::contains(&fetch_log(&mut h), unknown));
    stop_fetch_box(&mut h);
}
#[test]
fn a_started_group_s_fetch_line_is_named_and_dropped() {
    let mut h = asked_fetch();
    finish_child(&mut h, b"fetch 3", 0);
    finish_child(&mut h, b"forgejo 3", 1);
    h.sim.advance_to(h.sim.now().saturating_add(crate::limits::small().backoff));
    h.run_until_idle();
    let child = h.child_index(b"fetch 3");
    h.write_stdout(child, FETCH_LINE);
    assert!(Harness::contains(&h.stderr(), b"box: fetch: unknown line\n"));
    assert!(!Harness::contains(&h.reply_bytes(), b"fetched"));
    assert!(Harness::contains(&fetch_log(&mut h), FETCH_LINE));
    stop_fetch_box(&mut h);
}
#[test]
fn a_stalled_pipe_keeps_the_latest_and_every_reply() {
    // Three group steps give five reply slots (host::reply_entries).
    // Sixteen progress lines exceed that queue while an eight-byte pipe stalls.
    const PROGRESS_LINES: u64 = 16;
    const PIPE_BYTES: u32 = 8;
    let mut h = fetch_harness(crate::host::Echo::None, PIPE_BYTES, &[]);
    h.run_until_idle();
    h.ask(b"start 3\n");
    h.run_until_idle();
    let child = h.child_index(b"fetch 3");
    for done in 1..=PROGRESS_LINES {
        let line = crate::lines::joined(&[b"fetched runner ", skein_lib::Decimal::of(done).as_bytes(), b" bytes\n"]);
        h.write_stdout(child, &line);
    }
    receive_replies(&mut h, b"step 3 fetch\nfetched 3 runner 16 none\n");
    // The first new figure is in flight; only the latest waits behind it.
    for done in PROGRESS_LINES + 1..=PROGRESS_LINES * 2 {
        let line = crate::lines::joined(&[b"fetched runner ", skein_lib::Decimal::of(done).as_bytes(), b" bytes\n"]);
        h.write_stdout(child, &line);
    }
    finish_child(&mut h, b"fetch 3", 7);
    receive_replies(&mut h, b"fetched 3 runner 17 none\nfailed 3 fetch 3 exited 7\n");
    h.ask(b"stop 3\n");
    h.run_until_idle();
    receive_replies(&mut h, b"stopped 3\n");
    assert!(Harness::contains(&fetch_log(&mut h), b"fetched runner 32 bytes\n"));
    stop_fetch_box(&mut h);
}

#[test]
fn owned_hands_wait_for_the_log_slot_keep_their_time_and_never_echo() {
    use crate::host::Echo;
    use skein_lib::Wall;
    use skein_shell::Host;
    for echo in [Echo::None, Echo::Steps] {
        let mut steps = grouped_steps();
        steps.asked[0].recorded = false;
        steps.asked[0].steps = Box::new([(temper_forge_setup::Stage::Run, long(b"forgejo 3", &[b"never"]))]);
        steps.asked[0].steps[0]
            .1
            .set_started_notice(b"Forgejo: api http://127.0.0.1:3000/api/v1\nForgejo: person ada\n".as_slice().into());
        let mut h = Harness::configured_echo(steps, &[], true, 16, &crate::limits::small(), echo);
        h.run_until_idle();
        h.write_child(h.serve_index(), 1, b"http://first\n");
        h.run_until_idle();
        h.wall = Wall::from_nanos(3_723_000_000_000);
        h.hold_spawn = Some(b"forgejo 3".as_slice().into());
        h.ask(b"start 3\n");
        h.run_until_idle();
        assert!(h.held_spawn.is_some());
        h.hold_log_write = true;
        h.write_child(h.serve_index(), 2, b"occupied\n");
        h.run_until_idle();
        assert!(h.held_log_write.is_some());
        h.release_spawn();
        h.run_until_idle();
        assert!(Harness::contains(&h.log(), b"01:02:03Z Forge 3 (Forgejo): starting\n"));
        assert!(!Harness::contains(&h.log(), b"Forge 3 (Forgejo): started\n"), "the busy log holds the line");
        receive_replies(&mut h, b"step 3 run\nstarted 3\n");
        assert!(!h.service.work_pending(h.sim.now()), "a busy log waits for its completion");
        // Serve's sign-in channel progresses while the diagnostic log is busy.
        h.write_child(h.serve_index(), 1, b"http://fresh\n");
        h.run_until_idle();
        assert!(Harness::contains(&h.stdout(), b"http://fresh\n"));
        h.wall = Wall::from_nanos(7_200_000_000_000);
        h.release_log_write();
        h.run_until_idle();
        let root = h.machine.reopen_root(0);
        let fd = h.machine.open(root, b"box/logs/forgejo-3.log", skein_fake_machine::How::Read).unwrap();
        let log = h.machine.read(fd, 0, 4096).unwrap();
        assert_eq!(log, b"1970-01-01T01:02:03Z Forgejo: api http://127.0.0.1:3000/api/v1\n1970-01-01T01:02:03Z Forgejo: person ada\n");
        h.machine.close(fd);
        h.machine.close(root);
        let output = h.terminal_bytes();
        assert_eq!(bytes::count(&h.log(), b"01:02:03Z Forge 3 (Forgejo): started\n", 2), 1);
        assert!(!Harness::contains(&output, b"Forgejo: api "));
        assert!(!Harness::contains(&output, b"Forgejo: person "));
        h.ask(b"start 3\n");
        h.run_until_idle();
        assert_eq!(bytes::count(&h.log(), b"Forge 3 (Forgejo): started\n", 2), 1, "a reask is wire-only");
        h.terminate();
        assert_eq!(h.run_to_end(), Exit::Code(0));
        h.sim.assert_quiescent(h.pid);
        h.sim.assert_no_open_fds(h.pid);
        assert_eq!(h.machine.open_handles(), 0);
    }
}

#[test]
fn bound_kind_labels_follow_the_number_through_start_stop_and_failure() {
    for label in [b"Forge $forge (Forgejo)".as_slice(), b"Playground"] {
        for forge in [17, u16::MAX] {
            let mut steps = super::fixtures::kind_steps();
            steps.asked[0].label = label.into();
            let mut h = Harness::grouped(steps, &[], true);
            h.run_until_idle();
            h.write_child(h.serve_index(), 1, b"http://first\n");
            h.run_until_idle();
            let number = skein_lib::Decimal::of(u64::from(forge));
            let start = crate::lines::joined(&[b"start ", number.as_bytes(), b" fake\n"]);
            h.ask(&start);
            h.run_until_idle();
            h.ask(&crate::lines::joined(&[b"stop ", number.as_bytes(), b"\n"]));
            h.run_until_idle();
            finish_child(&mut h, b"fake-forge", 0);
            let expected = if label == b"Playground" {
                bytes::copy_of(b"Playground")
            } else {
                crate::lines::joined(&[b"Forge ", number.as_bytes(), b" (Forgejo)"])
            };
            for suffix in [b": starting\n".as_slice(), b": started\n", b": stopped\n"] {
                assert!(Harness::contains(&h.log(), &crate::lines::joined(&[b"00:00:00Z ", &expected, suffix])));
            }
            h.hold_spawn = Some(b"fake-forge".as_slice().into());
            h.ask(&start);
            h.run_until_idle();
            h.fail_spawn();
            h.run_until_idle();
            assert!(Harness::contains(
                &h.terminal_bytes(),
                &crate::lines::joined(&[&expected, b": failed, fake-forge-", number.as_bytes(), b" did not start\n"])
            ));
            stop_fetch_box(&mut h);
        }
    }
}

#[test]
fn each_link_reads_on_stdout_as_its_form_has_it() {
    use crate::terminal::LinkForm;
    for (form, first, fresh) in [
        (LinkForm::Bare, b"http://local/first\n".as_slice(), b"http://fresh/link\n".as_slice()),
        (
            LinkForm::Osc8,
            b"\x1b]8;;http://local/first\x1b\\http://local/first\x1b]8;;\x1b\\\n".as_slice(),
            b"\x1b]8;;http://fresh/link\x1b\\http://fresh/link\x1b]8;;\x1b\\\n".as_slice(),
        ),
    ] {
        let mut harness = Harness::with_terminal(
            temper_forge_setup::Steps { always: Box::new([long(b"serve", &[b"never"])]), asked: Box::new([]) },
            &[],
            true,
            16_384,
            &crate::limits::small(),
            crate::host::Echo::None,
            super::fixtures::Opening {
                parent: None,
                link_form: form,
                log_lines: Box::new([]),
                front: None,
                first_link: Some(crate::host::FirstLink {
                    url: bytes::copy_of(FIRST_URL),
                    local: None,
                    secret: skein_lib::Secret::new(bytes::copy_of(FIRST_SECRET)),
                }),
                after_block: bytes::copy_of(AFTER_BLOCK),
            },
        );
        harness.run_until_idle();
        assert_eq!(harness.stdout().as_ref(), first);
        let serve = harness.child_index(b"serve");
        harness.write_child(serve, 1, b"http://fresh");
        harness.run_until_idle();
        assert!(harness.stdout().is_empty());
        harness.write_child(serve, 1, b"/link\n");
        harness.run_until_idle();
        assert_eq!(harness.stdout().as_ref(), fresh);
        assert_eq!(
            harness.terminal_bytes(),
            crate::lines::joined(&[crate::terminal::PRELUDE, first, FIRST_FOOTER, AFTER_BLOCK, b"\n", fresh,])
        );
        stop_first_link(&mut harness);
    }
}
#[test]
fn the_widest_link_is_whole_on_a_terminal_and_one_byte_more_is_refused() {
    for overflow in [false, true] {
        let mut harness = Harness::with_terminal(
            temper_forge_setup::Steps { always: Box::new([long(b"serve", &[b"never"])]), asked: Box::new([]) },
            &[],
            true,
            16_384,
            &crate::limits::small(),
            crate::host::Echo::None,
            super::fixtures::Opening {
                parent: None,
                link_form: crate::terminal::LinkForm::Osc8,
                log_lines: Box::new([]),
                front: None,
                first_link: None,
                after_block: Box::new([]),
            },
        );
        harness.run_until_idle();
        let serve = harness.child_index(b"serve");
        let cap = super::fixtures::line_cap();
        harness.write_child(serve, 1, &cap);
        harness.run_until_idle();
        assert!(harness.stdout().is_empty(), "a partial link prints nothing");
        harness.write_child(serve, 1, if overflow { b"x\n" } else { b"\n" });
        harness.run_until_idle();
        if overflow {
            assert!(harness.stdout().is_empty(), "none of an oversized URL is printed");
            harness.write_child(serve, 1, b"good\n");
            harness.run_until_idle();
            assert_eq!(harness.stdout().as_ref(), b"\x1b]8;;good\x1b\\good\x1b]8;;\x1b\\\n");
            assert!(Harness::contains(&harness.terminal_bytes(), b"box: serve: sign-in link too long\nOpen this link"));
        } else {
            let stdout = harness.stdout();
            assert_eq!(stdout.len(), 8207);
            assert_eq!(stdout, crate::lines::joined(&[b"\x1b]8;;", &cap, b"\x1b\\", &cap, b"\x1b]8;;\x1b\\\n"]));
        }
        stop_first_link(&mut harness);
    }
}

fn source_steps() -> temper_forge_setup::Steps {
    let mut steps = grouped_steps();
    let mut group = List::with_capacity(u32::try_from(steps.asked[0].steps.len().checked_add(1).unwrap()).unwrap());
    for step in &steps.asked[0].steps {
        group.push(step.clone()).unwrap();
    }
    group
        .push((
            temper_forge_setup::Stage::Source,
            Step::new(
                b"source 3".as_slice().into(),
                Box::new([b"never".as_slice().into(), b"source 3".as_slice().into()]),
                temper_forge_setup::Kind::Source,
            )
            .ending_ignored(),
        ))
        .unwrap();
    steps.asked[0].steps = group.into_boxed();
    steps
}
#[test]
fn source_outcomes_precede_started_are_replayed_and_failures_are_logged_once() {
    for line in [b"source added\n".as_slice(), b"source failed discovery unavailable\n"] {
        let mut h = Harness::grouped(source_steps(), &[], true);
        h.run_until_idle();
        h.ask(b"start 3\n");
        h.run_until_idle();
        let child = h.child_index(b"source 3");
        h.write_stdout(child, line);
        finish_child(&mut h, b"source 3", 0);
        let outcome = crate::pipe::source(3, &temper_forge_setup::sign_in::Source::read(line).unwrap());
        let ending = crate::lines::joined(&[&outcome, b"started 3\n"]);
        assert!(h.reply_bytes().ends_with(&ending));
        assert!(!Harness::contains(&h.reply_bytes(), b"failed 3"));
        h.ask(b"start 3\n");
        h.run_until_idle();
        assert!(h.reply_bytes().ends_with(&crate::lines::joined(&[&ending, &ending])));
        let count = bytes::count(&h.log(), b"no sign-in through temper:", u32::MAX);
        assert_eq!(count, u32::from(line.starts_with(b"source failed")));
        assert_eq!(h.spawn_times(b"source 3").len(), 1, "another start does not bypass the retry wait");
        if line.starts_with(b"source failed") {
            h.sim.advance_to(h.sim.now().saturating_add(crate::limits::small().backoff));
            h.run_until_idle();
            let child = h.child_index(b"source 3");
            h.write_stdout(child, b"source added\n");
            finish_child(&mut h, b"source 3", 0);
            assert!(h.reply_bytes().ends_with(b"source 3 added\n"));
            assert_eq!(h.spawn_times(b"source 3").len(), 2);
        }
        stop_fetch_box(&mut h);
    }
}

#[test]
fn a_failed_source_retries_with_doubling_waits_and_says_only_changed_endings() {
    use skein_shell::Host;
    let mut h = Harness::grouped(source_steps(), &[], true);
    h.run_until_idle();
    h.ask(b"start 3\n");
    h.run_until_idle();
    for attempt in 0..3 {
        let child = h.child_index(b"source 3");
        h.write_stdout(child, if attempt == 2 { b"source added\n" } else { b"source failed discovery unavailable\n" });
        finish_child(&mut h, b"source 3", 0);
        if attempt == 0 {
            assert!(h.reply_bytes().ends_with(b"source 3 failed discovery unavailable\nstarted 3\n"));
        }
        if attempt < 2 {
            let before = h.sim.now();
            let delay = crate::limits::small().backoff.saturating_mul(if attempt == 0 { 1 } else { 2 });
            assert_eq!(h.service.next_policy_deadline(), Some(before.saturating_add(delay)));
            h.sim.advance_to(skein_lib::Time::from_nanos(
                before.saturating_add(delay).as_nanos().checked_sub(1).unwrap(),
            ));
            h.run_until_idle();
            assert_eq!(h.spawn_times(b"source 3").len(), attempt + 1);
            h.sim.advance_to(before.saturating_add(delay));
            h.run_until_idle();
            assert_eq!(h.spawn_times(b"source 3").len(), attempt + 2);
        }
    }
    let replies = h.reply_bytes();
    assert!(replies.ends_with(b"source 3 added\n"), "{replies:?}");
    assert_eq!(bytes::count(&replies, b"source 3 failed", u32::MAX), 1, "same retry ending is silent");
    assert_eq!(bytes::count(&replies, b"started 3\n", u32::MAX), 1, "only first ending completes start");
    assert_eq!(bytes::count(&h.log(), b"no sign-in through temper:", u32::MAX), 1);
    assert!(h.service.next_policy_deadline().is_none(), "added disarms retries");
    h.sim.advance_to(h.sim.now().saturating_add(crate::limits::small().backoff_max));
    h.ask(b"start 3\n");
    h.run_until_idle();
    assert!(h.reply_bytes().ends_with(b"source 3 added\nstarted 3\n"));
    assert_eq!(h.spawn_times(b"source 3").len(), 3, "added never runs again in this group");
    stop_fetch_box(&mut h);
}

#[test]
fn a_restored_group_waits_for_its_source_before_reporting_started() {
    use skein_fake_machine::Item;
    let items =
        [Item::directory(b"box"), Item::directory(b"box/started"), Item::file(b"box/started/3", b"").mode(0o600)];
    let mut steps = source_steps();
    steps.asked[0].before = 1;
    let mut h = Harness::grouped(steps, &items, true);
    h.run_until_idle();
    assert_eq!(h.spawn_times(b"source 3").len(), 1, "the restored group starts without an ask");
    h.ask(b"start 3\n");
    h.run_until_idle();
    assert!(!Harness::contains(&h.reply_bytes(), b"started 3"), "the source has not answered");
    assert!(!Harness::contains(&h.log(), b"no sign-in through temper"), "no premature source failure");
    let child = h.child_index(b"source 3");
    h.write_stdout(child, b"source added\n");
    finish_child(&mut h, b"source 3", 0);
    assert!(h.reply_bytes().ends_with(b"source 3 added\nstarted 3\n"));
    assert!(!Harness::contains(&h.log(), b"no sign-in through temper"));
    stop_fetch_box(&mut h);
}
#[test]
fn a_silent_source_ending_gets_the_boxs_failure_line_and_still_starts() {
    for exit in [Exit::Code(0), Exit::Code(7), Exit::Signal(9)] {
        let mut h = Harness::grouped(source_steps(), &[], true);
        h.run_until_idle();
        h.ask(b"start 3\n");
        h.run_until_idle();
        let pid = h.hosted.get(h.child_index(b"source 3")).unwrap().pid;
        h.sim.finish_service(pid, exit);
        h.run_until_idle();
        assert!(Harness::contains(&h.reply_bytes(), b"source 3 failed source "));
        assert!(h.reply_bytes().ends_with(b"started 3\n"));
        assert_eq!(bytes::count(&h.log(), b"no sign-in through temper:", u32::MAX), 1);
        assert_eq!(h.spawn_times(b"source 3").len(), 1);
        stop_fetch_box(&mut h);
    }
}

#[test]
fn a_source_that_never_started_has_a_logged_outcome_and_never_fails_the_group() {
    let mut h = Harness::grouped(source_steps(), &[], true);
    h.run_until_idle();
    h.hold_spawn = Some(b"source 3".as_slice().into());
    h.ask(b"start 3\n");
    h.run_until_idle();
    assert!(h.held_spawn.is_some());
    h.fail_spawn();
    h.run_until_idle();
    assert!(h.reply_bytes().ends_with(b"source 3 failed source did not start\nstarted 3\n"));
    assert_eq!(bytes::count(&h.log(), b"no sign-in through temper:", u32::MAX), 1);
    assert_eq!(h.spawn_times(b"source 3").len(), 1);
    stop_fetch_box(&mut h);
}
