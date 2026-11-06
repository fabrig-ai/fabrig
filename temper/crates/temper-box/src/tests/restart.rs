//! Deliver the abort's real simulator completions in every endpoint order.
use super::fixtures::{Harness, call, long, socket};
use crate::host::pressure::restart_gate;
use skein_io::kernel::{Done, Exit, Fd, Op};
use skein_lib::{Duration, List, bytes};
use skein_shell::Host;
use skein_sim::{Event, Pid, Summary};

// Four child pipes, the ready probe, and the process exit: 6! delivery orders.
const ORDERS: u32 = 720;
const RESTART: Duration = Duration::from_millis(500);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Ending {
    Stdout,
    Stderr,
    Requests,
    Replies,
    Ready,
    Exited,
}

fn next_order(order: &mut [Ending; 6]) -> bool {
    let mut pivot = order.len().checked_sub(1).unwrap();
    while pivot > 0 {
        let before = pivot.checked_sub(1).unwrap();
        if order.get(before).unwrap() < order.get(pivot).unwrap() {
            break;
        }
        pivot = before;
    }
    if pivot == 0 {
        return false;
    }
    let before = pivot.checked_sub(1).unwrap();
    let mut next = order.len().checked_sub(1).unwrap();
    while order.get(next).unwrap() <= order.get(before).unwrap() {
        next = next.checked_sub(1).unwrap();
    }
    let lower = *order.get(before).unwrap();
    let higher = *order.get(next).unwrap();
    *order.get_mut(before).unwrap() = higher;
    *order.get_mut(next).unwrap() = lower;
    order.get_mut(pivot..).unwrap().reverse();
    true
}

fn probing_serve() -> (Harness, Pid, Fd, Fd) {
    let address = ([127, 0, 0, 1], 3000).into();
    let mut h = Harness::with_stubborn(&[long(b"serve", &[b"never"]).with_ready(address)], true);
    let peer = h.sim.spawn_process();
    let listener = socket(&mut h.sim, peer);
    assert_eq!(call(&mut h.sim, peer, Op::Bind { fd: listener, addr: address }).result, Ok(Done::Bound(address)));
    assert_eq!(call(&mut h.sim, peer, Op::Listen { fd: listener, backlog: 1 }).result, Ok(Done::Nothing));
    h.run_until_idle();
    let Ok(Done::Accepted { fd, .. }) = call(&mut h.sim, peer, Op::Accept { fd: listener }).result else {
        panic!("the version probe connects");
    };
    assert_eq!(call(&mut h.sim, peer, Op::Close { fd: listener }).result, Ok(Done::Nothing));
    let mut probe = None;
    for entry in h.sim.trace() {
        if entry.pid == h.pid
            && let Event::Submit { kind: Summary::Connect { fd, .. }, .. } = entry.event
        {
            probe = Some(fd);
        }
    }
    (h, peer, fd, probe.expect("box's version connection"))
}

fn abort_endings(busy: bool, early_reply: bool) {
    let mut order = [Ending::Stdout, Ending::Stderr, Ending::Requests, Ending::Replies, Ending::Ready, Ending::Exited];
    let mut failures = List::with_capacity(ORDERS);
    let mut rows = 0_u32;
    loop {
        let (mut h, peer, ready_peer, probe) = probing_serve();
        let serve = h.serve_index();
        let child = h.hosted.get(serve).unwrap();
        let pid = child.pid;
        let requests = child.output.unwrap();
        let replies = child.input.unwrap();
        let stdout = child.stdio[1].unwrap();
        let stderr = child.stdio[2].unwrap();
        // Spawn declares stdout, stderr, requests, replies in that order.
        let fds = [child.parent_pipes[0], child.parent_pipes[1], child.parent_pipes[2], child.parent_pipes[3], probe];
        // A write failure is how a replies pipe reports its peer's early end.
        // Retain it while the request is still readable, before holding reads.
        if early_reply {
            h.endings.fds[3] = Some(fds[3]);
            h.close_child_fd(serve, replies);
            h.ask(b"start 99\n");
            h.run_until_idle();
        } else {
            h.close_child_fd(serve, replies);
        }
        if busy {
            h.hold_log_write = true;
            h.write_child(serve, 2, b"diagnostic in flight\n");
            h.run_until_idle();
            assert!(h.held_log_write.is_some(), "the log write straddles the abort");
        }
        for (slot, fd) in h.endings.fds.iter_mut().zip(fds) {
            *slot = Some(fd);
        }
        h.endings.exit = true;
        for fd in [requests, stdout, stderr] {
            h.close_child_fd(serve, fd);
        }
        assert_eq!(call(&mut h.sim, peer, Op::Close { fd: ready_peer }).result, Ok(Done::Nothing));
        let aborted = h.sim.now();
        h.sim.finish_service(pid, Exit::Signal(6));
        h.run_until_idle();
        for ending in order {
            match ending {
                Ending::Stdout => h.endings.fds[0] = None,
                Ending::Stderr => h.endings.fds[1] = None,
                Ending::Requests => h.endings.fds[2] = None,
                Ending::Replies => h.endings.fds[3] = None,
                Ending::Ready => h.endings.fds[4] = None,
                Ending::Exited => h.endings.exit = false,
            }
            h.release_endings();
            h.run_until_idle();
        }
        if busy {
            h.release_log_write();
            h.run_until_idle();
        }
        h.sim.advance_to(aborted.saturating_add(RESTART));
        h.run_until_idle();
        let gate = restart_gate(&h.service);
        assert!(
            !gate.open() || h.service.work_pending(h.sim.now()),
            "the box would wait with its restart gate open: {gate:?}"
        );
        let spawns = h.spawn_times(b"serve");
        if spawns.len() != 2 || spawns[1] > aborted.saturating_add(RESTART) {
            failures
                .push((
                    order,
                    restart_gate(&h.service),
                    h.service.work_pending(h.sim.now()),
                    h.service.next_deadline(),
                    // Initial Starting, replacement Starting, one excess to detect a duplicate.
                    bytes::count(&h.log(), b"Starting serve\n", 3),
                ))
                .unwrap();
        } else {
            h.terminate();
            assert_eq!(h.run_to_end(), Exit::Code(0));
            h.sim.assert_quiescent(h.pid);
            h.sim.assert_no_open_fds(h.pid);
            h.sim.assert_no_open_fds(peer);
            assert_eq!(h.machine.open_handles(), 0);
        }
        rows = rows.checked_add(1).unwrap();
        if !next_order(&mut order) {
            break;
        }
    }
    assert_eq!(rows, ORDERS);
    assert!(
        failures.is_empty(),
        "{} of {ORDERS} ending orders stalled (busy log={busy}); first: {:?}",
        failures.len(),
        failures.get(0)
    );
}

#[test]
fn an_aborted_serve_restarts_in_every_ending_order() {
    abort_endings(false, false);
}

#[test]
fn an_aborted_serve_restarts_after_a_busy_log() {
    abort_endings(true, false);
}

#[test]
fn an_aborted_serve_restarts_after_early_reply_pipe_failure() {
    abort_endings(false, true);
}

#[test]
fn early_reply_pipe_failure_and_busy_logs_allow_restarting() {
    abort_endings(true, true);
}

#[test]
fn a_group_file_with_its_record_starts_at_the_box_start() {
    use skein_fake_machine::Item;
    let items = [
        Item::directory(b"box"),
        Item::directory(b"box/groups"),
        Item::directory(b"box/started"),
        Item::file(b"box/groups/17", b"fake\n"),
        Item::file(b"box/started/17", b""),
    ];
    let mut h = Harness::grouped(super::fixtures::kind_steps(), &items, true);
    h.run_until_idle();
    assert_eq!(h.spawn_times(b"fake-forge").len(), 1);
    assert_eq!(h.spawn_times(b"remove").len(), 0);
    assert_eq!(h.spawn_times(b"serve").len(), 1);
    let mut seen = false;
    for launch in &h.launches {
        if launch.name.as_ref() == b"fake-forge" {
            assert_eq!(launch.args.get(1).unwrap().as_ref(), b"17");
            seen = true;
        }
    }
    assert!(seen);
    h.terminate();
    assert_eq!(h.run_to_end(), Exit::Code(0));
    h.sim.assert_quiescent(h.pid);
    h.sim.assert_no_open_fds(h.pid);
    assert_eq!(h.machine.open_handles(), 0);
}
#[test]
fn a_group_file_without_its_record_stays_dormant() {
    use skein_fake_machine::Item;
    let items = [Item::directory(b"box"), Item::directory(b"box/groups"), Item::file(b"box/groups/17", b"fake\n")];
    let mut h = Harness::grouped(super::fixtures::kind_steps(), &items, true);
    h.run_until_idle();
    assert_eq!(h.spawn_times(b"fake-forge").len(), 0);
    assert_eq!(h.spawn_times(b"remove").len(), 0);
    assert_eq!(h.spawn_times(b"serve").len(), 1);
    h.ask(b"remove 17\n");
    h.run_until_idle();
    assert_eq!(h.reply_bytes().as_ref(), b"removed 17\n");
    h.terminate();
    assert_eq!(h.run_to_end(), Exit::Code(0));
    h.sim.assert_quiescent(h.pid);
    h.sim.assert_no_open_fds(h.pid);
    assert_eq!(h.machine.open_handles(), 0);
}
#[test]
fn two_started_files_of_one_kind_start_the_higher_number_with_a_notice() {
    use skein_fake_machine::Item;
    let items = [
        Item::directory(b"box"),
        Item::directory(b"box/groups"),
        Item::directory(b"box/started"),
        Item::file(b"box/groups/3", b"fake\n"),
        Item::file(b"box/started/3", b""),
        Item::file(b"box/groups/17", b"fake\n"),
        Item::file(b"box/started/17", b""),
    ];
    let mut h = Harness::grouped(super::fixtures::kind_steps(), &items, true);
    h.run_until_idle();
    assert_eq!(h.spawn_times(b"fake-forge").len(), 1);
    assert!(Harness::contains(&h.stderr(), b"box: forge 3: a second fake group is not started\n"));
    for launch in &h.launches {
        if launch.name.as_ref() == b"fake-forge" {
            assert_eq!(launch.args.get(1).unwrap().as_ref(), b"17");
        }
    }
    h.terminate();
    assert_eq!(h.run_to_end(), Exit::Code(0));
    h.sim.assert_quiescent(h.pid);
    h.sim.assert_no_open_fds(h.pid);
    assert_eq!(h.machine.open_handles(), 0);
}

#[test]
fn group_file_listing_is_refused_above_its_configured_bound() {
    use skein_fake_machine::Item;
    let items = [
        Item::directory(b"box"),
        Item::directory(b"box/groups"),
        Item::file(b"box/groups/3", b"fake\n"),
        Item::file(b"box/groups/17", b"fake\n"),
    ];
    for bound in [0, 1] {
        let limits = crate::limits::Limits { groups: bound, ..crate::limits::small() };
        let mut h =
            Harness::configured(super::fixtures::kind_steps(), &items, true, skein_sim::Config::calm().buffer, &limits);
        h.run_until_idle();
        assert_eq!(h.spawn_times(b"fake-forge").len(), 0);
        assert!(Harness::contains(&h.stderr(), b"box.groups"));
        h.terminate();
        assert_eq!(h.run_to_end(), Exit::Code(0));
        h.sim.assert_quiescent(h.pid);
        h.sim.assert_no_open_fds(h.pid);
        assert_eq!(h.machine.open_handles(), 0);
    }
}
