//! A child's inherited stdout is the parent's open pipe (runtime/io/processes.md, 1).

use crate::{Answer, Config, Handle, Pid, Program, Reply, Sim};
use alloc::boxed::Box;
use skein_io::kernel::{Complete, Done, Exit, Fd, Op, Pipe, Spawn, Submit, Way};
use skein_lib::{Queue, Token};

fn call(sim: &mut Sim, pid: Pid, op: Op) -> Complete {
    let mut submissions = Queue::with_capacity(1);
    submissions.push(Submit { op: Token::new(1), kind: op });
    sim.submit(pid, &mut submissions);
    let mut completions = Queue::with_capacity(1);
    sim.reap(pid, &mut completions);
    completions.pop().expect("the operation completes at once")
}

#[test]
fn an_inherited_stdout_writes_to_the_parents_pipe() {
    let mut sim = Sim::new(7, Config::calm());
    let parent = sim.spawn_process();
    // Model the service's startup stdout at its standard descriptor number.
    sim.process_mut(parent).next_fd = 1_i32;
    let output = sim.open_inherited_write(parent);
    assert_eq!(output, Fd::new(1));
    sim.process_mut(parent).next_fd = 3_i32;
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
                pipes: Box::new([Pipe { child: 1, way: Way::Inherit, parent: None }]),
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
    assert!(complete.is_valid(), "inherited parent slots stay empty on success");
    let Op::Spawn { spawn } = complete.kind else { panic!("spawn") };
    assert!(spawn.pipes.iter().all(|pipe| pipe.parent.is_none()), "inherit has no parent end");
    let Ok(Done::Spawned { pidfd, .. }) = complete.result else { panic!("service spawn") };
    let (child, inherited) = sim.bind_service(parent, pidfd);
    assert_eq!(inherited, [(1, Fd::new(1))]);
    assert_eq!(
        call(&mut sim, child, Op::PipeWrite { fd: Fd::new(1), bytes: Box::from(&b"hi"[..]), from: 0 }).result,
        Ok(Done::Count(2))
    );
    assert_eq!(sim.peer_drain(parent, output, 2), b"hi", "the parent's peer receives the child's bytes");
    assert_eq!(call(&mut sim, parent, Op::Close { fd: output }).result, Ok(Done::Nothing));
    assert!(!sim.peer_eof(parent, output), "the child's copy still holds the writer");
    assert_eq!(call(&mut sim, child, Op::Close { fd: Fd::new(1) }).result, Ok(Done::Nothing));
    assert!(sim.peer_eof(parent, output), "the last writer closed");
    sim.assert_no_open_fds(child);
    sim.finish_service(child, Exit::Code(0));
    for reap in [false, true] {
        assert_eq!(call(&mut sim, parent, Op::Wait { pidfd, reap }).result, Ok(Done::Exit(Exit::Code(0))));
    }
    assert_eq!(call(&mut sim, parent, Op::Close { fd: pidfd }).result, Ok(Done::Nothing));
    let _closed = sim.cut(parent);
    sim.assert_no_open_fds(parent);
}
