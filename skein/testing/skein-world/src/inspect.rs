//! Frozen inspection records and shared checks (skein/docs/design/testing/inspection.md, 3–6).
//! The adapter belongs to its component, beside it under its tree's `testing/` (section 2).

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::String;
use alloc::vec::Vec;
use skein_lib::Time;

/// When an invariant ran: the run's seed, the world's time, and its iteration.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Moment {
    pub seed: u64,
    pub at: Time,
    pub iteration: u64,
}

/// What one component holds at a quiet point, as its adapter read it.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Inspection {
    component: &'static str,
    kinds: &'static [&'static str],
    waits: Vec<Wait>,
    tables: Vec<Table>,
    violation: Option<(String, String)>,
}

impl Inspection {
    /// `kinds`: the waits its design documents, each in the design's words.
    #[must_use]
    pub fn new(component: &'static str, kinds: &'static [&'static str]) -> Inspection {
        Inspection { component, kinds, waits: Vec::new(), tables: Vec::new(), violation: None }
    }

    #[must_use]
    pub fn component(&self) -> &'static str {
        self.component
    }

    pub fn wait(&mut self, wait: Wait) {
        self.waits.push(wait);
    }

    pub fn table(&mut self, table: Table) {
        self.tables.push(table);
    }

    #[must_use]
    pub fn waits(&self) -> &[Wait] {
        &self.waits
    }

    #[must_use]
    pub fn tables(&self) -> &[Table] {
        &self.tables
    }

    /// Let a harness attach its own wakes to the adapter's observations.
    pub fn waits_mut(&mut self) -> &mut [Wait] {
        &mut self.waits
    }

    /// Let a harness mark its fixture's entries without copying the table.
    pub fn tables_mut(&mut self) -> &mut [Table] {
        &mut self.tables
    }

    /// Record the first violation of a component's own invariant, not a wait or table rule.
    pub fn violate(&mut self, object: String, what: String) {
        if self.violation.is_none() {
            self.violation = Some((object, what));
        }
    }

    /// Report that violation at the moment the world took this snapshot.
    pub fn check_violation(&self, moment: Moment) -> Result<(), String> {
        match &self.violation {
            Some((object, what)) => Err(self.finding(moment, object, what)),
            None => Ok(()),
        }
    }

    /// The shared one-line form for a check's finding (section 6).
    #[must_use]
    pub fn finding(&self, moment: Moment, object: &str, what: &str) -> String {
        format!(
            "seed {}: at {} ns, iteration {}: {} {object}: {what}",
            moment.seed,
            moment.at.as_nanos(),
            moment.iteration,
            self.component,
        )
    }
}

/// One wait a live waiter is in.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Wait {
    /// The waiter, by the name every inspection of the world gives it: `task 12`.
    pub waiter: String,
    /// Its kind, one of the inspection's `kinds`, or what the component holds where it is none.
    pub kind: String,
    /// What would wake it.
    pub wakes: Vec<Wake>,
    /// The deadline its design gives it, where it has one.
    pub bound: Option<Time>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Wake {
    /// An armed deadline of the component's own.
    Timer(Time),
    /// Another waiter of the world, by its name, which wakes it when it moves.
    Waiter(String),
    /// A party outside the component: a person, a peer, a fake, by name.
    Party(String),
    /// A source the component still names that has ended.
    Ended(String),
}

/// One bounded table or slot.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Table {
    /// The table, in its design's words: `forge write entries`.
    pub name: &'static str,
    pub capacity: u64,
    pub entries: Vec<Entry>,
    /// Newcomers refused for room that are still waiting to be admitted.
    pub refused: Vec<Refused>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Entry {
    pub key: String,
    /// The rule that releases it, in its design's words: `the write's settlement`.
    pub rule: &'static str,
    pub release: Release,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Release {
    /// Its rule's moment hasn't come.
    Pending,
    /// Its rule's moment has passed, and the entry is still held.
    Due,
    /// An ended record kept on purpose: an ended task's row, a kept answer within its retention.
    Kept,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Refused {
    pub key: String,
    /// When it was first refused.
    pub since: Time,
    /// The bound its design gives its admission, where it has one.
    pub bound: Option<Time>,
}

/// Every wait has a declared kind, a live wake and an unexpired bound (section 4.1).
pub fn check_waits(inspections: &[&Inspection], moment: Moment) -> Result<(), String> {
    let mut waits: BTreeMap<&str, Vec<&Wait>> = BTreeMap::new();
    for inspection in inspections {
        for wait in inspection.waits() {
            waits.entry(&wait.waiter).or_default().push(wait);
        }
    }

    let mut live = BTreeSet::new();
    // A changing pass adds at least one waiter; none can become live twice.
    for _ in 0..waits.len() {
        let mut changed = false;
        for (waiter, own_waits) in &waits {
            if !live.contains(waiter)
                && own_waits.iter().any(|wait| wait.wakes.iter().any(|wake| wake_is_live(wake, &waits, &live)))
            {
                live.insert(*waiter);
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }

    for inspection in inspections {
        for wait in inspection.waits() {
            if !inspection.kinds.contains(&wait.kind.as_str()) {
                return Err(inspection.finding(moment, &wait.waiter, &format!("undeclared wait kind: {}", wait.kind)));
            }
            if !wait.wakes.iter().any(|wake| wake_is_live(wake, &waits, &live)) {
                let wakes: Vec<String> = wait.wakes.iter().map(|wake| describe_wake(wake, &waits)).collect();
                return Err(inspection.finding(
                    moment,
                    &wait.waiter,
                    &format!("waits on {} with no live wake: [{}]", wait.kind, wakes.join("; ")),
                ));
            }
            if let Some(bound) = wait.bound
                && bound < moment.at
            {
                return Err(inspection.finding(
                    moment,
                    &wait.waiter,
                    &format!("waits on {} past bound {} ns", wait.kind, bound.as_nanos()),
                ));
            }
        }
    }
    Ok(())
}

fn wake_is_live(wake: &Wake, waits: &BTreeMap<&str, Vec<&Wait>>, live: &BTreeSet<&str>) -> bool {
    match wake {
        Wake::Timer(_) | Wake::Party(_) => true,
        Wake::Waiter(waiter) => !waits.contains_key(waiter.as_str()) || live.contains(waiter.as_str()),
        Wake::Ended(_) => false,
    }
}

fn describe_wake(wake: &Wake, waits: &BTreeMap<&str, Vec<&Wait>>) -> String {
    match wake {
        Wake::Timer(at) => format!("timer at {} ns", at.as_nanos()),
        Wake::Party(party) => format!("party {party}"),
        Wake::Ended(source) => format!("ended {source}"),
        Wake::Waiter(waiter) => {
            let kinds: Vec<&str> =
                waits.get(waiter.as_str()).into_iter().flatten().map(|wait| wait.kind.as_str()).collect();
            format!("waiter {waiter} (waits: {})", kinds.join(", "))
        }
    }
}

/// Every entry is released by its rule, and every refusal still has a reason (section 4.2).
pub fn check_tables(inspections: &[&Inspection], moment: Moment) -> Result<(), String> {
    for inspection in inspections {
        for table in inspection.tables() {
            for entry in &table.entries {
                match entry.release {
                    Release::Due => {
                        return Err(inspection.finding(
                            moment,
                            &format!("{} {}", table.name, entry.key),
                            &format!("still held after release rule: {}", entry.rule),
                        ));
                    }
                    Release::Pending | Release::Kept => {}
                }
            }
            for refused in &table.refused {
                let object = format!("{} {}", table.name, refused.key);
                if let Some(bound) = refused.bound
                    && bound < moment.at
                {
                    return Err(inspection.finding(
                        moment,
                        &object,
                        &format!("refused admission past bound {} ns", bound.as_nanos()),
                    ));
                }
                if u64::try_from(table.entries.len()).is_ok_and(|held| held < table.capacity) {
                    return Err(inspection.finding(moment, &object, "refused admission while the table has room"));
                }
            }
        }
    }
    Ok(())
}
