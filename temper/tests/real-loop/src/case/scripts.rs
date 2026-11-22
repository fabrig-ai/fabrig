//! The reference case's judgements, played word for word (testing/reference-case.md).
use super::{
    A, A_REPORT, A_SPEC_PREFIX, A_SPEC_SUFFIX, ANSWERED, ASKED, Agent, DESIGN_DELEGATED, DESIGN_DOC, DESIGN_LINE,
    DESIGN_REPORT, DESIGN_SPEC, DESIGN_TITLE, DESIGN_WORDS, DIRECTOR, DIRECTOR_REPORT, GREEN_LIGHT,
    OWNER_EFFORT_NUMBER, OWNER_MODEL_NUMBER, REVIEWED, SCHEDULED, draft_commit,
};
use serde_json::{Value, json};
use skein_fake_llm_domain::api::{Finish, Line, Rule, Script, Turn};
use std::net::SocketAddr;

/// Smith renders its opening in smith-domain-run/src/prompt.rs.
const BEGIN: &str = "Begin the work your brief describes.";
/// Smith returns crossed finish feedback in smith-domain-run/src/run.rs via prompt.rs.
const CROSSED: &str = "Finish was not accepted because messages arrived.";
/// Temper renders smith's message acknowledgement in temper-engine-smith/src/answers.rs.
const MESSAGE: &str = "Message ";
/// Temper renders smith's delegation acknowledgement in temper-engine-smith/src/answers.rs.
const DELEGATED: &str = "Delegated tasks: ";
/// Smith renders labelled inbox deliveries in smith-domain-run/src/inbox.rs.
const DELIVERY: &str = ": ";

/// The turn rules for `step_4_1`'s scripts, index-parallel (skein's testing/fakes/llm.md, 2.4).
#[must_use]
pub fn step_4_1_rules() -> Box<[Box<[Rule]>]> {
    let director = [
        (A_REPORT, 3),
        (CROSSED, 3),
        (DESIGN_REPORT, 2),
        (MESSAGE, 2),
        (&format!("{DELEGATED}{A}"), 1),
        (DELIVERY, 1),
        (BEGIN, 0),
        ("", 1),
    ]
    .map(|(cue, turn)| Rule { cue: cue.as_bytes().into(), turn });
    let a = [
        (GREEN_LIGHT, 4),
        (CROSSED, 4),
        (&format!("Result from task {}:\n", super::DESIGN), 2),
        (MESSAGE, 3),
        (&format!("{DELEGATED}{}", super::DESIGN), 1),
        (DELIVERY, 1),
        (BEGIN, 0),
        ("", 1),
    ]
    .map(|(cue, turn)| Rule { cue: cue.as_bytes().into(), turn });
    Box::new([Box::new(director), Box::new(a), Box::new([]), Box::new([])])
}

/// The opener and check judge fresh input, including messages beside tool results.
#[must_use]
pub fn step_4_1_4_3_rules() -> Box<[Box<[Rule]>]> {
    let mut rules = step_4_1_rules().into_vec();
    // A due timer can arrive beside subscribe's result, and B's report beside
    // cancel's result. Read those messages before choosing another wait.
    let check = [
        (super::B_REPORT, 6),
        (CROSSED, 6),
        (&format!("{}: ", super::CHECK), 4),
        (super::BENCHMARK_RUNNING, 2),
        ("Subscription ", 3),
        ("Done", 5),
        (BEGIN, 0),
        ("", 1),
    ]
    .map(|(cue, turn)| Rule { cue: cue.as_bytes().into(), turn });
    let b = [(super::CHECK_REASON, 3), (CROSSED, 3), (DELEGATED, 1), (BEGIN, 0), ("", 2)]
        .map(|(cue, turn)| Rule { cue: cue.as_bytes().into(), turn });
    rules.push(Box::new(check));
    rules.push(Box::new(b));
    rules.push(Box::default());
    rules.into_boxed_slice()
}

/// The chat follows fresh input across retry; A retains its counted conversation.
#[must_use]
pub fn step_4_6_rules() -> Box<[Box<[Rule]>]> {
    // The direct answer wakes A independently. Its result may already be in
    // the chat's next input, so a counted acknowledgement wait could wait
    // forever for a result it just read. Keep all turns, selecting that finish
    // from the result and the acknowledgement wait from the notice alone.
    let chat = [
        (super::PHASE_REPORT, 4),
        (CROSSED, 4),
        ("Person answered task", 3),
        (super::PHASE_QUESTION, 1),
        (DELEGATED, 2),
        (DELIVERY, 2),
        (BEGIN, 0),
        ("", 2),
    ]
    .map(|(cue, turn)| Rule { cue: cue.as_bytes().into(), turn });
    Box::new([Box::new(chat), Box::new([])])
}

fn call(name: &[u8], value: &Value) -> Turn {
    Turn {
        lines: Box::new([Line::Call {
            name: name.into(),
            arguments: serde_json::to_vec(value).expect("tool JSON").into_boxed_slice(),
        }]),
        finish: Finish::ToolCalls,
        tokens: 20,
    }
}
fn wait(words: &str) -> Turn {
    Turn {
        lines: Box::new([
            Line::Text { text: words.as_bytes().into() },
            Line::Call { name: b"wait".as_slice().into(), arguments: b"{}".as_slice().into() },
        ]),
        finish: Finish::ToolCalls,
        tokens: 20,
    }
}
fn then_wait(mut turn: Turn, words: &str) -> Turn {
    let mut lines = turn.lines.into_vec();
    lines.extend(wait(words).lines);
    turn.lines = lines.into_boxed_slice();
    turn
}
fn report(words: &str) -> Turn {
    call(b"finish", &json!({"form":"report","text":words,"fields":{}}))
}
fn grants(forge: SocketAddr, kinds: &[u8]) -> Value {
    let mut grants = Vec::new();
    for kind in kinds {
        grants.push(
            json!({"connector":1,"kind":kind,"segments":["forge",forge.to_string()],"terminal":"open","last":"acme"}),
        );
    }
    Value::Array(grants)
}

#[must_use]
pub fn step_4_1(forge: SocketAddr) -> Box<[Script]> {
    let owner = json!({"batch":[{
        "executor":{"kind":"agent","charter":1},"tracked":{"priority":1},
        "spec":{"words":format!("{A_SPEC_PREFIX}{}{A_SPEC_SUFFIX}", draft_commit()),"parameters":[]},
        "contract":{"kind":"report","words":128},
        "llm":{"main":{"model":OWNER_MODEL_NUMBER,"effort":OWNER_EFFORT_NUMBER}},
        "authority":{"tools":3,"grants":grants(forge, &[2,3,4,7,8,10,11,12,13]),
            "delegation":{"kinds":[{"kind":"procedure","number":2},{"kind":"agent","number":1},{"kind":"agent","number":4}],"tasks":6,"depth":3},
            "budget":{"spend":20},"notes":0}
    }]});
    let design = json!({"batch":[{
        "executor":{"kind":"procedure","connector":1,"code":2},
        "spec":{"words":DESIGN_SPEC,"parameters":[
            {"name":1,"kind":"resource","connector":1,"resource":temper_engine_domain_forge::tag(1, crate::forge::ID)},
            {"name":2,"kind":"bytes","value":"main"}]},
        "contract":{"kind":"change","connector":1,"change_kind":1,"words":128},
        "authority":{"tools":1,"grants":grants(forge, &[2,3,4,10,11,12]),
            "delegation":{"kinds":[{"kind":"agent","number":1},{"kind":"agent","number":4}],"tasks":4,"depth":2},
            "budget":{"spend":10},"notes":0}
    }]});
    Box::new([
        Script {
            cue: Agent::Director.cue().into(),
            turns: Box::new([call(b"delegate", &owner), wait(SCHEDULED), wait(ANSWERED), report(DIRECTOR_REPORT)]),
        },
        Script {
            cue: Agent::A.cue().into(),
            turns: Box::new([
                call(b"delegate", &design),
                wait(DESIGN_DELEGATED),
                call(b"message", &json!({"target":DIRECTOR,"form":"question","words":DESIGN_REPORT})),
                wait(ASKED),
                report(A_REPORT),
            ]),
        },
        Script {
            cue: Agent::Design.cue().into(),
            turns: Box::new([
                call(b"write", &json!({"path":DESIGN_DOC,"content":DESIGN_WORDS})),
                call(b"finish", &json!({"form":"change","fields":{"title":DESIGN_TITLE,"body":DESIGN_LINE}})),
            ]),
        },
        Script {
            cue: Agent::Review.cue().into(),
            turns: Box::new([call(
                b"finish",
                &json!({"form":"verdict","label":"1","text":REVIEWED,"fields":{},"items":[]}),
            )]),
        },
    ])
}

// Account stories use no forge grants and adopt no repository.
fn owner(spec: &str, survey: bool) -> Turn {
    call(
        b"delegate",
        &json!({"batch":[{
            "executor":{"kind":"agent","charter":1},"tracked":{"priority":1},
            "spec":{"words":spec,"parameters":[]},"contract":{"kind":"report","words":128},
            "llm":{"main":{"model":OWNER_MODEL_NUMBER,"effort":OWNER_EFFORT_NUMBER}},
            "authority":{"tools":3,"grants":[],
                "delegation":{"kinds":[{"kind":"agent","number":1}],"tasks":u8::from(survey),"depth":1},
                "budget":{"spend":20},"notes":0}
        }]}),
    )
}

#[must_use]
pub fn step_4_3(_forge: SocketAddr) -> Box<[Script]> {
    use super::{B_REPORT, B_SPEC, BENCHMARK, BENCHMARK_RUNNING, BENCHMARK_SPEC, CHECK, CHECK_REASON, CHECK_REPORT};
    use skein_fake_llm_domain::api::Piece;
    let delegate = |spec, tools, tasks, depth, spend| {
        call(
            b"delegate",
            &json!({"batch":[{
                "executor":{"kind":"agent","charter":2},
                "spec":{"words":spec,"parameters":[]},"contract":{"kind":"report","words":128},
                "authority":{"tools":tools,"grants":[],
                    "delegation":{"kinds":[{"kind":"agent","number":2}],"tasks":tasks,"depth":depth},
                    "budget":{"spend":spend},"notes":0}
            }]}),
        )
    };
    let mut arm = call(b"subscribe", &json!({"kind":"timer","at":0}));
    let mut lines = vec![Line::Pieces {
        pieces: Box::new([Piece {
            text: b"All workers placed; arm the two-hour check.".as_slice().into(),
            hold: Some(crate::cluster::PLACED.into()),
        }]),
    }];
    lines.extend(arm.lines);
    arm.lines = lines.into_boxed_slice();
    Box::new([
        Script {
            cue: Agent::Check.cue().into(),
            turns: Box::new([
                delegate(B_SPEC, 3, 1, 1, 20),
                wait("B scheduled; awaiting the benchmark on s2."),
                arm,
                wait("The two-hour check is armed."),
                call(b"cancel", &json!({"target":BENCHMARK,"reason":CHECK_REASON})),
                wait("The benchmark's cancellation was requested."),
                report(CHECK_REPORT),
            ]),
        },
        Script {
            cue: Agent::B.cue().into(),
            turns: Box::new([
                delegate(BENCHMARK_SPEC, 2, 0, 0, 5),
                // A grandchild needs an introduction to message the director.
                call(b"introduce", &json!({"left":CHECK,"right":BENCHMARK})),
                wait("B's benchmark is delegated."),
                report(B_REPORT),
            ]),
        },
        Script {
            cue: Agent::Benchmark.cue().into(),
            turns: Box::new([then_wait(
                call(b"message", &json!({"target":CHECK,"form":"words","words":BENCHMARK_RUNNING})),
                BENCHMARK_RUNNING,
            )]),
        },
    ])
}

/// The opener and the check share a forge, with a script for each distinct cue.
#[must_use]
pub fn step_4_1_4_3(forge: SocketAddr) -> Box<[Script]> {
    let mut scripts = step_4_1(forge).into_vec();
    scripts.extend(step_4_3(forge));
    scripts.into_boxed_slice()
}

/// Commit the question read before waiting through a worker's ending.
#[must_use]
pub fn step_4_6(_forge: SocketAddr) -> Box<[Script]> {
    use super::{HANG, HANG_PERSON, HANG_READ, HANG_REPORT, PHASE_QUESTION, PHASE_REPORT, PHASE_SPEC};
    Box::new([
        Script {
            cue: HANG_PERSON.as_bytes().into(),
            turns: Box::new([
                owner(PHASE_SPEC, false),
                // Consult the project's phase notes before directing A. Unlike
                // wait, the completed tool turn commits the question read.
                {
                    let mut consulted = call(
                        b"recall",
                        &json!({"kind":"search","scopes":[{"kind":"project","project":super::PROJECT}],"query":"Export's next phase"}),
                    );
                    let mut lines = vec![Line::Text { text: HANG_READ.as_bytes().into() }];
                    lines.extend(consulted.lines);
                    consulted.lines = lines.into();
                    consulted
                },
                wait("The committed question waits for its answer."),
                wait("The next phase has its answer."),
                report(HANG_REPORT),
            ]),
        },
        Script {
            cue: PHASE_SPEC.as_bytes().into(),
            turns: Box::new([
                call(b"message", &json!({"target":HANG,"form":"question","words":PHASE_QUESTION})),
                wait("The next phase awaits its answer."),
                report(PHASE_REPORT),
            ]),
        },
    ])
}

// Act 2, step 5.1 (testing/reference-case.md)
#[must_use]
pub fn step_5_1(_forge: SocketAddr) -> Box<[Script]> {
    use super::{
        RULES, RULES_ANSWERED, RULES_DELEGATED, RULES_REPORT, SCOPE, SCOPE_ASKED, SCOPE_QUESTION, SCOPE_QUESTION_HOLD,
        SCOPE_REPORT, SCOPE_SPEC,
    };
    use skein_fake_llm_domain::api::Piece;
    let mut question = call(b"message", &json!({"target":RULES,"form":"question","words":SCOPE_QUESTION}));
    let mut lines = vec![Line::Pieces {
        pieces: Box::new([Piece {
            text: b"Ask x3's scope after the director is held.".as_slice().into(),
            hold: Some(SCOPE_QUESTION_HOLD.into()),
        }]),
    }];
    lines.extend(question.lines);
    question.lines = lines.into();
    let director: Box<[Turn]> =
        Box::new([owner(SCOPE_SPEC, false), wait(RULES_DELEGATED), wait(RULES_ANSWERED), report(RULES_REPORT)]);
    Box::new([
        // The committed delegation in the fresh brief selects the resumed
        // script before the ordinary director cue. Its Begin waits, instead
        // of repeating the delegation while A's cancelled turn resumes.
        Script { cue: format!("result: Delegated tasks: {SCOPE}").into_bytes().into(), turns: director.clone() },
        Script { cue: super::Agent::Rules.cue().into(), turns: director },
        Script {
            cue: super::Agent::Scope.cue().into(),
            turns: Box::new([question, wait(SCOPE_ASKED), report(SCOPE_REPORT)]),
        },
    ])
}

#[must_use]
pub fn step_5_1_rules() -> Box<[Box<[Rule]>]> {
    // The trial with counted turns delegated again after Release: the fresh
    // conversation resets its assistant count. Select only the director by
    // fresh input; A retains its counted conversation across its parked wait.
    let director = [
        (super::SCOPE_REPORT, 3),
        (CROSSED, 3),
        (super::SCOPE_QUESTION, 2),
        (MESSAGE, 2),
        (DELEGATED, 1),
        (DELIVERY, 1),
        (BEGIN, 0),
        ("", 1),
    ]
    .map(|(cue, turn)| Rule { cue: cue.as_bytes().into(), turn });
    let mut resumed = director.clone();
    for rule in &mut resumed {
        if rule.cue.as_ref() == BEGIN.as_bytes() {
            rule.turn = 1;
        }
    }
    Box::new([Box::new(resumed), Box::new(director), Box::new([])])
}

// Act 2, step 5.3 (testing/reference-case.md)
#[must_use]
pub fn step_5_3(_forge: SocketAddr) -> Box<[Script]> {
    use super::{
        MOVER, MOVER_AMENDMENT_REVISION, MOVER_ASKED, MOVER_QUESTION, MOVER_REPORT, MOVER_SPEC, NEW_BRIEF, SHAPE,
        SHAPE_AMENDED, SHAPE_DELEGATED, SHAPE_REASON, SHAPE_REPORT,
    };
    Box::new([
        Script {
            cue: Agent::Shape.cue().into(),
            turns: Box::new([
                call(
                    b"delegate",
                    &json!({"batch":[{
                        "executor":{"kind":"agent","charter":2},"tracked":{"priority":1},
                        "spec":{"words":MOVER_SPEC,"parameters":[]},"contract":{"kind":"report","words":128},
                        "authority":{"tools":3,"grants":[],"delegation":{"kinds":[],"tasks":0,"depth":0},
                            "budget":{"spend":20},"notes":0}
                    }]}),
                ),
                wait(SHAPE_DELEGATED),
                call(
                    b"amend",
                    &json!({"target":MOVER,"revision":MOVER_AMENDMENT_REVISION - 1,
                    "amendment":{"instructions":NEW_BRIEF,"reason":SHAPE_REASON}}),
                ),
                wait(SHAPE_AMENDED),
                report(SHAPE_REPORT),
            ]),
        },
        Script {
            cue: Agent::Mover.cue().into(),
            turns: Box::new([
                call(b"message", &json!({"target":SHAPE,"form":"question","words":MOVER_QUESTION})),
                wait(MOVER_ASKED),
                report(MOVER_REPORT),
            ]),
        },
    ])
}

// Act 2, step 5.2 (testing/reference-case.md)
#[must_use]
pub fn step_5_2_rules() -> Box<[Box<[Rule]>]> {
    use super::{BOWNER, BOWNER_REPORT, H1};
    let table = |rows: &[(&str, u32)]| {
        rows.iter().map(|(cue, turn)| Rule { cue: cue.as_bytes().into(), turn: *turn }).collect()
    };
    Box::new([
        table(&[(BOWNER_REPORT, 2), (CROSSED, 2), (&format!("{DELEGATED}{BOWNER}"), 1), (BEGIN, 0), ("", 1)]),
        table(&[
            (&format!("Result from task {H1}:"), 2),
            (CROSSED, 2),
            (&format!("{DELEGATED}{H1}"), 1),
            (BEGIN, 0),
            ("", 1),
        ]),
        Box::new([]),
        Box::new([]),
    ])
}

#[must_use]
pub fn step_5_2(forge: SocketAddr) -> Box<[Script]> {
    use super::{BOWNER_REPORT, BOWNER_SPEC, H1_DOC, H1_LINE, H1_SPEC, H1_WORDS, PAUSE_REPORT, REVIEW_HOLD};
    use skein_fake_llm_domain::api::Piece;
    let owner = json!({
        "executor":{"kind":"agent","charter":1},"tracked":{"priority":1},
        "spec":{"words":BOWNER_SPEC,"parameters":[]},"contract":{"kind":"report","words":128},
        "authority":{"tools":3,"grants":grants(forge,&[2,3,4,7,8,10,11,12,13]),
            "delegation":{"kinds":[{"kind":"agent","number":1},{"kind":"agent","number":4},{"kind":"procedure","number":2}],"tasks":4,"depth":3},
            "budget":{"spend":40},"notes":0}
    });
    let change = json!({
        "executor":{"kind":"procedure","connector":1,"code":2},
        "spec":{"words":H1_SPEC,"parameters":[
            {"name":1,"kind":"resource","connector":1,"resource":temper_engine_domain_forge::tag(1, crate::forge::ID)},
            {"name":2,"kind":"bytes","value":"main"}]},
        "contract":{"kind":"change","connector":1,"change_kind":1,"words":128},
        "authority":{"tools":1,"grants":grants(forge,&[2,3,4,10,11,12]),
            "delegation":{"kinds":[{"kind":"agent","number":1},{"kind":"agent","number":4}],"tasks":2,"depth":2},
            "budget":{"spend":10},"notes":0}
    });
    let mut verdict = call(b"finish", &json!({"form":"verdict","label":"1","text":REVIEWED,"fields":{},"items":[]}));
    let mut lines = vec![Line::Pieces {
        pieces: Box::new([Piece { text: b"h1's review is held.".as_slice().into(), hold: Some(REVIEW_HOLD.into()) }]),
    }];
    lines.extend(verdict.lines);
    verdict.lines = lines.into();
    Box::new([
        Script {
            cue: Agent::Pause.cue().into(),
            turns: Box::new([call(b"delegate", &json!({"batch":[owner]})), wait("B owns h1."), report(PAUSE_REPORT)]),
        },
        Script {
            cue: Agent::Bowner.cue().into(),
            turns: Box::new([
                call(b"delegate", &json!({"batch":[change]})),
                wait("h1 is delegated."),
                report(BOWNER_REPORT),
            ]),
        },
        Script {
            cue: H1_SPEC.as_bytes().into(),
            turns: Box::new([
                call(b"write", &json!({"path":H1_DOC,"content":H1_WORDS})),
                call(b"finish", &json!({"form":"change","fields":{"title":"h1","body":H1_LINE}})),
            ]),
        },
        Script { cue: Agent::Review.cue().into(), turns: Box::new([verdict]) },
    ])
}

// Act 2, step 5.4 (testing/reference-case.md)
fn direction_owner(forge: SocketAddr, spec: &str, spend: u64, notes: u32, tasks: u32) -> Value {
    json!({
        "executor":{"kind":"agent","charter":1},"tracked":{"priority":1},
        "spec":{"words":spec,"parameters":[]},"contract":{"kind":"report","words":128},
        "authority":{"tools":391,"notes":notes,"grants":grants(forge,&[2,3,4,10,11,12]),
            "delegation":{"kinds":[{"kind":"agent","number":1},{"kind":"agent","number":4},{"kind":"procedure","number":2}],"tasks":tasks,"depth":3},
            "budget":{"spend":spend}}
    })
}
fn direction_change(forge: SocketAddr, spec: &str, path: &str, tasks: u32) -> Value {
    repository_change(forge, crate::forge::ID, spec, path, tasks)
}
fn repository_change(forge: SocketAddr, repository: u32, spec: &str, path: &str, tasks: u32) -> Value {
    json!({
        "executor":{"kind":"procedure","connector":1,"code":2},
        "spec":{"words":spec,"parameters":[
            {"name":1,"kind":"resource","connector":1,"resource":temper_engine_domain_forge::tag(1, repository)},
            {"name":2,"kind":"bytes","value":"main"},
            {"name":6,"kind":"bytes","value":path}]},
        "contract":{"kind":"change","connector":1,"change_kind":1,"words":128},
        "authority":{"tools":1,"notes":0,"grants":grants(forge,&[2,3,4,10,11,12]),
            "delegation":{"kinds":[{"kind":"agent","number":1},{"kind":"agent","number":4}],"tasks":tasks,"depth":2},"budget":{"spend":10}}
    })
}
fn direction_held(name: &[u8], mut turn: Turn) -> Turn {
    use skein_fake_llm_domain::api::Piece;
    let mut lines = vec![Line::Pieces {
        pieces: Box::new([Piece { text: super::DIRECTION_HELD_WORDS.as_bytes().into(), hold: Some(name.into()) }]),
    }];
    lines.extend(turn.lines);
    turn.lines = lines.into();
    turn
}

#[must_use]
pub fn step_5_4(forge: SocketAddr) -> Box<[Script]> {
    use super::{
        B_DELEGATE_HOLD, B_FILE, B_WORDS, DEFERRED_NOTE, DEFERRED_NOTE_NUMBER, DEFERRED_WORDS, DIRECTION_BRIEF,
        DIRECTION_REASON, DIRECTION_REPORT, DIRECTION_REVISION, OVERLAP_OWNER_REPORT, OVERLAP_OWNER_SPEC, PLANNED,
        PLANNED_CANCEL_REASON, PLANNED_CODING_HOLD, PLANNED_SPEC, PROJECT, STORY_CHANGE, STORY_GREEN, STORY_OWNER,
        STORY_OWNER_REPORT, STORY_OWNER_SPEC, STORY_RECALL_HOLD, STORY_SPEC, STORY_WORDS, X3, X3_CANCEL_REASON,
        X3_FILE, X3_REVIEW_HOLD, X3_SPEC, X3_WORDS,
    };
    let verdict = || call(b"finish", &json!({"form":"verdict","label":"1","text":REVIEWED,"fields":{},"items":[]}));
    Box::new([
        Script {
            cue: Agent::Direction.cue().into(),
            turns: Box::new([
                then_wait(
                    call(
                        b"delegate",
                        &json!({"batch":[direction_owner(forge, STORY_OWNER_SPEC,30,2,6),direction_owner(forge, OVERLAP_OWNER_SPEC,20,0,2)]}),
                    ),
                    "A and B own kit's work.",
                ),
                then_wait(
                    call(
                        b"amend",
                        &json!({"target":STORY_OWNER,"revision":DIRECTION_REVISION-1,"amendment":{"instructions":DIRECTION_BRIEF,"reason":DIRECTION_REASON}}),
                    ),
                    "A has the new direction.",
                ),
                report(DIRECTION_REPORT),
                wait("The director is waiting for direction or the owners' reports."),
            ]),
        },
        Script {
            cue: Agent::StoryOwner.cue().into(),
            turns: Box::new([
                call(b"delegate", &json!({"batch":[direction_change(forge, X3_SPEC,X3_FILE,2)]})),
                wait("x3's review waits."),
                call(b"cancel", &json!({"target":X3,"reason":X3_CANCEL_REASON})),
                call(
                    b"note",
                    &json!({"scope":{"kind":"project","project":PROJECT},"description":DEFERRED_NOTE,"body":DEFERRED_WORDS,"tags":["deferred"]}),
                ),
                wait("x3's cancellation is settling before a1 starts."),
                call(b"delegate", &json!({"batch":[direction_change(forge, STORY_SPEC,B_FILE,2)]})),
                wait("a1 is delegated."),
                direction_held(STORY_RECALL_HOLD, call(b"recall", &json!({"kind":"name","name":DEFERRED_NOTE_NUMBER}))),
                report(STORY_OWNER_REPORT),
            ]),
        },
        Script {
            cue: Agent::OverlapOwner.cue().into(),
            turns: Box::new([
                direction_held(
                    B_DELEGATE_HOLD,
                    then_wait(
                        call(b"delegate", &json!({"batch":[direction_change(forge, PLANNED_SPEC,B_FILE,1)]})),
                        "h1's planned path waits.",
                    ),
                ),
                then_wait(
                    call(b"cancel", &json!({"target":PLANNED,"reason":PLANNED_CANCEL_REASON})),
                    "h1's cancellation is settling.",
                ),
                report(OVERLAP_OWNER_REPORT),
            ]),
        },
        Script {
            cue: task_cue(Agent::X3Writer.cue()),
            turns: Box::new([
                call(b"write", &json!({"path":X3_FILE,"content":X3_WORDS})),
                call(b"finish", &json!({"form":"change","fields":{"title":"x3","body":"The deferred plan."}})),
            ]),
        },
        Script {
            cue: Agent::PlannedWriter.cue().into(),
            turns: Box::new([direction_held(PLANNED_CODING_HOLD, report("h1 did not produce a change."))]),
        },
        Script {
            cue: task_cue(Agent::StoryWriter.cue()),
            turns: Box::new([
                call(b"edit", &json!({"path":B_FILE,"edits":[{"old":B_WORDS,"new":STORY_WORDS}]})),
                call(b"finish", &json!({"form":"change","fields":{"title":"a1","body":STORY_GREEN}})),
            ]),
        },
        // Each review starts with a fresh transcript. Select x3's by its
        // requester in the system brief; a1's review needs no hold.
        Script {
            cue: format!("[Requested by task {X3}]").into_bytes().into(),
            turns: Box::new([direction_held(X3_REVIEW_HOLD, verdict())]),
        },
        Script { cue: format!("[Requested by task {STORY_CHANGE}]").into_bytes().into(), turns: Box::new([verdict()]) },
    ])
}

/// B's hold can wake the director before the person sends the new direction.
/// Follow delivered input so notices cannot consume the amendment or report.
#[must_use]
pub fn step_5_4_rules() -> Box<[Box<[Rule]>]> {
    use super::{
        DEFERRED_NOTE_NUMBER, DIRECTION_BRIEF, DIRECTION_MESSAGE, DIRECTION_REVISION, OVERLAP_OWNER, STORY_CHANGE,
        STORY_OWNER, STORY_OWNER_REPORT, X3,
    };
    let director = [
        (STORY_OWNER_REPORT, 2),
        (CROSSED, 2),
        (&format!("Done: task {STORY_OWNER} is at spec revision {DIRECTION_REVISION}"), 3),
        (DIRECTION_MESSAGE, 1),
        ("Held task needs a decision", 3),
        (&format!("{DELEGATED}{STORY_OWNER}, {OVERLAP_OWNER}"), 3),
        (BEGIN, 0),
        ("", 3),
    ]
    .map(|(cue, turn)| Rule { cue: cue.as_bytes().into(), turn });
    let owner = [
        (&format!("Note {DEFERRED_NOTE_NUMBER} revision 1: ") as &str, 8),
        // Landing news may cross the report after the note has been recalled.
        (CROSSED, 8),
        (&format!("Result from task {STORY_CHANGE}:"), 7),
        (&format!("Result from task {X3}:"), 5),
        (&format!("Note {DEFERRED_NOTE_NUMBER} revision 1 written · 1 lines"), 4),
        ("Done", 3),
        (DIRECTION_BRIEF, 2),
        (&format!("{DELEGATED}{STORY_CHANGE}"), 6),
        (&format!("{DELEGATED}{X3}"), 1),
        (BEGIN, 0),
        ("", 1),
    ]
    .map(|(cue, turn)| Rule { cue: cue.as_bytes().into(), turn });
    Box::new([
        Box::new(director),
        Box::new(owner),
        Box::new([]),
        Box::new([]),
        Box::new([]),
        Box::new([]),
        Box::new([]),
        Box::new([]),
    ])
}

fn batch_member(agent: Agent, spend: u64, tasks: u32, depth: u32, deadline: u64) -> Value {
    json!({
        "executor":{"kind":"agent","charter":1},
        "spec":{"words":String::from_utf8_lossy(agent.cue()),"parameters":[]},
        "contract":{"kind":"report","words":128},
        "authority":{"tools":if depth == 0 {0} else {3},"grants":[],"notes":0,
            "delegation":{"kinds":[{"kind":"agent","number":1}],"tasks":tasks,"depth":depth},
            "budget":{"spend":spend,"deadline":deadline}}
    })
}
fn batch_held(hold: &[u8], mut turn: Turn) -> Turn {
    use skein_fake_llm_domain::api::Piece;
    let mut lines = vec![Line::Pieces {
        pieces: Box::new([
            Piece { text: super::BATCH_WRITING.as_bytes().into(), hold: None },
            Piece { text: b" Report follows.".as_slice().into(), hold: Some(hold.into()) },
        ]),
    }];
    lines.extend(turn.lines);
    turn.lines = lines.into();
    turn
}

// Act 3, step 6.5 (testing/reference-case.md)
#[must_use]
pub fn step_6_5(_forge: SocketAddr) -> Box<[Script]> {
    use super::{
        A_DEADLINE, A1_DEADLINE, BUDGET_A5, BUDGET_A5_REPORT, BUDGET_ADVISOR_REPORT, BUDGET_ADVISOR_SPEC,
        BUDGET_AMEND_HOLD, BUDGET_ANSWER, BUDGET_DIRECTOR, BUDGET_DIRECTOR_REPORT, BUDGET_FINDING, BUDGET_HIGH_HOLD,
        BUDGET_HIGH_NUMBER, BUDGET_MEDIUM_NUMBER, BUDGET_OWNER, BUDGET_OWNER_REPORT, BUDGET_QUESTION,
        BUDGET_QUESTION_NUMBER, BUDGET_REASON, BUDGET_REVISION, BUDGET_SUBOWNER, BUDGET_SUBOWNER_REPORT,
        BUDGET_UP_QUESTION, DEFAULT_MODEL_NUMBER, OWNER_EFFORT_NUMBER, OWNER_MODEL_NUMBER,
    };
    let mut owner = batch_member(Agent::BudgetOwner, 35, 3, 2, A_DEADLINE);
    owner["authority"]["tools"] = (7 | 32).into();
    owner["tracked"] = json!({"priority":1});
    let mut subowner = batch_member(Agent::BudgetSubowner, 25, 2, 1, A1_DEADLINE);
    subowner["authority"]["tools"] = (7 | 32).into();
    let mut a5 = batch_member(Agent::BudgetInvestigator, 10, 0, 0, A1_DEADLINE);
    a5["authority"]["tools"] = (2 | 32).into();
    a5["llm"] = json!({"main":{"model":DEFAULT_MODEL_NUMBER,"effort":BUDGET_MEDIUM_NUMBER}});
    let mut advisor = batch_member(Agent::BudgetAdvisor, 9, 0, 0, A1_DEADLINE);
    advisor["spec"]["words"] = BUDGET_ADVISOR_SPEC.into();
    advisor["llm"] = json!({"main":{"model":OWNER_MODEL_NUMBER,"effort":OWNER_EFFORT_NUMBER}});
    Box::new([
        Script {
            cue: Agent::BudgetDirector.cue().into(),
            turns: Box::new([
                then_wait(call(b"delegate", &json!({"batch":[owner]})), "A owns the budget question."),
                report(BUDGET_DIRECTOR_REPORT),
                wait("The person's inbox holds A's question; A is still working."),
                wait("A has the person's answer."),
            ]),
        },
        Script {
            cue: Agent::BudgetOwner.cue().into(),
            turns: Box::new([
                then_wait(call(b"delegate", &json!({"batch":[subowner]})), "A1 owns the investigation."),
                then_wait(
                    call(b"message", &json!({"target":BUDGET_DIRECTOR,"form":"question","words":BUDGET_UP_QUESTION})),
                    "The budget awaits the person's answer.",
                ),
                then_wait(
                    call(
                        b"message",
                        &json!({"target":BUDGET_SUBOWNER,"form":"answer","question":BUDGET_QUESTION_NUMBER,"words":BUDGET_ANSWER}),
                    ),
                    "A1 has the person's direction.",
                ),
                report(BUDGET_OWNER_REPORT),
                wait("A1 is still working on the cost."),
            ]),
        },
        Script {
            cue: Agent::BudgetSubowner.cue().into(),
            turns: Box::new([
                then_wait(
                    call(b"message", &json!({"target":BUDGET_OWNER,"form":"question","words":BUDGET_QUESTION})),
                    "A's budget decision waits.",
                ),
                then_wait(call(b"delegate", &json!({"batch":[a5]})), "a5 investigates at medium."),
                batch_held(
                    BUDGET_AMEND_HOLD,
                    then_wait(
                        call(
                            b"amend",
                            &json!({"target":BUDGET_A5,"revision":BUDGET_REVISION-1,"amendment":{"llm":{"main":{"model":DEFAULT_MODEL_NUMBER,"effort":BUDGET_HIGH_NUMBER}},"reason":BUDGET_REASON}}),
                        ),
                        "a5 will try again at high.",
                    ),
                ),
                then_wait(
                    call(b"delegate", &json!({"batch":[advisor]})),
                    "The advisor reads a5's report and evidence.",
                ),
                report(BUDGET_SUBOWNER_REPORT),
                wait("The investigation or advisor is still working."),
            ]),
        },
        Script {
            cue: Agent::BudgetInvestigator.cue().into(),
            turns: Box::new([
                call(b"subscribe", &json!({"kind":"timer","at":u64::MAX})),
                then_wait(
                    call(b"message", &json!({"target":BUDGET_SUBOWNER,"form":"words","words":BUDGET_FINDING})),
                    "a5 awaits a next try.",
                ),
                batch_held(BUDGET_HIGH_HOLD, report(BUDGET_A5_REPORT)),
                wait("a5 awaits the amendment's reason."),
            ]),
        },
        Script { cue: Agent::BudgetAdvisor.cue().into(), turns: Box::new([report(BUDGET_ADVISOR_REPORT)]) },
    ])
}

#[must_use]
pub fn step_6_5_rules() -> Box<[Box<[Rule]>]> {
    use super::{
        BUDGET_A5_REPORT, BUDGET_ADVISOR_REPORT, BUDGET_ANSWER, BUDGET_FINDING, BUDGET_OWNER_REPORT,
        BUDGET_PERSON_ANSWER, BUDGET_QUESTION, BUDGET_REASON, BUDGET_SUBOWNER_REPORT, BUDGET_TIMER_NUMBER,
        BUDGET_UP_QUESTION,
    };
    let table = |entries: &[(&str, u32)]| -> Box<[Rule]> {
        entries.iter().map(|(cue, turn)| Rule { cue: cue.as_bytes().into(), turn: *turn }).collect()
    };
    Box::new([
        table(&[
            (BUDGET_OWNER_REPORT, 1),
            (CROSSED, 1),
            ("Person answered task", 3),
            (BUDGET_UP_QUESTION, 2),
            (MESSAGE, 2),
            (DELEGATED, 2),
            ("Held task needs a decision", 2),
            (BEGIN, 0),
            ("", 2),
        ]),
        table(&[
            (BUDGET_SUBOWNER_REPORT, 3),
            (CROSSED, 3),
            (BUDGET_PERSON_ANSWER, 2),
            (BUDGET_QUESTION, 1),
            (MESSAGE, 4),
            (DELEGATED, 4),
            ("Held task needs a decision", 4),
            (BEGIN, 0),
            ("", 4),
        ]),
        table(&[
            (BUDGET_ADVISOR_REPORT, 4),
            (CROSSED, 4),
            (BUDGET_A5_REPORT, 3),
            (BUDGET_FINDING, 2),
            (BUDGET_ANSWER, 1),
            (MESSAGE, 5),
            (DELEGATED, 5),
            ("Held task needs a decision", 5),
            (BEGIN, 0),
            ("", 5),
        ]),
        table(&[
            (BUDGET_REASON, 2),
            (CROSSED, 2),
            (MESSAGE, 3),
            ("Held task needs a decision", 3),
            (&format!("Subscription {BUDGET_TIMER_NUMBER} added"), 1),
            (BEGIN, 0),
            ("", 3),
        ]),
        Box::new([]),
    ])
}

// Act 3, step 6.4 (testing/reference-case.md)
#[must_use]
pub fn step_6_4(_forge: SocketAddr) -> Box<[Script]> {
    use super::{
        Agent, RED_BASE, RED_BROKEN, RED_FILE, RED_FIRST_HEAD, RED_REPAIR, RED_REPAIR_HOLD, RED_REVIEW_HOLD,
        RED_UPDATED_HEAD, REVIEWED,
    };
    let verdict = || call(b"finish", &json!({"form":"verdict","label":"1","text":REVIEWED,"fields":{},"items":[]}));
    Box::new([
        Script { cue: review_cue(RED_FIRST_HEAD), turns: Box::new([direction_held(RED_REVIEW_HOLD, verdict())]) },
        Script {
            cue: task_cue(Agent::RedRepairCoder.cue()),
            turns: Box::new([direction_held(
                RED_REPAIR_HOLD,
                finish_change(
                    call(b"edit", &json!({"path":RED_FILE,"edits":[{"old":RED_BROKEN,"new":RED_BASE}]})),
                    "Repair kit CI",
                    "Restore the green cue.",
                ),
            )]),
        },
        Script { cue: format!("[Requested by task {RED_REPAIR}]").into_bytes().into(), turns: Box::new([verdict()]) },
        Script { cue: review_cue(RED_UPDATED_HEAD), turns: Box::new([verdict()]) },
    ])
}

#[must_use]
pub fn step_6_4_rules() -> Box<[Box<[Rule]>]> {
    Box::new([Box::new([]), Box::new([]), Box::new([]), Box::new([])])
}

// A review now includes the change's words before its requester and head.
// Select a producer by the start of its task, so its cue in a review's
// quoted change cannot hide the review script's later, more specific cue.
fn task_cue(words: &[u8]) -> Box<[u8]> {
    [b"## Task\n\n".as_slice(), words].concat().into()
}

fn review_cue(head: u64) -> Box<[u8]> {
    use std::fmt::Write;
    let mut hex = String::new();
    for byte in &skein_fake_forge_domain::object_bytes(head)[..20] {
        write!(hex, "{byte:02x}").expect("head hex");
    }
    format!("Pull request at head {hex}").into_bytes().into()
}

fn finish_change(mut work: Turn, title: &str, body: &str) -> Turn {
    let mut lines = work.lines.into_vec();
    lines.extend(call(b"finish", &json!({"form":"change","fields":{"title":title,"body":body}})).lines);
    work.lines = lines.into();
    work
}

// Act 3, step 6.6 (testing/reference-case.md)
#[must_use]
pub fn step_6_6(forge: SocketAddr) -> Box<[Script]> {
    use super::waiting::{APP_ID, E1_SPEC, FILE, FIXED, KIT_ID, PIN, REPIN_HOLD, REPIN_SPEC, REPORT, pin_input};
    let e1 = repository_change(forge, KIT_ID, E1_SPEC, FILE, 2);
    let mut repin = repository_change(forge, APP_ID, REPIN_SPEC, PIN, 2);
    repin["dependencies"] = json!([{"kind":"batch","number":0}]);
    Box::new([
        Script {
            cue: Agent::WaitingStarter.cue().into(),
            turns: Box::new([
                then_wait(call(b"delegate", &json!({"batch":[e1,repin]})), "Both changes delegated."),
                report(REPORT),
                wait("Waiting for app to land."),
            ]),
        },
        Script {
            cue: task_cue(Agent::WaitingKitCoder.cue()),
            turns: Box::new([finish_change(
                call(b"write", &json!({"path":FILE,"content":FIXED})),
                "e1",
                "Fix the fake.",
            )]),
        },
        Script {
            cue: task_cue(Agent::WaitingAppCoder.cue()),
            turns: Box::new([direction_held(
                REPIN_HOLD,
                finish_change(
                    call(b"write", &json!({"path":PIN,"content":pin_input()})),
                    "Repin kit",
                    "Use e1's landed kit commit.",
                ),
            )]),
        },
        Script {
            cue: Agent::Review.cue().into(),
            turns: Box::new([call(
                b"finish",
                &json!({"form":"verdict","label":"1","text":REVIEWED,"fields":{},"items":[]}),
            )]),
        },
    ])
}

#[must_use]
pub fn step_6_6_rules() -> Box<[Box<[Rule]>]> {
    use super::waiting::REPIN;
    Box::new([
        Box::new([
            Rule { cue: format!("Result from task {REPIN}:").into_bytes().into(), turn: 1 },
            Rule { cue: CROSSED.as_bytes().into(), turn: 1 },
            Rule { cue: DELEGATED.as_bytes().into(), turn: 2 },
            Rule { cue: BEGIN.as_bytes().into(), turn: 0 },
            Rule { cue: Box::new([]), turn: 2 },
        ]),
        Box::new([]),
        Box::new([]),
        Box::new([]),
    ])
}

// Act 3, step 6.8 (testing/reference-case.md).
#[must_use]
pub fn step_6_8(forge: SocketAddr) -> Box<[Script]> {
    use super::qa::{
        CHANGE_SPEC, DIRECTOR_REPORT, OWNER_REPORT, OWNER_SPEC, QA_TEST, QA_TEST_PATH, SUBOWNER_REPORT, SUBOWNER_SPEC,
        T, TEST_REPORT, TESTER_SPEC,
    };
    let mut owner = direction_owner(forge, OWNER_SPEC, 80, 0, 6);
    owner["authority"]["tools"] = 3.into();
    owner["authority"]["delegation"]["depth"] = 4.into();
    let mut subowner = direction_owner(forge, SUBOWNER_SPEC, 50, 0, 4);
    subowner.as_object_mut().expect("subowner").remove("tracked");
    subowner["authority"]["tools"] = 3.into();
    let mut tester = batch_member(Agent::QaTester, 10, 0, 0, super::A1_DEADLINE);
    tester["spec"]["words"] = TESTER_SPEC.into();
    let mut change = direction_change(forge, CHANGE_SPEC, QA_TEST_PATH, 2);
    change["spec"]["inputs"] = json!([T]);
    Box::new([
        Script {
            cue: Agent::QaDirector.cue().into(),
            turns: Box::new([
                then_wait(call(b"delegate", &json!({"batch":[owner]})), "A owns QA."),
                report(DIRECTOR_REPORT),
                wait("A is still working."),
            ]),
        },
        Script {
            cue: Agent::QaOwner.cue().into(),
            turns: Box::new([
                then_wait(call(b"delegate", &json!({"batch":[subowner]})), "A1 owns QA."),
                report(OWNER_REPORT),
                wait("A1's QA continues."),
            ]),
        },
        Script {
            cue: Agent::QaSubowner.cue().into(),
            turns: Box::new([
                then_wait(call(b"delegate", &json!({"batch":[tester]})), "T tests M1."),
                then_wait(call(b"delegate", &json!({"batch":[change]})), "c lands T's test."),
                report(SUBOWNER_REPORT),
                wait("QA's change is still working."),
            ]),
        },
        Script { cue: Agent::QaTester.cue().into(), turns: Box::new([report(TEST_REPORT)]) },
        Script {
            cue: Agent::QaCoder.cue().into(),
            turns: Box::new([finish_change(
                call(b"write", &json!({"path":QA_TEST_PATH,"content":QA_TEST})),
                "M1 QA",
                "Land T's regression test.",
            )]),
        },
        Script {
            cue: Agent::Review.cue().into(),
            turns: Box::new([call(
                b"finish",
                &json!({"form":"verdict","label":"1","text":REVIEWED,"fields":{},"items":[]}),
            )]),
        },
    ])
}

#[must_use]
pub fn step_6_8_rules() -> Box<[Box<[Rule]>]> {
    use super::qa::{C, OWNER_REPORT, SUBOWNER_REPORT, T};
    let table = |entries: &[(&str, u32)]| -> Box<[Rule]> {
        entries.iter().map(|(cue, turn)| Rule { cue: cue.as_bytes().into(), turn: *turn }).collect()
    };
    Box::new([
        table(&[(OWNER_REPORT, 1), (CROSSED, 1), (DELEGATED, 2), (BEGIN, 0), ("", 2)]),
        table(&[(SUBOWNER_REPORT, 1), (CROSSED, 1), (DELEGATED, 2), (BEGIN, 0), ("", 2)]),
        table(&[
            (&format!("Result from task {C}:"), 2),
            (CROSSED, 2),
            (&format!("Result from task {T}:"), 1),
            (DELEGATED, 3),
            (BEGIN, 0),
            ("", 3),
        ]),
        Box::new([]),
        Box::new([]),
        Box::new([]),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use skein_fake_llm_domain::{self as fake, api};
    use skein_lib::{Env, Queue, ReplyTo, Time, Token, Wall};

    fn answer(scripts: Box<[Script]>, rules: Box<[Box<[Rule]>]>, agent: Agent, fresh: &str) -> api::Answer {
        let mut recipe = crate::llm::played_by_input(scripts, rules);
        let message =
            |role, words: &[u8]| api::Message { role, parts: Box::new([api::Part::Text { text: words.into() }]) };
        let query = api::Query {
            credential: None,
            caching: api::Caching::Unscoped,
            model: b"fake".as_slice().into(),
            system: agent.cue().into(),
            tools: ["finish", "wait"]
                .map(|name| api::ToolSpec {
                    name: name.as_bytes().into(),
                    description: Box::new([]),
                    parameters: b"{}".as_slice().into(),
                })
                .into(),
            choice: api::ToolChoice::Auto,
            messages: Box::new([
                message(api::Role::User, CROSSED.as_bytes()),
                message(api::Role::Assistant, b"The preceding feedback has been read."),
                message(api::Role::User, fresh.as_bytes()),
            ]),
            max_tokens: 4096,
        };
        let mut env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: recipe.domain_limits };
        let mut out = Queue::uncapped();
        fake::step(
            &mut recipe.domain,
            &env,
            fake::Event::Call { reply_to: ReplyTo::new(Token::new(1)), call: Token::new(1), query },
            &mut out,
        );
        loop {
            if let Some(request) = out.pop() {
                let fake::Request::Reply { result, .. } = request else { panic!("unheld script reply") };
                return result.expect("script accepted the query");
            }
            env.now = recipe.domain.next_deadline().expect("a reply is scheduled");
            fake::fire(&mut recipe.domain, &env, &mut out);
        }
    }

    fn check_finish_feedback(scripts: &[Script], rules: &[Box<[Rule]>], agent: Agent, report: &str) {
        let crossed = format!("{CROSSED} Read them before finishing.\n2: forge landing 910a2dec -> 6e73e372");
        for (fresh, finishes) in [("2: forge landing 910a2dec -> 975835de", false), (crossed.as_str(), true)] {
            let reply = answer(scripts.into(), rules.into(), agent, fresh);
            assert_eq!(reply.finish, Finish::ToolCalls);
            let calls: Vec<_> = reply
                .parts
                .iter()
                .filter_map(|part| match part {
                    api::Part::ToolCall { name, arguments, .. } => Some((name.as_ref(), arguments.as_ref())),
                    api::Part::Opaque { .. } | api::Part::Text { .. } | api::Part::ToolOutput { .. } => None,
                })
                .collect();
            assert_eq!(calls.len(), 1, "one scripted judgement");
            let (name, arguments) = calls[0];
            let arguments: Value = serde_json::from_slice(arguments).expect("tool arguments");
            if finishes {
                assert_eq!(name, b"finish", "new feedback must complete the final report");
                assert_eq!(arguments, json!({"form":"report","text":report,"fields":{}}));
            } else {
                assert_eq!(name, b"wait", "ordinary fresh landing must not replay an older crossed finish");
                assert_eq!(arguments, json!({}));
            }
        }
    }

    #[test]
    fn case_4_3_reads_messages_beside_working_results_before_waiting() {
        use super::super::{B_REPORT, BENCHMARK, CHECK, CHECK_REASON, CHECK_REPORT};
        let cases = [
            (Agent::Check, "Subscription 12 added".to_owned(), "wait", json!({})),
            (
                Agent::Check,
                format!("Subscription 12 added\n{CHECK}: \n"),
                "cancel",
                json!({"target":BENCHMARK,"reason":CHECK_REASON}),
            ),
            (Agent::Check, format!("{CHECK}: \n"), "cancel", json!({"target":BENCHMARK,"reason":CHECK_REASON})),
            (
                Agent::Check,
                format!("Done\n{B_REPORT}"),
                "finish",
                json!({"form":"report","text":CHECK_REPORT,"fields":{}}),
            ),
            (Agent::Check, "Done".to_owned(), "wait", json!({})),
            (
                Agent::B,
                format!("Tasks introduced\n{CHECK_REASON}"),
                "finish",
                json!({"form":"report","text":B_REPORT,"fields":{}}),
            ),
        ];
        for (agent, fresh, tool, arguments) in cases {
            let reply = answer(step_4_1_4_3(([127, 0, 0, 1], 3000).into()), step_4_1_4_3_rules(), agent, &fresh);
            let calls: Vec<_> = reply
                .parts
                .iter()
                .filter_map(|part| match part {
                    api::Part::ToolCall { name, arguments, .. } => Some((name.as_ref(), arguments.as_ref())),
                    api::Part::Opaque { .. } | api::Part::Text { .. } | api::Part::ToolOutput { .. } => None,
                })
                .collect();
            assert_eq!(calls.len(), 1, "one judgement for {agent:?}: {fresh}");
            assert_eq!(calls[0].0, tool.as_bytes(), "{agent:?}: {fresh}");
            assert_eq!(
                serde_json::from_slice::<Value>(calls[0].1).expect("arguments"),
                arguments,
                "{agent:?}: {fresh}"
            );
        }
    }

    #[test]
    fn case_5_4_owner_finishes_after_crossed_report_and_waits_on_fresh_landing() {
        check_finish_feedback(
            &step_5_4(([127, 0, 0, 1], 3000).into()),
            &step_5_4_rules(),
            Agent::StoryOwner,
            super::super::STORY_OWNER_REPORT,
        );
    }
}
