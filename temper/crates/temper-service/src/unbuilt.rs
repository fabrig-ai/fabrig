//! Uninhabited worker types for an engine build (protocol/binaries.md, 1 and protocol/components.md).
use crate::settings;

#[derive(Debug)]
pub enum Worker {}
#[derive(Debug)]
pub enum WorkerRuntime {}
#[derive(Debug)]
pub enum Agent {}

impl Agent {
    pub(crate) fn charter_bytes(&self) -> u64 {
        match *self {}
    }
    pub(crate) fn tool_payload(&self) -> u32 {
        match *self {}
    }
    pub(crate) fn host_ask_input(&self) -> u32 {
        match *self {}
    }
    pub(crate) fn host_reply_text(&self) -> u32 {
        match *self {}
    }
    pub(crate) fn turn_body(&self) -> u32 {
        match *self {}
    }
    pub(crate) fn outcome_bytes(&self) -> u64 {
        match *self {}
    }
    pub(crate) fn run_limits(&self) -> &smith_domain_run::Limits {
        match *self {}
    }
    pub(crate) fn bounds(&self) -> &jig_domain_link::Bounds {
        match *self {}
    }
    pub(crate) fn programs(&self) -> &[settings::Program] {
        match *self {}
    }
    pub(crate) fn resolves_endpoint(&self, _name: &[u8]) -> bool {
        match *self {}
    }
}

impl Worker {
    pub(crate) fn drain_notes(&mut self, _sink: &mut skein_lib::Queue<crate::Note>) {
        match *self {}
    }
    pub(crate) fn pending(&self, _now: skein_lib::Time) -> crate::diagnostics::Worker {
        match *self {}
    }
    pub(crate) fn reuse_workspaces(&mut self) {
        match *self {}
    }
    pub(crate) fn dump_llm(&mut self, _directory: &[u8]) {
        match *self {}
    }
    pub(crate) fn is_empty(&self) -> bool {
        match *self {}
    }
    pub(crate) fn operations(&self) -> u32 {
        match *self {}
    }
    pub(crate) fn next_deadline(&self) -> Option<skein_lib::Time> {
        match *self {}
    }
    pub(crate) fn work_pending(&self, _now: skein_lib::Time) -> bool {
        match *self {}
    }
    pub(crate) fn stopping(&self) -> bool {
        match *self {}
    }
    pub(crate) fn abort(&mut self) {
        match *self {}
    }
    pub(crate) fn stop(&mut self) {
        match *self {}
    }
    pub(crate) fn submitted(&mut self) -> &mut skein_lib::Queue<skein_io::kernel::Submit> {
        match *self {}
    }
    pub(crate) fn io(&mut self) -> &mut skein_lib::Queue<skein_io::Request> {
        match *self {}
    }
    pub(crate) fn complete(
        &mut self,
        _now: skein_lib::Time,
        _complete: skein_io::kernel::Complete,
    ) -> Result<(), skein_io::kernel::Complete> {
        match *self {}
    }
    pub(crate) fn agents_ended(&self) -> bool {
        match *self {}
    }
    pub(crate) fn iterate(&mut self, _now: skein_lib::Time, _wall: skein_lib::Wall) {
        match *self {}
    }
    pub(crate) fn refused(&self) -> Option<&jig_protocol_config::Problems> {
        match *self {}
    }
    pub(crate) fn accepts(&self, _event: &skein_io::Event) -> bool {
        match *self {}
    }
    pub(crate) fn up(&mut self, _now: skein_lib::Time, _event: skein_io::Event) {
        match *self {}
    }
}

/// The shell's worker adapter cannot be instantiated without the worker component.
pub mod worker_alone {
    pub use crate::worker_vocabulary::{End, Note, Stop};
    #[derive(Debug)]
    pub enum Service {}
    impl Service {
        pub fn notes(&mut self) -> &mut skein_lib::Queue<Note> { match *self {} }
        #[must_use]
        pub fn end(&self) -> Option<&End> { match *self {} }
    }
}
