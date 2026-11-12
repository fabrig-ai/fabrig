//! A service whose standard descriptors belong only to this process.

use skein_conformance::Backend;
use skein_conformance_real::Real;
use skein_io::kernel::{Complete, Done, Exit, Op, Pipe, Spawn, Submit, Way};
use skein_lib::{Duration, Queue, Token};

const PATIENCE: Duration = Duration::from_secs(3);

fn call(real: &mut Real, process: usize, kind: Op) -> Complete {
    let mut submissions = Queue::with_capacity(1);
    submissions.push(Submit { op: Token::new(1), kind });
    real.submit(process, &mut submissions);
    let until = real.now().saturating_add(PATIENCE);
    real.wait_begins("the inherited descriptor child", PATIENCE);
    let mut completions = Queue::with_capacity(1);
    loop {
        real.reap(process, &mut completions);
        if let Some(complete) = completions.pop() {
            assert!(complete.is_valid(), "the spawn backend keeps its record contract");
            return complete;
        }
        if real.now() >= until {
            real.wait_unanswered();
            panic!("the inherited descriptor child did not settle: {:?}", real.wait_figures());
        }
        real.pass(process, Duration::from_millis(5));
    }
}

fn main() {
    skein_fail_stop::install();
    let mut real = Real::new(skein_shell::Backend::Readiness);
    let process = real.open();
    let root = real.root(process, &[]);
    let complete = call(
        &mut real,
        process,
        Op::Spawn {
            spawn: Box::new(Spawn {
                program: Box::from(&b"/bin/sh"[..]),
                args: Box::new([
                    Box::from(&b"-c"[..]),
                    Box::from(&b"IFS= read -r word; printf '%s' \"$word\"; printf error >&2"[..]),
                ]),
                env: Box::new([]),
                root,
                dir: Box::new([]),
                pipes: Box::new([
                    Pipe { child: 0, way: Way::Inherit, parent: None },
                    Pipe { child: 1, way: Way::Inherit, parent: None },
                    Pipe { child: 2, way: Way::Inherit, parent: None },
                ]),
            }),
        },
    );
    let Ok(Done::Spawned { pidfd, .. }) = complete.result else { panic!("spawn failed: {complete:?}") };
    let Op::Spawn { spawn } = complete.kind else { panic!("spawn record") };
    assert!(spawn.pipes.iter().all(|pipe| pipe.parent.is_none()), "inherit creates no parent end");
    for reap in [false, true] {
        assert_eq!(call(&mut real, process, Op::Wait { pidfd, reap }).result, Ok(Done::Exit(Exit::Code(0))));
    }
    for fd in [pidfd, root] {
        assert_eq!(call(&mut real, process, Op::Close { fd }).result, Ok(Done::Nothing));
    }
    real.assert_settled(process);
}
