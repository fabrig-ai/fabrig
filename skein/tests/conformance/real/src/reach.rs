//! Host support for the shared reach scenarios (testing/conformance.md, 1).

use std::ffi::OsStr;
use std::fs;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

use skein_io::kernel::{Fd, Rights, Rule};

use crate::Real;

pub(super) fn machine_rules(real: &mut Real, process: usize, executable: &[u8]) -> Vec<Rule> {
    real.set_helper(process, Box::from(executable));
    let root = skein_shell::open_root(Path::new("/")).expect("the machine root");
    real.kernel(process).descriptors.insert(root);
    let fixture = fs::canonicalize(Path::new(OsStr::from_bytes(executable))).expect("the probe's path");
    let mut paths = vec![(PathBuf::from("/usr"), Rights::READ_EXECUTE)];
    for library in ["/lib", "/lib64"] {
        if fs::symlink_metadata(library).is_ok_and(|metadata| metadata.is_dir()) {
            paths.push((PathBuf::from(library), Rights::READ_EXECUTE));
        }
    }
    paths.push((fixture.parent().expect("the probe directory").to_path_buf(), Rights::READ_EXECUTE));
    #[cfg(target_os = "linux")]
    paths.push((PathBuf::from("/proc"), Rights::READ));
    paths
        .into_iter()
        .map(|(path, rights)| Rule {
            root,
            path: Box::from(path.strip_prefix("/").expect("a machine path beneath /").as_os_str().as_bytes()),
            rights,
        })
        .collect()
}

#[cfg(target_os = "linux")]
pub(super) fn set_id_copy(root: Fd, executable: &[u8], path: &[u8]) -> Result<(), String> {
    use std::os::unix::fs::{PermissionsExt, chown};

    let directory = fs::canonicalize(format!("/proc/self/fd/{}", root.raw())).expect("the scenario's root path");
    if nosuid(&directory) {
        return Err("the scratch filesystem is mounted nosuid".into());
    }
    let status = fs::read_to_string("/proc/self/status").expect("the service's ids");
    let uid = ids(&status, "Uid:");
    let gid = ids(&status, "Gid:");
    let copy = directory.join(OsStr::from_bytes(path));
    let (user, group, mode) = if uid == 0 {
        // Any nonzero user ID differs from root; no account database is needed.
        (Some(1), None, 0o4755)
    } else {
        let groups = status.lines().find_map(|line| line.strip_prefix("Groups:")).expect("supplementary groups");
        let Some(group) =
            groups.split_whitespace().map(|part| part.parse::<u32>().expect("a group ID")).find(|id| *id != gid)
        else {
            return Err("the service has no supplementary group other than its primary group and is not root".into());
        };
        (None, Some(group), 0o2755)
    };
    fs::copy(Path::new(OsStr::from_bytes(executable)), &copy).expect("the set-id probe copy");
    chown(&copy, user, group).expect("the set-id copy's owner");
    fs::set_permissions(&copy, fs::Permissions::from_mode(mode)).expect("the set-id copy's mode");
    Ok(())
}

#[cfg(target_os = "linux")]
fn ids(status: &str, name: &str) -> u32 {
    status
        .lines()
        .find_map(|line| line.strip_prefix(name))
        .expect("the process's ids")
        .split_whitespace()
        .next()
        .expect("the real id")
        .parse()
        .expect("a numeric id")
}

#[cfg(target_os = "linux")]
fn nosuid(path: &Path) -> bool {
    let mounts = fs::read_to_string("/proc/self/mountinfo").expect("the scratch filesystem's mount options");
    mounts
        .lines()
        .filter_map(|line| {
            let mut fields = line.split_whitespace();
            let mount = fields.nth(4)?;
            let options = fields.next()?;
            let mount =
                mount.replace("\\040", " ").replace("\\011", "\t").replace("\\012", "\n").replace("\\134", "\\");
            let mount = PathBuf::from(mount);
            path.starts_with(&mount)
                .then(|| (mount.components().count(), options.split(',').any(|option| option == "nosuid")))
        })
        .max_by_key(|(depth, _)| *depth)
        .expect("the scratch path has a mount")
        .1
}

#[cfg(target_os = "macos")]
pub(super) fn set_id_copy(_root: Fd, _executable: &[u8], _path: &[u8]) -> Result<(), String> {
    Err("macOS refuses reached spawns before the probe executes".into())
}
