//! Whole selected page rows and their owner continuations (protocol/client/wire.md, 1, and protocol/client/README.md, 4).
use super::{cursor, w};
use alloc::boxed::Box;
use jig_client_protocol_wire::Record as _;
use jig_domain_core_vocabulary::reads;
mod documents;
mod rows;

pub fn page(
    models: &[crate::Model],
    read: &reads::Read,
    page: &reads::Page,
    private: &crate::Private,
    limits: &w::Limits,
) -> Result<w::ReadValue, w::Problem> {
    let invalid = w::Problem { path: w::paths::READ_VALUE_TAG, reason: skein_codec::Reason::Tag };
    check_edges(read, page)?;
    let before = match &page.before {
        Some(value) => Some(wire_cursor(value, limits)?),
        None => None,
    };
    let after = match &page.next {
        Some(value) => Some(wire_cursor(value, limits)?),
        None => None,
    };
    Ok(match read {
        reads::Read::Activity { task, .. } => documents::activity(*task, page, before, after, private, limits)?,
        reads::Read::ActivityBeside { task, side, .. } => match side {
            reads::Side::Earlier => documents::activity(*task, page, None, after, private, limits)?,
            reads::Side::Later => documents::activity(*task, page, after, None, private, limits)?,
        },
        reads::Read::History { task, subtree, .. } => {
            documents::history(models, *task, *subtree, page, before, after, limits)?
        }
        reads::Read::Inbox { .. } => documents::inbox(page, before, after, limits)?,
        reads::Read::EndedDelegates { .. } => documents::ended(page, before, after, limits)?,
        reads::Read::Spend { project, .. } => documents::spend(*project, page, before, after, limits)?,
        reads::Read::Notes { .. } | reads::Read::ScopedNotes { .. } => {
            documents::notes(models, page, before, after, limits)?
        }
        reads::Read::Tasks { .. }
        | reads::Read::Application { .. }
        | reads::Read::Frame
        | reads::Read::TaskText { .. }
        | reads::Read::Task { .. }
        | reads::Read::Proposal { .. }
        | reads::Read::Escalation { .. }
        | reads::Read::Question { .. }
        | reads::Read::PersonTask { .. }
        | reads::Read::Result { .. }
        | reads::Read::Reply { .. }
        | reads::Read::Turn { .. }
        | reads::Read::Project { .. }
        | reads::Read::Settings
        | reads::Read::Setting { .. }
        | reads::Read::Revision { .. }
        | reads::Read::Revisions { .. }
        | reads::Read::Deployment => return Err(invalid),
    })
}

fn check_edges(read: &reads::Read, page: &reads::Page) -> Result<(), w::Problem> {
    let invalid = w::Problem { path: w::paths::CURSOR_VALUE, reason: skein_codec::Reason::Tag };
    for cursor in [&page.before, &page.next] {
        let Some(cursor) = cursor else { continue };
        let matches = match cursor {
            reads::Cursor::Live { scope, .. } | reads::Cursor::Store { scope, .. } => {
                let expected = match read {
                    reads::Read::Activity { task, .. } | reads::Read::ActivityBeside { task, .. } => {
                        reads::Scope::Activity { task: *task }
                    }
                    reads::Read::History { task, subtree, .. } => {
                        reads::Scope::History { task: *task, subtree: *subtree }
                    }
                    reads::Read::EndedDelegates { task, .. } => reads::Scope::EndedDelegates { task: *task },
                    reads::Read::Inbox { group, filter, .. } => reads::Scope::Inbox { group: *group, filter: *filter },
                    reads::Read::Tasks { query, .. } => reads::Scope::Tasks { query: query.clone() },
                    reads::Read::Spend { project, .. } => reads::Scope::Spend { project: *project },
                    reads::Read::Notes { project, .. } => reads::Scope::Notes { project: *project },
                    reads::Read::ScopedNotes { scope, description, .. } => {
                        reads::Scope::ScopedNotes { scope: scope.clone(), description: description.clone() }
                    }
                    reads::Read::Application { .. }
                    | reads::Read::Frame
                    | reads::Read::TaskText { .. }
                    | reads::Read::Task { .. }
                    | reads::Read::Proposal { .. }
                    | reads::Read::Escalation { .. }
                    | reads::Read::Question { .. }
                    | reads::Read::PersonTask { .. }
                    | reads::Read::Result { .. }
                    | reads::Read::Reply { .. }
                    | reads::Read::Turn { .. }
                    | reads::Read::Project { .. }
                    | reads::Read::Settings
                    | reads::Read::Setting { .. }
                    | reads::Read::Revision { .. }
                    | reads::Read::Revisions { .. }
                    | reads::Read::Deployment => return Err(invalid),
                };
                *scope == expected
            }
        };
        if !matches {
            return Err(invalid);
        }
    }
    Ok(())
}

fn wire_cursor(value: &reads::Cursor, limits: &w::Limits) -> Result<w::Cursor, w::Problem> {
    let bytes = cursor::encode(value, limits.cursor_value.min(w::CEILINGS.cursor_value))
        .ok_or(w::Problem { path: w::paths::CURSOR_VALUE, reason: skein_codec::Reason::Bound })?;
    Ok(w::Cursor { value: bytes })
}

pub(super) fn tasks(
    page: &reads::TaskPage,
    query: &jig_domain_core_vocabulary::documents::TaskQuery,
    slots: &skein_lib::List<super::parts::Slot>,
    limits: &w::Limits,
) -> Result<w::ReadValue, w::Problem> {
    let ended = match &page.ended {
        reads::EndedGroup::None => w::EndedGroup::None,
        reads::EndedGroup::Rows { rows, next, cut } => w::EndedGroup::Rows(w::EndedRows {
            rows: task_pieces(rows, slots, limits, limits.ended_rows_rows, w::paths::ENDED_ROWS_ROWS)?,
            after: match next {
                Some(v) => Some(task_cursor(v, query, limits)?),
                None => None,
            },
            cut: *cut,
        }),
    };
    Ok(w::ReadValue::Tasks(w::TaskPage {
        ended_mark: page.ended_mark,
        live: task_pieces(&page.live, slots, limits, limits.task_page_live, w::paths::TASK_PAGE_LIVE)?,
        live_after: match &page.live_next {
            Some(v) => Some(task_cursor(v, query, limits)?),
            None => None,
        },
        ended,
    }))
}
fn task_pieces(
    lines: &skein_lib::List<jig_domain_core_vocabulary::documents::TaskLine>,
    slots: &skein_lib::List<super::parts::Slot>,
    limits: &w::Limits,
    bound: u32,
    path: w::Path,
) -> Result<Box<[u8]>, w::Problem> {
    let mut pieces = skein_lib::List::with_capacity(lines.len());
    for line in lines {
        let line = super::queries::line_parts(line, slots, limits)?;
        let mut writer = skein_lib::Writer::new(skein_lib::length::usize_of(line.measure(limits)?));
        line.write(limits, &mut writer).expect("measured line");
        pieces.push(writer.finish()).expect("same line count");
    }
    super::pieces(&pieces, bound, path)
}

fn task_cursor(
    value: &reads::Cursor,
    query: &jig_domain_core_vocabulary::documents::TaskQuery,
    limits: &w::Limits,
) -> Result<w::Cursor, w::Problem> {
    let mut value = value.clone();
    let (reads::Cursor::Live { scope, .. } | reads::Cursor::Store { scope, .. }) = &mut value;
    if let reads::Scope::Tasks { query: held } = scope {
        *held = query.clone();
    } else {
        return Err(super::problem(skein_codec::Reason::Tag));
    }
    wire_cursor(&value, limits)
}
