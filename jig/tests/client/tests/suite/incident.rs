//! The two independent incident journeys of pages.md, section 10.
use jig_client_domain::contract::*;
use jig_client_scripted_engine::{Change, Rows, Source};
use jig_client_test_domain::{Address as RootAddress, family};
use jig_client_view::address::{Address, TaskSection};
use jig_client_world::{Scripted, Test, Until, World, stories};
use skein_fake_person::{Find, Step};
use skein_lib::{Duration, Wall};
use skein_ui::Role;
const WITHIN: Duration = Duration::from_secs(1);
fn see(words: &[u8]) -> Step<RootAddress> {
    Step::See { find: Find::text(words), within: WITHIN }
}
fn press(name: &[u8]) -> Step<RootAddress> {
    Step::Press { find: Find::named(Role::Button, name) }
}
fn finish(w: &mut World<Test, Scripted>, withdrawn: bool) {
    let now = w.now();
    let mut state = w.front().engine.application().clone();
    let subject = family::Subject { scope: family::Scope::Project(1), kind: family::Kind::Item(1) };
    let family::Document::Connector(item) =
        state.documents.get_mut(&subject).expect("incident fixture fits and exists");
    let jig_client_test_domain::connector_values::ClientDocument::Object { now: object, .. } = item.value.as_mut()
    else {
        panic!("incident object")
    };
    object.revision += 1;
    let mut delegate = stories::task();
    delegate.chip.task = 10;
    delegate.chip.title = jig_client_domain::contract::DocumentText::complete(stories::text(b"Delegate ten"));
    let lead = stories::reported(stories::task(), b"Incident resolved");
    let delegate = stories::reported(delegate, if withdrawn { b"Already healthy by hand" } else { b"Service healthy" });
    let p = stories::proposal();
    let proposal = stories::decided_proposal(
        &p,
        if withdrawn {
            OwnerProposalHistoryState::Withdrawn
        } else {
            OwnerProposalHistoryState::Accepted { by: OwnerRequester::Person(7) }
        },
    );
    w.front()
        .engine
        .act_application(
            now,
            Wall::EPOCH,
            jig_client_test_domain::FIRST,
            1,
            state,
            vec![
                Change::Object { project: 1, key: ObjectKey::Task(9), value: ReadValue::Task(Box::new(lead)) },
                Change::Object { project: 1, key: ObjectKey::Task(10), value: ReadValue::Task(Box::new(delegate)) },
                Change::Object { project: 1, key: proposal.key, value: ReadValue::Historical(Box::new(proposal)) },
                Change::Source(Source {
                    party: 7,
                    read: Read::Inbox {
                        group: jig_client_domain::contract::InboxReadGroup::Needs,
                        project: None,
                        kind: None,
                        goal: None,
                        after: None,
                    },
                    rows: Rows::Inbox(vec![]),
                }),
            ],
        )
        .expect("resolved incident is one durable application and jig commit");
}
#[test]
fn an_incident_delegates_accepts_on_the_leads_plan_and_resolves_with_its_report() {
    resolved_incident(51);
}

#[test]
fn an_incident_delegates_accepts_on_the_leads_plan_and_resolves_with_its_report_with_a_second_seed() {
    resolved_incident(52);
}

fn resolved_incident(seed: u64) {
    let mut w = stories::incident_world(seed);
    let c = w.client(7, stories::incident());
    let inbox = w.client(7, stories::inbox());
    w.person(
        c,
        vec![
            see(b"Object: 1"),
            see(b"Chat nine"),
            Step::Go { address: stories::chat() },
            see(b"delegate"),
            Step::Go { address: RootAddress::Jig(Address::Task { task: 9, section: TaskSection::Plan }) },
            see(b"Delegate ten"),
            see(b"Resolve the incident"),
            press(b"Accept"),
            press(b"Confirm"),
            see(b"Accepted"),
            Step::Go { address: RootAddress::Jig(Address::Task { task: 10, section: TaskSection::Panels }) },
            see(b"Result code: 31"),
        ],
    );
    w.person(inbox, vec![see(b"Resolve the incident"), see(b"Accepted")]);
    w.run(Until::Done);
    let now = w.now();
    let mut state = w.front().engine.application().clone();
    let panel = family::Subject { scope: family::Scope::Project(1), kind: family::Kind::Panel(10) };
    let family::Document::Connector(part) = state.documents.get_mut(&panel).expect("incident fixture fits and exists");
    let jig_client_test_domain::connector_values::ClientDocument::Panel { result, .. } = part.value.as_mut() else {
        panic!("incident panel")
    };
    *result = Some(32);
    w.front()
        .engine
        .act_application(now, Wall::EPOCH, jig_client_test_domain::FIRST, 1, state, vec![])
        .expect("incident fixture fits and exists");
    w.person(c, vec![see(b"Result code: 32")]);
    w.run(Until::Done);
    finish(&mut w, false);
    w.person(c, vec![Step::Go { address: stories::incident() }, see(b"Incident resolved")]);
    w.run(Until::Done);
    assert_eq!(w.front().engine.commits().iter().filter(|c| c.key.is_some()).count(), 1);
}

#[test]
fn an_incident_proposal_withdrawn_while_waiting_leaves_with_its_delegates_reason() {
    for seed in [53, 54] {
        let mut w = stories::incident_world(seed);
        let c = w.client(7, stories::inbox());
        w.person(c, vec![see(b"Resolve the incident")]);
        w.run(Until::Done);
        finish(&mut w, true);
        w.person(
            c,
            vec![
                see(b"Withdrawn"),
                Step::Go { address: RootAddress::Jig(Address::Task { task: 10, section: TaskSection::Overview }) },
                see(b"Already healthy by hand"),
            ],
        );
        w.run(Until::Done);
        assert!(w.front().engine.commits().iter().all(|c| c.key.is_none()));
    }
}
