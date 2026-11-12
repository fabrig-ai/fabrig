//! A person's trial through the root, without a project or a real host
//! (temper/docs/design/domain/trial.md, 2 to 6).
use jig_domain_accounts as accounts;
use jig_domain_core as core;
use jig_domain_host as host;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_lib::{Duration, ReplyTo, Token};
use temper_engine_domain::{self as engine, Delivery};
use temper_engine_domain_trials as trials;
use temper_engine_domain_world::{direct::Driver, protocol, store::Store, walking};

fn config() -> engine::Config {
    let mut config = walking::config(106);
    config.core.seed_projects = Box::new([]);
    config.core.workers_in_process = true;
    config.core.owners[0].project = people::DEPLOYMENT;
    let mut trial = walking::charter();
    trial.number = 10;
    trial.policy.inspect = false;
    trial.policy.modify = false;
    trial.policy.shell = false;
    trial.policy.agents = false;
    trial.policy.tools = tasks::Tools(0);
    let mut second = trial.policy.model.clone();
    second.provider = 2;
    second.name = b"second".as_slice().into();
    trial.policy.alternatives = Box::new([second.clone()]);
    config.core.settings.models = Box::new([trial.policy.model.clone(), second]);
    let mut providers = config.core.settings.llm_providers.providers.to_vec();
    let mut provider = providers[0].clone();
    provider.number = 2;
    provider.name = b"Provider 2".as_slice().into();
    providers.push(provider);
    config.core.settings.llm_providers.providers = providers.into();
    config.core.defaults.charters = Box::new([walking::charter(), trial]);
    config
}

fn driver() -> Driver {
    configured(config(), 1)
}

fn configured(config: engine::Config, provider: u32) -> Driver {
    let mut limits = walking::limits();
    limits.tasks.charters = 2;
    limits.people.sign_in_lifetime = Duration::from_secs(180);
    limits.people.requests = 8;
    limits.accounts.accounts = 4;
    limits.accounts.holder_accounts = 3;
    limits.authority.accounts = 4;
    limits.accounts.credentials = 4;
    limits.fleet.grants = 8;
    let mut driver = Driver::cold(Store::new(), config, &limits);
    driver.broken("trial restart cancels live tasks and drops rows; these stories check that projection explicitly");
    driver.period_sent = true;
    driver.settle();
    driver.sign_in();
    driver.settle();
    driver.send(engine::Event::Party(engine::PartyInput::Key {
        reply_to: ReplyTo::new(Token::new(95)),
        sign_in: driver.session(),
        key: [95; 16],
        ask: Box::new(people::Ask::AccountChange {
            holder: people::AccountHolder::Own,
            change: people::AccountChange::AddKey { provider, join: None, digest: [8; 32] },
        }),
        carried: core::Carried { number: 1_u64 << 63, digest: [8; 32] },
    }));
    driver.settle();
    driver.hello(Token::new(7));
    driver.settle();
    driver
}

fn account(driver: &Driver) -> u32 {
    driver
        .store
        .rows
        .values()
        .find_map(|row| {
            if let engine::Record::Core(core::Record::Accounts(accounts::Stored::Account {
                number,
                holder: accounts::Holder::Party(_),
                ..
            })) = row
            {
                Some(*number)
            } else {
                None
            }
        })
        .expect("the person's fake key account")
}

fn ask(driver: &mut Driver, account: u32, again: bool, key: u8) {
    driver.send(engine::Event::Party(engine::PartyInput::Application {
        scope: people::ApplicationScope::Own,
        reply_to: ReplyTo::new(Token::new(u64::from(key))),
        sign_in: driver.session(),
        key: [key; 16],
        digest: [key; 32],
        carried: None,
        request: engine::Application::Try { account, again },
    }));
    driver.pump();
    acknowledge(driver);
}

#[test]
fn a_try_runs_one_task_of_charter_ten_naming_no_llm() {
    let mut driver = driver();
    let account = account(&driver);
    let carried = core::Carried { number: (1_u64 << 63) | 0x60, digest: [96; 32] };
    driver.send(engine::Event::Party(engine::PartyInput::Application {
        scope: people::ApplicationScope::Own,
        reply_to: ReplyTo::new(Token::new(96)),
        sign_in: driver.session(),
        key: [96; 16],
        digest: [96; 32],
        carried: Some(carried),
        request: engine::Application::Try { account, again: false },
    }));
    driver.settle();
    assert!(driver.delivered.iter().any(|delivery| matches!(delivery,
        Delivery::CarriedUnused { number } if *number == carried.number)));
    assert_eq!(claims(&driver), 1);
    assert_eq!(tasks_made(&driver), 1);
    assert!(
        driver.delivered.iter().any(|delivery| matches!(
            delivery,
            Delivery::Application { outcome: Some(engine::ApplicationAnswer::Try(engine::TryAnswer::Tried)), .. }
        )),
        "an admitted try is acknowledged"
    );
    let assignment = driver.last_assignment();
    assert_eq!(assignment.charter, 10);
    assert_eq!(assignment.run.policy.model.name.as_ref(), b"scripted");
    let task = driver.task(assignment.task).expect("one live trial task");
    assert_eq!(task.project, 0);
    assert_eq!(task.executor, tasks::Executor::Agent { charter: 10 });
    assert!(task.llm.is_none());
    assert_eq!(task.kept_account, Some(account));
    assert_eq!(task.spec.words.as_ref(), b"Say one word.");
    assert_eq!(task.contract, tasks::Contract::Report { words: 64 });
    assert!(
        task.spec.job.is_none() && !task.spec.asks && task.spec.parameters.is_empty() && task.spec.inputs.is_empty()
    );
    assert!(
        !driver
            .store
            .rows
            .values()
            .any(|row| matches!(row, engine::Record::Core(core::Record::People(people::Stored::Policy { .. }))))
    );
}

const CHANNEL: Token = Token::new(7);
const WATCHER: Token = Token::new(401);

fn watch(driver: &mut Driver, watcher: Token, sign_in: u64) {
    driver.send(engine::Event::Party(engine::PartyInput::Watch {
        watcher,
        sign_in,
        subject: core::documents::Watched::Application {
            scope: people::ApplicationScope::Own,
            connector: 0,
            subject: Token::new(engine::TRIALS_SUBJECT),
        },
        window: core::documents::Window { rows: 64, after: None },
    }));
    driver.settle();
}

fn rows(driver: &Driver) -> &[trials::Row] {
    driver
        .delivered
        .iter()
        .rev()
        .find_map(
            |delivery| {
                if let Delivery::Trials { watcher: WATCHER, rows } = delivery { Some(rows.as_ref()) } else { None }
            },
        )
        .expect("the family's own trials document")
}

fn row(driver: &Driver) -> &trials::Row {
    let rows = rows(driver);
    assert_eq!(rows.len(), 1);
    &rows[0]
}

fn watched() -> (Driver, engine::Assignment) {
    let mut driver = driver();
    let session = driver.session();
    watch(&mut driver, WATCHER, session);
    assert_eq!(rows(&driver), []);
    let account = account(&driver);
    driver.send(engine::Event::Party(engine::PartyInput::Delivered { watcher: WATCHER }));
    ask(&mut driver, account, false, 96);
    assert_eq!(row(&driver).state, trials::State::Running);
    let assignment = driver.last_assignment();
    (driver, assignment)
}

fn acknowledge(driver: &mut Driver) {
    for _ in 0..4 {
        let before = driver.delivered.len();
        driver.send(engine::Event::Party(engine::PartyInput::Delivered { watcher: WATCHER }));
        driver.pump();
        if driver.delivered.len() == before {
            return;
        }
    }
    panic!("the trials watch drained its coalesced updates");
}

fn claims(driver: &Driver) -> usize {
    driver.delivered.iter().filter(|delivery| matches!(delivery, Delivery::Assigned { .. })).count()
}

fn tasks_made(driver: &Driver) -> usize {
    driver
        .store
        .rows
        .values()
        .filter(|record| {
            matches!(
                record,
                engine::Record::Core(core::Record::Tasks(tasks::Stored::Live(_) | tasks::Stored::Ended(_)))
            )
        })
        .count()
}

fn cancelled(driver: &Driver, task: u64) -> bool {
    driver.delivered.iter().any(|delivery| {
        matches!(delivery,
        Delivery::Link { down: host::LinkDown::Cancel { run, .. }, .. } if run.raw() == task)
    })
}

fn end_cancelled(driver: &mut Driver, run: &engine::Assignment) {
    driver.answer(CHANNEL, run.task, run.attempt, 1, tasks::End::Refused, engine::Parts::none());
    driver.settle();
    assert!(driver.task(run.task).is_none(), "cancellation ends the task once its host answers");
}

fn active(driver: &mut Driver, run: &engine::Assignment) {
    driver.send(protocol::up(
        CHANNEL,
        host::LinkUp::Activity {
            run: Token::new(run.task),
            attempt: Token::new(run.attempt),
            activity: host::AgentActivity::Phase(jig_domain_link::Phase::Active),
        },
        engine::Parts::none(),
    ));
    driver.settle();
}

fn change(driver: &mut Driver, change: people::AccountChange, key: u8) {
    driver.send(engine::Event::Party(engine::PartyInput::Ask {
        reply_to: ReplyTo::new(Token::new(u64::from(key))),
        sign_in: driver.session(),
        key: [key; 16],
        models: Box::new([]),
        ask: people::Ask::AccountChange { holder: people::AccountHolder::Own, change },
    }));
    driver.pump();
}

#[test]
fn its_first_turn_answers_and_ends_the_task() {
    let (mut driver, run) = watched();
    acknowledge(&mut driver);
    driver.fire_at(Duration::from_secs(3));
    driver.send(protocol::turn(CHANNEL, run.task, run.attempt, 1, 1, None, b"Word"));
    driver.settle();
    assert_eq!(row(&driver).state, trials::State::Answered { seconds: 3 });
    assert!(cancelled(&driver, run.task));
    end_cancelled(&mut driver, &run);
    driver.send(protocol::turn(CHANNEL, run.task, run.attempt, 2, 2, None, b"Too late"));
    driver.fire_at(Duration::from_secs(40));
    driver.settle();
    assert_eq!(claims(&driver), 1);
    let turns = driver
        .store
        .rows
        .values()
        .filter(|record| {
            matches!(record,
        engine::Record::Core(core::Record::Core(core::CoreRecord::Turn(turn))) if turn.task == run.task)
        })
        .count();
    assert_eq!(turns, 1, "no second turn is committed");
}

#[test]
fn a_failed_run_fails_it_and_no_retry_starts() {
    for (end, class) in [
        (tasks::End::Failed(tasks::Class::Transient), trials::Class::Transient),
        (tasks::End::Failed(tasks::Class::Permanent), trials::Class::Permanent),
        (tasks::End::Failed(tasks::Class::Run), trials::Class::Run),
        (tasks::End::Failed(tasks::Class::Agent), trials::Class::Agent),
        (tasks::End::Failed(tasks::Class::Lost), trials::Class::Lost),
        (tasks::End::Failed(tasks::Class::Invalid), trials::Class::Invalid),
        (tasks::End::Refused, trials::Class::Refused),
    ] {
        let (mut driver, run) = watched();
        acknowledge(&mut driver);
        driver.answer(CHANNEL, run.task, run.attempt, 1, end, engine::Parts::none());
        driver.settle();
        assert_eq!(row(&driver).state, trials::State::Failed { why: trials::Why::Ended { class } });
        assert!(driver.task(run.task).is_none());
        driver.fire_at(Duration::from_secs(31));
        driver.settle();
        assert_eq!(claims(&driver), 1, "the first {class:?} ending is final");
    }
}

#[test]
fn a_minute_fails_it() {
    for hanging in [false, true] {
        let mut driver = driver();
        let account = account(&driver);
        if !hanging {
            change(&mut driver, people::AccountChange::SetUses { account, uses: Some(Box::new([])) }, 97);
        }
        let session = driver.session();
        watch(&mut driver, WATCHER, session);
        acknowledge(&mut driver);
        ask(&mut driver, account, false, 96);
        let task = row(&driver).task.expect("trial made even without a usable account");
        let run = hanging.then(|| driver.last_assignment());
        if let Some(run) = &run {
            acknowledge(&mut driver);
            active(&mut driver, run);
            assert_eq!(row(&driver).step, trials::Step::Answers);
        } else {
            assert_eq!(claims(&driver), 0);
        }
        acknowledge(&mut driver);
        driver.fire_at(Duration::from_secs(59));
        driver.pump();
        acknowledge(&mut driver);
        assert_eq!(row(&driver).state, trials::State::Running);
        assert_eq!(
            driver.root.next_deadline(&driver.env),
            Some(skein_lib::Time::from_nanos(Duration::from_secs(60).as_nanos()))
        );
        driver.fire_at(Duration::from_secs(60));
        driver.pump();
        acknowledge(&mut driver);
        assert_eq!(row(&driver).state, trials::State::Failed { why: trials::Why::Waited });
        if let Some(run) = &run {
            assert!(cancelled(&driver, task));
            end_cancelled(&mut driver, run);
        }
        assert!(driver.task(task).is_none());
        assert_eq!(claims(&driver), usize::from(hanging));
    }
}

#[test]
fn removing_the_account_cancels_it() {
    for later in [false, true] {
        let (mut driver, mut run) = watched();
        let first = account(&driver);
        let mut removed = first;
        if later {
            end_by_turn(&mut driver, &run);
            add_account(&mut driver, 1, 98);
            let second = own_accounts(&driver).into_iter().find(|account| *account != first).unwrap();
            acknowledge(&mut driver);
            removed = second;
            ask(&mut driver, second, false, 99);
            assert_eq!(rows(&driver).len(), 2);
            run = driver.last_assignment();
            acknowledge(&mut driver);
            assert_eq!(rows(&driver).iter().find(|row| row.task == Some(run.task)).unwrap().took, Some(second));
        }
        acknowledge(&mut driver);
        change(&mut driver, people::AccountChange::Remove { account: removed }, 100);
        acknowledge(&mut driver);
        if later {
            assert_eq!(rows(&driver).len(), 1, "removal leaves the first account's ended row");
            assert_eq!(row(&driver).account, first);
            assert_eq!(row(&driver).state, trials::State::Answered { seconds: 1 });
        } else {
            assert!(rows(&driver).is_empty(), "the removed account's row goes");
        }
        assert!(cancelled(&driver, run.task));
        end_cancelled(&mut driver, &run);
    }
}

fn add_account(driver: &mut Driver, provider: u32, key: u8) {
    driver.send(engine::Event::Party(engine::PartyInput::Key {
        reply_to: ReplyTo::new(Token::new(u64::from(key))),
        sign_in: driver.session(),
        key: [key; 16],
        ask: Box::new(people::Ask::AccountChange {
            holder: people::AccountHolder::Own,
            change: people::AccountChange::AddKey { provider, join: None, digest: [key; 32] },
        }),
        carried: core::Carried { number: (1_u64 << 63) + u64::from(key), digest: [key; 32] },
    }));
    driver.settle();
}

fn own_accounts(driver: &Driver) -> Vec<u32> {
    driver
        .store
        .rows
        .values()
        .filter_map(|record| {
            if let engine::Record::Core(core::Record::Accounts(accounts::Stored::Account {
                number,
                holder: accounts::Holder::Party(_),
                ..
            })) = record
            {
                Some(*number)
            } else {
                None
            }
        })
        .collect()
}

fn restart(driver: &mut Driver) {
    driver.reopen(config());
    driver.hello(CHANNEL);
    driver.settle();
    driver.sign_in();
    driver.settle();
    let session = driver.session();
    watch(driver, WATCHER, session);
}

#[test]
fn a_restart_cancels_it_and_claims_no_run() {
    let (mut driver, run) = watched();
    // In-process workers die with the engine; startup settles their attempt before Open.
    assert!(driver.task(run.task).is_some());
    restart(&mut driver);
    assert_eq!(rows(&driver), []);
    assert_eq!(claims(&driver), 0, "cancel sits after Open and before any due task can claim");
    assert!(driver.task(run.task).is_none());
}

#[test]
fn a_restart_starts_nothing() {
    let (mut driver, run) = watched();
    driver.answer(
        CHANNEL,
        run.task,
        run.attempt,
        1,
        tasks::End::Finished {
            result: tasks::TaskResult::Report { words: b"Word".as_slice().into() },
            cancel_delegates: false,
        },
        engine::Parts::none(),
    );
    driver.settle();
    assert!(driver.task(run.task).is_none());
    acknowledge(&mut driver);
    assert_eq!(row(&driver).state, trials::State::Answered { seconds: 1 });
    let before = tasks_made(&driver);
    restart(&mut driver);
    assert_eq!(rows(&driver), []);
    assert_eq!(claims(&driver), 0);
    assert_eq!(tasks_made(&driver), before);
    let account = account(&driver);
    ask(&mut driver, account, false, 96);
    assert!(
        driver
            .delivered
            .iter()
            .rev()
            .find_map(|delivery| {
                if let Delivery::Application { outcome, .. } = delivery { Some(outcome) } else { None }
            })
            .unwrap()
            .is_none(),
        "the cold root has no volatile outcome for the retained key"
    );
    assert_eq!(claims(&driver), 0);
    assert_eq!(tasks_made(&driver), before);
}

#[test]
fn a_second_try_spends_nothing() {
    let (mut driver, run) = watched();
    let account = account(&driver);
    for again in [false, true] {
        ask(&mut driver, account, again, if again { 98 } else { 97 });
        assert_eq!(claims(&driver), 1);
        assert_eq!(tasks_made(&driver), 1);
    }
    end_by_turn(&mut driver, &run);
    ask(&mut driver, account, false, 99);
    assert_eq!(claims(&driver), 1);
    assert_eq!(tasks_made(&driver), 1);
}

fn end_by_turn(driver: &mut Driver, run: &engine::Assignment) {
    acknowledge(driver);
    driver.send(protocol::turn(CHANNEL, run.task, run.attempt, 1, 1, None, b"Word"));
    driver.settle();
    end_cancelled(driver, run);
}

#[test]
fn again_runs_another() {
    let (mut driver, run) = watched();
    end_by_turn(&mut driver, &run);
    acknowledge(&mut driver);
    let account = account(&driver);
    ask(&mut driver, account, true, 97);
    assert_eq!(row(&driver).state, trials::State::Running);
    assert_ne!(row(&driver).task, Some(run.task));
    assert_eq!(claims(&driver), 2);
    assert_eq!(tasks_made(&driver), 2);
}

fn answer(driver: &Driver) -> engine::TryAnswer {
    driver
        .delivered
        .iter()
        .rev()
        .find_map(|delivery| {
            if let Delivery::Application { outcome: Some(engine::ApplicationAnswer::Try(answer)), .. } = delivery {
                Some(*answer)
            } else {
                None
            }
        })
        .expect("retained typed try answer")
}

#[test]
fn a_replayed_key_has_the_same_answer_and_spends_nothing() {
    for theirs in [false, true] {
        let (mut driver, run) = watched();
        let account = if theirs { account(&driver) } else { u32::MAX };
        ask(&mut driver, account, true, 97);
        let expected = if theirs { engine::TryAnswer::Tried } else { engine::TryAnswer::NotTheirs };
        assert_eq!(answer(&driver), expected);
        end_by_turn(&mut driver, &run);
        let before = tasks_made(&driver);
        driver.fire_at(Duration::from_secs(2));
        driver.settle();
        ask(&mut driver, account, true, 97);
        assert_eq!(answer(&driver), expected);
        assert_eq!(tasks_made(&driver), before);
        assert_eq!(claims(&driver), 1);
    }
}

#[test]
fn another_persons_account_is_not_theirs() {
    let mut driver = driver();
    let account = account(&driver);
    sign_in_second(&mut driver);
    ask(&mut driver, account, false, 96);
    assert_eq!(answer(&driver), engine::TryAnswer::NotTheirs);
    assert_eq!(tasks_made(&driver), 0);
    let session = driver.session();
    watch(&mut driver, WATCHER, session);
    assert_eq!(rows(&driver), []);
}

fn sign_in_second(driver: &mut Driver) {
    driver.send(engine::Event::Party(engine::PartyInput::SignedIn {
        held: None,
        reply_to: ReplyTo::new(Token::new(501)),
        identity: people::Identity {
            key: temper_engine_domain_forge::user_key(1, 8_u64),
            login: b"second".as_slice().into(),
            name: b"Second".as_slice().into(),
        },
    }));
    driver.settle();
    // Driver::session reads its first sign-in; keep the new person's session at the front.
    let index = driver
        .delivered
        .iter()
        .rposition(|delivery| matches!(delivery, Delivery::WebReply { reply: people::Reply::SignedIn { .. }, .. }))
        .expect("the second person signed in");
    let delivery = driver.delivered.remove(index);
    driver.delivered.insert(0, delivery);
}

#[test]
fn past_starting_moves_its_step() {
    let (mut driver, run) = watched();
    assert_eq!(row(&driver).step, trials::Step::Starts);
    acknowledge(&mut driver);
    active(&mut driver, &run);
    assert_eq!(row(&driver).step, trials::Step::Answers);
    let deliveries = driver.delivered.iter().filter(|delivery| matches!(delivery, Delivery::Trials { .. })).count();
    acknowledge(&mut driver);
    active(&mut driver, &run);
    assert_eq!(
        driver.delivered.iter().filter(|delivery| matches!(delivery, Delivery::Trials { .. })).count(),
        deliveries,
        "unchanged observations do not mark the document"
    );
}

#[test]
fn a_later_account_is_the_one_tried() {
    // The first account's provider, the later one's, and the model the later trial runs.
    for (first_provider, later_provider, model) in [
        (1, 2, b"second".as_slice()),   // Claude, then ChatGPT.
        (2, 1, b"scripted".as_slice()), // ChatGPT, then Claude.
        (1, 1, b"scripted".as_slice()), // A second account of one provider.
    ] {
        let mut driver = configured(config(), first_provider);
        let first = account(&driver);
        let session = driver.session();
        watch(&mut driver, WATCHER, session);
        acknowledge(&mut driver);
        ask(&mut driver, first, false, 96);
        let run = driver.last_assignment();
        assert!(driver.task(run.task).unwrap().llm.is_none());
        assert_eq!(row(&driver).took, Some(first));
        end_by_turn(&mut driver, &run);
        add_account(&mut driver, later_provider, 97);
        let later = own_accounts(&driver).into_iter().find(|account| *account != first).unwrap();
        acknowledge(&mut driver);
        ask(&mut driver, later, false, 98);
        let run = held_trial(&driver, later, model);
        end_by_turn(&mut driver, &run);
        acknowledge(&mut driver);
        ask(&mut driver, later, true, 99);
        let run = held_trial(&driver, later, model);
        assert_eq!(claims(&driver), 3);
        if first_provider == 1 && later_provider == 2 {
            end_by_turn(&mut driver, &run);
            acknowledge(&mut driver);
            ask(&mut driver, first, true, 100);
            held_trial(&driver, first, b"scripted");
            assert_eq!(claims(&driver), 4, "retry judges the first account again");
        }
    }
}

fn held_trial(driver: &Driver, account: u32, model: &[u8]) -> engine::Assignment {
    let run = driver.last_assignment();
    let row = rows(driver).iter().find(|row| row.account == account).expect("the tried account has a row");
    assert_eq!(row.took, Some(account));
    assert_eq!(row.task, Some(run.task));
    assert_eq!(run.run.policy.model.name.as_ref(), model);
    let task = driver.task(run.task).expect("the trial task is live");
    assert_eq!(task.llm.as_ref().expect("creator holds the account").account, Some(account));
    assert_eq!(task.kept_account, Some(account));
    run
}

#[test]
fn the_calendar_takes_no_period_of_project_zero() {
    let (mut driver, run) = watched();
    driver.send(engine::Event::Party(engine::PartyInput::Period { project: 0, period: 0, budget: 1_000, next: None }));
    driver.settle();
    let key = engine::Key::Core(core::Key::Tasks(tasks::Key::Ledger(tasks::Funder::Period { project: 0, period: 0 })));
    let engine::Record::Core(core::Record::Tasks(tasks::Stored::Ledger(period))) = driver.store.rows.get(&key).unwrap()
    else {
        panic!("person's period zero")
    };
    assert!(period.period.unwrap().next.is_none());
    end_by_turn(&mut driver, &run);
    driver.fire_at(Duration::from_secs(180));
    driver.settle();
    assert!(
        driver
            .root
            .next_deadline(&driver.env)
            .is_none_or(|due| due > skein_lib::Time::from_nanos(walking::PERIOD_SPAN.as_nanos())),
        "project zero arms no calendar renewal"
    );
}

#[test]
fn each_person_reads_their_own_rows() {
    let (mut driver, run) = watched();
    let first_session = driver.session();
    let first_account = account(&driver);
    end_by_turn(&mut driver, &run);
    add_account(&mut driver, 1, 97);
    let second_account = own_accounts(&driver).into_iter().find(|account| *account != first_account).unwrap();
    driver.fire_at(Duration::from_secs(2));
    driver.settle();
    acknowledge(&mut driver);
    ask(&mut driver, second_account, false, 98);
    assert_eq!(rows(&driver).iter().map(|row| row.account).collect::<Vec<_>>(), [second_account, first_account]);
    sign_in_second(&mut driver);
    add_account(&mut driver, 1, 99);
    let second_session = driver.session();
    watch(&mut driver, Token::new(402), second_session);
    let second_account = own_accounts(&driver)
        .into_iter()
        .find(|account| *account != first_account && *account != second_account)
        .unwrap();
    driver.send(engine::Event::Party(engine::PartyInput::Delivered { watcher: Token::new(402) }));
    ask(&mut driver, second_account, false, 100);
    driver.send(engine::Event::Party(engine::PartyInput::Delivered { watcher: Token::new(402) }));
    driver.pump();
    let second_person = driver
        .delivered
        .iter()
        .rev()
        .find_map(|delivery| {
            if let Delivery::Trials { watcher, rows } = delivery {
                (*watcher == Token::new(402)).then_some(rows)
            } else {
                None
            }
        })
        .unwrap();
    assert_eq!(second_person.len(), 1, "the family's own scope hides the first person's rows");
    assert_eq!(second_person[0].account, second_account);
    driver.send(engine::Event::Party(engine::PartyInput::Delivered { watcher: WATCHER }));
    driver.send(engine::Event::Party(engine::PartyInput::Unwatch { watcher: WATCHER }));
    driver.settle();
    watch(&mut driver, WATCHER, first_session);
    assert_eq!(rows(&driver).len(), 2);
}

fn read(driver: &mut Driver, read: core::reads::Read) -> core::reads::Answer {
    let reply = Token::new(601);
    driver.send(engine::Event::Party(engine::PartyInput::Read {
        reply_to: ReplyTo::new(reply),
        sign_in: driver.session(),
        read,
    }));
    driver.settle();
    driver
        .delivered
        .iter()
        .rev()
        .find_map(|delivery| {
            if let Delivery::Read { reply: to, answer } = delivery {
                (*to == ReplyTo::new(reply)).then_some(answer.clone())
            } else {
                None
            }
        })
        .expect("the person's read")
}

#[test]
fn it_is_in_no_list() {
    use core::{documents as d, reads as r};
    let mut config = config();
    let project = walking::config(106).core;
    config.core.seed_projects = project.seed_projects;
    config.core.owners = project.owners;
    let mut driver = configured(config, 1);
    driver.period();
    driver.settle();
    let account = account(&driver);
    ask(&mut driver, account, false, 96);
    let run = driver.last_assignment();
    for ended in [false, true] {
        if ended {
            end_by_turn(&mut driver, &run);
        }
        let r::Answer::Document(document) = read(&mut driver, r::Read::Frame) else { panic!("Home's frame") };
        let d::Document::Frame(frame) = *document else { panic!("frame") };
        assert_eq!(frame.projects.len(), 1, "a project is present");
        assert_eq!(frame.needs, 0);
        assert_eq!(frame.updates, d::Count::Known(0));
        let query = d::TaskQuery {
            requester: Some(tasks::Party::Person(frame.party)),
            ..jig_test_profile::fixtures::OWN_TASKS
        };
        let r::Answer::Tasks(page) = read(&mut driver, r::Read::Tasks { query, live: None, ended: None }) else {
            panic!("Chats query")
        };
        assert!(page.live.is_empty());
        if let r::EndedGroup::Rows { rows, .. } = &page.ended {
            assert!(rows.is_empty());
        }
        for group in [r::Group::Needs, r::Group::Updates, r::Group::Results] {
            let r::Answer::Page(page) = read(
                &mut driver,
                r::Read::Inbox { group, filter: r::Filter { project: None, goal: None }, from: None },
            ) else {
                panic!("inbox and results")
            };
            assert!(page.rows.is_empty());
        }
    }
}
