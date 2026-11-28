//! The root keeps an omitted LLM choice omitted and a given one as given, and refuses a
//! task whose authority allows none of its charter's models (domain/tasks/README.md, section 3).
use crate::tests::*;

const FUNDER: tasks::Funder = tasks::Funder::Period { project: 1, period: 1 };
const TASK: u64 = 1;

fn model_with_default() -> Model {
    let mut model = scripted_model();
    model.efforts = Box::new([b"low".as_slice().into(), b"medium".as_slice().into(), b"high".as_slice().into()]);
    model.effort = Some(b"medium".as_slice().into());
    model
}

fn task_with_default(
    choice: Option<tasks::LlmChoice>,
    main_effort: Option<&str>,
    subagent_effort: Option<&str>,
) -> Core {
    let mut configuration = config(17);
    let mut main = model_with_default();
    let mut subagent = main.clone();
    subagent.name = b"subagent".as_slice().into();
    subagent.effort = Some(b"high".as_slice().into());
    configuration.settings.models = Box::new([main.clone(), subagent.clone()]);

    main.effort = match main_effort {
        Some(effort) => Some(effort.as_bytes().into()),
        None => None,
    };
    subagent.effort = match subagent_effort {
        Some(effort) => Some(effort.as_bytes().into()),
        None => None,
    };
    configuration.defaults.charters[0].policy.model = main;
    configuration.defaults.charters[0].policy.alternatives = Box::new([subagent]);
    with_task(configuration, choice)
}

fn with_task(configuration: Config, choice: Option<tasks::LlmChoice>) -> Core {
    let (base, limits) = tree_core();
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
    let batch = [row];
    core.resolve_llm(&limits, &batch, &[]).unwrap();
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
    for (main, subagent, main_index, subagent_index, main_name, subagent_name) in [
        (None, None, 1, 2, "medium", "high"),
        (Some("low"), Some("medium"), 0, 1, "low", "medium"),
        (Some("high"), None, 2, 2, "high", "high"),
    ] {
        let core = task_with_default(None, main, subagent);
        let choice = core.next_llm(core.tasks.task(TASK).unwrap()).unwrap();
        assert_eq!(
            choice.main,
            tasks::ModelEffort { model: 0, effort: Some(main_index), speed: None },
            "{main:?}, {subagent:?}"
        );
        assert_eq!(
            choice.subagents.as_ref(),
            [tasks::ModelEffort { model: 1, effort: Some(subagent_index), speed: None }],
            "{main:?}, {subagent:?}"
        );
        let policy = core.chosen_policy(TASK);
        assert_eq!(policy.model.effort.as_deref(), Some(main_name.as_bytes()), "{main:?}, {subagent:?}");
        assert_eq!(policy.alternatives[0].effort.as_deref(), Some(subagent_name.as_bytes()), "{main:?}, {subagent:?}");
    }
}

#[test]
fn a_given_choice_keeps_low_despite_the_models_default() {
    let core = task_with_default(Some(chosen(0)), Some("high"), None);
    assert_eq!(core.next_llm(core.tasks.task(TASK).unwrap()).unwrap(), chosen(0));
    assert_eq!(core.chosen_policy(TASK).model.effort.as_deref(), Some(b"low".as_slice()));
}

fn chosen(effort: u8) -> tasks::LlmChoice {
    tasks::LlmChoice {
        later: None,
        account: None,
        main: tasks::ModelEffort { model: 0, effort: Some(effort), speed: None },
        subagents: Box::new([]),
    }
}

fn with_speeds(choice: Option<tasks::LlmChoice>) -> Core {
    let mut configuration = config(17);
    let mut main = model_with_default();
    main.name = b"gpt-x".as_slice().into();
    speed_example(&mut main);
    main.window = 10;
    main.output = 2;
    main.price_unit = 1_000;
    main.speed = Some(b"fast".as_slice().into());
    let mut subagent = main.clone();
    subagent.name = b"subagent".as_slice().into();
    subagent.speed = None;
    configuration.settings.models = Box::new([main.clone(), subagent.clone()]);
    let policy = &mut configuration.defaults.charters[0].policy;
    policy.model = main;
    policy.reserve_turns = 0;
    policy.alternatives = Box::new([subagent]);
    with_task(configuration, choice)
}

#[test]
fn chosen_speeds_set_main_cost_and_subagent_speed() {
    let mut choice = chosen(0);
    choice.main.speed = Some(skein_lib::Name::from_checked(b"ultrafast").unwrap());
    choice.subagents = Box::new([tasks::ModelEffort {
        model: 1,
        effort: Some(1),
        speed: Some(skein_lib::Name::from_checked(b"fast").unwrap()),
    }]);
    let core = with_speeds(Some(choice));
    let policy = core.chosen_policy(TASK);
    assert_eq!(policy.model.name.as_ref(), b"gpt-x");
    assert_eq!(policy.model.speed.as_deref(), Some(b"ultrafast".as_slice()));
    assert_eq!(most_costly(&policy.model), Some(270)); // 10 cached-write input tokens and 2 output at Ultrafast.
    assert_eq!(core.run_reserve(TASK), 270); // One completion with zero reserve turns.
    assert_eq!(policy.alternatives[0].speed.as_deref(), Some(b"fast".as_slice()));
}

#[test]
fn standard_overrides_fast_and_an_omitted_speed_inherits_it() {
    let fast = Some(b"fast".as_slice());
    for (llm, expected, request) in [
        (None, fast, None),
        (Some(chosen(0)), fast, None),
        (
            Some(tasks::LlmChoice {
                main: tasks::ModelEffort {
                    speed: Some(skein_lib::Name::from_checked(tasks::STANDARD_SPEED).unwrap()),
                    ..chosen(0).main
                },
                ..chosen(0)
            }),
            None,
            Some(b"default".as_slice()),
        ),
    ] {
        let core = with_speeds(llm);
        let model = core.chosen_policy(TASK).model;
        assert_eq!(model.speed.as_deref(), expected);
        assert_eq!(if model.speed.is_none() { model.standard_speed_override() } else { None }, request);
    }
}

#[test]
fn members_and_template_members_naming_no_choice_keep_none() {
    let (core, limits) = tree_core();
    let mut given = new_row(&core, 2, FUNDER, None);
    given.llm = Some(chosen(1));
    let mut procedure = new_row(&core, 3, FUNDER, None);
    procedure.executor = tasks::Executor::Procedure { connector: 1, code: 1 };
    procedure.recurring = Some(Box::new(tasks::RecurringTemplate {
        key: 1,
        batch: Box::new([new_row(&core, 4, FUNDER, None)]),
        overlap: tasks::RecurringOverlap::Skip,
    }));
    let batch = [new_row(&core, 1, FUNDER, None), given, procedure];
    core.resolve_llm(&limits, &batch, &[]).unwrap();
    let [defaulted, given, procedure] = batch;
    assert_eq!(defaulted.llm, None, "the run check resolves an omitted choice");
    assert_eq!(given.llm, Some(chosen(1)), "a given choice is kept");
    assert_eq!(procedure.llm, None, "a procedure runs no model");
    assert_eq!(procedure.recurring.unwrap().batch.first().unwrap().llm, None);
}

#[test]
fn a_member_whose_authority_allows_none_of_its_charter_s_models_is_refused() {
    let (core, limits) = tree_core();
    let mut barred = new_row(&core, 2, FUNDER, None);
    barred.authority.models = tasks::Models::NONE;
    let batch = [new_row(&core, 1, FUNDER, None), barred];
    assert_eq!(
        core.resolve_llm(&limits, &batch, &[]),
        Err(tasks::Problem::new(Some(2), tasks::Refusal::Llm { model: 0, why: tasks::LlmFault::Authority }))
    );
}
