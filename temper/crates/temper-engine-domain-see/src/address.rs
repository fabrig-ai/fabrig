//! Address grammar in the chat's project (temper/docs/design/domain/agent/ask-temper.md, 1).
use alloc::boxed::Box;
use jig_domain_core::settings::Setting;
use skein_lib::{List, Writer};

/// One address, as its line parses in the chat's project (domain/agent/ask-temper.md, 1).
#[derive(PartialEq, Eq, Debug)]
pub enum Address {
    Ways,
    Inventory,
    Setting(Setting),
    Landing,
    Gates { repository: Box<[u8]> },
    Revisions,
    Revision { number: u32 },
    Proposal { proposer: jig_domain_people::Proposer, number: u64 },
    Page { name: Box<[u8]> },
    Search { words: Box<[u8]> },
    File { repository: Box<[u8]>, path: Box<[u8]>, branch: Option<Box<[u8]>> },
    Me,
    Project,
    Goals { ended: bool },
    Tasks,
    Task { number: u64 },
    Tree { number: u64 },
    Activity { number: u64 },
    Turn { number: u64, attempt: u64, turn: u32 },
    Change { number: u64 },
    Changes,
    Inbox,
    Unknown,
}

/// ASCII words; positive decimal numbers and bounded page and repository names.
#[expect(clippy::too_many_lines, reason = "one grammar table names every see address")]
#[must_use]
pub fn address(line: &[u8], project: u32) -> Address {
    let mut words = List::with_capacity(u32::try_from(line.len()).expect("address bound"));
    let mut from = 0_usize;
    for (at, byte) in line.iter().enumerate() {
        if byte.is_ascii_whitespace() {
            if from < at {
                words.push(line.get(from..at).expect("address word")).expect("admitted address words");
            }
            from = at.checked_add(1).expect("address offset");
        }
    }
    if from < line.len() {
        words.push(line.get(from..).expect("address word")).expect("admitted address words");
    }
    match words.as_slice() {
        [b"ways"] => Address::Ways,
        [b"settings"] => Address::Inventory,
        [b"settings", b"limits"] => Address::Setting(Setting::Limits),
        [b"settings", b"temper-limits"] => Address::Setting(Setting::Application { kind: 22, scope: 0, number: 1 }),
        [b"settings", b"rules"] => Address::Setting(Setting::Rules),
        [b"settings", b"welcome"] => Address::Setting(Setting::Application { kind: 20, scope: 0, number: 1 }),
        [b"settings", b"landing"] => Address::Landing,
        [b"settings", b"gates", repository] if repository_name(repository) => {
            Address::Gates { repository: (*repository).into() }
        }
        [b"settings", b"project-areas"] => {
            Address::Setting(Setting::Application { kind: 14, scope: project, number: 1 })
        }
        [b"settings", b"agents"] => Address::Setting(Setting::Application { kind: 21, scope: 0, number: 1 }),
        [b"settings", b"project-triager"] => {
            Address::Setting(Setting::Application { kind: 15, scope: project, number: 1 })
        }
        [b"settings", b"project-words"] => {
            Address::Setting(Setting::Application { kind: 10, scope: project, number: 1 })
        }
        [b"settings", b"project-ways"] => Address::Setting(Setting::Application { kind: 7, scope: project, number: 1 }),
        [b"settings", b"project-questions"] => {
            Address::Setting(Setting::Application { kind: 6, scope: project, number: 1 })
        }
        [b"settings", b"project-classes"] => {
            Address::Setting(Setting::Application { kind: 8, scope: project, number: 1 })
        }
        [b"settings", b"dependency", number] => match small(number) {
            Some(number) => Address::Setting(Setting::Application { kind: 13, scope: project, number }),
            None => Address::Unknown,
        },
        [b"settings", b"follow", number] => match small(number) {
            Some(number) => Address::Setting(Setting::Application { kind: 12, scope: project, number }),
            None => Address::Unknown,
        },
        [b"settings", b"charter", number] => setting_number(b"charter", number),
        [b"settings", b"template", number] => setting_number(b"template", number),
        [b"settings", b"policy", number] => setting_number(b"policy", number),
        [b"settings", b"model", number] => setting_number(b"model", number),
        [b"settings", b"worker", number] => setting_number(b"worker", number),
        [b"settings", b"provider", number] => setting_number(b"provider", number),
        [b"settings", b"charter-words", number] => setting_number(b"charter-words", number),
        [b"settings", b"forge-charter", number] => setting_number(b"forge-charter", number),
        [b"settings", b"template-goal", number] => setting_number(b"template-goal", number),
        [b"settings", b"landing-rule", number] => setting_number(b"landing-rule", number),
        [b"settings", b"landing-rule", number, b"project"] => match small(number) {
            Some(number) => Address::Setting(Setting::Application { kind: 2, scope: project, number }),
            None => Address::Unknown,
        },
        [b"revisions"] => Address::Revisions,
        [b"revision", number] => match small(number) {
            Some(number) => Address::Revision { number },
            None => Address::Unknown,
        },
        [b"proposal", name] => proposal_name(name),
        [b"help"] => Address::Page { name: b"index".as_slice().into() },
        [b"help", b"search", rest @ ..] if !rest.is_empty() => {
            let mut length = 0_usize;
            for word in rest {
                if length != 0 {
                    length = length.checked_add(1).expect("address bound");
                }
                length = length.checked_add(word.len()).expect("address bound");
            }
            let mut joined = Writer::new(length);
            let mut first = true;
            for word in rest {
                if !first {
                    joined.put(b" ").expect("address bound");
                }
                joined.put(word).expect("address bound");
                first = false;
            }
            Address::Search { words: joined.finish() }
        }
        [b"help", name] if *name != b"search" && page_name(name) => Address::Page { name: (*name).into() },
        [b"file", repository, path] if repository_name(repository) => {
            Address::File { repository: (*repository).into(), path: (*path).into(), branch: None }
        }
        [b"file", repository, path, branch] if repository_name(repository) => {
            Address::File { repository: (*repository).into(), path: (*path).into(), branch: Some((*branch).into()) }
        }
        [b"me"] => Address::Me,
        [b"project"] => Address::Project,
        [b"goals"] => Address::Goals { ended: false },
        [b"goals", b"ended"] => Address::Goals { ended: true },
        [b"tasks"] => Address::Tasks,
        [b"task", number] => match natural(number) {
            Some(number) => Address::Task { number },
            None => Address::Unknown,
        },
        [b"task", number, b"tree"] => match natural(number) {
            Some(number) => Address::Tree { number },
            None => Address::Unknown,
        },
        [b"task", number, b"activity"] => match natural(number) {
            Some(number) => Address::Activity { number },
            None => Address::Unknown,
        },
        [b"task", number, b"turn", turn] => {
            let Some(number) = natural(number) else { return Address::Unknown };
            match turn_number(turn) {
                Some((attempt, turn)) => Address::Turn { number, attempt, turn },
                None => Address::Unknown,
            }
        }
        [b"change", number] => match natural(number) {
            Some(number) => Address::Change { number },
            None => Address::Unknown,
        },
        [b"changes"] => Address::Changes,
        [b"inbox"] => Address::Inbox,
        _ => Address::Unknown,
    }
}

fn natural(word: &[u8]) -> Option<u64> {
    if word.is_empty() {
        return None;
    }
    let mut number = 0_u64;
    for byte in word {
        if !byte.is_ascii_digit() {
            return None;
        }
        number = number.checked_mul(10)?.checked_add(u64::from(byte.checked_sub(b'0')?))?;
    }
    if number == 0 { None } else { Some(number) }
}
fn small(word: &[u8]) -> Option<u32> {
    u32::try_from(natural(word)?).ok()
}
fn setting_number(kind: &[u8], word: &[u8]) -> Address {
    let Some(number) = small(word) else { return Address::Unknown };
    let setting = match kind {
        b"charter" => Setting::Charter(number),
        b"template" => Setting::Template(number),
        b"policy" => Setting::Policy(number),
        b"model" => Setting::Model(number),
        b"worker" => Setting::Worker(number),
        b"provider" => Setting::Provider(number),
        b"charter-words" => Setting::Application { kind: 9, scope: 0, number },
        b"forge-charter" => Setting::Application { kind: 3, scope: 0, number },
        b"template-goal" => Setting::Application { kind: 11, scope: 0, number },
        b"landing-rule" => Setting::Application { kind: 1, scope: 0, number },
        _ => unreachable!("setting grammar names its kinds"),
    };
    Address::Setting(setting)
}
fn page_name(name: &[u8]) -> bool {
    if name.is_empty() || name.len() > 32 {
        return false;
    }
    for byte in name {
        if !byte.is_ascii_lowercase() && !byte.is_ascii_digit() && *byte != b'-' {
            return false;
        }
    }
    true
}
fn pair(word: &[u8], separator: u8) -> Option<(&[u8], &[u8])> {
    let at = skein_lib::bytes::find(word, &[separator])?;
    let after = at.checked_add(1)?;
    let left = word.get(..at)?;
    let right = word.get(after..)?;
    if skein_lib::bytes::find(right, &[separator]).is_some() {
        return None;
    }
    Some((left, right))
}
fn repository_name(name: &[u8]) -> bool {
    match pair(name, b'/') {
        Some((owner, name)) => !owner.is_empty() && !name.is_empty(),
        None => false,
    }
}
fn turn_number(word: &[u8]) -> Option<(u64, u32)> {
    let (attempt, turn) = pair(word, b'.')?;
    Some((natural(attempt)?, small(turn)?))
}
fn proposal_name(name: &[u8]) -> Address {
    let Some((prefix, tail)) = name.split_first() else { return Address::Unknown };
    if let Some((who, number)) = pair(tail, b'/')
        && let (Some(who), Some(number)) = (natural(who), natural(number))
    {
        let proposer = match prefix {
            b't' => jig_domain_people::Proposer::Task(who),
            b'p' => jig_domain_people::Proposer::Party(who),
            _ => return Address::Unknown,
        };
        return Address::Proposal { proposer, number };
    }
    Address::Unknown
}

#[cfg(test)]
mod tests;
