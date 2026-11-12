//! Versions and deployment checked at open (protocol/store/records.md, 3).
use crate::boundary::{Family, OpenRefusal};
use crate::codec::component;
use crate::patching::plan::Levels;
use crate::{Older, Owner};

pub(crate) const NONE: Levels = Levels { skein: None, smith: None, jig: None, application: None };

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Compared {
    Same,
    Behind,
}
/// Recognize the fixed baseline before comparing levels (protocol/store/records.md, 3).
pub(crate) fn found_levels(found: &component::Format, layouts: Option<&component::Layouts>, binary: Levels) -> Levels {
    let baseline = &crate::baseline::BASELINE;
    let wanted = component::Format { application: found.application, deployment: found.deployment, ..baseline.format };
    if found.levels != NONE
        || check_versions(found, &wanted, Compared::Same).is_err()
        || layouts != Some(&baseline.layouts)
    {
        return found.levels;
    }
    let mut levels = NONE;
    for owner in [Owner::Skein, Owner::Smith, Owner::Jig, Owner::Application] {
        if binary.of(owner).is_some() {
            levels.set(owner, Some(0));
        }
    }
    levels
}

/// Ahead wins over Unleveled, which wins over Behind, each upstream first (5.3).
pub(crate) fn compare(found: Levels, wanted: Levels) -> Result<Compared, OpenRefusal> {
    const OWNERS: [Owner; 4] = [Owner::Skein, Owner::Smith, Owner::Jig, Owner::Application];
    for owner in OWNERS {
        if found.of(owner) > wanted.of(owner) {
            return Err(OpenRefusal::Ahead { owner, found: found.of(owner), wanted: wanted.of(owner) });
        }
    }
    for owner in OWNERS {
        if found.of(owner).is_none() && wanted.of(owner).is_some() {
            return Err(OpenRefusal::OlderBuild { why: Older::Unleveled });
        }
    }
    for owner in OWNERS {
        if found.of(owner) < wanted.of(owner) {
            return Ok(Compared::Behind);
        }
    }
    Ok(Compared::Same)
}

#[must_use]
pub fn current(application: u32, deployment: [u8; 16]) -> component::Format {
    component::Format {
        component: 3,
        core: 6,
        tasks: 10,
        people: 14,
        notes: 1,
        accounts: 2,
        settings: 2,
        application,
        deployment,
        levels: NONE,
    }
}

pub fn check(found: &component::Format, wanted: &component::Format) -> Result<(), OpenRefusal> {
    check_versions(found, wanted, Compared::Same)
}

pub(crate) fn check_versions(
    found: &component::Format,
    wanted: &component::Format,
    compared: Compared,
) -> Result<(), OpenRefusal> {
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
    // Jig versions are checked against the store's level before a patch;
    // the application's patches rewrite its version in the sealing commit.
    if compared == Compared::Same && found.application != wanted.application {
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

/// Jig's versions at the store's level, from the same blocks the pending machines planned.
pub(crate) fn at_level(
    application: u32,
    deployment: [u8; 16],
    machines: &crate::patching::machines::Machines,
) -> component::Format {
    // Sets that do not name one of jig's families keep this build's version for it.
    let mut wanted = current(application, deployment);
    for (name, version) in [
        ("jig-store-component", &mut wanted.component),
        ("jig-store-core", &mut wanted.core),
        ("jig-store-tasks", &mut wanted.tasks),
        ("jig-store-people", &mut wanted.people),
        ("jig-store-notes", &mut wanted.notes),
        ("jig-store-accounts", &mut wanted.accounts),
        ("jig-store-settings", &mut wanted.settings),
    ] {
        if let Some(family) = machines.named(name.as_bytes()) {
            let machine = machines.get(family);
            if machine.owner == Owner::Jig {
                *version = machine.reached;
            }
        }
    }
    wanted
}

/// This build's layouts (protocol/store/records.md, 3): computed from the schemas built in,
/// committed beside the generated codecs, and checked against them by the drift test.
#[must_use]
pub fn layouts() -> component::Layouts {
    crate::layouts::LAYOUTS
}

/// Compare in Family's order, before any family's header is decoded.
pub fn check_layouts(found: &component::Layouts, wanted: &component::Layouts) -> Result<(), OpenRefusal> {
    check_unleveled_layouts(found, wanted, NONE, NONE)
}

pub(crate) fn needs_layouts(found: Levels, wanted: Levels) -> bool {
    (found.jig.is_none() && wanted.jig.is_none())
        || (found.skein.is_none() && wanted.skein.is_none())
        || (found.smith.is_none() && wanted.smith.is_none())
}

pub(crate) fn check_unleveled_layouts(
    found: &component::Layouts,
    wanted: &component::Layouts,
    found_levels: Levels,
    wanted_levels: Levels,
) -> Result<(), OpenRefusal> {
    for (family, found, wanted) in [
        (Family::Settings, found.settings, wanted.settings),
        (Family::Component, found.component, wanted.component),
        (Family::Core, found.core, wanted.core),
        (Family::Tasks, found.tasks, wanted.tasks),
        (Family::People, found.people, wanted.people),
        (Family::Notes, found.notes, wanted.notes),
        (Family::Accounts, found.accounts, wanted.accounts),
        (Family::Secrets, found.secrets, wanted.secrets),
        (Family::SkeinOauth, found.skein_oauth, wanted.skein_oauth),
        (Family::SmithTranscript, found.smith_transcript, wanted.smith_transcript),
    ] {
        let owner = match family {
            Family::SkeinOauth => Owner::Skein,
            Family::SmithTranscript => Owner::Smith,
            Family::Settings
            | Family::Component
            | Family::Core
            | Family::Tasks
            | Family::People
            | Family::Notes
            | Family::Accounts
            | Family::Secrets
            | Family::Application => Owner::Jig,
        };
        if found_levels.of(owner).is_none() && wanted_levels.of(owner).is_none() && found != wanted {
            return Err(OpenRefusal::OlderBuild { why: Older::Layout(family) });
        }
    }
    Ok(())
}
