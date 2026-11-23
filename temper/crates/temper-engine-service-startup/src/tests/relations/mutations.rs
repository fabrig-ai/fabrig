//! Section 6 draws each mutation from its row; intersecting relations supply companions.
use super::{Kind, Profile, Relation, input_slots, item, profile, startup, stock, value};
use jig_domain_core::settings::Override;
use skein_lib::List;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Decision {
    Followed,
    Kept,
    Refused,
    Explicit,
}
#[derive(Debug)]
struct Cached {
    worker: bool,
    full_start: bool,
    changes: Box<[Override]>,
    result: Result<startup::Resolved, startup::Problems>,
}
struct Walk {
    cache: List<Cached>,
    count: u32,
    prepared: startup::Preflight,
    worker_prepared: startup::Preflight,
    inputs: startup::LimitInputs,
    worker: startup::LimitInputs,
    base: Profile,
    worker_base: Profile,
    failures: List<String>,
}
fn supplied(changes: &[Override], name: &str) -> bool {
    changes.iter().any(|c| c.name.as_ref() == name.as_bytes())
}
fn writable(name: &str) -> bool {
    jig_protocol_config::names::find(name.as_bytes()).is_some() || crate::limits::find(name.as_bytes()).is_some()
}
fn candidate(base: &Profile, changes: &[Override], rows: &List<Relation>) -> Profile {
    let mut p = *base;
    for change in changes {
        if change.name.as_ref() == b"worker.slots" {
            p.slots = u32::try_from(change.value).expect("test slots");
        } else if jig_protocol_config::names::find(&change.name).is_some() {
            jig_protocol_config::overrides::apply(&mut p.jig, &change.name, change.value).expect("typed jig mutation");
        } else {
            crate::limits::apply(&mut p.own, &change.name, change.value).expect("typed temper mutation");
        }
    }
    for row in rows {
        if row.implicit && !supplied(changes, row.figure) {
            let required = row.required(&p);
            crate::limits::apply(&mut p.own, row.figure.as_bytes(), required).expect("implicit follower");
        }
    }
    p
}
fn conflict<'a>(row: &Relation, rows: &'a List<Relation>, p: &Profile) -> Option<&'a Relation> {
    // Jig's preflight checks run before the application-side checks.
    for jig in [true, false] {
        for other in rows {
            if other.name == row.name
                || (other.worker && !row.worker)
                || jig_protocol_config::names::find(other.figure.as_bytes()).is_some() != jig
            {
                continue;
            }
            match other.kind {
                Kind::Follows | Kind::SetByTemper | Kind::MayRefuseBatch => {}
                Kind::Floor | Kind::CheckedTogether | Kind::HoldsAll => {
                    if !other.holds(p) {
                        return Some(other);
                    }
                }
            }
        }
    }
    None
}
fn slots(inputs: &mut startup::LimitInputs, count: u64) {
    let mut doc: serde_json::Value = serde_json::from_slice(&inputs.document).expect("fixture document");
    *doc.get_mut("workers")
        .expect("workers")
        .as_array_mut()
        .expect("worker array")
        .first_mut()
        .expect("one worker")
        .get_mut("slots")
        .expect("slots") = serde_json::json!(count);
    inputs.document = serde_json::to_vec(&doc).expect("fixture document bytes").into_boxed_slice();
}
impl Walk {
    #[expect(
        clippy::too_many_lines,
        reason = "one mutation judgment keeps resolution and its expected decision together"
    )]
    fn judge(&mut self, row: &Relation, name: &str, changes: &[Override], decision: Decision, refusal: &Relation) {
        self.count = self.count.checked_add(1).expect("mutation count");
        eprintln!(
            "mutation {name}: {} ({:?}; {}) -> {decision:?} {}",
            row.name,
            row.kind,
            row.design,
            if decision == Decision::Refused { refusal.name } else { "" }
        );
        let mut jig = List::with_capacity(100);
        let mut temper = List::with_capacity(100);
        for change in changes {
            if change.name.as_ref() == b"worker.slots" {
                slots(&mut self.worker, change.value);
            } else if jig_protocol_config::names::find(&change.name).is_some() {
                jig.push(change.clone()).expect("mutation overrides");
            } else {
                temper.push(change.clone()).expect("mutation overrides");
            }
        }
        let inputs = if row.worker { &self.worker } else { &self.inputs };
        // A shared source is one mutation at each check depth. Resolve it once,
        // then let every row judge that same preflight or full-start decision.
        let result = match self
            .cache
            .iter()
            .find(|c| c.worker == row.worker && c.full_start == row.full_start && c.changes.as_ref() == changes)
        {
            Some(cached) => cached.result.clone(),
            None => {
                let prepared = if row.worker { &self.worker_prepared } else { &self.prepared };
                let result = prepared.check(inputs, jig.as_slice(), temper.as_slice(), row.full_start);
                self.cache
                    .push(Cached {
                        worker: row.worker,
                        full_start: row.full_start,
                        changes: changes.into(),
                        result: result.clone(),
                    })
                    .expect("bounded unique mutations");
                result
            }
        };
        let failure = match result {
            Ok(resolved) => {
                let mut p = profile(&resolved);
                p.slots = input_slots(inputs);
                match decision {
                    Decision::Followed => {
                        let actual = row.actual(&p);
                        let expected = row.required(&p);
                        if actual == expected && super::unit_rule(row, &p) {
                            None
                        } else {
                            Some(format!("followed: got {actual}, formula {expected}"))
                        }
                    }
                    Decision::Kept | Decision::Explicit => {
                        let wanted = changes
                            .iter()
                            .find(|o| o.name.as_ref() == row.figure.as_bytes())
                            .expect("figure override")
                            .value;
                        let actual = row.actual(&p);
                        if actual == wanted { None } else { Some(format!("kept: got {actual}, wanted {wanted}")) }
                    }
                    Decision::Refused => Some("accepted instead of refused".to_owned()),
                }
            }
            Err(problems) => {
                let text = String::from_utf8(startup::lines(&problems).into_vec()).expect("diagnostic UTF-8");
                match decision {
                    Decision::Followed | Decision::Kept => Some(format!("refused: {text}")),
                    Decision::Refused | Decision::Explicit => {
                        let names = if writable(refusal.figure) {
                            text.contains(refusal.figure)
                        } else {
                            refusal.sources.iter().any(|s| supplied(changes, s) && text.contains(s))
                        };
                        let detail = match refusal.kind {
                            Kind::Floor => text.contains("short of the source's derivation"),
                            Kind::CheckedTogether => refusal.sources.iter().all(|s| text.contains(s)),
                            Kind::HoldsAll => {
                                refusal.sources.iter().any(|s| text.contains(s))
                                    && (text.contains("bound") || text.contains("ceiling") || text.contains("short"))
                            }
                            Kind::Follows | Kind::SetByTemper | Kind::MayRefuseBatch => true,
                        };
                        if names && detail {
                            None
                        } else {
                            Some(format!("refused without {}'s names: {text}", refusal.name))
                        }
                    }
                }
            }
        };
        if row.worker {
            slots(&mut self.worker, u64::from(self.worker_base.slots));
        }
        if let Some(failure) = failure {
            self.failures
                .push(format!("{name}: relation {} ({}): {failure}", row.name, row.design))
                .expect("bounded failures");
        }
    }
    fn case(&mut self, rows: &List<Relation>, row: &Relation, name: &str, change: Override, decision: Decision) {
        if decision == Decision::Refused || decision == Decision::Explicit {
            self.judge(row, name, &[change], decision, row);
            return;
        }
        let base = if row.worker { self.worker_base } else { self.base };
        let mut changes = List::with_capacity(100);
        changes.push(change).expect("one mutation");
        let p = candidate(&base, changes.as_slice(), rows);
        let Some(other) = conflict(row, rows, &p) else {
            self.judge(row, name, changes.as_slice(), decision, row);
            return;
        };
        self.judge(row, name, changes.as_slice(), Decision::Refused, other);
        // Never change the original mutation or a fixed ceiling to manufacture acceptance.
        for _ in rows {
            let p = candidate(&base, changes.as_slice(), rows);
            let Some(other) = conflict(row, rows, &p) else {
                self.judge(row, &format!("{name} with companions"), changes.as_slice(), decision, row);
                return;
            };
            if !writable(other.figure) || supplied(changes.as_slice(), other.figure) {
                return;
            }
            changes.push(item(other.figure, other.required(&p))).expect("at most one companion per row");
        }
        panic!("companion relations did not settle for {name}");
    }
}
pub(super) fn walk(rows: &List<Relation>) {
    let base = stock();
    let worker = super::super::limit_settings::inputs_for(true);
    let worker_base = super::resolved_profile(&worker, &[], &[]);
    let inputs = super::super::limit_settings::inputs();
    let mut walk = Walk {
        prepared: startup::Preflight::new(&inputs).expect("stock preflight"),
        worker_prepared: startup::Preflight::new(&worker).expect("worker preflight"),
        cache: List::with_capacity(512),
        count: 0,
        inputs,
        worker,
        base,
        worker_base,
        failures: List::with_capacity(512),
    };
    for row in rows {
        let base = if row.worker { worker_base } else { base };
        match row.kind {
            Kind::Follows | Kind::SetByTemper => {
                for source in row.sources {
                    for (label, changed) in [
                        ("half", value(&base, source).checked_div(2).expect("half")),
                        ("twice", value(&base, source).checked_mul(2).expect("twice")),
                    ] {
                        walk.case(rows, row, &format!("{source} {label}"), item(source, changed), Decision::Followed);
                    }
                }
                if row.kind == Kind::Follows && writable(row.figure) {
                    let changed = row.explicit.unwrap_or(row.actual(&base).checked_div(2).expect("half figure").max(1));
                    walk.case(
                        rows,
                        row,
                        &format!("{} explicit", row.figure),
                        item(row.figure, changed),
                        Decision::Explicit,
                    );
                }
            }
            Kind::Floor => {
                let required = row.required(&base);
                for (label, changed, decision) in [
                    ("at derivation", required, Decision::Kept),
                    ("one below", required.checked_sub(1).expect("positive derivation"), Decision::Refused),
                    ("one above", required.checked_add(1).expect("derivation room"), Decision::Kept),
                ] {
                    walk.case(rows, row, &format!("{} {label}", row.figure), item(row.figure, changed), decision);
                }
            }
            Kind::CheckedTogether => {
                let changed = if row.minimum {
                    row.required(&base).checked_sub(1).expect("below source")
                } else {
                    row.required(&base).checked_add(1).expect("past source")
                };
                walk.case(
                    rows,
                    row,
                    &format!("{} past its source", row.figure),
                    item(row.figure, changed),
                    Decision::Refused,
                );
                for source in row.sources {
                    let changed = row.actual(&base).checked_add(1).expect("past partner");
                    walk.case(
                        rows,
                        row,
                        &format!("{source} past its partner"),
                        item(source, changed),
                        Decision::Refused,
                    );
                }
            }
            Kind::HoldsAll | Kind::MayRefuseBatch => {
                let change = match row.overload {
                    Some(overload) => overload(&base),
                    None => item(row.figure, row.required(&base).checked_sub(1).expect("positive capacity")),
                };
                let decision = if row.kind == Kind::MayRefuseBatch { Decision::Kept } else { Decision::Refused };
                walk.case(rows, row, &format!("{} one below what it holds", row.figure), change, decision);
            }
        }
    }
    for failure in &walk.failures {
        eprintln!("{failure}");
    }
    assert!(walk.failures.is_empty(), "{} of {} generated mutations failed", walk.failures.len(), walk.count);
    eprintln!(
        "{} generated row mutations got their decision through {} distinct resolutions (preflight, or full start for composed checks)",
        walk.count,
        walk.cache.len()
    );
}
