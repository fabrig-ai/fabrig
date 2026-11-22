//! A director's live wait and its delegate's report (testing/checks.md, 1).
use super::*;
use jig_domain_core::{documents, reads};

fn delegated() -> (Driver, engine::Assignment, engine::Assignment) {
    let (mut driver, director) = batch_fixture_with(2, 1);
    let a = call_batch(&mut driver, &director, 940, Box::new([report_delegate(b"Own the scope", Box::new([]))]))[0];
    let a = assigned_task(&driver, a);
    driver.send(protocol::up(
        Token::new(7),
        host::LinkUp::Activity {
            run: Token::new(director.task),
            attempt: Token::new(director.attempt),
            activity: host::AgentActivity::Phase(jig_domain_link::Phase::Waiting),
        },
        engine::Parts::none(),
    ));
    driver.settle();
    (driver, director, a)
}

#[test]
fn a_director_waits_in_its_run_with_a_live_delegate() {
    let (mut driver, director, a) = delegated();
    driver.send(engine::Event::Party(engine::PartyInput::Read {
        reply_to: ReplyTo::new(Token::new(941)),
        sign_in: driver.session(),
        read: Box::new(reads::Read::Task { task: director.task }),
    }));
    driver.pump();
    let document = driver
        .delivered
        .iter()
        .rev()
        .find_map(|delivery| match delivery {
            Delivery::Read { reply, answer: reads::Answer::Document(document) }
                if *reply == ReplyTo::new(Token::new(941)) =>
            {
                Some(document)
            }
            _ => None,
        })
        .expect("task read");
    let documents::Document::Task(document) = document.as_ref() else { panic!("task document") };
    assert_eq!(document.chip.details.observation.as_ref().expect("live activity").work, documents::ChipWork::Waiting);
    assert_eq!(document.chip.phase, documents::PhaseLine::Active(tasks::Active::Running { attempt: director.attempt }));
    assert!(document.delegates.iter().any(|delegate| delegate.chip.number == a.task));
    assert!(matches!(live(&driver, director.task).phase,
        tasks::Phase::Active(tasks::Active::Running { attempt }) if attempt == director.attempt));
    // End the volatile wait before the driver's cold-restart comparison.
    driver.send(protocol::up(
        Token::new(7),
        host::LinkUp::Activity {
            run: Token::new(director.task),
            attempt: Token::new(director.attempt),
            activity: host::AgentActivity::Phase(jig_domain_link::Phase::Active),
        },
        engine::Parts::none(),
    ));
    driver.settle();
}

#[test]
fn a_waiting_director_and_its_delegate_end_with_their_reports() {
    let (mut driver, director, a) = delegated();
    for (run, words) in [(&a, b"Scope reported".as_slice()), (&director, b"Direction reported".as_slice())] {
        finish_task(&mut driver, run, tasks::TaskResult::Report { words: words.into() });
        let Some(Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Ended(row)))) =
            driver.store.rows.get(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Ended(run.task))))
        else {
            panic!("original task ended")
        };
        assert_eq!(
            row.phase,
            tasks::Phase::Ended(tasks::Ending::Done(tasks::TaskResult::Report { words: words.into() }))
        );
    }
}
