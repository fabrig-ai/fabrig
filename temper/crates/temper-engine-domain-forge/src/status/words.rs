//! The facts a status agent can summarize (temper/docs/design/domain/forge/attention.md, 3).
use super::{Box, Domain, ProjectFacts, Wall};
use core::cmp::Reverse;
use skein_lib::{Decimal, List, Map};
use temper_engine_domain_forge_tiers::Failures;

fn append(words: &mut List<u8>, part: &[u8]) {
    for byte in part {
        words.push(*byte).expect("internal status words");
    }
}

pub(super) fn specification(d: &Domain, row: &ProjectFacts, now: Wall) -> Box<[u8]> {
    let since = match &row.brief {
        Some(brief) => brief.at,
        None => Wall::EPOCH,
    };
    let covered = match &row.brief {
        Some(brief) => brief.landed,
        None => 0,
    };
    let hours = now
        .as_nanos()
        .saturating_sub(since.as_nanos())
        .checked_div(super::HOUR.as_nanos())
        .expect("an hour is nonzero");
    let mut words = List::uncapped();
    append(&mut words, b"Since the last brief: ");
    append(&mut words, Decimal::of(hours).as_bytes());
    append(&mut words, b" hours; ");
    append(&mut words, Decimal::of(row.landed.saturating_sub(covered)).as_bytes());
    append(&mut words, b" changes landed.\nLandings, newest first:\n");
    let mut titles = Map::uncapped();
    for (repository, log) in &d.landing_logs {
        let Some(adopted) = d.repositories.get(repository) else { continue };
        if adopted.project != row.project {
            continue;
        }
        for landing in &log.landings {
            if landing.at <= since {
                continue;
            }
            for change in &landing.changes {
                titles.insert((Reverse(landing.at), *repository, change.task), &change.title).expect("retained titles");
            }
        }
    }
    for (_, title) in titles.iter().take(20) {
        append(&mut words, b"- ");
        append(&mut words, title);
        append(&mut words, b"\n");
    }
    for ((repository, tier), held) in &d.tiers {
        let Some(adopted) = d.repositories.get(repository) else { continue };
        if adopted.project != row.project {
            continue;
        }
        append(&mut words, b"Tier ");
        append(&mut words, tier);
        append(&mut words, b": ");
        match &held.record.failing {
            Failures::Many => append(&mut words, b"many failing tests"),
            Failures::Tests(tests) => {
                append(&mut words, Decimal::of(skein_lib::len_u64(tests.len())).as_bytes());
                append(&mut words, b" failing tests");
                for test in tests.iter().take(5) {
                    append(&mut words, b"; ");
                    append(&mut words, test);
                }
            }
        }
        append(&mut words, b"\n");
    }
    words.into_boxed()
}
