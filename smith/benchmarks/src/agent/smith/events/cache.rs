//! Cache reuse joins adjacent requests only within an affinity/window.
//! R sums reported cache reads over cacheable predecessors' disjoint prompt
//! counts; eligibility, the threshold and full-miss rate follow
//! benchmarks/catalogue.md, section 1.

use std::collections::{BTreeMap, BTreeSet};

use smith_events::{Event, Outcome, Usage};

use crate::{CacheObservation, CheckVerdict, Fault, Measure, Stream, bound, refuse_if, stream};

use super::Observer;

fn prompt(usage: &Usage) -> Option<u128> {
    Some(u128::from(usage.input_tokens?) + u128::from(usage.cache_read_tokens?) + u128::from(usage.cache_write_tokens?))
}

fn measured(observer: &Observer) -> Result<(u128, u128, u64, u64), Fault> {
    let mut reads = 0_u128;
    let mut predecessors = 0_u128;
    let mut eligible = 0_u64;
    let mut misses = 0_u64;
    for session in &observer.sessions {
        let mut openings = BTreeMap::new();
        let mut requests = BTreeMap::new();
        let mut completed = BTreeSet::new();
        let mut previous = BTreeMap::new();
        for record in &session.records {
            if let Event::ConversationOpened(value) = &record.event {
                openings.insert((value.run, value.conversation), value);
            }
            if let Event::ResponseStarted(value) = &record.event {
                let opening = openings
                    .get(&(value.run, value.conversation))
                    .ok_or_else(|| stream("cache request lacks its conversation opening"))?;
                if opening.model != value.model || value.window == 0 {
                    return Err(stream("cache request lacks a valid matching model/window"));
                }
                let scope = (value.run, value.conversation, opening.thread, value.window);
                if let Some(existing) = requests.insert((value.run, value.conversation, value.response), scope)
                    && existing != scope
                {
                    return Err(stream("repeated request changed its cache scope"));
                }
            }
            if let Event::ResponseCompleted(value) = &record.event {
                let identity = (value.run, value.conversation, value.response);
                if !completed.insert(identity) {
                    continue;
                }
                let scope = requests.get(&identity).ok_or_else(|| stream("cache response lacks its request"))?;
                // A failed attempt without reported prompt usage cannot supply a
                // known predecessor; do not bridge across it to an older request.
                if let Some(predecessor) = previous.insert(*scope, prompt(&value.usage)) {
                    let predecessor =
                        predecessor.ok_or_else(|| stream("provider did not report the predecessor's prompt counts"))?;
                    if predecessor < 1152 {
                        continue;
                    }
                    if value.outcome != Outcome::Completed {
                        return Err(stream("cacheable request did not complete with usage"));
                    }
                    let read = u128::from(
                        value.usage.cache_read_tokens.ok_or_else(|| stream("provider did not report cache reads"))?,
                    );
                    reads = reads.checked_add(read).ok_or_else(|| bound("cache read sum overflow"))?;
                    predecessors =
                        predecessors.checked_add(predecessor).ok_or_else(|| bound("cache predecessor sum overflow"))?;
                    eligible = eligible.checked_add(1).ok_or_else(|| bound("cache request count overflow"))?;
                    if read.checked_mul(2).ok_or_else(|| bound("cache miss comparison overflow"))? < predecessor {
                        misses = misses.checked_add(1).ok_or_else(|| bound("cache miss count overflow"))?;
                    }
                }
            }
        }
    }
    refuse_if(eligible == 0, Stream, "no request has a reported cacheable predecessor in its window")?;
    Ok((reads, predecessors, eligible, misses))
}

pub(super) fn observation(
    observer: &Observer,
    median_at_least: f64,
    full_misses_per_ten_at_most: u32,
) -> Measure<CacheObservation> {
    let counts = measured(observer).and_then(|(reads, prompts, eligible, full_misses)| {
        let cache_read = u64::try_from(reads).map_err(|_| bound("cache reads exceed serialized count bound"))?;
        let predecessor_prompt =
            u64::try_from(prompts).map_err(|_| bound("cache prompts exceed serialized count bound"))?;
        Ok(CacheObservation {
            cache_read,
            predecessor_prompt,
            eligible,
            full_misses,
            median_at_least,
            full_misses_per_ten_at_most,
        })
    });
    match counts {
        Ok(value) => Measure::Observed { value },
        Err(reason) => Measure::unavailable(reason.detail),
    }
}

pub(super) fn check(observer: &Observer, median_at_least: f64, full_misses_per_ten_at_most: u32) -> CheckVerdict {
    let measurement = observation(observer, median_at_least, full_misses_per_ten_at_most);
    let evidence = match &measurement {
        Measure::Observed { value } => vec![format!(
            "reuse={}; cache_read={}; predecessor_prompt={}; eligible={}; full_misses={}; triplet judgment pending",
            value.reuse(),
            value.cache_read,
            value.predecessor_prompt,
            value.eligible,
            value.full_misses
        )],
        Measure::LowerBound { .. } | Measure::Unavailable { .. } => Vec::new(),
    };
    CheckVerdict {
        check: "cache-reuse".into(),
        passed: Measure::unavailable("cache reuse is judged on a complete three-repetition triplet"),
        evidence,
    }
}
