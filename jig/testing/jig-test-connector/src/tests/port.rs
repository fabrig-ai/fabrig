//! Judgements and releases at the connector's in-process boundary.
use super::{LIMITS, domain_with_limits, service};
use crate::{Domain, Effect, Event, Form, KindSpec, Limits, Out, Record, Recovery, SettingsKind, SystemRequest};
use jig::{effects, settings};
use jig_domain_contract as jig;
use skein_lib::{Env, List, Queue, Time, Token, Wall};

const PORT_LIMITS: Limits = Limits { kinds: 1, staged: 1, entries: 1, made: 1, resources_per_effect: 1, ..LIMITS };

fn domain() -> Domain {
    domain_with_limits(&PORT_LIMITS)
}

fn step(domain: &mut Domain, event: Event) -> List<Out> {
    let env = Env { now: Time::ZERO, wall: Wall::from_nanos(1), limits: PORT_LIMITS };
    let mut out = Queue::uncapped();
    crate::step(domain, &env, event, &mut out);
    let mut results = List::with_capacity(out.len());
    while let Some(result) = out.pop() {
        results.push(result).expect("one slot per output");
    }
    results
}

#[test]
fn settings_judgement_names_the_asked_owner() {
    let mut domain = domain();
    domain.config.settings = Some(SettingsKind {
        kind: 9,
        project: false,
        value_bytes: 16,
        per_scope: 4,
        charters: false,
        defaults: Box::new([]),
    });
    let owner = Token::new(31);
    let out = step(
        &mut domain,
        Event::Ask(jig::Ask::Settings(settings::Ask::Judge {
            owner,
            revision: 7,
            proposal: None,
            items: Box::new([settings::Item {
                kind: 9,
                scope: 0,
                number: 1,
                edit: settings::Edit::Set(Box::from(&b"new"[..])),
            }]),
        })),
    );
    let [Out::Up(jig::Up::Settings(settings::Up::Judged { owner: answered, judged }))] = out.as_slice() else {
        panic!("the step must answer the settings judgement: {out:?}");
    };
    assert_eq!(*answered, owner);
    let [settings::Judged::Accept { size: 3, charters }] = judged.as_ref() else { panic!("one judged item") };
    assert!(charters.is_empty());
}

fn start_effect(domain: &mut Domain, owner: Token) {
    domain.config.kinds = Box::new([KindSpec {
        kind: 5,
        form: Form::Set,
        recovery: Recovery::Keyed,
        price: None,
        waits_for_dependencies: false,
    }]);
    let effect =
        Effect { kind: 5, resources: Box::new([service()]), purpose: 12, state: 42, target: 43, condition: None };
    let origin = effects::EffectOrigin::Projection { goal: 17, entry: None };
    let out = step(domain, Event::StageEffect { owner, effect, start: Some(origin) });
    let [
        Out::Up(jig::Up::Effects(effects::Up::Start {
            owner: started,
            origin: effects::EffectOrigin::Projection { goal: 17, entry: None },
        })),
    ] = out.as_slice()
    else {
        panic!("the staged effect starts")
    };
    assert_eq!(*started, owner);
}

fn describe(domain: &mut Domain, owner: Token) -> List<Out> {
    step(domain, Event::Ask(jig::Ask::Effects(effects::Ask::Describe { owner })))
}

#[test]
fn describes_its_started_effect_in_the_asking_step() {
    let mut domain = domain();
    let owner = Token::new(9);
    start_effect(&mut domain, owner);
    let out = describe(&mut domain, owner);
    let [Out::Up(jig::Up::Effects(effects::Up::Described { owner: described, description }))] = out.as_slice() else {
        panic!("the connector must describe the started effect: {out:?}");
    };
    assert_eq!(*described, owner);
    assert_eq!(description.purpose, 12);
    assert_eq!(description.effect.kind, 5);
    assert_eq!(description.effect.name.segments.as_ref(), service().segments());
}

#[test]
fn released_make_makes_only_the_named_entry() {
    let mut domain = domain();
    let owner = Token::new(9);
    start_effect(&mut domain, owner);
    drop(describe(&mut domain, owner));
    let entry = 12;
    let task = 17;
    let key = effects::EffectKey {
        deployment: [1; 16],
        task,
        purpose: 12,
        origin: effects::EffectPurpose::Projection { purpose: 12 },
    };
    let kept = step(
        &mut domain,
        Event::Ask(jig::Ask::Effects(effects::Ask::Keep { owner, entry, task, key, project: Some(1) })),
    );
    let [Out::Save(Record::Outbox(row))] = kept.as_slice() else { panic!("no apply before release") };
    assert_eq!(row.number, entry);
    assert!(step(&mut domain, Event::Released(jig::Released::Make { entry: 13 })).is_empty());
    let made = step(&mut domain, Event::Released(jig::Released::Make { entry }));
    let [Out::Save(Record::Outbox(row)), Out::System(SystemRequest::Apply { entry: applied, .. })] = made.as_slice()
    else {
        panic!("only the released entry makes an attempt: {made:?}");
    };
    assert_eq!((row.number, *applied), (entry, entry));
}
