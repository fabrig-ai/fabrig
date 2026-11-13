//! Engine log openings and bounded polls (temper/docs/design/redeploy.md, 4.7).
use crate::{
    description::Description,
    hosts::{Hosts, Part, fill},
    record::{Journal, Opening, Window},
    stage::part_name,
};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) const SECOND: u64 = 1_000_000_000;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Event {
    Opened { worker: String, wires: Vec<(String, u16)> },
    Refused { worker: String, text: String },
}
impl Event {
    #[must_use]
    pub fn worker(&self) -> &str {
        match self {
            Self::Opened { worker, .. } | Self::Refused { worker, .. } => worker,
        }
    }
}
/// The prefix may include the service manager's timestamp and host.
#[must_use]
pub fn parse(line: &str) -> Option<Event> {
    let (_, rest) = line.trim_end_matches('\n').split_once("serve: worker ")?;
    if let Some((worker, pairs)) = rest.rsplit_once(" opened: ") {
        let wires: Option<Vec<_>> = if pairs.is_empty() {
            Some(Vec::new())
        } else {
            pairs
                .split(", ")
                .map(|pair| {
                    let (name, version) = pair.rsplit_once(' ')?;
                    if name.is_empty() {
                        return None;
                    }
                    Some((name.into(), version.parse().ok()?))
                })
                .collect()
        };
        if let Some(wires) = wires {
            return Some(Event::Opened { worker: worker.into(), wires });
        }
    }
    let (worker, text) = rest.split_once(" refused at its opening: ")?;
    Some(Event::Refused { worker: worker.into(), text: text.into() })
}

pub(crate) struct Wait {
    line: String,
    seen: BTreeMap<String, usize>,
    awaited: Vec<String>,
    opened: BTreeSet<String>,
    refused: Option<String>,
    pub(crate) deadline: u64,
    last: u64,
    window: Option<u64>,
}
impl Wait {
    pub(crate) fn new(
        description: &Description,
        awaited: &[Part],
        window: Option<u64>,
        journal: &mut Journal,
        hosts: &mut dyn Hosts,
    ) -> Result<Self, String> {
        let start = hosts.now();
        let since = (start / SECOND).saturating_sub(1).to_string();
        let line = fill(&description.engine.log, &[("since", &since)]);
        // The one-second overlap can contain an earlier start's identical opening.
        // Count the existing occurrences before starting, not just distinct texts.
        let baseline = log(hosts, &line)?;
        let mut seen = BTreeMap::new();
        for line in baseline.lines() {
            *seen.entry(line.into()).or_default() += 1;
        }
        let awaited: Vec<_> = awaited.iter().map(|part| part_name(description, *part).to_owned()).collect();
        let grace = journal
            .record
            .graces
            .iter()
            .filter(|grace| awaited.contains(&grace.worker))
            .map(|grace| grace.grace_ms)
            .max()
            .unwrap_or(0);
        if awaited.len() == description.workers.len() {
            journal.record.openings.clear();
        } else {
            journal.record.openings.retain(|opening| !awaited.contains(&opening.worker));
        }
        journal.save()?;
        Ok(Self {
            line,
            seen,
            awaited,
            opened: BTreeSet::new(),
            refused: None,
            deadline: start.saturating_add(grace.saturating_mul(1_000_000)),
            last: start,
            window,
        })
    }
    pub(crate) fn poll(
        &mut self,
        journal: &mut Journal,
        hosts: &mut dyn Hosts,
    ) -> Result<Vec<(u64, String, Event)>, String> {
        let text = log(hosts, &self.line)?;
        let at = hosts.now();
        if at > self.deadline {
            return Err(self.expired(at).expect("the bound has passed"));
        }
        let mut counts = BTreeMap::new();
        let mut events = Vec::new();
        for line in text.lines() {
            let count = counts.entry(line.to_owned()).or_insert(0_usize);
            *count += 1;
            if *count <= self.seen.get(line).copied().unwrap_or(0) {
                continue;
            }
            let Some(event) = parse(line) else { continue };
            let worker = event.worker();
            let refused = match &event {
                Event::Opened { .. } => false,
                Event::Refused { .. } => true,
            };
            journal.record.openings.push(Opening { worker: worker.into(), at, line: line.into(), refused });
            if self.awaited.iter().any(|name| name == worker) {
                if refused {
                    self.refused = Some(format!("worker {worker} refused: {line}"));
                } else {
                    self.opened.insert(worker.into());
                    self.last = at;
                    if let Some(from) = self.window.take() {
                        journal.record.window = Some(Window { from, to: at, worker: worker.into() });
                    }
                }
            }
            events.push((at, line.into(), event));
        }
        self.seen = counts;
        Ok(events)
    }
    pub(crate) fn complete(&self) -> bool {
        self.awaited.iter().all(|worker| self.opened.contains(worker))
    }
    pub(crate) fn refusal(&self) -> Option<String> {
        self.refused.clone()
    }
    pub(crate) fn expired(&self, at: u64) -> Option<String> {
        (at >= self.deadline).then(|| {
            format!(
                "workers did not open within the largest grace: {}",
                self.awaited
                    .iter()
                    .filter(|worker| !self.opened.contains(*worker))
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        })
    }
    pub(crate) fn last_opening(&self) -> u64 {
        self.last
    }
}
fn log(hosts: &mut dyn Hosts, line: &str) -> Result<String, String> {
    let answer = hosts.run(Part::Engine, line);
    if answer.code == Some(0) {
        Ok(answer.stdout)
    } else {
        Err(format!("engine log: exit {:?}: {}", answer.code, answer.stderr.trim()))
    }
}
