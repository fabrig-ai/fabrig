//! Held forge and playground rows (temper/docs/design/draft/playground-reset.md, 3, 4 and 6.2).
use crate::{Domain, FIRST_NAME_BYTES, ForgeRecord, KeptAct, Key, Limits, Playground, Request, SetUpProject, Stored};
use alloc::boxed::Box;
use skein_lib::{List, Queue};

/// A held row cannot fit its count or byte bound.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Full;

impl Domain {
    #[must_use]
    pub fn forge_record(&self, forge: u16) -> Option<&ForgeRecord> {
        self.forges.get(&forge)
    }
    /// Forge records in number order.
    #[must_use]
    pub fn forge_records(&self) -> Box<[&ForgeRecord]> {
        let mut rows = List::with_capacity(self.forges.len());
        for (_, row) in &self.forges {
            rows.push(row).expect("held forge count");
        }
        rows.into_boxed()
    }
    #[must_use]
    pub fn forge_next(&self) -> u16 {
        self.forge_next.unwrap_or(1)
    }
    #[must_use]
    pub fn playground(&self) -> Option<&Playground> {
        self.playground.as_ref()
    }
    /// Kept acts in key order.
    #[must_use]
    pub fn kept_acts(&self) -> Box<[&KeptAct]> {
        let mut rows = List::with_capacity(self.kept_acts.len());
        for (_, row) in &self.kept_acts {
            rows.push(row).expect("held act count");
        }
        rows.into_boxed()
    }
}

pub(crate) fn first_fits(row: &ForgeRecord) -> bool {
    let Some(first) = &row.first else { return true };
    let limit = usize::try_from(FIRST_NAME_BYTES).expect("name bound fits usize");
    if first.org.len() > limit || first.repository.len() > limit || first.label.len() > limit {
        return false;
    }
    match &first.project {
        SetUpProject::Existing(_) => true,
        SetUpProject::New(name) => name.len() <= limit,
    }
}

#[cfg_attr(not(test), expect(dead_code, reason = "forge-records calls this request maker"))]
pub(crate) fn save_forge(d: &mut Domain, l: &Limits, row: ForgeRecord, out: &mut Queue<Request>) -> Result<(), Full> {
    if row.number == 0
        || row.number == u16::MAX
        || row.definition.len() > usize::try_from(l.definition_bytes).expect("definition bound fits usize")
        || !first_fits(&row)
    {
        return Err(Full);
    }
    d.forges.insert(row.number, row.clone()).or(Err(Full))?;
    save(out, Stored::Forge(row));
    Ok(())
}
#[cfg_attr(not(test), expect(dead_code, reason = "forge-remove calls this request maker"))]
pub(crate) fn erase_forge(d: &mut Domain, forge: u16, out: &mut Queue<Request>) {
    d.forges.remove(&forge);
    erase(out, Key::Forge(forge));
}
#[cfg_attr(not(test), expect(dead_code, reason = "forge-records calls this request maker"))]
pub(crate) fn give_number(d: &mut Domain, out: &mut Queue<Request>) -> Option<u16> {
    let number = d.forge_next();
    if number == u16::MAX {
        return None;
    }
    let next = number.checked_add(1).expect("exhausted numbers are refused");
    d.forge_next = Some(next);
    save(out, Stored::ForgeNumbers { next });
    Some(number)
}
#[cfg_attr(not(test), expect(dead_code, reason = "playground calls this request maker"))]
pub(crate) fn save_playground(d: &mut Domain, row: Playground, out: &mut Queue<Request>) {
    d.playground = Some(row);
    save(out, Stored::Playground(row));
}
#[cfg_attr(not(test), expect(dead_code, reason = "playground calls this request maker"))]
pub(crate) fn erase_playground(d: &mut Domain, out: &mut Queue<Request>) {
    d.playground = None;
    erase(out, Key::Playground);
}
#[cfg_attr(not(test), expect(dead_code, reason = "forge-remove calls this request maker"))]
pub(crate) fn keep_act(d: &mut Domain, row: KeptAct, out: &mut Queue<Request>) -> Result<(), Full> {
    d.kept_acts.insert(row.key, row).or(Err(Full))?;
    save(out, Stored::KeptAct(row));
    Ok(())
}
#[cfg_attr(not(test), expect(dead_code, reason = "forge-remove calls this request maker"))]
pub(crate) fn erase_act(d: &mut Domain, key: [u8; 16], out: &mut Queue<Request>) {
    d.kept_acts.remove(&key);
    erase(out, Key::KeptAct(key));
}
fn save(out: &mut Queue<Request>, record: Stored) {
    out.push(Request::Save { read_afresh: None, release: None, record });
}
fn erase(out: &mut Queue<Request>, key: Key) {
    out.push(Request::Erase { key });
}
