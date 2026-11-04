//! Repositories being made (temper/docs/design/domain/forge.md, 4.5).
use alloc::boxed::Box;
use skein_lib::Wall;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct MakingKey {
    pub forge: u16,
    pub org: Box<[u8]>,
    pub name: Box<[u8]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum MakeFrom {
    Empty,
    Copy { address: Box<[u8]> },
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct MakeInto {
    pub project: u32,
    pub home: bool,
    pub host: Box<[u8]>,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum MakeStep {
    Admin,
    Org,
    Repository,
    File,
    Adoption,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum MakeFailure {
    Taken,
    Empty,
    TimedOut,
    Refused { status: u16 },
    NotAllowed,
    Other,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct MakingFailed {
    pub step: MakeStep,
    pub reason: MakeFailure,
    pub words: Box<[u8]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum MakingState {
    Waiting,
    Making,
    Made { id: u64, branch: Box<[u8]> },
    Adopting { id: u64, branch: Box<[u8]> },
    Failed(MakingFailed),
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct RepositoryMaking {
    pub key: MakingKey,
    pub from: MakeFrom,
    pub into: Option<MakeInto>,
    pub asked: Wall,
    pub copying: bool,
    pub state: MakingState,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum MakeRefusal {
    NoSuchForge,
    NotOffered,
    NotOwner,
    Invalid,
    HomeHeld,
    Full,
}

impl crate::Domain {
    #[must_use]
    pub fn making(&self, key: &MakingKey) -> Option<&RepositoryMaking> {
        self.repository_makings.get(key)
    }
    #[must_use]
    pub fn next_making(&self, forge: u16) -> Option<&RepositoryMaking> {
        let mut oldest: Option<&RepositoryMaking> = None;
        for (_, row) in &self.repository_makings {
            if row.key.forge != forge {
                continue;
            }
            match &row.state {
                MakingState::Waiting => {}
                MakingState::Making
                | MakingState::Made { .. }
                | MakingState::Adopting { .. }
                | MakingState::Failed(_) => continue,
            }
            if let Some(kept) = oldest {
                if (row.asked, &row.key) < (kept.asked, &kept.key) {
                    oldest = Some(row);
                }
            } else {
                oldest = Some(row);
            }
        }
        oldest
    }
    #[must_use]
    pub fn making_due(&self, forge: u16) -> Option<MakingKey> {
        for (_, row) in &self.repository_makings {
            if row.key.forge != forge {
                continue;
            }
            match &row.state {
                MakingState::Made { .. } | MakingState::Adopting { .. } => return Some(row.key.clone()),
                MakingState::Waiting | MakingState::Making | MakingState::Failed(_) => {}
            }
        }
        None
    }
    #[expect(
        clippy::disallowed_methods,
        reason = "the boundary returns a borrowed iterator over one forge's rows in key order"
    )]
    pub fn makings(&self, forge: u16) -> impl Iterator<Item = &RepositoryMaking> {
        self.repository_makings.iter().map(|(_, row)| row).filter(move |row| row.key.forge == forge)
    }
}
