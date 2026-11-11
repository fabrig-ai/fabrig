//! The chat-work request's hand-made conversation in an isolated world (client/web/gallery.md, 2).
use super::{base_task, c, doc, limits, text};
use skein_lib::{List, Token, Wall};

pub(super) const AT: u64 = 1_793_923_680_000_000_000;
const REQUEST: &str = "please describe how temper works";
const CALLS: [(&str, &str, &str); 13] = [
    ("settings", r#"{"op":"inventory"}"#, "revision 1"),
    ("read_docs", r#"{"topic":"index"}"#, "temper"),
    ("read_docs", r#"{"topic":"index","from":1450}"#, "temper, on"),
    ("settings", r#"{"setting":{"kind":"charter","number":1}}"#, "default"),
    ("settings", r#"{"setting":{"kind":"template","number":2}}"#, "default"),
    ("read_docs", r#"{"topic":"landing-rule"}"#, "Landing rule"),
    ("read_docs", r#"{"topic":"change"}"#, "Changes"),
    ("read_docs", r#"{"topic":"goal"}"#, "Goals"),
    ("read_docs", r#"{"topic":"settings"}"#, "Settings"),
    ("read_docs", r#"{"topic":"task"}"#, "Tasks"),
    ("read_docs", r#"{"topic":"change","from":1438}"#, "Changes, on"),
    ("read_docs", r#"{"topic":"hold"}"#, "Hold"),
    ("read_docs", r#"{"topic":"settings","from":1482}"#, "Settings, on"),
];
const ANSWER: &[&str] = &[
    "**temper coordinates AI agents doing software work—from your request to a merged pull request.** It combines agent judgment with an engine that controls what work may run and land.",
    "### The workflow\n\n1. **You describe the work in a chat.** Small repository changes can go straight to a change task. Work you want planned and agreed first becomes a goal.\n2. **A goal coordinates the work.** It can request a read-only investigation, present a plan, ask for your agreement, and track its changes.\n3. **A producer writes each change** on a repository branch and opens a pull request.\n4. **temper manages landing:** dependencies, updates against the target branch, required CI, configured agent reviews and any required human approvals.\n5. **You receive the result:** what landed, its pull request and merge commit—or why the work stopped or nothing changed.",
    "### What temper enforces\n\nEvery task has bounded permissions, spending, delegation and time. The engine controls scheduling, dependencies, holds and landing requirements.\n\nAgents still supply judgment: understanding requests, designing solutions, writing code and reviewing it. **An instruction guides an agent; an engine rule constrains what can happen.** A successful CI status alone does not prove meaningful tests ran.",
    "### When work needs you\n\nQuestions and proposals wait in your inbox. Work can be held when it needs a decision or reaches a limit. You can also stop a task and its delegates.",
    "### How it is configured\n\n- **Charters:** agent instructions and run capabilities.\n- **Templates:** starting arrangements for new tasks.\n- **Policies and rules:** available authority and its ceilings.\n- **Landing rules:** requirements before merging.\n- **Project ways:** whether Drafts, Goals, Changes and Away controls appear.\n\nSettings changes are proposed to an owner for acceptance; I haven’t changed anything.\n\nThe documentation pages `task`, `goal`, `change` and `settings` explain these pieces in more detail.",
];
fn running(name: &str) -> bool {
    matches!(name, "chat-work-running" | "chat-work-writing" | "chat-work-empty")
}
pub(super) fn task(name: &str) -> c::Task {
    let mut task = base_task(1, REQUEST);
    task.spec.specification.words = doc(REQUEST);
    task.made = Wall::from_nanos(AT);
    task.last_activity = Wall::from_nanos(AT + 60_000_000_000);
    task.changed = task.last_activity;
    task.activity = match name {
        "chat-work-empty" => 0,
        "chat-work-one" | "chat-work-unknown" | "chat-work-separated" => 1,
        "chat-work-running" => 4,
        "chat-work-writing" => 13,
        _ => 15,
    };
    task.budget.numbers.spent = 70;
    task.budget.numbers.budget = 101_472;
    if running(name) {
        task.chip.phase = c::Phase::Running;
        task.chip.details.phase = c::ChipPhase::Active(c::RunStage::Running { attempt: 1 });
    }
    task
}
fn call(number: u64, tool: &str, input: &str, outcome: &str, failed: bool) -> c::Block {
    c::Block::Call(c::Call {
        number,
        tool: text(tool),
        input: text(input),
        outcome: Some(text(outcome)),
        failed,
        objects: List::with_capacity(0),
        parts: List::with_capacity(0),
    })
}
fn line(blocks: &[c::Block], number: u32) -> c::DecodedActivityLine {
    let blocks = List::from_boxed(blocks.to_vec().into_boxed_slice());
    let mut row = super::turn_line(&blocks, number, None, false);
    // Twenty-nine seconds span the calls, independent of the answer's later turns.
    row.source.task = 1;
    row.decoded.as_mut().expect("decoded call").task = 1;
    row.source.position = u64::from(number);
    row.source.line = c::Activity::Turn { reply: u64::from(number), attempt: 1, turn: number, read: None };
    row.decoded.as_mut().expect("decoded call").cumulative = u64::from(number);
    row.source.at = Wall::from_nanos(AT + u64::from(number.saturating_sub(1)) * 2_416_666_667);
    row.decoded.as_mut().expect("decoded call").at = row.source.at;
    row
}
fn whole_activity(name: &str) -> c::ActivityPage {
    let mut rows = List::with_capacity(16);
    if name == "chat-work-one" {
        rows.push(line(&[call(1, "read_docs", r#"{"topic":"hold"}"#, "Hold", false)], 1)).expect("one call");
    } else if name == "chat-work-unknown" {
        rows.push(line(
            &[call(1, "read_file", "{}", "one file", false), call(2, "read_file", "{}", "another file", false)],
            1,
        ))
        .expect("unknown tool");
    } else if name == "chat-work-separated" {
        rows.push(line(
            &[
                call(1, "read_docs", r#"{"topic":"hold"}"#, "Hold", false),
                c::Block::Text(text("The hold is clear.")),
                call(2, "settings", r#"{"op":"inventory"}"#, "revision 1", false),
            ],
            1,
        ))
        .expect("separated calls");
    } else if name != "chat-work-empty" {
        let count = if name == "chat-work-running" { 4 } else { CALLS.len() };
        for (index, (tool, input, outcome)) in CALLS.iter().take(count).enumerate() {
            let failed = name == "chat-work-failed" && index == 3;
            rows.push(line(
                &[call(1, tool, input, if failed { "not allowed" } else { outcome }, failed)],
                u32::try_from(index + 1).expect("few calls"),
            ))
            .expect("call row");
        }
        if name != "chat-work-running" && name != "chat-work-writing" {
            // The supplied window has sixteen lines. Keep the answer whole in two committed turns.
            for (index, body) in [ANSWER[..2].join("\n\n"), ANSWER[2..].join("\n\n")].iter().enumerate() {
                rows.push(line(
                    &[c::Block::Text(
                        c::Text::new(body.as_bytes().into(), limits().client.domain.turn_text)
                            .expect("committed answer fits"),
                    )],
                    14 + u32::try_from(index).expect("two answer turns"),
                ))
                .expect("answer row");
            }
        }
    }
    let mut reversed = rows.into_boxed().into_vec();
    reversed.reverse();
    let rows = List::from_boxed(reversed.into_boxed_slice());
    c::ActivityPage {
        rows,
        before: (name == "chat-work-reading").then(|| c::Cursor::new(b"older".as_slice().into(), 5).expect("cursor")),
        after: None,
        cut: false,
    }
}
pub(super) fn run(name: &str) -> c::Delivery {
    let writing = if name == "chat-work-writing" {
        Some(c::RunText {
            attempt: 1,
            began: Some(Wall::from_nanos(AT)),
            turn: c::ProvisionalTurnName { attempt: 1, number: 14, retry: 0 },
            from: 0,
            through: u32::try_from(ANSWER[0].len()).expect("short opening"),
            text: text(ANSWER[0]),
            cut: false,
            call: None,
            writing: None,
        })
    } else {
        None
    };
    c::Delivery::RunState(Box::new(c::RunState {
        task: 1,
        attempt: Some(1),
        committed_turn: 0,
        writing,
        call: (name == "chat-work-running").then(|| c::CurrentCall {
            identity: c::CallIdentity { owner: Token::new(1), sequence: 5, retry: 0, id: text("template") },
            tool: text("settings"),
            input: text(CALLS[4].1),
            cut: false,
            began: Some(Wall::from_nanos(AT)),
        }),
    }))
}

pub(super) fn activity(name: &str, before: Option<&c::Cursor>) -> c::ActivityPage {
    let whole = whole_activity(name);
    let offset = before.and_then(|cursor| cursor.value.bytes().first()).copied().map_or(0, usize::from);
    let mut rows = List::with_capacity(limits().client.domain.rows);
    for row in whole.rows.iter().skip(offset).take(usize::try_from(rows.capacity()).expect("small page")) {
        rows.push(row.clone()).expect("page capacity");
    }
    let next = offset + usize::try_from(rows.len()).expect("small page");
    c::ActivityPage {
        rows,
        before: if next < usize::try_from(whole.rows.len()).expect("small conversation") {
            Some(c::Cursor::new(Box::new([u8::try_from(next).expect("small offset")]), 1).expect("cursor"))
        } else {
            whole.before
        },
        after: None,
        cut: false,
    }
}
