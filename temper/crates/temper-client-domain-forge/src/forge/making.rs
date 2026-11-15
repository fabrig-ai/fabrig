//! Repository making facts and the client's checks (temper/docs/design/client/pages/README.md and temper/docs/design/client/forge/records.md;
//! temper/docs/design/draft/repository-copies.md, 5.2 to 5.4).
use alloc::boxed::Box;
use skein_lib::{List, bytes};
use temper_client_protocol_wire::{source, v1 as w};

/// Why the welcome's addresses cannot be sent.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CopiesRefused {
    NoAddress,
    NotAnAddress(Box<[u8]>),
    TooMany,
}

/// Each non-empty line as the greenfield mock's `parseCopies` reads it, then `source::parse`.
/// An address in `source::address`'s form, in the order written.
#[expect(
    clippy::disallowed_methods,
    reason = "the welcome trims person-entered lines as Unicode before checking source addresses"
)]
pub fn parse_copies(text: &[u8]) -> Result<List<Box<[u8]>>, CopiesRefused> {
    let Ok(text) = core::str::from_utf8(text) else {
        return Err(CopiesRefused::NotAnAddress(bytes::copy_of(text)));
    };
    let mut addresses = List::with_capacity(temper_client_protocol_wire::FIRST_COPIES);
    for raw in text.split('\n') {
        let raw = raw.trim();
        let line = match raw.strip_prefix("https://") {
            Some(line) => line,
            None => raw.strip_prefix("http://").unwrap_or(raw),
        };
        let line = line.strip_suffix(".git").unwrap_or(line).trim_end_matches('/');
        if line.is_empty() {
            continue;
        }
        let line = line.as_bytes();
        let address = match bytes::count(line, b"/", 3) {
            1 => bytes::concat(&[b"github.com/", line]),
            2 => bytes::copy_of(line),
            _ => return Err(CopiesRefused::NotAnAddress(bytes::copy_of(raw.as_bytes()))),
        };
        let Ok(source) = source::parse(&address) else {
            return Err(CopiesRefused::NotAnAddress(bytes::copy_of(raw.as_bytes())));
        };
        let address = source::address(&source);
        if address.len() > source::ADDRESS_BYTES {
            return Err(CopiesRefused::NotAnAddress(bytes::copy_of(raw.as_bytes())));
        }
        if addresses.push(address).is_err() {
            return Err(CopiesRefused::TooMany);
        }
    }
    if addresses.is_empty() {
        return Err(CopiesRefused::NoAddress);
    }
    Ok(addresses)
}

/// Each address keeps its own repair (temper/docs/design/client/forge/family.md, 4).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Line {
    To { address: Box<[u8]>, org: Box<[u8]>, name: Box<[u8]> },
    Bad { raw: Box<[u8]> },
    NotAHost { raw: Box<[u8]>, line: Box<[u8]> },
    Taken { raw: Box<[u8]>, org: Box<[u8]>, name: Box<[u8]> },
}

/// Parse independently so a bad address cannot hide the other lines.
#[must_use]
#[expect(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::type_complexity,
    reason = "the contract takes a source and target-name predicate and trims person-entered lines as parse_copies does"
)]
pub fn lines(text: &[u8], has: &dyn Fn(&[u8], &[u8], &[u8]) -> bool) -> List<Line> {
    let mut result = List::with_capacity(u32::try_from(text.len()).expect("bounded field length"));
    let Ok(text) = core::str::from_utf8(text) else {
        result.push(Line::Bad { raw: bytes::copy_of(text) }).expect("nonempty invalid UTF-8");
        return result;
    };
    for raw in text.split('\n') {
        let raw = raw.trim();
        if raw.is_empty() {
            continue;
        }
        result.push(line(raw, has)).expect("no more lines than bytes");
    }
    result
}
#[expect(
    clippy::disallowed_types,
    clippy::type_complexity,
    reason = "the sheet contract supplies its source and target-name predicate"
)]
fn line(raw: &str, has: &dyn Fn(&[u8], &[u8], &[u8]) -> bool) -> Line {
    let text = raw.strip_prefix("https://").unwrap_or(raw);
    let text = text.strip_prefix("http://").unwrap_or(text).trim_end_matches('/');
    let text = text.strip_suffix(".git").unwrap_or(text);
    let mut parts = text.split('/');
    let first = parts.next().unwrap_or("");
    let second = parts.next().unwrap_or("");
    let third = parts.next();
    let (host, org, name) = match third {
        Some(name) => (first, second, name),
        None => ("github.com", first, second),
    };
    let raw = raw.as_bytes();
    if org.is_empty() || name.is_empty() || parts.next().is_some() {
        return Line::Bad { raw: bytes::copy_of(raw) };
    }
    if !host.contains('.') {
        return Line::NotAHost {
            raw: bytes::copy_of(raw),
            line: bytes::concat(&[b"github.com/", org.as_bytes(), b"/", name.as_bytes()]),
        };
    }
    let address = bytes::concat(&[host.as_bytes(), b"/", org.as_bytes(), b"/", name.as_bytes()]);
    let Ok(source) = source::parse(&address) else {
        return Line::Bad { raw: bytes::copy_of(raw) };
    };
    let address = source::address(&source);
    if address.len() > source::ADDRESS_BYTES {
        return Line::Bad { raw: bytes::copy_of(raw) };
    }
    let (org, name) = copy_target(&source);
    if has(&address, &org, &name) {
        Line::Taken { raw: bytes::copy_of(raw), org, name }
    } else {
        Line::To { address, org, name }
    }
}

/// The copies of the real forge numbered lowest whose intent names them (client/pages/README.md).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Copies {
    pub forge: u16,
    pub addresses: Box<[Box<[u8]>]>,
}
#[must_use]
pub fn copies(rows: &super::forges::State, playground: Option<u16>) -> Option<Copies> {
    use super::forges::{Mode, StartsWith, State};
    let State::Rows(rows) = rows else { return None };
    let mut found: Option<Copies> = None;
    for row in rows {
        if row.mode == Mode::Fake || Some(row.number) == playground {
            continue;
        }
        let addresses = match &row.starts_with {
            StartsWith::Copies { addresses } => addresses,
            StartsWith::Repository | StartsWith::Nothing => continue,
        };
        let lower = match &found {
            Some(found) => row.number < found.forge,
            None => true,
        };
        if lower {
            found = Some(Copies { forge: row.number, addresses: addresses.clone() });
        }
    }
    found
}
/// A copy's org and name, the source's (repository-copies.md, 5.3).
#[must_use]
#[expect(clippy::type_complexity, reason = "the shared copy target contract returns the source org and name")]
pub fn target(address: &[u8]) -> Option<(Box<[u8]>, Box<[u8]>)> {
    Some(copy_target(&source::parse(address).ok()?))
}

/// One repository being made, in the forge watch's key order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Making {
    pub org: Box<[u8]>,
    pub name: Box<[u8]>,
    pub from: w::MakeFrom,
    pub into: Option<w::MakeInto>,
    pub state: w::MakingState,
}

/// A Ready copy keeps both its source address and its source's destination name.
#[must_use]
pub fn is_target(making: &Making, address: &[u8]) -> bool {
    let Some((org, name)) = target(address) else { return false };
    if making.org != org || making.name != name {
        return false;
    }
    match &making.from {
        w::MakeFrom::Copy(copy) => copy.address() == address,
        w::MakeFrom::Empty => false,
    }
}
/// A copy already asked or held anywhere on this forge (client/pages/README.md, Home).
#[must_use]
pub fn asked(row: &super::forges::Row, address: &[u8]) -> bool {
    for making in &row.makings {
        if is_target(making, address) {
            return true;
        }
    }
    let Some((org, name)) = target(address) else { return false };
    for repository in &row.repositories {
        if repository.owner == org && repository.name == name {
            return true;
        }
    }
    false
}

/// The part of a making request the client cannot accept.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refused {
    Org,
    Name,
    Source(source::SourceRefusal),
    TooLong,
}

/// Check the destination and public source; no source makes an empty repository.
pub fn ask(
    forge: u16,
    org: &[u8],
    name: &[u8],
    from: Option<&[u8]>,
    into: Option<w::MakeInto>,
) -> Result<w::MakeRepository, Refused> {
    if !source::org(org) {
        return Err(Refused::Org);
    }
    if !source::repository(name) {
        return Err(Refused::Name);
    }
    let from = match from {
        Some(address) => {
            let source = match source::parse(address) {
                Ok(source) => source,
                Err(why) => return Err(Refused::Source(why)),
            };
            let copy = w::MakeCopy::new(&w::CEILINGS, w::MakeCopyParts { address: source::address(&source) });
            match copy {
                Ok(copy) => w::MakeFrom::Copy(copy),
                Err(_) => return Err(Refused::TooLong),
            }
        }
        None => w::MakeFrom::Empty,
    };
    match w::MakeRepository::new(
        &w::CEILINGS,
        w::MakeRepositoryParts { forge, org: bytes::copy_of(org), name: bytes::copy_of(name), from, into },
    ) {
        Ok(request) => Ok(request),
        Err(_) => Err(Refused::TooLong),
    }
}

/// Default a copy's destination to its source; `ask` checks the owner's org grammar.
#[must_use]
pub fn copy_target(source: &source::Source<'_>) -> (Box<[u8]>, Box<[u8]>) {
    (bytes::copy_of(source.owner), bytes::copy_of(source.name))
}

/// Describe the kept state without offering a percentage for a forge migration.
#[must_use]
pub fn phrase(row: &Making) -> Box<[u8]> {
    let org = super::words::lossy(&row.org);
    let name = super::words::lossy(&row.name);
    match &row.state {
        w::MakingState::Failed(failed) => bytes::concat(&[
            &org,
            b"/",
            &name,
            b" failed at ",
            step(failed.step()),
            b": ",
            &super::words::lossy(failed.words()),
        ]),
        w::MakingState::Waiting | w::MakingState::Making | w::MakingState::Made(_) | w::MakingState::Adopting(_) => {
            match &row.from {
                w::MakeFrom::Empty => bytes::concat(&[b"Making ", &org, b"/", &name]),
                w::MakeFrom::Copy(copy) => {
                    let host = match source::parse(copy.address()) {
                        Ok(source) => source.host,
                        Err(_) => b"?",
                    };
                    bytes::concat(&[b"Copying ", &org, b"/", &name, b" from ", host])
                }
            }
        }
    }
}

const fn step(step: &w::MakeStep) -> &'static [u8] {
    match step {
        w::MakeStep::Admin => b"admin",
        w::MakeStep::Org => b"org",
        w::MakeStep::Repository => b"repository",
        w::MakeStep::File => b"file",
        w::MakeStep::Adoption => b"adoption",
    }
}

#[cfg(test)]
#[path = "../tests/making.rs"]
mod tests;
