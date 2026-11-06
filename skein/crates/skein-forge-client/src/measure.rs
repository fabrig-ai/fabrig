//! Durable payload accounting and independent answer row admission.
use crate::{Condition, Delivery, Echo, Effect, Stored, What, Writer};
use alloc::boxed::Box;
use skein_forge::measure;
pub use skein_forge::measure::answer_rows;
use skein_lib::len_u64;

/// Fixed bytes of a saved delivery or echo stamp.
pub const STAMP_BYTES: usize = 16;
/// Fixed bytes of a cached label slot.
pub const LABEL_BYTES: usize = 16;
/// Fixed bytes of a cached reviewer slot.
pub const REVIEWER_BYTES: usize = 8;
/// Fixed bytes of a configured writer slot.
pub const WRITER_BYTES: usize = 16;
const _: () = {
    assert!(STAMP_BYTES >= size_of::<Delivery>(), "stamp width covers delivery storage");
    assert!(STAMP_BYTES >= size_of::<Echo>(), "stamp width covers echo storage");
    assert!(LABEL_BYTES >= size_of::<Box<[u8]>>(), "label width covers string storage");
    assert!(REVIEWER_BYTES >= size_of::<u64>(), "reviewer width covers identifier storage");
    assert!(REVIEWER_BYTES >= 8, "reviewer width covers protocol identifier");
    assert!(WRITER_BYTES >= size_of::<Writer>(), "writer width covers writer storage");
};

/// Count a durable row's strings and fixed list widths.
#[must_use]
pub fn stored(row: &Stored) -> Option<u64> {
    match row {
        Stored::Repository(_) | Stored::Writer(_) => Some(0),
        Stored::Live(row) => {
            let mut held = match &row.watch.resource.what {
                What::Branch(branch) => len_u64(branch.len()),
                What::Repository | What::Pull(_) | What::Issue(_) => 0,
            };
            for count in [row.comments.len(), row.reviews.len(), row.echoes.len()] {
                held = held.checked_add(u64::try_from(count).ok()?.checked_mul(u64::try_from(STAMP_BYTES).ok()?)?)?;
            }
            if let Some(item) = &row.cached.item {
                held = held.checked_add(measure::summary(item)?)?;
            }
            if let Some(pull) = &row.cached.pull {
                held = held.checked_add(measure::pull(pull)?)?;
            }
            Some(held)
        }
    }
}

/// Count a write together with its client-side condition.
#[must_use]
pub fn effect(effect: &Effect) -> Option<u64> {
    let held = measure::write(&effect.write)?;
    match &effect.condition {
        Condition::Merge { base } => held.checked_add(len_u64(base.len())),
        Condition::None | Condition::Update { .. } | Condition::Review { .. } => Some(held),
    }
}
