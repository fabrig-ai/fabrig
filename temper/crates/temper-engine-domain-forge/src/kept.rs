//! Durable forge and playground rows (temper/docs/design/draft/playground-reset.md, 3, 4 and 6.2).
use crate::{Domain, ForgeRecord, KeptAct, Key, Limits, Playground, Request, SetUpProject, Stored};
use alloc::boxed::Box;
use skein_lib::{List, Queue};

/// A retained row cannot fit its configured table or byte bounds.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Full;

impl Domain {
    #[must_use]
    pub fn forge_record(&self, forge: u16) -> Option<&ForgeRecord> {
        self.forges.get(&forge)
    }

    /// A snapshot in forge-number order.
    #[must_use]
    pub fn forge_records(&self) -> Box<[ForgeRecord]> {
        let mut rows = List::with_capacity(self.forges.len());
        for (_, row) in &self.forges {
            rows.push(row.clone()).expect("snapshot fits retained rows");
        }
        rows.into_boxed()
    }

    #[must_use]
    pub fn forge_next(&self) -> u16 {
        self.forge_next.unwrap_or(1)
    }

    #[must_use]
    pub const fn playground(&self) -> Option<&Playground> {
        self.playground.as_ref()
    }

    /// A snapshot in act-key order.
    #[must_use]
    pub fn kept_acts(&self) -> Box<[KeptAct]> {
        let mut rows = List::with_capacity(self.kept_acts.len());
        for (_, row) in &self.kept_acts {
            rows.push(*row).expect("snapshot fits retained rows");
        }
        rows.into_boxed()
    }
}

#[cfg_attr(not(test), expect(dead_code, reason = "forge-records saves durable forge definitions"))]
pub(crate) fn save_forge(d: &mut Domain, l: &Limits, row: ForgeRecord, out: &mut Queue<Request>) -> Result<(), Full> {
    if row.number == 0
        || row.number == u16::MAX
        || row.definition.len() > usize::try_from(l.definition_bytes).expect("u32 fits usize")
    {
        return Err(Full);
    }
    if let Some(first) = &row.first {
        let limit = usize::try_from(crate::FIRST_NAME_BYTES).expect("u32 fits usize");
        for name in [&first.org, &first.repository, &first.label] {
            if name.len() > limit {
                return Err(Full);
            }
        }
        match &first.project {
            SetUpProject::New(name) if name.len() > limit => return Err(Full),
            SetUpProject::New(_) | SetUpProject::Existing(_) => {}
        }
    }
    d.forges.insert(row.number, row.clone()).or(Err(Full))?;
    out.push(Request::Save { read_afresh: None, release: None, record: Stored::Forge(row) });
    Ok(())
}

#[cfg_attr(not(test), expect(dead_code, reason = "forge-remove erases a forge after its kept acts settle"))]
pub(crate) fn erase_forge(d: &mut Domain, forge: u16, out: &mut Queue<Request>) {
    if d.forges.remove(&forge).is_some() {
        out.push(Request::Erase { key: Key::Forge(forge) });
    }
}

#[cfg_attr(not(test), expect(dead_code, reason = "forge-records assigns monotone forge numbers"))]
pub(crate) fn give_number(d: &mut Domain, out: &mut Queue<Request>) -> Option<u16> {
    let number = d.forge_next();
    let next = number.checked_add(1)?;
    d.forge_next = Some(next);
    out.push(Request::Save { read_afresh: None, release: None, record: Stored::ForgeNumbers { next } });
    Some(number)
}

#[cfg_attr(not(test), expect(dead_code, reason = "playground saves its durable reset progress"))]
pub(crate) fn save_playground(d: &mut Domain, row: Playground, out: &mut Queue<Request>) {
    d.playground = Some(row);
    out.push(Request::Save { read_afresh: None, release: None, record: Stored::Playground(row) });
}

#[cfg_attr(not(test), expect(dead_code, reason = "playground erases its completed reset"))]
pub(crate) fn erase_playground(d: &mut Domain, out: &mut Queue<Request>) {
    if d.playground.take().is_some() {
        out.push(Request::Erase { key: Key::Playground });
    }
}

#[cfg_attr(not(test), expect(dead_code, reason = "forge-remove keeps acts until their saved answers"))]
pub(crate) fn keep_act(d: &mut Domain, row: KeptAct, out: &mut Queue<Request>) -> Result<(), Full> {
    d.kept_acts.insert(row.key, row).or(Err(Full))?;
    out.push(Request::Save { read_afresh: None, release: None, record: Stored::KeptAct(row) });
    Ok(())
}

#[cfg_attr(not(test), expect(dead_code, reason = "forge-remove erases acts on their saved answers"))]
pub(crate) fn erase_act(d: &mut Domain, key: [u8; 16], out: &mut Queue<Request>) {
    if d.kept_acts.remove(&key).is_some() {
        out.push(Request::Erase { key: Key::KeptAct(key) });
    }
}
