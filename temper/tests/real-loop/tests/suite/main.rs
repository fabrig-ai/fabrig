//! Real-loop integration scenarios (temper/docs/design/testing/real-loop.md).
use skein_world::domain::heap::Counting;

#[global_allocator]
static HEAP: Counting = Counting;

mod account_key;
mod case_5_3;
mod cluster;
mod diagnostics;
mod github;
mod github_land;
mod hung_model;
mod lifecycle;
mod lock;
mod lost;
mod model_check;
mod peers;
mod restart;
mod sample;
mod sign_in;
mod teardown;
mod terminal;
mod turn_landing;

mod world {
    mod teardown;
}

mod fuzzy {
    mod case_4_1_4_3;
    mod case_4_6;
    mod case_5_1;
    mod case_5_2;
    mod case_5_4;
    mod case_6_4;
    mod case_6_6;
    mod case_6_8;
    mod green_light;
    mod landing;
    mod program_landing;
    mod repair;
}

// Synchronous Usage operations measure this process and its reaped children;
// the metric's readiness adapter hosts no service and starts no second world.
fn cpu() -> u64 {
    use skein_io::kernel::{Done, Op, Submit};
    use skein_lib::{Queue, Token};
    let mut kernel =
        skein_shell::Kernel::open_on(skein_shell::Config { operations: 1 }, skein_shell::Backend::Readiness)
            .expect("CPU accounting");
    let mut submits = Queue::uncapped();
    submits.push(Submit { op: Token::new(1), kind: Op::Usage });
    kernel.submit(&mut submits, skein_shell::Wait::No);
    let mut completions = Queue::uncapped();
    kernel.reap(&mut completions);
    let Done::Usage(usage) = completions.pop().expect("synchronous usage").result.expect("usage read") else {
        panic!("usage")
    };
    [usage.own.user, usage.own.system, usage.children.user, usage.children.system]
        .into_iter()
        .map(skein_lib::Duration::as_nanos)
        .sum()
}
