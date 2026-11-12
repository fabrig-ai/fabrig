//! The composition's lines (temper/docs/design/protocol/services.md, 7).
use temper_engine_shell::lines as engine;
use temper_worker_shell::lines as worker;
use temper_service::{End, Note, Unsettled};
use temper_shell::directories::Directories;
#[must_use]
pub fn note_line(binary: &str, note: Note) -> String { match note { Note::Engine(note) => engine::note_line(binary, note), Note::Worker(note) => worker::note_line(binary, note) } }
#[must_use]
pub fn ending_with_binary(binary: &str, end: Option<&End>, failed: bool, directories: &Directories, store: &[u8], unsettled: &[Unsettled]) -> (String, u8) { match end { Some(End::Engine(end)) => engine::ending_with_binary(binary, Some(end), failed, directories, store, unsettled), Some(End::Worker(end)) => worker::ending_with_binary(binary, Some(end), failed, directories, store), None => engine::ending_with_binary(binary, None, failed, directories, store, unsettled) } }
#[cfg(test)]
mod tests;
