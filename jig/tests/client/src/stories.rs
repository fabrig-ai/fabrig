//! Fixture facts and scripts (jig/docs/design/client/domain/testing.md, 4–6).
#![cfg_attr(
    not(test),
    expect(clippy::wildcard_imports, reason = "story fixtures share the complete client vocabulary")
)]
use crate::{Scripted, Settings, Test, World, scripted::Family, test::list};
use jig_client_domain::contract::*;
use jig_client_scripted_engine::{Bounds, Script, Truth};
use jig_client_test_domain::Address as RootAddress;
use jig_client_view::address::Address;
use skein_lib::{Duration, Wall};
pub mod client_construction;
pub mod closing;
pub mod conversations;
pub mod goals;
pub mod own_proposal;
pub mod person_confirmation;
pub mod shared;
pub mod text_pages;
/// Creates a bounded fixture word sequence.
#[must_use]
pub fn text(bytes: &[u8]) -> Text {
    Text::new(bytes.into(), 32).expect("fixture text fits")
}
/// Creates a signed-in party with a chat and goal charter.
#[must_use]
pub fn frame(party: u64) -> Me {
    Me {
        providers: skein_lib::List::with_capacity(0),
        providers_cut: false,
        key_bytes: 256,
        redirect_bytes: 4096,
        party: MeParty {
            number: party,
            kind: PartyKind::Person,
            login: DocumentText::complete(text(b"fixture-login")),
            name: DocumentText::complete(text(b"Pat")),
        },
        projects: list([MeProject {
            models: skein_lib::List::with_capacity(0),
            templates: conversations::templates(),
            creating: Some(goals::creating()),
            project: 1,
            name: DocumentText::complete(text(b"First")),
            role: HeldRole::Policy { role: 1 },
            pool: Some(PoolLedger {
                source: Funder::Pool { project: 1, person: party, period: 1 },
                parent: None,
                numbers: OwnerNumbers { budget: 100, spent: 0, spent_below: 0, reserved: 0 },
                left: 100,
                closed: false,
                began: None,
                next: None,
            }),
        }]),
        needs: Needs { questions: 0, proposals: 0, escalations: 0, person_tasks: 0 },
        needs_total: 0,
        results_seen: 0,
        cut: false,
        updates: Count { value: 0, limited: false },

        retention: Duration::from_secs(1000),
        deployment_role: Some(HeldRole::Policy { role: 1 }),
    }
}
/// Starts the standard fixture at the requested seed.
#[must_use]
pub fn world(seed: u64) -> World<Test, Scripted> {
    configured(Settings { seed, ..Settings::default() })
}
/// Starts the standard fixture with explicit loop settings.
#[must_use]
pub fn configured(settings: Settings) -> World<Test, Scripted> {
    configured_rows(settings, 8)
}
/// Starts a scripted fixture with the test client's declared page window.
#[must_use]
pub fn seeded_world(seed: u64) -> World<Test, Scripted> {
    configured_rows(Settings { seed, ..Settings::default() }, crate::test::limits().client.domain.rows)
}
fn configured_rows(settings: Settings, rows: u32) -> World<Test, Scripted> {
    let seed = settings.seed;
    let mut truth = Truth::default();
    truth.frames.insert(7, frame(7));
    truth.frames.insert(8, frame(8));
    truth.sign_ins.insert(7, 7);
    World::new(
        settings,
        Scripted::new(
            Script {
                versions: VersionRange { first: 19, last: 19 },
                truth,
                bounds: Bounds {
                    text: 32,
                    rows,
                    authority: 8,
                    segments: 4,
                    document_bytes: 65536,
                    commit_delay: Duration::from_millis(5),
                    heartbeat: Duration::from_millis(20),
                    watches: 64,
                    watches_per_party: 32,
                },
                rules: vec![],
            },
            seed,
            Family::default(),
        ),
    )
}
/// Names the party’s inbox.
#[must_use]
pub fn inbox() -> RootAddress {
    RootAddress::Jig(Address::Inbox { project: None })
}

fn owner_authority() -> OwnerAuthorityDocument {
    OwnerAuthorityDocument {
        authority: OwnerAuthority {
            models: None,
            tools: 0,
            grants: list([]),
            delegation: OwnerDelegation { kinds: list([]), tasks: 1, depth: 1 },
            spend: 1,
            deadline: None,
            notes: 0,
            note_resources: list([]),
        },
        cut: false,
    }
}
fn batch_task(number: u64, spec: SpecificationDocument, executor: OwnerExecutor, budget: u64) -> ProposedTask {
    ProposedTask {
        deadline: jig_client_domain::contract::NewDeadline::Named,
        number,
        spec,
        executor,
        contract: TaskContractDocument::Report { words: 32 },
        authority: OwnerAuthorityDocument {
            authority: OwnerAuthority {
                models: None,
                tools: 0,
                grants: list([]),
                delegation: OwnerDelegation { kinds: list([]), tasks: 1, depth: 1 },
                spend: budget,
                deadline: None,
                notes: 0,
                note_resources: list([]),
            },
            cut: false,
        },
        budget,
        funder: Funder::Period { project: 1, period: 3 },
        dependencies: list([]),
        resources: list([]),
        wake: OwnerWakePolicy {
            words: WakeRule::Never,
            notices: WakeRule::Never,
            news: WakeRule::Never,
            results: ResultsWake::Never,
            questions: false,
            answers: false,
            timers: false,
        },
        priority: None,
        cut: false,
    }
}
/// Creates the initial chat task and its write offer.
#[must_use]
pub fn task() -> Task {
    Task {
        dependency_facts: skein_lib::List::with_capacity(0),
        kept_account: None,
        run_wait: None,
        settings: Some(TaskSettings { task: 1, run: None, newest: 1 }),
        llm: None,
        label: 0,
        flags: TaskFlags { ended_by: EndedBy::Result, conversation: false, listed: true, retention: Retention::Kept },
        last_activity: Wall::EPOCH,
        waits_on: WaitsOn::Nothing,
        chip: Chip {
            task: 9,
            title: DocumentText::complete(text(b"Chat nine")),
            executor: Executor::Agent { charter: 1 },
            phase: Phase::Running,
            details: Box::new(owner_chip(OwnerChipPhase::Active(RunStage::Running { attempt: 1 }), None)),
        },
        goal: None,
        priority: Some(1),
        lineage_chips: list([]),
        inputs: list([]),
        project: 1,
        requester: OwnerRequester::Person(7),
        lineage: list([]),
        spec: specification(text(b"spec")),
        spec_revision: Revision { value: 1 },
        person_revision: None,
        taken_by: None,
        instructions: Some(DocumentText::complete(text(b"instructions"))),
        wake: OwnerWakePolicy {
            words: WakeRule::Never,
            notices: WakeRule::Never,
            news: WakeRule::Never,
            results: ResultsWake::Never,
            questions: false,
            answers: false,
            timers: false,
        },
        subscriptions: Subscriptions { entries: list([]) },
        result_contract: TaskContractDocument::Report { words: 32 },
        dependencies: list([]),
        cut: false,
        references: list([]),
        resources: list([]),
        budget: OwnerBudget {
            numbers: OwnerNumbers { budget: 1, spent: 0, spent_below: 0, reserved: 0 },
            left: 1,
            funder: Funder::Pool { project: 1, person: 7, period: 3 },
            deadline: None,
        },
        authority: owner_authority(),
        hold: None,
        proposals: list([]),
        activity: 0,
        changed: Wall::EPOCH,
        acts: list([Offered { effect: None, act: ActKind::Message, admission: Admission::Allowed }]),
        result: None,
        delegates: list([]),

        context: Some(Box::new(jig_client_domain::contract::TaskContext {
            parts: list([]),
            closing: None,
            spend: list([]),
            runtime: Runtime::Idle,
            impact: Impact {
                tasks: PhaseCounts {
                    ended: 0,
                    waiting: 0,
                    running: 0,
                    held: 0,
                    closing: 0,
                    done: 0,
                    failed: 0,
                    cancelled: 0,
                },
                live_runs: 0,
                proposals: 0,
            },
            writer_slots: list([]),
        })),
    }
}

/// Starts the scripted first-chat decision fixture.
#[must_use]
pub fn chat_world(seed: u64) -> World<Test, Scripted> {
    let mut world = world(seed);
    let task = conversations::chat(9, b"Chat nine", Wall::EPOCH);
    world
        .front()
        .engine
        .script(jig_client_scripted_engine::Rule {
            party: Some(7),
            ask: jig_client::shapes::chat(&frame(7), 1, text(b"Hello")).expect("chat recipe"),
            answer: Answer::Done(Outcome::Made { what: Made::Tasks { tasks: list([9]) } }),
            changes: vec![
                jig_client_scripted_engine::Change::Source(jig_client_scripted_engine::Source {
                    party: 7,
                    read: Read::Tasks {
                        query: jig_client_view::queries::conversations(7, 1, None),
                        live: None,
                        ended: None,
                    },
                    rows: jig_client_scripted_engine::Rows::Tasks(vec![TaskLine {
                        chip: {
                            let mut chip = task.chip.clone();
                            chip.title = DocumentText::complete(text(b"Hello"));
                            chip
                        },
                        project: 1,
                        requester: jig_client_domain::contract::OwnerRequester::Person(1),
                        tracked: None,
                        label: 1,
                        flags: jig_client_domain::contract::TaskFlags {
                            ended_by: jig_client_domain::contract::EndedBy::Result,
                            conversation: true,
                            listed: false,
                            retention: jig_client_domain::contract::Retention::ForgottenOnceRead,
                        },
                        changed: Wall::EPOCH,
                        last_activity: Wall::EPOCH,
                        ended_at: None,
                        budget: task.budget.clone(),
                        waits_on: jig_client_domain::contract::WaitsOn::Nothing,
                        phases: jig_client_domain::contract::OwnerPhaseCounts {
                            waiting: 0,
                            idle: 0,
                            starting: 0,
                            running: 0,
                            backing_off: 0,
                            closing: 0,
                            held: 0,
                            done: 0,
                            failed: 0,
                            cancelled: 0,
                        },
                        held_below: 0,
                        conversation: Some(jig_client_domain::contract::ConversationRead {
                            seen: 0,
                            unread: Count { value: 0, limited: false },
                        }),
                    }]),
                }),
                jig_client_scripted_engine::Change::Object {
                    project: 1,
                    key: ObjectKey::Task(9),
                    value: ReadValue::Task(Box::new(task)),
                },
            ],
        })
        .expect("chat rule fits");
    world
}
/// Names the party’s conversation list.
#[must_use]
pub fn conversation() -> RootAddress {
    RootAddress::Jig(Address::Conversations { project: None })
}
/// Names the initial chat’s activity page.
#[must_use]
pub fn chat() -> RootAddress {
    RootAddress::Jig(Address::Task { task: 9, section: jig_client_view::address::TaskSection::Activity })
}
/// Starts with the initial chat already durable.
#[must_use]
pub fn chat_ready(seed: u64) -> World<Test, Scripted> {
    let mut world = world(seed);
    let now = world.now();
    world
        .front()
        .engine
        .act(
            now,
            Wall::EPOCH,
            jig_client_scripted_engine::Change::Object {
                project: 1,
                key: ObjectKey::Task(9),
                value: ReadValue::Task(Box::new(task())),
            },
        )
        .expect("task fits");
    world
}

/// Project facts used by settings, goals and memory stories.
#[must_use]
pub fn project() -> OwnerProject {
    let task = task();
    let recurring = Recurring {
        chip: task.chip.clone(),
        key: 1,
        batch: list([batch_task(1, specification(text(&[b'r'; 32])), OwnerExecutor::Agent { charter: 1 }, 10)]),
        budget: 10,
        overlap: RecurringOverlap::Skip,
        current_period: 3,
        began: Some(Wall::EPOCH),
        next: Some(Wall::EPOCH),
        last_period: 2,
        pending_period: None,
        last_batch: list([8, 9]),
        cut: false,
    };
    let resource = OwnerProjectResource {
        part: OwnerPart {
            connector: jig_client_test_domain::FIRST,
            kind: OwnerPartKind::Resource,
            key: OwnerPartKey::Path(list([Box::from(b"project".as_slice())])),
        },
        role: OwnerProjectResourceRole::Owned,
    };
    let pool = |person| PoolLedger {
        source: Funder::Pool { project: 1, person, period: 3 },
        parent: None,
        numbers: OwnerNumbers { budget: 10, spent: 0, spent_below: 0, reserved: 0 },
        left: 10,
        closed: false,
        began: Some(Wall::EPOCH),
        next: None,
    };
    OwnerProject {
        number: 1,
        name: DocumentText::complete(text(&[b'p'; 32])),
        resources: list([resource.clone(), resource]),
        policy: OwnerProjectPolicy {
            templates: list([]),
            ceiling: owner_authority(),
            period_spend: 20,
            escalation_role: Some(1),
            projections: list([]),
            roles: list([
                OwnerProjectPolicyRole {
                    number: 1,
                    authority: owner_authority(),
                    period_spend: 10,
                    requests: 0,
                    decides: 3,
                },
                OwnerProjectPolicyRole {
                    number: 2,
                    authority: owner_authority(),
                    period_spend: 10,
                    requests: 0,
                    decides: 3,
                },
            ]),
            requirements: list([]),
            permissions: None,
            permissions_cut: false,
            cut: false,
        },
        members: list([
            OwnerProjectMember {
                party: 7,
                kind: PartyKind::Person,
                role: HeldRole::Policy { role: 1 },
                login: DocumentText::complete(text(b"Pat")),
                name: DocumentText::complete(text(b"Pat")),
            },
            OwnerProjectMember {
                party: 8,
                kind: PartyKind::Service,
                role: HeldRole::Policy { role: 2 },
                login: DocumentText::complete(text(&[b's'; 32])),
                name: DocumentText::complete(text(&[b's'; 32])),
            },
        ]),
        current_period: Some(pool(7)),
        pools: list([pool(7), pool(8)]),
        configured_period: 3,
        configured_pools: list([pool(7), pool(8)]),
        configured_pools_cut: false,
        spend: list([
            OwnerProjectSpend {
                chip: task.chip.clone(),
                funder: pool(7).source,
                numbers: OwnerNumbers { budget: 10, spent: 5, spent_below: 0, reserved: 0 },
            },
            OwnerProjectSpend {
                chip: task.chip,
                funder: pool(8).source,
                numbers: OwnerNumbers { budget: 10, spent: 6, spent_below: 0, reserved: 0 },
            },
        ]),
        recurring: list([recurring.clone(), Recurring { key: 2, ..recurring }]),
        cut: false,
        acts: list([Offered { effect: None, act: ActKind::Reorder, admission: Admission::Allowed }]),
    }
}
/// Bounded deployment owner facts for capacity panels.
#[must_use]
pub fn deployment() -> OwnerDeployment {
    let chip = task().chip;
    let run = |attempt| OwnerHostRun {
        task: OwnerTaskLink::Live(chip.clone()),
        attempt,
        kept: 0,
        listed: true,
        state: OwnerAttemptState::Adopted,
        location: OwnerRunLocation::Adrift {
            channel: Some(1),
            until: Some(Wall::EPOCH),
            remaining: Some(Duration::from_secs(60)),
        },
        grace_until: Some(Wall::EPOCH),
    };
    let account = |number| OwnerAccount {
        number,
        place: 0,
        uses: None,
        plan: false,
        provider: None,
        provider_name: None,
        manage_usage: None,
        display: DocumentText::complete(text(b"Fixture account")),
        holder: OwnerAccountHolder::Party(7),
        sources: list([]),
        generation: None,
        state: OwnerAccountState::Misconfigured,
        usable: false,
        expiring: false,
        attention: true,
        spent_until: None,
        runs: list([chip.clone(), chip.clone()]),
        waiting: list([chip.clone(), chip.clone()]),
        cut: false,
    };
    let host = |channel| OwnerHost {
        name: Some(Box::from(b"worker".as_slice())),
        awaiting_logins: false,
        rule: HostRule::Open,
        programs: list([]),
        channel,
        kind: OwnerHostKind::Worker,
        slots: 2,
        used: 1,
        free: 1,
        draining: false,
        connected: false,
        grace_until: Some(Wall::EPOCH),
        grace_remaining: Some(Duration::from_secs(60)),
        runs: list([run(1), run(2)]),
        cut: false,
    };
    OwnerDeployment {
        store: StoreFill { bytes: 1, mark: 1024 },
        hosts: list([host(1), host(2)]),
        attempts: list([run(1), run(2)]),
        accounts: list([account(1), account(2)]),
        tasks: OwnerCapacity { used: 1, limit: 2 },
        workers: OwnerCapacity { used: 2, limit: 4 },
        attempts_capacity: OwnerCapacity { used: 1, limit: 2 },
        waiting_attempts: 0,
        proposals: OwnerCapacity { used: 2, limit: 4 },
        projects: list([OwnerProjectCapacity { project: 1, tasks: OwnerCapacity { used: 1, limit: 2 }, proposals: 2 }]),
        grace_bound: Duration::from_secs(60),
        hello_refusals: list([]),
        hosts_forgotten: 0,
        hello_refusals_forgotten: 0,
        person_room_refusals: list([]),
        person_room_refusals_forgotten: 0,
        person_answer_retention: Duration::from_secs(1000),
        task_room_refusals: list([]),
        task_room_refusals_forgotten: 0,
        cut: false,
        connectors: list([jig_client_test_domain::FIRST, jig_client_test_domain::SECOND]),
        connectors_cut: false,
        providers: list([]),
        providers_cut: false,
        key_bytes: 256,
        readiness: list([]),
        readiness_cut: false,
    }
}

/// Immutable delegate proposal shared by inbox, plan, two-tab and incident stories.
#[must_use]
pub fn proposal() -> OwnerProposal {
    let key = ProposalKey { proposer: Proposer::Task(10), number: 1 };
    OwnerProposal {
        key,
        project: 1,
        reason: DocumentText::complete(text(b"Resolve the incident")),
        action: OwnerProposalAction::Effect {
            part: OwnerPart {
                connector: jig_client_test_domain::FIRST,
                kind: OwnerPartKind::Effect,
                key: OwnerPartKey::Number(1),
            },
            authority: owner_authority(),
            attempt: 1,
            completion: 1,
            position: 1,
            purpose: 1,
        },
        as_holder: false,
        waiting: OwnerProposalWaiting { holder: OwnerHistoryHolder::Person(7), since: Wall::EPOCH },
        funding: list([OwnerFundingOffer {
            source: Funder::Pool { project: 1, person: 7, period: 3 },
            needed: 1,
            left: Some(1),
            after: Some(0),
        }]),
        acts: list([Offered { effect: None, act: ActKind::Accept, admission: Admission::Allowed }]),
    }
}
/// The same proposal appears in its lead chat and the authoritative inbox.
#[must_use]
pub fn proposal_world(seed: u64) -> World<Test, Scripted> {
    proposal_fixture(seed, false)
}
/// A proposal in the task's plan and in the committed call that made it.
#[must_use]
pub fn arranged_proposal_world(seed: u64) -> World<Test, Scripted> {
    let mut world = proposal_world(seed);
    let mut lead = task();
    lead.proposals = list([proposal().key]);
    let context = lead.context.as_mut().expect("task fixture supplies context");
    context.impact.tasks.running = 1;
    world
        .front()
        .engine
        .act(
            skein_lib::Time::ZERO,
            Wall::EPOCH,
            jig_client_scripted_engine::Change::Object {
                project: 1,
                key: ObjectKey::Task(9),
                value: ReadValue::Task(Box::new(lead)),
            },
        )
        .expect("arranged task fits");
    world
        .front()
        .engine
        .act(
            skein_lib::Time::ZERO,
            Wall::EPOCH,
            jig_client_scripted_engine::Change::Object {
                project: 1,
                key: ObjectKey::Task(10),
                value: ReadValue::Task(Box::new(incident_delegate())),
            },
        )
        .expect("plan delegate fits");
    let tree = OwnerTree {
        root: task().chip,
        rows: list([OwnerTreeRow {
            chip: incident_delegate().chip,
            parent: 9,
            depth: 1,
            dependencies: list([]),
            proposals: list([proposal().key]),
            spent: 0,
            cut: false,
        }]),
        folded: list([]),
        held_below: 0,
        next: None,
    };
    world
        .front()
        .engine
        .act(skein_lib::Time::ZERO, Wall::EPOCH, jig_client_scripted_engine::Change::Tree(tree))
        .expect("arranged plan fits");
    accepted_proposal(&mut world, &proposal());
    world
}
fn proposal_fixture(seed: u64, delegate_chip: bool) -> World<Test, Scripted> {
    let mut world = chat_ready(seed);
    let p = proposal();
    let mut lead = task();
    lead.proposals = list([p.key]);
    if delegate_chip {
        lead.delegates = list([LiveDelegate { chip: incident_delegate().chip, takeover_budget: 1 }]);
        lead.context.as_mut().expect("supplied task context").impact.tasks.running = 1;
    }
    for change in [
        jig_client_scripted_engine::Change::Object {
            project: 1,
            key: ObjectKey::Task(9),
            value: ReadValue::Task(Box::new(lead)),
        },
        jig_client_scripted_engine::Change::Object {
            project: 1,
            key: ObjectKey::Proposal(p.key),
            value: ReadValue::OwnerProposal(Box::new(p.clone())),
        },
        jig_client_scripted_engine::Change::Source(jig_client_scripted_engine::Source {
            party: 7,
            read: Read::Inbox {
                group: jig_client_domain::contract::InboxReadGroup::Needs,
                project: None,
                kind: None,
                goal: None,
                after: None,
            },
            rows: jig_client_scripted_engine::Rows::Inbox(vec![crate::stories::inbox_entry(
                ObjectKey::Proposal(p.key),
                1,
                InboxGroup::Proposals,
                1,
                Wall::EPOCH,
                None,
            )]),
        }),
    ] {
        world.front().engine.act(skein_lib::Time::ZERO, Wall::EPOCH, change).expect("proposal facts fit");
    }
    let turn = Turn {
        messages: list([Message {
            speaker: Speaker::Assistant,
            blocks: list([Block::Call(Call {
                number: 1,
                tool: text(b"delegate"),
                input: text(b"Resolve incident"),
                outcome: Some(text(b"Delegate ten")),
                failed: false,
                objects: if delegate_chip {
                    list([ObjectKey::Task(10), ObjectKey::Proposal(p.key)])
                } else {
                    list([ObjectKey::Proposal(p.key)])
                },
                parts: list([]),
            })]),
        }]),
        used: Usage { input: 1, output: 1, cached: 0, cache_written: 0 },
        spent: 1,
        read_through: None,
        cut: false,
    };
    world
        .front()
        .engine
        .act(
            skein_lib::Time::ZERO,
            Wall::EPOCH,
            jig_client_scripted_engine::Change::Source(jig_client_scripted_engine::Source {
                party: 7,
                read: Read::Activity { task: 9, before: None },
                rows: jig_client_scripted_engine::Rows::Activity(vec![crate::stories::turn_event(
                    9,
                    1,
                    TurnName { attempt: 1, number: 1 },
                    TurnRead::Decoded(turn),
                )]),
            }),
        )
        .expect("delegation turn fits");
    accepted_proposal(&mut world, &p);
    world
}
fn accepted_proposal(world: &mut World<Test, Scripted>, p: &OwnerProposal) {
    let decided = decided_proposal(p, OwnerProposalHistoryState::Accepted { by: OwnerRequester::Person(7) });
    let ReadValue::Task(lead) =
        world.front().engine.truth().objects.get(&ObjectKey::Task(9)).expect("lead task exists")
    else {
        unreachable!()
    };
    let mut lead = lead.as_ref().clone();
    lead.activity = 2;
    let before = world
        .front()
        .engine
        .truth()
        .sources
        .iter()
        .find(|s| matches!(s.read, Read::Activity { task: 9, .. }))
        .expect("fixture truth is present")
        .clone();
    let mut after = before;
    if let jig_client_scripted_engine::Rows::Activity(rows) = &mut after.rows {
        rows.push(ActivityLine {
            source: Box::new(OwnerActivityLine {
                task: 9,
                position: 2,
                at: Wall::EPOCH,
                line: OwnerActivity::Words {
                    message: 2,
                    by: OwnerRequester::Person(7),
                    kind: OwnerMessageKind::ProposalDecision { proposal: p.key.number, accepted: true },
                    words: text(b"Accepted"),
                },
            }),
            decoded: None,
        });
    }
    let mut changes = vec![
        jig_client_scripted_engine::Change::Object {
            project: 1,
            key: ObjectKey::Proposal(p.key),
            value: ReadValue::Historical(Box::new(decided)),
        },
        jig_client_scripted_engine::Change::Object {
            project: 1,
            key: ObjectKey::Task(9),
            value: ReadValue::Task(Box::new(lead)),
        },
        jig_client_scripted_engine::Change::Source(after),
        jig_client_scripted_engine::Change::Source(jig_client_scripted_engine::Source {
            party: 7,
            read: Read::Inbox {
                group: jig_client_domain::contract::InboxReadGroup::Needs,
                project: None,
                kind: None,
                goal: None,
                after: None,
            },
            rows: jig_client_scripted_engine::Rows::Inbox(vec![]),
        }),
    ];
    if let Some(tree) = world.front().engine.truth().trees.get(&9) {
        let mut tree = tree.clone();
        for index in 0..tree.rows.len() {
            tree.rows.get_mut(index).expect("bounded tree row exists").proposals.clear();
        }
        changes.push(jig_client_scripted_engine::Change::Tree(tree));
    }
    world
        .front()
        .engine
        .script(jig_client_scripted_engine::Rule {
            party: Some(7),
            ask: Ask::Decide {
                project: 1,
                on: DecideOn::Proposal { proposal: p.key, decision: Decision::Accepted, running: None },
            },
            answer: Answer::Done(Outcome::Owner(OwnerOutcome::ProposalDecided {
                proposal: p.key,
                by: 7,
                at: Wall::EPOCH,
                choice: OwnerProposalChoice::Accepted,
                repeated: false,
            })),
            changes,
        })
        .expect("accepted activity rule fits");
}

/// A held chat with the engine's release offer and independent hold revision.
#[must_use]
pub fn held_chat(seed: u64) -> World<Test, Scripted> {
    let mut world = world(seed);
    let mut held = task();
    let reason = OwnerChipHold {
        placed: None,
        words: None,
        circle: None,
        account: None,
        source: None,
        reason: HoldCategory::Stopped,
        revision: 4,
        before: OwnerPriorPhase::Active(RunStage::Running { attempt: 1 }),
        escalation: None,
        tries: OwnerFailureTries { transient: 0, permanent: 0, run: 0, agent: 0, lost: 0, invalid: 0 },
        hold_wait_since: Some(Wall::EPOCH),
    };
    held.hold = Some(Box::new(reason.clone()));
    held.chip.phase = Phase::Held;
    held.chip.details = Box::new(owner_chip(OwnerChipPhase::Held { reason: HoldCategory::Stopped, revision: 4 }, None));
    {
        let ChipPhase { details, .. } = held.chip.details.as_mut();
        details.hold = Some(reason);
    }
    held.context.as_mut().expect("supplied task context").runtime = Runtime::Held { was: None };
    held.acts = list([Offered { effect: None, act: ActKind::Release, admission: Admission::Allowed }]);
    world
        .front()
        .engine
        .act(
            skein_lib::Time::ZERO,
            Wall::EPOCH,
            jig_client_scripted_engine::Change::Object {
                project: 1,
                key: ObjectKey::Task(9),
                value: ReadValue::Task(Box::new(held)),
            },
        )
        .expect("held task fits");
    world
}
/// Person's new-chat sequence, reused by focused and random worlds.
#[must_use]
pub fn start_chat_steps() -> Vec<skein_fake_person::Step<RootAddress>> {
    use skein_fake_person::{Find, Step};
    use skein_ui::Role;
    vec![
        Step::See { find: Find::named(Role::TextArea, b"First words"), within: Duration::from_secs(1) },
        Step::Type { find: Find::named(Role::TextArea, b"First words"), words: Box::from(*b"Hello") },
        Step::Press { find: Find::named(Role::Button, b"Start a conversation") },
    ]
}

/// A goal draft beyond the allotment, with the engine's concrete admission facts.
#[must_use]
pub fn goal_world(seed: u64) -> World<Test, Scripted> {
    let mut world = world(seed);
    let doc = project();
    let creating = Creating {
        within: CreateBounds {
            authority: Box::new(owner_authority()),
            pool_left: 1,
            procedures: list([]),
            charters: list([1]),
            latest: None,
            budget: 1,
            maximum_budget: 100,
            maximum_deadline: None,
        },
        beyond: Beyond::Proposed { to: Recipient::Role { project: 1, role: 1 } },
    };
    let mut me = frame(7);
    me.projects.get_mut(0).expect("project").creating = Some(creating);
    let ask = goals::ask(&me, OwnerExecutor::Agent { charter: 1 });
    world
        .front()
        .engine
        .act(skein_lib::Time::ZERO, Wall::EPOCH, jig_client_scripted_engine::Change::Me(me))
        .expect("creating offer fits");
    world
        .front()
        .engine
        .act(skein_lib::Time::ZERO, Wall::EPOCH, jig_client_scripted_engine::Change::OwnerProject(Box::new(doc)))
        .expect("project facts fit");
    let mut p = proposal();
    p.key.proposer = Proposer::Party(7);
    p.action = OwnerProposalAction::Batch(list([batch_task(
        1,
        specification(text(b"Resolve incident")),
        OwnerExecutor::Agent { charter: 1 },
        2,
    )]));
    world
        .front()
        .engine
        .script(jig_client_scripted_engine::Rule {
            party: Some(7),
            ask,
            answer: Answer::Done(Outcome::Owner(OwnerOutcome::Proposed { proposal: p.key.number, to: 1 })),
            changes: vec![jig_client_scripted_engine::Change::Object {
                project: 1,
                key: ObjectKey::Proposal(p.key),
                value: ReadValue::OwnerProposal(Box::new(p)),
            }],
        })
        .expect("proposed goal rule fits");
    world
}

/// A procedure-only goal form using the exact supplied pair and offered judgment.
#[must_use]
pub fn procedure_goal_world(seed: u64) -> World<Test, Scripted> {
    let mut world = world(seed);
    let procedure_frame = frame(7);

    world
        .front()
        .engine
        .act(skein_lib::Time::ZERO, Wall::EPOCH, jig_client_scripted_engine::Change::Me(procedure_frame))
        .expect("procedure-only frame fits");
    let doc = project();
    let creating = Creating {
        within: CreateBounds {
            authority: Box::new(owner_authority()),
            pool_left: 1,
            procedures: list([CreateProcedure { connector: 2, code: 7 }]),
            charters: list([]),
            latest: None,
            budget: 1,
            maximum_budget: 100,
            maximum_deadline: None,
        },
        beyond: Beyond::Proposed { to: Recipient::Role { project: 1, role: 1 } },
    };
    let mut me = frame(7);
    me.projects.get_mut(0).expect("project").creating = Some(creating);
    let ask = goals::ask(&me, OwnerExecutor::Procedure { connector: 2, code: 7 });
    world
        .front()
        .engine
        .act(skein_lib::Time::ZERO, Wall::EPOCH, jig_client_scripted_engine::Change::Me(me))
        .expect("creating offer fits");
    world
        .front()
        .engine
        .act(skein_lib::Time::ZERO, Wall::EPOCH, jig_client_scripted_engine::Change::OwnerProject(Box::new(doc)))
        .expect("project facts fit");
    let mut p = proposal();
    p.key = ProposalKey { proposer: Proposer::Party(7), number: 1 };
    p.waiting = OwnerProposalWaiting {
        holder: OwnerHistoryHolder::Policy { project: 1, kind: OwnerMessageProposalKind::Batch },
        since: Wall::EPOCH,
    };
    p.funding = list([OwnerFundingOffer {
        source: Funder::Pool { project: 1, person: 7, period: 3 },
        needed: 2,
        left: Some(2),
        after: Some(0),
    }]);
    p.action = OwnerProposalAction::Batch(list([batch_task(
        1,
        specification(text(b"Resolve incident")),
        OwnerExecutor::Procedure { connector: 2, code: 7 },
        2,
    )]));
    world
        .front()
        .engine
        .script(jig_client_scripted_engine::Rule {
            party: Some(7),
            ask,
            answer: Answer::Done(Outcome::Owner(OwnerOutcome::Proposed { proposal: p.key.number, to: 1 })),
            changes: vec![jig_client_scripted_engine::Change::Object {
                project: 1,
                key: ObjectKey::Proposal(p.key),
                value: ReadValue::OwnerProposal(Box::new(p)),
            }],
        })
        .expect("proposed goal rule fits");
    world
}

fn incident_delegate() -> Task {
    let mut delegate = task();
    delegate.chip.task = 10;
    delegate.chip.title = DocumentText::complete(text(b"Delegate ten"));
    delegate.chip.executor = Executor::Procedure { connector: jig_client_test_domain::FIRST, kind: 1 };
    delegate.chip.details = Box::new(owner_chip(
        OwnerChipPhase::Active(RunStage::Running { attempt: 1 }),
        Some(OwnerPart {
            connector: jig_client_test_domain::FIRST,
            kind: OwnerPartKind::Procedure,
            key: OwnerPartKey::Number(10),
        }),
    ));
    delegate.requester = OwnerRequester::Task(9);
    delegate.proposals = list([proposal().key]);
    delegate
}
/// Lead task, delegate and connector parts behind the incident journey.
#[must_use]
pub fn incident_world(seed: u64) -> World<Test, Scripted> {
    incident_world_with_second(seed, None)
}
/// An incident naming the plans of its lead task and a second task.
#[must_use]
pub fn two_plans_world(seed: u64) -> World<Test, Scripted> {
    incident_world_with_second(seed, Some(10))
}
fn incident_world_with_second(seed: u64, second: Option<u64>) -> World<Test, Scripted> {
    use jig_client_test_domain::family as f;
    let mut world = proposal_fixture(seed, true);
    let delegate = incident_delegate();
    let mut lead = task();
    lead.delegates = list([LiveDelegate { chip: delegate.chip.clone(), takeover_budget: 1 }]);
    lead.context.as_mut().expect("supplied task context").impact.tasks.running = 1;
    for change in [
        jig_client_scripted_engine::Change::Object {
            project: 1,
            key: ObjectKey::Task(9),
            value: ReadValue::Task(Box::new(lead.clone())),
        },
        jig_client_scripted_engine::Change::Object {
            project: 1,
            key: ObjectKey::Task(10),
            value: ReadValue::Task(Box::new(delegate.clone())),
        },
        jig_client_scripted_engine::Change::Tree(OwnerTree {
            root: lead.chip,
            rows: list([OwnerTreeRow {
                chip: delegate.chip,
                parent: 9,
                depth: 1,
                dependencies: list([]),
                proposals: list([proposal().key]),
                spent: 0,
                cut: false,
            }]),
            folded: list([]),
            held_below: 0,
            next: None,
        }),
    ] {
        world.front().engine.act(skein_lib::Time::ZERO, Wall::EPOCH, change).expect("incident task facts fit");
    }
    world.front().compositions.insert(
        jig_client_test_domain::first::Address { project: 1, item: 1 },
        f::PageComposition { task: 9, second, proposals: list([]) },
    );
    let mut family = world.front().engine.application().clone();
    let item = f::Subject { scope: f::Scope::Project(1), kind: f::Kind::Item(1) };
    family.documents.insert(
        item,
        f::Document::Connector(jig_client_test_domain::connector_values::ConnectorDocument {
            subject: jig_client_test_domain::family::Subject {
                scope: jig_client_test_domain::family::Scope::Project(1),
                kind: jig_client_test_domain::family::Kind::Item(1),
            },
            value: Box::new(jig_client_test_domain::connector_values::ClientDocument::Object {
                object: 1,
                now: jig_client_test_domain::connector_values::ApplicationObject { revision: 1, value: 1 },
            }),
        }),
    );
    let subject = f::Subject { scope: f::Scope::Project(1), kind: f::Kind::Panel(10) };
    family.documents.insert(
        subject,
        f::Document::Connector(jig_client_test_domain::connector_values::ConnectorDocument {
            subject,
            value: Box::new(jig_client_test_domain::connector_values::ClientDocument::Panel {
                task: 10,
                procedure: None,
                result: Some(31),
                resources: list([]),
                effects: list([]),
            }),
        }),
    );
    world
        .front()
        .engine
        .act_application(skein_lib::Time::ZERO, Wall::EPOCH, jig_client_test_domain::FIRST, 1, family, vec![])
        .expect("incident family fits");
    world
}
/// The application's family incident address.
#[must_use]
pub fn incident() -> RootAddress {
    RootAddress::First(jig_client_test_domain::first::Address { project: 1, item: 1 })
}
/// Complete a scripted task with its whole final report.
#[must_use]
pub fn reported(mut task: Task, report: &[u8]) -> Task {
    task.chip.phase = Phase::Done;
    task.chip.details = Box::new(owner_chip(OwnerChipPhase::Ended, None));
    {
        let ChipPhase { details, .. } = task.chip.details.as_mut();
        details.ending = Some(OwnerChipEnding::Done(ResultValue::Report(DocumentText::complete(text(report)))));
        details.ended_at = Some(Wall::EPOCH);
    }
    task.context.as_mut().expect("supplied task context").runtime = Runtime::Ended;
    task.acts = list([]);
    task.proposals = list([]);
    task.result = Some(jig_client_domain::contract::TaskResult::Document(Box::new(Result {
        task: task.chip.clone(),
        ending: Ending::Done,
        value: Some(ResultValue::Report(DocumentText::complete(text(report)))),
        at: Some(Wall::EPOCH),
        acts: list([]),

        details: None,
    })));
    task
}

fn specification(words: jig_client_domain::contract::Text) -> jig_client_domain::contract::SpecificationDocument {
    jig_client_domain::contract::SpecificationDocument::complete(jig_client_domain::contract::Specification {
        asks: false,
        words,
        parameters: skein_lib::List::with_capacity(0),
        inputs: skein_lib::List::with_capacity(0),
    })
}

#[must_use]
pub fn party(number: u64) -> Party {
    Party { number, kind: PartyKind::Person, name: text(b"Pat") }
}

pub mod unread_turn;

use jig_client::contract::DecideOn;

pub(crate) fn inbox_entry(
    object: ObjectKey,
    project: u32,
    group: InboxGroup,
    position: u64,
    at: Wall,
    reply: Option<Box<TaskLine>>,
) -> OwnerInboxEntry {
    let (task, object) = match object {
        ObjectKey::Question { task, message } => (task, OwnerInboxObject::Question { message }),
        ObjectKey::Proposal(key) => (
            match key.proposer {
                Proposer::Task(n) | Proposer::Party(n) => n,
            },
            OwnerInboxObject::Proposal { proposer: key.proposer, number: key.number },
        ),
        ObjectKey::Escalation(task) => (task, OwnerInboxObject::Escalation { revision: 1 }),
        ObjectKey::PersonTask(task) => (task, OwnerInboxObject::PersonTask),
        ObjectKey::Task(task) => (task, OwnerInboxObject::Reply { message: position }),
        ObjectKey::Result(task) => (task, OwnerInboxObject::Result { position }),
        ObjectKey::Note { .. } => unreachable!("notes have no inbox row"),
    };
    let row = OwnerInboxEntry {
        reply,
        task: OwnerTaskLink::Stored { number: task, status: None },
        project: Some(project),
        object,
        activity: None,
        at: Some(at),
        unread: None,
        cursor: OwnerInboxCursor::Number { number: position },
    };
    assert_eq!(row.group(), group);
    row
}

fn owner_chip(phase: OwnerChipPhase, procedure: Option<OwnerPart>) -> ChipPhase {
    ChipPhase {
        phase,
        details: Box::new(OwnerChipDetails {
            dependencies: list([]),
            project: 1,
            executor: OwnerExecutor::Agent { charter: 1 },
            phase_since: Wall::EPOCH,
            run_since: None,
            attempt: Some(1),
            retry: None,
            wake: OwnerWakePolicy {
                words: WakeRule::Never,
                notices: WakeRule::Never,
                news: WakeRule::Never,
                results: ResultsWake::Never,
                questions: false,
                answers: false,
                timers: false,
            },
            person_wait: None,
            hold: None,
            budget: OwnerBudget {
                numbers: OwnerNumbers { budget: 1, spent: 0, spent_below: 0, reserved: 0 },
                left: 1,
                funder: Funder::Pool { project: 1, person: 7, period: 3 },
                deadline: None,
            },
            activity: 0,
            resources: list([]),
            resources_cut: false,
            resources_observed: false,
            procedure,
            live_delegates: 0,
            closing_connectors: None,
            ending: None,
            ended_at: None,
            observation: None,
        }),
    }
}

/// Retained proposal facts supplied after a decision, independently of the pending card.
#[must_use]
pub fn decided_proposal(p: &OwnerProposal, state: OwnerProposalHistoryState) -> OwnerHistorical {
    let action = match &p.action {
        OwnerProposalAction::Effect { part, authority, attempt, completion, position, purpose } => {
            OwnerRetainedAction::Effect {
                part: part.clone(),
                authority: authority.clone(),
                attempt: *attempt,
                completion: *completion,
                position: *position,
                purpose: *purpose,
            }
        }
        OwnerProposalAction::Batch(tasks) => OwnerRetainedAction::Batch(tasks.clone()),
        OwnerProposalAction::Settings(_)
        | OwnerProposalAction::ProcedureEffect { .. }
        | OwnerProposalAction::Amend { .. }
        | OwnerProposalAction::Widen { .. }
        | OwnerProposalAction::Release { .. } => unreachable!("story supplies an effect or batch"),
    };
    OwnerHistorical {
        key: ObjectKey::Proposal(p.key),
        value: OwnerHistoricalValue::Proposal(OwnerProposalHistory {
            action,
            number: p.key.number,
            proposer: match p.key.proposer {
                Proposer::Task(n) | Proposer::Party(n) => n,
            },
            as_holder: p.as_holder,
            state,
        }),
        at: Some(Wall::EPOCH),
        by: Some(OwnerRequester::Person(7)),
    }
}
/// A committed turn and its decoded archive have the owner's independent identities.
#[must_use]
pub fn turn_event(task: u64, position: u64, name: TurnName, turn: TurnRead) -> ActivityLine {
    ActivityLine {
        source: Box::new(OwnerActivityLine {
            task,
            position,
            at: Wall::EPOCH,
            line: OwnerActivity::Turn { reply: position, attempt: name.attempt, turn: name.number, read: None },
        }),
        decoded: Some(ArchivedTurn {
            task,
            name,
            cumulative: match &turn {
                TurnRead::Decoded(value) => value.spent,
                TurnRead::Undecodable { .. } => 0,
            },
            read: None,
            at: Wall::EPOCH,
            turn,
        }),
    }
}
