//! A shared key-ordered walk for part changes (hearth/docs/design/domain/render/README.md, 4.1).
use super::Keyed;
use alloc::boxed::Box;
use core::fmt::Debug;
use skein_lib::{List, Map};

/// Compares records before projecting changed values, retaining no unchanged values.
pub(crate) fn keyed<T: PartialEq + Debug, C: Debug>(
    before: &[T],
    after: &[T],
    key: fn(&T) -> &[u8],
    make: fn(Keyed<&T>) -> C,
) -> Box<[C]> {
    let capacity = u32::try_from(before.len().saturating_add(after.len())).expect("bounded entries");
    let mut pairs = Map::with_capacity(capacity);
    for row in before {
        pairs.insert(Box::<[u8]>::from(key(row)), (Some(row), None)).expect("entry count");
    }
    for row in after {
        if let Some((_, value)) = pairs.get_mut(key(row)) {
            *value = Some(row);
        } else {
            pairs.insert(Box::<[u8]>::from(key(row)), (None, Some(row))).expect("entry count");
        }
    }
    let mut changes = List::with_capacity(capacity);
    for (key, (before, after)) in &pairs {
        let change = match (before, after) {
            (None, Some(value)) => Keyed::Added { key: key.clone(), value: *value },
            (Some(value), None) => Keyed::Removed { key: key.clone(), value: *value },
            (Some(before), Some(after)) if before != after => {
                Keyed::Changed { key: key.clone(), before: *before, after: *after }
            }
            (Some(_), Some(_)) => continue,
            (None, None) => unreachable!("union entry"),
        };
        changes.push(make(change)).expect("entry count");
    }
    changes.into_boxed()
}

impl<T> Keyed<T> {
    pub(crate) fn map<V>(self, value: fn(T) -> V) -> Keyed<V> {
        match self {
            Self::Added { key, value: row } => Keyed::Added { key, value: value(row) },
            Self::Removed { key, value: row } => Keyed::Removed { key, value: value(row) },
            Self::Changed { key, before, after } => Keyed::Changed { key, before: value(before), after: value(after) },
        }
    }
}
