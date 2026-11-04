//! Task execution facts (temper/docs/design/domain/after-landing.md, 10).
use super::{Box, Domain, TierRow, candidate, joined, p, part_branch};

fn identity(row: &TierRow, run: Option<&skein_forge::api::Run>) -> Box<[u8]> {
    let Some(part) = &row.record.run else { return Box::new([]) };
    let label: &[u8] = match &part.part {
        p::Part::Whole => b"Judged run",
        p::Part::Rerun { .. } => b"Failed-test rerun",
        p::Part::Probe { .. } => b"Bisection probe",
    };
    part_identity(label, candidate(part).0, &part_branch(row), run)
}

fn part_identity(
    label: &[u8],
    commit: skein_forge::api::Commit,
    branch: &[u8],
    run: Option<&skein_forge::api::Run>,
) -> Box<[u8]> {
    let hex = crate::commit_text(&commit);
    let base = joined(&[label, b": commit ", &hex, b", branch ", branch]);
    match run {
        Some(run) if run.number != 0 && !run.url.is_empty() => {
            joined(&[&base, b", forge run ", skein_lib::Decimal::of(run.number).as_bytes(), b" ", &run.url, b"\n"])
        }
        Some(_) | None => joined(&[&base, b"\n"]),
    }
}

pub(super) fn observe(row: &mut TierRow, follow: &crate::runs::Follow) {
    let Some(run) = &follow.run else { return };
    if run.number == 0 || run.url.is_empty() {
        return;
    }
    let line = identity(row, Some(run));
    if skein_lib::bytes::find(&row.observations, &line).is_none() {
        row.observations = joined(&[&row.observations, &line]);
    }
}

impl Domain {
    /// Initial tier task words name its repository and judged execution before dispatch.
    #[must_use]
    pub fn tier_specification(
        &self,
        repository: skein_forge::api::Repository,
        tier: &[u8],
        number: u64,
        tip: skein_forge::api::Commit,
    ) -> Option<Box<[u8]>> {
        let adopted = self.repository(repository)?;
        let mut goal_branch = Box::from(b"".as_slice());
        if let Some(row) = self.tiers.get(&(repository, Box::from(tier)))
            && let Some(goal) = row.goal
            && let Some(run) = &row.record.run
            && run.number == number
        {
            for entry in &row.goals {
                if entry.goal == goal {
                    goal_branch = joined(&[b"Goal branch: ", &entry.branch, b"\n"]);
                }
            }
        }
        Some(joined(&[
            b"Tier ",
            tier,
            b" run ",
            skein_lib::Decimal::of(number).as_bytes(),
            b"\nRepository: ",
            &adopted.owner,
            b"/",
            &adopted.name,
            b"\n",
            &goal_branch,
            &part_identity(b"Judged run", tip, &super::branch(tier, number), None),
        ]))
    }

    /// Execution facts for this live tier task, including its current part before dispatch.
    #[must_use]
    pub fn tier_execution_words(&self, task: u64) -> Option<Box<[u8]>> {
        let row = self.tier_task(task)?;
        Some(joined(&[&row.observations, &identity(row, None)]))
    }
}

pub(crate) fn projected(d: &mut Domain, task: u64, words: &[u8], out: &mut skein_lib::Queue<crate::Request>) {
    let Some(key) = super::task_key(d, task) else { return };
    let row = d.tiers.get_mut(&key).expect("tier task");
    let mut remaining =
        skein_lib::List::with_capacity(u32::try_from(row.observations.len()).expect("bounded observations"));
    let mut from = 0;
    for at in 0..row.observations.len() {
        if row.observations.get(at) != Some(&b'\n') {
            continue;
        }
        let line = row.observations.get(from..=at).expect("bounded observation line");
        from = at.checked_add(1).expect("bounded observation cursor");
        if skein_lib::bytes::find(words, line).is_none() {
            for byte in line {
                remaining.push(*byte).expect("existing observations");
            }
        }
    }
    if remaining.len() != u32::try_from(row.observations.len()).expect("bounded observations") {
        row.observations = remaining.into_boxed();
        super::save(row, out);
    }
}
