//! Notes rows and their bounded resource patterns (protocol/store/records.md, 1).
use crate::notes_v1::{self as wire, Record};
use alloc::boxed::Box;
use jig_domain_notes as domain;
use skein_lib::{List, Reader, Wall};

/// Translate the notes child's bounds, refusing a schema ceiling.
#[must_use]
pub fn limits(domain: &domain::Limits) -> Option<wire::Limits> {
    let limits = wire::Limits {
        segment_bytes: domain.pattern_bytes,
        pattern_segments: domain.pattern_bytes,
        entry_description: domain.description_bytes,
        line_description: domain.description_bytes,
        entry_body: domain.body_bytes,
        entry_references: domain.references,
    };
    wire::check_limits(&limits).ok()?;
    Some(limits)
}

/// Encode one versioned notes row within the configured bounds.
pub fn encode(row: &domain::Record, limits: &wire::Limits) -> Result<Box<[u8]>, wire::Problem> {
    let value = match row {
        domain::Record::Entry(domain::Entry {
            name,
            scope,
            description,
            body,
            references,
            author,
            revision,
            changed,
        }) => {
            let author = match author {
                domain::Author::Party { party } => wire::Author::Party(wire::Party { party: *party }),
                domain::Author::Task { task, attempt } => {
                    wire::Author::Task(wire::Task { task: *task, attempt: *attempt })
                }
            };
            let mut translated_references = List::with_capacity(references.capacity());
            for task in references {
                translated_references.push(wire::Reference { task: *task }).expect("same number of references");
            }
            wire::Value::Entry(wire::Entry {
                name: *name,
                scope: encode_scope(scope, limits)?,
                description: description.clone(),
                body: body.clone(),
                references: translated_references,
                author,
                revision: *revision,
                changed: changed.as_nanos(),
            })
        }
        domain::Record::Line(domain::Line { scope, name, description, revision }) => wire::Value::Line(wire::Line {
            scope: encode_scope(scope, limits)?,
            name: *name,
            description: description.clone(),
            revision: *revision,
        }),
    };
    let row = wire::Row { value };
    row.encode(limits)
}

/// Decode one notes row, refusing another version or trailing bytes.
pub fn decode(bytes: &[u8], limits: &wire::Limits) -> Result<domain::Record, wire::Problem> {
    let wire::Row { value } = wire::Row::decode(limits, &mut Reader::new(bytes))?;
    Ok(match value {
        wire::Value::Entry(entry) => {
            let wire::Entry { name, scope, description, body, references, author, revision, changed } = entry;
            let author = match author {
                wire::Author::Party(party) => {
                    let wire::Party { party } = party;
                    domain::Author::Party { party }
                }
                wire::Author::Task(task) => {
                    let wire::Task { task, attempt } = task;
                    domain::Author::Task { task, attempt }
                }
            };
            let mut translated_references = List::with_capacity(references.len());
            for reference in references.into_boxed() {
                let wire::Reference { task } = reference;
                translated_references.push(task).expect("same number of references");
            }
            domain::Record::Entry(domain::Entry {
                name,
                scope: decode_scope(scope, limits)?,
                description,
                body,
                references: translated_references,
                author,
                revision,
                changed: Wall::from_nanos(changed),
            })
        }
        wire::Value::Line(line) => {
            let wire::Line { scope, name, description, revision } = line;
            domain::Record::Line(domain::Line { scope: decode_scope(scope, limits)?, name, description, revision })
        }
    })
}

fn encode_scope(scope: &domain::Scope, limits: &wire::Limits) -> Result<wire::Scope, wire::Problem> {
    Ok(match scope {
        domain::Scope::Party { party } => wire::Scope::Party(wire::Party { party: *party }),
        domain::Scope::Deployment => wire::Scope::Deployment,
        domain::Scope::Project { project } => wire::Scope::Project(wire::Project { project: *project }),
        domain::Scope::Goal { project, goal } => wire::Scope::Goal(wire::Goal { project: *project, goal: *goal }),
        domain::Scope::Resources { project, connector, pattern } => {
            let domain::Pattern { segments, last } = pattern;
            let mut bytes = match last {
                domain::Last::Exact(bytes) | domain::Last::Open(bytes) => bytes.len(),
            };
            let mut members = List::with_capacity(limits.pattern_segments);
            for segment in segments {
                bytes = bytes.saturating_add(segment.len());
                let segment = wire::Segment { bytes: segment.clone() };
                if members.push(segment).is_err() {
                    return Err(pattern_problem());
                }
            }
            if bytes > skein_lib::length::usize_of(limits.segment_bytes) {
                return Err(pattern_problem());
            }
            let last = match last {
                domain::Last::Exact(bytes) => wire::Last::Exact(wire::Segment { bytes: bytes.clone() }),
                domain::Last::Open(bytes) => wire::Last::Open(wire::Segment { bytes: bytes.clone() }),
            };
            let pattern = wire::Pattern { segments: members, last };
            wire::Scope::Resources(wire::Resources { project: *project, connector: *connector, pattern })
        }
    })
}

fn decode_scope(scope: wire::Scope, limits: &wire::Limits) -> Result<domain::Scope, wire::Problem> {
    Ok(match scope {
        wire::Scope::Party(value) => domain::Scope::Party { party: value.party },
        wire::Scope::Deployment => domain::Scope::Deployment,
        wire::Scope::Project(project) => {
            let wire::Project { project } = project;
            domain::Scope::Project { project }
        }
        wire::Scope::Goal(goal) => {
            let wire::Goal { project, goal } = goal;
            domain::Scope::Goal { project, goal }
        }
        wire::Scope::Resources(resources) => {
            let wire::Resources { project, connector, pattern } = resources;
            let wire::Pattern { segments, last } = pattern;
            let last = match last {
                wire::Last::Exact(segment) => domain::Last::Exact(segment.bytes),
                wire::Last::Open(segment) => domain::Last::Open(segment.bytes),
            };
            let mut bytes = match &last {
                domain::Last::Exact(bytes) | domain::Last::Open(bytes) => bytes.len(),
            };
            let mut members = List::with_capacity(segments.len());
            for segment in segments.into_boxed() {
                let wire::Segment { bytes: segment } = segment;
                bytes = bytes.saturating_add(segment.len());
                members.push(segment).expect("same number of segments");
            }
            if bytes > skein_lib::length::usize_of(limits.segment_bytes) {
                return Err(pattern_problem());
            }
            domain::Scope::Resources {
                project,
                connector,
                pattern: domain::Pattern { segments: members.into_boxed(), last },
            }
        }
    })
}

fn pattern_problem() -> wire::Problem {
    wire::Problem { path: wire::paths::PATTERN_SEGMENTS, reason: skein_codec::Reason::Bound }
}
