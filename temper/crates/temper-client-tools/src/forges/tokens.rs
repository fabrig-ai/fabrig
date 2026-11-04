//! A tokens file's items (temper/docs/design/client/tools.md, 5.5).
use crate::jig::render::Text;
use alloc::boxed::Box;
use skein_lib::Secret;

pub(crate) struct Tokens<'a> {
    api: Option<&'a [u8]>,
    git: Option<&'a [u8]>,
    webhook_secret: Option<&'a [u8]>,
}
pub(crate) struct Error {
    line: Option<usize>,
    why: &'static [u8],
}
impl Error {
    fn at(index: usize, why: &'static [u8]) -> Self {
        Self { line: Some(index.checked_add(1).expect("bounded file")), why }
    }
    fn missing(why: &'static [u8]) -> Self {
        Self { line: None, why }
    }
    fn write(&self, out: &mut Text) {
        out.put(b"tokens file");
        if let Some(line) = self.line {
            out.put(b" line ");
            out.number(u64::try_from(line).expect("bounded file"));
            out.put(b":");
        }
        out.put(b" ");
        out.put(self.why);
    }
    pub(crate) fn words(&self) -> Box<[u8]> {
        let mut measure = Text::measure();
        self.write(&mut measure);
        let mut out = Text::writing_exact(measure);
        self.write(&mut out);
        out.finish()
    }
}
impl<'a> Tokens<'a> {
    pub(crate) fn read(file: &'a [u8], connect: bool) -> Result<Secret, Error> {
        Self::parse(file)?.bundle(connect)
    }
    pub(crate) fn parse(file: &'a [u8]) -> Result<Self, Error> {
        let mut tokens = Self { api: None, git: None, webhook_secret: None };
        let mut index = 0_usize;
        let mut from = 0_usize;
        loop {
            let end = skein_lib::bytes::find_from(file, b"\n", from).unwrap_or(file.len());
            let line = file.get(from..end).expect("line within file");
            let split = skein_lib::bytes::find(line, b" ");
            let name = match split {
                Some(at) => line.get(..at).expect("space within line"),
                None => line,
            };
            let slot = match name {
                b"api" => &mut tokens.api,
                b"git" => &mut tokens.git,
                b"webhook_secret" => &mut tokens.webhook_secret,
                _ => return Err(Error::at(index, b"unknown name")),
            };
            if slot.is_some() {
                return Err(Error::at(index, b"a name twice"));
            }
            let at = split.ok_or(Error::at(index, b"no value"))?;
            let value = line.get(at.checked_add(1).expect("bounded file")..).expect("value within line");
            if value.is_empty() {
                return Err(Error::at(index, b"no value"));
            }
            *slot = Some(value);
            if end == file.len() {
                break;
            }
            from = end.checked_add(1).expect("bounded file");
            index = index.checked_add(1).expect("bounded file");
        }
        Ok(tokens)
    }
    pub(crate) fn bundle(&self, connect: bool) -> Result<Secret, Error> {
        if connect {
            if self.api.is_none() {
                return Err(Error::missing(b"names no api"));
            }
            if self.git.is_none() {
                return Err(Error::missing(b"names no git"));
            }
        } else if self.api.is_none() && self.git.is_none() {
            return Err(Error::missing(b"names no token"));
        }
        temper_client_domain::forge::forges::pasted_tokens(self.api, self.git, self.webhook_secret)
            .ok_or(Error::missing(b"exceeds the carried value bound"))
    }
}
