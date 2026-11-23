//! A core procedure document can precede the forge child's change record.
use jig_domain_core::documents::{Watched, Window};
use jig_domain_people::ApplicationScope;
use jig_domain_tasks as tasks;
use skein_lib::{ReplyTo, Token};
use temper_engine_domain as root;
use temper_engine_domain_forge as forge;
use temper_engine_domain_world::{direct::Driver, fixtures, forge_stand::ForgeStand, walking};

const TASK: u64 = 51;
const PRODUCER: u64 = 52;
const EARLY: Token = Token::new(700);
const LATE: Token = Token::new(701);

pub(super) fn before_change() -> Driver {
    // Keep the core's procedure and its live producer independently of the
    // forge child: the panel is watched before that child's record arrives.
    let mut core = jig_tasks_kit::World::new(71, jig_tasks_kit::LIMITS);
    let mut new = jig_tasks_kit::task(TASK, &[]);
    new.executor = tasks::Executor::Procedure { connector: 0, code: 2 };
    new.contract = tasks::Contract::Change { connector: 0, kind: 1, words: 32, choices: Box::new([]) };
    new.spec.parameters = Box::new([tasks::Parameter::Resource { name: 1, connector: 0, resource: forge::tag(1, 2) }]);
    new.authority = fixtures::report_delegate(b"change", Box::new([])).authority;
    new.authority.delegation =
        tasks::Delegation { kinds: Box::new([tasks::AuthorityExecutor::Charter(1)]), tasks: 1, depth: 1 };
    new.numbers.budget = new.authority.budget.spend;
    assert_eq!(core.make(tasks::Party::Person(1), vec![new]), jig_tasks_kit::Reply::Made(vec![TASK]));
    let mut producer = jig_tasks_kit::task(PRODUCER, &[]);
    producer.executor = tasks::Executor::Agent { charter: 1 };
    producer.contract = tasks::Contract::Change { connector: 0, kind: 1, words: 32, choices: Box::new([]) };
    producer.authority = fixtures::report_delegate(b"produce", Box::new([])).authority;
    producer.numbers.budget = producer.authority.budget.spend;
    producer.funder = tasks::Funder::Task(TASK);
    core.send(tasks::Event::Procedure {
        reply_to: ReplyTo::new(Token::new(99)),
        task: TASK,
        step: 1,
        read: None,
        decision: tasks::ProcedureDecision {
            batch: Box::new([producer]),
            ..tasks::ProcedureDecision::new(tasks::ProcedureEnd::Wait)
        },
    });
    assert_eq!(core.record(TASK).phase, tasks::Phase::Active(tasks::Active::Idle));
    // Keep the producer live without a worker while the panel boundary is exercised.
    core.send(tasks::Event::Hold { task: PRODUCER, why: tasks::Hold::Stopped });

    let limits = fixtures::forge_limits();
    let mut store = fixtures::restored_store(walking::config(71), &limits);
    let mut header = store.header();
    header.tasks = PRODUCER;
    let mut records: Vec<_> =
        core.records.into_values().map(|record| root::Record::Core(jig_domain_core::Record::Tasks(record))).collect();
    records.push(root::Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::Deployment(header))));
    store.seed(records.into_boxed_slice(), Box::new([]));
    let mut driver = Driver::configured(store, [walking::config(71), walking::config(71)], &limits);
    driver.forge = Some(ForgeStand::new());
    driver.settle();
    assert!(driver.root.ready() && !driver.stopped, "restored root: {:?}", driver.root.start_phase());
    driver.sign_in();
    driver.settle();
    driver.send(root::Event::Party(root::PartyInput::Application {
        scope: ApplicationScope::Project(1),
        reply_to: ReplyTo::new(Token::new(95)),
        sign_in: driver.session(),
        key: [95; 16],
        digest: [5; 32],
        carried: None,
        request: root::Application::Forge(forge::Ask::Adopt(fixtures::adoption(1))),
    }));
    driver.settle();
    assert_eq!(
        driver.task(TASK).expect("core task exists").executor,
        tasks::Executor::Procedure { connector: 0, code: 2 }
    );
    driver
}

// A quiet producing change has the delegate acknowledged by the root; a bare
// unrequested row still needs its first procedure step.
pub(super) fn producing_row() -> forge::ChangeRow {
    let mut row = temper_engine_forge_world::changes::change_row();
    row.change.state = temper_engine_domain_forge_change::State::Producing { requested: true };
    row.change.since = skein_lib::Wall::from_nanos(0);
    row.delegate = Some((PRODUCER, temper_engine_domain_forge_change::Delegate::Produce));
    row.producing_charter = 1;
    row
}

fn watch(driver: &mut Driver, watcher: Token) {
    driver.send(root::Event::Party(root::PartyInput::Watch {
        watcher,
        sign_in: driver.session(),
        subject: Watched::Application { scope: ApplicationScope::Project(1), connector: 0, subject: Token::new(TASK) },
        window: Window { rows: 0, after: None },
    }));
    driver.settle();
}

#[test]
fn a_panel_before_its_change_ends_gone_and_a_fresh_watch_reads_the_change() {
    let mut driver = before_change();
    watch(&mut driver, EARLY);
    assert!(
        driver.delivered.iter().any(|delivery| matches!(delivery,
            root::Delivery::WatchEnded { watcher, why: jig_domain_views::End::Gone } if *watcher == EARLY
        )),
        "the pre-record watch ended Gone: {:?}",
        driver.delivered
    );
    driver.send(root::Event::Forge(Box::new(forge::Event::Change { row: producing_row() })));
    driver.settle();
    watch(&mut driver, LATE);
    assert!(driver.delivered.iter().any(|delivery| matches!(delivery,
        root::Delivery::Panel { watcher, panel } if *watcher == LATE && panel.task == TASK && panel.step == forge::PanelStep::Producing
    )), "a fresh watch can read the newly made change: {:?}", driver.delivered);
    let original: Vec<_> = driver
        .delivered
        .iter()
        .filter(|delivery| {
            matches!(delivery,
                root::Delivery::Panel { watcher, .. } | root::Delivery::WatchEnded { watcher, .. } if *watcher == EARLY
            )
        })
        .collect();
    assert!(
        original.len() == 1
            && matches!(original[0], root::Delivery::WatchEnded { why: jig_domain_views::End::Gone, .. }),
        "the original watch stays ended Gone: {original:?}"
    );
}
