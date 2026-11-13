//! The offer and previews share run selection (domain/engine/views.md, 1; accounts.md, 4.3).
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros,
    reason = "unit fixtures are ordinary Rust under programming-model.md, 10.2"
)]
use crate::tests::runs::run_choice::{account, granted, host, inventory, login};
use crate::tests::*;
use jig_domain_link::{LoginState, SourceName};

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
    other.provider = OTHER_PROVIDER;
    other.name = b"other".as_slice().into();
    other.efforts = Box::new([b"low".as_slice().into(), b"high".as_slice().into()]);
    other.effort = Some(b"high".as_slice().into());
    configuration.settings.models = Box::new([scripted_model(), same.clone(), other.clone()]);
    configuration.defaults.charters[0].policy.model = other;
    configuration.defaults.charters[0].policy.alternatives = Box::new([same, scripted_model()]);

    configuration.settings.llm_providers.providers[0].offers.key = true;
    let mut provider = configuration.settings.llm_providers.providers[0].clone();
    provider.number = OTHER_PROVIDER;

    configuration.settings.llm_providers.providers =
        Box::new([configuration.settings.llm_providers.providers[0].clone(), provider]);
    let mut core = seeded_core(configuration, &limits).unwrap();
    core.people = base.people;
    core.accounts = accounts::Domain::new(&limits.accounts, core.settings.llm_providers.clone());
    host(&mut core, &limits, HOST, &[b"scripted", b"same", b"other"]);
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

fn listed_fixture() -> (Core, Limits) {
    use crate::settings::{CatalogueModel, Default, ModelSource, Setting, Value};
    let (mut core, limits) = fixture();
    let mut models = core.settings.models.to_vec();
    models[2].provider = PROVIDER;
    let mut person = scripted_model();
    person.name = b"personal".as_slice().into();
    models.push(person);
    for (at, source) in
        [ModelSource::Stock, ModelSource::Stock, ModelSource::List, ModelSource::Person].into_iter().enumerate()
    {
        let number = u32::try_from(at).unwrap().checked_add(1).unwrap();
        let row = CatalogueModel {
            number,
            model: models[at].clone(),
            shown: None,
            description: None,
            picker: at != 1,
            withdrawn: false,
            source,
            estimated: false,
            first_read: None,
        };
        core.current
            .defaults
            .insert(Setting::Model(number), Default { value: Value::Model(Box::new(row)), revision: 1 })
            .unwrap();
    }
    core.settings.models = models.into();
    host(&mut core, &limits, HOST, &[b"scripted", b"same", b"other", b"personal"]);
    (core, limits)
}

#[test]
fn account_lists_narrow_the_offer_and_readiness_by_model_source() {
    use fixtures::list_record;
    // The first two stock settings are bits 0 and 1; the list's setting is bit 2.
    const FIRST: u64 = 0b001;
    const SECOND_AND_LIST: u64 = 0b110;
    for (label, records, expected) in [
        ("no read", vec![(OWN, SOURCE, None)], vec![(0, vec![OWN]), (1, vec![OWN]), (3, vec![OWN])]),
        (
            "good read, including a picker-off stock model",
            vec![(OWN, SOURCE, Some(list_record(SECOND_AND_LIST, true, None)))],
            vec![(1, vec![OWN]), (2, vec![OWN]), (3, vec![OWN])],
        ),
        (
            "only failures",
            vec![(OWN, SOURCE, Some(list_record(0, false, Some(accounts::ReadFailure::Deadline))))],
            vec![(0, vec![OWN]), (1, vec![OWN]), (3, vec![OWN])],
        ),
        (
            "good read followed by failure",
            vec![(OWN, SOURCE, Some(list_record(SECOND_AND_LIST, true, Some(accounts::ReadFailure::Unreachable))))],
            vec![(1, vec![OWN]), (2, vec![OWN]), (3, vec![OWN])],
        ),
        (
            "two accounts give their union",
            vec![
                (OWN, SOURCE, Some(list_record(FIRST, true, None))),
                (SECOND, SECOND_SOURCE, Some(list_record(SECOND_AND_LIST, true, None))),
            ],
            vec![(0, vec![OWN]), (1, vec![SECOND]), (2, vec![SECOND]), (3, vec![OWN, SECOND])],
        ),
        (
            "both accounts rule stock and listed models out",
            vec![
                (OWN, SOURCE, Some(list_record(0, true, None))),
                (SECOND, SECOND_SOURCE, Some(list_record(0, true, None))),
            ],
            vec![(3, vec![OWN, SECOND])],
        ),
    ] {
        let (mut core, limits) = listed_fixture();
        fixtures::listed_accounts(&mut core, &limits, PROVIDER, &records);
        assert_eq!(offered(&core, &limits), expected, "{label}");
        let coverage = core.account_coverage(tasks::Payer::Party(PARTY), Some(PROJECT));
        for (at, model) in core.settings.models.iter().enumerate() {
            let ready = if expected.iter().any(|(index, _)| usize::from(*index) == at) {
                Readiness::Usable
            } else {
                Readiness::Unavailable
            };
            assert_eq!(
                core.model_readiness(tasks::Payer::Party(PARTY), coverage.as_slice(), model),
                (ready, false),
                "{label}: model {at} readiness"
            );
        }
        let mut held = scripted_model();
        held.name = b"held-without-catalogue-row".as_slice().into();
        assert!(core.account_allows(OWN, &held), "{label}: a held name without a row stays allowed");
    }
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
        later: None,
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

fn program_login(core: &mut Core, limits: &Limits, state: LoginState) {
    inventory(core, limits, HOST, Box::new([login(b"personal", state), login(b"pending", LoginState::NotSignedIn)]));
}

#[test]
fn offers_require_a_host_with_current_source_facts() {
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
            sources: Box::new([SourceName::new(b"personal").unwrap(), SourceName::new(b"pending").unwrap()]),
        }),
    ));
    assert_eq!(offered(&core, &limits).len(), 0);
    program_login(&mut core, &limits, LoginState::Usable);
    assert_eq!(offered(&core, &limits), [(0, vec![0])]);
    program_login(&mut core, &limits, LoginState::NotSignedIn);
    assert_eq!(offered(&core, &limits).len(), 0);
    program_login(&mut core, &limits, LoginState::Usable);
    assert_eq!(offered(&core, &limits), [(0, vec![0])]);
    drop(watch_outputs(&mut core, &limits, Event::Fleet(fleet::Event::Lost { channel: HOST })));
    assert_eq!(offered(&core, &limits).len(), 0);

    let (mut core, limits) = fixture();
    key(&mut core, &limits, OWN, SOURCE, PROVIDER);
    core.fleet = runs::run_choice::placement_fleet(&core, &limits);
    assert_eq!(offered(&core, &limits).len(), 0);
    host(&mut core, &limits, HOST, &[b"scripted", b"same", b"other"]);
    assert_eq!(offered(&core, &limits), [(0, vec![OWN]), (1, vec![OWN])]);
    drop(watch_outputs(
        &mut core,
        &limits,
        Event::Fleet(fleet::Event::Sources { sources: Box::new([SourceName::new(b"pending").unwrap()]) }),
    ));
    assert_eq!(offered(&core, &limits).len(), 0);
    inventory(&mut core, &limits, HOST, Box::new([login(b"pending", LoginState::NotSignedIn)]));
    assert_eq!(offered(&core, &limits), [(0, vec![OWN]), (1, vec![OWN])]);
}

fn two_accounts() -> (Core, Limits) {
    let (mut core, limits) = fixture();
    for record in [
        accounts::Stored::Account {
            list: None,
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
    for (event, moves_offer) in [
        (Event::Account(accounts::Event::Limited { account: SOURCE, reset: None }), true),
        (Event::Fleet(fleet::Event::Logins { channel: HOST, logins: Box::new([]) }), false),
        (Event::Fleet(fleet::Event::Lost { channel: HOST }), false),
    ] {
        let before = offered(&core, &limits);
        drop(watch_outputs(&mut core, &limits, event));
        assert!(marked(&core));
        assert_eq!(offered(&core, &limits) != before, moves_offer);
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
    for (name, has_account, reset, now, due, usable) in [
        ("no alarm", false, None, 0, false, false),
        ("future grant", true, None, 0, false, true),
        ("future limit", true, Some(10), 9, false, false),
        ("elapsed limit", true, Some(10), 10, true, true),
    ] {
        let (mut core, limits) = watching();
        if !has_account {
            core.accounts = accounts::Domain::new(&limits.accounts, core.settings.llm_providers.clone());
        }
        if let Some(reset) = reset {
            account(
                &mut core,
                &limits,
                accounts::Event::Limited { account: SOURCE, reset: Some(Time::from_nanos(reset)) },
            );
        }
        let before = frame(&core, &limits);
        let deadline = core.accounts.next_deadline();
        let env = Env { now: Time::from_nanos(now), wall: Wall::from_nanos(now), limits };
        assert_eq!(core.accounts.is_due(env.now), due, "{name}");
        let requests = fire(&mut core, &env, Timer::Account);
        assert_eq!(marked(&core), due, "{name}: only a due account timer marks me");
        if !due {
            assert!(requests.is_empty(), "{name}: an early firing emits nothing");
            assert_eq!(core.accounts.next_deadline(), deadline, "{name}: the alarm remains scheduled");
        }
        let after = frame(&core, &limits);
        assert_eq!(!offered(&core, &limits).is_empty(), usable, "{name}: current offer");
        assert_eq!(before != after, due, "{name}: only expiry moves the offer");
        let mut deliveries = finish_decision(&mut core, &env);
        if due {
            let Some(Request::Held(held)) = deliveries.pop() else { panic!("{name}: a due firing delivers me") };
            let Held::Delivery { watcher, document } = *held else { panic!("{name}: a Frame delivery") };
            assert_eq!(watcher, WATCHER, "{name}");
            assert_eq!(*document, Document::Frame(after), "{name}: the changed offer reaches the client");
        }
        assert!(deliveries.is_empty(), "{name}: no extra delivery");
    }
}
