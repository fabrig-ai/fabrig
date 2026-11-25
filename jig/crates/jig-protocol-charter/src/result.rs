//! Results read after the fleet's fence (protocol/channel/owners.md, section 1.1;
//! domain/hosts/charters.md). The routing maps these into domain values.
use crate::generated::charter::field_to_domain;
use alloc::boxed::Box;
use skein_lib::{List, Reader};
use smith_charter::Record;
use smith_charter::{CEILINGS, Form};

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Field {
    pub name: Box<[u8]>,
    pub value: Box<[u8]>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum RunResult {
    Report { words: Box<[u8]> },
    Verdict { label: Box<[u8]>, fields: Box<[Field]>, words: Box<[u8]> },
    Failure { reason: Box<[u8]> },
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Unreadable {
    Malformed,
    Form,
}

pub fn result(bytes: &[u8]) -> Result<RunResult, Unreadable> {
    let mut reader = Reader::new(bytes);
    let Ok(value) = smith_charter::RunResult::decode(&CEILINGS, &mut reader) else {
        return Err(Unreadable::Malformed);
    };
    if !reader.is_empty() {
        return Err(Unreadable::Malformed);
    }
    match value.form {
        Form::Change => Err(Unreadable::Form),
        Form::Report => {
            if value.label.is_some() || !value.items.is_empty() {
                return Err(Unreadable::Form);
            }
            Ok(RunResult::Report { words: value.text })
        }
        Form::Failure => {
            if value.label.is_some() || !value.items.is_empty() {
                return Err(Unreadable::Form);
            }
            Ok(RunResult::Failure { reason: value.text })
        }
        Form::Verdict => {
            if !value.items.is_empty() {
                return Err(Unreadable::Form);
            }
            let Some(label) = value.label else {
                return Err(Unreadable::Form);
            };
            let mut fields = List::with_capacity(value.fields.len());
            for field in value.fields.into_boxed() {
                fields.push(field_to_domain(field)).expect("capacity for every decoded field");
            }
            Ok(RunResult::Verdict { label, fields: fields.into_boxed(), words: value.text })
        }
    }
}
