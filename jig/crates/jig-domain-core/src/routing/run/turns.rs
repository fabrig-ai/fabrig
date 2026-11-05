//! Turns and their current windows commit together (domain/engine.md, 7.2).
use crate::{Core, CoreKey, CoreRecord, Held, Key, Limits, Record, Request, Write};
use alloc::boxed::Box;
use jig_domain_tasks as tasks;
use skein_lib::{Env, Queue};

pub(crate) fn route_accepted_turn(
    core: &mut Core,
    env: &Env<Limits>,
    accepted: tasks::Accepted,
    record: crate::TurnRecord,
) -> Queue<Request> {
    let (task, attempt, turn) = (record.task, record.attempt, record.turn);
    let refused = &core.tasks.task(task).expect("accepted turn task live").refused;
    let proof = core.proofs.get_mut(&task).expect("turn proof reserved before child mutation");
    assert!(proof.attempt == attempt, "turn callback retains its claim");
    let mut out = Queue::uncapped();
    match accepted {
        tasks::Accepted::New => {
            let mut outstanding = skein_lib::List::with_capacity(env.limits.tasks.run_inbox);
            for &number in &proof.outstanding {
                if Some(number) > record.read && !refused.contains(&number) {
                    outstanding.push(number).expect("remaining offers fit");
                }
            }
            proof.outstanding = outstanding.into_boxed();
            proof.turn = Some(crate::TurnProof { turn, cumulative: record.spent, read: record.read });
            if record.opens_window {
                proof.window = Some(crate::CurrentWindow { attempt, turn, closed: false });
            }
            out.push(Request::Write(Write::Save(Record::Core(CoreRecord::RunProof(proof.clone())))));
            out.push(Request::Write(Write::Save(Record::Core(CoreRecord::Turn(record)))));
            for key in core.retire_calls(task, attempt, turn, env.limits.call_records) {
                out.push(Request::Write(Write::Erase(Key::Core(CoreKey::Call(key)))));
            }
            crate::routing::view::routes::touch_number(core, &env.limits, task);
        }
        tasks::Accepted::Already => {}
    }
    out.push(Request::Held(Box::new(Held::TaskTurnKept { task, attempt, turn })));
    if let Some(relay) = crate::effects::relays::next(core, task, env.limits.tasks.run_inbox) {
        out.push(Request::Held(Box::new(relay)));
    }
    out
}
