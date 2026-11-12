//! Read-only figures for tuning a deployment (qa/ledger.md, 6).
use crate::{Configuration, Exit, Ledger, Session, SessionKind};
use std::fmt::Write;

impl Ledger {
    #[must_use]
    pub fn tune(&self, configuration: &Configuration, now: u64) -> String {
        let mut text = String::new();
        let mut suggestions = String::new();
        for (kind, name, bound) in [
            (SessionKind::Triage, "triage", configuration.bounds.triage),
            (SessionKind::Pass, "pass", configuration.bounds.pass),
        ] {
            let sessions: Vec<_> = self.sessions.iter().filter(|session| {
                session.kind == kind && session.ended.is_some()
                    && session.exit != Some(Exit::Lost) && session.exit != Some(Exit::Unstarted)
            }).collect();
            session_line(&mut text, name, bound, &sessions);
            let recent: Vec<_> = sessions.iter().rev().take(20).collect();
            let bounded = recent.iter().filter(|session| session.exit == Some(Exit::Bound)).count();
            if bounded != 0 {
                writeln!(suggestions, "suggest: raise bounds.{name} past {bound} s: {bounded} of its last {} sessions ended by it", recent.len()).expect("string");
            }
        }
        let waits: Vec<_> = self.cases.iter().map(|case| {
            let landings = case.landings();
            let kept = self.builds.iter().filter(|build| {
                build.holds.as_ref().is_some_and(|holds| landings.iter().all(|landing| holds.contains(landing)))
            }).map(|build| build.kept).min().unwrap_or(now);
            kept.saturating_sub(case.queued)
        }).collect();
        let bound = configuration.bounds.build_wait;
        let past = waits.iter().filter(|wait| **wait > bound).count();
        writeln!(text, "build_wait: {past} cases waited past {bound} s; the longest wait for a kept build {} s", waits.iter().max().unwrap_or(&0)).expect("string");
        let since = now.saturating_sub(7 * 24 * 60 * 60);
        let waits: Vec<_> = self.place_waits.iter().filter_map(|wait| {
            let end = wait.to.unwrap_or(now).min(now);
            let start = wait.from.max(since);
            (end > start).then_some(end.saturating_sub(start))
        }).collect();
        let total = waits.iter().copied().fold(0u64, u64::saturating_add);
        writeln!(text, "places: {}; {} waits for a place in the last 7 days, the longest {} s, {total} s in all", configuration.places.len(), waits.len(), waits.iter().max().unwrap_or(&0)).expect("string");
        if total >= 3600 {
            writeln!(suggestions, "suggest: another place: passes waited {total} s for one in the last 7 days").expect("string");
        }
        text.push_str(&suggestions);
        text
    }
}

fn session_line(text: &mut String, name: &str, bound: u64, sessions: &[&Session]) {
    let bounded = sessions.iter().filter(|session| session.exit == Some(Exit::Bound)).count();
    let mut times: Vec<_> = sessions.iter().map(|session| session.ended.expect("ended").saturating_sub(session.started)).collect();
    let mut tokens: Vec<_> = sessions.iter().filter_map(|session| session.tokens).collect();
    let time = figures(&mut times);
    let tokens = if tokens.is_empty() {
        "tokens none".into()
    } else {
        let (median, most) = figures(&mut tokens);
        format!("tokens median {median}, most {most}")
    };
    writeln!(text, "{name}: {} sessions; {bounded} ended by the bound of {bound} s; time median {} s, longest {} s; {tokens}", sessions.len(), time.0, time.1).expect("string");
}

fn figures(values: &mut [u64]) -> (u64, u64) {
    values.sort_unstable();
    let middle = values.len() / 2;
    let upper = values.get(middle).copied().unwrap_or(0);
    let median = if values.len().is_multiple_of(2) && middle != 0 {
        let lower = values.get(middle.saturating_sub(1)).copied().unwrap_or(0);
        lower.saturating_add(upper.saturating_sub(lower) / 2)
    } else { upper };
    (median, values.last().copied().unwrap_or(0))
}
