//! Owned setup cuts (temper/docs/design/protocol/setup/testing.md, 1).
use crate::tests::fixtures::{Harness, Opening, call, with_setup};
use crate::{host::Echo, limits};
use alloc::boxed::Box;
use skein_fake_machine::{How, Machine, Refusal};
use skein_io::kernel::{Done, Exit, Op};
use skein_lib::bytes;
use temper_forge_setup::{Steps, kinds, machine::BoxKind, setup_file};

const PORT: u16 = 3000;
const GROUP: &[u8] = b"box/groups/1";
const STARTED: &[u8] = b"box/started/1";
const DIRECTORY: &[u8] = b"forge-1";
const START: &[u8] = b"start 1 owned\n";
const REMOVE: &[u8] = b"remove 1\n";
const TO_RUN: &[&[u8]] = &[b"fetch", b"prepare"];
const BEFORE_READY: &[u8] = b"step 1 fetch\nstep 1 prepare\nstep 1 run\n";

fn steps() -> Steps {
    let directories = jig_protocol_config::Directories::given(b"/config", b"/data", b"/cache").unwrap();
    let file = setup_file::chosen(BoxKind::Owned, PORT, b"ada", false, &directories).unwrap();
    let setup_file::SetupFile::Owned(owned) = file else { panic!("owned setup") };
    kinds(Some(&owned), &[], Box::new([]), true, false)
}

fn open(mut disk: Machine) -> Harness {
    let root = disk.reopen_root(0);
    let mut h = Harness::on_disk(
        steps(),
        (disk, root),
        true,
        skein_sim::Config::calm().buffer,
        &limits::stock(),
        Echo::None,
        Opening {
            parent: None,
            link_form: crate::terminal::LinkForm::Bare,
            log_lines: Box::new([]),
            first_link: None,
            front: None,
            after_block: Box::new([]),
        },
    );
    h.script_owned = true;
    h.run_until_idle();
    h
}

fn first_box() -> Harness {
    let mut disk = Machine::new();
    let root = disk.lay(&with_setup(&[], BoxKind::Owned, 1, PORT));
    disk.close(root);
    let mut h = open(disk);
    h.ask(START);
    h.run_until_idle();
    h
}

fn finish(h: &mut Harness, name: &[u8]) {
    let pid = h.hosted.get(h.child_index(name)).unwrap().pid;
    h.sim.finish_service(pid, Exit::Code(0));
    h.run_until_idle();
}

fn complete(h: &mut Harness, children: &[&[u8]]) {
    for name in children {
        finish(h, name);
    }
}

fn ready(h: &mut Harness) {
    let child = h.hosted.get(h.child_index(b"forgejo")).unwrap();
    let pid = child.pid;
    let listener = *child.listeners.first().expect("Forgejo inherits the held listener");
    let Ok(Done::Accepted { fd, .. }) = call(&mut h.sim, pid, Op::Accept { fd: listener }).result else {
        panic!("the readiness probe reaches Forgejo")
    };
    let version = b"HTTP/1.1 200 OK\r\n";
    assert_eq!(
        call(&mut h.sim, pid, Op::Send { fd, bytes: bytes::copy_of(version), from: 0 }).result,
        Ok(Done::Count(u32::try_from(version.len()).unwrap())),
    );
    assert_eq!(call(&mut h.sim, pid, Op::Close { fd }).result, Ok(Done::Nothing));
    h.run_until_idle();
}

fn read_reply(h: &mut Harness, expected: &[u8]) {
    assert_eq!(h.read_reply(u32::try_from(expected.len()).unwrap()).as_ref(), expected);
}

fn kept(h: &mut Harness, started: bool) {
    let root = h.machine.reopen_root(0);
    for (path, contents) in [(GROUP, b"owned\n".as_slice()), (STARTED, b"".as_slice())] {
        let file = h.machine.open(root, path, How::ReadNoFollow);
        if path == STARTED && !started {
            assert_eq!(file, Err(Refusal::NotFound), "a setup still in progress has no started record");
        } else {
            let file = file.expect("the cut keeps this record");
            assert_eq!(h.machine.read(file, 0, 8).unwrap(), contents);
            h.machine.close(file);
        }
    }
    let directory = h.machine.open(root, DIRECTORY, How::DirectoryNoFollow).expect("the forge is kept on disk");
    h.machine.close(directory);
    h.machine.close(root);
}

fn removed(mut h: Harness, running: bool) {
    h.ask(REMOVE);
    h.run_until_idle();
    if running {
        finish(&mut h, b"runner");
        finish(&mut h, b"forgejo");
    }
    // The scripted remover performs the child's filesystem effect. The host
    // must wait for it before unlinking either record or answering serve.
    let remover = h.child_index(b"remove");
    assert!(!Harness::contains(&h.reply_bytes(), b"removed 1\n"));
    kept(&mut h, running);
    let root = h.machine.reopen_root(0);
    let directory = h.machine.open(root, DIRECTORY, How::DirectoryNoFollow).unwrap();
    h.machine.remove(directory, b"setup", false).unwrap();
    h.machine.close(directory);
    h.machine.remove(root, DIRECTORY, true).unwrap();
    let pid = h.hosted.get(remover).unwrap().pid;
    h.sim.finish_service(pid, Exit::Code(0));
    h.run_until_idle();
    read_reply(&mut h, b"removed 1\n");
    for (path, how) in [(DIRECTORY, How::DirectoryNoFollow), (GROUP, How::ReadNoFollow), (STARTED, How::ReadNoFollow)] {
        assert_eq!(h.machine.open(root, path, how), Err(Refusal::NotFound), "removed {path:?}");
    }
    h.machine.close(root);
    h.terminate();
    assert_eq!(h.run_to_end(), Exit::Code(0));
    h.sim.assert_quiescent(h.pid);
    h.sim.assert_no_open_fds(h.pid);
    h.sim.assert_hosted_settled(h.pid);
    assert_eq!(h.machine.open_handles(), 0);
}

fn restore(disk: Machine, started: bool) -> Harness {
    let mut h = open(disk);
    assert_eq!(h.spawn_times(b"fetch").len(), usize::from(started), "only a recorded group restarts its steps");
    for child in [b"prepare".as_slice(), b"forgejo", b"runner", b"remove"] {
        assert!(h.spawn_times(child).is_empty());
    }
    if started {
        assert!(h.spawn_times(b"serve").is_empty(), "a restored group runs before serve");
        complete(&mut h, TO_RUN);
        ready(&mut h);
        assert_eq!(h.spawn_times(b"runner").len(), 1);
    } else {
        assert!(h.group_opens.is_empty(), "a dormant group does not open its listener at startup");
    }
    assert_eq!(h.spawn_times(b"serve").len(), 1);
    kept(&mut h, started);
    h
}

#[test]
fn an_owned_setup_cut_at_each_step_is_kept_unfinished_and_removed_whole() {
    let fetched = [b"fetch".as_slice()];
    let cuts = [
        ([].as_slice(), b"fetch".as_slice(), false),
        (fetched.as_slice(), b"prepare", false),
        (TO_RUN, b"forgejo", false),
        (TO_RUN, b"runner", true),
    ];
    for (completed, running, started) in cuts {
        let mut h = first_box();
        complete(&mut h, completed);
        if started {
            read_reply(&mut h, BEFORE_READY);
            ready(&mut h);
            assert!(h.reply_bytes().ends_with(b"started 1\n"));
            // Serve is held: only the box has recorded the start. No further
            // read of the reply pipe is driven before the kill.
        }
        let child = h.hosted.get(h.child_index(running)).unwrap();
        assert!(h.sim.service_running(child.pid), "cut with {running:?} still running");
        kept(&mut h, started);
        let mut next = restore(h.abandon(), started);
        if started {
            // The restored group answers as already running, without running again.
            next.ask(b"start 1\n");
            next.run_until_idle();
            read_reply(&mut next, b"started 1\n");
            assert_eq!(next.spawn_times(b"forgejo").len(), 1);
        }
        removed(next, started);
    }
}

#[test]
fn a_removal_cut_after_its_answer_finishes_at_the_next_start() {
    let mut h = first_box();
    complete(&mut h, TO_RUN);
    ready(&mut h);
    read_reply(&mut h, BEFORE_READY);
    read_reply(&mut h, b"step 1 run\nstarted 1\n");
    // Serve has answered the owner's removal. The old box is held with the
    // removal waiting on its pipe: no turn can settle it or answer serve.
    h.ask(REMOVE);
    kept(&mut h, true);
    assert!(h.spawn_times(b"remove").is_empty());
    let next = restore(h.abandon(), true);
    // On opening, serve replays its kept removal (the engine domain's tests
    // drive that decision); no owner has to ask again.
    removed(next, true);
}
