use jig_channel_world::usage_read::{self, Case, READ};
use jig_protocol_worker_wire as wire;

#[test]
fn a_worker_reads_two_usage_windows_without_a_run() {
    let run = usage_read::run(Case::Read);
    assert_eq!(run.answers.len(), 1);
    assert_eq!(run.answers[0].read(), READ);
    let wire::UsageOutcome::Read(windows) = run.answers[0].outcome() else { panic!("{:?}", run.answers) };
    assert_eq!(windows.windows().len(), 2);
    let first = windows.windows().get(0).unwrap();
    assert_eq!(first.label(), b"five_hour");
    assert_eq!(*first.length(), Some(skein_lib::Duration::from_secs(18_000)));
    assert_eq!(*first.used(), Some(3450));
    let reset = |day, seconds| {
        let midnight = (skein_lib::Civil { year: 2026, month: 11, day }).wall().expect("fixture date");
        midnight.as_nanos().checked_add(skein_lib::Duration::from_secs(seconds).as_nanos()).expect("fixture reset")
    };
    assert_eq!(*first.resets(), Some(reset(19, 4 * 3600 + 30 * 60)));
    assert_eq!(first.state(), 1);
    let second = windows.windows().get(1).unwrap();
    assert_eq!(second.label(), b"seven_day");
    assert_eq!(*second.length(), Some(skein_lib::Duration::from_secs(604_800)));
    assert_eq!(*second.used(), Some(10_000));
    assert_eq!(*second.resets(), Some(reset(23, 5 * 3600)));
    assert_eq!(second.state(), 3);
    assert_eq!(run.observations.iter().filter(|row| row.arguments.iter().any(|word| word == "/usage")).count(), 1);
}
#[test]
fn a_usage_read_without_a_configuration_directory_starts_no_program() {
    let run = usage_read::run(Case::Missing);
    assert_eq!(run.answers.len(), 1);
    assert_eq!(run.answers[0].read(), READ);
    assert!(matches!(run.answers[0].outcome(), wire::UsageOutcome::NotSignedIn));
    assert!(!run.usage_started);
    assert!(run.observations.is_empty(), "the empty initial source inventory starts no program");
}
#[test]
fn a_spent_usage_read_reaches_the_engine_as_spent() {
    let run = usage_read::run(Case::Spent);
    assert_eq!(run.answers.len(), 1);
    assert_eq!(run.answers[0].read(), READ);
    assert!(matches!(run.answers[0].outcome(), wire::UsageOutcome::Spent));
}
#[test]
fn a_usage_read_lost_while_running_sends_nothing_after_reconnect() {
    let run = usage_read::run(Case::Lost);
    assert!(run.usage_started);
    assert!(run.reconnected);
    assert_eq!(run.answers, []);
    assert_eq!(run.observations.iter().filter(|row| row.arguments.iter().any(|word| word == "/usage")).count(), 1);
}
#[test]
fn usage_and_list_reads_wait_for_sign_out_and_precede_the_queued_check() {
    let run = usage_read::run(Case::Ordered);
    let records: Vec<_> = run
        .records
        .into_iter()
        .filter(|kind| {
            [
                wire::Kind::Logins.number(),
                wire::Kind::SignedOut.number(),
                wire::Kind::UsageRead.number(),
                wire::Kind::ListRead.number(),
            ]
            .contains(kind)
        })
        .collect();
    assert_eq!(
        records,
        [
            wire::Kind::SignedOut.number(),
            wire::Kind::UsageRead.number(),
            wire::Kind::ListRead.number(),
            wire::Kind::Logins.number()
        ]
    );
    assert_eq!(
        run.observations.iter().map(|row| row.arguments.first().map(String::as_str)).collect::<Vec<_>>(),
        [Some("auth"), Some("auth"), Some("-p"), Some("auth")]
    );
}
