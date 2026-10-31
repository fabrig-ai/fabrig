//! Bounded machine ownership (programming-model.md, section 6.3;
//! protocol/agent.md, section 2).

use skein_json::tokenizer;
use skein_lib::Duration;

/// File translation capacity, checked before effects.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Limits {
    /// Maximum simultaneous file requests.
    pub operations: u32,
    /// Maximum admitted workspace roots.
    pub roots: u32,
    /// Maximum relative path bytes.
    pub path_bytes: u32,
    /// Maximum content bytes for load or store.
    pub file_bytes: u32,
    /// Maximum combined command, pattern or filter argument bytes.
    pub tool_payload: u32,
    /// Maximum entries retained from a tree listing.
    pub entries: u32,
    /// Maximum retained entry cells and names.
    pub entry_bytes: u64,
    /// Maximum simultaneous child processes.
    pub processes: u32,
    /// Upper bound for either command output window.
    pub output_bytes: u32,
    /// Upper bound for retained search hits.
    pub search_hits: u32,
    /// Upper bound for retained search path and text bytes.
    pub search_bytes: u32,
    /// Maximum one rg JSON or NUL-delimited path record accepted for parsing.
    pub search_line_bytes: u32,
    /// Maximum configured command environment bytes.
    pub env_bytes: u32,
    /// Grace between a polite signal and a kill.
    pub group_stop: Duration,
}

/// Whether the machine's capacities and search edge fit (programming-model.md, 6.3).
#[must_use]
pub fn valid(limits: &Limits) -> bool {
    limits.processes.checked_mul(2).is_some()
        && tokenizer::tokenizer_bound(&tokenizer::Limits {
            depth: 8,
            string: limits.search_line_bytes.max(1),
            number: 20,
            chunk: 64,
            length: limits.search_line_bytes.max(1),
        })
        .is_some()
}

/// Numeric declarations needed by this layer's transitional derivation.
/// The service supplies only owned numbers and this domain's own child limits.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Derivation {
    pub operations: u32,
    pub processes: u32,
    pub roots: u32,
    pub path_bytes: u32,
    pub file_bytes: u32,
    pub tool_payload: u32,
    pub entries: u32,
    pub entry_bytes: u64,
    pub shell_head: u32,
    pub shell_tail: u32,
    pub search_hits: u32,
    pub search_bytes: u32,
    pub environment_bytes: u32,
    pub group_stop: Duration,
}

/// The first field this layer cannot derive, returned to its parent before effects.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Field {
    /// A JSON search line, derived from retained bytes and escaping.
    SearchLineBytes,
}

impl Field {
    /// The field name in the effective limits, without its layer.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::SearchLineBytes => "search_line_bytes",
        }
    }
}

/// Derive this layer from declarations, refusing checked arithmetic overflow.
pub fn derive(inputs: &Derivation) -> Result<Limits, Field> {
    Ok(Limits {
        operations: inputs.operations,
        roots: inputs.roots,
        path_bytes: inputs.path_bytes,
        file_bytes: inputs.file_bytes,
        tool_payload: inputs.tool_payload,
        entries: inputs.entries,
        entry_bytes: inputs.entry_bytes,
        processes: inputs.processes,
        output_bytes: inputs.shell_head.max(inputs.shell_tail).max(inputs.search_bytes),
        search_hits: inputs.search_hits,
        search_bytes: inputs.search_bytes,
        // JSON escapes one text/path byte into at most six bytes. The fixed
        // envelope includes match offsets, line number and routing keys.
        search_line_bytes: inputs
            .search_bytes
            .checked_add(inputs.path_bytes)
            .ok_or(Field::SearchLineBytes)?
            .checked_mul(6)
            .ok_or(Field::SearchLineBytes)?
            .checked_add(256)
            .ok_or(Field::SearchLineBytes)?,
        env_bytes: inputs.environment_bytes,
        group_stop: inputs.group_stop,
    })
}

#[cfg(test)]
mod memory_tests;
