//! Demand loads and read pages (temper/docs/design/domain/landing-history.md, 4.3 and 5.1).
use super::Event;
use crate::Key;
use alloc::boxed::Box;
use skein_lib::Wall;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum HistoryRange {
    Events { project: u32 },
    Change { project: u32, change: u64 },
    Landing { project: u32, landing: u64 },
    Last { project: u32 },
    Numbers { project: u32, numbers: Box<[u64]> },
}
impl HistoryRange {
    pub fn valid(&self) -> bool {
        if let Self::Numbers { numbers, .. } = self {
            return numbers.len() <= 32;
        }
        true
    }
    pub fn contains(&self, key: &Key) -> bool {
        match self {
            Self::Events { project } => {
                let Key::HistoryEvent { project: found, .. } = key else { return false };
                project == found
            }
            Self::Change { project, change } => {
                let Key::HistoryChange { project: found, change: task, .. } = key else { return false };
                project == found && change == task
            }
            Self::Landing { project, landing } => {
                let Key::HistoryLanding { project: found, landing: task, .. } = key else { return false };
                project == found && landing == task
            }
            Self::Last { project } => {
                let Key::HistoryLast { project: found, .. } = key else { return false };
                project == found
            }
            Self::Numbers { project, numbers } => {
                let Key::HistoryEvent { project: found, number } = key else { return false };
                self.valid() && project == found && numbers.contains(number)
            }
        }
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Of {
    Project,
    Change(u64),
    Landing(u64),
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Query {
    pub project: u32,
    pub of: Of,
    pub since: Option<Wall>,
    pub until: Option<Wall>,
    pub after: Option<Cursor>,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Cursor {
    pub upper: u64,
    pub after: u64,
}
impl Cursor {
    pub fn encode(self) -> Box<[u8]> {
        let mut writer = skein_lib::Writer::new(17);
        writer.put(&[1]).expect("cursor version fits");
        writer.put(&self.upper.to_be_bytes()).expect("cursor upper fits");
        writer.put(&self.after.to_be_bytes()).expect("cursor after fits");
        writer.finish()
    }
    pub fn decode(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != 17 {
            return None;
        }
        let mut bytes = bytes.iter();
        if bytes.next() != Some(&1) {
            return None;
        }
        let mut upper = [0; 8];
        let mut after = [0; 8];
        for byte in &mut upper {
            *byte = *bytes.next()?;
        }
        for byte in &mut after {
            *byte = *bytes.next()?;
        }
        Some(Self { upper: u64::from_be_bytes(upper), after: u64::from_be_bytes(after) })
    }
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Reply {
    Page { events: Box<[Event]>, after: Option<Cursor>, upper: u64, whole_since: Option<Wall> },
    Gone { from: u64, first: Option<u64> },
    Busy,
}
