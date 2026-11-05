use hearth_protocol_accounts::{File, limits::Limits, read};
use skein_heap::Meter;
/// The most heap one read holds, checked against the scenario's ceiling; the result is dropped after.
#[must_use]
pub fn read_metered(file: File, bytes: &[u8], limits: &Limits, ceiling: u64) -> u64 {
    let meter = Meter::new();
    meter.start();
    let result = read(file, bytes, limits);
    let step = meter.end();
    drop(result);
    meter.check(step, ceiling, &file)
}
