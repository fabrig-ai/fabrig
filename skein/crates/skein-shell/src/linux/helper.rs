//! Landlock and the child-only helper calls (runtime/kernel-cgroups.md, 1).
//!
//! # The `unsafe`, and why it is sound
//!
//! Landlock/openat2 borrow initialized uapi records and C strings only during
//! each call. Descriptor calls take integers; newly opened descriptors have one
//! owner. Dispatch owns the three descriptors installed by spawn. execve borrows
//! NUL-terminated strings and pointer arrays until it replaces this process.

#![expect(clippy::disallowed_types, reason = "helper transport descriptors have local, unique ownership")]

use std::ffi::CString;
use std::fs::File;
use std::io::{Read, Write};
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
use std::{mem, ptr};

use skein_io::kernel::{Reach, Unenforced};

use crate::helper::{CAPACITY, MARKER, Record, Spec, abi_error, gaps, rule_rights};
use crate::posix::process::cstring;

// Linux uapi linux/landlock.h: ABI 6 ruleset and path-beneath attributes.
#[repr(C)]
struct RulesetAttr {
    handled_access_fs: u64,
    handled_access_net: u64,
    scoped: u64,
}

#[repr(C, packed)]
struct PathAttr {
    allowed_access: u64,
    parent_fd: i32,
}

pub(crate) fn abi() -> Result<u32, Unenforced> {
    // SAFETY: VERSION (1) takes a null attribute and no borrowed storage.
    let answer = unsafe { libc::syscall(libc::SYS_landlock_create_ruleset, ptr::null::<RulesetAttr>(), 0, 1) };
    if answer < 0 {
        Err(abi_error(super::last_errno()))
    } else {
        Ok(u32::try_from(answer).expect("Landlock ABI fits u32"))
    }
}

fn owned(raw: libc::c_long) -> Result<OwnedFd, i32> {
    if raw < 0 {
        Err(super::last_errno())
    } else {
        // SAFETY: the successful syscall returned a newly owned descriptor.
        Ok(unsafe { OwnedFd::from_raw_fd(i32::try_from(raw).expect("descriptor fits i32")) })
    }
}

pub(crate) fn domain(reach: &Reach) -> Result<OwnedFd, Unenforced> {
    let attr = RulesetAttr {
        handled_access_fs: u64::from(u16::MAX), // every ABI 6 filesystem right, including IOCTL_DEV
        handled_access_net: 0,
        scoped: 3, // LANDLOCK_SCOPE_ABSTRACT_UNIX_SOCKET | LANDLOCK_SCOPE_SIGNAL
    };
    // SAFETY: attr has the uapi layout and lives until the call returns.
    let raw = unsafe { libc::syscall(libc::SYS_landlock_create_ruleset, &attr, size_of::<RulesetAttr>(), 0) };
    let ruleset = owned(raw).map_err(|errno| Unenforced::Restrict { errno })?;
    for (index, rule) in reach.rules.iter().enumerate() {
        let added = (|| {
            let path = cstring(if rule.path.is_empty() { b"." } else { &rule.path })?;
            // SAFETY: open_how is an integer C record, zero is valid.
            let mut how: libc::open_how = unsafe { mem::zeroed() };
            how.flags = u64::try_from(libc::O_PATH | libc::O_CLOEXEC).expect("positive flags");
            how.resolve = libc::RESOLVE_BENEATH | libc::RESOLVE_NO_MAGICLINKS;
            // SAFETY: path and how remain live for this synchronous open.
            let raw = unsafe {
                libc::syscall(libc::SYS_openat2, rule.root.raw(), path.as_ptr(), &how, size_of::<libc::open_how>())
            };
            let path = owned(raw)?;
            // SAFETY: stat is a plain C output record, zero is valid.
            let mut stat: libc::stat = unsafe { mem::zeroed() };
            // SAFETY: path is open and stat is writable throughout fstat.
            if unsafe { libc::fstat(path.as_raw_fd(), ptr::from_mut(&mut stat)) } != 0 {
                return Err(super::last_errno());
            }
            let rights = rule_rights(rule.rights, stat.st_mode & libc::S_IFMT == libc::S_IFDIR);
            if rights != 0 {
                let attr = PathAttr { allowed_access: rights, parent_fd: path.as_raw_fd() };
                // SAFETY: PATH_BENEATH (1) reads this packed uapi record only during the call.
                if unsafe { libc::syscall(libc::SYS_landlock_add_rule, ruleset.as_raw_fd(), 1, &attr, 0) } != 0 {
                    return Err(super::last_errno());
                }
            }
            Ok(())
        })();
        added.map_err(|errno| Unenforced::Rule { index: u32::try_from(index).expect("at most 64 rules"), errno })?;
    }
    Ok(ruleset)
}

pub(crate) fn pipe() -> Result<(OwnedFd, OwnedFd), i32> {
    let mut pair = [0; 2];
    // SAFETY: pair holds two writable descriptor slots.
    if unsafe { libc::pipe2(pair.as_mut_ptr(), libc::O_CLOEXEC) } != 0 {
        return Err(super::last_errno());
    }
    let [read, write] = pair;
    Ok((owned(libc::c_long::from(read))?, owned(libc::c_long::from(write))?))
}

pub(crate) fn spec_pipe(spec: &Spec) -> Result<OwnedFd, i32> {
    let bytes = spec.encode()?;
    let (read, write) = pipe()?;
    // SAFETY: fcntl queries the live pipe's capacity, borrowing no memory.
    let capacity = unsafe { libc::fcntl(write.as_raw_fd(), libc::F_GETPIPE_SZ) };
    let needs = i32::try_from(bytes.len()).map_err(|_| libc::E2BIG)?;
    if capacity < needs {
        // SAFETY: fcntl changes only this new pipe's capacity, before any write.
        if unsafe { libc::fcntl(write.as_raw_fd(), libc::F_SETPIPE_SZ, needs) } < 0 {
            return Err(super::last_errno());
        }
    }
    File::from(write).write_all(&bytes).map_err(|error| errno(&error))?;
    Ok(read)
}

fn errno(error: &std::io::Error) -> i32 {
    error.raw_os_error().unwrap_or(libc::EIO)
}

pub(crate) fn report(read: OwnedFd) -> Result<(), skein_io::kernel::Error> {
    use skein_io::kernel::Error;
    // Two records are the longest valid report. Still require EOF after them.
    let mut bytes = Vec::new();
    File::from(read).take(11).read_to_end(&mut bytes).map_err(|_| Error::Reach(Unenforced::Helper))?;
    let records: Option<Vec<_>> = bytes.chunks(5).map(Record::decode).collect();
    match records.as_deref() {
        Some([Record::Restricted]) => Ok(()),
        Some([Record::Restricted, Record::Command(errno)]) => Err(super::process::map_error(*errno)),
        Some([Record::Restrict(errno)]) => Err(Error::Reach(Unenforced::Restrict { errno: *errno })),
        _ => Err(Error::Reach(Unenforced::Helper)),
    }
}

#[expect(
    clippy::disallowed_methods,
    reason = "first-line dispatch reads only the helper marker and its three descriptor arguments"
)]
pub(crate) fn dispatch() {
    use std::os::unix::ffi::OsStrExt;
    let mut args = std::env::args_os().skip(1);
    if args.next().as_deref() != Some(std::ffi::OsStr::new(MARKER)) {
        return;
    }
    let numbers: Option<Vec<i32>> =
        args.map(|word| std::str::from_utf8(word.as_bytes()).ok()?.parse::<i32>().ok().filter(|fd| *fd > 2)).collect();
    let Some([spec, report, ruleset]) = numbers.as_deref() else { std::process::exit(127) };
    if spec == report || spec == ruleset || report == ruleset {
        std::process::exit(127);
    }
    // A marker supplied by hand must not manufacture ownership of a closed fd.
    for descriptor in [spec, report, ruleset] {
        // SAFETY: F_GETFD only checks an integer descriptor, borrowing no memory.
        if unsafe { libc::fcntl(*descriptor, libc::F_GETFD) } < 0 {
            std::process::exit(127);
        }
    }
    // SAFETY: these are open, distinct descriptors relinquished to first-line dispatch.
    let spec = unsafe { OwnedFd::from_raw_fd(*spec) };
    // SAFETY: distinct from the spec and ruleset, installed for dispatch.
    let mut report = File::from(unsafe { OwnedFd::from_raw_fd(*report) });
    // SAFETY: distinct from spec and report, installed for dispatch.
    let ruleset = unsafe { OwnedFd::from_raw_fd(*ruleset) };
    let result = restrict(spec, &report, ruleset);
    let spec = match result {
        Ok(spec) => spec,
        Err(error) => fail(&mut report, Record::Restrict(error)),
    };
    if let Err(error) = report.write_all(&Record::Restricted.encode()) {
        fail(&mut report, Record::Restrict(errno(&error)));
    }
    // SAFETY: report is uniquely owned; fcntl sets its descriptor flag only.
    if unsafe { libc::fcntl(report.as_raw_fd(), libc::F_SETFD, libc::FD_CLOEXEC) } < 0 {
        fail(&mut report, Record::Command(super::last_errno()));
    }
    if spec.program.is_empty() {
        std::process::exit(0);
    }
    let error = execute(&spec);
    fail(&mut report, Record::Command(error));
}

fn restrict(spec: OwnedFd, report: &File, ruleset: OwnedFd) -> Result<Spec, i32> {
    let mut bytes = Vec::new();
    File::from(spec).take(CAPACITY.saturating_add(1)).read_to_end(&mut bytes).map_err(|error| errno(&error))?;
    let spec = Spec::decode(&bytes)?;
    // SAFETY: no_new_privs takes integers and changes only this helper.
    if unsafe { libc::prctl(libc::PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0) } != 0 {
        return Err(super::last_errno());
    }
    // SAFETY: ruleset is open; restrict_self borrows no memory.
    if unsafe { libc::syscall(libc::SYS_landlock_restrict_self, ruleset.as_raw_fd(), 0) } != 0 {
        return Err(super::last_errno());
    }
    drop(ruleset);
    for (from, to) in gaps(&spec.named, u32::try_from(report.as_raw_fd()).expect("positive descriptor")) {
        // SAFETY: each gap excludes report and the descriptors the command keeps.
        if unsafe { libc::syscall(libc::SYS_close_range, from, to, 0) } != 0 {
            return Err(super::last_errno());
        }
    }
    Ok(spec)
}

fn execute(spec: &Spec) -> i32 {
    let strings = (|| {
        let path = cstring(&spec.program)?;
        let args = std::iter::once(&spec.program)
            .chain(spec.args.iter())
            .map(|s| cstring(s))
            .collect::<Result<Vec<_>, _>>()?;
        let env = spec.env.iter().map(|s| cstring(s)).collect::<Result<Vec<_>, _>>()?;
        Ok::<_, i32>((path, args, env))
    })();
    let (path, args, env): (CString, _, _) = match strings {
        Ok(strings) => strings,
        Err(errno) => return errno,
    };
    let argument_pointers: Vec<_> = args.iter().map(|s| s.as_ptr()).chain(std::iter::once(ptr::null())).collect();
    let envp: Vec<_> = env.iter().map(|s| s.as_ptr()).chain(std::iter::once(ptr::null())).collect();
    // SAFETY: all strings and terminated pointer arrays stay live until execve returns or replaces us.
    unsafe { libc::execve(path.as_ptr(), argument_pointers.as_ptr(), envp.as_ptr()) };
    super::last_errno()
}

fn fail(report: &mut File, record: Record) -> ! {
    let _written = report.write_all(&record.encode());
    std::process::exit(127)
}
