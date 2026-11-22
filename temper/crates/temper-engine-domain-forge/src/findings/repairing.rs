//! Repair admission owns the forge's checks, without making a task (domain/red-repairs.md, 4.2).
use super::{Domain, FindingRefusal, FindingState, FindingWrite, RepairAct};
use alloc::boxed::Box;
use skein_lib::{Decimal, List, Map};

impl Domain {
    /// Check all findings before returning any writes, oldest first.
    #[expect(
        clippy::too_many_lines,
        reason = "one ordered repair admission checks every finding before returning writes"
    )]
    pub fn prepare_repair_findings(
        &self,
        project: u32,
        number: u32,
        act: &RepairAct,
    ) -> Result<Box<[FindingWrite]>, FindingRefusal> {
        let repairs = match self.project_repairs(project) {
            Ok(repairs) => repairs,
            Err(why) => return Err(FindingRefusal::Words(why)),
        };
        let mut numbers = List::with_capacity(16);
        numbers.push(number).expect("first finding");
        for &number in &act.more {
            if numbers.as_slice().contains(&number) || numbers.push(number).is_err() {
                return Err(FindingRefusal::Words(b"findings".as_slice().into()));
            }
        }
        if !crate::settings::words_fit(&act.words, 4096) {
            return Err(FindingRefusal::Words(b"words".as_slice().into()));
        }
        let mut rows = List::with_capacity(16);
        for &number in &numbers {
            let row = self.finding(project, number).ok_or(FindingRefusal::NoFinding)?;
            rows.push(row.clone()).expect("sixteen findings");
        }
        for row in &rows {
            if !row.uncleared_red() {
                return Err(FindingRefusal::Words(crate::tiers::joined(&[
                    &address(project, row.number),
                    b" is not an uncleared red filed under landed",
                ])));
            }
        }
        for row in &rows {
            if row.state != FindingState::New {
                let state: &[u8] = match row.state {
                    FindingState::New => b"new",
                    FindingState::Routed => b"routed",
                    FindingState::Backlog => b"backlog",
                    FindingState::Dropped => b"dropped",
                };
                return Err(FindingRefusal::Words(crate::tiers::joined(&[
                    &address(project, row.number),
                    b" is ",
                    state,
                    b", not new",
                ])));
            }
        }
        let first = rows.as_slice().first().expect("first finding");
        let repository = first.signature.as_ref().expect("landed red signature").repository;
        for row in &rows {
            if row.signature.as_ref().expect("landed red signature").repository != repository {
                return Err(FindingRefusal::Words(crate::tiers::joined(&[
                    b"findings ",
                    Decimal::of(u64::from(project)).as_bytes(),
                    b"/",
                    Decimal::of(u64::from(first.number)).as_bytes(),
                    b" and ",
                    Decimal::of(u64::from(project)).as_bytes(),
                    b"/",
                    Decimal::of(u64::from(row.number)).as_bytes(),
                    b" are in two repositories",
                ])));
            }
        }
        for row in &rows {
            if let Some(repair) = &row.repair
                && (repair.attempts >= repairs.attempts || repair.cancelled)
            {
                return Err(FindingRefusal::Words(crate::tiers::joined(&[
                    b"ask before repairing ",
                    &address(project, row.number),
                    b" again",
                ])));
            }
        }
        let mut live = Map::uncapped();
        for (_, row) in self.findings(project) {
            if let Some(repair) = &row.repair
                && repair.until.is_some()
            {
                live.insert(repair.goal, ()).expect("distinct live goals");
            }
        }
        if live.len() >= u32::from(repairs.goals) {
            let mut words = List::uncapped();
            for part in [
                b"project ".as_slice(),
                Decimal::of(u64::from(project)).as_bytes(),
                b" has its ",
                Decimal::of(u64::from(repairs.goals)).as_bytes(),
                b" repair goals live: ",
            ] {
                append(&mut words, part);
            }
            for (at, (&goal, ())) in live.iter().enumerate() {
                if at != 0 {
                    append(&mut words, b", ");
                }
                append(&mut words, b"goal ");
                append(&mut words, Decimal::of(goal).as_bytes());
            }
            return Err(FindingRefusal::Words(words.into_boxed()));
        }
        let mut ordered = Map::with_capacity(16);
        for row in rows.into_boxed() {
            ordered.insert((row.at, row.number), FindingWrite { row, erase: None }).expect("sixteen findings");
        }
        let mut writes = List::with_capacity(16);
        for _ in 0..ordered.len() {
            let key = *ordered.first().expect("ordered finding count").0;
            let write = ordered.remove(&key).expect("first finding held");
            writes.push(write).expect("sixteen findings");
        }
        Ok(writes.into_boxed())
    }
}
fn address(project: u32, number: u32) -> Box<[u8]> {
    crate::tiers::joined(&[
        b"finding ",
        Decimal::of(u64::from(project)).as_bytes(),
        b"/",
        Decimal::of(u64::from(number)).as_bytes(),
    ])
}
fn append(out: &mut List<u8>, words: &[u8]) {
    for &byte in words {
        out.push(byte).expect("repair refusal words");
    }
}
