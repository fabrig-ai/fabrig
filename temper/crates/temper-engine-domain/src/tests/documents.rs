//! Charter documents bind before gathering and retain their titles through completion
//! (temper/docs/design/protocol/configuration/settings.md, 6).
use super::resume_fixture;
use crate::assemble::DOCUMENT_KINDS;
use crate::boundary::Work;
use crate::{BriefBody, BriefKind, Decision, Domain, ForgeBriefKind, Limits};
use alloc::boxed::Box;
use jig_domain_brief as brief;
use jig_domain_contract::{Ask, Connector, Up, parts};
use jig_domain_core as core;
use jig_domain_core::engine::{In, Out, Peer, store};
use jig_domain_tasks as tasks;
use skein_lib::{Env, List, Queue, Token};
use temper_engine_domain_forge as forge;
use temper_engine_domain_forge_change as change;

fn document(heading: Option<&[u8]>) -> parts::Document {
    parts::Document {
        connector: 0,
        name: jig_domain_authority::Name {
            segments: Box::new([
                Box::from(b"forge".as_slice()),
                Box::from(b"forge.example".as_slice()),
                Box::from(b"org".as_slice()),
                Box::from(b"rules".as_slice()),
                Box::from(b"branch".as_slice()),
                Box::from(b"guides".as_slice()),
                Box::from(b"main".as_slice()),
            ]),
        },
        path: b"docs/rules.md".as_slice().into(),
        heading: match heading {
            Some(heading) => Some(Box::from(heading)),
            None => None,
        },
    }
}

fn root(sections: u32, documents: Box<[parts::Document]>) -> (Domain, Env<Limits>) {
    use jig_domain_accounts as accounts;
    use jig_domain_people as people;

    let mut limits = resume_fixture::limits();
    limits.brief.sections = sections;
    limits.forge.brief_sections =
        limits.brief.briefs.checked_mul(limits.brief.sections).unwrap().checked_mul(2).unwrap();
    let mut config = resume_fixture::config(7);
    config.core.defaults.charters[0].documents = documents;
    resume_fixture::root_with_records(
        config,
        &limits,
        &[
            crate::Record::Core(core::Record::People(people::Stored::Person {
                number: 1,
                identity: people::Identity {
                    key: people::IdentityKey { provider: 0, subject: 7_u64.to_be_bytes().into() },
                    login: Box::new([]),
                    name: Box::new([]),
                },
            })),
            crate::Record::Core(core::Record::People(people::Stored::Roles {
                project: 1,
                holdings: Box::new([people::Holding { person: 1, role: people::Role::Maintainer }]),
            })),
            crate::Record::Core(core::Record::Accounts(accounts::Stored::Account {
                number: 1,
                provider: 1,
                holder: accounts::Holder::Deployment,
                place: 0,
                uses: None,
                subject: Box::new([]),
                display: Box::new([]),
                list: None,
            })),
            crate::Record::Core(core::Record::Accounts(accounts::Stored::Source {
                account: 1,
                number: 1,
                place: 0,
                kind: accounts::Kind::None,
                name: Box::new([]),
                held: None,
            })),
            crate::Record::Core(core::Record::Accounts(accounts::Stored::Next { number: 2 })),
        ],
    )
}

fn route(domain: &mut Domain, env: &Env<Limits>, out: Queue<Out>) {
    let mut decision = Decision::decide(&mut domain.journal, &env.limits.journal);
    crate::route::route_engine_requests(domain, env, &mut decision, out);
    accept(domain, env, decision);
}

fn accept(domain: &mut Domain, env: &Env<Limits>, decision: Decision) {
    crate::decision::accept_routed(
        &mut domain.journal,
        &mut domain.engine.core_mut().counters,
        &env.limits.journal,
        decision,
        false,
    )
    .unwrap();
}

fn ask(domain: &mut Domain, env: &Env<Limits>, ask: parts::Ask) {
    let mut out = Queue::with_capacity(1);
    out.push(Out::Ask(Connector(domain.config.forge_connector), Ask::Parts(ask)));
    route(domain, env, out);
}

fn ask_plan(domain: &mut Domain, env: &Env<Limits>, requester: tasks::Party, documents: Box<[parts::Document]>) {
    ask(
        domain,
        env,
        parts::Ask::Plan(Box::new(parts::Preparation {
            task: 8,
            project: 1,
            requester,
            root: 7,
            documents,
            core: Box::new([]),
            slots: env.limits.brief.sections,
            budget: env.limits.brief.brief_bytes,
        })),
    );
}

fn take_plan(domain: &mut Domain, task: u64) -> Box<[parts::Section]> {
    while let Some(work) = domain.work.pop() {
        if let Work::Connector(_, Up::Parts(parts::Up::Planned { task: owner, sections })) = work {
            assert_eq!(owner, task);
            return sections;
        }
    }
    panic!("forge brief plan")
}

#[test]
fn two_documents_are_planned_on_the_forge_with_their_own_limits() {
    let (mut domain, env) = root(8, Box::new([document(None), document(Some(b"Safety"))]));
    let task = make_agent(&mut domain, &env, tasks::Party::Deployment { project: 1 });
    let plan = plan_from_engine(&mut domain, &env, task);
    ask(&mut domain, &env, parts::Ask::Plan(Box::new(plan)));
    let mut tokens = List::with_capacity(2);
    for heading in [None, Some(Box::from(b"Safety".as_slice()))] {
        let Some(Work::Forge(forge::Event::PlanDocument { section, document })) = domain.work.pop() else {
            panic!("documents registered before the plan")
        };
        assert_eq!(
            document.as_ref(),
            &forge::BriefDocument {
                host: b"forge.example".as_slice().into(),
                owner: b"org".as_slice().into(),
                name: b"rules".as_slice().into(),
                branch: b"guides/main".as_slice().into(),
                path: b"docs/rules.md".as_slice().into(),
                heading,
            }
        );
        tokens.push(section).unwrap();
    }
    let sections = take_plan(&mut domain, task);
    assert!(domain.work.is_empty());
    assert_eq!(sections.len(), 2);
    let titles = domain.brief_document_titles.get(&task).unwrap();
    assert_eq!(titles.len(), 2);
    for (index, section) in sections.iter().enumerate() {
        let index = u16::try_from(index).unwrap();
        assert_eq!(section.kind, DOCUMENT_KINDS + index);
        assert_eq!(section.document, Some(index));
        assert_eq!(section.token, *tokens.get(u32::from(index)).unwrap());
        assert_eq!(section.limit, 4096);
        assert_eq!(section.priority, parts::DOCUMENT_PRIORITY);
        assert!(!section.required);
    }
    assert_eq!(titles[0].as_ref(), b"org/rules docs/rules.md");
    assert_eq!(titles[1].as_ref(), b"org/rules docs/rules.md, \"Safety\"");
}

#[test]
fn a_missing_document_section_is_titled_with_its_document() {
    let (mut domain, env) = root(8, Box::new([document(None), document(Some(b"Safety"))]));
    // Titles still belong to this preparation after the connector is told to drop its sections.
    let task = make_agent(&mut domain, &env, tasks::Party::Deployment { project: 1 });
    let plan = plan_from_engine(&mut domain, &env, task);
    ask(&mut domain, &env, parts::Ask::Plan(Box::new(plan)));
    let Some(Work::Forge(forge::Event::PlanDocument { section: first, .. })) = domain.work.pop() else {
        panic!("first document planned")
    };
    let Some(Work::Forge(forge::Event::PlanDocument { section: second, .. })) = domain.work.pop() else {
        panic!("second document planned")
    };
    let Some(Work::Connector(..)) = domain.work.pop() else { panic!("documents offered to the engine") };
    for section in [first, second] {
        ask(&mut domain, &env, parts::Ask::Drop { task, section });
    }
    let completed = crate::assemble::take_sections(
        &mut domain,
        task,
        Box::new([
            brief::GatherPlaced::Missing { connector: 0, kind: DOCUMENT_KINDS, why: brief::GatherMissing::Failed },
            brief::GatherPlaced::Missing { connector: 0, kind: DOCUMENT_KINDS + 1, why: brief::GatherMissing::Late },
        ]),
    );
    for (section, (title, why)) in completed.iter().zip([
        (b"org/rules docs/rules.md".as_slice(), brief::GatherMissing::Failed),
        (b"org/rules docs/rules.md, \"Safety\"".as_slice(), brief::GatherMissing::Late),
    ]) {
        assert_eq!(section.kind, BriefKind::Forge(ForgeBriefKind::Document(title.into())));
        assert_eq!(section.body, BriefBody::Missing(why));
    }
    assert_eq!(crate::budget::section_bytes(&completed), 23 + 33);
    assert!(domain.brief_document_titles.is_empty());
    domain.reclaim();
    assert!(domain.brief_connectors.is_empty());
}

#[test]
fn a_name_that_is_no_forge_branch_is_missing_without_asking_the_forge() {
    let (mut domain, env) = root(8, Box::new([]));
    let mut document = document(Some(b"unused heading"));
    document.name.segments = Box::new([Box::from(b"other".as_slice())]);
    ask_plan(&mut domain, &env, tasks::Party::Person(1), Box::new([document]));
    let Some(Work::Connector(_, Up::Parts(parts::Up::Planned { task: 8, sections }))) = domain.work.pop() else {
        panic!("no forge request for an invalid document")
    };
    assert!(domain.work.is_empty());
    let [parts::Section { token, .. }] = sections.as_ref() else { panic!("planned document") };
    ask(
        &mut domain,
        &env,
        parts::Ask::Gather {
            task: 8,
            section: *token,
            budget: 4096,
            deadline: env.now.saturating_add(env.limits.brief.gather),
        },
    );
    let Some(Work::Connector(_, Up::Parts(parts::Up::Failed { task: 8, section, failure }))) = domain.work.pop() else {
        panic!("missing document")
    };
    assert_eq!(section, *token);
    assert_eq!(failure.fate, parts::Fate::Permanent);
    assert_eq!(failure.words.as_ref(), b"the connector cannot hold this section");
    assert!(domain.work.is_empty());
}

#[test]
fn abandonment_drops_every_document_row_and_its_titles() {
    let (mut domain, env) = root(8, Box::new([document(None), document(None)]));
    let task = make_agent(&mut domain, &env, tasks::Party::Deployment { project: 1 });
    let plan = plan_from_engine(&mut domain, &env, task);
    let (section_plans, answer) = forge_plan(&mut domain, &env, plan);
    let gathered = step(&mut domain, &env, answer);
    assert_eq!(gathered.len(), 2);
    let abandoned =
        step(&mut domain, &env, In::Core(core::Event::Tasks(tasks::Event::Hold { task, why: tasks::Hold::Effects })));
    route(&mut domain, &env, abandoned);
    assert_drops(&mut domain, &env, &section_plans);
}

#[test]
fn releasing_a_restored_close_change_cancels_its_held_job() {
    use jig_domain_tasks as tasks;
    const FAILURE: &[u8] = b"could not prepare the close job";
    let (source, _, change_task) = super::live_task(None, tasks::Executor::Procedure { connector: 0, code: 2 });
    let mut parent = source.engine.core().tasks.task(change_task).unwrap().clone();
    let child_number = change_task + 1;
    parent.delegates = Box::new([child_number]);
    parent.phase = tasks::Phase::Active(tasks::Active::Due);
    parent.due_reason = tasks::Why::Released;
    parent.revisions.hold = 1;
    let mut child = parent.clone();
    child.number = child_number;
    child.requester = tasks::Party::Task(change_task);
    child.depth = 1;
    child.delegates = Box::new([]);
    child.executor = tasks::Executor::Job { kind: forge::CLOSE_KIND };
    child.contract = tasks::Contract::Job { bytes: 1 };
    child.spec.words = Box::new([]);
    child.spec.job = Some(Box::new(tasks::JobSpec {
        bytes: b"close".as_slice().into(),
        deadline: skein_lib::Duration::from_secs(600),
    }));
    child.phase = tasks::Phase::Held {
        was: tasks::Was::Active(tasks::Active::Preparing),
        why: tasks::Hold::Failures(tasks::Class::Permanent),
        source: Some(Box::new(tasks::HoldSource::Preparation { words: FAILURE.into() })),
    };
    let (mut domain, env) = root(8, Box::new([]));
    domain.engine.core_mut().tasks = tasks::Domain::new(
        &env.limits.tasks,
        tasks::Kept { revisions: env.limits.settings.revisions, versions: env.limits.settings.versions },
        7,
        source.engine.core().tasks.newest(),
        source.engine.core().tasks.charters().into(),
        Box::new([0]),
    );
    let mut restored = Queue::uncapped();
    for row in [parent, child] {
        tasks::step(
            &mut domain.engine.core_mut().tasks,
            &env.with(env.limits.tasks),
            tasks::Event::Restore { record: tasks::Stored::Live(Box::new(row)) },
            &mut restored,
        );
    }
    for request in &restored {
        if let tasks::Request::RestoreRefused { .. } = request {
            panic!("fixture restored: {request:?}; {:?}", domain.engine.core().tasks.restore_failure());
        }
    }
    let mut row = super::change_row::change_row(Some(7));
    row.repository.forge = 1;
    row.change.last_head = Some([1; 32]);
    row.task = change_task;
    row.change.task = change_task;
    row.change.state = change::State::Held {
        was: Box::new(change::State::Producing { requested: true }),
        hold: change::Hold::new(change::Cause::Failed, skein_lib::Wall::EPOCH),
    };
    row.close = Some(b"close request".as_slice().into());
    row.delegate = Some((child_number, change::Delegate::Close));
    row.delegate_status = change::Status::Failed;
    row.producer_failure = Some(FAILURE.into());
    domain.forge.restore_begin(env.wall);
    forge::step(
        &mut domain.forge,
        &crate::translate::environment_forge(&env),
        forge::Event::Restore { record: forge::Stored::Change(Box::new(row)) },
        &mut Queue::uncapped(),
    );
    let context = domain.engine.core().tasks.run_context(change_task).expect("released procedure");
    let mut decision = Decision::decide(&mut domain.journal, &env.limits.journal);
    assert!(!crate::route::forge_route::delegate_record::start(&mut domain, &env, &mut decision, &context));
    let row = domain.forge.change(change_task).unwrap();
    assert_eq!(row.change.released, None);
    assert_eq!(row.change.state, change::State::Producing { requested: false });
    assert_eq!(row.delegate, None);
    assert_eq!(row.delegate_status, change::Status::Failed);
    assert_eq!(row.producer_failure.as_deref(), Some(FAILURE));
    let Some(Work::Tasks(tasks::Event::Cancel { by, operation, .. })) = domain.work.pop() else {
        panic!("release must cancel the restored job before the change steps")
    };
    assert_eq!(by, tasks::Party::Task(change_task));
    assert_eq!(operation.task, child_number);
    assert_eq!(operation.reason.as_ref(), b"its change was released");
}

fn restore_change(domain: &mut Domain, env: &Env<Limits>, mut row: forge::ChangeRow, parent: u64, task: u64) {
    row.task = parent;
    row.change.task = parent;
    row.repository.forge = 1;
    row.change.last_head = Some([1; 32]);
    row.delegate = Some((task, change::Delegate::Repair(change::Repair::Ci)));
    domain.forge.restore_begin(env.wall);
    forge::step(
        &mut domain.forge,
        &crate::translate::environment_forge(env),
        forge::Event::Restore { record: forge::Stored::Change(Box::new(row)) },
        &mut Queue::uncapped(),
    );
}

#[test]
fn a_completed_changes_delegate_plans_no_forge_sections() {
    for state in [change::State::Landed { since: skein_lib::Wall::EPOCH, merge: [2; 32] }, change::State::Ended] {
        let (mut domain, env) = root(8, Box::new([]));
        let mut row = super::change_row::change_row(Some(7));
        row.change.state = state;
        restore_change(&mut domain, &env, row, 7, 8);
        ask_plan(&mut domain, &env, tasks::Party::Task(7), Box::new([]));
        assert!(take_plan(&mut domain, 8).is_empty(), "no pull or reason for completed work");
        assert!(domain.work.is_empty(), "no forge section registration");
    }
}

fn make_agent(domain: &mut Domain, env: &Env<Limits>, requester: tasks::Party) -> u64 {
    use jig_domain_link as link;
    use skein_lib::Duration;

    let mut authority = super::held_assignment().run.authority;
    authority.budget.spend = 99;
    authority.models = tasks::Models::All;
    let task = super::projection_settled::make_task_with_budget(
        domain,
        env,
        None,
        tasks::Executor::Agent { charter: 1 },
        requester,
        authority,
        99,
    );
    jig_domain_fleet::step(
        &mut domain.engine.core_mut().fleet,
        &env.with(env.limits.fleet),
        jig_domain_fleet::Event::Hello {
            worker: 0,
            channel: Token::new(7),
            hello: link::Hello {
                fresh: false,
                reads_usage: true,
                jobs: Box::new([]),
                bounds: jig_test_profile::fixtures::bounds(),
                stop_bound: Duration::ZERO,
                slots: 1,
                workstreams: Box::new([]),
                hosting: Box::new([]),
            },
        },
        &mut Queue::uncapped(),
    );
    jig_domain_fleet::step(
        &mut domain.engine.core_mut().fleet,
        &env.with(env.limits.fleet),
        jig_domain_fleet::Event::Logins { channel: Token::new(7), logins: Box::new([]) },
        &mut Queue::uncapped(),
    );
    task
}

fn step(domain: &mut Domain, env: &Env<Limits>, input: In) -> Queue<Out> {
    crate::engine::step(&mut domain.engine, env, input)
}

fn plan_from_engine(domain: &mut Domain, env: &Env<Limits>, task: u64) -> parts::Preparation {
    let context = domain.engine.core().tasks.run_context(task).unwrap();
    assert!(
        domain.engine.core().run_choice(task).is_ok(),
        "fixture run choice: {:?}",
        domain.engine.core().run_choice(task).err()
    );
    let mut out = step(domain, env, In::Core(core::Event::Activate { context, ready: true }));
    for _ in 0..=env.limits.notes.scopes {
        let mut next = None;
        while let Some(output) = out.pop() {
            if let Out::Ask(Connector(connector), Ask::Parts(parts::Ask::Plan(plan))) = output {
                assert_eq!(connector, domain.config.forge_connector);
                assert_eq!(plan.task, task);
                return *plan;
            } else if let Out::Held(Peer::Store(store::Out::Load { load, range: store::Range::Notes(_) })) = output {
                next = Some(load);
            } else if let Out::Core(core::Request::Write(_)) = output {
            } else {
                panic!("expected a note load or plan, got {output:?}");
            }
        }
        let Some(load) = next else {
            panic!("no notes load or forge plan, task: {:?}", domain.engine.core().tasks.task(task));
        };
        out = step(
            domain,
            env,
            In::Store(store::In::Loaded {
                load,
                result: store::Answer::Rows(store::Rows::Records { records: Box::new([]), more: false }),
            }),
        );
    }
    panic!("engine asks the forge to plan after the notes index")
}

fn forge_plan(domain: &mut Domain, env: &Env<Limits>, plan: parts::Preparation) -> (Box<[parts::Section]>, In) {
    ask(domain, env, parts::Ask::Plan(Box::new(plan)));
    while let Some(work) = domain.work.pop() {
        if let Work::Forge(event) = work {
            let mut out = Queue::uncapped();
            forge::step(&mut domain.forge, &crate::translate::environment_forge(env), event, &mut out);
            assert!(out.is_empty(), "registering sections emits nothing");
        } else if let Work::Connector(connector, Up::Parts(parts::Up::Planned { task, sections })) = work {
            let answer = In::Connector(connector, Up::Parts(parts::Up::Planned { task, sections: sections.clone() }));
            return (sections, answer);
        } else {
            panic!("unexpected forge plan work: {work:?}");
        }
    }
    panic!("forge answers the plan")
}

fn repair_fixture(sections: u32) -> (Domain, Env<Limits>, u64) {
    let (mut domain, env) = root(sections, Box::new([document(None), document(None)]));
    let parent = super::projection_settled::make_task(
        &mut domain,
        &env,
        None,
        tasks::Executor::Procedure { connector: 0, code: 2 },
        tasks::Party::Deployment { project: 1 },
    );
    let task = make_agent(&mut domain, &env, tasks::Party::Task(parent));
    restore_change(&mut domain, &env, super::change_row::change_row(Some(7)), parent, task);
    (domain, env, task)
}

#[test]
fn a_repair_replaces_optional_core_sections_and_keeps_both_documents() {
    let (mut domain, env, task) = repair_fixture(5);
    let plan = plan_from_engine(&mut domain, &env, task);
    assert_eq!((plan.core.len(), plan.slots), (2, 1));
    assert!(plan.core[0].required);
    assert!(!plan.core[1].required);
    let (sections, answer) = forge_plan(&mut domain, &env, plan);
    let out = step(&mut domain, &env, answer);
    let asks = out.into_boxed();
    assert_eq!(asks.len(), 4);
    let mut ready = Queue::uncapped();
    for (output, (section, kind)) in asks.iter().zip(sections.iter().zip([DOCUMENT_KINDS, DOCUMENT_KINDS + 1, 1, 3])) {
        let Out::Ask(Connector(connector), Ask::Parts(parts::Ask::Gather { task: owner, section: token, .. })) = output
        else {
            panic!("five slots fit the task, both documents and the required repair sections: {asks:?}");
        };
        assert_eq!((*owner, *token, section.kind), (task, section.token, kind));
        ready = step(
            &mut domain,
            &env,
            In::Connector(Connector(*connector), Up::Parts(parts::Up::Sized { task, section: *token, size: 1 })),
        );
    }
    let Some(Out::Core(core::Request::Now(workspace))) = ready.pop() else { panic!("brief ready for its workspace") };
    let core::Now::WorkspaceRequest { attempt, .. } = *workspace else { panic!("workspace request") };
    let mut prepared = step(
        &mut domain,
        &env,
        In::Core(core::Event::WorkspacePrepared {
            task,
            attempt,
            holds: Box::new([]),
            writes: Some(Box::new([])),
            credentials: Box::new([]),
            workspace: None,
        }),
    );
    while let Some(output) = prepared.pop() {
        if let Out::Core(core::Request::Now(now)) = output
            && let core::Now::RunPrepared { brief, .. } = *now
        {
            let [brief::GatherPlaced::Core { kind: brief::Core::Task, .. }, sections @ ..] = brief.as_ref() else {
                panic!("only optional core context is displaced")
            };
            assert_eq!(sections.len(), 4);
            for (placed, expected) in sections.iter().zip([DOCUMENT_KINDS, DOCUMENT_KINDS + 1, 1, 3]) {
                let brief::GatherPlaced::Connector { kind, .. } = placed else { panic!("kept forge section") };
                assert_eq!(*kind, expected);
            }
            return;
        }
    }
    panic!("run prepared with the placed brief");
}

#[test]
fn required_forge_context_without_optional_core_room_abandons_the_documents() {
    let (mut domain, env, task) = repair_fixture(4);
    let plan = plan_from_engine(&mut domain, &env, task);
    assert_eq!(plan.slots, 0);
    let (sections, answer) = forge_plan(&mut domain, &env, plan);
    let out = step(&mut domain, &env, answer);
    assert_eq!(sections.len(), 4);
    route(&mut domain, &env, out);
    let Some(crate::Fact::BriefFailed { task: failed, counts, reason }) = domain.pop_fact() else {
        panic!("required sections cannot displace documents or required core context")
    };
    assert_eq!(
        (failed, counts, reason.as_ref()),
        (task, "permanent", b"brief: no room for its required sections".as_slice())
    );
    assert_drops(&mut domain, &env, &sections);
}

fn assert_drops(domain: &mut Domain, env: &Env<Limits>, sections: &[parts::Section]) {
    let mut out = Queue::uncapped();
    for planned in sections {
        let Some(Work::Forge(forge::Event::DropBrief { section })) = domain.work.pop() else {
            panic!("every planned section is dropped")
        };
        assert_eq!(section, planned.token);
        forge::step(
            &mut domain.forge,
            &crate::translate::environment_forge(env),
            forge::Event::DropBrief { section },
            &mut out,
        );
    }
    assert!(out.is_empty());
    assert!(domain.forge.briefs_idle());
    assert!(domain.work.is_empty(), "only the planned sections are dropped");
    assert!(domain.brief_document_titles.is_empty());
    domain.reclaim();
    assert!(domain.brief_connectors.is_empty());
}

enum DocumentHandoff {
    Held,
    Lost,
    DifferentSize,
}

fn gathered_document(handoff: DocumentHandoff) -> (crate::Assignment, u32) {
    let (mut domain, env) = root(8, Box::new([document(Some(b"Safety"))]));
    let task = make_agent(&mut domain, &env, tasks::Party::Deployment { project: 1 });
    let plan = plan_from_engine(&mut domain, &env, task);
    let (sections, answer) = forge_plan(&mut domain, &env, plan);
    let [section] = sections.as_ref() else { panic!("one document") };
    assert_eq!(section.kind, DOCUMENT_KINDS);
    let gathered = step(&mut domain, &env, answer);
    route(&mut domain, &env, gathered);
    let Some(Work::Forge(event @ forge::Event::GatherPlanned { section: token, .. })) = domain.work.pop() else {
        panic!("forge gather")
    };
    assert_eq!(token, section.token);
    let mut out = Queue::uncapped();
    forge::step(&mut domain.forge, &crate::translate::environment_forge(&env), event, &mut out);
    let mut decision = Decision::decide(&mut domain.journal, &env.limits.journal);
    crate::route::forge_route::outputs(&mut domain, &env, &mut decision, &mut out);
    accept(&mut domain, &env, decision);
    let Some(Work::Connector(connector, Up::Parts(parts::Up::Sized { task: owner, section, size }))) =
        domain.work.pop()
    else {
        panic!("forge sized the document")
    };
    assert_eq!((owner, section), (task, token));
    let reported = match handoff {
        DocumentHandoff::Held => size,
        DocumentHandoff::Lost => {
            assert!(domain.forge.take_brief(token).is_some());
            size
        }
        DocumentHandoff::DifferentSize => size.checked_add(1).unwrap(),
    };
    let completed = step(
        &mut domain,
        &env,
        In::Connector(connector, Up::Parts(parts::Up::Sized { task, section, size: reported })),
    );
    assert!(domain.brief_document_titles.contains_key(&task));
    assert_eq!(domain.held_bytes(), 0, "gathered text is not yet charged to the assignment");
    route(&mut domain, &env, completed);
    let Some(Work::Core(prepared @ core::Event::WorkspacePrepared { .. })) = domain.work.pop() else {
        panic!("workspace prepared after the brief")
    };
    let prepared = step(&mut domain, &env, In::Core(prepared));
    route(&mut domain, &env, prepared);
    let assignment = crate::link::take_assignment(&mut domain, task).expect("prepared assignment");
    assert_eq!(domain.held_bytes(), crate::budget::section_bytes(&assignment.sections));
    assert!(domain.brief_document_titles.is_empty());
    domain.reclaim();
    assert!(domain.brief_connectors.is_empty());
    assert!(domain.forge.briefs_idle());
    (assignment, size)
}

#[test]
fn a_gathered_document_keeps_its_kind_and_frees_the_binding_when_prepared() {
    let (assignment, size) = gathered_document(DocumentHandoff::Held);
    let section = assignment.sections.last().expect("document follows core text");
    assert_eq!(
        section.kind,
        BriefKind::Forge(ForgeBriefKind::Document(b"org/rules docs/rules.md, \"Safety\"".as_slice().into()))
    );
    assert_eq!(
        section.body,
        BriefBody::Text(b"Repository org/rules on forge.example is not adopted.".as_slice().into())
    );
    assert_eq!(crate::budget::section_bytes(::core::slice::from_ref(section)), u64::from(size) + 33);
}

#[test]
fn an_unavailable_gathered_document_goes_into_the_prepared_run_as_missing() {
    for handoff in [DocumentHandoff::Lost, DocumentHandoff::DifferentSize] {
        let (assignment, _) = gathered_document(handoff);
        let section = assignment.sections.last().expect("missing document follows core text");
        assert_eq!(
            section.kind,
            BriefKind::Forge(ForgeBriefKind::Document(b"org/rules docs/rules.md, \"Safety\"".as_slice().into()))
        );
        assert_eq!(section.body, BriefBody::Missing(brief::GatherMissing::Failed));
        assert_eq!(crate::budget::section_bytes(::core::slice::from_ref(section)), 33, "only its title is charged");
    }
}

// Own-scope application documents (temper/docs/design/domain/forge/attention.md, 1).
fn dismissal_root() -> (Domain, Env<Limits>) {
    use jig_domain_people as people;
    let mut config = resume_fixture::config_with_project_room(1, 2);
    let seed = config.core.seed_projects.first().unwrap();
    let second = jig_domain_core::SeedProject { number: 2, name: "Second".into(), policy: seed.policy.clone() };
    let first =
        jig_domain_core::SeedProject { number: seed.number, name: seed.name.clone(), policy: seed.policy.clone() };
    config.core.seed_projects = Box::new([first, second]);
    let mut limits = resume_fixture::limits();
    limits.authority.projects = 2;
    limits.people.projects = 3;
    let mut records = List::uncapped();
    for (party, subject) in [(1, b"first".as_slice()), (2, b"second".as_slice())] {
        for row in [
            people::Stored::Person {
                number: party,
                identity: people::Identity {
                    key: people::IdentityKey { provider: 0, subject: subject.into() },
                    login: Box::new([]),
                    name: Box::new([]),
                },
            },
            people::Stored::SignIn {
                number: party.checked_mul(10).unwrap(),
                person: party,
                expires: skein_lib::Wall::from_nanos(100),
            },
        ] {
            records.push(crate::Record::Core(jig_domain_core::Record::People(row))).unwrap();
        }
    }
    records
        .push(crate::Record::Core(jig_domain_core::Record::People(people::Stored::Roles {
            project: 1,
            holdings: Box::new([people::Holding { person: 1, role: people::Role::Observer }]),
        })))
        .unwrap();
    records
        .push(crate::Record::Forge {
            row: Box::new(forge::Stored::Dismissed { party: 2, what: forge::PersonFact::ToBegin { project: 2 } }),
        })
        .unwrap();
    resume_fixture::root_with_records(config, &limits, records.as_slice())
}

fn settle_dismissals(domain: &mut Domain, env: &Env<Limits>) -> Box<[crate::Request]> {
    let mut out = Queue::uncapped();
    let mut requests = List::uncapped();
    for _ in 0_u32..32 {
        crate::release(domain, env, &mut out);
        crate::release(domain, env, &mut out);
        while let Some(request) = out.pop() {
            if let crate::Request::Store(crate::StoreRequest::Commit { number, .. }) = request {
                crate::try_step(domain, env, crate::Event::Store(crate::StoreInput::Committed { number, fill: 0 }))
                    .unwrap();
            } else if let crate::Request::ToChild(released) = request {
                crate::try_step(domain, env, crate::Event::Released(released)).unwrap();
            } else {
                requests.push(request).unwrap();
            }
        }
        crate::try_step(domain, env, crate::Event::Resume).unwrap();
    }
    requests.into_boxed()
}

fn dismissal_document(watcher: Token, projects: &[u32]) -> crate::Request {
    let mut rows = List::with_capacity(u32::try_from(projects.len()).unwrap());
    for project in projects {
        rows.push(forge::PersonFact::ToBegin { project: *project }).unwrap();
    }
    crate::Request::Party(crate::PartyRequest::Deliver(crate::Delivery::ForgeDocument {
        watcher,
        document: Box::new(forge::Document::Dismissals(rows.into_boxed())),
    }))
}

fn watch_dismissals(domain: &mut Domain, env: &Env<Limits>, party: u64) -> Token {
    let watcher = Token::new(100_u64.checked_add(party).unwrap());
    crate::try_step(
        domain,
        env,
        crate::Event::Party(crate::PartyInput::Watch {
            watcher,
            sign_in: party.checked_mul(10).unwrap(),
            subject: jig_domain_core::documents::Watched::Application {
                scope: jig_domain_people::ApplicationScope::Own,
                connector: 0,
                subject: Token::new(forge::DISMISSED_SUBJECT),
            },
            window: jig_domain_core::documents::Window { rows: 1, after: None },
        }),
    )
    .unwrap();
    let projects: &[u32] = if party == 1 { &[] } else { &[2] };
    assert_eq!(settle_dismissals(domain, env).as_ref(), [dismissal_document(watcher, projects)]);
    crate::try_step(domain, env, crate::Event::Party(crate::PartyInput::Delivered { watcher })).unwrap();
    watcher
}

fn commit_dismissal(domain: &mut Domain, env: &Env<Limits>) -> Box<[crate::Request]> {
    crate::try_step(
        domain,
        env,
        crate::Event::Party(crate::PartyInput::Application {
            scope: jig_domain_people::ApplicationScope::Project(1),
            reply_to: skein_lib::ReplyTo::new(Token::new(90)),
            sign_in: 10,
            key: [1; 16],
            digest: [1; 32],
            carried: None,
            request: crate::Application::Dismiss { what: forge::PersonFact::ToBegin { project: 1 } },
        }),
    )
    .unwrap();
    let mut out = Queue::uncapped();
    let mut commit = None;
    for _ in 0_u32..8 {
        crate::release(domain, env, &mut out);
        while let Some(request) = out.pop() {
            let crate::Request::Store(crate::StoreRequest::Commit { number, writes }) = request else {
                panic!("dismissal delivered before commit: {request:?}");
            };
            assert!(writes.contains(&crate::Write::Save(crate::Record::Forge {
                row: Box::new(forge::Stored::Dismissed { party: 1, what: forge::PersonFact::ToBegin { project: 1 } })
            })));
            assert!(commit.replace(number).is_none());
        }
    }
    crate::try_step(
        domain,
        env,
        crate::Event::Store(crate::StoreInput::Committed { number: commit.expect("dismissal commit"), fill: 0 }),
    )
    .unwrap();
    settle_dismissals(domain, env)
}

fn dismissal_answer() -> crate::Request {
    crate::Request::Party(crate::PartyRequest::Deliver(crate::Delivery::Application {
        to: skein_lib::ReplyTo::new(Token::new(90)),
        outcome: Some(crate::ApplicationAnswer::Dismiss(forge::DismissAnswer::Dismissed)),
    }))
}

#[test]
fn a_dismissal_makes_the_watchers_document_again() {
    let (mut domain, env) = dismissal_root();
    let watcher = watch_dismissals(&mut domain, &env, 1);
    let got = commit_dismissal(&mut domain, &env);
    assert_eq!(got.len(), 2, "one answer and one document: {got:?}");
    assert!(got.contains(&dismissal_answer()));
    assert!(got.contains(&dismissal_document(watcher, &[1])));
}

#[test]
fn two_parties_watching_dismissed_get_their_own_documents() {
    let (mut domain, env) = dismissal_root();
    let first = watch_dismissals(&mut domain, &env, 1);
    let second = watch_dismissals(&mut domain, &env, 2);
    let got = commit_dismissal(&mut domain, &env);
    assert_eq!(got.len(), 3, "one answer and two documents: {got:?}");
    assert!(got.contains(&dismissal_answer()));
    assert!(got.contains(&dismissal_document(first, &[1])));
    assert!(got.contains(&dismissal_document(second, &[2])));
}
