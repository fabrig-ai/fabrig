//! Test-only tally of every whole-set check, including calls outside the accessor.
#![expect(clippy::disallowed_types, reason = "the test tally isolates each test thread")]
use std::{sync::Mutex, thread::ThreadId};

static CHECKS: Mutex<Vec<(ThreadId, u32)>> = Mutex::new(Vec::new());

fn count(reset: bool, ran: bool) -> u32 {
    let thread = std::thread::current().id();
    let mut checks = CHECKS.lock().expect("test counter lock");
    for (id, count) in checks.iter_mut() {
        if *id == thread {
            if reset {
                *count = 0;
            } else if ran {
                *count = count.checked_add(1).expect("open's check count fits");
            }
            return *count;
        }
    }
    let count = u32::from(ran);
    checks.push((thread, count));
    count
}

pub(crate) fn reset() {
    count(true, false);
}

pub(crate) fn get() -> u32 {
    count(false, false)
}

pub(super) fn ran() {
    count(false, true);
}
