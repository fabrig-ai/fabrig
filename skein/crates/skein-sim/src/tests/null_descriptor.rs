//! Null child descriptors (skein/docs/design/kernel.md, 6.2).

use crate::{Answer, Config, Handle, Pid, Program, Reply, Sim};
use alloc::boxed::Box;
use skein_io::kernel::{Complete, Done, Exit, Op, Pipe, Spawn, Submit, Way};
use skein_lib::{Queue, Token};

fn call(sim: &mut Sim, pid: Pid, op: Op) -> Complete {
    let mut submissions = Queue::with_capacity(1);
    submissions.push(Submit { op: Token::new(1), kind: op });
    sim.submit(pid, &mut submissions);
    let mut completions = Queue::with_capacity(1);
    sim.reap(pid, &mut completions);
    completions.pop().expect("null operations complete at once")
}

#[test]
fn null_descriptors_read_end_and_take_every_write_without_parent_ends() {
    let mut config = Config::calm();
    config.buffer = 1;
    let mut sim = Sim::new(7, config);
    let parent = sim.spawn_process();
    let root = sim.root(parent, Handle::new(1));
    let mut submissions = Queue::with_capacity(1);
    submissions.push(Submit {
        op: Token::new(1),
        kind: Op::Spawn {
            spawn: Box::new(Spawn {
                program: Box::from(&b"service"[..]),
                args: Box::new([]),
                env: Box::new([]),
                root,
                dir: Box::new([]),
                pipes: Box::new([
                    Pipe { child: 0, way: Way::Null, parent: None },
                    Pipe { child: 1, way: Way::Null, parent: None },
                    Pipe { child: 2, way: Way::Null, parent: None },
                ]),
            }),
        },
    });
    sim.submit(parent, &mut submissions);
    let mut calls = Queue::with_capacity(1);
    sim.calls(&mut calls);
    let asked = calls.pop().expect("spawn call");
    let mut answers = Queue::with_capacity(1);
    answers.push(Answer { ticket: asked.ticket, result: Ok(Reply::Program(Program::Service)) });
    sim.answer(&mut answers);
    let mut completions = Queue::with_capacity(1);
    sim.reap(parent, &mut completions);
    let complete = completions.pop().expect("spawned");
    assert!(complete.is_valid(), "null parent slots stay empty on success");
    let Op::Spawn { spawn } = complete.kind else { panic!("spawn") };
    assert!(spawn.pipes.iter().all(|pipe| pipe.parent.is_none()), "null has no parent end");
    let Ok(Done::Spawned { pidfd, .. }) = complete.result else { panic!("service spawn") };
    let (child, inherited) = sim.bind_service(parent, pidfd);
    assert_eq!(inherited.len(), 3, "three child null descriptors");
    for (number, (descriptor, fd)) in (0..3).zip(inherited) {
        assert_eq!(descriptor, number);
        assert_eq!(call(&mut sim, child, Op::PipeRead { fd, buf: Box::new([0; 4]) }).result, Ok(Done::Count(0)));
        for _ in 0..2_u32 {
            assert_eq!(
                call(&mut sim, child, Op::PipeWrite { fd, bytes: Box::from(&b"abcd"[..]), from: 1 }).result,
                Ok(Done::Count(3)),
                "null takes all bytes beyond the pipe buffer"
            );
        }
        assert_eq!(call(&mut sim, child, Op::Close { fd }).result, Ok(Done::Nothing));
    }
    sim.assert_no_open_fds(child);
    sim.finish_service(child, Exit::Code(0));
    for reap in [false, true] {
        assert_eq!(call(&mut sim, parent, Op::Wait { pidfd, reap }).result, Ok(Done::Exit(Exit::Code(0))));
    }
    assert_eq!(call(&mut sim, parent, Op::Close { fd: pidfd }).result, Ok(Done::Nothing));
    let _closed = sim.cut(parent);
    sim.assert_no_open_fds(parent);
}
