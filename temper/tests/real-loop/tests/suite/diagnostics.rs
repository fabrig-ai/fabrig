//! Forced overdue diagnostics on the reference case's real cluster.
use skein_lib::Time;
use skein_world::{Referee, real::World};
use temper_real_loop_world::{self as real_loop, world::Proc};

struct Forced {
    judge: real_loop::world::Judge,
}
impl Referee<Proc> for Forced {
    fn act(&mut self, now: Time, procs: &mut [Proc]) {
        self.judge.act(now, procs);
    }
    fn watch(&mut self, now: Time, procs: &[Proc]) -> Result<(), String> {
        self.judge.watch(now, procs)?;
        if procs[1].ended_tasks().contains(&real_loop::case::REVIEW) {
            // An independent terminal witness triggers the fake verdict, even if
            // the judge's task-observation hook has been disconnected.
            if procs[1].has_task_stub(real_loop::case::REVIEW)? {
                return Err("ordinary delegate completion needs no stub".into());
            }
            let sync = procs[1].terminal_sync().ok_or("terminal sync")?;
            if procs[1].operation_pending(sync) {
                return Err("completion hook removes the finished sync".into());
            }
            self.judge.force_failed();
        }
        Ok(())
    }
    fn next_deadline(&self) -> Option<Time> {
        self.judge.next_deadline()
    }
    fn overdue(&self, now: Time) -> Option<String> {
        self.judge.overdue(now)
    }
    fn passed(&self) -> bool {
        self.judge.passed()
    }
}

#[test]
fn forced_overdue_observes_real_tasks_operations_and_dispatch() {
    use real_loop::world;
    const WORKERS: &[&str] = &["ws", "s1"];
    let scratch: Vec<_> = ["engine", "ws", "s1"].map(real_loop::Scratch::new).into();
    let state = std::rc::Rc::new(std::cell::RefCell::new(None));
    let mut world = World::new_controlled(|controls| Forced {
        judge: world::cluster_judge(controls, WORKERS, Some(state.clone()), real_loop::PATIENCE),
    });
    world::cluster_roots(
        &mut world,
        WORKERS,
        &[],
        skein_lib::Duration::from_millis(10),
        &scratch,
        || real_loop::llm::scripted(Box::new([])),
        Some((
            real_loop::case::fixture(),
            real_loop::case::scripts::step_4_1,
            real_loop::case::scripts::step_4_1_rules,
        )),
        real_loop::case::acts::step_4_1,
    );
    world.spawn(|| {
        let (forge, peer) = real_loop::forge::State::new(real_loop::case::fixture());
        *state.borrow_mut() = Some(forge);
        Proc::forge(peer)
    });
    let result = world.try_run(&skein_shell::Clock::new(), real_loop::PATIENCE);
    real_loop::git::share(None);
    let Err(message) = result else { panic!("the review terminal forces the overdue dispatch") };
    for expected in [
        "the referee failed the real loop before patience",
        "task=2 state=landing",
        "task=3",
        "reviewed_below=1",
        "wait=story-step",
        "kind=ctl age_ns=",
        "child-run=ctl",
        "operation=pipe-read",
        "observed_age_ns=",
        "outstanding total=",
        "shown=",
        "omitted=",
        "caller step=",
        "forced failed deadline figures",
    ] {
        assert!(message.contains(expected), "missing {expected}: {message}");
    }
    assert!(!message.contains("task=5 state=absent"), "the review terminal has no stub: {message}");
}
