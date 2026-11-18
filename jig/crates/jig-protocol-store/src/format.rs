//! Versions and deployment checked at open (protocol/store/records.md, 3).
use crate::boundary::{Family, OpenRefusal};
use crate::codec::component;
use crate::upgrade::admission::{self, Levels};
use crate::{Older, Owner};

pub(crate) const NONE: Levels = Levels { skein: None, smith: None, jig: None, application: None };

/// Ahead wins over Unleveled, which wins over Behind, each upstream first (protocol/store/records.md, 3).
pub(crate) fn compare(found: Levels, wanted: Levels) -> Result<(), OpenRefusal> {
    const OWNERS: [Owner; 4] = [Owner::Skein, Owner::Smith, Owner::Jig, Owner::Application];
    for owner in OWNERS {
        if admission::level(&found, owner) > admission::level(&wanted, owner) {
            return Err(OpenRefusal::Ahead {
                owner,
                found: admission::level(&found, owner),
                wanted: admission::level(&wanted, owner),
            });
        }
    }
    for owner in OWNERS {
        if admission::level(&found, owner).is_none() && admission::level(&wanted, owner).is_some() {
            return Err(OpenRefusal::OlderBuild { why: Older::Unleveled });
        }
    }
    for owner in OWNERS {
        if admission::level(&found, owner) < admission::level(&wanted, owner) {
            return Err(OpenRefusal::OlderBuild { why: Older::Behind });
        }
    }
    Ok(())
}

#[must_use]
pub fn current(application: u32, deployment: [u8; 16]) -> component::Format {
    component::Format {
        component: 3,
        core: 6,
        tasks: 13,
        people: 17,
        notes: 1,
        accounts: 2,
        settings: 3,
        application,
        deployment,
        levels: NONE,
    }
}

pub fn check(found: &component::Format, wanted: &component::Format) -> Result<(), OpenRefusal> {
    check_versions(found, wanted)
}

pub(crate) fn check_versions(found: &component::Format, wanted: &component::Format) -> Result<(), OpenRefusal> {
    for (family, found, wanted) in [
        (Family::Component, u32::from(found.component), u32::from(wanted.component)),
        (Family::Core, u32::from(found.core), u32::from(wanted.core)),
        (Family::Tasks, u32::from(found.tasks), u32::from(wanted.tasks)),
        (Family::People, u32::from(found.people), u32::from(wanted.people)),
        (Family::Notes, u32::from(found.notes), u32::from(wanted.notes)),
        (Family::Accounts, u32::from(found.accounts), u32::from(wanted.accounts)),
        (Family::Settings, u32::from(found.settings), u32::from(wanted.settings)),
    ] {
        if found != wanted {
            return Err(OpenRefusal::Format { family, found, wanted });
        }
    }
    if found.application != wanted.application {
        return Err(OpenRefusal::Format {
            family: Family::Application,
            found: found.application,
            wanted: wanted.application,
        });
    }
    if found.deployment != wanted.deployment {
        return Err(OpenRefusal::Deployment);
    }
    Ok(())
}
