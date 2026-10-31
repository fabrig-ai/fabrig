//! Retained call payload bounds, independent of admission (domain/engine.md, 6).
use super::CallPart;
use crate::Limits;
use jig_domain_notes as notes;

impl CallPart {
    #[expect(clippy::wildcard_enum_match_arm, reason = "only these call parts contain variable-size collections")]
    pub(super) fn excess(&self, limits: &Limits, authority: &jig_domain_authority::Limits) -> Option<(u64, u64)> {
        match self {
            Self::Delegated(rows) => excess(rows.len(), limits.tasks.batch),
            Self::EffectDenied { findings, .. }
            | Self::ToolDenied { findings, .. }
            | Self::DelegationDenied { findings, .. } => excess(
                findings.len(),
                jig_domain_authority::stored_finding_limit(authority).expect("configured finding bound"),
            ),
            Self::NoteRecalled { entries, .. } => {
                if let Some(failure) = excess(entries.len(), limits.notes.recalled) {
                    return Some(failure);
                }
                for entry in entries {
                    if let Some(failure) = note_excess(entry, &limits.notes) {
                        return Some(failure);
                    }
                }
                None
            }
            _ => None,
        }
    }
}

fn excess(held: usize, limit: u32) -> Option<(u64, u64)> {
    let held = u64::try_from(held).expect("allocated collection fits u64");
    let limit = u64::from(limit);
    if held > limit { Some((held, limit)) } else { None }
}

fn note_excess(entry: &notes::Entry, limits: &notes::Limits) -> Option<(u64, u64)> {
    for (held, limit) in [
        (entry.description.len(), limits.description_bytes),
        (entry.body.len(), limits.body_bytes),
        (usize::try_from(entry.references.capacity()).expect("reference bound fits usize"), limits.references),
    ] {
        if let Some(failure) = excess(held, limit) {
            return Some(failure);
        }
    }
    match &entry.scope {
        notes::Scope::Resources { pattern, .. } => {
            if let Some(failure) = excess(pattern.segments.len(), limits.pattern_bytes) {
                return Some(failure);
            }
            let mut bytes = match &pattern.last {
                notes::Last::Exact(last) | notes::Last::Open(last) => last.len(),
            };
            for segment in &pattern.segments {
                bytes = bytes.saturating_add(segment.len());
            }
            excess(bytes, limits.pattern_bytes)
        }
        notes::Scope::Deployment | notes::Scope::Project { .. } | notes::Scope::Goal { .. } => None,
    }
}
