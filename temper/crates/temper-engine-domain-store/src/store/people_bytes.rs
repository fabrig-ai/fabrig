//! Owned payload of the party requests added by jig's batch and compound vocabulary.
use jig_domain_people as people;
use skein_lib::List;

pub(super) fn batch(batch: &[people::Member]) -> Option<u64> {
    let mut bytes = skein_lib::len_u64(batch.len()).checked_mul(u64::try_from(size_of::<people::Member>()).ok()?)?;
    for member in batch {
        bytes = bytes.checked_add(people::member_bytes(member)?)?;
    }
    Some(bytes)
}

pub(super) fn compound(operations: &List<people::Operation>) -> Option<u64> {
    let mut bytes = u64::from(operations.len()).checked_mul(u64::try_from(size_of::<people::Operation>()).ok()?)?;
    for operation in operations {
        let owned = match operation {
            people::Operation::Cancel { reason, .. } | people::Operation::Move { reason, .. } => {
                skein_lib::len_u64(reason.len())
            }
            people::Operation::Message { words, .. } => skein_lib::len_u64(words.len()),
            people::Operation::Amend { amendment, .. } => {
                u64::try_from(size_of::<people::Amendment>()).ok()?.checked_add(people::amendment_bytes(amendment)?)?
            }
            people::Operation::Release { .. } | people::Operation::Stop { .. } => 0,
        };
        bytes = bytes.checked_add(owned)?;
    }
    Some(bytes)
}
