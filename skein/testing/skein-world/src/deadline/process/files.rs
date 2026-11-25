//! Open procfs records for repeated arming scans (testing/service-tiers.md, 1).

use alloc::collections::{BTreeMap, btree_map::Entry};
use std::fs::File;
use std::io::{self, Read};
use std::os::unix::fs::FileExt;

/// Two close-on-exec descriptors per task, well below the usual 1024 fd limit.
const TASK_CAP: usize = 192;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct TaskIdentity {
    pub pid: u32,
    pub process_start: u64,
    pub inode: u64,
}

#[derive(Debug, Default)]
pub(in crate::deadline) struct SampleFiles {
    tasks: BTreeMap<u32, TaskFiles>,
}

impl SampleFiles {
    pub(super) fn task(&mut self, tid: u32, identity: TaskIdentity) -> Option<&mut TaskFiles> {
        let room = self.tasks.len() < TASK_CAP;
        let task = match self.tasks.entry(tid) {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) if room => entry.insert(TaskFiles::new(identity)),
            Entry::Vacant(_) => return None,
        };
        if task.identity != identity {
            *task = TaskFiles::new(identity);
        }
        task.seen = true;
        Some(task)
    }

    pub(super) fn drop_unseen(&mut self) {
        self.tasks.retain(|_, task| core::mem::take(&mut task.seen));
    }
}

#[derive(Debug)]
pub(super) struct TaskFiles {
    identity: TaskIdentity,
    seen: bool,
    schedstat: Option<File>,
    children: Option<File>,
}

impl TaskFiles {
    fn new(identity: TaskIdentity) -> Self {
        Self { identity, seen: false, schedstat: None, children: None }
    }

    pub(super) fn filling(&self) -> bool {
        self.schedstat.is_none() || self.children.is_none()
    }

    pub(super) fn schedstat(&mut self) -> &mut Option<File> {
        &mut self.schedstat
    }

    pub(super) fn children(&mut self) -> &mut Option<File> {
        &mut self.children
    }

    pub(super) fn clear(&mut self) {
        self.schedstat = None;
        self.children = None;
    }
}

pub(super) fn read_fresh(file: &File, text: &mut String) -> io::Result<usize> {
    // Each scan starts the procfs record again at offset zero. Retaining a
    // descriptor never retains any previous scheduler or children figures.
    AtStart { file, offset: 0 }.take(u64::MAX).read_to_string(text)
}

struct AtStart<'a> {
    file: &'a File,
    offset: u64,
}

impl Read for AtStart<'_> {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        let read = self.file.read_at(bytes, self.offset)?;
        self.offset = self.offset.saturating_add(u64::try_from(read).unwrap_or(u64::MAX));
        Ok(read)
    }
}
