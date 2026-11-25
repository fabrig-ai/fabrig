//! The root holds a closing task with the forge's release account (domain/forge/releases.md, 2).
use super::{adopted, change_start_fixture, restore_forge, task_change_row, task_event};
use crate::{
    Decision, Work,
    route::forge_route::{forge, tasks},
};
use alloc::boxed::Box;
use skein_lib::{Decimal, Queue, bytes};

#[test]
fn release_failure_holds_with_the_forge_words_and_optional_entry() {
    for (entry, account) in [
        (Some(90), b"close item: forge refused permission; resource org/repo #1".as_slice()),
        (None, b"close item: not sent: the repository is context; resource org/repo #1".as_slice()),
    ] {
        let (mut domain, env, context) = change_start_fixture();
        let task = context.task;
        let mut record = domain.core.tasks.task(task).unwrap().clone();
        let ledger = *domain.core.tasks.funding(record.funder).unwrap();
        record.phase = tasks::Phase::Closing(tasks::Closing {
            stage: tasks::Stage::Releases,
            ending: tasks::Ending::Failed { reason: Box::from(b"failed".as_slice()) },
        });
        domain.core.tasks =
            crate::Domain::new(crate::tests::resume_fixture::config(7), &env.limits).unwrap().core.tasks;
        for event in [
            tasks::Event::Restore { record: tasks::Stored::Live(Box::new(record)) },
            tasks::Event::Restore { record: tasks::Stored::Ledger(ledger) },
            tasks::Event::Check,
            tasks::Event::Restored,
        ] {
            task_event(&mut domain, &env, event);
        }
        assert_eq!(domain.core.tasks.restore_failure(), None);
        let mut row = task_change_row(task, Some(1));
        row.repository = adopted(1).provider;
        restore_forge(&mut domain, &env, forge::Stored::Change(Box::new(row)));

        let mut out = Queue::uncapped();
        out.push(forge::Request::ReleaseFailed { task, entry, words: Box::from(account) });
        let mut decision = Decision::decide(&mut domain.journal, &env.limits.journal);
        super::super::outputs(&mut domain, &env, &mut decision, &mut out);
        let Some(Work::Tasks(event)) = domain.work.pop() else { panic!("the root asks for a task hold") };
        assert!(domain.work.is_empty());
        task_event(&mut domain, &env, event);
        let tasks::Phase::Held { was: tasks::Was::Closing(_), why: tasks::Hold::EffectFailed, source: Some(source) } =
            &domain.core.tasks.task(task).unwrap().phase
        else {
            panic!("failed effect holds the close")
        };
        let tasks::HoldSource::Account { connector, entry: kept, words } = source.as_ref() else {
            panic!("release account")
        };
        assert_eq!(*connector, domain.config.forge_connector);
        assert_eq!(*kept, entry);
        if entry.is_some() {
            assert_eq!(words.as_ref(), account, "the outbox account is kept verbatim");
        } else {
            assert!(words.starts_with(account), "the account comes from the forge");
            let tail = words.get(account.len()..).unwrap();
            let prefix = bytes::concat(&[
                b" for branch temper/7 in org/repo; resolve this reason, then task release ",
                Decimal::of(task).as_bytes(),
                b" --hold ",
            ]);
            assert!(tail.starts_with(&prefix), "{tail:?}");
        }
    }
}
