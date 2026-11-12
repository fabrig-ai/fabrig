//! Payload-free opening steps and driver counts (protocol/store/opening.md, 1 and 5).
use super::Hosted;
use crate::{directory::Opening, patching::phase::Patching};

#[derive(Debug)]
pub struct Wait {
    pub adopted: bool,
    pub started: bool,
    pub root_pending: bool,
    pub phase: crate::Phase,
    pub opening: &'static str,
    pub rehearsal: Option<&'static str>,
    pub maps_opened: u8,
    pub lanes: [WaitLane; 3],
    pub open_failure: Option<crate::restore::OpenFailure>,
}

#[derive(Debug)]
pub struct WaitLane {
    pub requests: u32,
    pub terminals: u32,
    pub handles: u32,
    pub takes: bool,
}

impl Hosted {
    /// Construct only on inspection; no file names, stored values or credentials.
    #[must_use]
    pub fn wait_state(&self) -> Wait {
        Wait {
            adopted: self.drivers.adopted,
            started: self.drivers.started,
            root_pending: self.drivers.root.is_some(),
            phase: self.component.phase,
            opening: opening(&self.component.directory.opening),
            rehearsal: crate::rehearsal::waiting(&self.component),
            maps_opened: self.component.directory.opened,
            lanes: [self.wait_lane(0), self.wait_lane(1), self.wait_lane(2)],
            open_failure: self.component.start_open_failure().cloned(),
        }
    }
    fn wait_lane(&self, index: usize) -> WaitLane {
        let driver = self.drivers.drivers.get(index).expect("three lanes");
        WaitLane {
            requests: self.drivers.queue(index).len(),
            terminals: driver.above.len(),
            handles: driver.handles.len(),
            takes: driver.io.takes(),
        }
    }
}

fn opening(step: &Opening) -> &'static str {
    match step {
        Opening::Idle => "idle",
        Opening::Locking => "lock deployment",
        Opening::Reading => "read active",
        Opening::SurveyOpen(_) => "open survey directory",
        Opening::SurveyPage { .. } => "list survey directory",
        Opening::SurveyClose(_) => "close survey directory",
        Opening::Refusing(_) => "close before refusal",
        Opening::LeftoverOpen => "open leftover generation",
        Opening::LeftoverRemoving { .. } => "remove leftover generation",
        Opening::LeftoverClose => "close leftover generation",
        Opening::Making(_) => "make first generation",
        Opening::FirstSecrets => "open first secrets",
        Opening::FirstManifest(_) => "write first manifest",
        Opening::FirstActive => "write first active",
        Opening::CheckingOpen => "open active generation",
        Opening::CheckingPage { .. } => "list active generation",
        Opening::CheckingClose => "close active generation",
        Opening::StoreOpen => "open store directory",
        Opening::SecretsOpen => "open secrets directory",
        Opening::Stores => "recover maps and check opening rows",
        Opening::SweepOpen { .. } => "open sweep directory",
        Opening::SweepPage(_) => "list sweep directory",
        Opening::SweepFile(_) => "remove stray file",
        Opening::SweepRemoving { .. } => "remove stray directory",
        Opening::SweepSync { .. } => "sync sweep directory",
        Opening::SweepClose(_) => "close sweep directory",
        Opening::TranscriptsMade => "make transcripts directory",
        Opening::TranscriptsOpen => "open transcripts directory",
        Opening::Patching(step) => match step.as_ref() {
            Patching::Closing { .. } => "close maps before patching",
            Patching::Dropping(_) => "drop previous generation",
            Patching::Copying(_) => "copy patch generation",
            Patching::Mapping { .. } => "open patch directories",
            Patching::Recovering { .. } => "recover patch maps",
            Patching::Walking(_) => "walk patches",
            Patching::Sealing { .. } => "seal patches",
            Patching::Checking(_) => "check patched records",
        },
        Opening::Pending(_) => "pending generation switch",
    }
}
