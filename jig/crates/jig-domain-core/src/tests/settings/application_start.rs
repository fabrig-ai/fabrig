//! Application defaults join the start's revision and project deletion (jig/docs/design/domain/engine.md, 17.5–17.7).
use crate::settings::{self as s, Kind, KindScope, application as app};
use crate::start::{Action, ApplicationWhy, Failure, Phase};
use crate::tests::*;

fn kind(number: u32, connector: u16, scope: KindScope) -> Kind {
    Kind { number, connector, scope, value_bytes: 8, per_scope: 2, charters: 1 }
}
fn configuration() -> Config {
    Config { kinds: Box::new([kind(1, 2, KindScope::Deployment)]), ..config(41) }
}
fn environment() -> Env<Limits> {
    Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits() }
}
fn setting(kind: u32, scope: u32, number: u32) -> s::Setting {
    s::Setting::Application { kind, scope, number }
}
fn default(size: u32, differs: bool) -> connector::DefaultState {
    connector::DefaultState { kind: 1, scope: 0, number: 1, size, charters: Box::new([1]), differs }
}
fn answer(core: &mut Core, env: &Env<Limits>, connector: u16, defaults: Box<[connector::DefaultState]>) {
    assert!(
        step(core, env, Event::EffectConnector(connector::Event::SettingsDefaults { connector, defaults })).is_empty()
    );
}

/// The scripted root supplies defaults before completing that connector's restore.
fn complete(
    core: &mut Core,
    env: &Env<Limits>,
    mut action: Action,
    defaults: Box<[connector::DefaultState]>,
) -> Box<[Request]> {
    let mut defaults = Some(defaults);
    let mut out = Queue::uncapped();
    for _ in 0_u32..64 {
        action = match action {
            Action::Step(Phase::RestoreConnector { connector: 2 }) => {
                answer(core, env, 2, defaults.take().unwrap());
                core.start_done(env, Phase::RestoreConnector { connector: 2 }, &mut out)
            }
            Action::Step(Phase::Sweep) => core.start_sweep(env, crate::start::SweepInput::Finished, &mut out),
            Action::Step(phase) => core.start_done(env, phase, &mut out),
            Action::Commit(token) => core.start_resume(env, token, &mut out),
            Action::Idle => {
                assert!(core.start_opened());
                assert!(defaults.is_none());
                let mut requests = List::with_capacity(out.len());
                while let Some(request) = out.pop() {
                    requests.push(request).unwrap();
                }
                return requests.into_boxed();
            }
            Action::Refused(why) => panic!("scripted start refused: {why:?}"),
        };
    }
    panic!("bounded start");
}
fn first() -> (Core, Env<Limits>, Box<[Request]>) {
    let env = environment();
    let mut core = Core::new(configuration(), &env.limits).unwrap();
    let action = core.start_begin(false, env.wall);
    let out = complete(&mut core, &env, action, Box::new([default(3, true)]));
    (core, env, out)
}
fn snapshot(core: &mut Core, out: &[Request]) -> Map<Key, Record> {
    let mut rows = Map::with_capacity(128);
    for request in out {
        if let Request::Write(Write::Save(record)) = request {
            rows.insert(record.key(), record.clone()).unwrap();
        }
    }
    let header = Record::Core(CoreRecord::Deployment(core.counters.next_commit()));
    rows.insert(header.key(), header).unwrap();
    rows
}
fn restart(configured: Config, env: &Env<Limits>, rows: &Map<Key, Record>) -> Core {
    let mut core = Core::new(configured, &env.limits).unwrap();
    let _begun = core.start_begin(false, env.wall);
    for range in RestoreRange::LIVE {
        for (key, record) in rows {
            if range.admits(key) {
                assert_ne!(core.restore(env, record.clone()), Restore::Refused);
            }
        }
    }
    core
}
fn saved(out: &[Request]) -> Box<[(u16, Option<u64>)]> {
    let mut saves = List::with_capacity(4);
    for request in out {
        if let Request::Ask { connector, ask: Ask::Effect(connector::Ask::SaveSettingsDefaults { revision }) } = request
        {
            saves.push((*connector, *revision)).unwrap();
        }
    }
    saves.into_boxed()
}
fn revisions(out: &[Request]) -> Box<[s::Revision]> {
    let mut revisions = List::with_capacity(4);
    for request in out {
        if let Request::Write(Write::Save(Record::Core(CoreRecord::Settings(s::Record::Revision {
            history: false,
            revision,
        })))) = request
        {
            revisions.push(revision.clone()).unwrap();
        }
    }
    revisions.into_boxed()
}

#[test]
fn first_start_sizes_the_default_in_revision_one_and_saves_it_once() {
    let (core, _, out) = first();
    assert_eq!(
        core.current.applications.get(&setting(1, 0, 1)),
        Some(&app::State { follows: app::Follows::Default, size: 3, charters: Box::new([1]) })
    );
    let revisions = revisions(&out);
    assert_eq!(revisions.len(), 1);
    assert_eq!(revisions[0].head.number, 1);
    assert_eq!(revisions[0].head.by, s::Decider::FirstStart);
    assert_eq!(
        revisions[0].applications.as_ref(),
        &[app::RevisionRecorded {
            kind: 1,
            scope: 0,
            number: 1,
            edit: app::SizedEdit::Set { size: 3 },
            replaced: app::Replaced::Default
        }]
    );
    assert_eq!(saved(&out).as_ref(), &[(2, Some(1))]);
}

#[test]
fn restart_without_a_default_difference_keeps_the_revision_and_saves_without_one() {
    let (mut core, env, out) = first();
    let rows = snapshot(&mut core, &out);
    let mut restarted = restart(configuration(), &env, &rows);
    assert_eq!(restarted.current.applications.get(&setting(1, 0, 1)), core.current.applications.get(&setting(1, 0, 1)));
    let out = complete(&mut restarted, &env, Action::Step(Phase::LoadCore), Box::new([default(3, false)]));
    assert_eq!(restarted.current.newest(), 1);
    assert!(revisions(&out).is_empty());
    assert_eq!(saved(&out).as_ref(), &[(2, None)]);
}

#[test]
fn a_changed_default_makes_a_release_revision_with_its_size_and_save() {
    let (mut core, env, out) = first();
    let rows = snapshot(&mut core, &out);
    let mut restarted = restart(configuration(), &env, &rows);
    let out = complete(&mut restarted, &env, Action::Step(Phase::LoadCore), Box::new([default(6, true)]));
    assert_eq!(restarted.current.newest(), 2);
    assert_eq!(restarted.current.applications.get(&setting(1, 0, 1)).unwrap().size, 6);
    let revisions = revisions(&out);
    assert_eq!(revisions.len(), 1);
    assert_eq!(revisions[0].head.by, s::Decider::Release);
    assert_eq!(
        revisions[0].applications.as_ref(),
        &[app::RevisionRecorded {
            kind: 1,
            scope: 0,
            number: 1,
            edit: app::SizedEdit::Set { size: 6 },
            replaced: app::Replaced::Default
        }]
    );
    assert_eq!(saved(&out).as_ref(), &[(2, Some(2))]);
}

#[test]
fn a_changed_or_removed_state_stays_over_a_changed_default_without_a_revision() {
    for follows in [app::Follows::Changed { revision: 1 }, app::Follows::Removed { revision: 1 }] {
        let (mut core, env, out) = first();
        let mut rows = snapshot(&mut core, &out);
        let state = app::State { follows, size: 5, charters: Box::new([1]) };
        let record = Record::Core(CoreRecord::Settings(s::Record::Application {
            setting: setting(1, 0, 1),
            state: state.clone(),
        }));
        rows.insert(record.key(), record).unwrap();
        let mut restarted = restart(configuration(), &env, &rows);
        let out = complete(&mut restarted, &env, Action::Step(Phase::LoadCore), Box::new([default(6, true)]));
        assert_eq!(restarted.current.newest(), 1);
        assert_eq!(restarted.current.applications.get(&setting(1, 0, 1)), Some(&state));
        assert!(revisions(&out).is_empty());
        assert_eq!(saved(&out).as_ref(), &[(2, None)]);
    }
}

#[test]
fn a_default_no_longer_reported_is_erased_in_a_release_revision() {
    let (mut core, env, out) = first();
    let rows = snapshot(&mut core, &out);
    let mut restarted = restart(configuration(), &env, &rows);
    let out = complete(&mut restarted, &env, Action::Step(Phase::LoadCore), Box::default());
    assert!(restarted.current.applications.is_empty());
    let revisions = revisions(&out);
    assert_eq!(revisions.len(), 1);
    assert_eq!(revisions[0].head.number, 2);
    assert_eq!(
        revisions[0].applications.as_ref(),
        &[app::RevisionRecorded {
            kind: 1,
            scope: 0,
            number: 1,
            edit: app::SizedEdit::Remove,
            replaced: app::Replaced::Default
        }]
    );
    assert_eq!(saved(&out).as_ref(), &[(2, Some(2))]);
    let mut erased = false;
    for request in &out {
        if let Request::Write(Write::Erase(Key::Core(CoreKey::Settings(s::Key::Changed(found))))) = request {
            erased |= *found == setting(1, 0, 1);
        }
    }
    assert!(erased);
}

#[test]
fn a_stored_state_of_an_undeclared_kind_refuses_the_start() {
    let env = environment();
    let mut core = Core::new(configuration(), &env.limits).unwrap();
    let _begun = core.start_begin(false, env.wall);
    let row = CoreRecord::Settings(s::Record::Application {
        setting: setting(99, 0, 7),
        state: app::State { follows: app::Follows::Default, size: 1, charters: Box::default() },
    });
    assert_eq!(core.restore_core(row, &env.limits), Restored::Rejected);
    assert_eq!(
        core.start_failure(),
        Some(Failure::Application { kind: 99, scope: 0, number: 7, why: ApplicationWhy::Undeclared })
    );
}

#[test]
fn an_unoffered_charter_in_a_default_refuses_as_a_reference() {
    let env = environment();
    let mut core = Core::new(configuration(), &env.limits).unwrap();
    let _begun = core.start_begin(false, env.wall);
    answer(&mut core, &env, 2, Box::new([connector::DefaultState { charters: Box::new([99]), ..default(3, true) }]));
    let mut out = Queue::uncapped();
    let mut action = Action::Step(Phase::LoadCore);
    for _ in 0_u32..8 {
        let Action::Step(phase) = action else { panic!("load precedes the first seed") };
        if phase == Phase::Link {
            break;
        }
        action = core.start_done(&env, phase, &mut out);
    }
    let Action::Refused(failure) = core.start_done(&env, Phase::Link, &mut out) else { panic!("reference refused") };
    assert_eq!(
        failure,
        Failure::Settings {
            failure: s::Failure { setting: setting(1, 0, 1), fault: s::Fault::Reference },
            changed: false
        }
    );
    assert!(out.is_empty());
}

#[test]
fn defaults_wait_for_settings_restore_and_every_kind_owner_even_an_empty_reply() {
    let (mut original, env, out) = first();
    let rows = snapshot(&mut original, &out);
    let configured = Config {
        kinds: Box::new([
            kind(1, 2, KindScope::Deployment),
            kind(2, 1, KindScope::Deployment),
            kind(3, 1, KindScope::Project),
        ]),
        ..config(41)
    };
    let mut core = Core::new(configured, &env.limits).unwrap();
    let action = core.start_begin(false, env.wall);
    answer(&mut core, &env, 2, Box::new([default(6, true)]));
    assert!(core.current.applications.is_empty(), "early defaults do not replace the stored states");
    for range in RestoreRange::LIVE {
        for (key, record) in &rows {
            if range.admits(key) {
                assert_ne!(core.restore(&env, record.clone()), Restore::Refused);
            }
        }
    }
    assert_eq!(core.current.applications.get(&setting(1, 0, 1)).unwrap().size, 3);
    let before = collect(finish_started(&mut core, &env, action));
    assert_eq!(core.start_phase(), Some(Phase::Link));
    assert_eq!(core.current.newest(), 1);
    assert!(saved(&before).is_empty());
    answer(&mut core, &env, 1, Box::default());
    let mut out = Queue::uncapped();
    let action = core.start_done(&env, Phase::Link, &mut out);
    let mut following = finish_started(&mut core, &env, action);
    while let Some(request) = following.pop() {
        out.push(request);
    }
    assert!(core.start_opened());
    assert_eq!(core.current.newest(), 2);
    assert_eq!(saved(&collect(out)).as_ref(), &[(1, Some(2)), (2, Some(2))]);
}

#[test]
fn defaults_outside_the_declared_kind_refuse_with_the_setting_and_reason() {
    let cases = [
        (connector::DefaultState { kind: 99, ..default(1, true) }, ApplicationWhy::Undeclared),
        (connector::DefaultState { scope: 1, ..default(1, true) }, ApplicationWhy::Scope),
        (default(9, true), ApplicationWhy::Shape),
        (connector::DefaultState { charters: Box::new([1, 2]), ..default(1, true) }, ApplicationWhy::Shape),
    ];
    for (default, why) in cases {
        let env = environment();
        let mut core = Core::new(configuration(), &env.limits).unwrap();
        let _begun = core.start_begin(false, env.wall);
        let (kind, scope, number) = (default.kind, default.scope, default.number);
        answer(&mut core, &env, 2, Box::new([default]));
        assert_eq!(core.start_failure(), Some(Failure::Application { kind, scope, number, why }));
    }
}

#[test]
fn stored_states_and_defaults_share_the_kinds_per_scope_bound() {
    let env = environment();
    let mut core = Core::new(configuration(), &env.limits).unwrap();
    let _begun = core.start_begin(false, env.wall);
    for number in 1..=3 {
        let row = CoreRecord::Settings(s::Record::Application {
            setting: setting(1, 0, number),
            state: app::State { follows: app::Follows::Changed { revision: 1 }, size: 1, charters: Box::default() },
        });
        let restored = core.restore_core(row, &env.limits);
        if number <= 2 {
            assert_eq!(restored, Restored::Live);
        } else {
            assert_eq!(restored, Restored::Rejected);
        }
    }
    assert_eq!(
        core.start_failure(),
        Some(Failure::Application { kind: 1, scope: 0, number: 3, why: ApplicationWhy::Count })
    );
    let mut core = Core::new(configuration(), &env.limits).unwrap();
    let _begun = core.start_begin(false, env.wall);
    answer(
        &mut core,
        &env,
        2,
        Box::new([
            default(1, true),
            connector::DefaultState { number: 2, ..default(1, true) },
            connector::DefaultState { number: 3, ..default(1, true) },
        ]),
    );
    assert_eq!(
        core.start_failure(),
        Some(Failure::Application { kind: 1, scope: 0, number: 1, why: ApplicationWhy::Count })
    );
}

#[test]
fn project_deletion_erases_its_states_with_release_project_and_preserves_other_scopes() {
    let env = environment();
    let configured =
        Config { kinds: Box::new([kind(1, 2, KindScope::Deployment), kind(2, 2, KindScope::Project)]), ..config(41) };
    let mut core = Core::new(configured, &env.limits).unwrap();
    let _started = finish_start_without_defaults(&mut core, &env);
    for (kind, scope, follows) in [
        (1, 0, app::Follows::Default),
        (2, 1, app::Follows::Changed { revision: 1 }),
        (2, 1, app::Follows::Removed { revision: 1 }),
        (2, 2, app::Follows::Default),
    ] {
        let number = match follows {
            app::Follows::Removed { .. } => 2,
            app::Follows::Default | app::Follows::Changed { .. } => 1,
        };
        core.current
            .applications
            .insert(setting(kind, scope, number), app::State { follows, size: 1, charters: Box::default() })
            .unwrap();
    }
    let mut work = Queue::uncapped();
    let mut pending = Queue::uncapped();
    deletion::begin(&mut core, &env, &mut work, &mut pending, Token::new(90), 1, None).unwrap();
    deletion::tasks_erased(&mut core, &env, &mut pending, 1);
    let mut erased = List::with_capacity(4);
    let mut released = List::with_capacity(2);
    while let Some(request) = pending.pop() {
        match request {
            Request::Held(held) => {
                if let Held::NotesLoad { owner, .. } = *held {
                    let mut follow = step(
                        &mut core,
                        &env,
                        Event::Notes(notes::Event::Loaded {
                            owner,
                            rows: notes::Rows { records: List::with_capacity(env.limits.notes.load_rows) },
                            more: false,
                        }),
                    );
                    while let Some(request) = follow.pop() {
                        pending.push(request);
                    }
                }
            }
            Request::Write(Write::Erase(Key::Core(CoreKey::Settings(s::Key::Changed(setting))))) => {
                erased.push(setting).unwrap();
            }
            Request::Ask { connector, ask: Ask::ReleaseProject { project: 1 } } => {
                assert!(!core.current.applications.contains_key(&setting(2, 1, 1)));
                assert!(!core.current.applications.contains_key(&setting(2, 1, 2)));
                released.push(connector).unwrap();
            }
            Request::ProjectMade { .. } | Request::Write(_) | Request::Ask { .. } | Request::Now(_) => {}
        }
    }
    assert_eq!(erased.as_slice(), &[setting(2, 1, 1), setting(2, 1, 2)]);
    assert_eq!(released.as_slice(), &[1, 2]);
    assert!(core.current.applications.contains_key(&setting(1, 0, 1)));
    assert!(core.current.applications.contains_key(&setting(2, 2, 1)));
}
fn finish_start_without_defaults(core: &mut Core, env: &Env<Limits>) -> Box<[Request]> {
    let action = core.start_begin(false, env.wall);
    complete(core, env, action, Box::default())
}

fn collect(mut out: Queue<Request>) -> Box<[Request]> {
    let mut requests = List::with_capacity(out.len());
    while let Some(request) = out.pop() {
        requests.push(request).unwrap();
    }
    requests.into_boxed()
}

#[test]
fn jig_and_application_defaults_make_one_release_and_save_its_revision() {
    let (mut original, env, out) = first();
    let rows = snapshot(&mut original, &out);
    let mut configured = configuration();
    configured.defaults.charters[0].policy.instructions = b"Changed instructions".as_slice().into();
    let mut core = restart(configured, &env, &rows);
    let out = complete(&mut core, &env, Action::Step(Phase::LoadCore), Box::new([default(6, true)]));
    let revisions = revisions(&out);
    assert_eq!(revisions.len(), 1);
    assert_eq!(revisions[0].head.number, 2);
    assert_eq!(revisions[0].items.len(), 1);
    assert_eq!(revisions[0].items[0].setting, s::Setting::Charter(1));
    assert_eq!(revisions[0].applications.len(), 1);
    assert_eq!(saved(&out).as_ref(), &[(2, Some(2))]);
}

#[test]
fn a_restored_change_holds_its_charters_against_the_supplied_defaults() {
    let (mut original, env, out) = first();
    let mut rows = snapshot(&mut original, &out);
    let row = Record::Core(CoreRecord::Settings(s::Record::Application {
        setting: setting(1, 0, 1),
        state: app::State { follows: app::Follows::Changed { revision: 1 }, size: 3, charters: Box::new([99]) },
    }));
    rows.insert(row.key(), row).unwrap();
    let mut core = restart(configuration(), &env, &rows);
    answer(&mut core, &env, 2, Box::default());
    let mut out = Queue::uncapped();
    let mut action = Action::Step(Phase::LoadCore);
    for _ in 0_u32..8 {
        let Action::Step(phase) = action else { panic!("restored load precedes Link") };
        if phase == Phase::Link {
            break;
        }
        action = core.start_done(&env, phase, &mut out);
    }
    let Action::Refused(failure) = core.start_done(&env, Phase::Link, &mut out) else {
        panic!("changed reference refused")
    };
    assert_eq!(
        failure,
        Failure::Settings {
            failure: s::Failure { setting: setting(1, 0, 1), fault: s::Fault::Reference },
            changed: true
        }
    );
    assert!(saved(&collect(out)).is_empty());
}

#[test]
fn first_start_waits_before_seeding_and_recovers_a_header_without_settings() {
    for header_only in [false, true] {
        let env = environment();
        let mut core = Core::new(configuration(), &env.limits).unwrap();
        let action = core.start_begin(false, env.wall);
        if header_only {
            let mut header = core.counters.deployment();
            header.commits = 1;
            assert_eq!(
                core.restore_core(CoreRecord::Deployment(header), &env.limits),
                Restored::Deployment { commits: 1 }
            );
        }
        let before = finish_started(&mut core, &env, action);
        assert!(before.is_empty(), "no seed is persisted before its defaults arrive");
        assert_eq!(core.start_phase(), Some(Phase::Link));
        answer(&mut core, &env, 2, Box::new([default(3, true)]));
        let mut out = Queue::uncapped();
        let action = core.start_done(&env, Phase::Link, &mut out);
        let mut following = finish_started(&mut core, &env, action);
        while let Some(request) = following.pop() {
            out.push(request);
        }
        let out = collect(out);
        let revisions = revisions(&out);
        assert_eq!(revisions.len(), 1);
        assert_eq!(revisions[0].head.number, 1);
        assert_eq!(revisions[0].head.by, s::Decider::FirstStart);
        assert_eq!(revisions[0].applications.len(), 1);
        assert_eq!(saved(&out).as_ref(), &[(2, Some(1))]);
        assert!(core.open_project(1));
    }
}

#[test]
fn an_undeclared_default_is_refused_even_without_declared_kind_owners() {
    let env = environment();
    let mut core = Core::new(config(41), &env.limits).unwrap();
    let _begun = core.start_begin(false, env.wall);
    answer(&mut core, &env, 2, Box::new([default(1, true)]));
    assert_eq!(
        core.start_failure(),
        Some(Failure::Application { kind: 1, scope: 0, number: 1, why: ApplicationWhy::Undeclared })
    );
}
