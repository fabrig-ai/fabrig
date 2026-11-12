//! Drive's lines, sharing the journeys' field codec (client/drive.md, 3–5).
use skein_ui::{Act, Role};
use skein_ui_screen_terminal::Key;
use temper_client_shell_web::journey::{Field, Fields, read_fields};

#[derive(Debug, PartialEq, Eq)]
pub struct Line {
    source: Box<[u8]>,
    record: Record,
}
#[derive(Debug, PartialEq, Eq)]
pub enum Record {
    Empty,
    End,
    Web(Web),
    Tui(Tui),
    Ctl(Vec<Box<[u8]>>),
}
#[derive(Debug, PartialEq, Eq)]
pub enum Web {
    Act { role: Role, name: Box<[u8]>, order: u32, act: Act },
    See(Box<[u8]>),
    Reload(Box<[u8]>),
    Look(Option<u32>),
}
#[derive(Debug, PartialEq, Eq)]
pub enum Tui {
    Key(Key),
    See(Box<[u8]>),
    Stop,
    Look,
}
impl Line {
    /// Parse one line without a terminating newline.
    pub fn parse(source: &[u8]) -> Result<Self, String> {
        let mut fields = read_fields(source).map_err(str::to_owned)?;
        let record = if source.is_empty() {
            Record::Empty
        } else {
            match word(&mut fields, "client or end")?.as_ref() {
                b"end" => Record::End,
                b"web" => Record::Web(web(&mut fields)?),
                b"tui" => Record::Tui(tui(&mut fields)?),
                b"ctl" => {
                    if fields.remaining() == 0 {
                        return Err("missing ctl argument".into());
                    }
                    let mut args = vec![];
                    while let Some(field) = fields.field() {
                        args.push(match field {
                            Field::Word(bytes) | Field::Bytes(bytes) => bytes,
                        });
                    }
                    Record::Ctl(args)
                }
                other => {
                    return Err(format!(
                        "unknown client {:?}; expected web, tui, ctl or end",
                        String::from_utf8_lossy(other)
                    ));
                }
            }
        };
        if let Some(extra) = fields.field() {
            return Err(format!("extra field: {extra:?}"));
        }
        Ok(Self { source: source.into(), record })
    }
    /// The exact parsed spelling, without a newline.
    #[must_use]
    pub fn print(&self) -> &[u8] {
        &self.source
    }
    #[must_use]
    pub fn record(&self) -> &Record {
        &self.record
    }
    #[must_use]
    pub fn kept(&self) -> bool {
        !matches!(self.record, Record::Empty | Record::End | Record::Web(Web::Look(_)) | Record::Tui(Tui::Look))
    }
}
fn word(fields: &mut Fields, name: &str) -> Result<Box<[u8]>, String> {
    fields.word().map_err(|why| format!("expected {name}: {why}"))
}
fn bytes(fields: &mut Fields, name: &str) -> Result<Box<[u8]>, String> {
    fields.bytes().map_err(|why| format!("expected {name}: {why}"))
}
fn web(fields: &mut Fields) -> Result<Web, String> {
    let record = word(fields, "web record")?;
    Ok(match record.as_ref() {
        b"press" | b"submit" | b"toggle" | b"input" => {
            let role = match word(fields, "role")?.as_ref() {
                b"button" => Role::Button,
                b"link" => Role::Link,
                b"textbox" => Role::TextField,
                b"checkbox" => Role::Check,
                other => return Err(format!("unknown role {:?}", String::from_utf8_lossy(other))),
            };
            let name = bytes(fields, "control name")?;
            let act = match record.as_ref() {
                b"press" => Act::Press,
                b"submit" => Act::Submit,
                b"toggle" => Act::Toggle,
                b"input" => Act::Input { value: bytes(fields, "input value")? },
                _ => unreachable!("act selected"),
            };
            let order = if fields.remaining() == 0 {
                0
            } else {
                fields.count().map_err(|why| format!("expected order: {why}"))?
            };
            Web::Act { role, name, order, act }
        }
        b"see" => Web::See(bytes(fields, "text")?),
        b"reload" => {
            let path = bytes(fields, "path and query")?;
            if !path.starts_with(b"/") || path.starts_with(b"//") || path.contains(&b'\\') {
                return Err("reload needs a path on the box's origin, starting with one /".into());
            }
            Web::Reload(path)
        }
        b"look" => {
            let width = if fields.remaining() == 0 {
                None
            } else {
                let width = fields.count().map_err(|why| format!("expected look width: {why}"))?;
                if !(320..=3840).contains(&width) {
                    return Err("look width must be 320 to 3840".into());
                }
                Some(width)
            };
            Web::Look(width)
        }
        other => return Err(format!("unknown web record {:?}", String::from_utf8_lossy(other))),
    })
}
fn tui(fields: &mut Fields) -> Result<Tui, String> {
    Ok(match word(fields, "tui record")?.as_ref() {
        b"key" => Tui::Key(key(fields)?),
        b"see" => Tui::See(bytes(fields, "text")?),
        b"stop" => Tui::Stop,
        b"look" => Tui::Look,
        other => return Err(format!("unknown tui record {:?}", String::from_utf8_lossy(other))),
    })
}
fn key(fields: &mut Fields) -> Result<Key, String> {
    Ok(match word(fields, "key")?.as_ref() {
        b"enter" => Key::Enter,
        b"tab" => Key::Tab,
        b"backtab" => Key::BackTab,
        b"backspace" => Key::Backspace,
        b"delete" => Key::Delete,
        b"up" => Key::Up,
        b"down" => Key::Down,
        b"left" => Key::Left,
        b"right" => Key::Right,
        b"home" => Key::Home,
        b"end" => Key::End,
        b"pageup" => Key::PageUp,
        b"pagedown" => Key::PageDown,
        b"escape" => Key::Escape,
        b"char" => Key::Char(character(fields)?),
        b"alt" => Key::Alt(character(fields)?),
        b"paste" => Key::Paste(bytes(fields, "key's pasted bytes")?),
        b"control" => {
            let letter = bytes(fields, "control key's ASCII letter")?;
            let [letter] = letter.as_ref() else {
                return Err("control key needs one ASCII letter".into());
            };
            if !letter.is_ascii_alphabetic() {
                return Err("control key needs one ASCII letter".into());
            }
            Key::Control(*letter)
        }
        other => return Err(format!("unknown key {:?}", String::from_utf8_lossy(other))),
    })
}
fn character(fields: &mut Fields) -> Result<char, String> {
    let bytes = bytes(fields, "key's character")?;
    let text = std::str::from_utf8(&bytes).map_err(|_| "key needs one UTF-8 character")?;
    let mut chars = text.chars();
    let ch = chars.next().ok_or("key needs one character")?;
    if chars.next().is_some() {
        return Err("key needs one character".into());
    }
    Ok(ch)
}
#[cfg(test)]
mod tests;
