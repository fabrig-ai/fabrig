//! Wire facts for the quit key's order against a chat's work (temper/docs/design/client/tui/box.md, 11).
use crate::{
    host::{Host, Report, Settings, Start},
    limits,
};
use jig_client_protocol_wire as w;
use skein_lib::{Time, Wall};
use skein_ui_screen::Size;
use temper_client_tui::{Origin, Scheme, words::Words};

/// Fresh hosts without rebuilding and checking their identical vocabulary per seed.
pub struct Hosts {
    settings: Settings,
    words: Words,
}

impl Hosts {
    #[must_use]
    pub fn new(most: Size) -> Self {
        let (mut limits, words) = limits::checked_tui();
        limits.most = most;
        Self { settings: Settings { limits }, words }
    }

    #[must_use]
    pub fn front(&self, seed: u64) -> Host {
        let mut host = Host::with_checked_words(&self.settings, self.words.clone(), seed, Time::ZERO, Wall::EPOCH);
        host.report(Report::Start(Start {
            size: self.settings.limits.most,
            scheme: Some(Scheme::Dark),
            background: None,
            plain: false,
            version: Box::default(),
            address: Origin::Http("127.0.0.1:8367".parse().expect("fixture socket")),
            local: true,
            boxed: true,
            kept: Vec::new(),
            credential: None,
            note: None,
            front: true,
            set_up: false,
            box_log: Box::default(),
        }));
        host
    }
}

pub const CHAT: u64 = 9;
pub const PROJECT: u32 = 3;
const TITLE: &[u8] = b"csv-export";

#[derive(Clone, Copy, Debug)]
pub enum Read {
    Starting,
    Working,
    Writing,
    Tool,
    Retrying,
    Delegating,
    ParkedOnDelegate,
    Woken,
    Parked,
}

impl Read {
    #[must_use]
    pub fn facts(self, mut details: w::ChipDetailsRow) -> ([w::Fact; 3], w::WaitsOn) {
        let (phase, stage, work) = match self {
            Self::Starting => (w::Phase::Due, w::RunStage::Due, None),
            Self::Working => running(1, w::ChipWork::Active),
            Self::Writing => running(1, w::ChipWork::Writing),
            Self::Tool => running(
                1,
                w::ChipWork::Tool(w::ChipWorkTool { call: b"1".as_slice().into(), tool: b"shell".as_slice().into() }),
            ),
            Self::Retrying => {
                (w::Phase::BackingOff, w::RunStage::BackingOff(w::RunStageBackingOff { until: 100 }), None)
            }
            Self::Delegating => running(2, w::ChipWork::Active),
            Self::Woken => running(3, w::ChipWork::Active),
            Self::ParkedOnDelegate | Self::Parked => (w::Phase::Idle, w::RunStage::Idle, None),
        };
        details.value.project = PROJECT;
        details.value.retry = matches!(self, Self::Retrying)
            .then_some(w::RetryNotice::Refused(w::RetryNoticeRefused { count: 1, reason: None }));
        details.value.observation =
            work.map(|work| w::ChipObservation { work, channel: 1, since: 7, reported_at: 7, started: Some(7) });
        details.value.live_delegates = u32::from(matches!(self, Self::Delegating | Self::ParkedOnDelegate));
        let waits_on = if matches!(self, Self::ParkedOnDelegate) { w::WaitsOn::Delegates } else { w::WaitsOn::Nothing };
        let rows = [
            w::Fact::Chip(w::ChipRow {
                parent: None,
                value: w::ChipFields {
                    task: CHAT,
                    title: w::DocumentText { bytes: TITLE.into(), cut: false },
                    executor: w::ChipExecutor::Agent(w::ChipExecutorAgent { charter: 7 }),
                    phase,
                    details: 1,
                },
            }),
            w::Fact::ChipPhase(w::ChipPhaseRow {
                parent: Some(0),
                value: w::ChipPhaseOwnerFields {
                    phase: w::ChipPhase::Active(w::ChipPhaseActive { value: stage }),
                    details: 2,
                },
            }),
            w::Fact::ChipDetails(Box::new(details)),
        ];
        (rows, waits_on)
    }
}

fn running(attempt: u64, work: w::ChipWork) -> (w::Phase, w::RunStage, Option<w::ChipWork>) {
    (w::Phase::Running, w::RunStage::Running(w::RunStageRunning { attempt }), Some(work))
}
