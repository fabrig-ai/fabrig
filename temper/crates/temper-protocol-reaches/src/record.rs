//! The table and reach records (temper/docs/design/protocol/configuration.md, 13.2).
use crate::{
    Class, Classes, ITEMS, Item, Kind, NAME_BYTES, PATTERN_BYTES, PATTERNS, PathSet, Reach, Reason, SETS, Subject,
    WORDS_BYTES,
};
use alloc::boxed::Box;
use skein_lib::List;
use temper_client_protocol_wire::v1;

/// Every record decodes to a table; `Classes::check` judges it.
#[must_use]
pub fn classes_from(record: &v1::ProjectClasses) -> Classes {
    let mut items = List::with_capacity(record.items().len());
    for item in record.items() {
        items
            .push(Item {
                subject: subject_from(item.subject()),
                words: item.words().into(),
                class: class_from(item.class()),
            })
            .expect("same item count");
    }
    let mut sets = List::with_capacity(record.sets().len());
    for set in record.sets() {
        sets.push(PathSet {
            number: set.number(),
            name: set.name().into(),
            patterns: set.patterns().clone().into_boxed(),
        })
        .expect("same set count");
    }
    Classes { items: items.into_boxed(), sets: sets.into_boxed() }
}

/// None where the table passes a record's bound.
#[must_use]
pub fn classes_record(classes: &Classes) -> Option<v1::ProjectClasses> {
    let mut items = List::with_capacity(u32::try_from(classes.items.len()).ok()?);
    for item in &classes.items {
        items
            .push(
                v1::ClassItem::new(
                    &v1::CEILINGS,
                    v1::ClassItemParts {
                        subject: subject_record(item.subject),
                        words: item.words.clone(),
                        class: class_record(item.class),
                    },
                )
                .ok()?,
            )
            .ok()?;
    }
    let mut sets = List::with_capacity(u32::try_from(classes.sets.len()).ok()?);
    for set in &classes.sets {
        let mut patterns = List::with_capacity(u32::try_from(set.patterns.len()).ok()?);
        for pattern in &set.patterns {
            patterns.push(pattern.clone()).ok()?;
        }
        sets.push(
            v1::PathSet::new(&v1::CEILINGS, v1::PathSetParts { number: set.number, name: set.name.clone(), patterns })
                .ok()?,
        )
        .ok()?;
    }
    v1::ProjectClasses::new(&v1::CEILINGS, v1::ProjectClassesParts { items, sets }).ok()
}

#[must_use]
pub fn subject_from(record: &v1::ClassSubject) -> Subject {
    match record {
        v1::ClassSubject::Kind(kind) => Subject::Kind(kind_from(kind.kind())),
        v1::ClassSubject::Set(set) => Subject::Set(set.number()),
    }
}

#[must_use]
pub fn subject_record(subject: Subject) -> v1::ClassSubject {
    match subject {
        Subject::Kind(kind) => v1::ClassSubject::Kind(
            v1::ReachKindOf::new(&v1::CEILINGS, v1::ReachKindOfParts { kind: kind_record(kind) })
                .expect("fixed kind field"),
        ),
        Subject::Set(number) => v1::ClassSubject::Set(set_record(number)),
    }
}

fn set_record(number: u8) -> v1::ReachSet {
    v1::ReachSet::new(&v1::CEILINGS, v1::ReachSetParts { number }).expect("fixed set field")
}

/// None for a reason naming set 0 or a set past `SETS`.
#[must_use]
pub fn reach_from(record: &v1::Reach) -> Option<Reach> {
    let reason = match record.reason() {
        v1::ReachReason::Kind => Reason::Kind,
        v1::ReachReason::Always => Reason::Always,
        v1::ReachReason::Set(set) => {
            if set.number() == 0 || usize::from(set.number()) > SETS {
                return None;
            }
            Reason::Set(set.number())
        }
    };
    Some(Reach {
        kind: kind_from(record.kind()),
        class: class_from(record.class()),
        reason,
        under: record.under(),
        sets: record.sets(),
        matched: record.matched(),
        away: record.away(),
    })
}

#[must_use]
pub fn reach_record(reach: &Reach) -> v1::Reach {
    let reason = match reach.reason {
        Reason::Kind => v1::ReachReason::Kind,
        Reason::Set(number) => v1::ReachReason::Set(set_record(number)),
        Reason::Always => v1::ReachReason::Always,
    };
    v1::Reach::new(
        &v1::CEILINGS,
        v1::ReachParts {
            kind: kind_record(reach.kind),
            class: class_record(reach.class),
            reason,
            under: reach.under,
            sets: reach.sets,
            matched: reach.matched,
            away: reach.away,
        },
    )
    .expect("fixed reach fields")
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

fn kind_from(record: &v1::ReachKind) -> Kind {
    match record {
        v1::ReachKind::Wording => Kind::Wording,
        v1::ReachKind::Retry => Kind::Retry,
        v1::ReachKind::Implementation => Kind::Implementation,
        v1::ReachKind::Tests => Kind::Tests,
        v1::ReachKind::Dependency => Kind::Dependency,
        v1::ReachKind::Departure => Kind::Departure,
        v1::ReachKind::Money => Kind::Money,
        v1::ReachKind::Irreversible => Kind::Irreversible,
        v1::ReachKind::Rules => Kind::Rules,
        v1::ReachKind::Other => Kind::Other,
    }
}

fn kind_record(value: Kind) -> v1::ReachKind {
    match value {
        Kind::Wording => v1::ReachKind::Wording,
        Kind::Retry => v1::ReachKind::Retry,
        Kind::Implementation => v1::ReachKind::Implementation,
        Kind::Tests => v1::ReachKind::Tests,
        Kind::Dependency => v1::ReachKind::Dependency,
        Kind::Departure => v1::ReachKind::Departure,
        Kind::Money => v1::ReachKind::Money,
        Kind::Irreversible => v1::ReachKind::Irreversible,
        Kind::Rules => v1::ReachKind::Rules,
        Kind::Other => v1::ReachKind::Other,
    }
}

fn class_from(record: &v1::ReachClass) -> Class {
    match record {
        v1::ReachClass::Decide => Class::Decide,
        v1::ReachClass::Ask => Class::Ask,
        v1::ReachClass::AskAway => Class::AskAway,
        v1::ReachClass::Wait => Class::Wait,
    }
}

fn class_record(value: Class) -> v1::ReachClass {
    match value {
        Class::Decide => v1::ReachClass::Decide,
        Class::Ask => v1::ReachClass::Ask,
        Class::AskAway => v1::ReachClass::AskAway,
        Class::Wait => v1::ReachClass::Wait,
    }
}
