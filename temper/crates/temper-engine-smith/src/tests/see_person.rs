//! Hand-made person documents read as lines (domain/agent/ask-temper.md, 1).
use jig_domain_core::{
    documents as d,
    reads::{Answer, EndedGroup, TaskPage},
};
use jig_domain_tasks as tasks;
use skein_lib::{List, Wall};
use temper_engine_domain::{SeeAnswer, SeeName, SeeNames, SeePerson, SeeRead, SeeWords};

fn text(words: &[u8]) -> d::Text {
    d::Text { bytes: words.into(), cut: false }
}
fn budget() -> d::BudgetLine {
    d::BudgetLine {
        numbers: tasks::Numbers { budget: 1250, spent: 123, spent_below: 101, reserved: 300 },
        left: 726,
        funder: tasks::Funder::Task(1),
        deadline: None,
    }
}
fn chip() -> d::Chip {
    d::Chip {
        number: 41,
        title: text(b"Add CSV export"),
        executor: d::ExecutorKind::Agent,
        phase: d::PhaseLine::Active(tasks::Active::Running { attempt: 1 }),
        details: Box::new(d::ChipDetails {
            dependencies: List::uncapped(),
            project: 7,
            executor: tasks::Executor::Agent { charter: 2 },
            phase_since: Wall::EPOCH,
            run_since: None,
            attempt: None,
            retry: None,
            wake: tasks::WakePolicy::DEFAULT,
            person_wait: None,
            hold: None,
            budget: budget(),
            activity: 1,
            resources: List::uncapped(),
            resources_cut: false,
            resources_observed: false,
            procedure: None,
            live_delegates: 0,
            closing_connectors: None,
            ending: None,
            ended_at: None,
            observation: None,
        }),
    }
}
fn row() -> d::TaskLine {
    d::TaskLine {
        chip: chip(),
        project: 7,
        requester: tasks::Party::Person(9),
        tracked: None,
        label: 1,
        flags: tasks::Flags::PLAIN,
        changed: Wall::EPOCH,
        last_activity: Wall::from_nanos(1_794_210_000_000_000_000),
        ended_at: None,
        budget: budget(),
        waits_on: d::WaitsOn::Nothing,
        phases: d::PhaseCounts::ZERO,
        delegates_done: 1,
        delegates_total: 3,
        held_below: 0,
        conversation: None,
    }
}
fn names() -> SeeNames {
    SeeNames {
        project: SeeName { number: 7, name: b"Shop".as_slice().into() },
        people: Box::new([SeeName { number: 9, name: b"Ada".as_slice().into() }]),
        charters: Box::new([SeeName { number: 2, name: b"the producer's".as_slice().into() }]),
    }
}
fn tasks_page(rows: &[d::TaskLine]) -> Answer {
    let mut live = List::uncapped();
    for row in rows {
        live.push(row.clone()).unwrap();
    }
    Answer::Tasks(Box::new(TaskPage { live, live_next: None, ended: EndedGroup::None }))
}

fn list_row(value: d::TaskLine, ended: bool, more: bool) -> Answer {
    let mut rows = List::uncapped();
    rows.push(value).unwrap();
    let next = more.then_some(continuation());
    Answer::Tasks(Box::new(if ended {
        TaskPage { live: List::uncapped(), live_next: None, ended: EndedGroup::Rows { rows, next, cut: false } }
    } else {
        TaskPage { live: rows, live_next: next, ended: EndedGroup::None }
    }))
}
fn continuation() -> jig_domain_core::reads::Cursor {
    jig_domain_core::reads::Cursor::Store { scope: jig_domain_core::reads::Scope::EndedTasks, after: None }
}
fn authority() -> d::AuthorityLine {
    d::AuthorityLine {
        models: tasks::Models::NONE,
        tools: tasks::Tools(0),
        grants: List::uncapped(),
        executors: List::uncapped(),
        tasks: 0,
        depth: 0,
        ceiling: tasks::Budget { spend: 0, deadline: None },
        notes: tasks::Scopes(0),
        note_resources: List::uncapped(),
    }
}
fn ledger(source: tasks::Funder) -> d::Ledger {
    d::Ledger {
        source,
        parent: None,
        numbers: budget().numbers,
        left: budget().left,
        closed: false,
        began: Some(Wall::EPOCH),
        next: Some(Wall::from_nanos(604_800_000_000_000)),
    }
}
fn project() -> Answer {
    let mut members = List::uncapped();
    for (party, role) in [(9, jig_domain_people::Role::Owner), (88, jig_domain_people::Role::Member)] {
        members
            .push(d::Member {
                party,
                kind: jig_domain_people::Kind::Person,
                role,
                login: text(b"unused-login"),
                name: text(b"unused-name"),
            })
            .unwrap();
    }
    let mut pools = List::uncapped();
    pools.push(ledger(tasks::Funder::Pool { project: 7, person: 9, period: 3 })).unwrap();
    Answer::Document(Box::new(d::Document::Project(Box::new(d::Project {
        acts: List::uncapped(),
        number: 7,
        name: text(b"Shop"),
        resources: List::uncapped(),
        policy: d::Policy {
            ceiling: authority(),
            period_spend: 1250,
            escalation_role: None,
            projections: List::uncapped(),
            roles: List::uncapped(),
            requirements: List::uncapped(),
            permissions: None,
            permissions_cut: false,
            templates: List::uncapped(),
            cut: false,
        },
        members,
        current_period: Some(ledger(tasks::Funder::Period { project: 7, period: 3 })),
        pools,
        configured_period: 3,
        configured_pools: List::uncapped(),
        configured_pools_cut: false,
        spend: List::uncapped(),
        recurring: List::uncapped(),
        cut: false,
    }))))
}
fn needs(more: bool) -> Answer {
    let mut rows = List::uncapped();
    for object in [
        d::InboxObject::Question { message: 4 },
        d::InboxObject::Proposal { proposer: jig_domain_people::Proposer::Task(41), number: 1 },
        d::InboxObject::Proposal { proposer: jig_domain_people::Proposer::Task(41), number: 2 },
    ] {
        rows.push(jig_domain_core::reads::PageRow::Inbox(d::InboxEntry {
            reply: None,
            task: d::TaskLink::Live(chip()),
            project: Some(7),
            object,
            activity: None,
            at: Some(Wall::EPOCH),
            unread: None,
            cursor: d::Cursor::Inbox { task: 41, object, at: Some(Wall::EPOCH) },
            projects: List::uncapped(),
        }))
        .unwrap();
    }
    page(rows, more)
}

fn question_needs() -> Answer {
    let mut rows = List::uncapped();
    for (task, message) in [(41, 4), (42, 4), (41, 5)] {
        let mut chip = chip();
        chip.number = task;
        let object = d::InboxObject::Question { message };
        rows.push(jig_domain_core::reads::PageRow::Inbox(d::InboxEntry {
            reply: None,
            task: d::TaskLink::Live(chip),
            project: Some(7),
            object,
            activity: None,
            at: Some(Wall::EPOCH),
            unread: None,
            cursor: d::Cursor::Inbox { task, object, at: Some(Wall::EPOCH) },
            projects: List::uncapped(),
        }))
        .unwrap();
    }
    page(rows, false)
}

fn question_words(words: &[u8]) -> Box<[SeeWords]> {
    Box::new([
        SeeWords {
            task: 41,
            message: 4,
            words: Answer::Document(Box::new(d::Document::Question(d::Question {
                acts: List::uncapped(),
                task: chip(),
                message: 4,
                from: tasks::Party::Task(41),
                at: Wall::EPOCH,
                words: text(words),
            }))),
        },
        SeeWords { task: 42, message: 4, words: Answer::None(jig_domain_core::reads::Why::Unknown) },
        SeeWords { task: 41, message: 5, words: Answer::None(jig_domain_core::reads::Why::Busy) },
    ])
}
fn page(rows: List<jig_domain_core::reads::PageRow>, more: bool) -> Answer {
    Answer::Page(Box::new(jig_domain_core::reads::Page {
        rows,
        before: None,
        next: more.then_some(continuation()),
        cut: false,
        inbox_selection: None,
    }))
}
fn goal_row(ended: bool) -> d::TaskLine {
    let mut value = row();
    value.chip.number = 12;
    value.label = 2;
    value.tracked = Some(1);
    if ended {
        value.chip.phase = d::PhaseLine::Ended;
        value.chip.details.ending = Some(d::Ending::Done(d::ResultValue::Report(text(b"Export landed."))));
    } else {
        value.waits_on = d::WaitsOn::Party {
            whom: jig_domain_people::Whom::Person(9),
            entries: 1,
            task: 12,
            kind: jig_domain_people::EntryKind::Question { message: 4 },
        };
    }
    value
}
fn reports() -> Answer {
    let mut rows = List::uncapped();
    for (number, description, body, changed) in [
        (1, b"work: scratch".as_slice(), b"not a report".as_slice(), 0),
        (2, b"report: An older headline.", b"older body", 1),
        (3, b"report: The export lands this week.", b"The first paragraph.\n\nLeft: Choose a delimiter.", 2),
    ] {
        rows.push(jig_domain_core::reads::PageRow::Note(d::Note {
            acts: List::uncapped(),
            number,
            scope: d::NoteLocation::Goal { project: 7, task: 12 },
            description: text(description),
            body: text(body),
            writer: jig_domain_notes::Author::Task { task: 12, attempt: 1 },
            tasks: List::uncapped(),
            changed: Some(Wall::from_nanos(changed)),
            revision: 1,
            cut: false,
        }))
        .unwrap();
    }
    page(rows, false)
}
fn task() -> d::Task {
    d::Task {
        talk: None,
        session: None,
        settings: None,
        waits_on: d::WaitsOn::Nothing,
        label: 2,
        flags: tasks::Flags::PLAIN,
        last_activity: row().last_activity,
        made: Wall::EPOCH,
        chip: goal_row(false).chip,
        acts: List::uncapped(),
        project: 7,
        goal: Some(12),
        priority: Some(1),
        executor: tasks::Executor::Agent { charter: 2 },
        requester: tasks::Party::Person(9),
        lineage: List::uncapped(),
        delegates: List::uncapped(),
        changed: Wall::EPOCH,
        spec_revision: 1,
        person_revision: None,
        taken_by: None,
        activity: 3,
        spec: d::SpecLine {
            words: text(b"Add CSV export"),
            parameters: List::uncapped(),
            inputs: List::uncapped(),
            cut: false,
            asks: false,
        },
        words: text(b"Add CSV export\nsecret metadata stays off the summary"),
        parameters: List::uncapped(),
        instructions: None,
        llm: Some(tasks::LlmChoice {
            account: None,
            main: tasks::ModelEffort { model: 1, effort: None },
            subagents: Box::new([]),
            later: None,
        }),
        kept_account: None,
        run_wait: None,
        resources: List::uncapped(),
        inputs: List::uncapped(),
        dependencies: List::uncapped(),
        dependency_facts: List::uncapped(),
        references: List::uncapped(),
        subscriptions: List::uncapped(),
        wake: tasks::WakePolicy::DEFAULT,
        contract: d::ContractLine::Report { words: 1024 },
        result: None,
        budget: budget(),
        authority: authority(),
        hold: None,
        proposals: List::uncapped(),
        cut: false,
    }
}
fn task_answer(value: d::Task) -> Answer {
    Answer::Document(Box::new(d::Document::Task(Box::new(value))))
}
fn held_row() -> d::TaskLine {
    let mut value = row();
    value.chip.phase = d::PhaseLine::Held { reason: tasks::Hold::Budget, revision: 3 };
    value
}
fn ended_task() -> Answer {
    let mut value = task();
    value.chip.phase = d::PhaseLine::Ended;
    value.contract = d::ContractLine::Change { connector: temper_client_protocol_wire::FORGE, kind: 1, words: 1024 };
    value.label = 0;
    let mut parent = chip();
    parent.number = 6;
    parent.title = text(b"Make export available");
    value.lineage.push(parent).unwrap();
    value.requester = tasks::Party::Task(6);
    let ending = d::Ending::Done(d::ResultValue::Change {
        kind: 1,
        part: tasks::Part {
            connector: temper_client_protocol_wire::FORGE,
            kind: tasks::PartKind::Resource,
            key: tasks::PartKey::Number(1),
        },
        words: text(b"Export landed.\nMore retained words."),
    });
    value.chip.details.ending = Some(ending.clone());
    let result = d::ResultDoc {
        acts: List::uncapped(),
        task: value.chip.clone(),
        contract: value.contract.clone(),
        ending,
        at: Some(Wall::EPOCH),
        requester: value.requester,
        executor: value.executor,
        chosen_by: None,
        followups: None,
        delegates: List::uncapped(),
        proposal: None,
        cut: false,
    };
    Answer::Ended(Box::new(jig_domain_core::reads::Ended {
        value: jig_domain_core::reads::EndedValue::Task { document: Box::new(value), result: Box::new(result) },
        at: Some(Wall::EPOCH),
        by: None,
    }))
}
fn catalogue() -> [jig_domain_core::settings::CatalogueModel; 1] {
    [jig_domain_core::settings::CatalogueModel {
        number: 2,
        model: jig_domain_core::Model { name: b"gpt-6".as_slice().into(), ..jig_domain_core::Model::hole() },
        shown: None,
        description: None,
        picker: true,
        withdrawn: false,
        source: jig_domain_core::settings::ModelSource::Stock,
        estimated: false,
        first_read: None,
    }]
}

// One minute-resolution time, shared with the existing task row.
const AT: Wall = Wall::from_nanos(1_794_210_000_000_000_000);
const OVERSIZE: u64 = 8_388_609; // One byte beyond the client's carried-turn ceiling.
const HASH: [u8; 32] = [0xab; 32];

fn turn_record(input: &[u8]) -> jig_domain_core::TurnRecord {
    turn_record_with(
        input,
        b"## Task\nRead the charter.\nBrief continuation stays in the archive.\n## Project\nShop\nMore project context",
        b"propose_settings",
        b"{ \"more\": false }",
        b"proposed, waiting for an owner\nProposal details",
    )
}
fn remembering_record() -> jig_domain_core::TurnRecord {
    turn_record_with(
        b"{ \"address\": \"settings charter 4\" }",
        b"## Kept about this person\n\n1. Prefers short answers.\n2. Uses a screen reader.\n\n## Task\nRead the charter.",
        b"remember",
        br#"{"action":"add","text":"Uses a screen reader."}"#,
        b"remember: line 2 kept; 2 of 12 lines.\n1. Prefers short answers.\n2. Uses a screen reader.",
    )
}
fn turn_record_with(
    input: &[u8],
    system: &[u8],
    second_tool: &[u8],
    second_input: &[u8],
    second_result: &[u8],
) -> jig_domain_core::TurnRecord {
    use smith_transcript::v7 as s;
    const BLOCKS: u32 = 5; // Words and two call/result pairs.
    let bounds = &s::CEILINGS;
    let mut blocks = List::with_capacity(BLOCKS);
    blocks
        .push(s::Block::Text(
            s::Text::new(
                bounds,
                s::TextParts {
                    text: b"I will read the charter.\nThen propose the settings.".as_slice().into(),
                    replay: None,
                },
            )
            .unwrap(),
        ))
        .unwrap();
    for (id, tool, input, result) in [
        (b"first".as_slice(), b"see".as_slice(), input, b"charter 4, the reviewer's\nLong settings text".as_slice()),
        (b"second", second_tool, second_input, second_result),
    ] {
        blocks
            .push(s::Block::Call(
                s::Call::new(
                    bounds,
                    s::CallParts { id: id.into(), name: tool.into(), input: input.into(), replay: None },
                )
                .unwrap(),
            ))
            .unwrap();
        blocks
            .push(s::Block::Result(
                s::ToolResult::new(
                    bounds,
                    s::ToolResultParts {
                        id: id.into(),
                        returned: s::Returned::Said(
                            s::Said::new(bounds, s::SaidParts { text: result.into(), error: false }).unwrap(),
                        ),
                    },
                )
                .unwrap(),
            ))
            .unwrap();
    }
    let source = s::Turn::new(
        bounds,
        s::TurnParts {
            system: Some(system.into()),
            model: b"gpt-6".as_slice().into(),
            effort: None,
            kind: s::TurnKind::Ordinary,
            opening: None,
            endpoint: b"unused-endpoint".as_slice().into(),
            dialect: 1,
            place: 2,
            usage: s::Usage::new(
                bounds,
                s::UsageParts { input: None, output: None, cache_read: None, cache_write: None, reasoning: None },
            )
            .unwrap(),
            spent: 12,
            jobs: 0,
            messages: List::from_boxed(Box::new([s::Message::new(
                bounds,
                s::MessageParts { role: s::Role::Assistant, blocks },
            )
            .unwrap()])),
        },
    )
    .unwrap();
    let mut out = skein_lib::Writer::new(usize::try_from(source.measure()).unwrap());
    source.encode(&mut out).unwrap();
    jig_domain_core::TurnRecord {
        opens_window: false,
        task: 41,
        attempt: 3,
        turn: 2,
        spent: 224,
        read: None,
        at: AT,
        transcript: out.finish(),
    }
}
fn whole_turn(input: &[u8]) -> Answer {
    turn_answer(turn_record(input), false)
}
fn turn_answer(record: jig_domain_core::TurnRecord, withheld: bool) -> Answer {
    Answer::Ended(Box::new(jig_domain_core::reads::Ended {
        value: if withheld {
            jig_domain_core::reads::EndedValue::WithheldTurn(record)
        } else {
            jig_domain_core::reads::EndedValue::Turn(record)
        },
        at: Some(AT),
        by: None,
    }))
}
fn turn_activity(record: jig_domain_core::TurnRecord, withheld: bool) -> Answer {
    use jig_domain_core::reads::{PageRow, PageTurn};
    page(
        List::from_boxed(Box::new([PageRow::Activity {
            line: tasks::ActivityLine {
                task: 41,
                position: 3,
                at: AT,
                line: tasks::Line::Turn { reply: 1, attempt: 3, turn: 2, read: None },
            },
            turn: Some(if withheld { PageTurn::Withheld(record) } else { PageTurn::Carried(record) }),
        }])),
        false,
    )
}
fn activity() -> Answer {
    use jig_domain_core::reads::{PageRow, PageTurn};
    let turn = turn_record(b"{ \"address\": \"settings charter 4\" }");
    let rows = List::from_boxed(Box::new([
        PageRow::Activity {
            line: tasks::ActivityLine {
                task: 41,
                position: 5,
                at: AT,
                line: tasks::Line::RunEnded { attempt: 3, end: tasks::EndKind::Finished, spent: 224 },
            },
            turn: None,
        },
        PageRow::Activity {
            line: tasks::ActivityLine {
                task: 41,
                position: 4,
                at: AT,
                line: tasks::Line::Turn { reply: 1, attempt: 3, turn: 3, read: None },
            },
            turn: Some(PageTurn::TooLarge { spent: 224, read: None, at: AT, bytes: OVERSIZE }),
        },
        PageRow::Activity {
            line: tasks::ActivityLine {
                task: 41,
                position: 3,
                at: AT,
                line: tasks::Line::Turn { reply: 1, attempt: 3, turn: 2, read: None },
            },
            turn: Some(PageTurn::Carried(turn)),
        },
        PageRow::Activity {
            line: tasks::ActivityLine {
                task: 41,
                position: 2,
                at: AT,
                line: tasks::Line::Words {
                    message: 1,
                    by: tasks::Party::Person(9),
                    kind: tasks::MessageKind::Words,
                    words: b"Please check.\nMore request words".as_slice().into(),
                },
            },
            turn: None,
        },
        PageRow::Activity {
            line: tasks::ActivityLine {
                task: 41,
                position: 1,
                at: AT,
                line: tasks::Line::RunBegan { attempt: 3, why: tasks::Why::Made, resumed: false },
            },
            turn: None,
        },
    ]));
    page(rows, true)
}
fn panel(pull: Option<u64>) -> Box<temper_engine_domain_forge::Document> {
    use temper_engine_domain_forge as f;
    use temper_engine_domain_forge_change as c;
    Box::new(f::Document::Change(f::Panel {
        way: c::Way::Queue,
        in_landing: None,
        task: 41,
        forge: 1,
        owner: b"shop".as_slice().into(),
        name: b"app".as_slice().into(),
        branch: b"export".as_slice().into(),
        landing: b"main".as_slice().into(),
        head: Some(HASH),
        pull,
        step: f::PanelStep::Queued { place: 1, ahead: Box::new([]) },
        hold: Some(c::Hold::Repairs),
        runs: None,
        ci: c::Status::Passed,
        ci_required: true,
        gates: Box::new([
            f::PanelGate { number: 4, kind: c::GateKind::Check, blocking: true, verdict: c::Status::Passed },
            f::PanelGate { number: 5, kind: c::GateKind::Agent, blocking: true, verdict: c::Status::Failed },
        ]),
        approvals: Box::new([f::PanelApproval {
            scope: 7,
            rule: 1,
            role: 2,
            freshness: f::JudgeFreshness::Exact,
            required: 2,
            read: f::ReviewRead::Complete,
            counted: Box::new([f::Reviewer {
                at: Wall::from_nanos(0),
                person: 9,
                role: 2,
                head: HASH,
                verdict: skein_forge::api::Verdict::Approve,
            }]),
            remaining: Some(1),
            refused: false,
        }]),
        unmapped_reviews: Box::new([]),
        repairs: f::PanelUsed { used: 1, limit: 2 },
        resolutions: f::PanelUsed { used: 0, limit: 2 },
        updates: f::PanelUsed { used: 0, limit: 2 },
        questions: f::PanelQuestions {
            route: c::Route::Up,
            taken: f::PanelUsed { used: 0, limit: 2 },
            open: Box::new([f::PanelQuestion {
                asker: 41,
                number: 3,
                line: b"Use commas?".as_slice().into(),
                stands: f::PanelStands::Asked,
            }]),
        },
    }))
}
fn changes() -> Box<temper_engine_domain_forge::Document> {
    use temper_engine_domain_forge as f;
    Box::new(f::Document::Changes(f::ChangesView {
        landed_default: Box::new([]),
        landed_default_more: 0,
        project: 7,
        landing: b"main".as_slice().into(),
        more: 0,
        landed_more: 0,
        queues: Box::new([
            f::ChangesQueue {
                way: temper_engine_domain_forge_change::Way::Queue,
                in_flight: None,
                repository: b"shop/app".as_slice().into(),
                branch: b"main".as_slice().into(),
                red: false,
                changes: Box::new([
                    f::ChangesQueued {
                        task: 42,
                        title: b"Add export".as_slice().into(),
                        ready_since: AT,
                        alone: false,
                        checks_since: Some(AT),
                    },
                    f::ChangesQueued {
                        task: 41,
                        title: b"Add import".as_slice().into(),
                        ready_since: AT,
                        alone: false,
                        checks_since: None,
                    },
                ]),
            },
            f::ChangesQueue {
                way: temper_engine_domain_forge_change::Way::Queue,
                in_flight: None,
                repository: b"shop/app".as_slice().into(),
                branch: b"stable".as_slice().into(),
                red: false,
                changes: Box::new([f::ChangesQueued {
                    task: 43,
                    title: b"Fix quoting".as_slice().into(),
                    ready_since: AT,
                    alone: false,
                    checks_since: Some(AT),
                }]),
            },
        ]),
        landed: Box::new([f::ChangesLanded {
            signed: Box::new([]),
            commit: [0xab; 32],
            default_branch: true,
            task: 40,
            title: b"Keep delimiters".as_slice().into(),
            at: AT,
        }]),
    }))
}
fn landings() -> Box<temper_engine_domain_forge::Document> {
    use temper_engine_domain_forge as f;
    Box::new(f::Document::Landings(f::LandingsView {
        project: 7,
        turn: Some(50),
        landings: Box::new([f::LandingLine {
            train: None,
            task: 50,
            phase: temper_engine_domain_forge_landing::Phase::Waiting,
            changes: Box::new([44, 45]),
            repositories: Box::new([]),
            landed: Box::new([HASH]),
        }]),
    }))
}
#[test]
#[expect(clippy::disallowed_methods, reason = "the test inspects rendered UTF-8 text")]
#[expect(clippy::too_many_lines, reason = "each address and its boundary cases share one answer contract table")]
fn person_answers_read_as_named_bounded_lines() {
    const TITLE_BYTES: usize = 160;
    const TASKS_NEXT: &str = "task N · goals · inbox";
    const PROJECT_NEXT: &str = "goals · tasks · changes · inbox · settings";
    const GOALS_NEXT: &str = "task N · task N tree · goals ended";
    const TASK_NEXT: &str = "task N tree · task N activity · change N · tasks";
    const TREE_NEXT: &str = "task N · task N activity · task N tree";
    const ME_NEXT: &str = "inbox · project · goals · tasks";
    let mut unnamed = row();
    unnamed.chip.details.executor = tasks::Executor::Person(tasks::PersonAddress::Person(88));
    unnamed.label = 15;
    let mut live_delegate = row();
    live_delegate.chip.details.executor =
        tasks::Executor::Procedure { connector: temper_client_protocol_wire::FORGE, code: 2 };
    let mut ended_delegate = row();
    ended_delegate.chip.phase = d::PhaseLine::Ended;
    ended_delegate.chip.details.executor = tasks::Executor::Person(tasks::PersonAddress::Person(9));
    let mut long = row();
    long.chip.title = text("é".repeat(TITLE_BYTES / 2 + 1).as_bytes());
    let long_line = format!(
        "task 41, {}… · a chat · an agent, charter 2, the producer's · working · 2026-11-09 07:40",
        "é".repeat(78)
    );
    let mut exact = row();
    exact.chip.title = text("é".repeat(TITLE_BYTES / 2).as_bytes());
    let exact_line = format!(
        "task 41, {} · a chat · an agent, charter 2, the producer's · working · 2026-11-09 07:40",
        "é".repeat(80)
    );
    // The raw JSON framing is 12 bytes; 250 two-byte letters put it at 512.
    let input_exact = format!("{{\"query\":\"{}\"}}", "é".repeat(250));
    let input_past = format!("{input_exact} ");
    let exact_input_line = format!("see: {input_exact}");
    let cut_input_line = format!("see: {input_exact}…");
    let long_question = "é".repeat(TITLE_BYTES / 2 + 1);
    let cut_question_line = format!("question · task 41, Add CSV export · {}…", "é".repeat(78));
    let mut malformed = remembering_record();
    malformed.transcript = Box::new([]);
    for (address, read, first, required, following) in [
        (
            "task 41 turn 3.2",
            SeeRead::Turn { number: 41, attempt: 3, turn: 2, read: turn_answer(remembering_record(), true) },
            "turn 3.2 of task 41 · $0.12",
            &[
                "## Kept about this person\nWithheld: shown to its person alone.\n## Task\nRead the charter.",
                "remember: {}\nWithheld: shown to its person alone.",
            ][..],
            "task N activity · task N",
        ),
        (
            "task 41 turn 3.2",
            SeeRead::Turn { number: 41, attempt: 3, turn: 2, read: turn_answer(remembering_record(), false) },
            "turn 3.2 of task 41 · $0.12",
            &[
                "## Kept about this person\n1. Prefers short answers.\n## Task\nRead the charter.",
                "remember: {\"action\":\"add\",\"text\":\"Uses a screen reader.\"}\nremember: line 2 kept; 2 of 12 lines.\n1. Prefers short answers.\n2. Uses a screen reader.",
            ],
            "task N activity · task N",
        ),
        (
            "task 41 activity",
            SeeRead::Activity { number: 41, activity: turn_activity(remembering_record(), true) },
            "activity of task 41: 1 events, oldest first",
            &["  remember: Withheld: shown to its person alone."],
            "task N turn A.T · task N · task N tree",
        ),
        (
            "task 41 activity",
            SeeRead::Activity { number: 41, activity: turn_activity(remembering_record(), false) },
            "activity of task 41: 1 events, oldest first",
            &["  remember: remember: line 2 kept; 2 of 12 lines."],
            "task N turn A.T · task N · task N tree",
        ),
        (
            "task 41 turn 3.2",
            SeeRead::Turn { number: 41, attempt: 3, turn: 2, read: turn_answer(malformed.clone(), true) },
            "turn 3.2 of task 41: unreadable turn",
            &[],
            "task N activity · task N",
        ),
        (
            "task 41 activity",
            SeeRead::Activity { number: 41, activity: turn_activity(malformed, true) },
            "activity of task 41: 1 events, oldest first",
            &["2026-11-09 07:40 · its run · turn 3.2 · unreadable turn\n  see: task 41 turn 3.2"],
            "task N turn A.T · task N · task N tree",
        ),
        (
            "inbox",
            SeeRead::Inbox {
                needs: question_needs(),
                words: question_words(b"Which delimiter should the export use?\nKeep this second line out."),
            },
            "waiting on you in project 7, Shop: 3, oldest first; each is answered on Needs you",
            &[
                "question · task 41, Add CSV export · Which delimiter should the export use?…",
                "question · task 42, Add CSV export",
                "question · task 41, Add CSV export",
            ][..],
            "task N · me",
        ),
        (
            "me",
            SeeRead::Me {
                person: 9,
                project: project(),
                needs: question_needs(),
                words: question_words(long_question.as_bytes()),
            },
            "you: Ada · owner in project 7, Shop",
            &[cut_question_line.as_str()],
            ME_NEXT,
        ),
        (
            "task 41 turn 3.2",
            SeeRead::Turn { number: 41, attempt: 3, turn: 2, read: whole_turn(input_exact.as_bytes()) },
            "turn 3.2 of task 41 · $0.12",
            &[exact_input_line.as_str()][..],
            "task N activity · task N",
        ),
        (
            "task 41 turn 3.2",
            SeeRead::Turn { number: 41, attempt: 3, turn: 2, read: whole_turn(input_past.as_bytes()) },
            "turn 3.2 of task 41 · $0.12",
            &[cut_input_line.as_str()],
            "task N activity · task N",
        ),
        (
            "task 41 activity",
            SeeRead::Activity { number: 41, activity: activity() },
            "activity of task 41: 5 or more events, oldest first",
            &[
                "2026-11-09 07:40 · its run · run 3 began\n2026-11-09 07:40 · Ada · Please check.…\n2026-11-09 07:40 · its run · turn 3.2 · 2 calls",
                "  see: read settings charter 4\n  propose_settings: proposed, waiting for an owner",
                "2026-11-09 07:40 · its run · turn 3.3 · 8388609 bytes, too large for this page\n  see: task 41 turn 3.3\n2026-11-09 07:40 · its run · run 3 ended\nearlier events not read",
            ][..],
            "task N turn A.T · task N · task N tree",
        ),
        (
            "task 41 turn 3.2",
            SeeRead::Turn {
                number: 41,
                attempt: 3,
                turn: 2,
                read: whole_turn(b"{ \"address\": \"settings charter 4\" }"),
            },
            "turn 3.2 of task 41 · $0.12",
            &[
                "## Task\nRead the charter.\n## Project\nShop",
                "I will read the charter.\nThen propose the settings.",
                "see: { \"address\": \"settings charter 4\" }\ncharter 4, the reviewer's\nLong settings text",
                "propose_settings: { \"more\": false }\nproposed, waiting for an owner\nProposal details",
            ],
            "task N activity · task N",
        ),
        (
            "change 41",
            SeeRead::Change { number: 41, panel: panel(Some(17)) },
            "change 41 · shop/app export into main · queued",
            &[
                "pull request 17",
                "CI: passed\ngate 4: passed\ngate 5: failed\n1 of 2 approvals by role 2\nrepairs: 1 of 2\nquestion 3: Use commas? · waiting for a person\nhold: the repair allowance was exhausted\nfirst in the queue into main",
            ],
            "task N · task N activity · changes · settings gates OWNER/NAME",
        ),
        (
            "changes",
            SeeRead::Changes { changes: Some(changes()), landings: Some(landings()) },
            "changes of project 7, Shop: 3 queued, 1 landing by turn",
            &[
                "into shop/app main: 2\nchange 42, Add export\nchange 41, Add import\ninto shop/app stable: 1\nchange 43, Fix quoting\nlanding by turn: 1\nlanding task 50: change 44, change 45\nlanded lately: 1\nchange 40, Keep delimiters · 2026-11-09 07:40",
            ],
            "change N · settings landing · ways",
        ),
        (
            "task 41 activity",
            SeeRead::Activity { number: 41, activity: page(List::uncapped(), false) },
            "activity of task 41: 0 events, oldest first",
            &[],
            "task N turn A.T · task N · task N tree",
        ),
        (
            "change 41",
            SeeRead::Change { number: 41, panel: panel(None) },
            "change 41 · shop/app export into main · queued",
            &["no pull request yet"],
            "task N · task N activity · changes · settings gates OWNER/NAME",
        ),
        (
            "changes",
            SeeRead::Changes { changes: Some(changes()), landings: None },
            "changes of project 7, Shop: 3 queued, not read landing by turn",
            &["landing by turn: not read", "landed lately: 1"],
            "change N · settings landing · ways",
        ),
        (
            "task 41 turn 3.3",
            SeeRead::Turn {
                number: 41,
                attempt: 3,
                turn: 3,
                read: Answer::None(jig_domain_core::reads::Why::TooLarge { bytes: OVERSIZE }),
            },
            "turn 3.3 of task 41: 8388609 bytes, past what one read holds",
            &[],
            "task N activity · task N",
        ),
        (
            "tasks",
            SeeRead::Tasks { tasks: tasks_page(&[row()]) },
            "live tasks of project 7, Shop: 1, the latest active first",
            &["task 41, Add CSV export · a chat · an agent, charter 2, the producer's · working · 2026-11-09 07:40"][..],
            TASKS_NEXT,
        ),
        (
            "me",
            SeeRead::Me { person: 9, project: project(), needs: needs(false), words: Box::new([]) },
            "you: Ada · owner in project 7, Shop",
            &[
                "your pool this period: $12.50, $7.26 left",
                "waiting on you here: 3 · 1 question · 2 proposals",
                "question · task 41, Add CSV export",
            ],
            ME_NEXT,
        ),
        (
            "project",
            SeeRead::Project {
                project: project(),
                held: tasks_page(&[held_row()]),
                repositories: Box::new([temper_engine_domain::RepositoryIdentity {
                    host: b"forge.example".as_slice().into(),
                    owner: b"shop".as_slice().into(),
                    name: b"app".as_slice().into(),
                }]),
            },
            "project 7, Shop",
            &[
                "repositories: 1",
                "forge.example/shop/app",
                "people: 2",
                "Ada, owner",
                "person 88, member",
                "this period: 3, from 1970-01-01 00:00 to 1970-01-08 00:00 · budget $12.50 · reserved $3.00 · spent $2.24 · left $7.26",
                "held tasks: 1",
                "task 41, Add CSV export · held · an agent, charter 2, the producer's · the budget is exhausted",
            ],
            PROJECT_NEXT,
        ),
        (
            "goals",
            SeeRead::Goals { ended: false, goals: tasks_page(&[goal_row(false)]), reports: reports() },
            "goals of project 7, Shop: 1, by priority",
            &["goal 12, Add CSV export · priority 1 · working, waiting on Ada · report: The export lands this week."],
            GOALS_NEXT,
        ),
        (
            "goals ended",
            SeeRead::Goals { ended: true, goals: list_row(goal_row(true), true, false), reports: reports() },
            "ended goals of project 7, Shop: 1, the latest ended first",
            &["goal 12, Add CSV export · priority 1 · done · report: The export lands this week."],
            "task N · goals",
        ),
        (
            "task 12",
            SeeRead::Task { number: 12, task: task_answer(task()), report: Some(reports()) },
            "task 12, Add CSV export · a goal · working",
            &[
                "requested by: Ada",
                "executor: an agent, charter 2, the producer's, on gpt-6",
                "spent: $2.24 of $12.50",
                "delegates: 0 live",
                "asked: Add CSV export…",
                "report: The export lands this week.",
                "The first paragraph.",
                "Left: Choose a delimiter.",
            ],
            TASK_NEXT,
        ),
        (
            "task 12",
            SeeRead::Task { number: 12, task: ended_task(), report: None },
            "task 12, Add CSV export · a change · done",
            &["requested by: task 6, Make export available", "result: done: Export landed.…"],
            TASK_NEXT,
        ),
        (
            "task 41 tree",
            SeeRead::Tree {
                number: 41,
                live: tasks_page(&[live_delegate.clone()]),
                ended: list_row(ended_delegate, true, false),
            },
            "delegates of task 41: 1 live, 1 ended",
            &[
                "task 41, Add CSV export · a procedure · working · spent $2.24 · delegates: 3",
                "task 41, Add CSV export · Ada · ended · spent $2.24 · delegates: 3",
            ],
            TREE_NEXT,
        ),
        (
            "inbox",
            SeeRead::Inbox { needs: needs(false), words: Box::new([]) },
            "waiting on you in project 7, Shop: 3, oldest first; each is answered on Needs you",
            &["question · task 41, Add CSV export", "proposal · task 41, Add CSV export"],
            "task N · me",
        ),
        (
            "tasks",
            SeeRead::Tasks { tasks: tasks_page(&[]) },
            "live tasks of project 7, Shop: 0, the latest active first",
            &[],
            TASKS_NEXT,
        ),
        (
            "tasks",
            SeeRead::Tasks { tasks: list_row(live_delegate, false, true) },
            "live tasks of project 7, Shop: 1 or more, the latest active first",
            &["task 41, Add CSV export · a change · a procedure · working · 2026-11-09 07:40"],
            TASKS_NEXT,
        ),
        (
            "goals ended",
            SeeRead::Goals { ended: true, goals: list_row(goal_row(true), true, true), reports: reports() },
            "ended goals of project 7, Shop: 1 or more, the latest ended first",
            &[],
            "task N · goals",
        ),
        (
            "inbox",
            SeeRead::Inbox { needs: needs(true), words: Box::new([]) },
            "waiting on you in project 7, Shop: 3 or more, oldest first; each is answered on Needs you",
            &["more wait on you"],
            "task N · me",
        ),
        (
            "project",
            SeeRead::Project {
                project: project(),
                held: Answer::None(jig_domain_core::reads::Why::Busy),
                repositories: Box::new([]),
            },
            "project 7, Shop",
            &["repositories: 0", "held tasks: not read"],
            PROJECT_NEXT,
        ),
        (
            "goals",
            SeeRead::Goals {
                ended: false,
                goals: tasks_page(&[goal_row(false)]),
                reports: Answer::None(jig_domain_core::reads::Why::Unknown),
            },
            "goals of project 7, Shop: 1, by priority",
            &["goal 12, Add CSV export · priority 1 · working, waiting on Ada · report: not read"],
            GOALS_NEXT,
        ),
        (
            "task 12",
            SeeRead::Task {
                number: 12,
                task: task_answer(task()),
                report: Some(Answer::None(jig_domain_core::reads::Why::Busy)),
            },
            "task 12, Add CSV export · a goal · working",
            &["report: not read", "spent: $2.24 of $12.50"],
            TASK_NEXT,
        ),
        (
            "goals",
            SeeRead::Goals {
                ended: false,
                goals: tasks_page(&[goal_row(false)]),
                reports: page(List::uncapped(), false),
            },
            "goals of project 7, Shop: 1, by priority",
            &["goal 12, Add CSV export · priority 1 · working, waiting on Ada · no report yet"],
            GOALS_NEXT,
        ),
        (
            "tasks",
            SeeRead::Tasks { tasks: tasks_page(&[unnamed]) },
            "live tasks of project 7, Shop: 1, the latest active first",
            &["task 41, Add CSV export · label 15 · person 88 · working · 2026-11-09 07:40"],
            TASKS_NEXT,
        ),
        (
            "tasks",
            SeeRead::Tasks { tasks: tasks_page(&[long]) },
            "live tasks of project 7, Shop: 1, the latest active first",
            &[long_line.as_str()],
            TASKS_NEXT,
        ),
        (
            "tasks",
            SeeRead::Tasks { tasks: tasks_page(&[exact]) },
            "live tasks of project 7, Shop: 1, the latest active first",
            &[exact_line.as_str()],
            TASKS_NEXT,
        ),
        (
            "goals",
            SeeRead::Goals {
                ended: false,
                goals: tasks_page(&[]),
                reports: Answer::None(jig_domain_core::reads::Why::Busy),
            },
            "goals of project 7, Shop: 0, by priority",
            &["report: not read"],
            GOALS_NEXT,
        ),
    ] {
        let answer = SeeAnswer::Person(Box::new(SeePerson { read, names: names() }));
        let (bytes, error) =
            crate::see_answer(address.as_bytes(), 0, &answer, &catalogue(), 20_000, crate::ForgeReadBounds::STOCK)
                .into_parts();
        let rendered = core::str::from_utf8(&bytes).unwrap();
        assert!(!error, "{address}: {rendered}");
        assert_eq!(rendered.lines().next(), Some(first), "{address}");
        for line in required {
            assert!(rendered.contains(&format!("\n{line}\n")), "{address}: missing {line}: {rendered}");
        }
        for hidden in [
            "digest",
            "cursor",
            "temper ctl",
            "unused-login",
            "unused-name",
            "secret metadata",
            "unused-endpoint",
            "abababababababab",
            "Brief continuation",
            "More project context",
            "Keep this second line out",
        ] {
            assert!(!rendered.contains(hidden), "{address}: {rendered}");
        }
        assert!(rendered.ends_with(&format!("see: {following}\n")), "{address}: {rendered}");
    }
}
