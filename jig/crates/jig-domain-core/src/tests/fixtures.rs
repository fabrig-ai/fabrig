pub fn bounds() -> Bounds {
    use skein_lib::Duration;
    Bounds {
        program_endpoints: Box::new([]),
        endpoints: Box::new([EndpointBound { endpoint: 1, window: 8192, output: 1024 }]),
        turns: u32::MAX,
        spend: u64::MAX,
        time: Duration::from_secs(3600),
        waiting: Duration::from_secs(3600),
        checks: Duration::from_nanos(1),
        message_bytes: u32::MAX,
        inbox: u32::MAX,
        unacknowledged: u32::MAX,
        reply_bytes: None,
    }
}

#[must_use]
pub fn needs() -> jig_domain_fleet::Needs {
    jig_domain_fleet::Needs { charters: Box::new([]), spend: 0, message_bytes: 0, start_messages: 0, unacknowledged: 0 }
}

/// A runtime unit fixture has completed the start's needs restoration.
pub(crate) fn restore_needs(domain: &mut jig_domain_fleet::Domain, limits: &Limits, needs: jig_domain_fleet::Needs) {
    let mut out = Queue::uncapped();
    jig_domain_fleet::step(
        domain,
        &Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.fleet },
        jig_domain_fleet::Event::Needs(needs),
        &mut out,
    );
    assert!(out.is_empty(), "fixture has no early hellos");
    settle_fleet(domain, &Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.fleet });
}

pub(crate) fn restored_fleet(limits: &Limits, needs: jig_domain_fleet::Needs) -> jig_domain_fleet::Domain {
    let mut domain = jig_domain_fleet::Domain::new(&limits.fleet, limits.settings.versions, needs.clone());
    restore_needs(&mut domain, limits, needs);
    domain
}

/// Make one bounded batch and apply its projection saves to the unit core.
pub(crate) fn make_batch(core: &mut Core, limits: &Limits, creator: tasks::Party, batch: Box<[tasks::New]>) {
    make_batch_at(core, limits, creator, batch, Wall::EPOCH);
}

pub(crate) fn make_batch_at(
    core: &mut Core,
    limits: &Limits,
    creator: tasks::Party,
    batch: Box<[tasks::New]>,
    wall: Wall,
) {
    use super::{Env, Queue, ReplyTo, Time, Token, projections, tasks};
    let mut out = Queue::uncapped();
    tasks::step(
        &mut core.tasks,
        &Env { now: Time::ZERO, wall, limits: limits.tasks },
        tasks::Event::Make { reply_to: ReplyTo::new(Token::new(1)), creator, batch },
        &mut out,
    );
    while let Some(request) = out.pop() {
        if let tasks::Request::Save { record: tasks::Stored::Live(row) } = request {
            let mut projections = Queue::uncapped();
            projections::save(core, limits, &row, &mut projections);
        }
    }
}

use super::*;
use jig_domain_link::{Bounds, EndpointBound};

pub(crate) fn list_record(names: u64, read: bool, failure: Option<accounts::ReadFailure>) -> accounts::ListRecord {
    accounts::ListRecord {
        names,
        read: read.then_some(Wall::EPOCH),
        tried: Some(Wall::from_nanos(1)),
        failure,
        left_out: 0,
    }
}

/// Restore key accounts with their last list records, as a restart does.
pub(crate) fn listed_accounts(
    core: &mut Core,
    limits: &Limits,
    provider: u32,
    rows: &[(u32, u32, Option<accounts::ListRecord>)],
) {
    core.accounts = accounts::Domain::new(&limits.accounts, core.settings.llm_providers.clone());
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.accounts };
    let mut out = Queue::uncapped();
    for &(number, source, list) in rows {
        for record in [
            accounts::Stored::Account {
                number,
                provider,
                holder: accounts::Holder::Deployment,
                place: 0,
                uses: None,
                subject: Box::new([]),
                display: Box::new([]),
                list,
            },
            accounts::Stored::Source {
                account: number,
                number: source,
                place: 0,
                kind: accounts::Kind::Key,
                name: Box::new([]),
                held: None,
            },
        ] {
            accounts::step(&mut core.accounts, &env, accounts::Event::Restore { record }, &mut out);
        }
    }
    runs::run_choice::account(core, limits, accounts::Event::Want);
    for &(_, source, _) in rows {
        runs::run_choice::granted(core, limits, source);
    }
}

pub(crate) fn query_fixture() -> (Core, Limits) {
    let (mut core, mut limits) = tree_core();
    let (auth, auth_limits) = watch_core();
    core.people = auth.people;
    limits.people = auth_limits.people;
    for (number, when, creator) in [(1, 10, 1), (3, 20, 2)] {
        let batch: Box<[tasks::New]> = if number == 1 {
            let first = new_row(&core, 1, tasks::Funder::Period { project: 1, period: 1 }, Some(1));
            let mut second = new_row(&core, 2, tasks::Funder::Period { project: 1, period: 1 }, None);
            second.dependencies = Box::new([1]);
            Box::new([first, second])
        } else {
            Box::new([new_row(&core, number, tasks::Funder::Period { project: 1, period: 1 }, None)])
        };
        make_batch_at(&mut core, &limits, tasks::Party::Person(creator), batch, Wall::from_nanos(when));
        assert!(core.tasks.task(number).is_some(), "task {number}");
    }
    (core, limits)
}

/// A fresh tasks domain over `core`'s configured charters and models, as a restart builds one.
pub(crate) fn fresh_tasks(core: &Core, limits: &Limits) -> tasks::Domain {
    tasks::Domain::new(
        &limits.tasks,
        tasks::Kept { revisions: limits.settings.revisions, versions: limits.settings.versions },
        17,
        core.current.newest(),
        charters::models(&core.current, &core.settings, limits),
        charters::efforts(&core.settings),
    )
}

pub(crate) fn restore(core: &mut Core, limits: &Limits, rows: Box<[tasks::TaskRecord]>) {
    core.tasks = fresh_tasks(core, limits);
    for row in rows {
        let number = row.number;
        send_task(core, limits, tasks::Event::Restore { record: tasks::Stored::Live(Box::new(row)) });
        assert!(core.tasks.task(number).is_some(), "restored task {number}");
    }
}

/// An ended report whose result and two replies can be marked read.
pub(crate) fn ended_report(number: u64, requester: tasks::Party) -> tasks::TaskRecord {
    let (mut core, limits) = tree_core();
    let new = new_row(&core, number, tasks::Funder::Period { project: 1, period: 1 }, None);
    make_batch(&mut core, &limits, tasks::Party::Person(1), Box::new([new]));
    let mut row = core.tasks.task(number).expect("fixture task").clone();
    row.requester = requester;
    row.flags = tasks::Flags {
        ended_by: tasks::EndedBy::Result,
        conversation: true,
        listed: false,
        retention: tasks::Retention::ForgottenOnceRead,
    };
    row.phase =
        tasks::Phase::Ended(tasks::Ending::Done(tasks::TaskResult::Report { words: b"done".as_slice().into() }));
    row.ended_at = Some(Wall::from_nanos(1));
    row.result_position = 7;
    row.last_reply = 2;
    row
}

pub(crate) fn unchanged(reason: &[u8]) -> tasks::Amendment {
    tasks::Amendment {
        tracked: None,
        addressed: None,
        instructions: None,
        spec: None,
        wake: None,
        dependencies: None,
        authority: None,
        llm: None,
        release: None,
        reason: reason.into(),
    }
}

/// An amendment retaining every field, with the supplied explanation.
#[must_use]
pub(crate) fn person_unchanged(reason: &[u8]) -> jig_domain_people::Amendment {
    jig_domain_people::Amendment {
        llm: None,
        tracked: None,
        addressed: None,
        instructions: None,
        spec: None,
        wake: None,
        dependencies: None,
        authority: None,
        reason: reason.into(),
    }
}

pub(crate) fn creating(core: &Core, limits: &Limits, party: u64, project: u32) -> crate::acts::Admission {
    crate::acts::bounds::creating(core, limits, party, project, core.people.role(party, project))
}

pub(crate) fn member(words: &[u8], spend: u64) -> jig_domain_people::Member {
    use jig_domain_people as people;
    people::Member {
        symbols: Box::new([]),
        llm: None,
        deadline: jig_domain_people::Deadline::Named,
        executor: people::TaskExecutor::Agent { charter: 1 },
        words: words.into(),
        parameters: Box::new([]),
        inputs: Box::new([]),
        budget: people::Budget::Named,
        named: Box::new([]),
        contract: people::Contract::Report { words: 16 },
        authority: people::Authority {
            tools: 0,
            models: None,
            grants: Box::new([]),
            delegation: people::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 },
            spend,
            deadline: None,
            notes: 0,
            note_resources: Box::new([]),
        },
        wake: Some(people::WakePolicy {
            words: people::WakeRule::Immediate,
            notices: people::WakeRule::Never,
            news: people::WakeRule::Never,
            results: people::ResultsWake::Never,
            questions: false,
            answers: false,
            timers: false,
        }),
        dependencies: Box::new([]),
        tracked: None,
        label: 0,
        flags: people::Flags::PLAIN,
    }
}

/// Test-only mutators of the settings' current state, kept with the core's
/// tests now that the state lives in `jig-domain-core-vocabulary`.
pub(crate) trait CurrentFixtures {
    fn test_charter_mut(&mut self, number: u32) -> &mut crate::Charter;
    fn test_templates(&mut self, templates: Box<[crate::Template]>);
}

#[expect(clippy::panic, reason = "the core's test fixtures assert their setup values")]
impl CurrentFixtures for crate::settings::Current {
    fn test_charter_mut(&mut self, number: u32) -> &mut crate::Charter {
        use crate::settings::{Setting, Value};
        drop(self.resolved.remove(&number));
        match &mut self.defaults.get_mut(&Setting::Charter(number)).expect("fixture default charter").value {
            Value::Charter(value) => value,
            Value::Overlay(_)
            | Value::Template(_)
            | Value::Policy(_)
            | Value::Rules(_)
            | Value::Model(_)
            | Value::Application(_) => {
                panic!("fixture charter value")
            }
        }
    }
    fn test_templates(&mut self, templates: Box<[crate::Template]>) {
        use crate::settings::{Setting, Value};
        let mut old = skein_lib::List::with_capacity(self.defaults.capacity());
        for (&setting, _) in &self.defaults {
            match setting {
                Setting::Template(_) => old.push(setting).expect("fixture defaults"),
                Setting::Charter(_)
                | Setting::Policy(_)
                | Setting::Model(_)
                | Setting::Rules
                | Setting::ProjectPolicy { .. } => {}
                Setting::Application { .. } => panic!("fixture defaults contain only jig settings"),
            }
        }
        for setting in old.into_boxed() {
            self.defaults.remove(&setting);
        }
        for template in templates {
            self.defaults
                .insert(
                    Setting::Template(template.number),
                    crate::settings::Default { value: Value::Template(Box::new(template)), revision: 1 },
                )
                .expect("fixture templates");
        }
    }
}
