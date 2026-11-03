//! Declaring application kinds has no startup behavior (jig/docs/design/domain/engine.md, 17.5).
use crate::settings::{Kind, KindScope};
use crate::tests::*;

#[test]
#[expect(
    clippy::disallowed_macros,
    reason = "ordinary unit proof compares complete Debug outputs because Request does not derive equality"
)]
fn declaring_an_application_kind_preserves_start_outputs() {
    for seed in [0, 1, u64::MAX] {
        let limits = limits();
        let configured = Config {
            kinds: Box::new([Kind {
                number: 1,
                scope: KindScope::Project,
                connector: 2,
                value_bytes: 64,
                per_scope: 4,
                charters: 1,
            }]),
            ..config(seed)
        };
        let mut declared = Core::new(configured, &limits).unwrap();
        let mut empty = Core::new(config(seed), &limits).unwrap();
        let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
        let declared_outputs = finish_start(&mut declared, &env);
        let empty_outputs = finish_start(&mut empty, &env);
        assert!(!empty_outputs.is_empty(), "the start emits its writes and asks");
        assert_eq!(format!("{declared_outputs:?}"), format!("{empty_outputs:?}"), "seed {seed}");
    }
}

use crate::settings::{self as s, application as app};

fn kind(number: u32, connector: u16) -> Kind {
    Kind { number, connector, scope: KindScope::Deployment, value_bytes: 8, per_scope: 2, charters: 1 }
}
fn setting(number: u32) -> s::Setting {
    s::Setting::Application { kind: 1, scope: 0, number }
}
fn item(number: u32, edit: s::Edit) -> s::Item {
    s::Item { setting: setting(number), edit }
}
fn set(number: u32) -> s::Item {
    item(number, s::Edit::Set(s::Value::Application(Box::from(b"value".as_slice()))))
}
fn accept(size: u32, charters: &[u32]) -> connector::Judged {
    connector::Judged::Accept { size, charters: charters.into() }
}
fn application_core() -> (Core, Limits) {
    application_core_with(Box::new([kind(1, 2), kind(2, 2), kind(3, 3)]))
}
fn application_core_with(kinds: Box<[Kind]>) -> (Core, Limits) {
    let (old, mut limits) = core();
    limits.people.projects = 2;
    limits.people.requests = 16;
    let mut configured = config(41);
    configured.kinds = kinds;
    configured.connectors = Box::new([2, 3]);
    let mut core = seeded_core(configured, &limits).unwrap();
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.people };
    for record in [
        people::Stored::Person { number: 1, identity: old.people.identity(1).unwrap().clone() },
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
    (core, limits)
}

/// The scripted root feeds every judge's answer before returning its gathered decision.
fn change(
    core: &mut Core,
    limits: &Limits,
    items: Box<[s::Item]>,
    answers: Box<[Box<[connector::Judged]>]>,
) -> Box<[Request]> {
    let key = [u8::try_from(core.current.newest()).unwrap(); 16];
    change_key(core, limits, items, answers, key)
}
fn change_key(
    core: &mut Core,
    limits: &Limits,
    items: Box<[s::Item]>,
    answers: Box<[Box<[connector::Judged]>]>,
    key: [u8; 16],
) -> Box<[Request]> {
    let previous = core.current.newest();
    let mut events = Queue::uncapped();
    events.push(Event::ChangeSettings {
        reply_to: ReplyTo::new(Token::new(91)),
        sign_in: 10,
        key,
        digest: [u8::try_from(previous).unwrap(); 32],
        change: Box::new(s::Change { against: previous, items, running: s::Running::Stays }),
    });
    let mut script = answers.into_iter();
    let mut output = List::with_capacity(256);
    while let Some(event) = events.pop() {
        let mut out = step(core, &Env { now: Time::ZERO, wall: Wall::EPOCH, limits: *limits }, event);
        while let Some(request) = out.pop() {
            if let Request::Ask {
                ask: Ask::Effect(connector::Ask::JudgeSettings { owner, revision, proposal, .. }),
                ..
            } = &request
            {
                assert_eq!(core.current.newest(), previous, "no change before all judges answer");
                assert_eq!(*revision, u64::from(previous.checked_add(1).unwrap()));
                assert_eq!(*proposal, None);
                events.push(Event::EffectConnector(connector::Event::SettingsJudged {
                    owner: *owner,
                    judged: script.next().expect("scripted judge answer"),
                }));
            }
            output.push(request).unwrap();
        }
    }
    assert!(script.next().is_none(), "every scripted judge was asked: {output:?}");
    core.reclaim();
    output.into_boxed()
}
fn outcome(output: &[Request]) -> people::Outcome {
    let mut result = None;
    for request in output {
        if let Request::Held(held) = request
            && let Held::PeopleReply { reply: people::Reply::Outcome(outcome), .. } = held.as_ref()
        {
            assert!(result.is_none(), "one keyed answer");
            result = Some(outcome.clone());
        }
    }
    result.expect("keyed outcome")
}
fn kept(output: &[Request]) -> Box<[app::Item]> {
    let mut items = List::with_capacity(16);
    for request in output {
        if let Request::Ask {
            ask: Ask::Effect(connector::Ask::KeepSettings { revision, proposal, items: group }),
            ..
        } = request
        {
            assert!(*revision > 1);
            assert_eq!(*proposal, None);
            for item in group {
                items.push(item.clone()).unwrap();
            }
        }
    }
    items.into_boxed()
}
fn application_writes(output: &[Request]) -> u32 {
    let mut count = 0_u32;
    for request in output {
        if let Request::Write(Write::Save(Record::Core(CoreRecord::Settings(s::Record::Application { .. })))) = request
        {
            count = count.checked_add(1).unwrap();
        }
    }
    count
}
fn assert_refused(core: &Core, output: &[Request], why: people::Refusal) {
    assert_eq!(outcome(output), people::Outcome::Refused(why));
    assert_eq!(core.current.newest(), 1);
    assert!(core.current.applications.is_empty());
    assert!(kept(output).is_empty());
    assert_eq!(application_writes(output), 0);
    for request in output {
        if let Request::Write(Write::Save(Record::Core(CoreRecord::Settings(s::Record::Revision { .. })))) = request {
            panic!("refused settings wrote a revision");
        }
    }
}

#[test]
fn application_set_is_judged_kept_and_recorded_without_its_bytes() {
    let (mut core, limits) = application_core();
    let out = change(&mut core, &limits, Box::new([set(1)]), Box::new([Box::new([accept(5, &[])])]));
    assert_eq!(outcome(&out), people::Outcome::SettingsChanged { revision: 2 });
    assert_eq!(
        kept(&out).as_ref(),
        &[app::Item { kind: 1, scope: 0, number: 1, edit: app::Edit::Set(Box::from(b"value".as_slice())) }]
    );
    assert_eq!(
        core.current.applications.get(&setting(1)),
        Some(&app::State { follows: app::Follows::Changed { revision: 2 }, size: 5, charters: Box::default() })
    );
    assert_eq!(application_writes(&out), 1);
    assert!(!core.current.changes.contains_key(&setting(1)));
    let mut saved = None;
    for request in &out {
        if let Request::Write(Write::Save(Record::Core(CoreRecord::Settings(s::Record::Revision {
            revision,
            history: false,
        })))) = request
        {
            saved = Some(revision);
        }
    }
    let revision = saved.unwrap();
    assert!(revision.items.is_empty());
    assert_eq!(
        revision.applications.as_ref(),
        &[app::Recorded { kind: 1, scope: 0, number: 1, edit: app::SizedEdit::Set { size: 5 } }]
    );
}
#[test]
fn application_refusal_keeps_its_words_and_commits_nothing() {
    let (mut core, limits) = application_core();
    let out = change(
        &mut core,
        &limits,
        Box::new([set(1)]),
        Box::new([Box::new([connector::Judged::Refuse { why: Box::from(b"not permitted".as_slice()) }])]),
    );
    assert_refused(
        &core,
        &out,
        people::Refusal::Judged { at: 0, setting: setting(1), why: Box::from(b"not permitted".as_slice()) },
    );
}
#[test]
fn application_same_is_empty_without_a_keep_or_state_write() {
    let (mut core, limits) = application_core();
    let out = change(&mut core, &limits, Box::new([set(1)]), Box::new([Box::new([connector::Judged::Same])]));
    assert_refused(&core, &out, people::Refusal::NotWellFormed { at: 0, why: people::Malformed::SettingsUnchanged });
}
#[test]
fn application_removal_stays_removed_and_reset_follows_the_default() {
    let (mut core, limits) = application_core();
    change(&mut core, &limits, Box::new([set(1)]), Box::new([Box::new([accept(5, &[])])]));
    let removed =
        change(&mut core, &limits, Box::new([item(1, s::Edit::Removed)]), Box::new([Box::new([accept(0, &[])])]));
    assert_eq!(outcome(&removed), people::Outcome::SettingsChanged { revision: 3 });
    assert_eq!(
        core.current.applications.get(&setting(1)).unwrap(),
        &app::State { follows: app::Follows::Removed { revision: 3 }, size: 0, charters: Box::default() }
    );
    assert_eq!(kept(&removed)[0].edit, app::Edit::Remove);
    let reset = change(&mut core, &limits, Box::new([item(1, s::Edit::Reset)]), Box::new([Box::new([accept(3, &[])])]));
    assert_eq!(outcome(&reset), people::Outcome::SettingsChanged { revision: 4 });
    assert_eq!(
        core.current.applications.get(&setting(1)).unwrap(),
        &app::State { follows: app::Follows::Default, size: 3, charters: Box::default() }
    );
    assert_eq!(kept(&reset)[0].edit, app::Edit::Reset);
}
#[test]
fn application_charter_reference_prevents_removing_the_offered_charter() {
    let (mut core, mut limits) = application_core();
    limits.tasks.charters = 2;
    let mut charter = core.current.charter(1).unwrap().clone();
    charter.number = 3;
    core.current
        .defaults
        .insert(s::Setting::Charter(3), s::Default { value: s::Value::Charter(Box::new(charter)), revision: 1 })
        .unwrap();
    let out = change(&mut core, &limits, Box::new([set(1)]), Box::new([Box::new([accept(5, &[3])])]));
    assert_eq!(outcome(&out), people::Outcome::SettingsChanged { revision: 2 });
    let refused = change(
        &mut core,
        &limits,
        Box::new([s::Item { setting: s::Setting::Charter(3), edit: s::Edit::Removed }]),
        Box::default(),
    );
    assert_eq!(
        outcome(&refused),
        people::Outcome::Refused(people::Refusal::NotWellFormed {
            at: 0,
            why: people::Malformed::Settings { setting: setting(1), fault: s::Fault::Reference }
        })
    );
    assert_eq!(core.current.newest(), 2);
    assert!(core.current.charter(3).is_some());
    assert!(kept(&refused).is_empty());
}
#[test]
fn application_bad_names_scopes_and_oversized_values_are_shape_before_judging() {
    for item in [
        s::Item { setting: s::Setting::Application { kind: 99, scope: 0, number: 1 }, ..set(1) },
        s::Item { setting: s::Setting::Application { kind: 1, scope: 1, number: 1 }, ..set(1) },
        set(0),
        item(1, s::Edit::Set(s::Value::Application(Box::from(b"123456789".as_slice())))),
    ] {
        let (mut core, limits) = application_core();
        let setting = item.setting;
        let out = change(&mut core, &limits, Box::new([item]), Box::default());
        assert_refused(
            &core,
            &out,
            people::Refusal::NotWellFormed {
                at: 0,
                why: people::Malformed::Settings { setting, fault: s::Fault::Shape },
            },
        );
    }
}
#[test]
fn application_count_cannot_pass_per_scope() {
    let (mut core, limits) = application_core();
    let out = change(&mut core, &limits, Box::new([set(1), set(2), set(3)]), Box::default());
    assert_refused(
        &core,
        &out,
        people::Refusal::NotWellFormed {
            at: 0,
            why: people::Malformed::Settings { setting: setting(1), fault: s::Fault::Count },
        },
    );
}

#[test]
fn application_removed_states_hold_their_slots() {
    let (mut core, limits) = application_core_with(Box::new([kind(1, 2)]));
    assert_eq!(core.current.applications.capacity(), 2);
    for number in 1..=2 {
        let out = change(&mut core, &limits, Box::new([set(number)]), Box::new([Box::new([accept(5, &[])])]));
        assert_eq!(outcome(&out), people::Outcome::SettingsChanged { revision: number * 2 });
        let out = change(
            &mut core,
            &limits,
            Box::new([item(number, s::Edit::Removed)]),
            Box::new([Box::new([accept(0, &[])])]),
        );
        assert_eq!(outcome(&out), people::Outcome::SettingsChanged { revision: number * 2 + 1 });
    }
    let refused = change(&mut core, &limits, Box::new([set(3)]), Box::default());
    assert_eq!(
        outcome(&refused),
        people::Outcome::Refused(people::Refusal::NotWellFormed {
            at: 0,
            why: people::Malformed::Settings { setting: setting(3), fault: s::Fault::Count },
        })
    );
    assert_eq!(core.current.newest(), 5);
    assert_eq!(core.current.applications.len(), 2);
    assert!(kept(&refused).is_empty());
    assert_eq!(application_writes(&refused), 0);
    for number in 1..=2 {
        assert_eq!(
            core.current.applications.get(&setting(number)).unwrap().follows,
            app::Follows::Removed { revision: number * 2 + 1 }
        );
    }
    let updated = change_key(&mut core, &limits, Box::new([set(1)]), Box::new([Box::new([accept(5, &[])])]), [99; 16]);
    assert_eq!(outcome(&updated), people::Outcome::SettingsChanged { revision: 6 });
    assert_eq!(core.current.applications.len(), 2);
    assert_eq!(core.current.applications.get(&setting(1)).unwrap().follows, app::Follows::Changed { revision: 6 });
}

#[test]
fn application_removal_and_reset_of_new_numbers_take_slots() {
    for edit in [s::Edit::Removed, s::Edit::Reset] {
        let (mut core, limits) = application_core_with(Box::new([kind(1, 2)]));
        let kept = change(
            &mut core,
            &limits,
            Box::new([item(1, edit.clone()), item(2, edit.clone())]),
            Box::new([Box::new([accept(0, &[]), accept(0, &[])])]),
        );
        assert_eq!(outcome(&kept), people::Outcome::SettingsChanged { revision: 2 });
        let refused = change(&mut core, &limits, Box::new([item(3, edit)]), Box::default());
        assert_eq!(
            outcome(&refused),
            people::Outcome::Refused(people::Refusal::NotWellFormed {
                at: 0,
                why: people::Malformed::Settings { setting: setting(3), fault: s::Fault::Count },
            })
        );
        assert_eq!(core.current.newest(), 2);
        assert_eq!(core.current.applications.len(), 2);
        assert_eq!(application_writes(&refused), 0);
    }
}
