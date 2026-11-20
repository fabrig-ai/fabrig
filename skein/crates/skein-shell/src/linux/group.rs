//! Linux's process observations and tree keeper calls (runtime/readiness.md, 5).

use crate::posix::group::negative_group;
pub(crate) use crate::posix::group::{group_live, process_live, signal_group};
use skein_io::kernel::{Error, Fd};

use super::process::map_error;

pub(crate) fn unreaped_children() -> Result<Box<[u32]>, i32> {
    let tasks = std::fs::read_dir("/proc/self/task").map_err(|error| error.raw_os_error().unwrap_or(libc::EIO))?;
    let mut children = Vec::new();
    for task in tasks {
        let task = task.map_err(|error| error.raw_os_error().unwrap_or(libc::EIO))?;
        let reading = match std::fs::read_to_string(task.path().join("children")) {
            Ok(reading) => reading,
            // A thread that ended contributes no children.
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => return Err(error.raw_os_error().unwrap_or(libc::EIO)),
        };
        for pid in reading.split_whitespace() {
            children.push(pid.parse::<u32>().map_err(|_| libc::EIO)?);
        }
    }
    Ok(children.into_boxed_slice())
}

pub(crate) fn child_id(handle: Fd) -> Result<u32, Error> {
    let info = std::fs::read_to_string(format!("/proc/self/fdinfo/{}", handle.raw()))
        .map_err(|error| map_error(error.raw_os_error().unwrap_or(libc::EBADF)))?;
    info.lines()
        .find_map(|line| line.strip_prefix("Pid:").and_then(|value| value.trim().parse().ok()))
        .ok_or(Error::Other(libc::EBADF))
}

pub(crate) fn group_has_other_members(group: u32) -> Result<bool, Error> {
    negative_group(group)?;
    let entries = std::fs::read_dir("/proc").map_err(|error| map_error(error.raw_os_error().unwrap_or(libc::EIO)))?;
    for entry in entries {
        let entry = entry.map_err(|error| map_error(error.raw_os_error().unwrap_or(libc::EIO)))?;
        let Some(pid) = entry.file_name().to_str().and_then(|name| name.parse::<u32>().ok()) else { continue };
        if pid == group {
            continue;
        }
        let stat = match std::fs::read_to_string(entry.path().join("stat")) {
            Ok(stat) => stat,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound || error.raw_os_error() == Some(libc::ESRCH) => {
                continue;
            }
            Err(error) => return Err(map_error(error.raw_os_error().unwrap_or(libc::EIO))),
        };
        let member = stat
            .rsplit_once(") ")
            .and_then(|(_, fields)| fields.split_whitespace().nth(2))
            .and_then(|field| field.parse::<u32>().ok());
        if member == Some(group) {
            return Ok(true);
        }
    }
    Ok(false)
}
