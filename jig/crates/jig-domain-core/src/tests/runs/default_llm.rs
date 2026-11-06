//! The root keeps an omitted LLM choice omitted and a given one as given, and refuses a
//! task whose authority allows none of its charter's models (domain/tasks.md, section 3).
use crate::tests::*;

const FUNDER: tasks::Funder = tasks::Funder::Period { project: 1, period: 1 };

fn chosen(effort: u8) -> tasks::LlmChoice {
    tasks::LlmChoice {
        account: None,
        main: tasks::ModelEffort { model: 0, effort: Some(effort) },
        subagents: Box::new([]),
    }
}

#[test]
fn members_and_template_members_naming_no_choice_keep_none() {
    let (core, _) = tree_core();
    let mut given = new_row(&core, 2, FUNDER, None);
    given.llm = Some(chosen(1));
    let mut procedure = new_row(&core, 3, FUNDER, None);
    procedure.executor = tasks::Executor::Procedure { connector: 1, code: 1 };
    procedure.recurring = Some(Box::new(tasks::RecurringTemplate {
        key: 1,
        batch: Box::new([new_row(&core, 4, FUNDER, None)]),
        overlap: tasks::RecurringOverlap::Skip,
    }));
    let mut batch = [new_row(&core, 1, FUNDER, None), given, procedure];
    core.resolve_llm(&mut batch).unwrap();
    let [defaulted, given, procedure] = batch;
    assert_eq!(defaulted.llm, None, "the run check resolves an omitted choice");
    assert_eq!(given.llm, Some(chosen(1)), "a given choice is kept");
    assert_eq!(procedure.llm, None, "a procedure runs no model");
    assert_eq!(procedure.recurring.unwrap().batch.first().unwrap().llm, None);
}

#[test]
fn a_member_whose_authority_allows_none_of_its_charter_s_models_is_refused() {
    let (core, _) = tree_core();
    let mut barred = new_row(&core, 2, FUNDER, None);
    barred.authority.models = tasks::Models::NONE;
    let mut batch = [new_row(&core, 1, FUNDER, None), barred];
    assert_eq!(
        core.resolve_llm(&mut batch),
        Err(tasks::Problem::new(Some(2), tasks::Refusal::Llm { model: 0, why: tasks::LlmFault::Authority }))
    );
}
