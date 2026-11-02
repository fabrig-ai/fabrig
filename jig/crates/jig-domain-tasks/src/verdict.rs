//! Closed named fields accompany a verdict without interpreting their values
//! (domain/tasks.md, section 3; programming-model.md, sections 4.4 and 6).
use crate::{FieldRule, ResultField, VERDICT_FIELDS, Verdict};

pub(crate) fn fields_bytes(fields: &[ResultField]) -> Option<usize> {
    if fields.len() > skein_lib::length::usize_of(VERDICT_FIELDS) {
        return None;
    }
    let mut bytes = 0_usize;
    for field in fields {
        if field.name.is_empty() {
            return None;
        }
        bytes = bytes.checked_add(field.name.len())?.checked_add(field.value.len())?;
    }
    Some(bytes)
}

pub(crate) fn rules_valid(choice: &Verdict, limit: u32) -> bool {
    let bound = skein_lib::length::usize_of(limit);
    if choice.fields.len() > skein_lib::length::usize_of(VERDICT_FIELDS.min(limit)) {
        return false;
    }
    let mut bytes = skein_lib::length::usize_of(choice.words);
    for (at, field) in choice.fields.iter().enumerate() {
        if field.name.is_empty() {
            return false;
        }
        bytes = match bytes.checked_add(field.name.len()) {
            Some(bytes) => bytes,
            None => return false,
        };
        bytes = match bytes.checked_add(skein_lib::length::usize_of(field.max)) {
            Some(bytes) if bytes <= bound => bytes,
            Some(_) | None => return false,
        };
        for earlier in choice.fields.iter().take(at) {
            if earlier.name == field.name {
                return false;
            }
        }
    }
    bytes <= bound
}

pub(crate) fn fields_valid(rules: &[FieldRule], fields: &[ResultField]) -> bool {
    if fields.len() != rules.len() {
        return false;
    }
    for (at, field) in fields.iter().enumerate() {
        for earlier in fields.iter().take(at) {
            if earlier.name == field.name {
                return false;
            }
        }
        let mut found = false;
        for rule in rules {
            if field.name == rule.name {
                if field.value.len() > skein_lib::length::usize_of(rule.max) {
                    return false;
                }
                found = true;
            }
        }
        if !found {
            return false;
        }
    }
    true
}
