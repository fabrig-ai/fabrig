//! One forge's group file (temper/docs/plan/playground-reset-r4/notes/contracts.md, 3).
use crate::{
    check::joined,
    problem::{Problems, What},
    setup::{Entry, Mode, read_entry},
};
use alloc::boxed::Box;
use jig_protocol_config::{self as config, Path, read as value};
use skein_forge_setup::limits::Limits;
use skein_lib::Decimal;

/// One forge's group file, `<data>/box/groups/N` (contracts, 3).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct GroupFile {
    pub entry: Entry,
    pub playground: bool,
}
/// The file's bound, matching the box's default `group_file_bytes`.
pub const BYTES: u32 = 65_536;
/// `box/groups/N`, beneath the data directory.
#[must_use]
pub fn path(forge: u16) -> Box<[u8]> {
    joined(&[b"box/groups/", Decimal::of(u64::from(forge)).as_bytes()])
}
/// Reads forge `forge`'s file whole: `{"setup": ENTRY, "playground": BOOL}`.
pub fn read(
    bytes: &[u8],
    forge: u16,
    limits: &Limits,
    directories: &config::Directories,
) -> Result<GroupFile, Problems> {
    let mut problems = Problems::empty();
    let d = match config::document(bytes, &config::DocumentLimits { bytes: BYTES, tokens: BYTES, depth: 32 }) {
        Ok(document) => document,
        Err(problem) => {
            let mut ps = config::Problems::empty();
            ps.push(problem.path, problem.what);
            problems.append(ps);
            return Err(problems);
        }
    };
    let root = Path::root();
    let mut object = match value::Object::new(&d, 0) {
        Ok(object) => object,
        Err(what) => {
            let mut ps = config::Problems::empty();
            ps.push(root, what);
            problems.append(ps);
            return Err(problems);
        }
    };
    let mut ps = config::Problems::empty();
    let setup = object.required(&d, b"setup", &root, &mut ps);
    let playground = object.required(&d, b"playground", &root, &mut ps);
    object.finish(&d, &root, &mut ps);
    let mut paths = config::Problems::empty();
    let entry = match setup {
        Some(at) => read_entry(&d, at, limits, directories, &root.member(b"setup"), &mut ps, &mut paths),
        None => None,
    };
    let playground = match playground {
        Some(at) => match value::boolean(&d, at) {
            Ok(playground) => Some(playground),
            Err(what) => {
                ps.push(root.member(b"playground"), what);
                None
            }
        },
        None => None,
    };
    problems.append(ps);
    problems.append(paths);
    if let Some(entry) = &entry {
        if entry.forge != forge {
            problems.push(root.member(b"setup").member(b"forge"), What::Reason(b"must match the group number"));
        }
        let (owned, fake) = match &entry.mode {
            Mode::Owned(_) => (true, false),
            Mode::Fake(_) => (false, true),
            Mode::Managed { .. } => (false, false),
        };
        if owned && !entry.asked {
            problems.push(root.member(b"setup").member(b"start"), What::Reason(b"must be asked"));
        }
        if playground == Some(true) && !fake {
            problems.push(root.member(b"playground"), What::Reason(b"requires a fake setup entry"));
        }
    }
    if problems.is_empty() {
        Ok(GroupFile { entry: entry.expect("clean setup entry"), playground: playground.expect("clean boolean") })
    } else {
        Err(problems)
    }
}
