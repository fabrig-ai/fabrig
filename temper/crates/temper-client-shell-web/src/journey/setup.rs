//! The setup before a journey's recording (client/journeys.md, 8.2).
use super::Unreadable;
use skein_ui::{Act, Role};
#[derive(Debug, PartialEq, Eq)]
pub enum SetupStep {
    Act { role: Role, name: Box<[u8]>, order: u32, act: Act },
    See { text: Box<[u8]> },
}
#[derive(Debug, PartialEq, Eq)]
pub struct MarkWait {
    pub mark: u32,
    pub text: Box<[u8]>,
}
#[derive(Debug, PartialEq, Eq)]
pub struct Setup {
    pub after: Option<Box<[u8]>>,
    pub steps: Vec<SetupStep>,
    pub reload: Box<[u8]>,
    pub marks: Vec<MarkWait>,
}
/// Read a setup's ordered records (client/journeys.md, 8.2).
pub fn setup(file: &[u8]) -> Result<Setup, Unreadable> {
    let mut setup = Setup { after: None, steps: Vec::new(), reload: Box::new([]), marks: Vec::new() };
    let mut loaded = false;
    let mut count = 0;
    for (index, part) in file.split_inclusive(|byte| *byte == b'\n').enumerate() {
        count = u32::try_from(index + 1).expect("setup line count fits u32");
        let line = part.strip_suffix(b"\n").ok_or(Unreadable { line: count, why: "missing final newline" })?;
        read(&mut setup, &mut loaded, line, count).map_err(|why| Unreadable { line: count, why })?;
    }
    if !loaded {
        return Err(Unreadable { line: count + 1, why: "missing reload" });
    }
    Ok(setup)
}
fn read(setup: &mut Setup, loaded: &mut bool, line: &[u8], number: u32) -> Result<(), &'static str> {
    let mut fields = super::line::fields(line)?;
    let kind = fields.word()?;
    if number == 1 {
        if kind.as_ref() != b"setup" || fields.count()? != 1 {
            return Err("unsupported setup version");
        }
    } else {
        match kind.as_ref() {
            b"after" => {
                if number != 2 {
                    return Err("after must be second");
                }
                setup.after = Some(fields.bytes()?);
            }
            b"reload" => {
                if *loaded {
                    return Err("second reload");
                }
                setup.reload = fields.bytes()?;
                *loaded = true;
            }
            b"mark" => {
                if !*loaded {
                    return Err("mark before reload");
                }
                let mark = fields.count()?;
                if mark == 0 || setup.marks.last().is_some_and(|last| mark <= last.mark) {
                    return Err("marks must increase from one");
                }
                setup.marks.push(MarkWait { mark, text: fields.bytes()? });
            }
            b"see" => {
                if *loaded {
                    return Err("step after reload");
                }
                setup.steps.push(SetupStep::See { text: fields.bytes()? });
            }
            b"press" | b"submit" | b"toggle" | b"input" => {
                if *loaded {
                    return Err("step after reload");
                }
                let role = match fields.word()?.as_ref() {
                    b"button" => Role::Button,
                    b"link" => Role::Link,
                    b"textbox" => Role::TextField,
                    b"checkbox" => Role::Check,
                    _ => return Err("unknown setup role"),
                };
                let name = fields.bytes()?;
                let act = match kind.as_ref() {
                    b"press" => Act::Press,
                    b"submit" => Act::Submit,
                    b"toggle" => Act::Toggle,
                    b"input" => Act::Input { value: fields.bytes()? },
                    _ => unreachable!("act record selected above"),
                };
                let order = if fields.remaining() == 0 { 0 } else { fields.count()? };
                setup.steps.push(SetupStep::Act { role, name, order, act });
            }
            _ => return Err("unknown setup record"),
        }
    }
    if fields.remaining() != 0 {
        return Err("extra field");
    }
    Ok(())
}
