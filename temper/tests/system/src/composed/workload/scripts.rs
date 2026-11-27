//! Name-cued scripts: task numbers are only resolved after the named tasks exist.
use super::super::call;
use super::{Goal, Script, World, engine};
use serde_json::{Value, json};
use temper_workload::draw::Change;

pub(super) fn goal_cue(goal: &Goal) -> String {
    format!("@goal:{}@", goal.name)
}
pub(super) fn change_cue(change: &Change) -> String {
    format!("@change:{}@", change.name)
}

pub(super) fn idle() -> Script {
    Script { cue: b"@unassigned@".as_slice().into(), turns: Box::default() }
}
fn turn(name: &[u8], value: &Value) -> skein_fake_llm_domain::api::Turn {
    call(name, &serde_json::to_vec(value).expect("script JSON"))
}

pub(super) fn continuing(world: &World, task: u64) -> bool {
    world.lanes[0].assignment.as_ref().is_some_and(|previous| previous.task == task) || world.parked.contains_key(&task)
}

pub(super) fn assigned(world: &World, assignment: &engine::Assignment) -> Script {
    if continuing(world, assignment.task) {
        return Script { cue: Box::default(), turns: Box::default() };
    }
    let row = world.root.tasks().task(assignment.task).expect("assigned task");
    let words = String::from_utf8_lossy(&row.spec.words);
    if words.starts_with("@chat:questions@") {
        return Script { cue: b"@chat:questions@".as_slice().into(), turns: Box::new([call(b"wait", b"{}")]) };
    }
    for goal in &world.workload().goals {
        if words.starts_with(&format!("@chat:{}@", goal.name)) {
            return Script {
                cue: format!("@chat:{}@", goal.name).into_bytes().into(),
                turns: Box::new([
                    turn(b"goal", &json!({"words":format!("{} Implement the train goal.", goal_cue(goal))})),
                    call(b"wait", b"{}"),
                ]),
            };
        }
        if words.starts_with(&goal_cue(goal)) {
            let changes: Vec<_> = goal.changes.iter().map(|change| {
                // The person's amendment adds already-landed cross-goal
                // dependencies; this call can name only the goal's own delegates.
                let after: Vec<_> = change.after.iter().filter(|name| goal.changes.iter().any(|change| &change.name == *name)).map(|name| json!(name)).collect();
                json!({"words":format!("{} Write this change's files.", change_cue(change)),
                    "name":change.name,"landing":if goal.train { "main" } else { "goal" },"after":after,"questions":"up"})
            }).collect();
            let mut turns = Vec::new();
            if goal.train {
                let person = match goal.party {
                    super::Party::First => 1,
                    super::Party::Second => 2,
                };
                turns.push(person_task(person, &format!("@latch:{}@ Release the ready train.", goal.name)));
            }
            turns.extend([
                turn(b"change", &json!({"changes":changes})),
                call(b"wait", b"{}"),
                turn(b"finish", &json!({"report":format!("{} complete", goal.name)})),
            ]);
            return Script { cue: goal_cue(goal).into_bytes().into(), turns: turns.into() };
        }
        for change in &goal.changes {
            let failed = format!("Failed test: {} ", change.name);
            if words.contains(&failed) {
                let mut script = producer(change, 0, true, false);
                script.cue = failed.into_bytes().into();
                return script;
            }
            if words.contains(&change_cue(change)) {
                let change_task = world.workload_names().task(&change.name).expect("change named before producer");
                let repair = world.store.change(change_task).expect("change").change.repairs > 0;
                let person = match goal.party {
                    super::Party::First => 1,
                    super::Party::Second => 2,
                };
                return producer(change, person, repair, world.workload_parks(assignment.task));
            }
        }
    }
    panic!("no logical name in task {} {:?}: {words}", assignment.task, row.executor);
}

fn person_task(person: u64, words: &str) -> skein_fake_llm_domain::api::Turn {
    turn(
        b"delegate",
        &json!({"batch":[{
            "executor":{"kind":"person","person":person}, "spec":{"words":words},
            "contract":{"kind":"report","words":128},
            "authority":{"tools":0,"grants":[],"delegation":{"kinds":[],"tasks":0,"depth":0},"budget":{"spend":0},"notes":0}
        }]}),
    )
}

fn producer(change: &Change, person: u64, repair: bool, park: bool) -> Script {
    let mut turns = Vec::new();
    if change.asks && !repair {
        turns.push(person_task(person, &format!("@ask:{}@ Write the specified files? I recommend yes.", change.name)));
    }
    if !repair {
        for file in &change.files {
            turns.push(turn(b"write", &json!({"path":file.path,"content":file.text})));
        }
    }
    turns.push(turn(b"read", &json!({"path":super::setup::marker(change)})));
    turns.push(turn(b"write", &json!({"path":super::setup::marker(change),"content":if change.red_once && !repair { format!("{}\n", change.name) } else { String::new() }})));
    let fields = json!({"title":change.name,"body":format!("Deliver {}", change.name)});
    turns.push(turn(b"deliver", &json!({"fields":fields})));
    if park {
        turns.push(person_task(person, &format!("@park:{}@ Finish the delivered change?", change.name)));
        turns.push(call(b"wait", b"{}"));
    }
    turns.push(turn(b"finish", &json!({"outcome":{"form":"change","fields":fields}})));
    Script { cue: change_cue(change).into_bytes().into(), turns: turns.into() }
}
