//! The connector's panel follows the core's watch admission and commit barrier.
use jig_domain_core::documents::{Watched, Window};
use jig_domain_people::ApplicationScope;
use skein_lib::{ReplyTo, Token, Wall};
use temper_engine_domain as root;
use temper_engine_domain_forge as forge;
use temper_engine_domain_world::{direct::Driver, fixtures};

const WATCHER: Token = Token::new(700);
const CHANGE: u64 = 51;

fn driver() -> Driver {
    let mut driver = Driver::forge();
    driver.settle();
    driver.sign_in();
    driver.settle();
    driver.send(root::Event::Party(root::PartyInput::Application {
        scope: ApplicationScope::Project(1),
        reply_to: ReplyTo::new(Token::new(95)),
        sign_in: driver.session(),
        key: [95; 16],
        digest: [5; 32],
        carried: None,
        request: temper_engine_domain::Application::Forge(forge::Ask::Adopt(fixtures::adoption(1))),
    }));
    driver.settle();
    driver.send(root::Event::Forge(Box::new(forge::Event::Change {
        row: temper_engine_forge_world::changes::change_row(),
    })));
    driver.settle();
    driver.immediate.clear();
    driver
}
fn watch(driver: &mut Driver, project: u32, task: u64) {
    driver.send(root::Event::Party(root::PartyInput::Watch {
        watcher: WATCHER,
        sign_in: driver.session(),
        subject: Watched::Application {
            scope: ApplicationScope::Project(project),
            connector: 0,
            subject: Token::new(task),
        },
        window: Window { rows: 1, after: None },
    }));
}
fn panels(driver: &Driver) -> Vec<&forge::Panel> {
    driver
        .delivered
        .iter()
        .filter_map(|delivery| {
            if let root::Delivery::Panel { watcher, panel } = delivery {
                assert_eq!(*watcher, WATCHER);
                Some(panel.as_ref())
            } else {
                None
            }
        })
        .collect()
}
#[test]
fn a_project_role_admits_the_watch_and_delivers_the_whole_change_panel() {
    let mut driver = driver();
    watch(&mut driver, 1, CHANGE);
    driver.settle();
    let got = panels(&driver);
    assert_eq!(got.len(), 1);
    let panel = got[0];
    assert_eq!(panel.task, CHANGE);
    assert_eq!(panel.forge, 1);
    assert_eq!(panel.owner.as_ref(), b"org");
    assert_eq!(panel.name.as_ref(), b"repo");
    assert_eq!(panel.branch.as_ref(), b"temper/51");
    assert_eq!(panel.landing.as_ref(), b"main");
    assert_eq!(panel.step, forge::PanelStep::Producing);
    assert_eq!(panel.head, None);
    assert_eq!(panel.pull, None);
    assert_eq!(panel.hold, None);
    assert_eq!(panel.runs, None);
    assert_eq!(panel.ci, temper_engine_domain_forge_change::Status::Unknown);
    assert!(panel.gates.is_empty());
    assert_eq!(panel.repairs, forge::PanelUsed { used: 0, limit: driver.env.limits.forge.change_policy.repairs });
    assert_eq!(
        panel.resolutions,
        forge::PanelUsed { used: 0, limit: driver.env.limits.forge.change_policy.resolutions }
    );
    assert_eq!(panel.updates, forge::PanelUsed { used: 0, limit: driver.env.limits.forge.change_policy.updates });
    assert!(
        !driver
            .immediate
            .iter()
            .any(|request| matches!(request, root::Request::Party(root::PartyRequest::WatchRefused { .. })))
    );
}
#[test]
fn a_party_without_a_role_in_the_scope_is_refused_before_the_connector() {
    let mut driver = driver();
    watch(&mut driver, 2, CHANGE);
    driver.settle();
    assert_eq!(panels(&driver).len(), 0);
    assert!(driver.immediate.iter().any(|request| matches!(request, root::Request::Party(root::PartyRequest::WatchRefused { watcher, why: jig_domain_core::WatchRefusal::NotAllowed }) if *watcher == WATCHER)));
}
#[test]
fn the_panel_stays_held_until_the_asking_decision_is_durable() {
    let mut driver = driver();
    watch(&mut driver, 1, CHANGE);
    driver.settle();
    assert_eq!(panels(&driver).len(), 1);
    driver.send(root::Event::Party(root::PartyInput::Delivered { watcher: WATCHER }));
    driver.settle();
    assert!(driver.store.pending.is_empty());
    // This save asks for the marked document in the same decision.
    driver.send(root::Event::Forge(Box::new(forge::Event::Delegated {
        task: CHANGE,
        child: 73,
        kind: temper_engine_domain_forge_change::Delegate::Produce,
    })));
    for _ in 0..8 {
        driver.advance(false);
    }
    assert_eq!(panels(&driver).len(), 1, "the marked panel is still held");
    assert!(!driver.store.pending.is_empty(), "the asking decision has an unacknowledged commit");
    driver.settle();
    let got = panels(&driver);
    assert_eq!(got.len(), 2);
    assert_eq!(got[1].runs, Some(73));
}
#[test]
fn a_saved_change_marks_the_forges_subject_and_delivers_the_panel_again() {
    let mut driver = driver();
    watch(&mut driver, 1, CHANGE);
    driver.settle();
    assert_eq!(panels(&driver).len(), 1);
    driver.send(root::Event::Party(root::PartyInput::Delivered { watcher: WATCHER }));
    driver.settle();
    driver.send(root::Event::Forge(Box::new(forge::Event::Delegated {
        task: CHANGE,
        child: 73,
        kind: temper_engine_domain_forge_change::Delegate::Produce,
    })));
    driver.settle();
    let got = panels(&driver);
    assert_eq!(got.len(), 2);
    assert_eq!(got[0].runs, None);
    assert_eq!(got[1].runs, Some(73));
}
#[test]
fn a_task_that_is_no_change_ends_the_watch_as_gone() {
    let mut driver = driver();
    watch(&mut driver, 1, 999);
    driver.settle();
    assert_eq!(panels(&driver).len(), 0);
    assert!(driver.delivered.iter().any(|delivery| matches!(delivery, root::Delivery::WatchEnded { watcher, why: jig_domain_views::End::Gone } if *watcher == WATCHER)));
}

#[test]
fn a_project_landings_watch_delivers_its_list_after_the_commit_barrier() {
    for ended in [false, true] {
        let mut driver = driver();
        // A preceding save supplies the durability barrier for this read-only watch.
        driver.send(root::Event::Forge(Box::new(forge::Event::Delegated {
            task: CHANGE,
            child: 73,
            kind: temper_engine_domain_forge_change::Delegate::Produce,
        })));
        watch(&mut driver, 1, forge::landings_subject(1, ended));
        for _ in 0..8 {
            driver.advance(false);
        }
        assert!(
            !driver.delivered.iter().any(|delivery| matches!(delivery, root::Delivery::ForgeDocument { .. })),
            "the whole list remains held until durability"
        );
        driver.settle();
        let documents: Vec<_> = driver
            .delivered
            .iter()
            .filter_map(|delivery| {
                if let root::Delivery::ForgeDocument { watcher, document } = delivery {
                    assert_eq!(*watcher, WATCHER);
                    Some(document.as_ref())
                } else {
                    None
                }
            })
            .collect();
        let [forge::Document::Landings(list)] = documents.as_slice() else { panic!("one whole list: {documents:?}") };
        assert_eq!(list.project, 1);
        assert_eq!(list.turn, None);
        assert!(list.landings.is_empty());
    }
}

#[test]
fn a_project_changes_watch_delivers_its_document_after_the_commit_barrier() {
    let mut driver = driver();
    driver.send(root::Event::Forge(Box::new(forge::Event::Delegated {
        task: CHANGE,
        child: 73,
        kind: temper_engine_domain_forge_change::Delegate::Produce,
    })));
    watch(&mut driver, 1, forge::changes_subject(1));
    for _ in 0..8 {
        driver.advance(false);
    }
    assert!(!driver.delivered.iter().any(|delivery| matches!(delivery, root::Delivery::ForgeDocument { .. })));
    driver.settle();
    let documents: Vec<_> = driver
        .delivered
        .iter()
        .filter_map(|delivery| {
            if let root::Delivery::ForgeDocument { watcher, document } = delivery {
                assert_eq!(*watcher, WATCHER);
                Some(document.as_ref())
            } else {
                None
            }
        })
        .collect();
    let [forge::Document::Changes(view)] = documents.as_slice() else {
        panic!("one whole changes document: {documents:?}")
    };
    assert_eq!(view.project, 1);
    assert_eq!(view.landing.as_ref(), b"main");
    assert!(view.queues.is_empty());
    assert!(view.landed.is_empty());
}
#[test]
fn a_changes_subject_for_another_project_is_gone() {
    let mut driver = driver();
    watch(&mut driver, 1, forge::changes_subject(2));
    driver.settle();
    assert!(driver.delivered.iter().any(|delivery| matches!(delivery, root::Delivery::WatchEnded { watcher, why: jig_domain_views::End::Gone } if *watcher == WATCHER)));
}
#[test]
fn a_changes_watch_without_a_project_role_is_refused() {
    let mut driver = driver();
    watch(&mut driver, 2, forge::changes_subject(2));
    driver.settle();
    assert!(driver.immediate.iter().any(|request| matches!(request, root::Request::Party(root::PartyRequest::WatchRefused { watcher, why: jig_domain_core::WatchRefusal::NotAllowed }) if *watcher == WATCHER)));
}

// The two parties have different sittings in the same project (domain/forge/attention.md, 2).
const OWN_PREVIOUS: Wall = Wall::from_nanos(11);
const OWN_LATEST: Wall = Wall::from_nanos(22);
const OTHER_PREVIOUS: Wall = Wall::from_nanos(33);
const LANDED: u64 = 6;
const OWN_SEEN: u64 = 2;
const OWN_NEXT_SEEN: u64 = 3;
const OTHER_SEEN: u64 = 4;

fn status_driver() -> Driver {
    let mut driver = driver();
    let party = driver
        .delivered
        .iter()
        .find_map(|delivery| {
            if let root::Delivery::WebReply { reply: jig_domain_people::Reply::SignedIn { person, .. }, .. } = delivery
            {
                Some(*person)
            } else {
                None
            }
        })
        .expect("the signed-in watcher has a party");
    let other_party = party.checked_add(1).expect("a second fixture party fits");
    let rows = [
        forge::Stored::ProjectFacts(forge::ProjectFacts {
            project: 1,
            landed: LANDED,
            last_landed: OWN_LATEST,
            brief: None,
            running: None,
        }),
        forge::Stored::Looked(forge::Looked {
            party,
            project: 1,
            previous: Some(OWN_PREVIOUS),
            latest: OWN_LATEST,
            seen: OWN_SEEN,
            next_seen: OWN_NEXT_SEEN,
        }),
        forge::Stored::Looked(forge::Looked {
            party: other_party,
            project: 1,
            previous: Some(OTHER_PREVIOUS),
            latest: OWN_LATEST,
            seen: OTHER_SEEN,
            next_seen: OTHER_SEEN,
        }),
    ];
    driver.store.seed(rows.map(|row| root::Record::Forge { row: Box::new(row) }).into(), Box::new([]));
    let limits = driver.env.limits;
    let store = driver.take_store();
    let now =
        Wall::from_nanos(OWN_LATEST.as_nanos().checked_add(forge::SITTING.as_nanos()).expect("the next sitting fits"));
    let mut driver = Driver::from_store_at(store, temper_engine_domain_world::walking::config(71), &limits, now);
    driver.settle();
    driver.sign_in();
    driver.settle();
    driver
}
fn statuses(driver: &Driver) -> Vec<&forge::StatusView> {
    driver
        .delivered
        .iter()
        .filter_map(|delivery| {
            if let root::Delivery::ForgeDocument { watcher, document } = delivery
                && let forge::Document::Status(view) = document.as_ref()
            {
                assert_eq!(*watcher, WATCHER);
                Some(view)
            } else {
                None
            }
        })
        .collect()
}
#[test]
fn a_status_document_is_the_watchers_own() {
    let mut driver = status_driver();
    watch(&mut driver, 1, forge::status_subject(1));
    driver.settle();
    let got = statuses(&driver);
    assert_eq!(got.len(), 1);
    assert_eq!(
        got[0],
        &forge::StatusView {
            project: 1,
            landed: LANDED,
            last_landed: Some(OWN_LATEST),
            brief: None,
            previous: Some(OWN_PREVIOUS),
            seen: OWN_SEEN,
            queued: 0,
            red: false,
            branch: b"main".as_slice().into(),
        }
    );
}
#[test]
fn a_status_subject_for_another_project_is_gone() {
    let mut driver = driver();
    watch(&mut driver, 1, forge::status_subject(2));
    driver.settle();
    assert_eq!(statuses(&driver).len(), 0);
    assert!(driver.delivered.iter().any(|delivery| matches!(delivery,
        root::Delivery::WatchEnded { watcher, why: jig_domain_views::End::Gone } if *watcher == WATCHER)));
}
#[test]
fn a_status_watch_without_a_project_role_is_refused() {
    let mut driver = driver();
    watch(&mut driver, 2, forge::status_subject(2));
    driver.settle();
    assert_eq!(statuses(&driver).len(), 0);
    assert!(driver.immediate.iter().any(|request| matches!(request,
        root::Request::Party(root::PartyRequest::WatchRefused {
            watcher, why: jig_domain_core::WatchRefusal::NotAllowed,
        }) if *watcher == WATCHER)));
}
#[test]
fn a_look_remakes_the_document() {
    let mut driver = status_driver();
    watch(&mut driver, 1, forge::status_subject(1));
    driver.settle();
    driver.send(root::Event::Party(root::PartyInput::Delivered { watcher: WATCHER }));
    driver.settle();
    driver.send(root::Event::Party(root::PartyInput::Application {
        scope: ApplicationScope::Own,
        reply_to: ReplyTo::new(Token::new(96)),
        sign_in: driver.session(),
        key: [96; 16],
        digest: [6; 32],
        carried: None,
        request: root::Application::Looked { project: 1 },
    }));
    // The look's answer and remade document both wait for its saved row.
    for _ in 0..8 {
        driver.advance(false);
    }
    assert_eq!(statuses(&driver).len(), 1);
    assert!(!driver.delivered.iter().any(|delivery| matches!(
        delivery,
        root::Delivery::Application { outcome: Some(root::ApplicationAnswer::Looked(_)), .. }
    )));
    driver.settle();
    let got = statuses(&driver);
    assert_eq!(got.len(), 2);
    assert_eq!((got[1].previous, got[1].seen), (Some(OWN_LATEST), OWN_NEXT_SEEN));
    assert!(driver.delivered.iter().any(|delivery| matches!(
        delivery,
        root::Delivery::Application { outcome: Some(root::ApplicationAnswer::Looked(forge::LookedAnswer::Looked)), .. }
    )));
}

const PROJECT: u32 = 1;
const OTHER_IDENTITY: u64 = 8;
const OWN_DECISION: u32 = 1;
const OTHER_DECISION: u32 = 2;
const AWAY_UNTIL: Wall = Wall::from_nanos(1000);
// Root turns for admission, the forge save, and its held answer.
const UNCOMMITTED_STEPS: usize = 8;

fn panel_sign_in(driver: &mut Driver, identity: u64) -> (u64, u64) {
    driver.serial += 1;
    let reply = Token::new(driver.serial);
    driver.send(root::Event::Party(root::PartyInput::SignedIn {
        held: None,
        reply_to: ReplyTo::new(reply),
        identity: jig_domain_people::Identity {
            key: temper_engine_domain_forge::user_key(1, identity),
            login: b"person".as_slice().into(),
            name: b"Person".as_slice().into(),
        },
    }));
    driver.settle();
    driver
        .delivered
        .iter()
        .find_map(|delivery| match delivery {
            root::Delivery::WebReply { to, reply: jig_domain_people::Reply::SignedIn { person, sign_in, .. } }
                if *to == ReplyTo::new(reply) =>
            {
                Some((*person, *sign_in))
            }
            _ => None,
        })
        .expect("the fixture identity signed in")
}
fn restarted_panels(driver: &mut Driver) -> Driver {
    let limits = driver.env.limits;
    let wall = driver.env.wall;
    let store = driver.take_store();
    let mut driver = Driver::from_store_at(store, temper_engine_domain_world::walking::config(71), &limits, wall);
    driver.settle();
    driver
}
fn parties_driver() -> (Driver, [(u64, u64); 2]) {
    let mut driver = driver();
    let first = driver
        .delivered
        .iter()
        .find_map(|delivery| match delivery {
            root::Delivery::WebReply { reply: jig_domain_people::Reply::SignedIn { person, sign_in, .. }, .. } => {
                Some((*person, *sign_in))
            }
            _ => None,
        })
        .expect("the owner signed in");
    let second = panel_sign_in(&mut driver, OTHER_IDENTITY);
    let (party, _) = first;
    let (other, _) = second;
    let mut rows = vec![root::Record::Core(jig_domain_core::Record::People(jig_domain_people::Stored::Roles {
        project: PROJECT,
        holdings: Box::new([
            jig_domain_people::Holding { person: party, role: jig_domain_people::Role::Owner },
            jig_domain_people::Holding { person: other, role: jig_domain_people::Role::Observer },
        ]),
    }))];
    for (party, number, took) in [(party, OWN_DECISION, 0), (other, OTHER_DECISION, 1)] {
        rows.push(root::Record::Forge {
            row: Box::new(forge::Stored::Decided(forge::Decided {
                project: PROJECT,
                number,
                party,
                task: CHANGE,
                at: OWN_LATEST,
                question: b"Which?".as_slice().into(),
                options: Box::new([b"First".as_slice().into(), b"Second".as_slice().into()]),
                took: Some(took),
                why: b"Fits".as_slice().into(),
                overturned: false,
                reach: temper_protocol_reaches::Reach {
                    kind: temper_protocol_reaches::Kind::Wording,
                    class: temper_protocol_reaches::Class::Decide,
                    reason: temper_protocol_reaches::Reason::Kind,
                    under: 0,
                    sets: 0,
                    matched: 0,
                    away: false,
                },
            })),
        });
    }
    driver.store.seed(rows.into_boxed_slice(), Box::new([]));
    let mut driver = restarted_panels(&mut driver);
    driver.delivered.clear();
    (driver, [first, second])
}
fn panel_application(driver: &mut Driver, sign_in: u64, key: u8, scope: ApplicationScope, request: root::Application) {
    driver.send(root::Event::Party(root::PartyInput::Application {
        scope,
        reply_to: ReplyTo::new(Token::new(u64::from(key))),
        sign_in,
        key: [key; 16],
        digest: [key; 32],
        carried: None,
        request,
    }));
}
fn panel_watch(driver: &mut Driver, sign_in: u64, watcher: Token, scope: ApplicationScope, subject: u64) {
    driver.send(root::Event::Party(root::PartyInput::Watch {
        watcher,
        sign_in,
        subject: Watched::Application { scope, connector: 0, subject: Token::new(subject) },
        window: Window { rows: 1, after: None },
    }));
    driver.settle();
}
fn party_documents(driver: &Driver, watcher: Token) -> Vec<&forge::Document> {
    driver
        .delivered
        .iter()
        .filter_map(|delivery| match delivery {
            root::Delivery::ForgeDocument { watcher: got, document } if *got == watcher => Some(document.as_ref()),
            _ => None,
        })
        .collect()
}
fn panel_answers(driver: &Driver) -> Vec<Option<root::ApplicationAnswer>> {
    driver
        .delivered
        .iter()
        .filter_map(|delivery| match delivery {
            root::Delivery::Application { outcome, .. } => Some(outcome.clone()),
            _ => None,
        })
        .collect()
}
fn set_away(on: bool) -> root::Application {
    root::Application::SetAway { project: PROJECT, on, until: Some(AWAY_UNTIL) }
}
#[test]
fn a_partys_away_is_its_own() {
    const KEY: u8 = 120;
    let (mut driver, parties) = parties_driver();
    for ((_, sign_in), on) in parties.into_iter().zip([true, false]) {
        panel_application(&mut driver, sign_in, KEY, ApplicationScope::Project(2), set_away(on));
        // A new row's answer cannot leave before its save commits.
        for _ in 0..UNCOMMITTED_STEPS {
            driver.advance(false);
        }
        assert_eq!(panel_answers(&driver).len(), 0);
        assert!(!driver.store.pending.is_empty());
        driver.settle();
        assert_eq!(panel_answers(&driver), [Some(root::ApplicationAnswer::SetAway(forge::SetAwayAnswer::Away))]);
        driver.delivered.clear();
    }
    for (((party, sign_in), on), watcher) in parties.into_iter().zip([true, false]).zip([WATCHER, Token::new(701)]) {
        panel_watch(&mut driver, sign_in, watcher, ApplicationScope::Own, forge::AWAY_SUBJECT);
        assert_eq!(
            party_documents(&driver, watcher),
            [&forge::Document::Aways(forge::AwaysView {
                rows: Box::new([forge::Away {
                    party,
                    project: PROJECT,
                    since: Wall::EPOCH,
                    until: Some(AWAY_UNTIL),
                    landed: 0,
                    off: if on { None } else { Some(Wall::EPOCH) }
                }]),
            })]
        );
    }
    let mut driver = restarted_panels(&mut driver);
    for ((_, sign_in), on) in parties.into_iter().zip([true, false]) {
        panel_application(&mut driver, sign_in, KEY, ApplicationScope::Project(2), set_away(on));
        driver.settle();
    }
    assert_eq!(panel_answers(&driver), [None, None], "lost volatile outcomes stay unknown");
}
#[test]
fn decided_is_the_asking_partys_own() {
    let (mut driver, [(_, own), (_, other)]) = parties_driver();
    let other_watcher = Token::new(701);
    for (sign_in, watcher, number, took) in [
        (own, WATCHER, OWN_DECISION, b"First".as_slice()),
        (other, other_watcher, OTHER_DECISION, b"Second".as_slice()),
    ] {
        panel_watch(&mut driver, sign_in, watcher, ApplicationScope::Project(PROJECT), forge::decided_subject(PROJECT));
        assert_eq!(
            party_documents(&driver, watcher),
            [&forge::Document::Decisions(forge::DecisionsView {
                project: PROJECT,
                rows: Box::new([forge::DecisionView {
                    number,
                    task: CHANGE,
                    at: OWN_LATEST,
                    question: b"Which?".as_slice().into(),
                    took: took.into(),
                    why: b"Fits".as_slice().into(),
                    overturned: false,
                    reach: temper_protocol_reaches::Reach {
                        kind: temper_protocol_reaches::Kind::Wording,
                        class: temper_protocol_reaches::Class::Decide,
                        reason: temper_protocol_reaches::Reason::Kind,
                        under: 0,
                        sets: 0,
                        matched: 0,
                        away: false
                    },
                }]),
                asked: Box::new([]),
                cut: false,
            })]
        );
        driver.send(root::Event::Party(root::PartyInput::Delivered { watcher }));
        driver.settle();
    }
    let request = root::Application::Overturn { project: PROJECT, number: OWN_DECISION };
    for (sign_in, key, expected) in [
        (other, 121, forge::OverturnAnswer::Unknown),
        (own, 122, forge::OverturnAnswer::Overturned),
        (own, 123, forge::OverturnAnswer::Already),
    ] {
        driver.delivered.clear();
        panel_application(&mut driver, sign_in, key, ApplicationScope::Project(2), request.clone());
        driver.settle();
        assert_eq!(panel_answers(&driver), [Some(root::ApplicationAnswer::Overturn(expected))]);
    }
    let mut driver = restarted_panels(&mut driver);
    panel_application(&mut driver, own, 122, ApplicationScope::Project(2), request);
    driver.settle();
    assert_eq!(panel_answers(&driver), [None], "the request survives its volatile answer");
}
#[test]
fn a_party_without_a_role_is_refused_at_admission() {
    let mut driver = driver();
    let sign_in = driver.session();
    for (key, request) in [
        (124, root::Application::SetAway { project: 2, on: true, until: None }),
        (125, root::Application::Overturn { project: 2, number: OWN_DECISION }),
    ] {
        driver.delivered.clear();
        panel_application(&mut driver, sign_in, key, ApplicationScope::Project(PROJECT), request);
        driver.settle();
        assert_eq!(
            driver.delivered,
            [root::Delivery::WebReply {
                to: ReplyTo::new(Token::new(u64::from(key))),
                reply: jig_domain_people::Reply::Outcome(jig_domain_people::Outcome::Refused(
                    jig_domain_people::Refusal::Role { held: None },
                )),
            }]
        );
    }
    panel_watch(&mut driver, sign_in, WATCHER, ApplicationScope::Project(2), forge::decided_subject(2));
    assert_eq!(party_documents(&driver, WATCHER).len(), 0);
    assert!(driver.immediate.iter().any(|request| matches!(request,
        root::Request::Party(root::PartyRequest::WatchRefused {
            watcher, why: jig_domain_core::WatchRefusal::NotAllowed,
        }) if *watcher == WATCHER)));
}

#[test]
fn a_service_is_refused_device_registration_at_its_own_admission() {
    let mut driver = driver();
    let session = driver.session();
    // Keep the sign-in and role rows, with a deployment-authenticated party.
    driver.store.seed(
        Box::new([root::Record::Core(jig_domain_core::Record::People(jig_domain_people::Stored::Person {
            number: 1,
            identity: jig_domain_people::Identity {
                key: jig_domain_people::IdentityKey { provider: 1, subject: b"push-service".as_slice().into() },
                login: b"service".as_slice().into(),
                name: b"Service".as_slice().into(),
            },
        }))]),
        Box::new([]),
    );
    let mut driver = restarted_panels(&mut driver);
    driver.delivered.clear();
    panel_application(
        &mut driver,
        session,
        121,
        ApplicationScope::Deployment,
        root::Application::AddDevice(forge::AddDevice {
            endpoint: b"http://push.test:4000/push/service".as_slice().into(),
            p256dh: [4; 65],
            auth: [7; 16],
            label: b"service".as_slice().into(),
        }),
    );
    driver.settle();
    assert_eq!(
        panel_answers(&driver),
        [Some(root::ApplicationAnswer::Device(forge::DeviceAnswer::Refused(forge::DeviceRefusal::Person)))]
    );
}
