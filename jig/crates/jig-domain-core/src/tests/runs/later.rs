//! A task's next run and document follow its activation (jig/docs/design/domain/tasks.md, 3).
use crate::tests::*;
use tasks::{LlmChoice, LlmLater, ModelEffort, Reasons};

fn first() -> LlmChoice {
    LlmChoice {
        account: Some(0),
        main: ModelEffort { model: 0, effort: Some(1) },
        subagents: Box::new([]),
        later: None,
    }
}

fn later() -> LlmChoice {
    LlmChoice {
        later: Some(LlmLater {
            main: ModelEffort { model: 0, effort: Some(0) },
            subagents: Box::new([ModelEffort { model: 0, effort: Some(1) }]),
        }),
        ..first()
    }
}

#[test]
fn next_run_and_document_take_later_after_the_first_activation() {
    let flattened = LlmChoice {
        main: ModelEffort { model: 0, effort: Some(0) },
        subagents: Box::new([ModelEffort { model: 0, effort: Some(1) }]),
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
fn a_partys_choice_carries_its_later_models_and_efforts() {
    let given = people::LlmChoice {
        account: Some(0),
        main: people::ModelEffort { model: 0, effort: Some(1) },
        subagents: Box::new([]),
        later: Some(people::LlmLater {
            main: people::ModelEffort { model: 0, effort: Some(0) },
            subagents: Box::new([people::ModelEffort { model: 1, effort: None }]),
        }),
    };
    let mut expected = later();
    expected.later.as_mut().unwrap().subagents = Box::new([ModelEffort { model: 1, effort: None }]);
    assert_eq!(crate::runs::llm::choice(Some(&given)), Some(expected));
}
