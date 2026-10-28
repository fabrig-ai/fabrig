//! GitHub additions to the forge's JSON lines (domain/testing.md, 4.2).
use crate::{Refusal, read};
use jig_protocol_config::Path;
use skein_fake_forge_github::{GitCall, Minted};
use skein_json::{
    Document,
    writer::{Encoder, Limits},
};
use skein_lib::{List, Overflow, Queue};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Observation {
    GitCall(GitCall),
    GitCallsFull { capacity: u32 },
    InstallationCreated { installation: u64 },
    InstallationDeleted { installation: u64 },
    InstallationsFull { capacity: u32 },
}
/// A length cursor over the layer's ring. Full is terminal, before evictions
/// could silently replace a record; no further observations may follow it.
#[derive(Debug)]
pub struct GitCalls {
    written: u32,
    full: bool,
}
impl Default for GitCalls {
    fn default() -> Self {
        Self::new()
    }
}
impl GitCalls {
    #[must_use]
    pub const fn new() -> Self {
        Self { written: 0, full: false }
    }
    /// Appends only new calls and returns true on terminal saturation.
    pub fn drain(&mut self, calls: &Queue<GitCall>, out: &mut List<u8>) -> Result<bool, Overflow> {
        if self.full {
            return Ok(true);
        }
        for call in calls.iter().skip(usize::try_from(self.written).expect("ring cursor")) {
            encode(Observation::GitCall(*call), out)?;
            self.written = self.written.checked_add(1).expect("bounded ring");
        }
        if calls.room() == 0 {
            encode(Observation::GitCallsFull { capacity: calls.capacity() }, out)?;
            self.full = true;
        }
        Ok(self.full)
    }
}
/// A bounded snapshot of the layer, compared after each process iteration.
/// The layer consumes its own change events for deliveries, so this observer
/// never takes those events away from the peer. Ids are never reused.
#[derive(Debug)]
pub struct Installations {
    previous: List<u64>,
    written: u32,
    capacity: u32,
    full: bool,
}
impl Installations {
    #[must_use]
    pub fn new(entries: u32, changes: u32) -> Self {
        Self { previous: List::with_capacity(entries), written: 0, capacity: changes, full: false }
    }
    /// A full stream is terminal, including when a snapshot exceeds its bound.
    pub fn drain(
        &mut self,
        installations: &skein_fake_forge_github::installations::Installations,
        out: &mut List<u8>,
    ) -> Result<bool, Overflow> {
        if self.full {
            return Ok(true);
        }
        for row in installations.iter() {
            if !self.previous.as_slice().contains(&row.id) {
                if self.written == self.capacity {
                    return self.end(out);
                }
                encode(Observation::InstallationCreated { installation: row.id }, out)?;
                self.written = self.written.checked_add(1).expect("bounded changes");
            }
        }
        for id in &self.previous {
            if installations.get(*id).is_none() {
                if self.written == self.capacity {
                    return self.end(out);
                }
                encode(Observation::InstallationDeleted { installation: *id }, out)?;
                self.written = self.written.checked_add(1).expect("bounded changes");
            }
        }
        self.previous.clear();
        for row in installations.iter() {
            if self.previous.push(row.id).is_err() {
                return self.end(out);
            }
        }
        if self.written == self.capacity {
            return self.end(out);
        }
        Ok(false)
    }
    fn end(&mut self, out: &mut List<u8>) -> Result<bool, Overflow> {
        encode(Observation::InstallationsFull { capacity: self.capacity }, out)?;
        self.full = true;
        Ok(true)
    }
}
fn encode(observation: Observation, out: &mut List<u8>) -> Result<(), Overflow> {
    let limits = Limits { depth: 1, length: 256 };
    let mut measure = Encoder::measure(&limits);
    render(observation, &mut measure);
    let len = measure.measured().or(Err(Overflow))?;
    if out.room() < len.checked_add(1).ok_or(Overflow)? {
        return Err(Overflow);
    }
    let mut json = Encoder::write(len, &limits);
    render(observation, &mut json);
    for byte in json.finish() {
        out.push(byte).expect("measured line");
    }
    out.push(b'\n').expect("measured newline");
    Ok(())
}
fn render(observation: Observation, json: &mut Encoder) {
    json.object_start();
    json.key(b"observation");
    match observation {
        Observation::GitCall(call) => {
            json.string(b"git_call");
            json.key(b"installation");
            match call.minted {
                Some(minted) => json.unsigned(minted.installation),
                None => json.null(),
            }
            json.key(b"mint");
            match call.minted {
                Some(minted) => json.unsigned(minted.mint),
                None => json.null(),
            }
            json.key(b"writes");
            json.boolean(call.writes);
            json.key(b"status");
            json.unsigned(u64::from(call.status));
        }
        Observation::InstallationCreated { installation } => {
            json.string(b"installation_created");
            json.key(b"installation");
            json.unsigned(installation);
        }
        Observation::InstallationDeleted { installation } => {
            json.string(b"installation_deleted");
            json.key(b"installation");
            json.unsigned(installation);
        }
        Observation::InstallationsFull { capacity } => {
            json.string(b"installations_full");
            json.key(b"capacity");
            json.unsigned(u64::from(capacity));
        }
        Observation::GitCallsFull { capacity } => {
            json.string(b"git_calls_full");
            json.key(b"capacity");
            json.unsigned(u64::from(capacity));
        }
    }
    json.object_end();
}
/// Recognizes and validates only GitHub's additions; None lets skein's
/// existing observation decoder read a domain observation or checkpoint.
pub fn decode(bytes: &[u8]) -> Result<Option<Observation>, Refusal> {
    let d = read::document(bytes)?;
    let p = Path::root();
    let mut o = read::object(&d, 0, &p)?;
    let Some(at) = o.member(&d, b"observation") else { return Ok(None) };
    let name = read::text(&d, at, 64, &p.member(b"observation"))?;
    let observation = match name.as_ref() {
        b"git_call" => {
            let installation =
                optional(&d, read::required(&mut o, &d, b"installation", &p)?, &p.member(b"installation"))?;
            let mint = optional(&d, read::required(&mut o, &d, b"mint", &p)?, &p.member(b"mint"))?;
            let minted = match (installation, mint) {
                (Some(installation), Some(mint)) => Some(Minted { installation, mint }),
                (None, None) => None,
                (Some(_), None) | (None, Some(_)) => {
                    return Err(read::why(&p, b"installation and mint must both be present or null"));
                }
            };
            let at = read::required(&mut o, &d, b"writes", &p)?;
            let writes = match d.token(at) {
                Some(token) if token.kind == skein_json::document::Kind::True => true,
                Some(token) if token.kind == skein_json::document::Kind::False => false,
                Some(_) | None => return Err(read::why(&p.member(b"writes"), b"expected boolean")),
            };
            let at = read::required(&mut o, &d, b"status", &p)?;
            let status = read::integer(&d, at, u64::from(u16::MAX), &p.member(b"status"))?;
            Observation::GitCall(GitCall { minted, writes, status: u16::try_from(status).expect("bounded status") })
        }
        b"installation_created" | b"installation_deleted" => {
            let at = read::required(&mut o, &d, b"installation", &p)?;
            let installation = read::integer(&d, at, u64::MAX, &p.member(b"installation"))?;
            if name.as_ref() == b"installation_created" {
                Observation::InstallationCreated { installation }
            } else {
                Observation::InstallationDeleted { installation }
            }
        }
        b"installations_full" => {
            let at = read::required(&mut o, &d, b"capacity", &p)?;
            let capacity = read::integer(&d, at, u64::from(u32::MAX), &p.member(b"capacity"))?;
            Observation::InstallationsFull { capacity: u32::try_from(capacity).expect("bounded capacity") }
        }
        b"git_calls_full" => {
            let at = read::required(&mut o, &d, b"capacity", &p)?;
            let capacity = read::integer(&d, at, u64::from(u32::MAX), &p.member(b"capacity"))?;
            Observation::GitCallsFull { capacity: u32::try_from(capacity).expect("bounded capacity") }
        }
        _ => return Ok(None),
    };
    read::finish(o, &d, &p)?;
    Ok(Some(observation))
}
fn optional(d: &Document, at: u32, p: &Path) -> Result<Option<u64>, Refusal> {
    if read::is_null(d, at) { Ok(None) } else { Ok(Some(read::integer(d, at, u64::MAX, p)?)) }
}
