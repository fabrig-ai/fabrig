//! Omitted choices follow current accounts; a successful claim keeps its provider
//! (jig/docs/design/domain/accounts.md, 4.3).
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    reason = "stories build fixtures and inspect child decisions"
)]
use super::run_choice::{account, granted, host_with_endpoints, inventory, login, placement_fleet};
use crate::tests::*;
use jig_domain_link::{Bounds, EndpointBound, Hello, Hosting, LoginState, Phase, SourceName};

const TASK: u64 = 1;
const HOST: Token = Token::new(7);
const OPENAI: u32 = 1;
const CLAUDE: u32 = 2;
const THIRD: u32 = 3;
const PROGRAM_ENDPOINT: u32 = 17;
const NAMES: [&[u8]; 4] = [b"openai-first", b"openai-second", b"anthropic", b"third"];
const FUNDER: tasks::Funder = tasks::Funder::Period { project: 1, period: 1 };
const EMPTY: RunWait = RunWait::Account { account: None, source: None, why: accounts::WaitWhy::Empty };

fn fixture(named: bool) -> (Core, Limits) {
    fixture_with_models(named, tasks::Models::ALL, true)
}
fn fixture_with_models(named: bool, allowed: tasks::Models, resolve: bool) -> (Core, Limits) {
    let (base, mut limits) = tree_core();
    limits.fleet.workers = 2;
    limits.accounts.accounts = 4;
    limits.accounts.holder_accounts = 4;
    limits.accounts.credentials = 3;
    limits.run_bytes = 4096;
    let mut configuration = config(41);
    let models: Vec<_> = NAMES
        .iter()
        .enumerate()
        .map(|(at, name)| {
            let mut model = scripted_model();
            model.name = (*name).into();
            model.provider = match at {
                0 | 1 => OPENAI,
                2 => CLAUDE,
                3 => THIRD,
                _ => unreachable!("fixture models"),
            };
            if at == 2 {
                model.program = Some(b"claude_code".as_slice().into());
                model.endpoint = PROGRAM_ENDPOINT;
            }
            model
        })
        .collect();
    configuration.settings.models = models.clone().into();
    configuration.defaults.charters[0].policy.model = models[0].clone();
    configuration.defaults.charters[0].policy.alternatives = models[1..].into();
    configuration.settings.llm_providers.providers =
        [(OPENAI, &NAMES[..2]), (CLAUDE, &NAMES[2..3]), (THIRD, &NAMES[3..])]
            .into_iter()
            .map(|(number, names)| accounts::Provider {
                list: None,
                number,
                name: names[0].into(),
                signed_in_plan: false,
                manage_usage: None,
                offers: accounts::Offers {
                    sign_in: false,
                    key: number != CLAUDE,
                    programs: if number == CLAUDE {
                        Box::new([b"claude_code".as_slice().into()])
                    } else {
                        Box::new([])
                    },
                },
            })
            .collect();
    let mut core = seeded_core(configuration, &limits).unwrap();
    core.people = base.people;
    send_task(&mut core, &limits, tasks::Event::Check);
    send_task(&mut core, &limits, tasks::Event::Restored);
    send_task(
        &mut core,
        &limits,
        tasks::Event::OpenPeriod { reply_to: ReplyTo::new(HOST), project: 1, period: 1, budget: 1000 },
    );
    let mut row = new_row(&core, TASK, FUNDER, None);
    row.authority.models = allowed;
    if named {
        row.llm = Some(named_choice());
    }
    let mut batch = [row];
    if resolve {
        core.resolve_llm(&mut batch).unwrap();
    }
    fixtures::make_batch(&mut core, &limits, tasks::Party::Person(1), batch.into());
    core.fleet = placement_fleet(&core, &limits);
    host_with_endpoints(&mut core, &limits, HOST, &NAMES, Box::new([]), Box::new([PROGRAM_ENDPOINT]));
    (core, limits)
}
fn named_choice() -> tasks::LlmChoice {
    tasks::LlmChoice {
        later: None,
        account: None,
        main: tasks::ModelEffort { model: 0, effort: Some(0) },
        subagents: Box::new([]),
    }
}
fn key(core: &mut Core, limits: &Limits, provider: u32) {
    account(core, limits, accounts::Event::Want);
    let mut out = Queue::uncapped();
    accounts::step(
        &mut core.accounts,
        &Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.accounts },
        accounts::Event::Change {
            request: HOST,
            holder: accounts::Holder::Deployment,
            change: accounts::Change::AddKey { provider, join: None, carried: u64::from(provider) },
        },
        &mut out,
    );
    let source = out
        .iter()
        .find_map(|request| {
            if let accounts::Request::Signing(accounts::Signing::Keep { source, .. }) = request {
                Some(*source)
            } else {
                None
            }
        })
        .expect("key accepted for keeping");
    granted(core, limits, source);
}
struct LoginAccount {
    number: u32,
    source: u32,
}
fn claude(core: &mut Core, limits: &Limits, state: LoginState) -> LoginAccount {
    account(core, limits, accounts::Event::Want);
    let mut out = Queue::uncapped();
    let source = accounts::programs::declare(
        &mut core.accounts,
        &Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.accounts },
        accounts::Holder::Party(1),
        CLAUDE,
        None,
        b"claude-login".as_slice().into(),
        &mut out,
    )
    .unwrap();
    let number = core.accounts.source(source).unwrap().account;
    let mut out = Queue::uncapped();
    fleet::step(
        &mut core.fleet,
        &Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.fleet },
        fleet::Event::Sources { sources: Box::new([SourceName::new(b"claude-login").unwrap()]) },
        &mut out,
    );
    inventory(core, limits, HOST, Box::new([login(b"claude-login", state)]));
    LoginAccount { number, source }
}
fn claim(core: &mut Core, limits: &Limits) {
    let choice = core.run_choice(TASK).unwrap();
    send_task(core, limits, tasks::Event::Prepare { reply_to: ReplyTo::new(HOST), task: TASK });
    send_task(
        core,
        limits,
        tasks::Event::Claim {
            session_end: None,
            offered: jig_domain_tasks::Reasons::ALL,
            reasons: jig_domain_tasks::Reasons::FIRST,
            reply_to: ReplyTo::new(HOST),
            task: TASK,
            attempt: 1,
            resumed: false,
            budget: 1,
            account: Some(choice.accounts[0].account),
            writes: Box::new([]),
        },
    );
}
fn document(core: &Core, limits: &Limits) -> Task {
    task_from_row(core, limits, 1, core.tasks.task(TASK).unwrap(), Wall::EPOCH).unwrap()
}

#[test]
fn only_a_claude_login_runs_without_recording_a_choice() {
    let (mut core, limits) = fixture(false);
    claude(&mut core, &limits, LoginState::Usable);
    let choice = core.run_choice(TASK).unwrap();
    assert_eq!(choice.policy.model.name.as_ref(), NAMES[2]);
    assert!(choice.policy.alternatives.is_empty(), "no other provider binds yet");
    assert_eq!(core.tasks.task(TASK).unwrap().llm, None);
    claim(&mut core, &limits);
    key(&mut core, &limits, OPENAI);
    key(&mut core, &limits, THIRD);
    // A claimed provider remains main; the other providers bind in charter order.
    assert_eq!(
        core.run_choice(TASK).unwrap().policy.alternatives.iter().map(|model| model.name.as_ref()).collect::<Vec<_>>(),
        [NAMES[0], NAMES[1], NAMES[3]]
    );
}

#[test]
fn only_an_openai_key_takes_the_first_charter_model() {
    let (mut core, limits) = fixture(false);
    key(&mut core, &limits, OPENAI);
    assert_eq!(core.run_choice(TASK).unwrap().policy.model.name.as_ref(), NAMES[0]);
    assert_eq!(core.tasks.task(TASK).unwrap().llm, None);
}

#[test]
#[expect(clippy::disallowed_macros, reason = "ordinary unit-test rows build account records")]
fn lists_narrow_only_unclaimed_implicit_choices_and_their_previews() {
    use fixtures::list_record;
    const OWN: u32 = 20;
    const SOURCE: u32 = 21;
    const SECOND: u32 = 30;
    const SECOND_SOURCE: u32 = 31;
    // The two OpenAI models are catalogue settings 1 and 2, bits 0 and 1.
    const FIRST: u64 = 0b01;
    const NEXT: u64 = 0b10;
    for (label, named, kept, records, limited, expected, preview) in [
        ("no list", false, false, vec![(OWN, SOURCE, None)], None, Ok((0, OWN)), Some(0)),
        (
            "first name missing",
            false,
            false,
            vec![(OWN, SOURCE, Some(list_record(NEXT, true, None)))],
            None,
            Ok((1, OWN)),
            Some(1),
        ),
        (
            "failed read retains names",
            false,
            false,
            vec![(OWN, SOURCE, Some(list_record(NEXT, true, Some(accounts::ReadFailure::Deadline))))],
            None,
            Ok((1, OWN)),
            Some(1),
        ),
        (
            "only failures use stock",
            false,
            false,
            vec![(OWN, SOURCE, Some(list_record(0, false, Some(accounts::ReadFailure::Unreachable))))],
            None,
            Ok((0, OWN)),
            Some(0),
        ),
        (
            "no allowed model",
            false,
            false,
            vec![(OWN, SOURCE, Some(list_record(0, true, None)))],
            None,
            Err(EMPTY),
            None,
        ),
        (
            "skip a disallowing account for the same model",
            false,
            false,
            vec![
                (OWN, SOURCE, Some(list_record(NEXT, true, None))),
                (SECOND, SECOND_SOURCE, Some(list_record(FIRST, true, None))),
            ],
            None,
            Ok((0, SECOND)),
            Some(0),
        ),
        (
            "only an unusable account allows the first model",
            false,
            false,
            vec![
                (OWN, SOURCE, Some(list_record(FIRST, true, None))),
                (SECOND, SECOND_SOURCE, Some(list_record(NEXT, true, None))),
            ],
            Some(SOURCE),
            Ok((1, SECOND)),
            Some(1),
        ),
        (
            "explicit choice survives a lost name",
            true,
            false,
            vec![(OWN, SOURCE, Some(list_record(NEXT, true, None)))],
            None,
            Ok((0, OWN)),
            Some(1),
        ),
        (
            "kept account survives a lost name",
            false,
            true,
            vec![(OWN, SOURCE, Some(list_record(NEXT, true, None)))],
            None,
            Ok((0, OWN)),
            Some(1),
        ),
    ] {
        let (mut core, limits) = fixture(named);
        if kept {
            fixtures::listed_accounts(&mut core, &limits, OPENAI, &[(OWN, SOURCE, None)]);
            claim(&mut core, &limits);
        }
        fixtures::listed_accounts(&mut core, &limits, OPENAI, &records);
        if let Some(source) = limited {
            account(&mut core, &limits, accounts::Event::Limited { account: source, reset: None });
        }
        assert_eq!(
            core.run_choice(TASK).map(|choice| {
                (
                    NAMES.iter().position(|name| *name == choice.policy.model.name.as_ref()).unwrap(),
                    choice.accounts[0].account,
                )
            }),
            expected,
            "{label}"
        );
        assert_eq!(
            core.preview_llm(1, core.current.newest(), tasks::Models::ALL, tasks::Payer::Project(1), 1)
                .map(|choice| choice.main.model),
            preview,
            "{label}: a template without a choice previews current availability"
        );
        if !named && !kept {
            assert_eq!(
                document(&core, &limits).llm.map(|choice| choice.main.model),
                preview,
                "{label}: task and preview agree"
            );
        }
    }
}

#[test]
fn a_claimed_login_stays_main_when_an_earlier_provider_arrives() {
    let (mut core, limits) = fixture(false);
    let login = claude(&mut core, &limits, LoginState::Usable);
    claim(&mut core, &limits);
    key(&mut core, &limits, OPENAI);
    assert_eq!(core.run_choice(TASK).unwrap().policy.model.name.as_ref(), NAMES[2]);
    assert_eq!(core.tasks.task(TASK).unwrap().kept_account, Some(login.number));
}

#[test]
fn an_empty_wait_uses_a_login_added_later() {
    let (mut core, limits) = fixture(false);
    assert_eq!(core.run_choice(TASK), Err(EMPTY));
    claude(&mut core, &limits, LoginState::Usable);
    assert_eq!(core.run_choice(TASK).unwrap().policy.model.name.as_ref(), NAMES[2]);
}

#[test]
fn a_named_task_waits_for_its_provider() {
    let (mut core, limits) = fixture(true);
    claude(&mut core, &limits, LoginState::Usable);
    assert_eq!(core.run_choice(TASK), Err(EMPTY));
    key(&mut core, &limits, THIRD);
    assert_eq!(core.run_choice(TASK), Err(EMPTY));
    assert_eq!(core.tasks.task(TASK).unwrap().llm, Some(named_choice()));
    assert_eq!(document(&core, &limits).llm, Some(named_choice()));
    assert_eq!(document(&core, &limits).llm, core.next_llm(core.tasks.task(TASK).unwrap()));
}

#[test]
fn a_removed_or_ineligible_kept_account_chooses_again() {
    for remove in [true, false] {
        let (mut core, limits) = fixture(false);
        let login = claude(&mut core, &limits, LoginState::Usable);
        claim(&mut core, &limits);
        key(&mut core, &limits, OPENAI);
        let change = if remove {
            accounts::Change::Remove { account: login.number }
        } else {
            accounts::Change::SetUses { account: login.number, uses: Some(Box::new([])) }
        };
        account(
            &mut core,
            &limits,
            accounts::Event::Change { request: HOST, holder: accounts::Holder::Party(1), change },
        );
        assert_eq!(core.run_choice(TASK).unwrap().policy.model.name.as_ref(), NAMES[0]);
    }
}

#[test]
fn a_limited_kept_account_waits_for_its_own_provider() {
    let (mut core, limits) = fixture(false);
    let login = claude(&mut core, &limits, LoginState::Usable);
    claim(&mut core, &limits);
    key(&mut core, &limits, OPENAI);
    account(&mut core, &limits, accounts::Event::Limited { account: login.source, reset: None });
    assert_eq!(
        core.run_choice(TASK),
        Err(RunWait::Account {
            account: Some(login.number),
            source: Some(login.source),
            why: accounts::WaitWhy::Source(accounts::State::Limited { until: None }),
        })
    );
}

#[test]
fn an_unsigned_login_wait_names_its_account_and_source() {
    let (mut core, limits) = fixture(false);
    let login = claude(&mut core, &limits, LoginState::NotSignedIn);
    assert_eq!(
        core.run_choice(TASK),
        Err(RunWait::Account { account: Some(login.number), source: Some(login.source), why: accounts::WaitWhy::Host })
    );
    assert_eq!(
        document(&core, &limits).run_wait,
        Some(TaskRunWait::Account { account: Some(login.number), source: Some(login.source), why: TaskWaitWhy::Host })
    );
}

#[test]
fn no_connected_host_waits_for_a_host() {
    let (mut core, limits) = fixture(false);
    claude(&mut core, &limits, LoginState::Usable);
    core.fleet = placement_fleet(&core, &limits);
    assert_eq!(core.run_choice(TASK), Err(RunWait::Host));
}

#[test]
fn a_busy_login_host_is_a_host_wait() {
    const BUSY: Token = Token::new(9);
    const SPARE: Token = Token::new(11);
    let (mut core, limits) = fixture(false);
    claude(&mut core, &limits, LoginState::Usable);
    core.fleet = placement_fleet(&core, &limits);
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.fleet };
    let mut out = Queue::uncapped();
    fleet::step(
        &mut core.fleet,
        &env,
        fleet::Event::Sources { sources: Box::new([SourceName::new(b"claude-login").unwrap()]) },
        &mut out,
    );
    fleet::step(
        &mut core.fleet,
        &env,
        fleet::Event::Hello {
            worker: 0,
            channel: HOST,
            hello: Hello {
                jobs: Box::default(),
                bounds: Bounds {
                    program_endpoints: Box::new([PROGRAM_ENDPOINT]),
                    endpoints: NAMES
                        .iter()
                        .filter_map(|name| core.settings.models.iter().find(|model| model.name.as_ref() == *name))
                        .map(|model| EndpointBound { endpoint: model.endpoint, window: 8192, output: 1024 })
                        .collect(),
                    ..fixtures::bounds()
                },
                stop_bound: Duration::ZERO,
                slots: 1,
                workstreams: Box::new([]),
                hosting: Box::new([Hosting { run: BUSY, attempt: BUSY, phase: Phase::Active }]),
            },
        },
        &mut out,
    );
    fleet::step(
        &mut core.fleet,
        &env,
        fleet::Event::Adopt {
            reply_to: ReplyTo::new(HOST),
            run: BUSY,
            attempt: BUSY,
            kept: 0,
            kind: fleet::HostKind::Worker,
            worked: false,
            holds: Box::new([]),
            grants: Box::new([]),
        },
        &mut out,
    );
    inventory(&mut core, &limits, HOST, Box::new([login(b"claude-login", LoginState::Usable)]));
    // A free host lacks the login, but the host holding it will have room again.
    host_with_endpoints(
        &mut core,
        &limits,
        SPARE,
        &NAMES,
        Box::new([login(b"claude-login", LoginState::NotSignedIn)]),
        Box::new([PROGRAM_ENDPOINT]),
    );
    assert_eq!(core.run_choice(TASK), Err(RunWait::Host));
}

#[test]
fn an_unnamed_document_shows_the_current_run_choice() {
    let (mut core, limits) = fixture(false);
    claude(&mut core, &limits, LoginState::Usable);
    assert_eq!(document(&core, &limits).llm, core.next_llm(core.tasks.task(TASK).unwrap()));
    let llm = document(&core, &limits).llm.unwrap();
    assert_eq!(llm.main, tasks::ModelEffort { model: 2, effort: Some(0) });
    assert_eq!(llm.subagents.iter().map(|model| model.model).collect::<Vec<_>>(), [0, 1, 3]);
}

#[test]
fn create_and_amend_refuse_an_authority_with_no_charter_model() {
    let (mut core, limits) = fixture_with_models(false, tasks::Models::ALL, false);
    let mut row = new_row(&core, 2, FUNDER, None);
    row.authority.models = tasks::Models::NONE;
    assert_eq!(
        core.resolve_llm(&mut [row]),
        Err(tasks::Problem::new(Some(2), tasks::Refusal::Llm { model: 0, why: tasks::LlmFault::Authority }))
    );
    let mut authority = core.tasks.task(TASK).unwrap().authority.clone();
    authority.models = tasks::Models::NONE;
    let mut out = Queue::uncapped();
    tasks::step(
        &mut core.tasks,
        &Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.tasks },
        tasks::Event::Amend {
            reply_to: ReplyTo::new(HOST),
            by: tasks::Party::Person(1),
            places: Box::new([]),
            funder: None,
            operation: tasks::Amend {
                task: TASK,
                message: 1,
                stop_run: false,
                amendment: Box::new(tasks::Amendment {
                    authority: Some(authority),
                    ..fixtures::unchanged(b"narrow models")
                }),
            },
        },
        &mut out,
    );
    let refusal = out.iter().find_map(|request| {
        if let tasks::Request::Refused { problem, .. } = request { Some(problem.why) } else { None }
    });
    assert_eq!(refusal, Some(tasks::Refusal::Llm { model: 0, why: tasks::LlmFault::Authority }));
}

#[test]
fn a_move_requires_an_authorized_model_and_counts_incompatibility() {
    // This unnamed task can use Claude only; a prospective charter has only OpenAI.
    let (mut core, limits) = fixture_with_models(false, tasks::Models::Named(1 << 2), false);
    let charters: Box<[tasks::CharterModels]> =
        Box::new([tasks::CharterModels { resumes: false, number: 1, revision: 2, models: Some(Box::new([0, 1])) }]);
    assert_eq!(core.tasks.incompatible_charters(&charters, 1), (Box::<[u64]>::from([TASK]), 0));
    send_task(&mut core, &limits, tasks::Event::Charters { newest: 2, charters, efforts: Box::new([2, 2, 2, 2]) });
    assert!(!core.tasks.may_move(TASK));
    send_task(&mut core, &limits, tasks::Event::MoveToNewest { from: None });
    assert_eq!(core.tasks.task(TASK).unwrap().settings.task, 1);
}
