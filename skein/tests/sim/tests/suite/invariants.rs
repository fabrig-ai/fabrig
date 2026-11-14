//! Each broken invariant of the contract fails the world, as do the checks
//! the simulator adds at quiescence.

use std::net::{Ipv6Addr, SocketAddr};

use skein_io::kernel::{Done, Fd, Op, Submit};
use skein_sim_tests::{World, local, recv_op};

#[test]
fn an_empty_receive_buffer() {
    let mut world = World::calm();
    let (client, server) = (world.spawn(), world.spawn());
    let (c, _) = world.pair(client, server);
    let submit = Submit { op: world.token(), kind: Op::Recv { fd: c, buf: Box::from(&[][..]) } };
    let finding = world.sim.check_submission(client, &submit).unwrap_err();
    assert!(finding.contains("an invalid record"), "{finding}");
}

#[test]
fn a_send_with_nothing_left() {
    let mut world = World::calm();
    let (client, server) = (world.spawn(), world.spawn());
    let (c, _) = world.pair(client, server);
    let submit = Submit { op: world.token(), kind: Op::Send { fd: c, bytes: Box::from(&b"ab"[..]), from: 2 } };
    let finding = world.sim.check_submission(client, &submit).unwrap_err();
    assert!(finding.contains("an invalid record"), "{finding}");
}

#[test]
fn a_token_reused_while_in_flight() {
    let mut world = World::calm();
    let (client, server) = (world.spawn(), world.spawn());
    let (c, _) = world.pair(client, server);
    let token = world.submit(client, recv_op(c, 4));
    let submit = Submit { op: token, kind: Op::Shutdown { fd: c } };
    let finding = world.sim.check_submission(client, &submit).unwrap_err();
    assert!(finding.contains("already in flight"), "{finding}");
}

#[test]
fn a_token_reused_before_its_completion_is_reaped() {
    let mut world = World::calm();
    let pid = world.spawn();
    let token = world.token();
    world.submit_as(pid, token, Op::Socket { family: skein_io::kernel::Family::Ipv4 });
    let submit = Submit { op: token, kind: Op::Socket { family: skein_io::kernel::Family::Ipv4 } };
    let finding = world.sim.check_submission(pid, &submit).unwrap_err();
    assert!(finding.contains("already in flight"), "{finding}");
}

#[test]
fn a_cancel_of_a_cancel() {
    let faults = skein_sim::Faults {
        cancel_race: 1000,
        latency_max: skein_lib::Duration::from_millis(1),
        ..skein_sim::Faults::NONE
    };
    let mut world = World::new(1, skein_sim::Config { faults, ..skein_sim::Config::calm() });
    let (client, server) = (world.spawn(), world.spawn());
    let (c, _) = world.pair(client, server);
    let recv = world.submit(client, recv_op(c, 4));
    let cancel = world.submit(client, Op::Cancel { target: recv });
    let submit = Submit { op: world.token(), kind: Op::Cancel { target: cancel } };
    let finding = world.sim.check_submission(client, &submit).unwrap_err();
    assert!(finding.contains("which is a Cancel"), "{finding}");
}

#[test]
fn a_bind_of_another_family() {
    let mut world = World::calm();
    let pid = world.spawn();
    let fd = world.socket(pid);
    let submit = Submit { op: world.token(), kind: Op::Bind { fd, addr: SocketAddr::from((Ipv6Addr::LOCALHOST, 0)) } };
    let finding = world.sim.check_submission(pid, &submit).unwrap_err();
    assert!(finding.contains("another family"), "{finding}");
}

#[test]
fn a_connect_to_another_family() {
    let mut world = World::calm();
    let pid = world.spawn();
    let fd = world.socket(pid);
    let submit =
        Submit { op: world.token(), kind: Op::Connect { fd, addr: SocketAddr::from((Ipv6Addr::LOCALHOST, 80)) } };
    let finding = world.sim.check_submission(pid, &submit).unwrap_err();
    assert!(finding.contains("another family"), "{finding}");
}

#[test]
fn two_receives_on_one_descriptor() {
    let mut world = World::calm();
    let (client, server) = (world.spawn(), world.spawn());
    let (c, _) = world.pair(client, server);
    world.submit(client, recv_op(c, 4));
    let submit = Submit { op: world.token(), kind: recv_op(c, 4) };
    let finding = world.sim.check_submission(client, &submit).unwrap_err();
    assert!(finding.contains("a second Recv"), "{finding}");
}

#[test]
fn two_sends_on_one_descriptor() {
    let mut world = World::new(1, skein_sim::Config { buffer: 1, ..skein_sim::Config::calm() });
    let (client, server) = (world.spawn(), world.spawn());
    let (c, _) = world.pair(client, server);
    world.submit(client, Op::Send { fd: c, bytes: Box::from(&b"ab"[..]), from: 0 });
    let submit = Submit { op: world.token(), kind: Op::Send { fd: c, bytes: Box::from(&b"cd"[..]), from: 0 } };
    let finding = world.sim.check_submission(client, &submit).unwrap_err();
    assert!(finding.contains("a second Send"), "{finding}");
}

#[test]
fn two_accepts_on_one_listener() {
    let mut world = World::calm();
    let pid = world.spawn();
    let (listener, _) = world.listener(pid);
    world.submit(pid, Op::Accept { fd: listener });
    let submit = Submit { op: world.token(), kind: Op::Accept { fd: listener } };
    let finding = world.sim.check_submission(pid, &submit).unwrap_err();
    assert!(finding.contains("a second Accept"), "{finding}");
}

#[test]
fn anything_beside_a_connect() {
    let mut world = World::calm();
    let (client, server) = (world.spawn(), world.spawn());
    let listener = world.socket(server);
    let addr = world.bind(server, listener, local(0)).unwrap();
    world.call(server, Op::Listen { fd: listener, backlog: 1 });
    let (first, second) = (world.socket(client), world.socket(client));
    assert_eq!(world.connect(client, first, addr), Ok(Done::Nothing));
    world.submit(client, Op::Connect { fd: second, addr });
    let submit = Submit { op: world.token(), kind: Op::Shutdown { fd: second } };
    let finding = world.sim.check_submission(client, &submit).unwrap_err();
    assert!(finding.contains("beside a Connect"), "{finding}");
}

#[test]
fn anything_but_close_after_a_failed_connect() {
    let mut world = World::calm();
    let pid = world.spawn();
    let fd = world.socket(pid);
    world.connect(pid, fd, local(80)).unwrap_err();
    let submit = Submit { op: world.token(), kind: recv_op(fd, 4) };
    let finding = world.sim.check_submission(pid, &submit).unwrap_err();
    assert!(finding.contains("after a Connect"), "{finding}");
}

#[test]
fn a_shutdown_during_a_send() {
    let mut world = World::new(1, skein_sim::Config { buffer: 1, ..skein_sim::Config::calm() });
    let (client, server) = (world.spawn(), world.spawn());
    let (c, _) = world.pair(client, server);
    world.send(client, c, b"a").unwrap();
    world.submit(client, Op::Send { fd: c, bytes: Box::from(&b"b"[..]), from: 0 });
    let submit = Submit { op: world.token(), kind: Op::Shutdown { fd: c } };
    let finding = world.sim.check_submission(client, &submit).unwrap_err();
    assert!(finding.contains("a Shutdown while a Send"), "{finding}");
}

#[test]
fn a_close_beside_a_receive() {
    let mut world = World::calm();
    let (client, server) = (world.spawn(), world.spawn());
    let (c, _) = world.pair(client, server);
    world.submit(client, recv_op(c, 4));
    let submit = Submit { op: world.token(), kind: Op::Close { fd: c } };
    let finding = world.sim.check_submission(client, &submit).unwrap_err();
    assert!(finding.contains("a Close while another operation"), "{finding}");
}

#[test]
fn a_close_beside_a_completion_not_yet_reaped() {
    let mut world = World::calm();
    let pid = world.spawn();
    let fd = world.socket(pid);
    world.submit(pid, Op::Bind { fd, addr: local(0) });
    let submit = Submit { op: world.token(), kind: Op::Close { fd } };
    let finding = world.sim.check_submission(pid, &submit).unwrap_err();
    assert!(finding.contains("a Close while another operation"), "{finding}");
}

#[test]
fn an_operation_after_close() {
    let mut world = World::calm();
    let pid = world.spawn();
    let fd = world.socket(pid);
    world.close(pid, fd);
    let submit = Submit { op: world.token(), kind: Op::Listen { fd, backlog: 1 } };
    let finding = world.sim.check_submission(pid, &submit).unwrap_err();
    assert!(finding.contains("not open in this process"), "{finding}");
}

#[test]
fn an_operation_on_another_process_descriptor() {
    let mut world = World::calm();
    let (a, b) = (world.spawn(), world.spawn());
    world.socket(a);
    let mine = world.socket(a);
    let submit = Submit { op: world.token(), kind: Op::Close { fd: mine } };
    let finding = world.sim.check_submission(b, &submit).unwrap_err();
    assert!(finding.contains("not open in this process"), "{finding}");
}

#[test]
fn an_operation_on_a_descriptor_never_issued() {
    let mut world = World::calm();
    let pid = world.spawn();
    let submit = Submit { op: world.token(), kind: Op::Close { fd: Fd::new(42) } };
    let finding = world.sim.check_submission(pid, &submit).unwrap_err();
    assert!(finding.contains("not open in this process"), "{finding}");
}

#[test]
fn quiescence_with_an_operation_in_flight() {
    let mut world = World::calm();
    let pid = world.spawn();
    let (listener, _) = world.listener(pid);
    world.submit(pid, Op::Accept { fd: listener });
    let finding = world.sim.check_quiescent(pid).unwrap_err();
    assert!(finding.contains("not quiescent"), "{finding}");
}

#[test]
fn quiescence_with_a_completion_not_reaped() {
    let mut world = World::calm();
    let pid = world.spawn();
    world.submit(pid, Op::Socket { family: skein_io::kernel::Family::Ipv4 });
    let finding = world.sim.check_quiescent(pid).unwrap_err();
    assert!(finding.contains("not quiescent"), "{finding}");
}

#[test]
fn quiescence_with_a_descriptor_open() {
    let mut world = World::calm();
    let pid = world.spawn();
    world.socket(pid);
    world.sim.assert_quiescent(pid);
    let finding = world.sim.check_no_open_fds(pid).unwrap_err();
    assert!(finding.contains("descriptors open"), "{finding}");
}

#[test]
fn a_report_names_the_time_finding_seed_and_trace() {
    let mut world = World::new(77, skein_sim::Config::calm());
    let pid = world.spawn();
    let fd = world.socket(pid);
    world.close(pid, fd);
    let message = world.sim.report("a finding");
    assert!(message.contains(&format!("skein-sim at {} ns, a finding", world.sim.now().as_nanos())), "{message}");
    assert!(message.contains("seed 77"), "{message}");
    assert!(message.contains("Close"), "{message}");
}

#[test]
fn a_listen_on_an_unbound_socket() {
    let mut world = World::calm();
    let pid = world.spawn();
    let fd = world.socket(pid);
    let submit = Submit { op: world.token(), kind: Op::Listen { fd, backlog: 1 } };
    let finding = world.sim.check_submission(pid, &submit).unwrap_err();
    assert!(finding.contains("a Listen on an unbound socket"), "{finding}");
}

#[test]
fn a_connect_on_a_listener() {
    let mut world = World::calm();
    let pid = world.spawn();
    let (listener, addr) = world.listener(pid);
    let submit = Submit { op: world.token(), kind: Op::Connect { fd: listener, addr } };
    let finding = world.sim.check_submission(pid, &submit).unwrap_err();
    assert!(finding.contains("a Connect on a socket that is not fresh"), "{finding}");
}

#[test]
fn a_connect_on_a_connected_socket() {
    let mut world = World::calm();
    let (client, server) = (world.spawn(), world.spawn());
    let (c, _) = world.pair(client, server);
    let submit = Submit { op: world.token(), kind: Op::Connect { fd: c, addr: local(80) } };
    let finding = world.sim.check_submission(client, &submit).unwrap_err();
    assert!(finding.contains("a Connect on a socket that is not fresh"), "{finding}");
}

#[test]
fn a_shutdown_on_a_fresh_socket() {
    let mut world = World::calm();
    let pid = world.spawn();
    let fd = world.socket(pid);
    let submit = Submit { op: world.token(), kind: Op::Shutdown { fd } };
    let finding = world.sim.check_submission(pid, &submit).unwrap_err();
    assert!(finding.contains("a Shutdown on a socket with no connection"), "{finding}");
}

#[test]
fn a_shutdown_on_a_listener() {
    let mut world = World::calm();
    let pid = world.spawn();
    let (listener, _) = world.listener(pid);
    let submit = Submit { op: world.token(), kind: Op::Shutdown { fd: listener } };
    let finding = world.sim.check_submission(pid, &submit).unwrap_err();
    assert!(finding.contains("a Shutdown on a socket with no connection"), "{finding}");
}

#[test]
fn a_truncate_on_a_descriptor_opened_to_read_fails_the_world() {
    let mut world = World::calm();
    let pid = world.spawn();
    let root = world.root(pid, &[skein_fake_machine::Item::file(b"file", b"bytes")]);
    let file = world.open(pid, root, b"file", skein_io::kernel::OpenHow::Read).unwrap();
    let submit = Submit { op: world.token(), kind: Op::Truncate { fd: file, length: 2 } };
    let finding = world.sim.check_submission(pid, &submit).unwrap_err();
    assert!(finding.contains("a Truncate on a descriptor not opened to write"), "{finding}");
}

#[test]
fn a_cancel_of_a_truncate_fails_the_world() {
    let mut world = World::calm();
    let pid = world.spawn();
    let root = world.root(pid, &[skein_fake_machine::Item::file(b"file", b"bytes")]);
    let file = world.open(pid, root, b"file", skein_io::kernel::OpenHow::Write).unwrap();
    world.serving = false;
    let target = world.submit(pid, Op::Truncate { fd: file, length: 2 });
    let submit = Submit { op: world.token(), kind: Op::Cancel { target } };
    let finding = world.sim.check_submission(pid, &submit).unwrap_err();
    assert!(finding.contains("which io never cancels"), "{finding}");
}
