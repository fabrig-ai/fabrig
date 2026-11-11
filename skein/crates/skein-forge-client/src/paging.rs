//! Inclusive item pages and byte-cut tie recovery (forge/client.md, 4, "Paging").
use skein_forge::api::{Kind, Read, Summary};
use skein_lib::Time;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(crate) struct Items {
    pub(crate) since: Time,
    pub(crate) page: u32,
    single: bool,
    /// Rows consumed at this inclusive time, including full pages before a cut.
    consumed: u32,
}
impl Items {
    pub(crate) fn first(since: Time) -> Self {
        Self { since, page: 1, single: false, consumed: 0 }
    }
    pub(crate) fn read(self, kind: Option<Kind>) -> Read {
        Read::Items { since: self.since, page: self.page, kind, single: self.single }
    }
    /// Returns whether the page advanced time, or None when its position exhausts.
    pub(crate) fn next(&mut self, items: &[Summary], cut: bool, now: Time) -> Option<bool> {
        self.since = self.since.min(now);
        match items.last() {
            Some(last) if last.updated.min(now) > self.since => {
                self.since = last.updated.min(now);
                self.page = 1;
                self.single = false;
                self.consumed = 0;
                Some(true)
            }
            Some(_) | None => {
                self.consumed = self.consumed.checked_add(u32::try_from(items.len()).ok()?)?;
                self.single |= cut;
                self.page = if self.single { self.consumed.checked_add(1)? } else { self.page.checked_add(1)? };
                Some(false)
            }
        }
    }
}
