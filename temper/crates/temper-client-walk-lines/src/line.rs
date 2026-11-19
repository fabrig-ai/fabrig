//! Walk lines and page printers (temper/docs/design/client/journey-walks.md, 3.1).
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
    Forge(Forge),
    Deployment(Deployment),
}
#[derive(Debug, PartialEq, Eq)]
pub enum Web {
    Act { role: Role, name: Box<[u8]>, order: u32, act: Act },
    See(Box<[u8]>),
    Reload(Box<[u8]>),
    Look(Option<u32>),
    Requests,
    Mark,
}
#[derive(Debug, PartialEq, Eq)]
pub enum Tui {
    Key(Key),
    See(Box<[u8]>),
    Stop,
    Look,
    Mark,
    Approve,
}
#[derive(Debug, PartialEq, Eq)]
pub enum Forge {
    Commit { repository: Box<[u8]>, branch: Option<Box<[u8]>>, message: Box<[u8]>, paths: Vec<Box<[u8]>> },
    Checks { repository: Box<[u8]> },
}
/// A deployment's configuration change (client/drive.md, 3.8).
#[derive(Debug, PartialEq, Eq)]
pub enum Deployment {
    Patch { document: Document, path: Box<[u8]> },
    Restart,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Document {
    Engine,
    Worker,
}
impl Document {
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Engine => "engine",
            Self::Worker => "worker",
        }
    }
    /// Relative to the scratch directory (testing/qa/deployment.md, 3).
    #[must_use]
    pub fn path(self) -> std::path::PathBuf {
        std::path::Path::new("config").join(self.name()).join(temper_config_shell::FILE)
    }
}
/// Names substituted only in ctl arguments (client/drive.md, 3.4).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BoxValue {
    Api,
    Git,
    Admin,
    GitProgram,
    ModelKey,
}
impl BoxValue {
    #[must_use]
    pub fn parse(bytes: &[u8]) -> Option<Self> {
        match bytes {
            b"{forgejo.api}" => Some(Self::Api),
            b"{forgejo.git}" => Some(Self::Git),
            b"{forgejo.admin}" => Some(Self::Admin),
            b"{git}" => Some(Self::GitProgram),
            b"{model.key}" => Some(Self::ModelKey),
            _ => None,
        }
    }
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
                b"forge" => Record::Forge(forge(&mut fields)?),
                b"deployment" => Record::Deployment(deployment(&mut fields)?),
                b"ctl" => {
                    if fields.remaining() == 0 {
                        return Err("missing ctl argument".into());
                    }
                    let mut args = vec![];
                    while let Some(field) = fields.field() {
                        let (Field::Word(bytes) | Field::Bytes(bytes)) = field;
                        if bytes.starts_with(b"{") && bytes.ends_with(b"}") && BoxValue::parse(&bytes).is_none() {
                            return Err(format!("unknown box value {}", String::from_utf8_lossy(&bytes)));
                        }
                        args.push(bytes);
                    }
                    for path in settings_paths(&args) {
                        relative_path(path.into())?;
                    }
                    Record::Ctl(args)
                }
                other => {
                    return Err(format!(
                        "unknown client {:?}; expected web, tui, ctl, forge, deployment or end",
                        String::from_utf8_lossy(other)
                    ));
                }
            }
        };
        if let Some(extra) = fields.field() {
            let (Field::Word(extra) | Field::Bytes(extra)) = extra;
            return Err(format!("extra field: {:?}", String::from_utf8_lossy(&extra)));
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
fn deployment(fields: &mut Fields) -> Result<Deployment, String> {
    let record = word(fields, "deployment record: patch or restart")?;
    if record.as_ref() == b"restart" {
        return Ok(Deployment::Restart);
    }
    if record.as_ref() != b"patch" {
        return Err(format!(
            "unknown deployment record {}; expected patch or restart",
            String::from_utf8_lossy(&record)
        ));
    }
    let name = word(fields, "DOCUMENT engine or worker")?;
    let document = match name.as_ref() {
        b"engine" => Document::Engine,
        b"worker" => Document::Worker,
        _ => {
            return Err(format!("unknown DOCUMENT {}; expected engine or worker", String::from_utf8_lossy(&name)));
        }
    };
    let field = fields.field().ok_or("expected PATH")?;
    let (Field::Word(path) | Field::Bytes(path)) = field;
    let path = relative_path(path)?;
    Ok(Deployment::Patch { document, path })
}
/// Settings inputs before the first flag; stdin remains closed even for `-`.
pub fn settings_paths(args: &[Box<[u8]>]) -> Vec<&[u8]> {
    if args.first().map(AsRef::as_ref) != Some(b"settings")
        || !args.get(1).is_some_and(|arg| [b"apply".as_slice(), b"propose"].contains(&arg.as_ref()))
    {
        return vec![];
    }
    args.iter()
        .skip(2)
        .take_while(|arg| arg.as_ref() == b"-" || !arg.starts_with(b"-"))
        .filter(|arg| arg.as_ref() != b"-")
        .map(AsRef::as_ref)
        .collect()
}
fn relative_path(path: Box<[u8]>) -> Result<Box<[u8]>, String> {
    if path.is_empty()
        || path.len() > 256
        || path.split(|b| *b == b'/').any(|p| p.is_empty() || p == b"." || p == b"..")
    {
        return Err("expected relative PATH of 1 to 256 bytes without empty, . or .. segments".into());
    }
    Ok(path)
}
fn forge(fields: &mut Fields) -> Result<Forge, String> {
    let record = word(fields, "forge record: commit, push or checks")?;
    if record.as_ref() != b"commit" && record.as_ref() != b"push" && record.as_ref() != b"checks" {
        return Err("unknown forge record; expected commit, push or checks".into());
    }
    if word(fields, "FORGE forgejo")?.as_ref() != b"forgejo" {
        return Err("expected FORGE forgejo".into());
    }
    let repository = word(fields, "REPOSITORY OWNER/NAME")?;
    let parts: Vec<_> = repository.split(|byte| *byte == b'/').collect();
    if parts.len() != 2 || parts.iter().any(|part| part.is_empty() || *part == b"." || *part == b"..") {
        return Err("expected REPOSITORY OWNER/NAME".into());
    }
    if record.as_ref() == b"checks" {
        return Ok(Forge::Checks { repository });
    }
    let branch = if record.as_ref() == b"push" { Some(word(fields, "BRANCH")?) } else { None };
    let message = bytes(fields, "MESSAGE")?;
    let mut paths = vec![];
    while fields.remaining() != 0 {
        let path = relative_path(bytes(fields, "PATH")?)?;
        if paths.contains(&path) {
            return Err("expected distinct PATHs".into());
        }
        paths.push(path);
    }
    if !(1..=16).contains(&paths.len()) {
        return Err("expected 1 to 16 PATHs".into());
    }
    Ok(Forge::Commit { repository, branch, message, paths })
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
        b"requests" => Web::Requests,
        b"mark" => Web::Mark,
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
        b"mark" => Tui::Mark,
        b"approve" => Tui::Approve,
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
