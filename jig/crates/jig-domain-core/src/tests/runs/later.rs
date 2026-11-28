//! A task's next run and document follow its activation (jig/docs/design/domain/tasks/README.md, 3).
use crate::tests::*;
use tasks::{LlmChoice, LlmLater, ModelEffort, Reasons};

fn first() -> LlmChoice {
    LlmChoice {
        account: Some(0),
        main: ModelEffort { model: 0, effort: Some(1), speed: None },
        subagents: Box::new([]),
        later: None,
    }
}

fn later() -> LlmChoice {
    LlmChoice {
        later: Some(LlmLater {
            main: ModelEffort { model: 0, effort: Some(0), speed: None },
            subagents: Box::new([ModelEffort { model: 0, effort: Some(1), speed: None }]),
        }),
        ..first()
    }
}

#[test]
fn next_run_and_document_take_later_after_the_first_activation() {
    let flattened = LlmChoice {
        main: ModelEffort { model: 0, effort: Some(0), speed: None },
        subagents: Box::new([ModelEffort { model: 0, effort: Some(1), speed: None }]),
        ..first()
    };
    let default = LlmChoice { account: None, subagents: Box::new([]), ..flattened.clone() };
    for (name, given, reasons, resumes, next, document) in [
        ("first", Some(later()), Reasons::FIRST, false, later(), later()),
        ("later", Some(later()), Reasons::EMPTY, false, flattened.clone(), flattened),
        ("without later", Some(first()), Reasons::EMPTY, false, first(), first()),
        ("now resuming", Some(later()), Reasons::EMPTY, true, later(), later()),
        ("default", None, Reasons::EMPTY, false, default.clone(), default),
    ] {
        let (mut core, limits) = core();
        make_task(&mut core, &limits, 1);
        super::run_choice::account(&mut core, &limits, accounts::Event::Want);
        super::run_choice::granted(&mut core, &limits, 1);
        core.current.test_charter_mut(1).policy.resume = resumes;
        let mut row = core.tasks.task(1).unwrap().clone();
        row.llm = given;
        row.reasons = reasons;
        assert_eq!(core.next_llm(&row), Some(next), "{name}: run");
        assert_eq!(core.document_llm(&row), Some(document), "{name}: document");
    }
}

#[test]
fn a_partys_choice_carries_its_models_efforts_and_speeds() {
    let fast = Some(skein_lib::Name::from_checked(b"fast").unwrap());
    let standard = Some(skein_lib::Name::from_checked(tasks::STANDARD_SPEED).unwrap());
    let given = people::LlmChoice {
        account: Some(0),
        main: people::ModelEffort { model: 0, effort: Some(1), speed: fast },
        subagents: Box::new([]),
        later: Some(people::LlmLater {
            main: people::ModelEffort { model: 0, effort: Some(0), speed: None },
            subagents: Box::new([people::ModelEffort { model: 1, effort: None, speed: standard }]),
        }),
    };
    let mut expected = later();
    expected.main.speed = fast;
    expected.later.as_mut().unwrap().subagents = Box::new([ModelEffort { model: 1, effort: None, speed: standard }]);
    assert_eq!(crate::runs::llm::choice(Some(&given)), Some(expected));
}
