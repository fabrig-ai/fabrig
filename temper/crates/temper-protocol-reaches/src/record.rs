//! The table and reach records (temper/docs/design/protocol/configuration/settings-kinds.md, 1).
use crate::{
    Class, Classes, ITEMS, Item, NAME_BYTES, PATTERN_BYTES, PATTERNS, PathSet, Reach, SETS, Subject, WORDS_BYTES,
    generated::client_wire as maps,
};
use alloc::boxed::Box;
use skein_lib::List;
use temper_client_protocol_wire::v1;
use v1::Record;

/// Every record decodes to a table; `Classes::check` judges it.
#[must_use]
pub fn classes_from(record: &v1::ProjectClasses) -> Classes {
    maps::project_classes_to_domain(record.clone())
}

/// None where the table passes a record's bound.
#[must_use]
pub fn classes_record(classes: &Classes) -> Option<v1::ProjectClasses> {
    let record = maps::project_classes_to_record(classes).ok()?;
    record.measure(&v1::CEILINGS).ok()?;
    Some(record)
}

#[must_use]
pub fn subject_from(record: &v1::ClassSubject) -> Subject {
    maps::class_subject_to_domain(record.clone())
}

#[must_use]
pub fn subject_record(subject: Subject) -> v1::ClassSubject {
    maps::class_subject_to_record(&subject)
}

/// None for a reason naming set 0 or a set past `SETS`.
#[must_use]
pub fn reach_from(record: &v1::Reach) -> Option<Reach> {
    if let v1::ReachReason::Set(set) = &record.reason
        && (set.number == 0 || usize::from(set.number) > SETS)
    {
        return None;
    }
    Some(maps::reach_to_domain(record.clone()))
}

#[must_use]
pub fn reach_record(reach: &Reach) -> v1::Reach {
    maps::reach_to_record(reach)
}

/// A checked table full at every record bound.
#[must_use]
pub fn largest() -> Classes {
    let mut items = List::with_capacity(u32::try_from(ITEMS).expect("fixed item bound"));
    for _ in 0..ITEMS {
        items
            .push(Item {
                subject: Subject::Set(u8::try_from(SETS).expect("fixed set bound")),
                words: Box::new([u8::MAX; WORDS_BYTES]),
                class: Class::AskAway,
            })
            .expect("bounded items");
    }
    let mut sets = List::with_capacity(u32::try_from(SETS).expect("fixed set bound"));
    for number in 1..=SETS {
        let mut patterns = List::with_capacity(u32::try_from(PATTERNS).expect("fixed pattern bound"));
        for _ in 0..PATTERNS {
            patterns.push(Box::<[u8]>::from([u8::MAX; PATTERN_BYTES])).expect("bounded patterns");
        }
        sets.push(PathSet {
            number: u8::try_from(number).expect("bounded set number"),
            name: Box::new([u8::MAX; NAME_BYTES]),
            patterns: patterns.into_boxed(),
        })
        .expect("bounded sets");
    }
    Classes { items: items.into_boxed(), sets: sets.into_boxed() }
}
