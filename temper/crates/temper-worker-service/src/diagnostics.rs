//! Worker snapshots (temper/docs/design/protocol/services.md, 5).
use jig_domain_link::Hosting;
use skein_lib::{List, Time};
#[derive(Debug)]
#[expect(clippy::struct_excessive_bools, reason = "each flag identifies a separate worker readiness gate")]
pub struct Worker {
    pub work_pending: bool,
    pub started: bool,
    pub aborted: bool,
    pub root_ready: bool,
    pub events: List<WorkerEvent>,
    pub process_events: u32,
    pub io: u32,
    pub submitted: u32,
    pub outputs: u32,
    pub link_outputs: u32,
    pub retained_turns: u32,
    pub kept_answers: u32,
    pub next_deadline: Option<Time>,
    pub attempts: List<Hosting>,
}

#[derive(Debug)]
pub enum WorkerEvent {
    SourceAnswered,
    Assign,
    Link(LinkEvent),
    Ready,
    Lost,
    GracePassed,
    Resend,
    RelayCancelled,
    Process,
    Live,
    Decoded,
    Encoded,
    Activity,
    Done,
    Shutdown,
}

#[derive(Debug)]
pub enum LinkEvent {
    ReadList,
    Sources,
    Check,
    SignOut,
    Assign,
    Message,
    Relayed,
    TurnKept,
    Acknowledge,
    Cancel,
    Grant,
}

#[derive(Debug)]
#[expect(clippy::struct_excessive_bools, reason = "each flag names a separate service readiness term")]
pub struct Pending {
    pub work_pending: bool,
    pub io_ready: bool,
    pub io_due: bool,
    pub completions: u32,
    pub told: u32,
    pub raw_submissions: u32,
    pub ended: bool,
    pub signals: bool,
    pub signals_closing: bool,
    pub worker: Worker,
}
#[derive(Debug)]
pub struct Wait {
    pub worker_empty: bool,
    pub held_listeners: u32,
    pub listener_closes: u32,
    pub signals_closing: bool,
    pub operations: bool,
    pub submissions: u32,
}
