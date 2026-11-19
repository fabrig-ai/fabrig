//! The facts a status agent can summarize (temper/docs/design/domain/forge/attention.md, 3).
use super::{Box, Domain, ProjectFacts, Wall};
use core::cmp::Reverse;
use skein_lib::{Decimal, List, Map};
use temper_engine_domain_forge_tiers::Failures;
use temper_engine_domain_forge_vocabulary::findings::{FindingKind, Severity};

fn kind_word(kind: FindingKind) -> &'static [u8] {
    match kind {
        FindingKind::Qa => b"qa",
        FindingKind::Flake => b"flake",
        FindingKind::Red => b"red",
        FindingKind::Review => b"review",
        FindingKind::Feedback => b"feedback",
    }
}

pub(super) fn findings(d: &Domain, project: u32) -> (Box<[u8]>, Option<[u8; 32]>) {
    let mut counts = [
        (FindingKind::Qa, (0_u64, 0_u64, 0_u64)),
        (FindingKind::Flake, (0, 0, 0)),
        (FindingKind::Red, (0, 0, 0)),
        (FindingKind::Review, (0, 0, 0)),
        (FindingKind::Feedback, (0, 0, 0)),
    ];
    let mut total = 0_u64;
    let mut high = 0_u64;
    let mut titles = List::uncapped();
    for (_, finding) in d.findings(project).rev() {
        if finding.state.ended() {
            continue;
        }
        total = total.checked_add(1).expect("bounded findings");
        for (kind, counts) in &mut counts {
            if *kind != finding.kind {
                continue;
            }
            let count = match finding.severity {
                Severity::High => &mut counts.0,
                Severity::Medium => &mut counts.1,
                Severity::Low => &mut counts.2,
            };
            *count = count.checked_add(1).expect("bounded findings");
        }
        if finding.severity == Severity::High {
            high = high.checked_add(1).expect("bounded findings");
            if titles.len() < 20 {
                titles.push(finding).expect("internal high findings");
            }
        }
    }
    let mut words = List::uncapped();
    if total == 0 {
        append(&mut words, b"Open findings: none\n");
        return (words.into_boxed(), None);
    }
    append(&mut words, b"Open findings: ");
    append(&mut words, Decimal::of(total).as_bytes());
    append(&mut words, b"\n");
    for (kind, (high, medium, low)) in counts {
        if high == 0 && medium == 0 && low == 0 {
            continue;
        }
        append(&mut words, b"- ");
        append(&mut words, kind_word(kind));
        append(&mut words, b": ");
        let mut separator = b"".as_slice();
        for (count, word) in [(high, b"high".as_slice()), (medium, b"medium".as_slice()), (low, b"low".as_slice())] {
            if count != 0 {
                append(&mut words, separator);
                append(&mut words, Decimal::of(count).as_bytes());
                append(&mut words, b" ");
                append(&mut words, word);
                separator = b", ";
            }
        }
        append(&mut words, b"\n");
    }
    if high != 0 {
        append(&mut words, b"High ones, newest first:\n");
        for finding in titles.into_boxed() {
            append(&mut words, b"- ");
            append(&mut words, &finding.title);
            append(&mut words, b" (");
            append(&mut words, kind_word(finding.kind));
            append(&mut words, b")\n");
        }
        if high > 20 {
            append(&mut words, b"- and ");
            append(&mut words, Decimal::of(high.saturating_sub(20)).as_bytes());
            append(&mut words, b" more\n");
        }
    }
    let mark = skein_lib::sha256::digest(words.as_slice());
    (words.into_boxed(), Some(mark))
}

fn append(words: &mut List<u8>, part: &[u8]) {
    for byte in part {
        words.push(*byte).expect("internal status words");
    }
}

pub(super) fn specification(
    d: &Domain,
    row: &ProjectFacts,
    brief: Option<&super::Brief>,
    since: Wall,
    now: Wall,
) -> (Box<[u8]>, Box<[u64]>) {
    let covered = match brief {
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
                titles.insert((Reverse(landing.at), *repository, change.task), change).expect("retained titles");
            }
        }
    }
    let mut signers = List::uncapped();
    for (_, change) in titles.iter().take(20) {
        append(&mut words, b"- ");
        append(&mut words, &change.title);
        if !change.signed.is_empty() {
            append(&mut words, b" (signed by ");
            for (at, signed) in change.signed.iter().enumerate() {
                if at != 0 {
                    let separator = if at == change.signed.len().saturating_sub(1) {
                        b" and ".as_slice()
                    } else {
                        b", ".as_slice()
                    };
                    append(&mut words, separator);
                }
                append(&mut words, b"person ");
                append(&mut words, Decimal::of(signed.person).as_bytes());
                if !signers.as_slice().contains(&signed.person) {
                    signers.push(signed.person).expect("named status signer");
                }
            }
            append(&mut words, b")");
        }
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
    (words.into_boxed(), signers.into_boxed())
}
