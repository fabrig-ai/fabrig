//! Receipt checks for a landing repair (temper/docs/design/domain/forge/change/repairs.md, A landing's repair).
use crate::LandingFailure;
use alloc::boxed::Box;
use skein_forge::api;
use skein_lib::List;

/// Overlay the receipt's gate jobs, reserving rows before the forge's checks.
pub fn append_gate_checks(statuses: &mut Box<[api::Status]>, failure: LandingFailure, rows: u32) {
    if failure.jobs.is_empty() {
        return;
    }
    let bound = usize::try_from(rows).expect("row bound fits usize");
    let gate_rows = failure.jobs.len().min(bound);
    let forge_rows = statuses.len().min(bound.saturating_sub(gate_rows));
    let cut = forge_rows < statuses.len() || gate_rows < failure.jobs.len();
    let mut combined = List::with_capacity(rows);
    for status in statuses.iter().take(forge_rows) {
        combined.push(status.clone()).expect("forge rows leave room for gate jobs");
    }
    for job in failure.jobs.iter().take(gate_rows) {
        let mut context = List::uncapped();
        for byte in b"temper/gate/".iter().chain(job.name.iter()) {
            context.push(*byte).expect("context is internal receipt text");
        }
        combined
            .push(api::Status {
                at: skein_lib::Time::ZERO,
                context: context.into_boxed(),
                check: api::Check::Failed,
                description: if cut {
                    Box::from(b"Row bound: gate jobs take precedence; remaining checks omitted.".as_slice())
                } else {
                    Box::new([])
                },
                url: failure.run.url.clone(),
                job: Some(api::JobAttempt {
                    head: failure.prefix,
                    run: failure.run.id,
                    job: job.job,
                    attempt: job.attempt,
                }),
            })
            .expect("gate jobs have reserved rows");
    }
    *statuses = combined.into_boxed();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gate_jobs_reserve_rows_and_report_displaced_forge_checks() {
        let failure = LandingFailure {
            report: crate::ReceiptReport::Unset,
            gate: b"gate".as_slice().into(),
            run: crate::ReceiptRun { id: 11, number: 3, url: b"https://forge.example/run/3".as_slice().into() },
            prefix: [7; 32],
            base: None,
            jobs: Box::new([crate::ReceiptJob { job: 17, attempt: 2, name: b"test".as_slice().into() }]),
        };
        let first = api::Status {
            at: skein_lib::Time::ZERO,
            context: b"build".as_slice().into(),
            check: api::Check::Passed,
            description: Box::new([]),
            url: Box::new([]),
            job: None,
        };
        let second = api::Status { context: b"fmt".as_slice().into(), ..first.clone() };
        for (rows, kept) in [(3, 2), (2, 1), (1, 0)] {
            let mut statuses: Box<[api::Status]> = Box::new([first.clone(), second.clone()]);
            append_gate_checks(&mut statuses, failure.clone(), rows);
            assert_eq!(statuses.len(), kept + 1);
            let (gate, own) = statuses.split_last().expect("gate job");
            assert_eq!(own, &([first.clone(), second.clone()])[..kept]);
            assert_eq!(gate.context.as_ref(), b"temper/gate/test");
            assert_eq!(gate.check, api::Check::Failed);
            assert_eq!(gate.url, failure.run.url);
            assert_eq!(gate.job, Some(api::JobAttempt { head: failure.prefix, run: 11, job: 17, attempt: 2 }));
            assert_eq!(gate.description.is_empty(), kept == 2);
            if kept < 2 {
                assert_eq!(
                    gate.description.as_ref(),
                    b"Row bound: gate jobs take precedence; remaining checks omitted."
                );
            }
        }
    }
}
