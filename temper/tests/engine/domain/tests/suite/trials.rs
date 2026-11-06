//! A person's trial through the root, without a project or a real host
//! (temper/docs/design/domain/trial.md, 2 to 6).
use jig_domain_accounts as accounts;
use jig_domain_core as core;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_lib::{ReplyTo, Token};
use temper_engine_domain::{self as engine, Delivery};
use temper_engine_domain_world::{direct::Driver, store::Store, walking};

fn config() -> engine::Config {
    let mut config = walking::config(106);
    config.core.seed_projects = Box::new([]);
    config.core.owners[0].project = people::DEPLOYMENT;
    let mut trial = walking::charter();
    trial.number = 10;
    trial.policy.inspect = false;
    trial.policy.modify = false;
    trial.policy.shell = false;
    trial.policy.agents = false;
    trial.policy.tools = tasks::Tools(0);
    config.core.defaults.charters = Box::new([walking::charter(), trial]);
    config
}

fn driver() -> Driver {
    let mut limits = walking::limits();
    limits.tasks.charters = 2;
    let mut driver = Driver::cold(Store::new(), config(), &limits);
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
            change: people::AccountChange::AddKey { provider: 1, join: None, digest: [8; 32] },
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
    driver.settle();
}

#[test]
fn a_try_runs_one_task_of_charter_ten_naming_no_llm() {
    let mut driver = driver();
    let account = account(&driver);
    ask(&mut driver, account, false, 96);
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
