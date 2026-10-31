use super::fixtures::{GROUP_PORT, Harness, built_group, group_bytes, group_file, long};
use crate::{host::Echo, limits};
use skein_fake_machine::{How, Item};
use skein_io::kernel::Exit;
use skein_lib::{Duration, List, bytes};
use temper_forge_setup::{Stage, Steps, group};

const FORGE: u16 = 7;
const SLOTS: u32 = 4;
fn harness(groups: Box<[group::Built]>, items: &[Item], ports: &[core::net::SocketAddr]) -> Harness {
    Harness::with_groups(
        Steps { always: Box::new([long(b"serve", &[b"never"])]), asked: Box::new([]) },
        groups,
        SLOTS,
        items,
        true,
        skein_sim::Config::calm().buffer,
        &limits::small(),
        Echo::Steps,
        ports,
    )
}
fn files(recorded: bool) -> Box<[Item]> {
    let mut items = List::with_capacity(5);
    for item in [Item::directory(b"box"), Item::directory(b"box/groups"), group_file(FORGE)] {
        items.push(item).unwrap();
    }
    if recorded {
        for item in [Item::directory(b"box/started"), Item::file(b"box/started/7", b"").mode(0o600)] {
            items.push(item).unwrap();
        }
    }
    items.into_boxed()
}
fn signals(h: &Harness) -> Box<[skein_io::kernel::Fd]> {
    let mut signals = List::with_capacity(2);
    for entry in h.sim.trace() {
        if entry.pid == h.pid
            && let skein_sim::Event::Submit { kind: skein_sim::Summary::Signal { pidfd, signal: 15, .. }, .. } =
                entry.event
        {
            signals.push(pidfd).unwrap();
        }
    }
    signals.into_boxed()
}
fn running() -> Harness {
    let mut h = harness(Box::new([]), &files(false), &[GROUP_PORT]);
    h.run_until_idle();
    h.ask(b"start 7\n");
    h.run_until_idle();
    h
}
fn finish_group(h: &mut Harness) {
    let index = h.child_index(b"fake forge 7");
    h.sim.finish_service(h.hosted.get(index).unwrap().pid, Exit::Code(0));
    h.run_until_idle();
}
fn exists(h: &mut Harness, path: &[u8]) -> bool {
    let root = h.machine.reopen_root(0);
    let found = match h.machine.open(root, path, How::Read) {
        Ok(fd) => {
            h.machine.close(fd);
            true
        }
        Err(_) => false,
    };
    h.machine.close(root);
    found
}
fn end(h: &mut Harness) {
    h.terminate();
    assert_eq!(h.run_to_end(), Exit::Code(0));
    h.sim.assert_quiescent(h.pid);
    h.sim.assert_no_open_fds(h.pid);
    assert_eq!(h.machine.open_handles(), 0);
}
#[test]
fn a_start_of_a_group_not_held_reads_its_file_and_starts_it() {
    let mut h = running();
    assert_eq!(h.reply_bytes().as_ref(), b"step 7 prepare\nstep 7 run\nstarted 7\n");
    let expected = built_group(FORGE);
    for (_, step) in &expected.group.steps {
        let launch = h.launch(step.name());
        assert_eq!(launch.arguments.as_ref(), step.arguments());
    }
    assert!(exists(&mut h, b"box/started/7"));
    end(&mut h);
}
#[test]
fn a_start_without_a_group_file_answers_not_asked() {
    let mut h = harness(Box::new([]), &[], &[]);
    h.run_until_idle();
    h.ask(b"start 7\n");
    h.run_until_idle();
    assert_eq!(h.reply_bytes().as_ref(), b"failed 7 not asked\n");
    assert!(!Harness::contains(&h.stderr(), b"box: forge"));
    end(&mut h);
}
#[test]
fn an_invalid_group_file_answers_unreadable_with_one_notice() {
    let mut items = files(false);
    items[2] = Item::file(b"box/groups/7", b"invalid");
    let mut h = harness(Box::new([]), &items, &[]);
    h.run_until_idle();
    h.ask(b"start 7\n");
    h.run_until_idle();
    assert_eq!(h.reply_bytes().as_ref(), b"failed 7 unreadable\n");
    assert_eq!(bytes::count(&h.stderr(), b"box: forge 7:", 2), 1);
    end(&mut h);
}
#[test]
fn a_group_naming_a_port_the_box_does_not_hold_is_unreadable() {
    let mut h = harness(Box::new([]), &files(false), &[]);
    h.run_until_idle();
    h.ask(b"start 7\n");
    h.run_until_idle();
    assert_eq!(h.reply_bytes().as_ref(), b"failed 7 unreadable\n");
    assert!(Harness::contains(&h.stderr(), b"127.0.0.1:8369"));
    end(&mut h);
}
#[test]
fn groups_given_at_the_start_with_a_record_start_before_serve() {
    let mut h = harness(Box::new([built_group(FORGE)]), &files(true), &[GROUP_PORT]);
    h.run_until_idle();
    let mut names = List::with_capacity(3);
    for launch in &h.launches {
        names.push(launch.name.as_ref()).unwrap();
    }
    assert_eq!(names.as_slice(), [b"prepare 7".as_slice(), b"fake forge 7", b"serve"]);
    end(&mut h);
}
#[test]
fn one_without_a_record_stays_dormant_until_asked() {
    let mut h = harness(Box::new([built_group(FORGE)]), &files(false), &[GROUP_PORT]);
    h.run_until_idle();
    assert_eq!(h.launches.len(), 1);
    h.ask(b"start 7\n");
    h.run_until_idle();
    assert_eq!(h.reply_bytes().as_ref(), b"step 7 prepare\nstep 7 run\nstarted 7\n");
    end(&mut h);
}
#[test]
fn remove_stops_the_group_runs_its_removal_then_removes_its_records() {
    let mut built = built_group(FORGE);
    let mut steps = List::with_capacity(3);
    for step in built.group.steps {
        steps.push(step).unwrap();
    }
    steps.push((Stage::Run, long(b"peer", &[b"never"]))).unwrap();
    built.group.steps = steps.into_boxed();
    let mut h = harness(Box::new([built]), &files(false), &[GROUP_PORT]);
    h.run_until_idle();
    h.ask(b"start 7\n");
    h.run_until_idle();
    let peer = h.hosted.get(h.child_index(b"peer")).unwrap();
    let (peer_pid, peer_fd) = (peer.pid, peer.pidfd);
    let forge_fd = h.hosted.get(h.child_index(b"fake forge 7")).unwrap().pidfd;
    h.ask(b"remove 7\n");
    h.run_until_idle();
    assert_eq!(signals(&h).as_ref(), &[peer_fd], "last child is stopped first");
    h.sim.finish_service(peer_pid, Exit::Code(0));
    h.run_until_idle();
    assert_eq!(signals(&h).as_ref(), &[peer_fd, forge_fd]);
    assert_eq!(h.spawn_times(b"remove 7").len(), 0, "removal waits for group close");
    assert!(exists(&mut h, b"box/started/7"));
    assert!(exists(&mut h, b"box/groups/7"));
    finish_group(&mut h);
    let launch = h.launch(b"remove 7");
    assert_eq!(launch.arguments.as_ref(), built_group(FORGE).remove.arguments());
    assert_eq!(h.spawn_times(b"remove 7").len(), 1);
    assert_eq!(h.reply_bytes().as_ref(), b"step 7 prepare\nstep 7 run\nstep 7 run\nstarted 7\nremoved 7\n");
    assert!(!exists(&mut h, b"box/started/7"));
    assert!(!exists(&mut h, b"box/groups/7"));
    assert!(Harness::contains(&h.stderr(), b"Forge 7 (fake): removed\n"));
    let mut active = None;
    let mut removed = List::with_capacity(2);
    for entry in h.sim.trace() {
        if entry.pid != h.pid {
            continue;
        }
        if let skein_sim::Event::Complete {
            kind: skein_sim::Summary::Open { path, .. },
            result: Ok(skein_io::kernel::Done::Fd(fd)),
            ..
        } = entry.event
            && (path == skein_sim::Text::of(b"box/started") || path == skein_sim::Text::of(b"box/groups"))
        {
            active = Some((fd, path));
        }
        if let skein_sim::Event::Submit { kind: skein_sim::Summary::Remove { dir, name, .. }, .. } = entry.event
            && name == skein_sim::Text::of(b"7")
        {
            let (fd, path) = active.unwrap();
            assert_eq!(fd, dir);
            removed.push(path).unwrap();
        }
    }
    assert_eq!(removed.as_slice(), &[skein_sim::Text::of(b"box/started"), skein_sim::Text::of(b"box/groups")]);
    end(&mut h);
}
#[test]
fn remove_of_a_group_whose_file_is_gone_answers_removed() {
    let present = [Item::directory(b"box"), Item::directory(b"box/started"), Item::file(b"box/started/7", b"")];
    for items in [&[][..], present.as_slice()] {
        let mut h = harness(Box::new([]), items, &[]);
        h.run_until_idle();
        h.ask(b"remove 7\n");
        h.run_until_idle();
        assert_eq!(h.reply_bytes().as_ref(), b"removed 7\n");
        assert!(!exists(&mut h, b"box/started/7"));
        h.ask(b"remove 7\n");
        h.run_until_idle();
        assert_eq!(h.reply_bytes().as_ref(), b"removed 7\nremoved 7\n");
        end(&mut h);
    }
}
#[test]
fn a_failed_removal_keeps_both_files_and_answers_failed() {
    let mut h = running();
    h.removal_exit = 1;
    h.ask(b"remove 7\n");
    h.run_until_idle();
    finish_group(&mut h);
    assert!(Harness::contains(&h.reply_bytes(), b"failed 7 removal exited 1\n"));
    assert!(exists(&mut h, b"box/started/7"));
    assert!(exists(&mut h, b"box/groups/7"));
    end(&mut h);
}
#[test]
fn a_removed_slot_takes_another_group_on_the_same_port() {
    let mut items = List::with_capacity(4);
    for item in files(false) {
        items.push(item).unwrap();
    }
    items.push(group_file(8)).unwrap();
    let mut h = harness(Box::new([]), items.as_slice(), &[GROUP_PORT]);
    h.run_until_idle();
    h.ask(b"start 7\n");
    h.run_until_idle();
    let first = h.hosted.get(h.child_index(b"fake forge 7")).unwrap().listeners.clone();
    h.ask(b"remove 7\n");
    h.run_until_idle();
    finish_group(&mut h);
    h.ask(b"start 8\n");
    h.run_until_idle();
    assert!(Harness::contains(&h.reply_bytes(), b"removed 7\nstep 8 prepare\nstep 8 run\nstarted 8\n"));
    let second = &h.hosted.get(h.child_index(b"fake forge 8")).unwrap().listeners;
    assert_eq!(first.len(), 1);
    assert_eq!(second.len(), 1);
    let first = h.launch(b"fake forge 7");
    let second = h.launch(b"fake forge 8");
    assert_eq!(first.pipes.get(2), second.pipes.get(2), "both spawns inherit the held listener token");
    end(&mut h);
}
#[test]
fn a_restarted_serve_can_remove_what_it_asked() {
    let mut h = running();
    let old = h.serve_index();
    let pid = h.hosted.get(old).unwrap().pid;
    h.sim.finish_service(pid, Exit::Code(0));
    h.run_until_idle();
    h.sim.advance_to(h.sim.now().saturating_add(Duration::from_millis(500)));
    h.run_until_idle();
    assert_ne!(h.serve_index(), old);
    h.ask(b"remove 7\n");
    h.run_until_idle();
    finish_group(&mut h);
    assert!(Harness::contains(&h.reply_bytes(), b"removed 7\n"));
    assert!(!exists(&mut h, b"box/groups/7"));
    end(&mut h);
}

#[test]
fn a_group_file_at_the_byte_bound_loads_and_one_past_is_unreadable() {
    let content = group_bytes(FORGE);
    for (length, reply) in [
        (group::FILE_BYTES, b"step 7 prepare\nstep 7 run\nstarted 7\n".as_slice()),
        (group::FILE_BYTES.checked_add(1).unwrap(), b"failed 7 unreadable\n"),
    ] {
        let length = usize::try_from(length).unwrap();
        let mut writer = skein_lib::Writer::new(length);
        writer.put(&content).unwrap();
        for _ in content.len()..length {
            writer.put(b" ").unwrap();
        }
        let body = writer.finish();
        let items =
            [Item::directory(b"box"), Item::directory(b"box/groups"), Item::file(b"box/groups/7", &body).mode(0o600)];
        let mut h = harness(Box::new([]), &items, &[GROUP_PORT]);
        h.run_until_idle();
        h.ask(b"start 7\n");
        h.run_until_idle();
        assert_eq!(h.reply_bytes().as_ref(), reply);
        end(&mut h);
    }
}
#[test]
fn a_retry_after_started_was_removed_finishes_the_group_removal() {
    let mut h = harness(Box::new([]), &files(false), &[GROUP_PORT]);
    h.run_until_idle();
    h.ask(b"remove 7\n");
    h.run_until_idle();
    assert_eq!(h.reply_bytes().as_ref(), b"removed 7\n");
    assert!(!exists(&mut h, b"box/groups/7"));
    assert!(!exists(&mut h, b"box/started/7"));
    assert_eq!(h.spawn_times(b"prepare 7").len(), 0);
    assert_eq!(h.spawn_times(b"remove 7").len(), 1);
    end(&mut h);
}

#[test]
fn a_failed_unrecord_after_the_first_file_is_retried_to_the_end() {
    let mut h = running();
    h.refuse_remove_after = Some(1);
    h.ask(b"remove 7\n");
    h.run_until_idle();
    finish_group(&mut h);
    assert!(Harness::contains(&h.reply_bytes(), b"failed 7 not removed\n"));
    assert!(!exists(&mut h, b"box/started/7"));
    assert!(exists(&mut h, b"box/groups/7"));
    h.ask(b"remove 7\n");
    h.run_until_idle();
    assert!(h.reply_bytes().ends_with(b"removed 7\n"));
    assert!(!exists(&mut h, b"box/started/7"));
    assert!(!exists(&mut h, b"box/groups/7"));
    assert_eq!(h.spawn_times(b"remove 7").len(), 2);
    end(&mut h);
}
#[test]
fn a_listener_refusal_names_ipv4_ipv6_and_scoped_addresses() {
    for (address, line) in [
        (GROUP_PORT, b"listener 127.0.0.1:8369 is not held by the box".as_slice()),
        ((core::net::Ipv6Addr::LOCALHOST, 8369).into(), b"listener [::1]:8369 is not held by the box"),
        (
            core::net::SocketAddr::V6(core::net::SocketAddrV6::new(
                core::net::Ipv6Addr::new(0xfe80, 0, 0, 0, 0, 0, 0, 1),
                8369,
                0,
                3,
            )),
            b"listener [fe80::1%3]:8369 is not held by the box",
        ),
        (
            (core::net::Ipv6Addr::new(1, 2, 3, 4, 5, 6, 7, 8), 65535).into(),
            b"listener [1:2:3:4:5:6:7:8]:65535 is not held by the box",
        ),
    ] {
        assert_eq!(crate::groups::unheld(address).as_ref(), line);
    }
}

#[test]
fn a_held_group_whose_file_disappeared_still_removes_its_record() {
    let mut h = running();
    let root = h.machine.reopen_root(0);
    let directory = h.machine.open(root, b"box/groups", How::Directory).unwrap();
    h.machine.remove(directory, b"7", false).unwrap();
    h.machine.close(directory);
    h.machine.close(root);
    h.ask(b"remove 7\n");
    h.run_until_idle();
    finish_group(&mut h);
    assert!(h.reply_bytes().ends_with(b"removed 7\n"));
    assert!(!exists(&mut h, b"box/started/7"));
    assert!(!exists(&mut h, b"box/groups/7"));
    assert_eq!(h.spawn_times(b"remove 7").len(), 1);
    end(&mut h);
}
