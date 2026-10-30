//! Raw retained main-map pages (jig/docs/design/protocol/store.md, 7.1, 9 and 14).
use crate::{Cursor, Store, pages, restore};
use alloc::boxed::Box;
use restore::{Arithmetic, Cause, Failure};
use skein_lib::List;

#[derive(Debug)]
pub struct Page {
    pub rows: Box<[skein_kv::Row]>,
    pub next: Option<Box<[u8]>>,
}

impl Store {
    pub fn retained_page(&mut self, after: Option<&[u8]>, rows: u32, bytes: u64) -> Result<Page, Failure> {
        self.consumed = true;
        restore::ready(self)?;
        if let Some(after) = after
            && after.len() > usize::try_from(self.limits.store.key).expect("map key allowance fits usize")
        {
            return Err(Failure::rowless(Cause::Range(restore::RangeFailure::Cursor)));
        }
        #[expect(
            clippy::manual_map,
            reason = "step code translates optional cursors without closures or function pointers"
        )]
        let cursor = match after {
            Some(after) => Some(Cursor { interval: 0, after: Box::from(after) }),
            None => None,
        };
        let intervals = [skein_kv::Range { start: Box::new([]), end: None }];
        let rows = rows.min(self.limits.store.page.rows);
        let walked = pages::reader::walk(self, &intervals, cursor.as_ref(), rows)?;
        let mut found = List::with_capacity(rows.min(self.limits.store.page.bytes));
        let mut used = 0_u64;
        let mut next: Option<Box<[u8]>> = match &cursor {
            Some(cursor) => Some(cursor.after.clone()),
            None => Some(Box::new([])),
        };
        let mut cut = false;
        for item in walked.rows {
            let size = pages::reader::row_bytes(self, &item.row)?;
            let total = used
                .checked_add(size)
                .ok_or(Failure::at(Cause::Arithmetic(Arithmetic::PageBytes), restore::site(self, &item.row.key)))?;
            if total > bytes {
                cut = true;
                break;
            }
            used = total;
            next = Some(item.row.key.clone());
            found.push(item.row).expect("walk bounded raw rows");
        }
        if !cut {
            next = match walked.next {
                Some(cursor) => Some(cursor.after),
                None => None,
            };
        }
        let rows = found.into_boxed();
        if rows.is_empty()
            && let Some(next) = &next
        {
            let initial = after.unwrap_or(&[]);
            assert!(next.as_ref() > initial, "empty nonterminal retained page must advance its cursor");
        }
        Ok(Page { rows, next })
    }
}
