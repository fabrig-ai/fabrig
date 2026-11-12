//! A live run's twin (testing/fakes.md, 1), read after a clean stop.
mod cli;
mod document;
mod frames;
mod takes;

pub use takes::{Place, ValueTake, read_takes};

pub use cli::{TABLE, main};

use std::path::PathBuf;

#[derive(Debug)]
pub struct Input {
    pub data: PathBuf,
    pub base: Box<[u8]>,
    pub takes: Box<[Take]>,
    pub replaces: Box<[Replace]>,
    pub values: Box<[ValueTake]>,
}
#[derive(Debug)]
pub struct Take {
    pub task: u64,
    pub cue: Box<[u8]>,
}
#[derive(Debug)]
pub struct Replace {
    pub old: Box<[u8]>,
    pub new: Box<[u8]>,
}
#[derive(Debug)]
pub struct Twin {
    pub document: Box<[u8]>,
    pub report: Report,
}
#[derive(Debug)]
pub struct Report {
    pub turns: Box<[(u64, u32)]>,
    pub replaced: Box<[u32]>,
    pub taken: Box<[u32]>,
    /// Opaque blocks omitted from each task, including tasks with none.
    pub dropped: Box<[(u64, u32)]>,
}
#[derive(Debug, PartialEq, Eq)]
pub enum Refusal {
    Read { path: PathBuf },
    Frame { task: u64, at: u64 },
    Version { task: u64, version: u16 },
    Turn { task: u64, turn: u32, why: Unheld },
    SameCue { cue: Box<[u8]> },
    Bounds { what: &'static str },
    Base,
    Takes { why: String },
    UnusedTake { name: String },
}
#[derive(Debug, PartialEq, Eq)]
pub enum Unheld {
    CutShort,
    Compaction,
}

/// Transform an existing twin without reading transcripts.
pub fn from_document(bytes: &[u8], replaces: &[Replace], values: &[ValueTake]) -> Result<Twin, Refusal> {
    let mut base = document::base(bytes)?;
    transforms(replaces, values)?;
    let mut counts = vec![0; replaces.len()];
    let mut taken = vec![0; values.len()];
    document::transform(&mut base, &["cued", "asked", "otherwise"], replaces, &mut counts, values, &mut taken)?;
    Ok(Twin {
        document: document::checked(&base)?,
        report: Report {
            turns: Box::new([]),
            replaced: counts.into_boxed_slice(),
            dropped: Box::new([]),
            taken: taken.into_boxed_slice(),
        },
    })
}
fn transforms(replaces: &[Replace], values: &[ValueTake]) -> Result<(), Refusal> {
    takes::validate(values)?;
    for replace in replaces {
        if replace.old.is_empty() {
            return Err(Refusal::Bounds { what: "empty replace" });
        }
    }
    Ok(())
}

/// Convert selected tasks' completions to scripts matched by their opening
/// request or system brief, replacing both script lists in the base.
/// The caller guarantees a cleanly stopped input.
///
/// # Errors
/// Refuses unreadable or invalid frames, unsupported turns, duplicate cues,
/// invalid bases and documents beyond the fake's bounds.
pub fn twin(input: &Input) -> Result<Twin, Refusal> {
    let mut base = document::base(&input.base)?;
    if input.takes.len() > temper_fake::ENTRIES as usize {
        return Err(Refusal::Bounds { what: "scripts" });
    }
    for (i, take) in input.takes.iter().enumerate() {
        if take.cue.is_empty() || take.cue.len() > 256 || std::str::from_utf8(&take.cue).is_err() {
            return Err(Refusal::Bounds { what: "cue" });
        }
        for previous in &input.takes[..i] {
            if previous.cue == take.cue {
                return Err(Refusal::SameCue { cue: take.cue.clone() });
            }
        }
    }
    transforms(&input.replaces, &input.values)?;
    let mut turns = Vec::new();
    let mut replaced = vec![0; input.replaces.len()];
    let mut cued = Vec::new();
    let mut asked = Vec::new();
    let mut dropped = Vec::new();
    for take in &input.takes {
        let completions = frames::read(&input.data, take.task)?;
        let count = u32::try_from(completions.len()).expect("sixteen turns");
        let mut blocks = 0;
        let script = document::script(take, &completions, &input.replaces, &mut replaced, &mut blocks)?;
        if document::asked(take, &completions) {
            asked.push(script);
        } else {
            cued.push(script);
        }
        dropped.push((take.task, blocks));
        turns.push((take.task, count));
    }
    document::finish(&mut base, &cued, &asked)?;
    let mut taken = vec![0; input.values.len()];
    document::transform(&mut base, &["cued", "asked"], &[], &mut [], &input.values, &mut taken)?;
    let document = document::checked(&base)?;
    Ok(Twin {
        document,
        report: Report {
            turns: turns.into_boxed_slice(),
            replaced: replaced.into_boxed_slice(),
            taken: taken.into_boxed_slice(),
            dropped: dropped.into_boxed_slice(),
        },
    })
}
