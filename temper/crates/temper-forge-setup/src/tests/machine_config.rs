//! Combined member fixtures for setup form tests, not deployment configuration.
pub(crate) use crate::machine::*;
use crate::problem::Problems;
use jig_protocol_config::{self as config, Path, read};
use skein_forge_setup::limits::Limits;
use skein_json::Document;
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Machine {
    pub owned: Option<OwnedMachine>,
    pub fake: Option<FakeMachine>,
}

/// Reads `application.machine`, recording paths and resolving data and cache paths as setup does.
pub fn read(
    d: &Document,
    at: u32,
    _limits: &Limits,
    directories: &config::Directories,
    problems: &mut Problems,
) -> Option<Machine> {
    let mut ps = config::Problems::empty();
    let result = read_config(d, at, directories, &mut ps);
    problems.append(ps);
    result
}
/// The application retains each typed problem's full member path.
pub fn read_config(
    d: &Document,
    at: u32,
    directories: &config::Directories,
    ps: &mut config::Problems,
) -> Option<Machine> {
    let p = Path::root().member(b"application").member(b"machine");
    let mut result = machine(d, at, &p, ps);
    if ps.is_empty()
        && let Some(machine) = &mut result
    {
        if let Some(owned) = &mut machine.owned {
            resolve_path(
                directories,
                config::Directory::Cache,
                &mut owned.forgejo,
                p.member(b"owned").member(b"forgejo"),
                ps,
            );
            if let Some(runner) = &mut owned.runner {
                resolve_path(
                    directories,
                    config::Directory::Cache,
                    &mut runner.binary,
                    p.member(b"owned").member(b"runner").member(b"binary"),
                    ps,
                );
            }
        }
        if let Some(fake) = &mut machine.fake {
            resolve_path(directories, config::Directory::Data, &mut fake.git, p.member(b"fake").member(b"git"), ps);
        }
    }
    if ps.is_empty() { result } else { None }
}
fn machine(d: &Document, at: u32, p: &Path, ps: &mut config::Problems) -> Option<Machine> {
    let mut o = match read::Object::new(d, at) {
        Ok(o) => o,
        Err(what) => {
            ps.push(p.clone(), what);
            return None;
        }
    };
    let owned_at = o.member(d, b"owned");
    let fake_at = o.member(d, b"fake");
    o.finish(d, p, ps);
    let owned = match owned_at {
        Some(at) => Some(owned(d, at, &p.member(b"owned"), ps)?),
        None => None,
    };
    let fake = match fake_at {
        Some(at) => Some(fake(d, at, &p.member(b"fake"), ps)?),
        None => None,
    };
    Some(Machine { owned, fake })
}

pub fn entry(
    machine: &Machine,
    forge: u16,
    kind: BoxKind,
    directories: &config::Directories,
) -> Option<crate::setup::Entry> {
    let file = match kind {
        BoxKind::Owned => crate::setup_file::SetupFile::Owned(machine.owned.clone()?),
        BoxKind::Fake => crate::setup_file::SetupFile::Fake(machine.fake.clone()?),
    };
    file.entry(forge, directories)
}
