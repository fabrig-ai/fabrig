//! Quiet-point checks, outside the runtime world's heap meter (testing/worker.md, 4).
use skein_lib::Env;
use skein_world::inspect::{Moment, check_tables, check_waits};
use temper_fake_checkout::Checkout;
use temper_worker_domain::{Domain, Limits};

use crate::builder::Builder;

pub(crate) fn check(
    seed: u64,
    env: &Env<Limits>,
    iteration: &mut u64,
    domain: &Domain,
    disk: Option<&Checkout>,
    builder: &Builder,
) -> Result<(), String> {
    *iteration += 1;
    let moment = Moment { seed, at: env.now, iteration: *iteration };
    let limits = &env.limits;
    let mut worker = temper_worker_inspect::inspection();
    if let Some(disk) = disk {
        temper_worker_inspect::inspect(domain, limits, disk, &mut worker);
    } else {
        temper_worker_inspect::inspect_root(domain, limits, &mut worker);
        temper_worker_inspect::inspect_checkout(domain.checkout(), &limits.checkout, &mut worker);
    }
    let builder = builder.inspection();
    let inspections = [&worker, &builder];
    check_waits(&inspections, moment)?;
    check_tables(&inspections, moment)
}
