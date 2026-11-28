//! A preparation's connector port (jig/docs/design/domain/engine/briefs.md, 1.4).

use crate::engine::{self, Engine, In, Out};
use crate::routing::run::{briefs::Sections, preparation};
use crate::tests::*;
use jig_domain_contract::{Connector, parts};

pub(super) const SERVING: u16 = 1;
const HOME: Token = Token::new(19);
const PULL: Token = Token::new(20);
const TASK: u64 = 2;

pub(super) fn tree_core_with_sections(serving: &[u16], bounds: brief::Limits) -> (Core, Limits) {
    tree_core_with_declarations(Box::new([]), Box::new([]), &[], &[], serving, bounds)
}

pub(super) fn tree_with_sections(serving: &[u16], bounds: brief::Limits) -> (Core, Limits) {
    let (core, limits) = tree_core_with_sections(serving, bounds);
    (make_tree_in(core, &limits), limits)
}

pub(super) fn prepare(core: &mut Core, limits: &Limits, task: u64) {
    send_task(core, limits, tasks::Event::Prepare { reply_to: ReplyTo::new(Token::new(task)), task });
    assert_eq!(core.tasks.task(task).unwrap().phase, tasks::Phase::Active(tasks::Active::Preparing));
    core.contexts.insert(task, core.tasks.run_context(task).unwrap()).unwrap();
    core.begin_transcript(task, 0, None);
}

/// Bare preparation fixtures start at the notes continuation.
pub(super) fn plan_fixture(core: &mut Core, limits: &Limits, task: u64) -> Box<[Request]> {
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: *limits };
    let mut work = Queue::uncapped();
    let out = preparation::plan_brief(core, &env, &mut work, task, List::with_capacity(0), 0);
    assert!(work.is_empty());
    watch_decision(out)
}

pub(super) fn section(name: &'static [u8], token: Token, kind: u16, limit: u32) -> parts::Section {
    parts::Section { name, kind, token, size: 0, limit, priority: 10, required: true, document: None }
}

fn pair() -> Box<[parts::Section]> {
    Box::new([section(b"home", HOME, 11, 100), section(b"pull", PULL, 12, 100)])
}

fn agent(serving: &[u16], sections: u32) -> (Engine, Env<Limits>) {
    let bounds = brief::Limits { briefs: 1, sections, ..limits().brief };
    let (mut core, limits) = tree_with_sections(serving, bounds);
    runs::run_choice::host(&mut core, &limits, Token::new(7), &[b"scripted"]);
    runs::run_choice::account(&mut core, &limits, accounts::Event::Want);
    runs::run_choice::granted(&mut core, &limits, 1);
    (Engine::from_core(core), Env { now: Time::from_nanos(17), wall: Wall::EPOCH, limits })
}

fn step(engine: &mut Engine, env: &Env<Limits>, input: In) -> Box<[Out]> {
    let mut out = Queue::uncapped();
    engine::step(engine, env, input, &mut out);
    let mut outputs = List::with_capacity(out.len());
    while let Some(one) = out.pop() {
        outputs.push(one).unwrap();
    }
    outputs.into_boxed()
}

fn answer(engine: &mut Engine, env: &Env<Limits>, connector: u16, up: parts::Up) -> Box<[Out]> {
    step(engine, env, In::Connector(Connector(connector), contract::Up::Parts(up)))
}

fn finish_index(engine: &mut Engine, env: &Env<Limits>, task: u64) -> Box<[Out]> {
    prepare(engine.core_mut(), &env.limits, task);
    let load = fixture_window(engine.core_mut(), task, (1, 1));
    let mut out = step(
        engine,
        env,
        In::Store(engine_store::In::Loaded {
            load,
            result: engine_store::Answer::Rows(engine_store::Rows::Turns { turns: Box::new([]), more: false }),
        }),
    );
    for _ in 0..env.limits.notes.scopes {
        let Some(next) = notes_load(&out) else { break };
        assert!(parts_asks(&out).is_empty(), "no ask before the notes index is in");
        out = step(engine, env, In::Store(point_answer(next, None, false)));
    }
    out
}

fn notes_load(out: &[Out]) -> Option<Token> {
    for request in out {
        if let Out::Held(engine::Peer::Store(engine_store::Out::Load { load, range: engine_store::Range::Notes(_) })) =
            request
        {
            return Some(*load);
        }
    }
    None
}

fn parts_asks(out: &[Out]) -> List<(u16, &parts::Ask)> {
    let mut asks = List::with_capacity(u32::try_from(out.len()).unwrap());
    for request in out {
        if let Out::Ask(Connector(number), contract::Ask::Parts(ask)) = request {
            asks.push((*number, ask)).unwrap();
        }
    }
    asks
}

fn now(out: &[Out]) -> List<&Now> {
    let mut outputs = List::with_capacity(u32::try_from(out.len()).unwrap());
    for request in out {
        if let Out::Core(Request::Now(now)) = request {
            outputs.push(now.as_ref()).unwrap();
        }
    }
    outputs
}

fn workspace(out: &[Out], task: u64) -> u64 {
    for request in &now(out) {
        if let Now::WorkspaceRequest { task: number, attempt, .. } = request {
            assert_eq!(*number, task);
            return *attempt;
        }
    }
    panic!("workspace requested in this decision: {out:?}");
}

fn workspace_answer(task: u64, attempt: u64, success: bool) -> In {
    In::Core(Event::WorkspacePrepared {
        guide_limit: 0,
        task,
        attempt,
        holds: Box::new([]),
        writes: if success { Some(Box::new([])) } else { None },
        credentials: Box::new([]),
        workspace: None,
    })
}

fn sized(engine: &mut Engine, env: &Env<Limits>, task: u64, section: Token, size: u32) -> Box<[Out]> {
    answer(engine, env, SERVING, parts::Up::Sized { task, section, size })
}

fn planned(engine: &mut Engine, env: &Env<Limits>, task: u64, sections: Box<[parts::Section]>) -> Box<[Out]> {
    answer(engine, env, SERVING, parts::Up::Planned { task, sections })
}

fn drops(out: &[Out], task: u64) -> List<Token> {
    let mut dropped = List::with_capacity(u32::try_from(out.len()).unwrap());
    for &(connector, ask) in &parts_asks(out) {
        if let parts::Ask::Drop { task: gone, section } = ask {
            assert_eq!((connector, *gone), (SERVING, task));
            dropped.push(*section).unwrap();
        }
    }
    dropped
}

fn complete_pair(engine: &mut Engine, env: &Env<Limits>, task: u64) -> u64 {
    finish_index(engine, env, task);
    planned(engine, env, task, pair());
    sized(engine, env, task, HOME, 8);
    workspace(&sized(engine, env, task, PULL, 9), task)
}

#[test]
fn only_a_section_declarer_plans_after_the_notes_index() {
    let (mut engine, env) = agent(&[SERVING], 6);
    let out = finish_index(&mut engine, &env, TASK);
    let [(SERVING, parts::Ask::Plan(plan))] = parts_asks(&out).as_slice() else { panic!("one plan: {out:?}") };
    assert_eq!((plan.task, plan.project, plan.requester, plan.root), (TASK, 1, tasks::Party::Task(1), 1));
    let task = read_brief_part(
        engine.core(),
        BriefTextLimits {
            parts: env.limits.brief_parts,
            read_bytes: env.limits.brief.read_bytes,
            brief_bytes: env.limits.brief.brief_bytes,
        },
        TASK,
        BriefPart::Spec,
    )
    .unwrap();
    let notes = note_index_text(&List::with_capacity(0), 0, env.limits.brief_core_budgets.notes);
    assert_eq!(
        plan.core.as_ref(),
        [
            parts::Claim {
                size: u32::try_from(task.len()).unwrap(),
                limit: env.limits.brief_core_budgets.task,
                priority: 0,
                required: true
            },
            parts::Claim {
                size: u32::try_from(notes.len()).unwrap(),
                limit: env.limits.brief_core_budgets.notes,
                priority: 4,
                required: false
            },
        ]
    );
    assert_eq!((plan.slots, plan.budget), (4, env.limits.brief.brief_bytes));
    assert!(plan.documents.is_empty());
    assert!(engine.core().workspace_flight(TASK).is_none());
}

#[test]
fn with_no_section_declarer_the_notes_answer_continues_to_the_workspace() {
    let (mut engine, env) = agent(&[], 6);
    let out = finish_index(&mut engine, &env, TASK);
    assert!(parts_asks(&out).is_empty());
    workspace(&out, TASK);
}

#[test]
fn the_last_size_releases_the_workspace_and_prepared_run_carries_the_placed_order() {
    let (mut engine, env) = agent(&[SERVING], 6);
    finish_index(&mut engine, &env, TASK);
    let out = planned(&mut engine, &env, TASK, pair());
    let deadline = env.now.saturating_add(env.limits.brief.gather);
    assert_eq!(
        parts_asks(&out).as_slice(),
        [
            (SERVING, &parts::Ask::Gather { task: TASK, section: HOME, budget: env.limits.brief.read_bytes, deadline }),
            (SERVING, &parts::Ask::Gather { task: TASK, section: PULL, budget: env.limits.brief.read_bytes, deadline }),
        ]
    );
    // Answers can land out of placed order.
    assert!(sized(&mut engine, &env, TASK, PULL, 9).is_empty());
    let attempt = workspace(&sized(&mut engine, &env, TASK, HOME, 8), TASK);
    let out = step(&mut engine, &env, workspace_answer(TASK, attempt, true));
    let [Now::RunPrepared { brief, .. }] = now(&out).as_slice() else { panic!("prepared run: {out:?}") };
    let [task, notes, home, pull] = brief.as_ref() else { panic!("four placed sections: {brief:?}") };
    let brief::GatherPlaced::Core { kind: brief::Core::Task, .. } = task else { panic!("task section first") };
    let brief::GatherPlaced::Core { kind: brief::Core::NotesIndex, .. } = notes else { panic!("notes second") };
    assert_eq!(*home, brief::GatherPlaced::Connector { connector: SERVING, kind: 11, token: HOME, size: 8 });
    assert_eq!(*pull, brief::GatherPlaced::Connector { connector: SERVING, kind: 12, token: PULL, size: 9 });
}

#[test]
fn a_cut_is_an_operation_and_its_size_finishes_the_brief() {
    let (mut engine, env) = agent(&[SERVING], 6);
    finish_index(&mut engine, &env, TASK);
    planned(&mut engine, &env, TASK, Box::new([section(b"home", HOME, 11, 4)]));
    let out = sized(&mut engine, &env, TASK, HOME, 8);
    assert_eq!(parts_asks(&out).as_slice(), [(SERVING, &parts::Ask::Cut { task: TASK, section: HOME, size: 4 })]);
    assert!(now(&out).is_empty());
    workspace(&sized(&mut engine, &env, TASK, HOME, 3), TASK);
    let Some(Sections::Complete(order)) = engine.core().brief_sections.get(&TASK) else { panic!("completed brief") };
    assert_eq!(
        order.last(),
        Some(&brief::GatherPlaced::Connector { connector: SERVING, kind: 11, token: HOME, size: 3 })
    );
}

#[test]
fn a_required_plan_with_too_few_slots_fails_permanently_and_drops_all_sections() {
    let (mut engine, env) = agent(&[SERVING], 2);
    finish_index(&mut engine, &env, TASK);
    let out = planned(&mut engine, &env, TASK, pair());
    assert_eq!(drops(&out, TASK).as_slice(), [HOME, PULL]);
    let [Now::BriefFailed { task: TASK, failure }, Now::RunPreparationFailed { task: TASK }] = now(&out).as_slice()
    else {
        panic!("permanent failure: {out:?}")
    };
    assert_eq!(failure.fate, parts::Fate::Permanent);
    assert_eq!(failure.reason().as_ref(), b"brief: no room for its required sections");
    assert_eq!(engine.core().tasks.task(TASK).unwrap().tries.permanent, 1);
}

#[test]
fn a_waiting_plan_does_not_use_the_only_gathering_slot() {
    let (mut engine, env) = agent(&[SERVING], 6);
    finish_index(&mut engine, &env, TASK);
    assert_eq!(engine.core().tasks.task(TASK).unwrap().phase, tasks::Phase::Active(tasks::Active::Preparing));
    assert!(engine.core().brief.is_idle());
    complete_pair(&mut engine, &env, 4);
    planned(&mut engine, &env, TASK, pair());
    sized(&mut engine, &env, TASK, HOME, 8);
    workspace(&sized(&mut engine, &env, TASK, PULL, 9), TASK);
}

#[test]
fn a_child_refusal_drops_sections_not_admitted_to_gathering() {
    let (mut engine, env) = agent(&[SERVING], 6);
    finish_index(&mut engine, &env, TASK);
    planned(&mut engine, &env, TASK, pair());
    finish_index(&mut engine, &env, 4);
    let out = planned(&mut engine, &env, 4, pair());
    assert_eq!(drops(&out, 4).as_slice(), [HOME, PULL]);
    let notes = now(&out);
    let [Now::BriefFailed { task: 4, failure }, Now::RunPreparationFailed { task: 4 }] = notes.as_slice() else {
        panic!("capacity failure: {out:?}")
    };
    assert_eq!(failure.fate, parts::Fate::Capacity);
    assert_eq!(failure.reason().as_ref(), b"brief: no brief slot is free");
    sized(&mut engine, &env, TASK, HOME, 8);
    workspace(&sized(&mut engine, &env, TASK, PULL, 9), TASK);
}

#[test]
fn a_notes_index_from_an_abandoned_preparation_cannot_plan_the_replacement_brief() {
    let (mut engine, env) = agent(&[SERVING], 6);
    engine.core_mut().opened = true;
    let context = engine.core().tasks.run_context(TASK).unwrap();
    let mut out = step(&mut engine, &env, In::Core(Event::Activate { context, ready: true }));
    assert!(notes_load(&out).is_some(), "the first preparation has a cold index in flight: {out:?}");

    step(&mut engine, &env, In::Core(Event::Tasks(tasks::Event::PreparationWaiting { task: TASK })));
    assert_eq!(engine.core().tasks.task(TASK).unwrap().phase, tasks::Phase::Active(tasks::Active::Due));
    let context = engine.core().tasks.run_context(TASK).unwrap();
    let replacement = step(&mut engine, &env, In::Core(Event::Activate { context, ready: true }));
    assert!(notes_load(&replacement).is_none(), "the replacement waits for notes to become idle");
    assert!(parts_asks(&replacement).is_empty());
    assert_eq!(engine.core().tasks.task(TASK).unwrap().phase, tasks::Phase::Active(tasks::Active::Preparing));

    for _ in 0..env.limits.notes.scopes {
        let Some(load) = notes_load(&out) else { break };
        assert!(parts_asks(&out).is_empty(), "no connector plans before the notes index is ready");
        out = step(&mut engine, &env, In::Store(point_answer(load, None, false)));
    }
    let [(SERVING, parts::Ask::Plan(plan))] = parts_asks(&out).as_slice() else {
        panic!("one plan for the replacement: {out:?}")
    };
    assert_eq!(plan.task, TASK);
    planned(&mut engine, &env, TASK, pair());
    sized(&mut engine, &env, TASK, HOME, 8);
    let attempt = workspace(&sized(&mut engine, &env, TASK, PULL, 9), TASK);
    let out = step(&mut engine, &env, workspace_answer(TASK, attempt, true));
    let [Now::RunPrepared { task: TASK, .. }] = now(&out).as_slice() else { panic!("replacement run: {out:?}") };
}

#[test]
fn stale_preparation_abandons_a_plan_or_drops_each_placed_section() {
    for advances in 0_u32..4 {
        let mut serving = List::with_capacity(2);
        serving.push(SERVING).unwrap();
        if advances == 1 {
            serving.push(2).unwrap();
        }
        let (mut engine, env) = agent(serving.as_slice(), 6);
        finish_index(&mut engine, &env, TASK);
        if advances > 0 {
            let out = planned(&mut engine, &env, TASK, pair());
            if advances == 1 {
                assert!(parts_asks(&out).is_empty(), "one declarer has not answered");
            }
        }
        if advances == 3 {
            sized(&mut engine, &env, TASK, HOME, 8);
            workspace(&sized(&mut engine, &env, TASK, PULL, 9), TASK);
        }
        let out = step(
            &mut engine,
            &env,
            In::Core(Event::Tasks(tasks::Event::Hold { task: TASK, why: tasks::Hold::Budget })),
        );
        let asks = parts_asks(&out);
        if advances == 0 {
            assert_eq!(asks.as_slice(), [(SERVING, &parts::Ask::Abandon { task: TASK })]);
        } else {
            assert_eq!(drops(&out, TASK).as_slice(), [HOME, PULL]);
            if advances == 1 {
                assert_eq!(asks.as_slice().last(), Some(&(2, &parts::Ask::Abandon { task: TASK })));
            }
        }
        assert!(engine.core().brief.is_idle());
    }
}

#[test]
fn workspace_and_early_claim_refusals_drop_the_completed_brief() {
    for claim in [false, true] {
        let (mut engine, env) = agent(&[SERVING], 6);
        let attempt = complete_pair(&mut engine, &env, TASK);
        let refusal =
            if claim { In::Core(Event::ClaimRefused { task: TASK }) } else { workspace_answer(TASK, attempt, false) };
        let out = step(&mut engine, &env, refusal);
        assert_eq!(drops(&out, TASK).as_slice(), [HOME, PULL]);
        for request in &now(&out) {
            if let Now::RunPrepared { .. } = request {
                panic!("a refused preparation cannot produce a run");
            }
        }
        assert!(engine.core().brief_sections.is_empty());
    }
}

#[test]
fn unawaited_plans_are_dropped_and_unawaited_sizes_are_ignored() {
    let (mut engine, env) = agent(&[SERVING], 6);
    let out = planned(&mut engine, &env, TASK, pair());
    assert_eq!(drops(&out, TASK).as_slice(), [HOME, PULL]);
    assert!(sized(&mut engine, &env, TASK, HOME, 8).is_empty());
    finish_index(&mut engine, &env, TASK);
    let out = answer(&mut engine, &env, 2, parts::Up::Planned { task: TASK, sections: pair() });
    assert_eq!(
        parts_asks(&out).as_slice(),
        [(2, &parts::Ask::Drop { task: TASK, section: HOME }), (2, &parts::Ask::Drop { task: TASK, section: PULL }),]
    );
    planned(&mut engine, &env, TASK, pair());
    assert!(sized(&mut engine, &env, TASK, Token::new(99), 8).is_empty());
    assert!(sized(&mut engine, &env, TASK, HOME, 8).is_empty());
    workspace(&sized(&mut engine, &env, TASK, PULL, 9), TASK);
}

#[test]
fn stopping_and_settling_let_the_brief_complete_but_do_not_request_the_workspace() {
    for event in [Event::Stopping, Event::Settling] {
        let (mut engine, env) = agent(&[SERVING], 6);
        finish_index(&mut engine, &env, TASK);
        planned(&mut engine, &env, TASK, pair());
        sized(&mut engine, &env, TASK, HOME, 8);
        step(&mut engine, &env, In::Core(event));
        let out = sized(&mut engine, &env, TASK, PULL, 9);
        assert!(now(&out).is_empty(), "no next preparation step: {out:?}");
        assert!(engine.core().workspace_flight(TASK).is_none());
        assert!(engine.core().brief.is_idle());
        let Some(Sections::Complete(_)) = engine.core().brief_sections.get(&TASK) else {
            panic!("brief stayed complete")
        };
    }
}
