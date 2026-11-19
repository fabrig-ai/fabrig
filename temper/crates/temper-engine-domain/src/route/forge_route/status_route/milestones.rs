//! Milestones in Home's status words (temper/docs/design/domain/milestones.md, 6, 9).
use super::{Domain, Env, Limits, forge, tasks};
use skein_lib::{Decimal, List, Map, Wall};

const MOST: u32 = 8;

pub(super) fn append(domain: &Domain, env: &Env<Limits>, project: u32, since: Wall, words: &mut List<u8>) {
    let mut live = Map::uncapped();
    for candidate in domain.core.tasks.view_tasks() {
        let Some(row) = domain.core.tasks.task(candidate.number) else { continue };
        if row.project != project || !tasks::gathering::executor(row.executor) {
            continue;
        }
        let landing = last_landing(domain.forge.milestone(row.number));
        let activity = landing.unwrap_or(Wall::EPOCH).max(row.last_activity);
        live.insert((activity, row.number), row).expect("one entry per milestone");
    }
    let ended = domain.forge.ended_milestones(project, since, MOST);
    if live.is_empty() && ended.is_empty() {
        return;
    }
    super::super::append(words, b"Milestones:\n");
    for (_, row) in live.iter().rev().take(skein_lib::usize_of(MOST)) {
        let Ok(chip) = jig_domain_core::documents::chip_from_row(&crate::core_limits(&env.limits), row) else {
            continue;
        };
        super::super::append(words, b"- ");
        super::super::append(words, &chip.title.bytes);
        super::super::append(words, if row.concluded.is_some() { b": whole; " } else { b": open; " });
        let count = row.ended_delegates;
        let total = super::delegate_total(row);
        counts(words, count.done, total, count.failed, count.cancelled);
        landing_words(words, last_landing(domain.forge.milestone(row.number)));
        reds_words(words, domain.forge.reds_owed(row.project, row.number));
    }
    for row in &ended {
        let Some(end) = &row.ended else { continue };
        super::super::append(words, b"- ");
        super::super::append(words, &row.name);
        super::super::append(words, b": ");
        let outcome = match end.outcome {
            forge::MilestoneOutcome::Complete => b"complete".as_slice(),
            forge::MilestoneOutcome::Ended => b"ended",
            forge::MilestoneOutcome::Closed => b"closed",
        };
        super::super::append(words, outcome);
        super::super::append(words, b" at ");
        time(words, end.at);
        super::super::append(words, b"; ");
        let total = end.done.saturating_add(end.failed);
        counts(words, end.done, total, end.failed, end.closed);
        landing_words(words, last_landing(Some(row)));
        reds_words(words, domain.forge.reds_owed(row.project, row.task));
    }
}

fn last_landing(row: Option<&forge::MilestoneRow>) -> Option<Wall> {
    let row = row?;
    match &row.last_landing {
        Some(landing) => Some(landing.at),
        None => None,
    }
}

fn counts(words: &mut List<u8>, done: u32, total: u32, failed: u32, closed: u32) {
    super::super::append(words, Decimal::of(u64::from(done)).as_bytes());
    super::super::append(words, b" of ");
    super::super::append(words, Decimal::of(u64::from(total)).as_bytes());
    super::super::append(words, b" goals done");
    for (count, label) in [(failed, b" failed".as_slice()), (closed, b" closed".as_slice())] {
        if count != 0 {
            super::super::append(words, b", ");
            super::super::append(words, Decimal::of(u64::from(count)).as_bytes());
            super::super::append(words, label);
        }
    }
}

fn landing_words(words: &mut List<u8>, landing: Option<Wall>) {
    if let Some(at) = landing {
        super::super::append(words, b"; last landing ");
        time(words, at);
    }
}

fn reds_words(words: &mut List<u8>, reds: u32) {
    if reds != 0 {
        super::super::append(words, b"; ");
        super::super::append(words, Decimal::of(u64::from(reds)).as_bytes());
        super::super::append(words, if reds == 1 { b" red owed" } else { b" reds owed" });
    }
    super::super::append(words, b"\n");
}

fn time(words: &mut List<u8>, wall: Wall) {
    let date = wall.civil();
    super::super::append(words, Decimal::of(u64::from(date.year)).as_bytes());
    super::super::append(words, b"-");
    two(words, u64::from(date.month));
    super::super::append(words, b"-");
    two(words, u64::from(date.day));
    super::super::append(words, b"T");
    two(words, wall.as_secs().div_euclid(3600).rem_euclid(24));
    super::super::append(words, b":");
    two(words, wall.as_secs().div_euclid(60).rem_euclid(60));
    super::super::append(words, b"Z");
}

fn two(words: &mut List<u8>, value: u64) {
    if value < 10 {
        super::super::append(words, b"0");
    }
    super::super::append(words, Decimal::of(value).as_bytes());
}
