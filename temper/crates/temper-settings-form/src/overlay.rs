//! Directory inputs and literal overlays (client/ctl/overlays.md, 2 and 3).
use crate::{Limits, view as c};
use alloc::{boxed::Box, format};
use skein_lib::{List, Writer};
use temper_charter_set::{File, Set};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DirectoryName {
    /// Item numbers are counted from one; directory indices from zero.
    pub item: u32,
    pub index: u32,
    pub path: Box<[u8]>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Markdown {
    pub name: Box<[u8]>,
    pub text: Box<[u8]>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Directory {
    pub path: Box<[u8]>,
    pub files: List<Markdown>,
}
#[must_use]
pub fn directories(items: &List<c::SettingItem>) -> List<DirectoryName> {
    let mut out = List::uncapped();
    for (item, value) in items.iter().enumerate() {
        if let c::SettingEdit::Set(set) = &value.edit
            && let c::SettingValue::Overlay(overlay) = &set.value
        {
            let mut parts = List::uncapped();
        for part in &overlay.parts { parts.push(part.clone()).expect("bounded parts"); }
        for (index, path) in overlay.charters.iter().enumerate() {
                out.push(DirectoryName {
                    item: u32::try_from(item).expect("bounded item").checked_add(1).expect("item number"),
                    index: u32::try_from(index).expect("bounded directory"),
                    path: path.clone(),
                }).expect("input bounded");
            }
        }
    }
    out
}
/// Context shared by host read failures and the pure assembly's refusals.
#[expect(clippy::disallowed_macros, reason = "input boundary renders item and directory diagnostics")]
#[must_use]
pub fn refusal(name: &DirectoryName, why: &str) -> Box<[u8]> {
    format!("settings file item {}: overlay.charters[{}]: \"{}\": {why}", name.item, name.index,
        core::str::from_utf8(&name.path).unwrap_or("<non-UTF-8 directory>")).into_bytes().into_boxed_slice()
}
/// Assemble every named set, including the files the selected charter does not take.
pub fn assemble(items: &mut List<c::SettingItem>, inputs: &[Directory], limits: &Limits) -> Result<(), Box<[u8]>> {
    for (item, value) in items.iter_mut().enumerate() {
        let c::SettingEdit::Set(set) = &mut value.edit else { continue };
        let c::SettingValue::Overlay(overlay) = &mut set.value else { continue };
        let c::Setting::Charter(number) = value.setting else { unreachable!("overlay charter checked") };
        let mut parts = List::uncapped();
        for part in &overlay.parts { parts.push(part.clone()).expect("bounded parts"); }
        for (index, path) in overlay.charters.iter().enumerate() {
            let name = DirectoryName {
                item: u32::try_from(item).expect("item count").checked_add(1).expect("item number"),
                index: u32::try_from(index).expect("directory count"), path: path.clone(),
            };
            let Some(directory) = inputs.iter().find(|input| input.path == *path) else {
                return Err(refusal(&name, "directory: could not be read"));
            };
            let mut set = Set { common: None, goal: None, designer: None, coordinator: None, implementer: None };
            for file in &directory.files {
                let file_name = core::str::from_utf8(&file.name).or(Err(refusal(&name, "file: UTF-8 name required")))?;
                let selected = match temper_charter_set::file(file_name) {
                    Ok(file) => file,
                    Err(why) => return Err(assembly_refusal(&name, &why)),
                };
                let text = core::str::from_utf8(&file.text).or(Err(refusal(&name, "file: UTF-8 text required")))?;
                match selected {
                    Some(File::Common) => set.common = Some(text),
                    Some(File::Goal) => set.goal = Some(text),
                    Some(File::Designer) => set.designer = Some(text),
                    Some(File::Coordinator) => set.coordinator = Some(text),
                    Some(File::Implementer) => set.implementer = Some(text),
                    None => {}
                }
            }
            let goal = temper_charter_set::goal_agent(&set, reason).map_err(|why| assembly_refusal(&name, &why))?;
            let instructions = if number.number == 5 { goal.instructions } else { temper_charter_set::implementer(&set) };
            overlay.instructions = join(&overlay.instructions, &instructions, limits.settings.charter_instructions)?;
            if number.number == 5 {
                for part in &goal.parts {
                    let mut on = 0;
                    for bit in &part.reasons { on |= bit; }
                    let instructions = c::Text::new(part.text.clone(), limits.settings.charter_instructions.min(limits.settings.charter_part_instructions))
                        .or(Err(refusal(&name, "part instructions: bound")))?;
                    parts.push(c::CharterPart { on, instructions }).expect("input bounded");
                }
            }
        }
        overlay.parts = parts;
        overlay.charters = List::with_capacity(0);
        if overlay.parts.len() > limits.settings.charter_parts {
            return Err(Box::from(b"settings file: overlay.parts: tasks.charter_parts bound".as_slice()));
        }
        if overlay.resume == Some(true) && !overlay.parts.is_empty() {
            return Err(Box::from(b"settings file: overlay.parts: a charter that resumes takes no parts".as_slice()));
        }
    }
    Ok(())
}
#[expect(clippy::disallowed_macros, reason = "the assembly owns its refusal words")]
fn assembly_refusal(name: &DirectoryName, why: &temper_charter_set::Refusal) -> Box<[u8]> {
    refusal(name, &format!("{why}"))
}
fn reason(name: &str) -> Option<u32> {
    for (index, candidate) in c::REASON_NAMES.iter().enumerate() {
        if candidate == &name.as_bytes() {
            return 1_u32.checked_shl(u32::try_from(index).expect("reason bit"));
        }
    }
    None
}
pub fn join(first: &c::Text, second: &[u8], maximum: u32) -> Result<c::Text, Box<[u8]>> {
    let first = trim(first.bytes());
    let second = trim(second);
    let separator = if first.is_empty() || second.is_empty() { b"".as_slice() } else { b"\n\n" };
    let length = first.len().checked_add(separator.len()).and_then(|n| n.checked_add(second.len()))
        .ok_or(Box::from(b"settings file: overlay.instructions: bound".as_slice()))?;
    if length > usize::try_from(maximum).expect("instruction bound") {
        return Err(Box::from(b"settings file: overlay.instructions: bound".as_slice()));
    }
    let mut out = Writer::new(length);
    for bytes in [first, separator, second] { out.put(bytes).expect("measured text"); }
    c::Text::new(out.finish(), maximum)
}
fn trim(mut bytes: &[u8]) -> &[u8] {
    while let Some(rest) = bytes.strip_prefix(b"\n") { bytes = rest; }
    while let Some(rest) = bytes.strip_suffix(b"\n") { bytes = rest; }
    bytes
}
