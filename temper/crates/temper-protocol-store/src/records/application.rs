//! Retained outcomes, field for field with the forge's application family.
use super::{FITS, names};
use crate::{Bounds, Undecodable, v25 as w};
use skein_lib::{Reader, Wall, Writer};
use temper_engine_domain_forge_vocabulary as d;
mod refusal;

pub(super) fn encode(token: u64, until: Wall, outcome: &d::ApplicationOutcome) -> w::Application {
    let outcome = match outcome {
        d::ApplicationOutcome::Opened { task } => w::ApplicationOutcome::Opened(w::Opened { task: *task }),
        d::ApplicationOutcome::OpenRefused(reason) => {
            w::ApplicationOutcome::OpenRefused(w::OpenRefused { reason: encode_open_refusal(reason) })
        }
        d::ApplicationOutcome::Follow(refusal) => {
            w::ApplicationOutcome::Follow(w::FollowOutcome { refusal: refusal.clone() })
        }
        d::ApplicationOutcome::MakeKept => w::ApplicationOutcome::MakeKept,
        d::ApplicationOutcome::MakeRefused(why) => {
            w::ApplicationOutcome::MakeRefused(w::MakeRefused { reason: super::making::encode_refusal(*why) })
        }
        d::ApplicationOutcome::Adopted(v) => w::ApplicationOutcome::Adopted(w::ApplicationAdopted {
            repository: names::encode_repository(v.repository),
            owner: v.owner.clone(),
            name: v.name.clone(),
            landing: v.landing.clone(),
            role: names::encode_role(v.role),
            home: v.home,
            kinds: names::encode_kinds(&v.kinds),
            protection: names::encode_protection(&v.protection),
        }),
        d::ApplicationOutcome::CoreRefused(v) => {
            let record =
                refusal::encode_refusal(v, &jig_protocol_store::people_v17::CEILINGS).expect("refusal facts fit");
            let mut writer = Writer::new(usize::try_from(record.measure()).expect("small refusal"));
            record.encode(&mut writer).expect("measured room");
            w::ApplicationOutcome::CoreRefused(w::CoreRefused { refusal: writer.finish() })
        }
        d::ApplicationOutcome::ForgeRefused(v) => {
            w::ApplicationOutcome::ForgeRefused(w::ForgeRefused { reason: encode_reason(*v) })
        }
    };
    w::Application { token, until: until.as_nanos(), outcome }
}
pub(super) fn decode(value: w::Application, bounds: &Bounds) -> Result<d::Stored, Undecodable> {
    let w::Application { token, until, outcome } = value;
    let outcome = match outcome {
        w::ApplicationOutcome::Opened(value) => d::ApplicationOutcome::Opened { task: value.task },
        w::ApplicationOutcome::OpenRefused(value) => {
            d::ApplicationOutcome::OpenRefused(decode_open_refusal(value.reason))
        }
        w::ApplicationOutcome::Follow(value) => d::ApplicationOutcome::Follow(value.refusal),
        w::ApplicationOutcome::MakeKept => d::ApplicationOutcome::MakeKept,
        w::ApplicationOutcome::MakeRefused(why) => {
            d::ApplicationOutcome::MakeRefused(super::making::decode_refusal(why.reason))
        }
        w::ApplicationOutcome::Adopted(v) => {
            let w::ApplicationAdopted { repository, owner, name, role, home, kinds, protection, landing } = v;
            d::ApplicationOutcome::Adopted(d::AdoptedRepository {
                repository: names::decode_repository(repository),
                owner,
                name,
                landing,
                role: names::decode_role(role),
                home,
                kinds: names::decode_kinds(kinds),
                protection: names::decode_protection(protection, bounds)?,
            })
        }
        w::ApplicationOutcome::CoreRefused(v) => {
            let mut reader = Reader::new(&v.refusal);
            let record =
                jig_protocol_store::people_v17::Refusal::decode(&jig_protocol_store::people_v17::CEILINGS, &mut reader)
                    .or(Err(Undecodable::Malformed))?;
            d::ApplicationOutcome::CoreRefused(refusal::decode_refusal(record))
        }
        w::ApplicationOutcome::ForgeRefused(v) => d::ApplicationOutcome::ForgeRefused(decode_reason(v.reason)),
    };
    Ok(d::Stored::Application { token, until: Wall::from_nanos(until), outcome })
}
fn encode_open_refusal(value: &d::OpenRefusal) -> w::OpenRefusal {
    match value {
        d::OpenRefusal::NoRepository => w::OpenRefusal::NoRepository,
        d::OpenRefusal::NotARepository => w::OpenRefusal::NotARepository,
        d::OpenRefusal::NamePast { bound } => w::OpenRefusal::NamePast(w::NamePast { bound: *bound }),
        d::OpenRefusal::UnderPrefix { owner, name, prefix } => w::OpenRefusal::UnderPrefix(w::UnderPrefix {
            owner: owner.clone(),
            name: name.clone(),
            prefix: prefix.clone(),
        }),
        d::OpenRefusal::Several(names) => {
            let mut repositories = skein_lib::List::with_capacity(w::CEILINGS.several_repositories_repositories);
            for named in names {
                repositories
                    .push(w::OpenRepository { owner: named.owner.clone(), name: named.name.clone() })
                    .expect(FITS);
            }
            w::OpenRefusal::Several(w::SeveralRepositories { repositories })
        }
    }
}
fn decode_open_refusal(value: w::OpenRefusal) -> d::OpenRefusal {
    match value {
        w::OpenRefusal::NoRepository => d::OpenRefusal::NoRepository,
        w::OpenRefusal::NotARepository => d::OpenRefusal::NotARepository,
        w::OpenRefusal::NamePast(value) => d::OpenRefusal::NamePast { bound: value.bound },
        w::OpenRefusal::UnderPrefix(value) => {
            d::OpenRefusal::UnderPrefix { owner: value.owner, name: value.name, prefix: value.prefix }
        }
        w::OpenRefusal::Several(value) => {
            let mut repositories = skein_lib::List::with_capacity(w::CEILINGS.several_repositories_repositories);
            for named in value.repositories.into_boxed() {
                repositories.push(d::DraftRepository { owner: named.owner, name: named.name }).expect(FITS);
            }
            d::OpenRefusal::Several(repositories.into_boxed())
        }
    }
}
fn encode_reason(v: d::AdoptionRefusal) -> w::ForgeReason {
    match v {
        d::AdoptionRefusal::NotInstalled => w::ForgeReason::NotInstalled,
        d::AdoptionRefusal::NoSuchForge => w::ForgeReason::NoSuchForge,
        d::AdoptionRefusal::NotFound => w::ForgeReason::NotFound,
        d::AdoptionRefusal::NoAccess => w::ForgeReason::NoAccess,
        d::AdoptionRefusal::Refused => w::ForgeReason::Refused,
        d::AdoptionRefusal::InFlight => w::ForgeReason::InFlight,
        d::AdoptionRefusal::TooLarge => w::ForgeReason::TooLarge,
        d::AdoptionRefusal::Unreachable => w::ForgeReason::Unreachable,
        d::AdoptionRefusal::InvalidAnswer => w::ForgeReason::InvalidAnswer,
        d::AdoptionRefusal::AnotherProject => w::ForgeReason::AnotherProject,
        d::AdoptionRefusal::HomeHeld => w::ForgeReason::HomeHeld,
        d::AdoptionRefusal::PrefixOccupied => w::ForgeReason::PrefixOccupied,
    }
}
fn decode_reason(v: w::ForgeReason) -> d::AdoptionRefusal {
    match v {
        w::ForgeReason::NotInstalled => d::AdoptionRefusal::NotInstalled,
        w::ForgeReason::NoSuchForge => d::AdoptionRefusal::NoSuchForge,
        w::ForgeReason::NotFound => d::AdoptionRefusal::NotFound,
        w::ForgeReason::NoAccess => d::AdoptionRefusal::NoAccess,
        w::ForgeReason::Refused => d::AdoptionRefusal::Refused,
        w::ForgeReason::InFlight => d::AdoptionRefusal::InFlight,
        w::ForgeReason::TooLarge => d::AdoptionRefusal::TooLarge,
        w::ForgeReason::Unreachable => d::AdoptionRefusal::Unreachable,
        w::ForgeReason::InvalidAnswer => d::AdoptionRefusal::InvalidAnswer,
        w::ForgeReason::AnotherProject => d::AdoptionRefusal::AnotherProject,
        w::ForgeReason::HomeHeld => d::AdoptionRefusal::HomeHeld,
        w::ForgeReason::PrefixOccupied => d::AdoptionRefusal::PrefixOccupied,
    }
}
