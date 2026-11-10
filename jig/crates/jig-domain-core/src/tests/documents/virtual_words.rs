//! A run woken by a word that never enters its inbox, a held delegate's escalation,
//! prepares as current (domain/engine/executors.md).
use crate::tests::*;
use jig_domain_link::{Form, Scheme};

const REQUESTER: u64 = 1;
const HELD: u64 = 3;
const ENTRY: u64 = 1;

#[test]
fn a_held_delegates_escalation_prepares_its_requesters_next_run() {
    let (mut core, limits) = make_tree();
    core.opened = true;
    runs::run_choice::host(&mut core, &limits, Token::new(7), &[b"scripted"]);
    watch_outputs(&mut core, &limits, Event::Account(accounts::Event::Want));
    watch_outputs(
        &mut core,
        &limits,
        Event::Account(accounts::Event::Component(accounts::FromComponent::Granted {
            form: Form::Credential(Scheme::Bearer),
            account: 1,
            generation: 1,
            valid: Duration::from_secs(60),
        })),
    );
    let revision = core.tasks.task(HELD).unwrap().revisions.hold;
    send_task(
        &mut core,
        &limits,
        tasks::Event::RoutedEscalation {
            task: HELD,
            revision,
            holder: tasks::EscalationHolder::Task(REQUESTER),
            entry: ENTRY,
        },
    );
    let context = core.tasks.run_context(REQUESTER).unwrap();
    // The escalation is the context's newest word, never the row's.
    assert_eq!(context.last_message, ENTRY);
    assert_eq!(core.tasks.task(REQUESTER).unwrap().last_message, 0);
    assert_eq!(core.tasks.run_message(REQUESTER), Some(ENTRY));
    let out = watch_outputs(&mut core, &limits, Event::Activate { context, ready: true, busy: false });
    let row = core.tasks.task(REQUESTER).unwrap();
    assert_eq!(row.phase, tasks::Phase::Active(tasks::Active::Preparing));
    assert_eq!(row.retry_notice, None);
    assert!(core.contexts.contains_key(&REQUESTER));
    let mut notes = false;
    for request in out {
        if let Request::Held(held) = request
            && let Held::NotesLoad { .. } = *held
        {
            notes = true;
        }
    }
    assert!(notes, "the brief goes on to its notes");
}
