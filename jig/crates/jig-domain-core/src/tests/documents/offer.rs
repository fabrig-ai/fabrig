//! The offer and previews share run selection (domain/engine.md, 11.1; accounts.md, 4.3).
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros,
    reason = "unit fixtures are ordinary Rust under programming-model.md, 10.2"
)]
use crate::tests::runs::run_choice::{account, granted, host, inventory, login};
use crate::tests::*;

const PARTY: u64 = 1;
const PROJECT: u32 = 1;
const OTHER_PROJECT: u32 = 2;
const TASK: u64 = 1;
const CHARTER: u32 = 1;
const PROVIDER: u32 = 1;
const OTHER_PROVIDER: u32 = 2;
const OWN: u32 = 20;
const SOURCE: u32 = 21;
const SECOND: u32 = 30;
const SECOND_SOURCE: u32 = 31;
const HOST: Token = Token::new(7);
const WATCHER: Token = Token::new(30);

fn fixture() -> (Core, Limits) {
    let (base, mut limits) = core();
    limits.accounts.accounts = 4;
    limits.accounts.holder_accounts = 4;
    limits.accounts.credentials = 4;
    limits.run_bytes = 4096;
    let mut configuration = config(41);
    let mut same = scripted_model();
    same.name = b"same".as_slice().into();
    let mut other = scripted_model();
    other.name = b"other".as_slice().into();
    other.efforts = Box::new([b"low".as_slice().into(), b"high".as_slice().into()]);
    other.effort = Some(b"high".as_slice().into());
    configuration.settings.models = Box::new([scripted_model(), same.clone(), other.clone()]);
    configuration.defaults.charters[0].policy.model = other;
    configuration.defaults.charters[0].policy.alternatives = Box::new([same, scripted_model()]);
    configuration.settings.llm_providers.providers[0].models =
        Box::new([b"scripted".as_slice().into(), b"same".as_slice().into()]);
    configuration.settings.llm_providers.providers[0].offers.key = true;
    let mut provider = configuration.settings.llm_providers.providers[0].clone();
    provider.number = OTHER_PROVIDER;
    provider.models = Box::new([b"other".as_slice().into()]);
    configuration.settings.llm_providers.providers =
        Box::new([configuration.settings.llm_providers.providers[0].clone(), provider]);
    let mut core = seeded_core(configuration, &limits).unwrap();
    core.people = base.people;
    core.accounts = accounts::Domain::new(&limits.accounts, core.settings.llm_providers.clone());
    (core, limits)
}

fn key(core: &mut Core, limits: &Limits, number: u32, source: u32, provider: u32) {
    restore_key_account(core, limits, number, source, provider);
    account(core, limits, accounts::Event::Want);
    granted(core, limits, source);
}

fn frame(core: &Core, limits: &Limits) -> Frame {
    let Document::Frame(value) =
        make(core, limits, PARTY, &Subject::Frame { party: PARTY }, &window(), Wall::EPOCH).unwrap()
    else {
        panic!("frame")
    };
    value
}

fn offered(core: &Core, limits: &Limits) -> Vec<(u8, Vec<u32>)> {
    frame(core, limits)
        .projects
        .get(0)
        .unwrap()
        .offer
        .iter()
        .map(|row| (row.model, row.accounts.as_slice().to_vec()))
        .collect()
}

fn template(number: u32, executor: people::TemplateExecutor, llm: Option<tasks::LlmChoice>) -> Template {
    Template {
        number,
        executor,
        llm,
        name: b"work".as_slice().into(),
        resources: Box::new([]),
        contract: tasks::Contract::Report { words: 32 },
        shallower: 0,
        budget: people::TemplateBudget::Given,
        deadline: people::TemplateDeadline::Given,
        tracked: None,
        flags: tasks::Flags::PLAIN,
        label: 0,
        grace: None,
    }
}

fn choice(model: u8, effort: u8) -> tasks::LlmChoice {
    tasks::LlmChoice {
        account: None,
        main: tasks::ModelEffort { model, effort: Some(effort) },
        subagents: Box::new([]),
    }
}

fn defaults(core: &Core, limits: &Limits) -> Vec<(u32, tasks::LlmChoice)> {
    frame(core, limits).projects.get(0).unwrap().defaults.as_slice().to_vec()
}

#[test]
fn only_a_served_providers_models_are_offered_in_configuration_order() {
    let (mut core, limits) = fixture();
    key(&mut core, &limits, OWN, SOURCE, PROVIDER);
    assert_eq!(offered(&core, &limits), [(0, vec![OWN]), (1, vec![OWN])]);
    assert_eq!(
        frame(&core, &limits).providers.iter().map(|row| row.provider).collect::<Vec<_>>(),
        [PROVIDER, OTHER_PROVIDER]
    );
}

fn program_login(core: &mut Core, limits: &Limits, state: fleet::LoginState) {
    inventory(
        core,
        limits,
        HOST,
        Box::new([login(b"personal", state), login(b"pending", fleet::LoginState::NotSignedIn)]),
    );
}

#[test]
fn keys_survive_missing_hosts_but_programs_require_current_logins() {
    let (mut core, limits) =
        runs::run_choice::placement_fixture(accounts::Holder::Party(PARTY), accounts::Holder::Party(PARTY), true);
    // The fixture's program account also has a later key; isolate the login.
    account(&mut core, &limits, accounts::Event::Limited { account: 1, reset: None });
    account(
        &mut core,
        &limits,
        accounts::Event::Change {
            request: Token::new(90),
            holder: accounts::Holder::Party(PARTY),
            change: accounts::Change::SetUses { account: 0, uses: Some(Box::new([Use::Party(PARTY)])) },
        },
    );
    assert_eq!(offered(&core, &limits), [(0, vec![0])]);
    drop(watch_outputs(
        &mut core,
        &limits,
        Event::Fleet(fleet::Event::Sources {
            sources: Box::new([
                fleet::SourceName::new(b"personal").unwrap(),
                fleet::SourceName::new(b"pending").unwrap(),
            ]),
        }),
    ));
    assert!(offered(&core, &limits).is_empty());
    program_login(&mut core, &limits, fleet::LoginState::Usable);
    assert_eq!(offered(&core, &limits), [(0, vec![0])]);
    program_login(&mut core, &limits, fleet::LoginState::NotSignedIn);
    assert!(offered(&core, &limits).is_empty());
    program_login(&mut core, &limits, fleet::LoginState::Usable);
    assert_eq!(offered(&core, &limits), [(0, vec![0])]);
    drop(watch_outputs(&mut core, &limits, Event::Fleet(fleet::Event::Lost { channel: HOST })));
    assert!(offered(&core, &limits).is_empty());

    let (mut core, limits) = fixture();
    key(&mut core, &limits, OWN, SOURCE, PROVIDER);
    assert_eq!(offered(&core, &limits), [(0, vec![OWN]), (1, vec![OWN])]);
    host(&mut core, &limits, HOST, &[b"scripted", b"same", b"other"]);
    drop(watch_outputs(
        &mut core,
        &limits,
        Event::Fleet(fleet::Event::Sources { sources: Box::new([fleet::SourceName::new(b"pending").unwrap()]) }),
    ));
    assert_eq!(offered(&core, &limits), [(0, vec![OWN]), (1, vec![OWN])]);
}

fn two_accounts() -> (Core, Limits) {
    let (mut core, limits) = fixture();
    for record in [
        accounts::Stored::Account {
            number: SECOND,
            provider: PROVIDER,
            holder: accounts::Holder::Party(PARTY),
            place: 0,
            uses: None,
            subject: b"second".as_slice().into(),
            display: Box::new([]),
        },
        accounts::Stored::Source {
            account: SECOND,
            number: SECOND_SOURCE,
            place: 0,
            kind: accounts::Kind::SignIn,
            name: Box::new([]),
            held: None,
        },
    ] {
        account(&mut core, &limits, accounts::Event::Restore { record });
    }
    key(&mut core, &limits, OWN, SOURCE, PROVIDER);
    granted(&mut core, &limits, SECOND_SOURCE);
    (core, limits)
}

#[test]
fn unusable_accounts_leave_only_usable_candidates_in_selection_order() {
    let (mut core, limits) = two_accounts();
    assert_eq!(offered(&core, &limits), [(0, vec![SECOND, OWN]), (1, vec![SECOND, OWN])]);
    make_task(&mut core, &limits, TASK);
    host(&mut core, &limits, HOST, &[b"scripted", b"same", b"other"]);
    assert_eq!(core.run_choice(TASK).unwrap().accounts[0].account, SECOND);
    for (label, event) in [
        ("limited", accounts::Event::Limited { account: SECOND_SOURCE, reset: None }),
        (
            "refused",
            accounts::Event::Refused { account: SECOND_SOURCE, code: b"401".as_slice().into(), reason: Box::new([]) },
        ),
        (
            "signed out",
            accounts::Event::SignOut {
                request: Token::new(91),
                holder: accounts::Holder::Party(PARTY),
                source: SECOND_SOURCE,
            },
        ),
        (
            "moving",
            accounts::Event::Change {
                request: Token::new(91),
                holder: accounts::Holder::Party(PARTY),
                change: accounts::Change::MoveOut { account: SECOND },
            },
        ),
    ] {
        let (mut core, limits) = two_accounts();
        account(&mut core, &limits, event);
        assert_eq!(offered(&core, &limits), [(0, vec![OWN]), (1, vec![OWN])], "{label}");
        account(&mut core, &limits, accounts::Event::Limited { account: SOURCE, reset: None });
        assert!(offered(&core, &limits).is_empty(), "{label}");
    }
}

#[test]
fn a_roles_model_mask_limits_the_offer() {
    let (mut core, limits) = fixture();
    key(&mut core, &limits, OWN, SOURCE, PROVIDER);
    let mut policy = core.authority.policy(PROJECT).unwrap().clone();
    policy.roles[0].authority.models = authority::Models::Named(0b010);
    authority::step(&mut core.authority, authority::Event::Policy { project: PROJECT, policy }, &mut Queue::uncapped());
    assert_eq!(offered(&core, &limits), [(1, vec![OWN])]);
}

#[test]
fn a_project_key_serves_the_party_only_in_that_project() {
    let (mut core, limits) = fixture();
    key(&mut core, &limits, OWN, SOURCE, PROVIDER);
    account(
        &mut core,
        &limits,
        accounts::Event::Change {
            request: Token::new(90),
            holder: accounts::Holder::Deployment,
            change: accounts::Change::SetUses { account: OWN, uses: Some(Box::new([Use::Project(PROJECT)])) },
        },
    );
    assert_eq!(offered(&core, &limits), [(0, vec![OWN]), (1, vec![OWN])]);
    assert!(core.offer(tasks::Payer::Party(PARTY), OTHER_PROJECT, tasks::Models::ALL).is_empty());
}

#[test]
fn previews_follow_charter_order_and_preserve_explicit_template_intent() {
    let (mut core, limits) = fixture();
    restore_key_account(&mut core, &limits, SECOND, SECOND_SOURCE, OTHER_PROVIDER);
    key(&mut core, &limits, OWN, SOURCE, PROVIDER);
    granted(&mut core, &limits, SECOND_SOURCE);
    let named = choice(0, 0);
    core.current.test_templates(Box::new([
        template(1, people::TemplateExecutor::Charter(CHARTER), None),
        template(2, people::TemplateExecutor::Charter(CHARTER), Some(named.clone())),
        template(3, people::TemplateExecutor::Chosen, None),
    ]));
    let mut preview = choice(2, 1);
    preview.subagents =
        Box::new([tasks::ModelEffort { model: 1, effort: Some(0) }, tasks::ModelEffort { model: 0, effort: Some(0) }]);
    assert_eq!(defaults(&core, &limits), [(1, preview), (2, named.clone())]);
    account(&mut core, &limits, accounts::Event::Limited { account: SECOND_SOURCE, reset: None });
    let mut preview = choice(1, 0);
    preview.subagents =
        Box::new([tasks::ModelEffort { model: 2, effort: Some(1) }, tasks::ModelEffort { model: 0, effort: Some(0) }]);
    assert_eq!(defaults(&core, &limits), [(1, preview), (2, named.clone())]);
    account(&mut core, &limits, accounts::Event::Limited { account: SOURCE, reset: None });
    assert_eq!(defaults(&core, &limits), [(2, named)]);
}

#[test]
fn an_omitted_tasks_run_takes_the_previews_main_model() {
    let (mut core, limits) = fixture();
    key(&mut core, &limits, OWN, SOURCE, PROVIDER);
    core.current.test_templates(Box::new([template(1, people::TemplateExecutor::Charter(CHARTER), None)]));
    let preview = defaults(&core, &limits).remove(0).1;
    make_task(&mut core, &limits, TASK);
    host(&mut core, &limits, HOST, &[b"scripted", b"same", b"other"]);
    let row = core.tasks.task(TASK).unwrap();
    assert_eq!(row.llm, None);
    assert_eq!(core.next_llm(row).unwrap().main, preview.main);
    assert_eq!(core.run_choice(TASK).unwrap().accounts[0].account, OWN);
}

#[test]
fn documents_omit_only_an_unserved_implicit_choice_and_keep_the_empty_wait() {
    let (mut core, limits) = fixture();
    make_task(&mut core, &limits, TASK);
    host(&mut core, &limits, HOST, &[b"scripted", b"same", b"other"]);
    let row = core.tasks.task(TASK).unwrap();
    let doc = task_from_row(&core, &limits, PARTY, row, Wall::EPOCH).unwrap();
    assert_eq!(doc.llm, None);
    assert_eq!(doc.run_wait, Some(TaskRunWait::Account { account: None, source: None, why: TaskWaitWhy::Empty }));
    key(&mut core, &limits, OWN, SOURCE, PROVIDER);
    account(&mut core, &limits, accounts::Event::Limited { account: SOURCE, reset: None });
    let mut row = core.tasks.task(TASK).unwrap().clone();
    assert_eq!(task_from_row(&core, &limits, PARTY, &row, Wall::EPOCH).unwrap().llm.unwrap().main.model, 1);
    row.llm = Some(choice(2, 1));
    assert_eq!(task_from_row(&core, &limits, PARTY, &row, Wall::EPOCH).unwrap().llm, row.llm);
}

fn watching() -> (Core, Limits) {
    let (mut core, mut limits) = fixture();
    let (auth, env) = documents::accounts_document::fixture(vec![]);
    core.people = auth.people;
    limits.people = env.limits.people;
    key(&mut core, &limits, OWN, SOURCE, PROVIDER);
    host(&mut core, &limits, HOST, &[b"scripted", b"same", b"other"]);
    drop(watch_outputs(
        &mut core,
        &limits,
        Event::Watch { watcher: WATCHER, sign_in: 10, subject: Watched::Frame, window: window() },
    ));
    drop(watch_outputs(&mut core, &limits, Event::Delivered { watcher: WATCHER }));
    drop(finish_watch_outputs(&mut core, &limits));
    (core, limits)
}

fn marked(core: &Core) -> bool {
    core.changed_views.contains(&core.named_watch(WATCHER).unwrap().subject)
}

#[test]
fn account_login_host_and_charter_changes_mark_me_with_unrelated_negatives() {
    let (mut core, limits) = watching();
    for event in [
        Event::Account(accounts::Event::Limited { account: SOURCE, reset: None }),
        Event::Fleet(fleet::Event::Logins { channel: HOST, logins: Box::new([]) }),
        Event::Fleet(fleet::Event::Lost { channel: HOST }),
    ] {
        drop(watch_outputs(&mut core, &limits, event));
        assert!(marked(&core));
        drop(finish_watch_outputs(&mut core, &limits));
    }
    let mut charter = core.charter(CHARTER).unwrap().clone();
    charter.policy.turns = charter.policy.turns.checked_add(1).unwrap();
    let change = crate::settings::Change {
        against: core.current.newest(),
        running: crate::settings::Running::Stays,
        items: Box::new([crate::settings::Item {
            setting: crate::settings::Setting::Charter(CHARTER),
            edit: crate::settings::Edit::Set(crate::settings::Value::Charter(Box::new(charter))),
        }]),
    };
    drop(watch_outputs(
        &mut core,
        &limits,
        Event::ChangeSettings {
            reply_to: ReplyTo::new(Token::new(92)),
            sign_in: 10,
            key: [4; 16],
            digest: [5; 32],
            change: Box::new(change),
        },
    ));
    assert!(marked(&core));
    drop(finish_watch_outputs(&mut core, &limits));
    drop(watch_outputs(
        &mut core,
        &limits,
        Event::Fleet(fleet::Event::Acknowledge { run: Token::new(90), attempt: Token::new(92) }),
    ));
    assert!(!marked(&core));
    core.touch_accounts(Some(2));
    assert!(!marked(&core));
    core.touch_deployment();
    assert!(!marked(&core));
}
