//! Each original page's whole owner rows (protocol/client/wire.md, 1).
use super::super::{activity, chips, history, inbox, objects, w};
use super::rows;
use alloc::boxed::Box;
use jig_domain_core_vocabulary::reads;
use skein_lib::List;
pub(super) fn activity(
    task: u64,
    page: &reads::Page,
    before: Option<w::Cursor>,
    after: Option<w::Cursor>,
    private: &crate::Private,
    limits: &w::Limits,
) -> Result<w::ReadValue, w::Problem> {
    let invalid = w::Problem { path: w::paths::READ_VALUE_TAG, reason: skein_codec::Reason::Tag };

    let mut rows = List::with_capacity(page.rows.len());
    for row in &page.rows {
        let (line, turn) = match row {
            reads::PageRow::Activity { line, turn } => (line, turn),
            reads::PageRow::EndedDelegate(_)
            | reads::PageRow::History(_)
            | reads::PageRow::Attempt(_)
            | reads::PageRow::Hold(_)
            | reads::PageRow::Inbox(_)
            | reads::PageRow::Line(_)
            | reads::PageRow::Note(_)
            | reads::PageRow::Spend(_) => return Err(invalid),
        };
        rows.push(activity::page_turn(task, line, turn.as_ref(), private, limits)?).expect("selected rows fit");
    }
    Ok(w::ReadValue::Activity(w::ActivityPage {
        cut: page.cut,
        rows: rows::activity_pieces(
            &rows,
            limits.activity_page_rows.min(w::CEILINGS.activity_page_rows),
            w::paths::ACTIVITY_PAGE_ROWS,
            limits,
        )?,
        before: after,
        after: before,
    }))
}

pub(super) fn history(
    models: &[crate::Model],
    task: u64,
    subtree: bool,
    page: &reads::Page,
    before: Option<w::Cursor>,
    after: Option<w::Cursor>,
    limits: &w::Limits,
) -> Result<w::ReadValue, w::Problem> {
    let invalid = w::Problem { path: w::paths::READ_VALUE_TAG, reason: skein_codec::Reason::Tag };

    let mut rows = List::with_capacity(page.rows.len());
    for row in &page.rows {
        rows.push(match row {
            reads::PageRow::Attempt(value) if subtree || value.task == task => {
                w::HistoryRow::Attempt(w::HistoryRowAttempt { value: objects::ended::attempt(value) })
            }
            reads::PageRow::History(value) if subtree || value.task == task => {
                w::HistoryRow::Change(Box::new(w::HistoryRowChange { value: history::line(models, value, limits)? }))
            }
            reads::PageRow::Hold(value) if subtree || value.task == task => {
                w::HistoryRow::Hold(w::HistoryRowHold { value: objects::ended::hold(value, limits)? })
            }
            reads::PageRow::EndedDelegate(_)
            | reads::PageRow::History(_)
            | reads::PageRow::Attempt(_)
            | reads::PageRow::Hold(_)
            | reads::PageRow::Activity { .. }
            | reads::PageRow::Inbox(_)
            | reads::PageRow::Line(_)
            | reads::PageRow::Note(_)
            | reads::PageRow::Spend(_) => return Err(invalid),
        })
        .expect("selected rows fit");
    }
    Ok(w::ReadValue::History(w::HistoryPage {
        cut: page.cut,
        rows: rows::history_pieces(
            &rows,
            limits.history_page_rows.min(w::CEILINGS.history_page_rows),
            w::paths::HISTORY_PAGE_ROWS,
            limits,
        )?,
        before,
        after,
    }))
}

pub(super) fn inbox(
    page: &reads::Page,
    before: Option<w::Cursor>,
    after: Option<w::Cursor>,
    limits: &w::Limits,
) -> Result<w::ReadValue, w::Problem> {
    let invalid = w::Problem { path: w::paths::READ_VALUE_TAG, reason: skein_codec::Reason::Tag };

    let mut rows = List::with_capacity(page.rows.len());
    for row in &page.rows {
        let value = match row {
            reads::PageRow::Inbox(value) => value,
            reads::PageRow::EndedDelegate(_)
            | reads::PageRow::History(_)
            | reads::PageRow::Attempt(_)
            | reads::PageRow::Hold(_)
            | reads::PageRow::Activity { .. }
            | reads::PageRow::Line(_)
            | reads::PageRow::Note(_)
            | reads::PageRow::Spend(_) => return Err(invalid),
        };
        rows.push(inbox::entry(value, limits)?).expect("selected rows fit");
    }
    Ok(w::ReadValue::Inbox(w::InboxPage {
        cut: page.cut,
        rows: rows::inbox_pieces(
            &rows,
            limits.inbox_page_rows.min(w::CEILINGS.inbox_page_rows),
            w::paths::INBOX_PAGE_ROWS,
            limits,
        )?,
        before,
        after,
        selection: inbox::selection(
            page.inbox_selection.as_ref(),
            limits,
            limits.inbox_page_selection_some,
            w::paths::INBOX_PAGE_SELECTION,
        )?,
    }))
}

pub(super) fn ended(
    page: &reads::Page,
    before: Option<w::Cursor>,
    after: Option<w::Cursor>,
    limits: &w::Limits,
) -> Result<w::ReadValue, w::Problem> {
    let invalid = w::Problem { path: w::paths::READ_VALUE_TAG, reason: skein_codec::Reason::Tag };

    let mut rows = List::with_capacity(page.rows.len());
    for row in &page.rows {
        let value = match row {
            reads::PageRow::EndedDelegate(value) => value,
            reads::PageRow::History(_)
            | reads::PageRow::Attempt(_)
            | reads::PageRow::Hold(_)
            | reads::PageRow::Activity { .. }
            | reads::PageRow::Inbox(_)
            | reads::PageRow::Line(_)
            | reads::PageRow::Note(_)
            | reads::PageRow::Spend(_) => return Err(invalid),
        };
        rows.push(w::EndedDelegate {
            chip: chips::chip(&value.chip, limits)?,
            ended: chips::ending(&value.ended, limits)?,
            at: match value.at {
                Some(value) => Some(value.as_nanos()),
                None => None,
            },
        })
        .expect("selected rows fit");
    }
    Ok(w::ReadValue::EndedPage(w::EndedPage {
        cut: page.cut,
        rows: rows::ended_pieces(
            &rows,
            limits.ended_page_rows.min(w::CEILINGS.ended_page_rows),
            w::paths::ENDED_PAGE_ROWS,
            limits,
        )?,
        before,
        after,
    }))
}

pub(super) fn spend(
    project: u32,
    page: &reads::Page,
    before: Option<w::Cursor>,
    after: Option<w::Cursor>,
    limits: &w::Limits,
) -> Result<w::ReadValue, w::Problem> {
    let invalid = w::Problem { path: w::paths::READ_VALUE_TAG, reason: skein_codec::Reason::Tag };

    if page.rows.capacity() > limits.spend_page_rows.min(w::CEILINGS.spend_page_rows) {
        return Err(w::Problem { path: w::paths::SPEND_PAGE_ROWS, reason: skein_codec::Reason::Bound });
    }
    let mut rows = List::with_capacity(page.rows.capacity());
    for row in &page.rows {
        let value = match row {
            reads::PageRow::Spend(value) => value,
            reads::PageRow::EndedDelegate(_)
            | reads::PageRow::History(_)
            | reads::PageRow::Attempt(_)
            | reads::PageRow::Hold(_)
            | reads::PageRow::Activity { .. }
            | reads::PageRow::Inbox(_)
            | reads::PageRow::Line(_)
            | reads::PageRow::Note(_) => return Err(invalid),
        };
        let same_project = match value.funder {
            jig_domain_tasks::Funder::Period { project: owner, .. } => owner == project,
            jig_domain_tasks::Funder::Task(_)
            | jig_domain_tasks::Funder::Pool { .. }
            | jig_domain_tasks::Funder::Recurring { .. } => false,
        };
        if !same_project {
            return Err(invalid);
        }
        rows.push(rows::funding(value)).expect("selected rows fit");
    }
    Ok(w::ReadValue::Spend(w::SpendPage { cut: page.cut, rows, before, after }))
}

pub(super) fn notes(
    models: &[crate::Model],
    page: &reads::Page,
    before: Option<w::Cursor>,
    after: Option<w::Cursor>,
    limits: &w::Limits,
) -> Result<w::ReadValue, w::Problem> {
    let invalid = w::Problem { path: w::paths::READ_VALUE_TAG, reason: skein_codec::Reason::Tag };

    let mut rows = List::with_capacity(page.rows.len());
    for row in &page.rows {
        let value = match row {
            reads::PageRow::Note(value) => value,
            reads::PageRow::EndedDelegate(_)
            | reads::PageRow::History(_)
            | reads::PageRow::Attempt(_)
            | reads::PageRow::Hold(_)
            | reads::PageRow::Activity { .. }
            | reads::PageRow::Inbox(_)
            | reads::PageRow::Line(_)
            | reads::PageRow::Spend(_) => return Err(invalid),
        };
        rows.push(rows::note(models, value, limits)?).expect("selected rows fit");
    }
    Ok(w::ReadValue::Notes(w::NotePage {
        cut: page.cut,
        rows: rows::note_pieces(
            &rows,
            limits.note_page_rows.min(w::CEILINGS.note_page_rows),
            w::paths::NOTE_PAGE_ROWS,
            limits,
        )?,
        before,
        after,
    }))
}
