//! Rules' path sets (temper/docs/design/domain/forge/change/branch-path.md, A rule by path).
use super::{Inputs, OVER};
use alloc::boxed::Box;
use skein_lib::{List, bytes};
use temper_engine_domain_forge_vocabulary::settings::LandingRule;
use temper_protocol_reaches::PathSet;

#[must_use]
pub fn by_path(rule: &LandingRule, sets: &[PathSet]) -> bool {
    if rule.sets.is_empty() {
        return false;
    }
    for number in &rule.sets {
        let mut found = false;
        for set in sets {
            found |= set.number == *number;
        }
        if !found {
            return false;
        }
    }
    true
}

#[must_use]
pub fn set_names(rule: &LandingRule, sets: &[PathSet]) -> Box<[u8]> {
    let mut names = Box::from(b"".as_slice());
    for number in &rule.sets {
        for set in sets {
            if set.number == *number {
                names = bytes::concat(&[&names, if names.is_empty() { b"" } else { b" or " }, &set.name]);
            }
        }
    }
    names
}

#[derive(Debug)]
pub(super) struct Coverage {
    pub all: bool,
    pub sets: List<u8>,
}
impl Coverage {
    pub fn new() -> Self {
        Self { all: false, sets: List::uncapped() }
    }
    pub fn add(&mut self, inputs: &Inputs, rule: &LandingRule) {
        if by_path(rule, &inputs.sets) {
            for number in &rule.sets {
                if !self.sets.as_slice().contains(number) {
                    self.sets.push(*number).expect("rule sets fit memory");
                }
            }
        } else {
            self.all = true;
        }
    }
    pub fn over(&self, inputs: &Inputs) -> Box<[u8]> {
        if self.all || self.sets.is_empty() {
            return Box::default();
        }
        if let [number] = self.sets.as_slice() {
            for set in &inputs.sets {
                if set.number == *number {
                    let words = bytes::concat(&[b"if ", &set.name]);
                    if words.len() <= OVER && bytes::is_utf8(&words) {
                        return words;
                    }
                }
            }
        }
        b"by path".as_slice().into()
    }
}

pub(super) fn patterns(inputs: &Inputs, numbers: &[u8], limit: usize) -> Box<[u8]> {
    let mut ordered = List::uncapped();
    for number in numbers {
        if !ordered.as_slice().contains(number) {
            ordered.push(*number).expect("path sets fit memory");
        }
    }
    let mut ordered = ordered.into_boxed();
    ordered.sort_unstable();
    let mut patterns: List<&[u8]> = List::uncapped();
    for number in &ordered {
        for set in &inputs.sets {
            if set.number == *number {
                for pattern in &set.patterns {
                    if bytes::is_utf8(pattern) {
                        patterns.push(pattern).expect("patterns fit memory");
                    }
                }
            }
        }
    }
    let mut whole = Box::from(b"".as_slice());
    for pattern in &patterns {
        whole = bytes::concat(&[&whole, if whole.is_empty() { b"" } else { " · ".as_bytes() }, pattern]);
    }
    if whole.len() <= limit {
        return whole;
    }
    let mut words = Box::from(b"".as_slice());
    for pattern in &patterns {
        let joined = bytes::concat(&[&words, if words.is_empty() { b"" } else { " · ".as_bytes() }, pattern]);
        if joined.len().saturating_add(" · …".len()) > limit {
            break;
        }
        words = joined;
    }
    bytes::concat(&[&words, if words.is_empty() { "…".as_bytes() } else { " · …".as_bytes() }])
}
