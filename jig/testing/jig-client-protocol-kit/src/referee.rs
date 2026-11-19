//! Independent observations of records, authentication and stream isolation (client/domain/client/README.md, 11).
use jig_client_protocol_wire::Record as _;
use jig_client_protocol_wire as wire;
use skein_lib::Reader;
use std::fmt::Write;

macro_rules! same {
    ($kind:ident, $encoded:expr, $crossed:expr) => {{
        let mut source = Reader::new($encoded);
        let mut received = Reader::new($crossed);
        let wanted = wire::$kind::decode(&wire::CEILINGS, &mut source).map_err(|_| Broken::Record)?;
        let actual = wire::$kind::decode(&wire::CEILINGS, &mut received).map_err(|_| Broken::Record)?;
        if source.remaining() != 0 || received.remaining() != 0 || actual != wanted {
            return Err(Broken::Record);
        }
        let encoded = actual.encode(&wire::CEILINGS).map_err(|_| Broken::Record)?;
        if encoded.as_ref() != $crossed {
            return Err(Broken::Record);
        }
    }};
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Broken {
    Record,
    Secret,
    Blocked,
}
#[derive(Default)]
pub struct Rules {
    secrets: Vec<String>,
    observed: String,
    pub records: u64,
    pub domains: u64,
    pub cuts: u64,
    pub isolated: u64,
}
impl Rules {
    pub fn secret(&mut self, bytes: &[u8]) {
        let rendered = format!("{bytes:?}");
        if !self.secrets.contains(&rendered) {
            self.secrets.push(rendered);
        }
    }
    /// Domain inputs are inspected as whole public values, never root state.
    /// # Errors
    /// Reports an authentication secret carried in that input.
    pub fn domain(&mut self, event: &impl std::fmt::Debug) -> Result<(), Broken> {
        if !self.secrets.is_empty() {
            self.observed.clear();
            write!(&mut self.observed, "{event:?}").expect("format a domain input");
            if self.secrets.iter().any(|form| self.observed.contains(form)) {
                return Err(Broken::Secret);
            }
        }
        self.domains += 1;
        Ok(())
    }
    /// # Errors
    /// Reports an altered or undecodable body after it crosses the stream.
    pub fn record(&mut self, path: &[u8], response: bool, encoded: &[u8], crossed: &[u8]) -> Result<(), Broken> {
        match (path, response) {
            (b"/jig/v1/request", false) => same!(Keyed, encoded, crossed),
            (b"/jig/v1/request", true) => same!(KeyedAnswer, encoded, crossed),
            (b"/jig/v1/batch", false) => same!(Batch, encoded, crossed),
            (b"/jig/v1/batch", true) => {
                if encoded != crossed {
                    return Err(Broken::Record);
                }
                let mut frames = Reader::new(crossed);
                while !frames.is_empty() {
                    let size = frames.u32().ok_or(Broken::Record)?;
                    let record = frames.bytes(size).ok_or(Broken::Record)?;
                    same!(BatchReply, record, record);
                }
            }
            (b"/jig/v1/read", false) => same!(Owned, encoded, crossed),
            (b"/jig/v1/read", true) => same!(ReadAnswer, encoded, crossed),
            _ => Self::watch_record(path, response, encoded, crossed)?,
        }
        self.records += 1;
        Ok(())
    }
    fn watch_record(path: &[u8], response: bool, encoded: &[u8], crossed: &[u8]) -> Result<(), Broken> {
        match (path, response) {
            (b"/jig/v1/watch", false) => same!(Open, encoded, crossed),
            (b"/jig/v1/watch", true) => same!(OpenAnswer, encoded, crossed),
            (b"/jig/v1/unwatch", false) => same!(WatchClose, encoded, crossed),
            (b"/jig/v1/unwatch", true) => same!(CloseAnswer, encoded, crossed),
            _ => Self::stream_record(path, response, encoded, crossed)?,
        }
        Ok(())
    }
    fn stream_record(path: &[u8], response: bool, encoded: &[u8], crossed: &[u8]) -> Result<(), Broken> {
        let path = path.split(|byte| *byte == b'?').next().expect("path prefix");
        match (path, response) {
            (b"/jig/v1/stream-gone", false) => same!(Gone, encoded, crossed),
            (b"/jig/v1/stream", true) => same!(StreamEvent, encoded, crossed),
            (path, true) if path.starts_with(b"/jig/v1/stream?version=") => same!(StreamEvent, encoded, crossed),
            (b"/jig/v1/sign-out", false) | (b"/jig/v1/stream-gone", true)
                if encoded.is_empty() && crossed.is_empty() => {}
            _ => Self::auth_record(path, response, encoded, crossed)?,
        }
        Ok(())
    }
    fn auth_record(path: &[u8], response: bool, encoded: &[u8], crossed: &[u8]) -> Result<(), Broken> {
        match (path, response) {
            (b"/jig/v1/sign-in/link", false) => same!(LinkSignIn, encoded, crossed),
            (b"/jig/v1/sign-in/link", true) => same!(SignInAnswer, encoded, crossed),
            (b"/jig/v1/sign-out", true) => same!(SignOutAnswer, encoded, crossed),
            _ => return Err(Broken::Record),
        }
        Ok(())
    }
    /// # Errors
    /// The independently observed other exchange must finish while the stream is unread.
    pub fn isolation(&mut self, unread: bool, other_answered: bool) -> Result<(), Broken> {
        if unread && !other_answered {
            return Err(Broken::Blocked);
        }
        if unread {
            self.isolated += 1;
        }
        Ok(())
    }
}
