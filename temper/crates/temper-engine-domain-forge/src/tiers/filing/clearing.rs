//! An enumerated run clears findings before giving the oldest many finding its ids (red-findings.md, 5–6).
use super::{BlameStrength, Domain, Env, FindingWrite, Limits, List, Request, TierRow, held, p, words};
use crate::findings::FindingCleared;
use skein_lib::Queue;

pub(super) fn judged(d: &mut Domain, env: &Env<Limits>, row: &TierRow, verdict: &p::Verdict, out: &mut Queue<Request>) {
    let p::Failures::Tests(failed) = &verdict.failing else { return };
    let project = d.repositories.get(&row.repository).expect("adopted tier").project;
    let mut changes = List::uncapped();
    let mut many = List::uncapped();
    let mut oldest = None;
    for (_, finding) in d.findings(project) {
        if !finding.uncleared_red() {
            continue;
        }
        let Some(signature) = &finding.signature else { continue };
        if signature.repository != row.repository || signature.source != row.tier {
            continue;
        }
        if signature.many {
            let key = (finding.at, finding.number);
            if oldest.is_none() || Some(key) < oldest {
                oldest = Some(key);
            }
            many.push(finding.clone()).expect("many findings");
            continue;
        }
        let mut fails = false;
        for id in &signature.ids {
            fails |= failed.contains(id);
        }
        if !fails {
            let mut finding = finding.clone();
            finding.cleared = Some(FindingCleared { run: verdict.number, at: env.wall });
            changes.push(finding).expect("cleared findings");
        }
    }
    for finding in changes.into_boxed() {
        d.commit_finding(&env.limits, FindingWrite { row: finding, erase: None }, out);
    }
    let (held, _) = held(d, row, project);
    let mut ids = List::uncapped();
    for id in failed {
        if !held.as_slice().contains(id) {
            ids.push(id.clone()).expect("unheld failures");
        }
    }
    let failing = p::Failures::Tests(ids.into_boxed());
    for mut finding in many.into_boxed() {
        let empty = match &failing {
            p::Failures::Tests(ids) => ids.is_empty(),
            p::Failures::Many => unreachable!("enumerated run"),
        };
        if Some((finding.at, finding.number)) == oldest && !empty {
            let signature = finding.signature.as_mut().expect("many signature");
            let p::Failures::Tests(ids) = &failing else { unreachable!("enumerated run") };
            signature.ids.clone_from(ids);
            signature.many = false;
            let blame = finding.blame.as_mut().expect("red blame");
            blame.strength = BlameStrength::Suspected;
            finding.title = words::counted(&finding.title, &failing);
        } else {
            finding.cleared = Some(FindingCleared { run: verdict.number, at: env.wall });
        }
        d.commit_finding(&env.limits, FindingWrite { row: finding, erase: None }, out);
    }
}
