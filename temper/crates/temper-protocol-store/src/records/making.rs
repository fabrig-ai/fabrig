//! Making rows (temper/docs/design/protocol/store.md, 2 and 5).
use crate::v25 as w;
use skein_lib::Wall;
use temper_engine_domain_forge_vocabulary as d;

#[expect(clippy::manual_map, reason = "explicit option arms keep store translations closure-free")]
pub(super) fn encode(row: &d::RepositoryMaking) -> w::RepositoryMaking {
    let key = w::MakingKey { forge: row.key.forge, org: row.key.org.clone(), name: row.key.name.clone() };
    let from = match &row.from {
        d::MakeFrom::Empty => w::MakeFrom::Empty,
        d::MakeFrom::Copy { address } => w::MakeFrom::Copy(w::CopyFrom { address: address.clone() }),
    };
    let into = match &row.into {
        Some(into) => Some(w::MakeInto { project: into.project, home: into.home, host: into.host.clone() }),
        None => None,
    };
    let state = match &row.state {
        d::MakingState::Waiting => w::MakingState::Waiting,
        d::MakingState::Making => w::MakingState::Making,
        d::MakingState::Made { id, branch } => w::MakingState::Made(made(*id, branch)),
        d::MakingState::Adopting { id, branch } => w::MakingState::Adopting(made(*id, branch)),
        d::MakingState::Failed(failed) => w::MakingState::Failed(w::MakingFailed {
            step: encode_step(failed.step),
            reason: encode_failure(&failed.reason),
            words: failed.words.clone(),
        }),
    };
    w::RepositoryMaking { key, from, into, asked: row.asked.as_nanos(), copying: row.copying, state }
}
fn made(id: u64, branch: &[u8]) -> w::MakingMade {
    w::MakingMade { id, branch: branch.into() }
}
pub(super) fn decode(row: w::RepositoryMaking) -> d::RepositoryMaking {
    let w::RepositoryMaking { key, from, into, asked, copying, state } = row;
    let w::MakingKey { forge, org, name } = key;
    let from = match from {
        w::MakeFrom::Empty => d::MakeFrom::Empty,
        w::MakeFrom::Copy(row) => d::MakeFrom::Copy { address: row.address },
    };
    let into = match into {
        Some(row) => {
            let w::MakeInto { project, home, host } = row;
            Some(d::MakeInto { project, home, host })
        }
        None => None,
    };
    let state = match state {
        w::MakingState::Waiting => d::MakingState::Waiting,
        w::MakingState::Making => d::MakingState::Making,
        w::MakingState::Made(row) => {
            let w::MakingMade { id, branch } = row;
            d::MakingState::Made { id, branch }
        }
        w::MakingState::Adopting(row) => {
            let w::MakingMade { id, branch } = row;
            d::MakingState::Adopting { id, branch }
        }
        w::MakingState::Failed(row) => {
            let w::MakingFailed { step, reason, words } = row;
            d::MakingState::Failed(d::MakingFailed { step: decode_step(step), reason: decode_failure(reason), words })
        }
    };
    d::RepositoryMaking {
        key: d::MakingKey { forge, org, name },
        from,
        into,
        asked: Wall::from_nanos(asked),
        copying,
        state,
    }
}
pub(super) fn encode_step(value: d::MakeStep) -> w::MakeStep {
    match value {
        d::MakeStep::Admin => w::MakeStep::Admin,
        d::MakeStep::Org => w::MakeStep::Org,
        d::MakeStep::Repository => w::MakeStep::Repository,
        d::MakeStep::File => w::MakeStep::File,
        d::MakeStep::Adoption => w::MakeStep::Adoption,
    }
}
pub(super) fn decode_step(value: w::MakeStep) -> d::MakeStep {
    match value {
        w::MakeStep::Admin => d::MakeStep::Admin,
        w::MakeStep::Org => d::MakeStep::Org,
        w::MakeStep::Repository => d::MakeStep::Repository,
        w::MakeStep::File => d::MakeStep::File,
        w::MakeStep::Adoption => d::MakeStep::Adoption,
    }
}
pub(super) fn encode_refusal(value: d::MakeRefusal) -> w::MakeRefusal {
    match value {
        d::MakeRefusal::NoSuchForge => w::MakeRefusal::NoSuchForge,
        d::MakeRefusal::NotOffered => w::MakeRefusal::NotOffered,
        d::MakeRefusal::NotOwner => w::MakeRefusal::NotOwner,
        d::MakeRefusal::Invalid => w::MakeRefusal::Invalid,
        d::MakeRefusal::HomeHeld => w::MakeRefusal::HomeHeld,
        d::MakeRefusal::Full => w::MakeRefusal::Full,
        d::MakeRefusal::NotFailed => w::MakeRefusal::NotFailed,
    }
}
pub(super) fn decode_refusal(value: w::MakeRefusal) -> d::MakeRefusal {
    match value {
        w::MakeRefusal::NoSuchForge => d::MakeRefusal::NoSuchForge,
        w::MakeRefusal::NotOffered => d::MakeRefusal::NotOffered,
        w::MakeRefusal::NotOwner => d::MakeRefusal::NotOwner,
        w::MakeRefusal::Invalid => d::MakeRefusal::Invalid,
        w::MakeRefusal::HomeHeld => d::MakeRefusal::HomeHeld,
        w::MakeRefusal::Full => d::MakeRefusal::Full,
        w::MakeRefusal::NotFailed => d::MakeRefusal::NotFailed,
    }
}
fn encode_failure(value: &d::MakeFailure) -> w::MakeFailure {
    match value {
        d::MakeFailure::Refused { status } => w::MakeFailure::Refused(w::MakeStatus { status: *status }),
        d::MakeFailure::Taken => w::MakeFailure::Taken,
        d::MakeFailure::Empty => w::MakeFailure::Empty,
        d::MakeFailure::TimedOut => w::MakeFailure::TimedOut,
        d::MakeFailure::NotAllowed => w::MakeFailure::NotAllowed,
        d::MakeFailure::Other => w::MakeFailure::Other,
    }
}
fn decode_failure(value: w::MakeFailure) -> d::MakeFailure {
    match value {
        w::MakeFailure::Refused(row) => d::MakeFailure::Refused { status: row.status },
        w::MakeFailure::Taken => d::MakeFailure::Taken,
        w::MakeFailure::Empty => d::MakeFailure::Empty,
        w::MakeFailure::TimedOut => d::MakeFailure::TimedOut,
        w::MakeFailure::NotAllowed => d::MakeFailure::NotAllowed,
        w::MakeFailure::Other => d::MakeFailure::Other,
    }
}
