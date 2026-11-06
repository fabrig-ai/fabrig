//! The root keeps an omitted LLM choice omitted and a given one as given, and refuses a
//! task whose authority allows none of its charter's models (domain/tasks.md, section 3).
use crate::tests::*;

const FUNDER: tasks::Funder = tasks::Funder::Period { project: 1, period: 1 };
const TASK: u64 = 1;

fn model_with_default() -> Model {
    let mut model = scripted_model();
    model.efforts = Box::new([b"low".as_slice().into(), b"medium".as_slice().into(), b"high".as_slice().into()]);
    model.effort = Some(b"medium".as_slice().into());
    model
}

fn task_with_default(choice: Option<tasks::LlmChoice>) -> Core {
    let (base, limits) = tree_core();
    let mut configuration = config(17);
    let main = model_with_default();
    let mut subagent = main.clone();
    subagent.name = b"subagent".as_slice().into();
    subagent.effort = Some(b"high".as_slice().into());
    configuration.settings.models = Box::new([main.clone(), subagent.clone()]);

    configuration.defaults.charters[0].policy.model = main;
    configuration.defaults.charters[0].policy.alternatives = Box::new([subagent]);
    let mut core = seeded_core(configuration, &limits).unwrap();
    core.people = base.people;
    core.accounts = base.accounts;
    send_task(&mut core, &limits, tasks::Event::Check);
    send_task(&mut core, &limits, tasks::Event::Restored);
    send_task(
        &mut core,
        &limits,
        tasks::Event::OpenPeriod { reply_to: ReplyTo::new(Token::new(1)), project: 1, period: 1, budget: 1000 },
    );
    let mut row = new_row(&core, TASK, FUNDER, None);
    row.llm = choice;
    let mut batch = [row];
    core.resolve_llm(&mut batch).unwrap();
    fixtures::make_batch(&mut core, &limits, tasks::Party::Person(1), batch.into());
    core
}

#[test]
fn a_models_default_is_named_else_first_else_none() {
    let mut model = model_with_default();
    assert_eq!(model.default_effort(), Some(b"medium".as_slice()));
    model.effort = None;
    assert_eq!(model.default_effort(), Some(b"low".as_slice()));
    model.efforts = Box::new([]);
    assert_eq!(model.default_effort(), None);
}

#[test]
fn an_omitted_choice_takes_each_models_own_default() {
    let core = task_with_default(None);
    let choice = core.next_llm(core.tasks.task(TASK).unwrap()).unwrap();
    assert_eq!(choice.main, tasks::ModelEffort { model: 0, effort: Some(1) });
    assert_eq!(choice.subagents.as_ref(), [tasks::ModelEffort { model: 1, effort: Some(2) }]);
    let policy = core.chosen_policy(TASK);
    assert_eq!(policy.model.effort.as_deref(), Some(b"medium".as_slice()));
    assert_eq!(policy.alternatives[0].effort.as_deref(), Some(b"high".as_slice()));
}

#[test]
fn a_given_choice_keeps_low_despite_the_models_default() {
    let core = task_with_default(Some(chosen(0)));
    assert_eq!(core.next_llm(core.tasks.task(TASK).unwrap()).unwrap(), chosen(0));
    assert_eq!(core.chosen_policy(TASK).model.effort.as_deref(), Some(b"low".as_slice()));
}

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
