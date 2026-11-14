//! Narrowing and conflict departures rejoin after their first turn (domain/landing.md, 4.3 and 9).
use super::{Drill, Judge, Line, Process, Script, Value, call, json};

pub(super) fn scripts(scripts: &mut Vec<Script>, drill: Drill) {
    if drill == Drill::Moving {
        let producer = scripts
            .iter_mut()
            .find(|script| script.cue.starts_with(b"## Task\n\n@drill-code-2"))
            .expect("moving producer");
        let mut held = call(b"read", &json!({"path":"walk/line-2"}));
        held.lines = [vec![super::super::batch_landing::hold(b"drill-moving")], held.lines.to_vec()].concat().into();
        let finish = producer.turns[2].clone();
        let Line::Call { arguments, .. } = &finish.lines[0] else { panic!("finish") };
        let fields: Value = serde_json::from_slice(arguments).expect("change fields");
        producer.turns = vec![
            producer.turns[0].clone(),
            producer.turns[1].clone(),
            call(b"deliver", &json!({"fields":fields["fields"]})),
            held,
            call(b"write", &json!({"path":"walk/line-2","content":"Changed 2\nMoved during proof\n"})),
            finish,
        ]
        .into();
        return;
    }
    let (cue, path) = if drill == Drill::Breaking {
        (b"## Task\n\nRepair the failed landing gate".as_slice(), "walk/red")
    } else {
        (b"## Task\n\nResolve the merge conflict".as_slice(), "walk/line-1")
    };
    if drill == Drill::Breaking {
        let producer = scripts
            .iter_mut()
            .find(|script| script.cue.starts_with(b"## Task\n\n@drill-code-3"))
            .expect("third producer");
        let mut turns = producer.turns.to_vec();
        turns.insert(2, call(b"write", &json!({"path":"walk/red","content":"red\n"})));
        producer.turns = turns.into();
    } else {
        let producer = scripts
            .iter_mut()
            .find(|script| script.cue.starts_with(b"## Task\n\n@drill-code-2"))
            .expect("conflicting producer");
        producer.turns[0] = call(b"read", &json!({"path":"walk/line-1"}));
        producer.turns[1] = call(b"write", &json!({"path":"walk/line-1","content":"Changed 2\n"}));
    }
    let mut read = call(b"read", &json!({"path":path}));
    read.lines = [vec![super::super::batch_landing::hold(b"drill-repair")], read.lines.to_vec()].concat().into();
    scripts.push(Script {cue:cue.into(),turns:Box::new([
        read,
        if drill==Drill::Breaking {call(b"remove",&json!({"path":path}))} else {call(b"write",&json!({"path":path,"content":"Changed 1\nResolved 2\n"}))},
        call(b"finish",&json!({"form":"change","fields":{"title":if drill==Drill::Breaking {"Drill change 3"} else {"Drill change 2"},"body":"Repair the departing member."}})),
    ])});
}
impl Judge<'_> {
    pub(super) fn departure(&mut self, procs: &mut [Process], panel: &Value) -> Result<(), String> {
        let members = panel["members"].as_array().ok_or("members")?;
        let index = if self.drill == Drill::Breaking { 2 } else { 1 };
        let member = members.get(index).ok_or("departing member")?;
        if !member["left"].is_null() {
            let expected = if self.drill == Drill::Breaking {
                member["left"]["blamed"].is_object()
            } else if self.drill == Drill::Moving {
                member["left"]["head_moved"]["head"].is_string()
            } else {
                member["left"] == "earlier_conflict" || member["left"] == "conflict"
            };
            if !expected {
                return Err(format!("named departure: {member}"));
            }
            self.batch.departed = true;
            let below = if self.drill == Drill::Breaking { 2 } else { 1 };
            if members.iter().take(below).all(|member| !member["landed"].is_null()) {
                self.batch.green_prefix = true;
                for proc in procs {
                    if let Process::Peer(peer) = proc {
                        peer.release(b"drill-repair").map_err(|why| format!("repair release: {why:?}"))?;
                    }
                }
            }
        }
        Ok(())
    }
}
