//! Repository making facts and the client's checks (temper/docs/design/client/README.md, 4.6;
//! temper/docs/design/draft/repository-copies.md, 5.2 to 5.4).
use alloc::boxed::Box;
use skein_lib::bytes;
use temper_client_protocol_wire::{source, v1 as w};

/// One repository being made, in the forge watch's key order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Making {
    pub org: Box<[u8]>,
    pub name: Box<[u8]>,
    pub from: w::MakeFrom,
    pub into: Option<w::MakeInto>,
    pub state: w::MakingState,
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
