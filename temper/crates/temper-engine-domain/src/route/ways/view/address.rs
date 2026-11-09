//! View addresses use ASCII spaces (temper/docs/design/client/README.md, 5.16).
use alloc::boxed::Box;
use jig_domain_people::Proposer;
use skein_lib::{List, bytes};

pub(super) enum Address {
    Way(Box<[u8]>),
    Path { repository: Box<[u8]>, branch: Option<Box<[u8]>>, revision: Option<u32> },
    Proposal { proposer: Proposer, number: u64 },
    Charter { number: u32, revision: Option<u32> },
    Rule { kind: u32, number: u32 },
    Roles,
    Unknown,
}

fn natural(word: &[u8]) -> Option<u32> {
    if word.is_empty() {
        return None;
    }
    let mut number = 0_u32;
    for byte in word {
        if !byte.is_ascii_digit() {
            return None;
        }
        number = number.checked_mul(10)?.checked_add(u32::from(byte.checked_sub(b'0')?))?;
    }
    if number == 0 { None } else { Some(number) }
}

fn repository_name(name: &[u8]) -> bool {
    let Some(at) = bytes::find(name, b"/") else { return false };
    let after = at.checked_add(1).expect("address offset");
    let owner = name.get(..at).expect("repository owner");
    let tail = name.get(after..).expect("repository name");
    !owner.is_empty() && !tail.is_empty() && bytes::find(tail, b"/").is_none()
}

pub(super) fn parse(line: &[u8]) -> Address {
    for byte in line {
        if byte.is_ascii_whitespace() && *byte != b' ' {
            return Address::Unknown;
        }
    }
    let mut words = List::uncapped();
    let mut from = 0;
    for (at, byte) in line.iter().enumerate() {
        if *byte == b' ' {
            if from < at {
                words.push(line.get(from..at).expect("address word")).expect("bounded address");
            }
            from = at.checked_add(1).expect("bounded address offset");
        }
    }
    if from < line.len() {
        words.push(line.get(from..).expect("address word")).expect("bounded address");
    }
    let mut words = words.as_slice();
    if let Some((first, names)) = words.split_first()
        && *first == b"way"
    {
        if names.is_empty() {
            return Address::Unknown;
        }
        let mut name = Box::from(b"".as_slice());
        for word in names {
            name = bytes::concat(&[&name, if name.is_empty() { b"" } else { b" " }, word]);
        }
        return Address::Way(name);
    }
    let mut revision = None;
    if let [rest @ .., b"@revision", number] = words {
        let Some(number) = natural(number) else { return Address::Unknown };
        revision = Some(number);
        words = rest;
    }
    match words {
        [word] if revision.is_none() && word.eq_ignore_ascii_case(b"roles") => Address::Roles,
        [b"path", repository] if repository_name(repository) => {
            Address::Path { repository: (*repository).into(), branch: None, revision }
        }
        [b"path", repository, branch] if repository_name(repository) && *branch != b"@revision" => {
            Address::Path { repository: (*repository).into(), branch: Some((*branch).into()), revision }
        }
        [b"charter", number] => match natural(number) {
            Some(number) => Address::Charter { number, revision },
            None => Address::Unknown,
        },
        [b"rule", word] if revision.is_none() => {
            let Some(at) = bytes::find(word, b"/") else { return Address::Unknown };
            let after = at.checked_add(1).expect("rule address offset");
            let kind = natural(word.get(..at).expect("rule kind"));
            let number = natural(word.get(after..).expect("rule number"));
            match (kind, number) {
                (Some(kind @ (1 | 2)), Some(number)) => Address::Rule { kind, number },
                _ => Address::Unknown,
            }
        }
        [b"proposal", ..] if revision.is_none() => match crate::route::see::address(line, 0) {
            crate::route::see::Address::Proposal { proposer, number } => Address::Proposal { proposer, number },
            crate::route::see::Address::Ways
            | crate::route::see::Address::Inventory
            | crate::route::see::Address::Setting(_)
            | crate::route::see::Address::Landing
            | crate::route::see::Address::Gates { .. }
            | crate::route::see::Address::Revisions
            | crate::route::see::Address::Revision { .. }
            | crate::route::see::Address::Page { .. }
            | crate::route::see::Address::Search { .. }
            | crate::route::see::Address::File { .. }
            | crate::route::see::Address::Me
            | crate::route::see::Address::Project
            | crate::route::see::Address::Goals { .. }
            | crate::route::see::Address::Tasks
            | crate::route::see::Address::Task { .. }
            | crate::route::see::Address::Tree { .. }
            | crate::route::see::Address::Activity { .. }
            | crate::route::see::Address::Turn { .. }
            | crate::route::see::Address::Change { .. }
            | crate::route::see::Address::Changes
            | crate::route::see::Address::Inbox
            | crate::route::see::Address::Unknown => Address::Unknown,
        },
        _ => Address::Unknown,
    }
}
