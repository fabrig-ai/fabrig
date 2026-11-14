//! File requests and terminal events for step machines above io.
//!
//! A file request carries the owner's token and produces exactly one event.
//! Files remain open across positioned operations; names stay beneath a root.

use crate::digest::Digest;
use crate::kernel::Fd;
use alloc::boxed::Box;
use skein_lib::Token;

/// The target version a store's owner permits replacing (runtime/io/files.md, section 2).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Expect {
    /// The owner permits replacing any target, without reading its content.
    Any,
    /// The final rename atomically creates the target only if its name is absent.
    Absent,
    /// Recheck this digest before a replacing rename; an outside writer can race it.
    Digest(Digest),
}

/// A temporary whose cleanup failed, reported by io at the store's terminal.
#[derive(Debug, PartialEq, Eq)]
pub struct Residue {
    /// A bounded leaf name beside the request's target, in the same directory.
    pub name: Box<[u8]>,
    pub error: crate::kernel::Error,
}

#[derive(PartialEq, Eq)]
pub enum Request {
    /// Takes an exclusive lock without waiting: `Locked`, `Busy` or `Failed`.
    Lock {
        owner: Token,
        root: Token,
    },
    /// Makes a directory beneath the owner's root: `Made` or `Failed`.
    MakeDirectory {
        owner: Token,
        root: Token,
        path: Box<[u8]>,
        mode: u32,
    },
    /// Opens or makes a private root: `Opened`, `Refused` or `Failed` (runtime/io/files.md, 3).
    OpenPrivate {
        owner: Token,
        root: Token,
        path: Box<[u8]>,
    },
    Create {
        owner: Token,
        root: Fd,
        name: Box<[u8]>,
        mode: u32,
    },
    /// Creates a file without following a symbolic link in any parent path part.
    CreateNoFollow {
        owner: Token,
        root: Fd,
        name: Box<[u8]>,
        mode: u32,
    },
    OpenRead {
        owner: Token,
        root: Fd,
        name: Box<[u8]>,
    },
    /// Opens a file for reading without following any symbolic link.
    OpenReadNoFollow {
        owner: Token,
        root: Fd,
        name: Box<[u8]>,
    },
    /// Opens an existing regular file to read and write, never creating it.
    OpenWrite {
        owner: Token,
        root: Fd,
        name: Box<[u8]>,
    },
    /// As `OpenWrite`, refusing a symbolic link in any path part.
    OpenWriteNoFollow {
        owner: Token,
        root: Fd,
        name: Box<[u8]>,
    },
    /// Opens a directory beneath a root and gives back a file token for it.
    OpenDirectory {
        owner: Token,
        root: Fd,
        name: Box<[u8]>,
        no_follow: bool,
    },
    /// Loads one whole file through a root token, refusing content above `max` bytes.
    Load {
        owner: Token,
        root: Token,
        path: Box<[u8]>,
        max: u32,
        no_follow: bool,
    },
    /// Scans a directory, retaining its first entries in name order within both bounds.
    Scan {
        owner: Token,
        root: Token,
        path: Box<[u8]>,
        max: u32,
        /// Maximum owned bytes of entry cells and names in the result.
        max_bytes: u64,
        no_follow: bool,
    },
    /// Replaces one file under its stated expectation, through a synced temporary.
    Store {
        owner: Token,
        root: Token,
        path: Box<[u8]>,
        bytes: Box<[u8]>,
        expected: Expect,
        no_follow: bool,
    },
    /// States an open file or directory.
    Stat {
        owner: Token,
        file: Token,
    },
    WriteAt {
        owner: Token,
        file: Token,
        offset: u64,
        bytes: Box<[u8]>,
    },
    ReadAt {
        owner: Token,
        file: Token,
        offset: u64,
        max: u32,
    },
    /// Sets the open file's length, durable once synced: Cut or Failed.
    Cut {
        owner: Token,
        file: Token,
        length: u64,
    },
    Sync {
        owner: Token,
        file: Token,
    },
    Close {
        owner: Token,
        file: Token,
    },
    SyncDirectory {
        owner: Token,
        root: Fd,
    },
    /// Moves a name between open directories: `Renamed` or `Failed` (runtime/io/files.md).
    Rename {
        owner: Token,
        from_root: Fd,
        from: Box<[u8]>,
        to_root: Fd,
        to: Box<[u8]>,
    },
    /// Gives an entry a second name: `Linked` or `Failed` (runtime/io/files.md).
    HardLink {
        owner: Token,
        from_root: Fd,
        from: Box<[u8]>,
        to_root: Fd,
        to: Box<[u8]>,
    },
    /// Removes one file or empty directory by name: `Removed` or `Failed` (runtime/io/files.md).
    Remove {
        owner: Token,
        root: Fd,
        name: Box<[u8]>,
        /// Removes an empty directory when true, a file when false; never its contents.
        directory: bool,
    },
    List {
        owner: Token,
        root: Fd,
    },
    /// Lists at most `max` entries from the owner's open directory (runtime/io/files.md).
    ListPage {
        owner: Token,
        directory: Token,
        max: u32,
    },
}

// runtime/io/files.md: a file's contents print as their length, never their bytes.
impl core::fmt::Debug for Request {
    #[expect(
        clippy::too_many_lines,
        reason = "runtime/io/files.md: exhaustively preserve every request variant and field"
    )]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Request::Lock { owner, root } => f.debug_struct("Lock").field("owner", owner).field("root", root).finish(),
            Request::MakeDirectory { owner, root, path, mode } => f
                .debug_struct("MakeDirectory")
                .field("owner", owner)
                .field("root", root)
                .field("path", path)
                .field("mode", mode)
                .finish(),
            Request::OpenPrivate { owner, root, path } => {
                f.debug_struct("OpenPrivate").field("owner", owner).field("root", root).field("path", path).finish()
            }
            Request::Create { owner, root, name, mode } => f
                .debug_struct("Create")
                .field("owner", owner)
                .field("root", root)
                .field("name", name)
                .field("mode", mode)
                .finish(),
            Request::CreateNoFollow { owner, root, name, mode } => f
                .debug_struct("CreateNoFollow")
                .field("owner", owner)
                .field("root", root)
                .field("name", name)
                .field("mode", mode)
                .finish(),
            Request::OpenRead { owner, root, name } => {
                f.debug_struct("OpenRead").field("owner", owner).field("root", root).field("name", name).finish()
            }
            Request::OpenReadNoFollow { owner, root, name } => f
                .debug_struct("OpenReadNoFollow")
                .field("owner", owner)
                .field("root", root)
                .field("name", name)
                .finish(),
            Request::OpenWrite { owner, root, name } => {
                f.debug_struct("OpenWrite").field("owner", owner).field("root", root).field("name", name).finish()
            }
            Request::OpenWriteNoFollow { owner, root, name } => f
                .debug_struct("OpenWriteNoFollow")
                .field("owner", owner)
                .field("root", root)
                .field("name", name)
                .finish(),
            Request::OpenDirectory { owner, root, name, no_follow } => f
                .debug_struct("OpenDirectory")
                .field("owner", owner)
                .field("root", root)
                .field("name", name)
                .field("no_follow", no_follow)
                .finish(),
            Request::Load { owner, root, path, max, no_follow } => f
                .debug_struct("Load")
                .field("owner", owner)
                .field("root", root)
                .field("path", path)
                .field("max", max)
                .field("no_follow", no_follow)
                .finish(),
            Request::Scan { owner, root, path, max, max_bytes, no_follow } => f
                .debug_struct("Scan")
                .field("owner", owner)
                .field("root", root)
                .field("path", path)
                .field("max", max)
                .field("max_bytes", max_bytes)
                .field("no_follow", no_follow)
                .finish(),
            Request::Store { owner, root, path, bytes, expected, no_follow } => f
                .debug_struct("Store")
                .field("owner", owner)
                .field("root", root)
                .field("path", path)
                .field("len", &bytes.len())
                .field("expected", expected)
                .field("no_follow", no_follow)
                .finish(),
            Request::Stat { owner, file } => f.debug_struct("Stat").field("owner", owner).field("file", file).finish(),
            Request::WriteAt { owner, file, offset, bytes } => f
                .debug_struct("WriteAt")
                .field("owner", owner)
                .field("file", file)
                .field("offset", offset)
                .field("len", &bytes.len())
                .finish(),
            Request::ReadAt { owner, file, offset, max } => f
                .debug_struct("ReadAt")
                .field("owner", owner)
                .field("file", file)
                .field("offset", offset)
                .field("max", max)
                .finish(),
            Request::Cut { owner, file, length } => {
                f.debug_struct("Cut").field("owner", owner).field("file", file).field("length", length).finish()
            }
            Request::Sync { owner, file } => f.debug_struct("Sync").field("owner", owner).field("file", file).finish(),
            Request::Close { owner, file } => {
                f.debug_struct("Close").field("owner", owner).field("file", file).finish()
            }
            Request::SyncDirectory { owner, root } => {
                f.debug_struct("SyncDirectory").field("owner", owner).field("root", root).finish()
            }
            Request::Rename { owner, from_root, from, to_root, to } => f
                .debug_struct("Rename")
                .field("owner", owner)
                .field("from_root", from_root)
                .field("from", from)
                .field("to_root", to_root)
                .field("to", to)
                .finish(),
            Request::HardLink { owner, from_root, from, to_root, to } => f
                .debug_struct("HardLink")
                .field("owner", owner)
                .field("from_root", from_root)
                .field("from", from)
                .field("to_root", to_root)
                .field("to", to)
                .finish(),
            Request::Remove { owner, root, name, directory } => f
                .debug_struct("Remove")
                .field("owner", owner)
                .field("root", root)
                .field("name", name)
                .field("directory", directory)
                .finish(),
            Request::List { owner, root } => f.debug_struct("List").field("owner", owner).field("root", root).finish(),
            Request::ListPage { owner, directory, max } => f
                .debug_struct("ListPage")
                .field("owner", owner)
                .field("directory", directory)
                .field("max", max)
                .finish(),
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
pub struct Entry {
    pub name: Box<[u8]>,
    pub kind: crate::kernel::Kind,
}

/// The unsafe metadata io found before reading a private root or file (runtime/io/files.md, 3).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Unsafe {
    /// The requested name or a parent is a symbolic link.
    Link,
    /// The opened root is not a directory or its file is not regular.
    Kind(crate::kernel::Kind),
    /// The opened node has these group or other permission bits.
    Mode(u32),
    /// The opened file has this link count, rather than one.
    Links(u32),
    /// The opened node belongs to someone other than the configured effective user.
    Owner { found: u32, expected: u32 },
}

#[derive(PartialEq, Eq)]
pub enum Event {
    /// The lock is taken, or already held through this root.
    Locked {
        owner: Token,
    },
    /// Another open holds the lock; the owner may ask again on its own timer.
    Busy {
        owner: Token,
    },
    /// The owner's directory request made its directory.
    Made {
        owner: Token,
    },
    /// The private root or load was refused unread with this unsafe metadata.
    Refused {
        owner: Token,
        found: Unsafe,
    },
    Opened {
        owner: Token,
        file: Token,
        len: u64,
    },
    /// A whole file loaded within its requested byte bound.
    Loaded {
        owner: Token,
        bytes: Box<[u8]>,
    },
    /// The first bounded directory entries in name order, and the number omitted.
    Scanned {
        owner: Token,
        entries: Box<[Entry]>,
        more: u64,
    },
    /// A whole-file replacement is durable and has this content digest.
    Stored {
        owner: Token,
        digest: Digest,
    },
    /// The expectation failed at the recheck, or an `Absent` target existed
    /// at the atomic final rename.
    Conflict {
        owner: Token,
        now: Option<Digest>,
    },
    /// A load exceeded its requested byte bound; `size` is the observed size.
    TooLarge {
        owner: Token,
        size: u64,
    },
    /// Kind, size, permissions, owner and links of the open file or directory.
    Stated {
        owner: Token,
        stat: crate::kernel::Stat,
    },
    Written {
        owner: Token,
    },
    Read {
        owner: Token,
        bytes: Box<[u8]>,
    },
    /// The file's length is the one asked for.
    Cut {
        owner: Token,
    },
    Synced {
        owner: Token,
    },
    Closed {
        owner: Token,
    },
    Linked {
        owner: Token,
    },
    Renamed {
        owner: Token,
    },
    Removed {
        owner: Token,
    },
    Listed {
        owner: Token,
        entries: Box<[Entry]>,
    },
    Failed {
        owner: Token,
        error: crate::kernel::Error,
        /// A Store's rename succeeded before this failure. The new file is
        /// present; its durability may be unknown (runtime/io/files.md, section 2).
        /// False for failures of other file requests.
        committed: bool,
        /// A temporary still beside the target after its removal failed.
        residue: Option<Residue>,
    },
    /// The owner stopped a file request before it completed.
    Cancelled {
        owner: Token,
    },
}

// runtime/io/files.md: a file's contents print as their length, never their bytes.
impl core::fmt::Debug for Event {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Event::Locked { owner } => f.debug_struct("Locked").field("owner", owner).finish(),
            Event::Busy { owner } => f.debug_struct("Busy").field("owner", owner).finish(),
            Event::Made { owner } => f.debug_struct("Made").field("owner", owner).finish(),
            Event::Refused { owner, found } => {
                f.debug_struct("Refused").field("owner", owner).field("found", found).finish()
            }
            Event::Opened { owner, file, len } => {
                f.debug_struct("Opened").field("owner", owner).field("file", file).field("len", len).finish()
            }
            Event::Loaded { owner, bytes } => {
                f.debug_struct("Loaded").field("owner", owner).field("len", &bytes.len()).finish()
            }
            Event::Scanned { owner, entries, more } => {
                f.debug_struct("Scanned").field("owner", owner).field("entries", entries).field("more", more).finish()
            }
            Event::Stored { owner, digest } => {
                f.debug_struct("Stored").field("owner", owner).field("digest", digest).finish()
            }
            Event::Conflict { owner, now } => {
                f.debug_struct("Conflict").field("owner", owner).field("now", now).finish()
            }
            Event::TooLarge { owner, size } => {
                f.debug_struct("TooLarge").field("owner", owner).field("size", size).finish()
            }
            Event::Stated { owner, stat } => {
                f.debug_struct("Stated").field("owner", owner).field("stat", stat).finish()
            }
            Event::Written { owner } => f.debug_struct("Written").field("owner", owner).finish(),
            Event::Read { owner, bytes } => {
                f.debug_struct("Read").field("owner", owner).field("len", &bytes.len()).finish()
            }
            Event::Cut { owner } => f.debug_struct("Cut").field("owner", owner).finish(),
            Event::Synced { owner } => f.debug_struct("Synced").field("owner", owner).finish(),
            Event::Closed { owner } => f.debug_struct("Closed").field("owner", owner).finish(),
            Event::Linked { owner } => f.debug_struct("Linked").field("owner", owner).finish(),
            Event::Renamed { owner } => f.debug_struct("Renamed").field("owner", owner).finish(),
            Event::Removed { owner } => f.debug_struct("Removed").field("owner", owner).finish(),
            Event::Listed { owner, entries } => {
                f.debug_struct("Listed").field("owner", owner).field("entries", entries).finish()
            }
            Event::Failed { owner, error, committed, residue } => f
                .debug_struct("Failed")
                .field("owner", owner)
                .field("error", error)
                .field("committed", committed)
                .field("residue", residue)
                .finish(),
            Event::Cancelled { owner } => f.debug_struct("Cancelled").field("owner", owner).finish(),
        }
    }
}

impl Event {
    pub(crate) fn failed(owner: Token, error: crate::kernel::Error) -> Event {
        Event::Failed { owner, error, committed: false, residue: None }
    }

    #[must_use]
    pub const fn owner(&self) -> Token {
        match self {
            Event::Locked { owner }
            | Event::Busy { owner }
            | Event::Made { owner }
            | Event::Refused { owner, .. }
            | Event::Opened { owner, .. }
            | Event::Loaded { owner, .. }
            | Event::Scanned { owner, .. }
            | Event::Stored { owner, .. }
            | Event::Conflict { owner, .. }
            | Event::TooLarge { owner, .. }
            | Event::Stated { owner, .. }
            | Event::Written { owner }
            | Event::Read { owner, .. }
            | Event::Cut { owner }
            | Event::Synced { owner }
            | Event::Closed { owner }
            | Event::Linked { owner }
            | Event::Renamed { owner }
            | Event::Removed { owner }
            | Event::Listed { owner, .. }
            | Event::Failed { owner, .. }
            | Event::Cancelled { owner } => *owner,
        }
    }
}
