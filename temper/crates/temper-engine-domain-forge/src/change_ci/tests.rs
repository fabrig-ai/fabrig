use super::{counts, release_baseline, runs_again};
use crate::{RunnerCi, Stored, tests::fixtures};
use skein_forge::{api, measure};
use skein_lib::{List, Time};
use temper_protocol_store::{bounds, keys, records, v25};

#[test]
fn a_full_checks_answer_round_trips_the_release_baseline() {
    // Both reads use forge.client.rows; the comparison list sets its ceiling.
    let rows = v25::CEILINGS.compare_answer_files;
    assert_eq!(rows, v25::CEILINGS.runner_baseline_attempts);
    let mut limits = crate::tests::LIMITS;
    limits.client.rows = rows;
    limits.client.answer_bytes = v25::CEILINGS.status_context;
    let bounds = bounds(&limits, limits.client.op_bytes).expect("the checks ceiling fits the store");
    assert_eq!(bounds.limits().runner_baseline_attempts, rows);

    let head = fixtures::HEAD;
    let mut checks = List::with_capacity(rows);
    for job in 1..=u64::from(rows) {
        checks
            .push(api::Status {
                at: Time::ZERO,
                context: format!("ci/{job}").into_bytes().into_boxed_slice(),
                check: api::Check::Failed,
                description: Box::new([]),
                url: Box::new([]),
                job: Some(api::JobAttempt { head, run: 1, job, attempt: 2 }),
            })
            .unwrap();
    }
    let checks = api::Answer::Checks(checks.into_boxed());
    assert_eq!(measure::answer_list_rows(&checks), usize::try_from(limits.client.rows).unwrap());
    assert!(measure::answer(&checks).unwrap() <= u64::from(limits.client.answer_bytes));
    let api::Answer::Checks(checks) = checks else { panic!("checks answer") };
    let mut change = fixtures::opening_row();
    change.change.state = temper_engine_domain_forge_change::State::Checking { head };
    change.runner_ci = Some(RunnerCi::Counting { attempts: release_baseline(&checks, head) });
    let key = keys::encode(&crate::Key::ChangeHot(change.task), u32::MAX).expect("the change key fits");
    let (_, hot) = change.split();
    let record = Stored::ChangeHot(Box::new(hot));
    let encoded = records::encode(&record);
    let mut restored = records::row(&key, &encoded, &bounds).expect("the release row decodes");
    let bytes_with_baseline = crate::stored_bytes(&restored).unwrap();
    let Stored::ChangeHot(hot) = &mut restored else { panic!("change hot row") };
    let Some(RunnerCi::Counting { attempts }) = hot.runner_ci.take() else { panic!("counting from the release") };
    assert_eq!(attempts.len(), checks.len());
    let bytes_without_baseline = crate::stored_bytes(&restored).unwrap();
    let baseline_bytes = u64::try_from(checks.len() * size_of::<api::JobAttempt>()).unwrap();
    assert_eq!(bytes_with_baseline, bytes_without_baseline + baseline_bytes);
    let counted = counts(&attempts, head);
    for (check, attempt) in checks.iter().zip(&attempts) {
        assert_eq!(Some(*attempt), check.job);
        assert_eq!(runs_again(&counted, *attempt), 0);
        assert_eq!(runs_again(&counted, api::JobAttempt { attempt: attempt.attempt + 1, ..*attempt }), 1);
    }
}
