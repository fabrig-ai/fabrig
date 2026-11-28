//! Whole charter files and their deployment's bounds (temper/docs/design/protocol/temper-dir.md, 3.3, 8–8.1).
use crate::{FILE_BYTES, judge::Report};
use alloc::boxed::Box;
use alloc::string::ToString;
use jig_domain_core_vocabulary::{CharterPart, limits::parts_owned_bytes};
use jig_domain_tasks::{REASON_NAMES, Reasons};
use skein_lib::{Decimal, List, Writer, bytes, len_u64};
use temper_charter_set::FormatFault;

/// Room left for a project charter by one named settings charter.
#[derive(Debug, Hash, PartialEq, Eq)]
pub struct CharterRoom {
    pub name: Box<[u8]>,
    /// None if even a charter with no project words cannot fit.
    pub bytes: Option<u64>,
    pub parts: u32,
}

/// The root's newest named settings charters and the bound on a project's set.
#[derive(Debug, Hash, PartialEq, Eq)]
pub struct DeploymentCharters {
    pub offered: Box<[CharterRoom]>,
    pub limit: u32,
}

#[expect(clippy::disallowed_methods, reason = "the protocol decodes charter text and locates invalid UTF-8")]
pub(crate) fn check(path: &[u8], body: &[u8], deployment: &DeploymentCharters, report: &mut Report) {
    let text = match core::str::from_utf8(body) {
        Ok(text) => text,
        Err(error) => {
            let prefix = body.get(..error.valid_up_to()).expect("the UTF-8 error gives its valid prefix");
            let before = bytes::count(prefix, b"\n", FILE_BYTES);
            report.add(path, before.saturating_add(1), bytes::copy_of(b"not UTF-8"));
            return;
        }
    };
    let mut reasons = [""; REASON_NAMES.len()];
    for (slot, name) in reasons.iter_mut().zip(REASON_NAMES) {
        *slot = core::str::from_utf8(name).expect("jig's reason names are ASCII");
    }
    let file = match temper_charter_set::read_charter(text, &reasons) {
        Ok(file) => file,
        Err(fault) => {
            report.add(path, fault.line, fault.to_string().into_bytes().into_boxed_slice());
            return;
        }
    };
    let stem = path
        .strip_prefix(b".temper/charters/")
        .expect("a charter file is in .temper/charters/")
        .strip_suffix(b".md")
        .expect("the listing held only NAME.md");
    let settings = match &file.settings {
        Some(name) => name.as_bytes(),
        None => stem,
    };
    for room in &deployment.offered {
        if room.name.as_ref() == settings {
            let parts = u32::try_from(file.parts.len()).expect("the file bounds its parts");
            let mut made = List::with_capacity(parts);
            for part in file.parts {
                let mut on = Reasons::EMPTY;
                for reason in &part.reasons {
                    on.0 |= Reasons::named(reason.as_bytes()).expect("the reader checked jig's names").0;
                }
                made.push(CharterPart { on, instructions: part.text.into_boxed_bytes() }).expect("parts counted");
            }
            let word_bytes = len_u64(file.instructions.len())
                .checked_add(parts_owned_bytes(made.as_slice()).expect("bounded file and parts"))
                .expect("bounded words");
            if let Some(bound) = room.bytes
                && word_bytes <= bound
                && parts <= room.parts
            {
                return;
            }
            report.add(path, 0, room_words(room, word_bytes, parts));
            return;
        }
    }
    let (line, words) = if file.settings.is_some() {
        // The shared overlay reader locates the key it refuses, including quoted TOML keys.
        let Err(FormatFault::Settings { line, .. }) = temper_charter_set::read_file(text) else {
            unreachable!("a valid project charter with `settings` is not an overlay file");
        };
        (line, bytes::concat(&[b"`settings` names `", settings, b"`, which this deployment's settings don't offer"]))
    } else {
        (0, bytes::concat(&[b"no `settings`, and this deployment offers no charter `", settings, b"`"]))
    };
    report.add(path, line, words);
}

pub(crate) fn check_count(count: u32, deployment: &DeploymentCharters, report: &mut Report) {
    if count > deployment.limit {
        report.add(
            b".temper/charters/",
            0,
            bytes::concat(&[
                Decimal::of(u64::from(count)).as_bytes(),
                b" charters; this deployment's tasks hold at most ",
                Decimal::of(u64::from(deployment.limit)).as_bytes(),
            ]),
        );
    }
}

fn room_words(room: &CharterRoom, bytes: u64, parts: u32) -> Box<[u8]> {
    bytes::concat(&[
        b"its words take ",
        &grouped(bytes),
        b" bytes and ",
        &grouped(u64::from(parts)),
        b" parts; charter `",
        &room.name,
        b"` leaves them ",
        &grouped(room.bytes.unwrap_or(0)),
        b" bytes and ",
        &grouped(u64::from(room.parts)),
        b" parts",
    ])
}

fn grouped(value: u64) -> Box<[u8]> {
    let digits = Decimal::of(value);
    let digits = digits.as_bytes();
    let commas = digits.len().saturating_sub(1).checked_div(3).expect("nonzero group size");
    let mut out = Writer::new(digits.len().saturating_add(commas));
    for (index, &digit) in digits.iter().enumerate() {
        if index != 0 && digits.len().saturating_sub(index).is_multiple_of(3) {
            out.put(b",").expect("comma counted");
        }
        out.put(&[digit]).expect("digit counted");
    }
    out.finish()
}
