//! Admission at the service boundary: new work waits at the watermark,
//! while completions of admitted work are always taken (skein's draft
//! `dynamic-memory.md`, 4).
use crate::boundary::{Event, Link, StoreInput, WorkerInput};
use crate::domain::{Domain, admits, step};
use crate::limits::Limits;
use skein_lib::Env;

/// Hand new work back unchanged at the watermark. Store answers and released
/// continuations are always taken; queued store answers keep arrival order and
/// wait only for earlier work to finish.
#[expect(clippy::result_large_err, reason = "the service admission boundary returns the original event by value")]
pub fn try_step(domain: &mut Domain, env: &Env<Limits>, event: Event) -> Result<(), Event> {
    domain.changed();
    crate::domain::invalidate_input(domain, &event);
    match event {
        event @ Event::Store(StoreInput::ReadLoaded { .. } | StoreInput::Switched) => {
            if domain.store_answers.is_empty() && domain.work.is_empty() {
                step(domain, env, event);
            } else {
                domain.store_answers.push(event);
            }
            Ok(())
        }
        event @ (Event::Store(
            StoreInput::Committed { .. }
            | StoreInput::Uncommitted { .. }
            | StoreInput::Loaded { .. }
            | StoreInput::Unloaded { .. },
        )
        | Event::Resume) => {
            step(domain, env, event);
            Ok(())
        }
        Event::Released(released) => {
            step(domain, env, Event::Released(released));
            Ok(())
        }
        event @ Event::Worker(WorkerInput::Link { link: Link::Up { .. }, .. }) => {
            // A worker's record keeps its bytes here until the fleet is done
            // with them; without room it is answered busy and sent again.
            if !admits(domain, &env.limits) || !crate::link::room(domain) {
                return Err(event);
            }
            step(domain, env, event);
            Ok(())
        }
        event @ (Event::Worker(_)
        | Event::Party(_)
        | Event::Account(_)
        | Event::Credential { .. }
        | Event::Forge(_)
        | Event::Setup { .. }
        | Event::Restart { .. }
        | Event::Stop) => {
            if !admits(domain, &env.limits) {
                return Err(event);
            }
            step(domain, env, event);
            Ok(())
        }
    }
}
