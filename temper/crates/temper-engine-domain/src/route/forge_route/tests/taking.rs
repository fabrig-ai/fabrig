//! Parameter 19 starts without a producer (temper/docs/design/domain/forge/change/opened.md, 3.1).
use super::super::{self as route, forge, forge_change as change, tasks};

fn start(text: &[u8], beside: Option<tasks::Parameter>) -> (bool, route::Domain, u64) {
    let (mut domain, env, mut context) = super::change_start_fixture();
    let mut parameters = skein_lib::List::uncapped();
    parameters.push(context.spec.parameters[0].clone()).unwrap();
    parameters.push(tasks::Parameter::Bytes { name: 19, value: text.into() }).unwrap();
    parameters.push(tasks::Parameter::Bytes { name: 10, value: b"The title".as_slice().into() }).unwrap();
    if let Some(parameter) = beside {
        parameters.push(parameter).unwrap();
    }
    context.spec.parameters = parameters.into_boxed();
    let accepted = route::start_change(&mut domain, &env, &context);
    (accepted, domain, context.task)
}

#[test]
#[expect(clippy::disallowed_methods, reason = "parameter text fixtures join branch and commit")]
fn a_source_starts_taking_on_its_own_held_branch_at_either_commit_width() {
    for (text, at) in [
        (b"docs/x".to_vec(), None),
        ([b"docs/x:".as_slice(), super::COMMITS[0].1].concat(), Some(super::SHA1)),
        ([b"docs/x:".as_slice(), super::COMMITS[1].1].concat(), Some(super::SHA256)),
    ] {
        let (accepted, domain, task) = start(&text, None);
        assert!(accepted);
        let row = domain.forge.change(task).unwrap();
        assert_eq!(row.source, Some(forge::ChangeSource { branch: b"docs/x".as_slice().into(), at }));
        assert_eq!(row.kept, None);
        assert_eq!(row.change.state, change::State::Taking { requested: false });
        assert_eq!(row.title.as_ref(), b"The title");
        assert_eq!(row.body.as_ref(), b"Make the fix");
        assert!(row.delegate.is_none());
        let root = domain.core.tasks.root(task).unwrap();
        let mut expected = skein_lib::List::uncapped();
        for bytes in [b"temper/".as_slice(), &route::decimal(root), b"/c", &route::decimal(task)] {
            route::append(&mut expected, bytes);
        }
        assert_eq!(row.branch.as_ref(), expected.as_slice());
        let mut held = false;
        for work in &domain.work {
            if let route::Work::Forge(forge::Event::Hold { task: holder, resource, .. }) = work {
                assert_eq!(*holder, task);
                assert_eq!(resource.what, forge::What::Branch(route::branch_parts(&row.branch, 4096).unwrap()));
                held = true;
            }
        }
        assert!(held);
    }
}

#[test]
#[expect(
    clippy::disallowed_methods,
    clippy::disallowed_macros,
    reason = "malformed parameter fixtures join and repeat bounded byte strings"
)]
fn malformed_sources_and_their_excluded_parameters_refuse_the_start() {
    let (_, env, _) = super::change_start_fixture();
    let at_bound = alloc::vec![b'x'; usize::try_from(env.limits.forge.name_bytes).unwrap()];
    assert!(start(&at_bound, None).0);
    for text in [
        [b"docs/x:".as_slice(), &[b'A'; 40]].concat(),
        [b"docs/x:".as_slice(), &[b'a'; 39]].concat(),
        [b":".as_slice(), &[b'a'; 40]].concat(),
        alloc::vec![b'x'; usize::try_from(env.limits.forge.name_bytes).unwrap() + 1],
        b"docs/x:".to_vec(),
        [b"docs:x:".as_slice(), &[b'a'; 40]].concat(),
    ] {
        let (accepted, domain, task) = start(&text, None);
        assert!(!accepted, "{text:?}");
        assert!(domain.forge.change(task).is_none());
        assert!(domain.work.is_empty());
    }
    for parameter in [
        tasks::Parameter::Bytes { name: 3, value: b"handed".as_slice().into() },
        tasks::Parameter::Number { name: 11, value: 1 },
        tasks::Parameter::Bytes { name: 16, value: super::COMMITS[0].1.into() },
        tasks::Parameter::Bytes { name: 17, value: b"close".as_slice().into() },
        tasks::Parameter::Bytes { name: 18, value: [b"3:".as_slice(), super::COMMITS[0].1].concat().into() },
    ] {
        let (accepted, domain, task) = start(b"docs/x", Some(parameter));
        assert!(!accepted);
        assert!(domain.forge.change(task).is_none());
        assert!(domain.work.is_empty());
    }
}

#[test]
fn taking_accounts_reach_the_task_under_the_existing_stalled_and_effect_failed_holds() {
    let (accepted, mut domain, task) = start(b"docs/x", None);
    assert!(accepted);
    let (_, env, _) = super::change_start_fixture();
    let words = b"taking docs/x: forge unavailable";
    for (cause, expected) in
        [(change::Hold::Stalled, tasks::Hold::Stalled), (change::Hold::Failed, tasks::Hold::EffectFailed)]
    {
        let mut row = domain.forge.change(task).unwrap().clone();
        row.change.state =
            change::State::Held { was: Box::new(change::State::Taking { requested: false }), why: cause };
        row.effect_account = Some(forge::EffectAccount { entry: 0, words: words.as_slice().into() });
        forge::step(
            &mut domain.forge,
            &env.with(env.limits.forge),
            forge::Event::Restore { record: forge::Stored::Change(Box::new(row)) },
            &mut skein_lib::Queue::uncapped(),
        );
        let decision = route::hold_change_decision(&domain, &env, task, cause, None);
        let route::StepEnd::HoldFrom { why, source } = decision.end else { panic!("taking hold has an account") };
        assert_eq!(why, expected);
        let tasks::HoldSource::Account { words: held, .. } = source.as_ref() else { panic!("taking keeps its words") };
        assert_eq!(held.as_ref(), words);
    }
}
