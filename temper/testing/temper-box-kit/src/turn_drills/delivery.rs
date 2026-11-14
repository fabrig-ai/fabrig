//! An in-flight producer across an engine and worker restart (domain/landing.md, 15).
use super::{Exit, Host, Judge, Line, Process, Script, Signal, Stage, Value, binary, call, fs, json};
use skein_fake_llm_domain::api::Part;

pub(super) fn scripts(scripts: &mut [Script]) {
    let goal = scripts.iter_mut().find(|script| script.cue.as_ref() == b"@drill-goal-1").expect("goal");
    let Line::Call { arguments, .. } = &goal.turns[0].lines[0] else { panic!("delegate") };
    let mut batch: Value = serde_json::from_slice(arguments).expect("batch");
    let mut second = batch["batch"][0].clone();
    second["spec"]["words"] = "@drill-code-2 Edit line 2".into();
    batch["batch"].as_array_mut().expect("batch").push(second);
    goal.turns[0] = call(b"delegate", &batch);
    let producer =
        scripts.iter_mut().find(|script| script.cue.starts_with(b"## Task\n\n@drill-code-2")).expect("second producer");
    let mut first = call(b"read", &json!({"path":"walk/line-2"}));
    first.lines = [vec![super::super::batch_landing::hold(b"drill-second")], first.lines.to_vec()].concat().into();
    let mut edit = call(b"write", &json!({"path":"walk/line-2","content":"Saved work\nDelivered work\n"}));
    edit.lines = [vec![super::super::batch_landing::hold(b"drill-saved")], edit.lines.to_vec()].concat().into();
    producer.turns = Box::new([
        first,
        call(b"write", &json!({"path":"walk/line-2","content":"Saved work\n"})),
        call(b"save", &json!({})),
        edit,
        call(b"deliver", &json!({"fields":{"title":"Drill change 2","body":"Deliver saved work."}})),
        call(b"finish", &json!({"form":"change","fields":{"title":"Drill change 2","body":"Finish saved work."}})),
    ]);
}
fn release(procs: &mut [Process], name: &[u8]) -> Result<(), String> {
    for proc in procs {
        if let Process::Peer(peer) = proc {
            peer.release(name).map_err(|why| format!("release: {why:?}"))?;
        }
    }
    Ok(())
}
fn saved(procs: &[Process]) -> bool {
    procs.iter().any(|proc| {
        let Process::Peer(peer) = proc else { return false };
        peer.observations().iter().any(|event| {
            let skein_fake_peers::llm::Observation::Query { query, .. } = event else { return false };
            let parts: Vec<&Part> = query.messages.iter().flat_map(|message| message.parts.iter()).collect();
            parts.iter().any(|part| {
                let Part::ToolCall { id, name, .. } = part else { return false };
                name.as_ref() == b"save"
                    && parts
                        .iter()
                        .any(|part| matches!(part,Part::ToolOutput {id:answer,is_error:false,..} if answer==id))
            })
        })
    })
}
impl Judge<'_> {
    pub(super) fn delivery(&mut self, procs: &mut [Process], panel: &Value) -> Result<(), String> {
        if panel["phase"] != "proving" {
            return Ok(());
        }
        let Some(run) = self.barrier.waiting(None) else { return Ok(()) };
        self.run = Some(run);
        release(procs, b"drill-second")?;
        if saved(procs) {
            if !self.delivery_armed {
                super::super::write_private(&self.root.join("delivery.hold"), b"");
                self.delivery_armed = true;
            }
            release(procs, b"drill-saved")?;
        }
        if self.root.join("delivery.entered").exists() {
            eprintln!("drill 6: saved producer delivering while its goal's earlier change proves: {panel}");
            binary(procs, 3).signal(Signal::Terminate);
            fs::remove_file(self.root.join("delivery.hold")).map_err(|why| why.to_string())?;
            self.stage = Stage::WorkerStopping;
        }
        Ok(())
    }
    pub(super) fn stopped_worker(&mut self, procs: &mut [Process]) -> Result<(), String> {
        if !procs[3].is_empty() {
            return Ok(());
        }
        if binary(procs, 3).exit_status() != Some(Exit::Code(0)) {
            return Err(format!("worker stop: {}", super::super::without_secrets(binary(procs, 3).stderr())));
        }
        binary(procs, 0).signal(Signal::Kill);
        self.stage = Stage::Restart;
        Ok(())
    }
    pub(super) fn saved_result(&self) -> Result<(), String> {
        let file = self.forge.repository_observation("acme", "widget", "contents/walk/line-2?ref=main");
        let encoded: Vec<_> =
            file["content"].as_str().ok_or("saved file")?.bytes().filter(|byte| !byte.is_ascii_whitespace()).collect();
        let bytes = skein_lib::base64::decode(skein_lib::base64::Alphabet::Standard, &encoded, 1024)
            .map_err(|why| format!("saved file: {why:?}"))?;
        if bytes.as_ref() != b"Saved work\nDelivered work\n" {
            return Err(format!("saved work retained: {}", String::from_utf8_lossy(&bytes)));
        }
        Ok(())
    }
    pub(super) fn remaining_delivery(&mut self, procs: &mut [Process]) -> Result<(), String> {
        let Some(list) = self.read(procs, &["landing", "list", "--project", "1", "--ended"]) else { return Ok(()) };
        if let Some(row) = list["landings"].as_array().ok_or("landings")?.iter().find(|row| row["phase"] != "landed") {
            self.panel = Some(row.clone());
            self.stage = Stage::Resume;
        }
        Ok(())
    }
}
