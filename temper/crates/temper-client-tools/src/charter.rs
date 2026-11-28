//! Charter copy's words, before a host touches the filesystem (temper/docs/design/client/ctl/tools-reads.md, 1).
use crate::jig::{
    FileEnd, Printed,
    call::{Call, Value, number, value},
    render,
    runner::FileCopy,
    table::{
        Keyed, Kind,
        Occurs::{Optional, Required},
        Tool, flag, positional,
    },
};
use alloc::boxed::Box;
use jig_client::contract as c;
use skein_lib::{Decimal, List, bytes::concat};
use temper_charter_set::{CharterFault, WritePart, write_charter};

const FILE_BYTES: usize = 65_536; // protocol/temper-dir.md, 3.3.
pub const CHARTER_COPY_AT: u8 = 124;
pub(crate) const CHARTER_COPY: Tool = Tool {
    name: b"charter copy",
    line: b"Copy a deployment charter's words into a project file",
    prints: b"The path written and its length in bytes",
    result: b"charter_file",
    forms: &[&[
        positional(
            b"number",
            b"NUMBER",
            Kind::NameOrNumber { min: 0, max: 4_294_967_295 },
            Required,
            b"The charter's name or number",
        ),
        flag(b"into", b"DIR", Kind::Literal, Optional, b"The destination directory; the working directory when absent"),
    ]],
    keyed: Keyed::Never,
    area: 1,
    notes: b"Writes .temper/charters/NAME.md without the deployment's overlay.
Never writes over a file already there. Takes its destination from command-line arguments, not --call.
",
};
fn copy(call: &Call, document: &c::SettingDocument, json: bool) -> Result<FileCopy, Box<[u8]>> {
    let number = number(call, 0).expect("resolved charter number");
    let c::Setting::Charter(key) = document.setting else { return Err(b"wrong setting kind".as_slice().into()) };
    if u64::from(key.number) != number {
        return Err(b"wrong charter in setting read".as_slice().into());
    }
    let Some(c::SettingValue::Charter(charter)) = &document.value else { return Err(b"unknown".as_slice().into()) };
    let Some(name) = &charter.name else {
        return Err(refusal(number, b" has no name"));
    };
    if !is_charter_name(name) {
        return Err(refusal(number, b" has no valid file name"));
    }
    let bytes = words(charter, document.overlay.as_deref())?;
    let into = if let Some(Value::Text(path)) = value(call, 1) { Some(path.clone()) } else { None };
    Ok(FileCopy { name: name.clone(), into, bytes, json })
}

// A setting's name becomes a file stem, so check the charter file's name grammar.
fn is_charter_name(name: &[u8]) -> bool {
    let Some((first, rest)) = name.split_first() else { return false };
    if !first.is_ascii_lowercase() || name.len() > 32 {
        return false;
    }
    for byte in rest {
        if !byte.is_ascii_lowercase() && !byte.is_ascii_digit() && *byte != b'-' {
            return false;
        }
    }
    true
}

#[expect(clippy::disallowed_methods, reason = "jig's Text already guarantees UTF-8; the charter writer takes str")]
fn words(charter: &c::Charter, overlay: Option<&c::Overlay>) -> Result<Box<[u8]>, Box<[u8]>> {
    let instructions = charter.instructions.bytes();
    let parts = charter.parts.as_slice();
    let (instructions, parts) = if let Some(overlay) = overlay {
        let Some(base) = crate::jig::settings_file::without_overlay(
            instructions,
            parts,
            overlay.instructions.bytes(),
            overlay.parts.as_slice(),
        ) else {
            return Err(refusal(u64::from(charter.number), b" does not end with its overlay's words"));
        };
        base
    } else {
        (instructions, parts)
    };
    let count = u32::try_from(parts.len()).expect("jig list length");
    let mut reasons = List::with_capacity(count);
    for part in parts {
        let mut selected = List::uncapped();
        for (index, name) in c::REASON_NAMES.iter().enumerate() {
            if part.on & (1u32 << u32::try_from(index).expect("reason bit")) != 0 {
                selected.push(*name).expect("bounded reasons");
            }
        }
        if part.on == 0 || part.on >> u32::try_from(c::REASON_NAMES.len()).expect("reason bits") != 0 {
            let part = Decimal::of(u64::from(reasons.len()).checked_add(1).expect("part index"));
            let why = concat(&[b" part ", part.as_bytes(), b" has invalid reasons"]);
            return Err(refusal(u64::from(charter.number), &why));
        }
        reasons.push(selected).expect("one reason list per part");
    }
    let mut writing = List::with_capacity(count);
    for (part, selected) in parts.iter().zip(&reasons) {
        writing
            .push(WritePart {
                reasons: selected.as_slice(),
                text: core::str::from_utf8(part.instructions.bytes()).expect("jig text is UTF-8"),
            })
            .expect("one write part per charter part");
    }
    let instructions = core::str::from_utf8(instructions).expect("jig texts split at UTF-8 boundaries");
    let mut file = match write_charter(instructions, writing.as_slice()) {
        Ok(file) => file,
        Err(why) => return Err(writer_refusal(u64::from(charter.number), &why)),
    };
    // The final line feed is framing, unless the file already occupies the form's bound.
    if !file.is_empty() && file.len() < FILE_BYTES {
        file.push('\n');
    }
    Ok(file.into_bytes().into_boxed_slice())
}

fn refusal(number: u64, why: &[u8]) -> Box<[u8]> {
    concat(&[b"charter ", Decimal::of(number).as_bytes(), why])
}

#[expect(clippy::disallowed_methods, reason = "CharterFault exposes its diagnostic words through Display")]
fn writer_refusal(number: u64, why: &CharterFault) -> Box<[u8]> {
    use alloc::string::ToString as _;
    let words = why.to_string();
    let detail = if why.line == 0 {
        concat(&[b": ", words.as_bytes()])
    } else {
        concat(&[b":", Decimal::of(u64::from(why.line)).as_bytes(), b": ", words.as_bytes()])
    };
    refusal(number, &detail)
}

pub(crate) fn print(
    reading: &crate::jig::runner::reads::Reading,
    client: &jig_client::Client,
    me: jig_client::Handle,
    limits: &crate::jig::Limits,
) -> Printed {
    use jig_client::Readout;
    let page = reading.pages.get(&1).or(reading.pages.get(&6)).expect("setting read answered");
    let json = reading.called.json;
    match page {
        Readout::Jig(c::ReadResult::Value(c::ReadValue::Setting(document))) => {
            match copy(&reading.called.call, document, json) {
                Ok(file) => Printed {
                    file: FileEnd::WriteNew { file: Box::new(file) },
                    status: 0,
                    stdout: Box::default(),
                    stderr: Box::default(),
                },
                Err(words) => render::failure(&words, json, 1),
            }
        }
        Readout::Jig(c::ReadResult::None(c::ReadNone::Unknown)) => render::failure(b"unknown", json, 1),
        Readout::Jig(c::ReadResult::Value(_)) | Readout::Family { .. } => {
            render::failure(b"unexpected document reading a charter", json, 1)
        }
        Readout::Jig(c::ReadResult::None(_)) | Readout::Limit | Readout::NotServed { .. } => {
            render::read(page, client, me, json, limits, reading.parts)
        }
    }
}
