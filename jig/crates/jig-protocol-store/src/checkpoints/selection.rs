//! Sources at the held cut, one map page per step (protocol/store/checkpoints.md, 4.3).
use super::FILES;
use crate::{
    CheckpointRefusal, Store,
    codec::component::{CheckpointFileParts, Transcript},
    pages,
};
use alloc::boxed::Box;
use skein_lib::{List, Reader, Set};

#[derive(Debug)]
pub(crate) struct Source {
    pub row: CheckpointFileParts,
    /// None means the immutable whole file, measured when opened.
    pub end: Option<u64>,
}
#[derive(Debug)]
pub(crate) enum Walk {
    Proofs,
    Places,
    Extents,
    Done,
}
#[derive(Debug)]
pub(crate) struct Selection {
    pub files: List<Source>,
    pub live: Set<u64>,
    pub walk: Walk,
    pub start: Box<[u8]>,
}
impl Selection {
    pub fn new() -> Self {
        Self {
            files: List::with_capacity(FILES),
            live: Set::with_capacity(FILES),
            walk: Walk::Proofs,
            start: Box::new([1, 0, 2]),
        }
    }
    pub fn add(&mut self, part: u8, number: u64, end: Option<u64>, link: bool) -> Result<(), CheckpointRefusal> {
        self.files
            .push(Source {
                row: CheckpointFileParts { part, number, length: end.unwrap_or(0), way: if link { 1 } else { 2 } },
                end,
            })
            .or(Err(CheckpointRefusal::Files))
    }
    pub fn held(
        &mut self,
        secret: bool,
        snapshot: bool,
        segments: &[skein_kv::HeldSegment],
    ) -> Result<(), CheckpointRefusal> {
        if snapshot {
            self.add(if secret { 5 } else { 1 }, 0, None, !secret)?;
        }
        for segment in segments {
            self.add(if secret { 6 } else { 2 }, segment.first, segment.end, !secret && segment.end.is_none())?;
        }
        Ok(())
    }
    pub fn page(&mut self, store: &mut Store) -> Result<bool, CheckpointRefusal> {
        let end: Box<[u8]> = match self.walk {
            Walk::Proofs => Box::new([1, 0, 3]),
            Walk::Places => Box::new([0, 2]),
            Walk::Extents => Box::new([0, 10]),
            Walk::Done => return Ok(true),
        };
        let page = pages::load(store, skein_kv::Range { start: self.start.clone(), end: Some(end) })
            .or(Err(CheckpointRefusal::Unavailable))?;
        for row in page.rows {
            match self.walk {
                Walk::Proofs => {
                    let task = number(&row.key, &[1, 0, 2]).ok_or(CheckpointRefusal::Unavailable)?;
                    self.live.insert(task).or(Err(CheckpointRefusal::Files))?;
                }
                Walk::Places => {
                    let task = number(&row.key, &[0, 1]).ok_or(CheckpointRefusal::Unavailable)?;
                    let place = Transcript::decode(&row.value).or(Err(CheckpointRefusal::Unavailable))?;
                    self.add(4, task, Some(place.length), !self.live.contains(&task))?;
                }
                Walk::Extents => {
                    let (log, extent) =
                        crate::logs::extent_row(&row.key, &row.value).ok_or(CheckpointRefusal::Unavailable)?;
                    let turns = log & 1 == 1;
                    self.add(3, log, Some(extent), turns && !self.live.contains(&(log >> 1_u32)))?;
                }
                Walk::Done => unreachable!("finished selection loads no page"),
            }
        }
        if let Some(next) = page.next {
            self.start = next;
        } else {
            self.walk = match self.walk {
                Walk::Proofs => {
                    self.start = Box::new([0, 1]);
                    Walk::Places
                }
                Walk::Places => {
                    self.start = Box::new([0, 9]);
                    Walk::Extents
                }
                Walk::Extents => Walk::Done,
                Walk::Done => unreachable!("finished selection loads no page"),
            };
        }
        Ok(match self.walk {
            Walk::Done => true,
            Walk::Proofs | Walk::Places | Walk::Extents => false,
        })
    }
}
fn number(key: &[u8], prefix: &[u8]) -> Option<u64> {
    let bytes = key.strip_prefix(prefix)?;
    let mut reader = Reader::new(bytes);
    let number = reader.u64()?;
    if reader.is_empty() { Some(number) } else { None }
}
