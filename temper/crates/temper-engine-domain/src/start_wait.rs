//! A long start step's observations, independent of admission (protocol/service-process.md, 1.3;
//! domain/root-restart.md, 1.1). The forge supplies its own restart terms.
use crate::{Domain, Fact, Limits, Range};
use alloc::boxed::Box;
use jig_domain_core::start::Phase;
use skein_lib::{Decimal, Env, List, bytes};
use temper_engine_domain_forge::wait_kind;

pub(crate) fn report(domain: &mut Domain, env: &Env<Limits>) {
    if domain.start_stop().is_some()
        || domain.journal.stopped()
        || domain.engine.core().stopping()
        || domain.settling.active()
    {
        domain.start_report = None;
        return;
    }
    let Some(step) = domain.start_phase() else {
        domain.start_report = None;
        return;
    };
    let Some(clock) = &mut domain.start_report else { return };
    let Some(age) = clock.due(env.now) else { return };
    let fact = Fact::StartWaiting { step, age, waits: words(domain, step) };
    if domain.facts.try_push(fact).is_err() {
        domain.facts_lost = domain.facts_lost.saturating_add(1);
    }
}

fn words(domain: &Domain, step: Phase) -> Box<[u8]> {
    let mut words = Box::default();
    if let Some(range) = &domain.startup_range
        && !domain.sweep_page
    {
        let name: &[u8] = match range {
            Range::Core(crate::CoreRange::Sweep) => b"the sweep's rows",
            Range::Core(
                crate::CoreRange::Deployment
                | crate::CoreRange::Settings
                | crate::CoreRange::Accounts
                | crate::CoreRange::Tasks
                | crate::CoreRange::RunProofs
                | crate::CoreRange::People
                | crate::CoreRange::Calls
                | crate::CoreRange::TaskResult { .. }
                | crate::CoreRange::RequesterWord { .. }
                | crate::CoreRange::Turns { .. }
                | crate::CoreRange::Notes(_),
            ) => b"the core's records",
            Range::Census(_) => b"the inbox",
            Range::Forge => b"the forge's records",
            Range::History(_) => unreachable!("history is not a start range"),
        };
        words = wait_kind(1, b"store page", b"store pages", &[name.into()]);
    }
    if let Some(oldest) = domain.oldest_commit_in_flight() {
        let count = domain.commits_in_flight();
        let mut names = List::with_capacity(3);
        for offset in 0..count.min(3) {
            let number = oldest.checked_add(offset).expect("sent commit number");
            names.push(bytes::concat(&[b"commit ", Decimal::of(number).as_bytes()])).expect("three oldest commits");
        }
        append(&mut words, &wait_kind(count, b"commit in flight", b"commits in flight", names.as_slice()));
    }
    match step {
        Phase::Switch if domain.start_pending && domain.rehearsal.is_none() => {
            append(&mut words, &wait_kind(1, b"store switch", b"store switch", &[]));
        }
        Phase::LoadCore
        | Phase::LoadInbox
        | Phase::RestoreConnector { .. }
        | Phase::Link
        | Phase::Sweep
        | Phase::Switch
        | Phase::Wires
        | Phase::AdoptRuns
        | Phase::Open => {}
    }
    if words.is_empty() { b"nothing named".as_slice().into() } else { words }
}
fn append(words: &mut Box<[u8]>, kind: &[u8]) {
    if kind.is_empty() {
        return;
    }
    let separator: &[u8] = if words.is_empty() { b"" } else { b", " };
    *words = bytes::concat(&[words, separator, kind]);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::resume_fixture;

    #[test]
    fn store_kinds_include_only_waiters_and_cut_commits_at_the_oldest_three() {
        let limits = resume_fixture::limits();
        let mut domain = Domain::new(resume_fixture::config(7), &limits).unwrap();
        assert_eq!(words(&domain, Phase::Link).as_ref(), b"nothing named");
        for (range, step, expected) in [
            (Range::Core(crate::CoreRange::Deployment), Phase::LoadCore, "1 store page (the core's records)"),
            (
                Range::Census(jig_domain_core::Census { questions: Box::new([]), results: Box::new([]) }),
                Phase::LoadInbox,
                "1 store page (the inbox)",
            ),
            (Range::Forge, Phase::RestoreConnector { connector: 0 }, "1 store page (the forge's records)"),
            (Range::Core(crate::CoreRange::Sweep), Phase::Sweep, "1 store page (the sweep's rows)"),
        ] {
            domain.startup_range = Some(range);
            assert_eq!(words(&domain, step).as_ref(), expected.as_bytes());
        }
        domain.sweep_page = true;
        assert_eq!(
            words(&domain, Phase::Sweep).as_ref(),
            b"nothing named",
            "the sweep's answered page is not owed again"
        );
        domain.sweep_page = false;
        assert_eq!(
            wait_kind(
                2,
                b"store page",
                b"store pages",
                &[b"the core's records".as_slice().into(), b"the inbox".as_slice().into(),]
            )
            .as_ref(),
            b"2 store pages (the core's records, the inbox)"
        );
        domain.startup_range = None;
        domain.durable_commit = 9;
        domain.sent_commit = 10;
        assert_eq!(words(&domain, Phase::Wires).as_ref(), b"1 commit in flight (commit 10)");
        domain.sent_commit = 13;
        assert_eq!(words(&domain, Phase::Wires).as_ref(), b"4 commits in flight (commit 10, commit 11, commit 12)");
        domain.durable_commit = 13;
        domain.start_pending = true;
        assert_eq!(words(&domain, Phase::Switch).as_ref(), b"1 store switch");
        domain.start_pending = false;
        assert_eq!(words(&domain, Phase::Switch).as_ref(), b"nothing named");
    }
}
