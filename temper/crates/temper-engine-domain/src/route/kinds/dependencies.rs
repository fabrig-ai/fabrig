//! Cross-domain judgement of kind 13 (temper/docs/design/domain/forge/dependencies.md, 2).
use crate::Domain;
use alloc::boxed::Box;
use skein_lib::{Decimal, List};
use temper_engine_domain_forge::settings;

pub(super) fn refusal(domain: &Domain, asked: &settings::Asked) -> Option<Box<[u8]>> {
    let settings::Edit::Set(value) = &asked.edit else { return None };
    let settings::Value::Dependency(value) = &value.value else { return None };
    let project = Decimal::of(u64::from(value.project));
    if value.project == asked.scope {
        return Some(words(asked.number, &[b"project ", project.as_bytes(), b" cannot depend on itself"]));
    }
    if !domain.core.open_project(value.project) {
        return Some(words(asked.number, &[b"project ", project.as_bytes(), b" is not a project of this deployment"]));
    }
    None
}

pub(super) fn words(number: u32, parts: &[&[u8]]) -> Box<[u8]> {
    let number = Decimal::of(u64::from(number));
    let mut words = List::uncapped();
    for part in [b"dependency ".as_slice(), number.as_bytes(), b": "] {
        for &byte in part {
            words.push(byte).expect("dependency refusal prefix");
        }
    }
    for part in parts {
        for &byte in *part {
            words.push(byte).expect("dependency refusal");
        }
    }
    words.into_boxed()
}
