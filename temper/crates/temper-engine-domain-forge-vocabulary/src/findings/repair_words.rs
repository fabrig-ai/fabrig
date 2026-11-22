//! Repair goals keep the act whole and omit only whole findings (domain/red-repairs.md, 4.4).
use super::{Finding, FindingWrite};
use alloc::boxed::Box;
use skein_lib::{Decimal, List, Set};

pub fn repair_words(
    repository: &[u8],
    findings: &[FindingWrite],
    words: &[u8],
    attempts: u8,
    limit: u32,
) -> Option<Box<[u8]>> {
    let mut tiers = List::with_capacity(16);
    let mut ids = Set::with_capacity(16_u32.saturating_mul(super::FindingSignature::IDS));
    let mut many = false;
    for write in findings {
        let signature = write.row.signature.as_ref().expect("landed red signature");
        if !tiers.as_slice().contains(&signature.source) {
            tiers.push(signature.source.clone()).expect("sixteen tiers");
        }
        many |= signature.many;
        for id in &signature.ids {
            ids.insert(id.clone()).expect("distinct test ids");
        }
    }
    let mut tier = List::uncapped();
    for (index, name) in tiers.as_slice().iter().enumerate() {
        if index != 0 {
            append(&mut tier, b", ");
        }
        append(&mut tier, name);
    }
    let tier = tier.into_boxed();
    let mut out = List::uncapped();
    append(&mut out, b"Repair ");
    if many {
        append(&mut out, b"many tests");
    } else {
        append(&mut out, Decimal::of(u64::from(ids.len())).as_bytes());
        append(&mut out, if ids.len() == 1 { b" test" } else { b" tests" });
    }
    append(&mut out, b" failing in ");
    let room = 60_usize.saturating_sub(out.as_slice().len());
    let mut characters = 0_usize;
    for &byte in &tier {
        if byte & 0xc0 != 0x80 {
            characters = characters.saturating_add(1);
        }
    }
    if characters <= room {
        append(&mut out, &tier);
    } else {
        let mut kept = 0_usize;
        for &byte in &tier {
            if byte & 0xc0 != 0x80 {
                if kept == room.saturating_sub(1) {
                    break;
                }
                kept = kept.saturating_add(1);
            }
            out.push(byte).expect("bounded tier title");
        }
        append(&mut out, "…".as_bytes());
    }
    for part in [
        b"\n\n".as_slice(),
        words,
        b"\n\nRepository: ",
        repository,
        b". Tier: ",
        &tier,
        b".\nName each finding a change repairs in its pull request's body, by its line:\n",
    ] {
        append(&mut out, part);
    }
    for write in findings {
        append(&mut out, b"Fixes-finding: ");
        append(&mut out, Decimal::of(u64::from(write.row.number)).as_bytes());
        append(&mut out, b"\n");
    }
    let mut sections = List::with_capacity(16);
    let mut length = out.as_slice().len();
    for write in findings {
        let section = finding(&write.row, attempts);
        length = length.saturating_add(section.len());
        sections.push(section).expect("sixteen findings");
    }
    let mut kept = findings.len();
    let mut suffix: Box<[u8]> = Box::new([]);
    if length > skein_lib::usize_of(limit) {
        for index in (0..findings.len()).rev() {
            let section = sections.as_slice().get(index).expect("one section per finding");
            length = length.saturating_sub(section.len());
            kept = index;
            suffix = omitted(findings.get(index..).expect("omitted suffix"));
            if length.saturating_add(suffix.len()) <= skein_lib::usize_of(limit) {
                break;
            }
        }
    }
    if length.saturating_add(suffix.len()) > skein_lib::usize_of(limit) {
        return None;
    }
    for section in sections.as_slice().get(..kept).expect("kept prefix") {
        append(&mut out, section);
    }
    append(&mut out, &suffix);
    Some(out.into_boxed())
}
fn finding(row: &Finding, attempts: u8) -> Box<[u8]> {
    let mut out = List::uncapped();
    append(&mut out, b"\nFinding ");
    address(&mut out, row);
    for part in [b": ".as_slice(), &row.title, b"\nAttempt "] {
        append(&mut out, part);
    }
    let attempt = match &row.repair {
        Some(repair) => repair.attempts.saturating_add(1),
        None => 1,
    };
    append(&mut out, Decimal::of(u64::from(attempt)).as_bytes());
    append(&mut out, b" of ");
    append(&mut out, Decimal::of(u64::from(attempts)).as_bytes());
    append(&mut out, b".\n");
    if let Some(repair) = &row.repair {
        append(&mut out, b"Still failing after goal ");
        append(&mut out, Decimal::of(repair.goal).as_bytes());
        append(&mut out, b": ");
        append(&mut out, Decimal::of(skein_lib::len_u64(repair.failing.len())).as_bytes());
        append(&mut out, b" of its tests.\n");
        for id in &repair.failing {
            append(&mut out, id);
            append(&mut out, b"\n");
        }
    }
    append(&mut out, &row.body);
    append(&mut out, b"\n");
    out.into_boxed()
}
fn omitted(findings: &[FindingWrite]) -> Box<[u8]> {
    let mut out = List::uncapped();
    append(&mut out, b"\nand more: ");
    for (index, write) in findings.iter().enumerate() {
        if index != 0 {
            append(&mut out, b", ");
        }
        append(&mut out, b"finding ");
        address(&mut out, &write.row);
    }
    out.into_boxed()
}
fn address(out: &mut List<u8>, row: &Finding) {
    append(out, Decimal::of(u64::from(row.project)).as_bytes());
    append(out, b"/");
    append(out, Decimal::of(u64::from(row.number)).as_bytes());
}
fn append(out: &mut List<u8>, words: &[u8]) {
    for &byte in words {
        out.push(byte).expect("repair goal words");
    }
}
