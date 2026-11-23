//! Typed positions in the restart walk (protocol/store/README.md, 4.3, and protocol/store/loads.md, 2).
//! The protocol's encoded intervals are checked against these typed places.
use crate::{Key, start::RestoreRange};

/// A key's exclusive cursor in a request's concatenated restore ranges.
/// The interval precedes the typed key in comparisons; no bytes, IO or
/// allocation are needed for a restore key. Keys outside the walk have no place.
#[must_use]
pub fn restore_cursor(ranges: &[RestoreRange], key: &Key) -> Option<(u32, Key)> {
    let mut offset = 0_u32;
    for range in ranges {
        let (count, interval) = place(range, key)?;
        if let Some(interval) = interval {
            return Some((offset.checked_add(interval)?, key.clone()));
        }
        offset = offset.checked_add(count)?;
    }
    None
}

// One typed description of each range's interval count and membership.
fn place(range: &RestoreRange, key: &Key) -> Option<(u32, Option<u32>)> {
    let admitted = range.admits(key);
    let (count, interval) = match range {
        RestoreRange::Deployment | RestoreRange::Settings => (1, 0),
        RestoreRange::Core => {
            let interval = if let Key::Core(crate::CoreKey::Part(_)) = key { 1 } else { 0 };
            (2, interval)
        }
        RestoreRange::Accounts => {
            let interval = match key {
                Key::Accounts(jig_domain_accounts::Key::Issued { .. }) => 0,
                Key::Accounts(jig_domain_accounts::Key::Facts { .. }) => 2,
                Key::Accounts(
                    jig_domain_accounts::Key::Next
                    | jig_domain_accounts::Key::Account { .. }
                    | jig_domain_accounts::Key::Source { .. },
                )
                | Key::Core(_)
                | Key::People(_)
                | Key::Tasks(_)
                | Key::Notes(_) => 1,
            };
            (3, interval)
        }
        RestoreRange::People => {
            let interval = match key {
                Key::People(
                    jig_domain_people::Key::Acted(_)
                    | jig_domain_people::Key::Deletion(_)
                    | jig_domain_people::Key::Deleted(_)
                    | jig_domain_people::Key::Public(_),
                ) => 1,
                Key::People(
                    jig_domain_people::Key::Person(_)
                    | jig_domain_people::Key::ReadResults { .. }
                    | jig_domain_people::Key::ReadConversation { .. }
                    | jig_domain_people::Key::SignIn(_)
                    | jig_domain_people::Key::Roles(_)
                    | jig_domain_people::Key::Policy(_)
                    | jig_domain_people::Key::Answer(_)
                    | jig_domain_people::Key::EndedRead { .. },
                )
                | Key::Accounts(_)
                | Key::Core(_)
                | Key::Tasks(_)
                | Key::Notes(_) => 0,
            };
            (2, interval)
        }
        RestoreRange::Tasks => {
            let interval = if let Key::Tasks(jig_domain_tasks::Key::PlacedHold(_)) = key { 1 } else { 0 };
            (2, interval)
        }
        RestoreRange::Census(census) => {
            let count = u32::try_from(census.results.len())
                .ok()?
                .checked_add(1)?
                .checked_add(u32::try_from(census.questions.len()).ok()?)?;
            return Some((count, census.interval(key)));
        }
    };
    Some((count, if admitted { Some(interval) } else { None }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CoreKey, PartKey};

    #[test]
    fn a_part_follows_the_other_core_records_in_the_restart_walk() {
        let key = Key::Core(CoreKey::Part(PartKey { task: 7, attempt: 2, number: 3 }));
        assert_eq!(restore_cursor(&[RestoreRange::Core], &key), Some((1, key)));
    }

    #[test]
    fn a_range_after_core_counts_the_part_interval_even_when_it_is_empty() {
        let key = Key::Tasks(jig_domain_tasks::Key::PlacedHold(7));
        assert_eq!(restore_cursor(&[RestoreRange::Core, RestoreRange::Tasks], &key), Some((3, key)));
    }
}
