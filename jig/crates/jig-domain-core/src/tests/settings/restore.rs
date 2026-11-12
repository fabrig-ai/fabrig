//! Restore requests through their paged loads and connector continuations (domain/engine/settings.md, 10).
#![expect(
    clippy::wildcard_enum_match_arm,
    clippy::match_like_matches_macro,
    reason = "tests select the boundary output under assertion"
)]
use crate::reads::{Loaded, Range, Row, Scope};
use crate::settings::{self as s, application as app};
use crate::tests::*;

fn fixture(newest: u32) -> (Core, Limits) {
    fixture_with_count(newest, 2)
}
fn fixture_with_count(newest: u32, count: u32) -> (Core, Limits) {
    let (owner, mut limits) = core();
    limits.people.projects = 3;
    limits.people.requests = 16;
    limits.tasks.charters = 2;
    limits.load_slots = 1;
    let mut configured = config(41);
    configured.seed_projects[0].policy.permissions = Box::default();
    configured.kinds = Box::new([
        s::Kind {
            number: 1,
            scope: s::KindScope::Deployment,
            connector: 1,
            value_bytes: 64,
            per_scope: count,
            charters: 1,
        },
        s::Kind {
            number: 2,
            scope: s::KindScope::Project,
            connector: 1,
            value_bytes: 64,
            per_scope: count,
            charters: 1,
        },
    ]);
    let mut core = seeded_core(configured, &limits).unwrap();
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.people };
    for record in [
        people::Stored::Person { number: 1, identity: owner.people.identity(1).unwrap().clone() },
        people::Stored::SignIn { number: 10, person: 1, expires: Wall::from_nanos(Duration::from_secs(60).as_nanos()) },
        people::Stored::Roles {
            project: 0,
            holdings: Box::new([people::Holding { person: 1, role: people::Role::Owner }]),
        },
        people::Stored::Roles {
            project: 1,
            holdings: Box::new([people::Holding { person: 1, role: people::Role::Owner }]),
        },
    ] {
        people::step(&mut core.people, &env, people::Event::Restore { record }, &mut Queue::uncapped());
    }
    finish_people(&mut core.people, &env);
    send_task(&mut core, &limits, tasks::Event::Check);
    send_task(&mut core, &limits, tasks::Event::Restored);
    core.current.head.number = newest;
    core.current.oldest = 1;
    (core, limits)
}
fn send(core: &mut Core, limits: &Limits, event: Event) -> Box<[Request]> {
    let mut out = step(core, &Env { now: Time::ZERO, wall: Wall::EPOCH, limits: *limits }, event);
    let mut result = List::with_capacity(256);
    while let Some(request) = out.pop() {
        result.push(request).unwrap();
    }
    core.reclaim();
    result.into_boxed()
}
fn ask(core: &mut Core, limits: &Limits, revision: u32, key: u8) -> Box<[Request]> {
    send(
        core,
        limits,
        Event::People(people::Event::Ask {
            reply_to: ReplyTo::new(Token::new(u64::from(key))),
            sign_in: 10,
            key: [key; 16],
            models: Box::default(),
            ask: people::Ask::Restore { revision, reason: b"review".as_slice().into() },
        }),
    )
}
fn outcome(output: &[Request]) -> people::Outcome {
    for request in output {
        if let Request::Held(held) = request
            && let Held::PeopleReply { reply: people::Reply::Outcome(outcome), .. } = held.as_ref()
        {
            return outcome.clone();
        }
    }
    panic!("keyed outcome: {output:?}")
}
fn load(output: &[Request]) -> (Token, Range) {
    for request in output {
        if let Request::Held(held) = request
            && let Held::ReadLoad { load, range } = held.as_ref()
        {
            return (*load, *range.clone());
        }
    }
    panic!("restore page: {output:?}")
}
fn record(
    core: &Core,
    number: u32,
    items: Box<[s::RevisionItem]>,
    applications: Box<[app::RevisionRecorded]>,
) -> s::Revision {
    let mut head = core.current.head.clone();
    head.number = number;
    head.against = number.checked_sub(1).unwrap();
    s::Revision { head, items, applications }
}
fn item(setting: s::Setting, replaced: s::Replaced) -> s::RevisionItem {
    s::RevisionItem { setting, replaced, edit: s::Edit::Reset }
}
fn charter(core: &Core, number: u32, words: &[u8]) -> s::Value {
    let mut charter = core.current.charter(1).unwrap().clone();
    charter.number = number;
    charter.policy.instructions = words.into();
    s::Value::Charter(Box::new(charter))
}
fn page(core: &mut Core, limits: &Limits, output: &[Request], records: Box<[s::Revision]>) -> Box<[Request]> {
    let (token, range) = load(output);
    let Scope::SettingsHistory { first, last, .. } = range.scope else { panic!("history range") };
    assert_eq!(usize::try_from(last.checked_sub(first).unwrap().checked_add(1).unwrap()).unwrap(), records.len());
    let mut rows = List::with_capacity(range.rows);
    for revision in records {
        let value = Record::Core(CoreRecord::Settings(s::Record::Revision {
            history: revision.head.number != core.current.newest(),
            revision,
        }));
        rows.push(Row {
            value,
            owner: None,
            ancestors: Box::default(),
            related: None,
            linked: Box::default(),
            inbox: None,
            unjoined: None,
            ended_read: None,
        })
        .unwrap();
    }
    send(core, limits, Event::ReadLoaded { load: token, result: Loaded::Page { rows: rows.into_boxed(), next: None } })
}
fn restored(core: &Core, output: &[Request]) -> (u64, s::Proposal) {
    let people::Outcome::Restored { proposal, to, left_out } = outcome(output) else { panic!("restored: {output:?}") };
    assert_eq!(to, people::Role::Owner.number());
    assert!(left_out.is_empty());
    (proposal, core.settings_proposals.get(&(Proposer::Party(1), u32::try_from(proposal).unwrap())).unwrap().clone())
}

#[test]
fn restoring_a_revision_proposes_what_later_revisions_replaced() {
    let (mut core, limits) = fixture(3);
    let earlier = charter(&core, 1, b"earlier");
    let later = charter(&core, 1, b"later");
    let third = record(&core, 3, Box::new([item(s::Setting::Charter(1), s::Replaced::Value(later))]), Box::default());
    let second = record(
        &core,
        2,
        Box::new([
            item(s::Setting::Charter(1), s::Replaced::Value(earlier.clone())),
            item(s::Setting::Charter(2), s::Replaced::Default),
            item(s::Setting::Template(1), s::Replaced::Removed),
            item(s::Setting::Policy(1), s::Replaced::NotOffered),
        ]),
        Box::default(),
    );
    let output = ask(&mut core, &limits, 1, 1);
    let output = page(&mut core, &limits, &output, Box::new([second, third]));
    let (proposal, value) = restored(&core, &output);
    assert_eq!(value.against, 3);
    assert_eq!(
        value.items.as_ref(),
        &[
            s::Item { setting: s::Setting::Charter(1), edit: s::Edit::Set(earlier) },
            s::Item { setting: s::Setting::Charter(2), edit: s::Edit::Reset },
            s::Item { setting: s::Setting::Template(1), edit: s::Edit::Removed },
            s::Item { setting: s::Setting::Policy(1), edit: s::Edit::Reset },
        ]
    );
    assert_eq!(core.tasks.person_proposal(1, proposal).unwrap().reason.as_ref(), b"restores revision 1: review");
    let replay = ask(&mut core, &limits, 1, 1);
    assert_eq!(outcome(&replay), outcome(&output), "the original restore outcome is retained");
}
#[test]
fn a_restore_reads_every_page_of_the_history_after_r() {
    let (mut core, mut limits) = fixture(6);
    limits.settings.history = 8;
    limits.documents.rows = 1;
    let value = charter(&core, 1, b"restored");
    let mut output = ask(&mut core, &limits, 1, 1);
    for number in (2..=6).rev() {
        let (_, range) = load(&output);
        assert_eq!(range.scope, Scope::SettingsHistory { first: number, last: number, newest: 6, whole: false });
        let revision = record(
            &core,
            number,
            Box::new([item(s::Setting::Charter(1), s::Replaced::Value(value.clone()))]),
            Box::default(),
        );
        output = page(&mut core, &limits, &output, Box::new([revision]));
    }
    assert_eq!(restored(&core, &output).1.items[0].edit, s::Edit::Set(value));
    assert!(core.restoring.pending.is_none());
}
#[test]
fn a_restore_of_a_forgotten_revision_names_the_oldest_kept() {
    let (mut core, mut limits) = fixture(6);
    limits.settings.history = 2;
    assert_eq!(
        outcome(&ask(&mut core, &limits, 3, 1)),
        people::Outcome::Refused(people::Refusal::Forgotten { oldest: 4 })
    );
}
#[test]
fn a_restore_of_the_newest_is_empty() {
    let (mut core, limits) = fixture(2);
    assert_eq!(
        outcome(&ask(&mut core, &limits, 2, 1)),
        people::Outcome::Refused(people::Refusal::NotWellFormed { at: 0, why: people::Malformed::SettingsUnchanged })
    );
}
#[test]
fn a_revision_made_while_a_restore_loads_refuses_it_stale() {
    let (mut core, limits) = fixture(2);
    let output = ask(&mut core, &limits, 1, 1);
    core.current.head.number = 3;
    let (load, _) = load(&output);
    let output = send(&mut core, &limits, Event::ReadLoaded { load, result: Loaded::Busy });
    assert_eq!(outcome(&output), people::Outcome::Refused(people::Refusal::Stale { newest: 3 }));
    assert!(core.restoring.pending.is_none());
}
#[test]
fn a_restore_past_the_whole_set_is_refused_too_large() {
    let (mut core, limits) = fixture(3);
    let third = record(
        &core,
        3,
        Box::new([
            item(s::Setting::Charter(1), s::Replaced::Default),
            item(s::Setting::Charter(2), s::Replaced::Default),
        ]),
        Box::default(),
    );
    let second = record(&core, 2, Box::new([item(s::Setting::Charter(3), s::Replaced::NotOffered)]), Box::default());
    let output = ask(&mut core, &limits, 1, 1);
    let output = page(&mut core, &limits, &output, Box::new([second, third]));
    assert_eq!(
        outcome(&output),
        people::Outcome::Refused(people::Refusal::NotWellFormed {
            at: 0,
            why: people::Malformed::Settings { setting: s::Setting::Charter(3), fault: s::Fault::Count, detail: None }
        })
    );
}
#[test]
fn a_restore_proposes_a_project_policy_it_replaced() {
    let (mut core, limits) = fixture(2);
    let mut policy = core.configured_policy(1).unwrap();
    policy.policy.period_spend = policy.policy.period_spend.checked_sub(1).unwrap();
    let expected = s::Value::Policy(Box::new(policy));
    let revision = record(
        &core,
        2,
        Box::new([item(s::Setting::ProjectPolicy { project: 1 }, s::Replaced::Value(expected.clone()))]),
        Box::default(),
    );
    let output = ask(&mut core, &limits, 1, 1);
    let output = page(&mut core, &limits, &output, Box::new([revision]));
    assert_eq!(
        restored(&core, &output).1.items.as_ref(),
        &[s::Item { setting: s::Setting::ProjectPolicy { project: 1 }, edit: s::Edit::Set(expected) }]
    );
}
#[test]
fn a_restore_leaves_out_and_names_a_deleted_projects_items() {
    let (mut core, limits) = fixture(2);
    let value = charter(&core, 1, b"restored");
    let revision = record(
        &core,
        2,
        Box::new([
            item(s::Setting::Charter(1), s::Replaced::Value(value)),
            item(
                s::Setting::ProjectPolicy { project: 2 },
                s::Replaced::Value(s::Value::Policy(Box::new(core.configured_policy(1).unwrap()))),
            ),
        ]),
        Box::new([app::RevisionRecorded {
            kind: 2,
            scope: 2,
            number: 1,
            edit: app::SizedEdit::Remove,
            replaced: app::Replaced::Changed { size: 4 },
        }]),
    );
    let output = ask(&mut core, &limits, 1, 1);
    let output = page(&mut core, &limits, &output, Box::new([revision]));
    let people::Outcome::Restored { proposal, left_out, .. } = outcome(&output) else { panic!("restored") };
    assert_eq!(
        left_out.as_ref(),
        &[s::Setting::Application { kind: 2, scope: 2, number: 1 }, s::Setting::ProjectPolicy { project: 2 }]
    );
    let set = core.settings_proposals.get(&(Proposer::Party(1), u32::try_from(proposal).unwrap())).unwrap();
    assert_eq!(set.items.len(), 1);
    assert!(set.applications.is_empty());
}
fn application_record(core: &Core) -> s::Revision {
    record(
        core,
        2,
        Box::default(),
        Box::new([
            app::RevisionRecorded {
                kind: 1,
                scope: 0,
                number: 1,
                edit: app::SizedEdit::Remove,
                replaced: app::Replaced::Changed { size: 4 },
            },
            app::RevisionRecorded {
                kind: 1,
                scope: 0,
                number: 2,
                edit: app::SizedEdit::Remove,
                replaced: app::Replaced::Default,
            },
        ]),
    )
}
#[test]
fn a_restore_asks_the_connector_for_an_applications_replaced_values() {
    let (mut core, limits) = fixture(3);
    let revision = application_record(&core);
    let mut newer = revision.clone();
    newer.head.number = 3;
    newer.head.against = 2;
    let output = ask(&mut core, &limits, 1, 1);
    let output = page(&mut core, &limits, &output, Box::new([revision, newer]));
    let mut ask = None;
    for request in &output {
        if let Request::Ask {
            connector: 1,
            ask: Ask::Effect(connector::Ask::RestoreSettings { owner, proposal, items }),
        } = request
        {
            assert_eq!(
                items.as_ref(),
                &[
                    app::Restored { kind: 1, scope: 0, number: 1, edit: app::RestoredEdit::Set { revision: 2 } },
                    app::Restored { kind: 1, scope: 0, number: 2, edit: app::RestoredEdit::Reset },
                ]
            );
            ask = Some((*owner, *proposal));
        }
    }
    let (owner, key) = ask.unwrap();
    let output = send(
        &mut core,
        &limits,
        Event::EffectConnector(connector::Event::SettingsRestored {
            owner,
            judged: Box::new([
                connector::Judged::Accept { size: 7, charters: Box::new([1]) },
                connector::Judged::Accept { size: 5, charters: Box::default() },
            ]),
        }),
    );
    let (_, value) = restored(&core, &output);
    assert_eq!(value.key, Some(key));
    assert!(value.items.is_empty());
    assert_eq!(
        value.applications.as_ref(),
        &[
            app::Recorded { kind: 1, scope: 0, number: 1, edit: app::SizedEdit::Set { size: 7 } },
            app::Recorded { kind: 1, scope: 0, number: 2, edit: app::SizedEdit::Reset },
        ]
    );
    assert!(core.current.applications.is_empty());
    for request in &output {
        assert!(match request {
            Request::Ask {
                ask:
                    Ask::Effect(connector::Ask::KeepProposalSettings { .. } | connector::Ask::DropProposalSettings { .. }),
                ..
            } => false,
            _ => true,
        });
    }
}
#[test]
fn erasing_a_revision_from_the_history_tells_connectors_to_forget_what_it_replaced() {
    let (mut core, mut limits) = fixture(2);
    limits.settings.history = 0;
    let value = charter(&core, 1, b"changed");
    let output = send(
        &mut core,
        &limits,
        Event::ChangeSettings {
            reply_to: ReplyTo::new(Token::new(99)),
            sign_in: 10,
            key: [1; 16],
            digest: [1; 32],
            change: Box::new(s::Change {
                against: 2,
                items: Box::new([s::Item { setting: s::Setting::Charter(1), edit: s::Edit::Set(value) }]),
                running: s::Running::Stays,
            }),
        },
    );
    assert_eq!(outcome(&output), people::Outcome::SettingsChanged { revision: 3 });
    let mut forgets = 0_u32;
    for request in &output {
        if let Request::Ask { connector: 1, ask: Ask::Effect(connector::Ask::ForgetReplaced { before }) } = request {
            assert_eq!(*before, 3);
            forgets = forgets.checked_add(1).unwrap();
        }
    }
    assert_eq!(forgets, 1, "one notification for the connector owning both kinds");
    let mut erased = false;
    for request in &output {
        if let Request::Write(Write::Erase(Key::Core(CoreKey::Settings(s::Key::History(1))))) = request {
            erased = true;
        }
    }
    assert!(erased);
}

#[test]
fn busy_and_unknown_restores_name_the_newest() {
    let (mut core, limits) = fixture(2);
    for (revision, key) in [(0, 1), (3, 2)] {
        assert_eq!(
            outcome(&ask(&mut core, &limits, revision, key)),
            people::Outcome::Refused(people::Refusal::Stale { newest: 2 })
        );
    }
    let output = ask(&mut core, &limits, 1, 3);
    assert_eq!(outcome(&ask(&mut core, &limits, 1, 4)), people::Outcome::Refused(people::Refusal::Stale { newest: 2 }));
    let revision = record(
        &core,
        2,
        Box::new([item(s::Setting::Charter(1), s::Replaced::Value(charter(&core, 1, b"restored")))]),
        Box::default(),
    );
    let output = page(&mut core, &limits, &output, Box::new([revision]));
    restored(&core, &output);
}
#[test]
fn a_restore_with_no_items_is_empty() {
    let (mut core, limits) = fixture(2);
    let revision = record(&core, 2, Box::default(), Box::default());
    let output = ask(&mut core, &limits, 1, 1);
    let output = page(&mut core, &limits, &output, Box::new([revision]));
    assert_eq!(
        outcome(&output),
        people::Outcome::Refused(people::Refusal::NotWellFormed { at: 0, why: people::Malformed::SettingsUnchanged })
    );
}
#[test]
fn two_project_policies_are_too_large_before_deleted_projects_are_left_out() {
    let (mut core, limits) = fixture(3);
    let value = s::Value::Policy(Box::new(core.configured_policy(1).unwrap()));
    let second = record(
        &core,
        2,
        Box::new([item(s::Setting::ProjectPolicy { project: 2 }, s::Replaced::Value(value.clone()))]),
        Box::default(),
    );
    let third = record(
        &core,
        3,
        Box::new([item(s::Setting::ProjectPolicy { project: 3 }, s::Replaced::Value(value))]),
        Box::default(),
    );
    let output = ask(&mut core, &limits, 1, 1);
    let output = page(&mut core, &limits, &output, Box::new([second, third]));
    assert_eq!(
        outcome(&output),
        people::Outcome::Refused(people::Refusal::NotWellFormed {
            at: 0,
            why: people::Malformed::Settings {
                setting: s::Setting::ProjectPolicy { project: 2 },
                fault: s::Fault::Count,
                detail: None
            }
        })
    );
}
#[test]
fn deleted_application_items_count_before_they_are_left_out() {
    let (mut core, limits) = fixture(3);
    let application = |number| app::RevisionRecorded {
        kind: 2,
        scope: 2,
        number,
        edit: app::SizedEdit::Remove,
        replaced: app::Replaced::Removed,
    };
    let second = record(&core, 2, Box::default(), Box::new([application(3)]));
    let third = record(&core, 3, Box::default(), Box::new([application(1), application(2)]));
    let output = ask(&mut core, &limits, 1, 1);
    let output = page(&mut core, &limits, &output, Box::new([second, third]));
    assert_eq!(
        outcome(&output),
        people::Outcome::Refused(people::Refusal::NotWellFormed {
            at: 0,
            why: people::Malformed::Settings {
                setting: s::Setting::Application { kind: 2, scope: 2, number: 3 },
                fault: s::Fault::Count,
                detail: None
            }
        })
    );
}
#[test]
fn a_connectors_refusal_drops_its_restore_proposal_values() {
    let (mut core, limits) = fixture(2);
    let revision = application_record(&core);
    let output = ask(&mut core, &limits, 1, 1);
    let output = page(&mut core, &limits, &output, Box::new([revision]));
    let (owner, key) = connector_restore(&output);
    let output = send(
        &mut core,
        &limits,
        Event::EffectConnector(connector::Event::SettingsRestored {
            owner,
            judged: Box::new([
                connector::Judged::Refuse { why: b"unavailable".as_slice().into() },
                connector::Judged::Accept { size: 5, charters: Box::default() },
            ]),
        }),
    );
    assert_eq!(
        outcome(&output),
        people::Outcome::Refused(people::Refusal::Judged {
            at: 0,
            setting: s::Setting::Application { kind: 1, scope: 0, number: 1 },
            why: b"unavailable".as_slice().into()
        })
    );
    assert!(drops(&output).as_ref().contains(&(1, key)));
    assert!(core.settings_proposals.is_empty());
    assert!(core.restoring.pending.is_none());
}
#[test]
fn a_restore_reason_is_cut_at_a_character_boundary() {
    let (mut core, mut limits) = fixture(2);
    limits.people.words = 23;
    let revision = record(
        &core,
        2,
        Box::new([item(s::Setting::Charter(1), s::Replaced::Value(charter(&core, 1, b"restored")))]),
        Box::default(),
    );
    let output = send(
        &mut core,
        &limits,
        Event::People(people::Event::Ask {
            reply_to: ReplyTo::new(Token::new(1)),
            sign_in: 10,
            key: [1; 16],
            models: Box::default(),
            ask: people::Ask::Restore { revision: 1, reason: "ééé".as_bytes().into() },
        }),
    );
    let output = page(&mut core, &limits, &output, Box::new([revision]));
    let (proposal, _) = restored(&core, &output);
    assert_eq!(core.tasks.person_proposal(1, proposal).unwrap().reason.as_ref(), "restores revision 1: é".as_bytes());
}

#[test]
fn application_sizes_and_charter_counts_are_checked_before_the_proposal_is_kept() {
    for (size, charters) in [(65, Box::from([])), (7, Box::from([1, 1]))] {
        let (mut core, limits) = fixture(2);
        let revision = application_record(&core);
        let output = ask(&mut core, &limits, 1, 1);
        let output = page(&mut core, &limits, &output, Box::new([revision]));
        let (owner, key) = connector_restore(&output);
        let output = send(
            &mut core,
            &limits,
            Event::EffectConnector(connector::Event::SettingsRestored {
                owner,
                judged: Box::new([
                    connector::Judged::Accept { size, charters },
                    connector::Judged::Accept { size: 5, charters: Box::default() },
                ]),
            }),
        );
        assert_eq!(
            outcome(&output),
            people::Outcome::Refused(people::Refusal::NotWellFormed {
                at: 0,
                why: people::Malformed::Settings {
                    setting: s::Setting::Application { kind: 1, scope: 0, number: 1 },
                    fault: s::Fault::Shape,
                    detail: None
                }
            })
        );
        assert!(core.settings_proposals.is_empty());
        assert!(drops(&output).as_ref().contains(&(1, key)));
    }
}
#[test]
fn a_whole_application_kind_is_bounded_by_its_declared_count() {
    let (mut core, limits) = fixture_with_count(2, 16);
    let mut applications = List::with_capacity(16);
    for number in 1..=16 {
        applications
            .push(app::RevisionRecorded {
                kind: 1,
                scope: 0,
                number,
                edit: app::SizedEdit::Reset,
                replaced: app::Replaced::Removed,
            })
            .unwrap();
    }
    let revision = record(&core, 2, Box::default(), applications.into_boxed());
    let output = ask(&mut core, &limits, 1, 1);
    let output = page(&mut core, &limits, &output, Box::new([revision]));
    let (owner, _) = connector_restore(&output);
    for request in &output {
        if let Request::Ask { ask: Ask::Effect(connector::Ask::RestoreSettings { items, .. }), .. } = request {
            assert_eq!(items.len(), 16);
            for item in items {
                assert_eq!(item.edit, app::RestoredEdit::Remove);
            }
        }
    }
    let mut judged = List::with_capacity(16);
    for _ in 0_u32..16 {
        judged.push(connector::Judged::Accept { size: 0, charters: Box::default() }).unwrap();
    }
    let output = send(
        &mut core,
        &limits,
        Event::EffectConnector(connector::Event::SettingsRestored { owner, judged: judged.into_boxed() }),
    );
    assert_eq!(restored(&core, &output).1.applications.len(), 16);
}

fn connector_restore(output: &[Request]) -> (Token, u64) {
    for request in output {
        if let Request::Ask { ask: Ask::Effect(connector::Ask::RestoreSettings { owner, proposal, .. }), .. } = request
        {
            return (*owner, *proposal);
        }
    }
    panic!("restore connector ask: {output:?}");
}
fn drops(output: &[Request]) -> Box<[(u16, u64)]> {
    let mut drops = List::with_capacity(u32::try_from(output.len()).unwrap());
    for request in output {
        if let Request::Ask { connector, ask: Ask::Effect(connector::Ask::DropProposalSettings { proposal }) } = request
        {
            drops.push((*connector, *proposal)).unwrap();
        }
    }
    drops.into_boxed()
}

#[test]
fn a_busy_history_load_is_retryable_under_the_same_key() {
    let (mut core, limits) = fixture(2);
    let output = ask(&mut core, &limits, 1, 1);
    let (load, _) = load(&output);
    let output = send(&mut core, &limits, Event::ReadLoaded { load, result: Loaded::Busy });
    let mut busy = false;
    for request in &output {
        if let Request::Held(held) = request
            && let Held::PeopleReply { reply: people::Reply::Again(people::Again::Busy), .. } = held.as_ref()
        {
            busy = true;
        }
    }
    assert!(busy);
    assert!(core.restoring.pending.is_none());
    let output = ask(&mut core, &limits, 1, 1);
    let (_, range) = self::load(&output);
    assert_eq!(range.scope, Scope::SettingsHistory { first: 2, last: 2, newest: 2, whole: false });
}

#[test]
fn deleted_application_scopes_share_the_kinds_whole_set_bound() {
    let (mut core, limits) = fixture(3);
    assert_eq!(limits.authority.projects, 1);
    let application = |scope, number| app::RevisionRecorded {
        kind: 2,
        scope,
        number,
        edit: app::SizedEdit::Remove,
        replaced: app::Replaced::Removed,
    };
    let second = record(&core, 2, Box::default(), Box::new([application(3, 1)]));
    let third = record(&core, 3, Box::default(), Box::new([application(2, 1), application(2, 2)]));
    let output = ask(&mut core, &limits, 1, 1);
    let output = page(&mut core, &limits, &output, Box::new([second, third]));
    assert_eq!(
        outcome(&output),
        people::Outcome::Refused(people::Refusal::NotWellFormed {
            at: 0,
            why: people::Malformed::Settings {
                setting: s::Setting::Application { kind: 2, scope: 3, number: 1 },
                fault: s::Fault::Count,
                detail: None
            }
        })
    );
}

#[test]
fn restoring_an_overlaid_revision_proposes_the_overlay_not_its_charter() {
    let (mut core, limits) = fixture(2);
    let value = s::Value::Overlay(Box::new(s::Overlay { instructions: b"B".as_slice().into(), ..s::Overlay::empty() }));
    let second = record(&core, 2,
        Box::new([item(s::Setting::Charter(1), s::Replaced::Value(value.clone()))]), Box::new([]));
    let output = ask(&mut core, &limits, 1, 1);
    let output = page(&mut core, &limits, &output, Box::new([second]));
    let (_, proposal) = restored(&core, &output);
    assert_eq!(proposal.items.as_ref(), &[s::Item { setting: s::Setting::Charter(1), edit: s::Edit::Set(value) }]);
}
