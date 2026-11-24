//! A unit's mixed stop, sharing the cluster-turn setup (draft/archive/worker-restarts.md, 2 and 9).
use super::{
    Call, Cluster, Exit, Host, Judge, Path, Root, Time, Value, check_calls, committed_reply, fs, json, one, setup,
    support,
};
use skein_lib::Duration;

const CUE: &[u8] = b"## Task\n\n@coding NOTICE";
const REPLY: &str = "The held answer settled.";
const SAVED: &str = "Saved work\nSettled turn\n";
// temper/docs/help/server.md, 5: the unit's timeout is the printed bound plus 34 s.
const UNIT_MARGIN: Duration = Duration::from_secs(34);
const BEFORE_GATE: &str = "Saved work\n";

pub(super) fn prepare(cluster: &mut Cluster) {
    cluster.record_llm();
    let path = cluster.dir("llm").join("llm.json");
    let mut llm = setup::read(&path);
    llm["control"] = serde_json::json!({"socket":"127.0.0.1:0"});
    llm["cued"][0]["turns"] = serde_json::json!([
        {"lines":[{"call":"read","arguments":{"path":"NOTICE"}}],"finish":"tool_calls","tokens":1},
        {"lines":[{"call":"write","arguments":{"path":"NOTICE","content":BEFORE_GATE}}],"finish":"tool_calls","tokens":1},
        {"lines":[
            {"pieces":[{"text":REPLY,"hold":"gate"}]},
            {"call":"write","arguments":{"path":"NOTICE","content":SAVED}}
        ],"finish":"tool_calls","tokens":1}
    ]);
    // The parent parks with its delegate outstanding. Only the producer's agent
    // remains when the worker's tree supplies the stray signal's target.
    llm["asked"][0]["turns"][1] = serde_json::json!({
        "lines":[{"text":"The producer is running."}],"finish":"stop","tokens":1
    });
    setup::write(&path, &llm);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Phase {
    Gate,
    Parent,
    Before,
    Agent,
    Signalled,
    StillLive,
    Released,
    Exit,
    Read,
    Turn,
}
#[derive(Debug)]
pub(super) struct Stop {
    phase: Phase,
    task: String,
    attempt: u64,
    bound_until: Time,
    drain_until: Time,
}
impl Stop {
    pub(super) fn new() -> Self {
        Self { phase: Phase::Gate, task: String::new(), attempt: 0, bound_until: Time::ZERO, drain_until: Time::ZERO }
    }
}
fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    haystack.windows(needle.len()).any(|part| part == needle)
}
fn owner(task: &Value) -> &Value {
    &task["chip"]["details"]["owner"]
}
fn activity(packet: &Value) -> Result<&Vec<Value>, String> {
    crate::complete_cluster_json(packet)?;
    packet["rows"].as_array().ok_or_else(|| format!("activity rows: {packet}"))
}

impl Judge {
    /// The fake has accepted the request for the gated answer. Its preceding
    /// write is in that request's history; without a release it cannot finish.
    fn held_answer_requested(&self) -> Result<bool, String> {
        use skein_fake_llm_domain::api::Part;
        use temper_fake::llm::observations::{Observation, decode};
        let path = self.members.dir("llm").join("observations.jsonl");
        let bytes = match fs::read(path) {
            Ok(bytes) => bytes,
            Err(why) if why.kind() == std::io::ErrorKind::NotFound => return Ok(false),
            Err(why) => return Err(format!("LLM query records: {why}")),
        };
        for line in bytes.split_inclusive(|byte| *byte == b'\n').filter(|line| line.ends_with(b"\n")) {
            match decode(line).map_err(|why| format!("LLM query record: {why:?}"))? {
                Observation::Query(query) => {
                    let parts = || query.messages.iter().flat_map(|message| &message.parts);
                    let producer = contains(&query.system, CUE)
                        || parts().any(|part| matches!(part, Part::Text { text } if contains(text, CUE)));
                    if !producer {
                        continue;
                    }
                    for part in parts() {
                        if let Part::ToolCall { id, name, arguments } = part
                            && name.as_ref() == b"write"
                            && serde_json::from_slice::<Value>(arguments).is_ok_and(|args| {
                                args["path"] == "NOTICE" && args["content"] == BEFORE_GATE
                            })
                            && parts().any(|part| matches!(part, Part::ToolOutput { id: output, is_error: false, .. } if output == id))
                        {
                            return Ok(true);
                        }
                    }
                }
                Observation::Checkpoint { lost: 0 } => {}
                Observation::Checkpoint { lost } => return Err(format!("LLM lost {lost} records")),
                Observation::Full { .. } => return Err("LLM records full".into()),
            }
        }
        Ok(false)
    }
    pub(super) fn advance_unit(&mut self, now: Time, hosts: &mut [Root]) -> Result<(), String> {
        let (client, roots) = hosts.split_last_mut().ok_or("client root")?;
        self.tick = Some(now.saturating_add(Duration::from_millis(20)));
        match self.unit.phase {
            Phase::Gate if self.held_answer_requested()? => {
                self.calls(client, vec![Call::new(&["task", "show", &self.chat, "--json"])])?;
                self.unit.phase = Phase::Parent;
            }
            Phase::Agent => self.stop_unit(now, client, roots)?,
            Phase::Signalled => {
                let worker = self.members.binary("w1", roots).ok_or("worker observer")?;
                if worker.exit_status().is_some() {
                    return Err("worker exited with its answer still held".into());
                }
                let stderr = String::from_utf8_lossy(worker.stderr());
                let prefix = format!("run {} attempt {}: its agent heard SIGTERM ", self.unit.task, self.unit.attempt);
                if stderr.lines().any(|line| line.contains(&prefix) && line.ends_with("; its stop is the worker's"))
                    && client.is_empty()
                {
                    check_calls(client)?;
                    if stderr.contains("Cancelled") {
                        return Err("the stray signal cancelled the held run".into());
                    }
                    self.unit_read(client)?;
                    self.unit.phase = Phase::StillLive;
                }
            }
            Phase::Exit if self.members.ended("w1") => {
                let worker = self.members.binary("w1", roots).ok_or("worker observer")?;
                if now >= self.unit.bound_until
                    || worker.exit_status() != Some(Exit::Code(0))
                    || !worker.tree_status().live_pids.is_empty()
                    || worker.tree_status().live_at_leader_exit.as_ref().is_none_or(|pids| !pids.is_empty())
                    || worker.tree_status().forced_cleanup
                {
                    return Err("unit stop exits 0 within its printed bound, with no tree left".into());
                }
                self.unit_read(client)?;
                self.unit.phase = Phase::Read;
            }
            Phase::Parent | Phase::Before | Phase::StillLive | Phase::Released | Phase::Read | Phase::Turn
                if client.is_empty() =>
            {
                let calls = check_calls(client)?;
                self.unit_completed(now, client, calls.ends())?;
            }
            Phase::Gate
            | Phase::Parent
            | Phase::Before
            | Phase::StillLive
            | Phase::Released
            | Phase::Exit
            | Phase::Read
            | Phase::Turn => {}
        }
        Ok(())
    }
    fn unit_read(&self, host: &mut Root) -> Result<(), String> {
        self.calls(
            host,
            vec![Call::new(&[
                "task",
                "show",
                &self.unit.task,
                "--activity",
                &temper_client_tools::limits::ROWS.to_string(),
                "--json",
            ])],
        )
    }
    fn stop_unit(&mut self, now: Time, client: &mut Root, roots: &[Root]) -> Result<(), String> {
        if !self.members.is_up() {
            return Ok(());
        }
        let worker = self.members.binary("w1", roots).ok_or("worker observer")?;
        let agents: Vec<_> = worker
            .tree_status()
            .live_pids
            .iter()
            .copied()
            .filter(|pid| {
                fs::read_link(format!("/proc/{pid}/exe"))
                    .is_ok_and(|path| path.file_name().is_some_and(|name| name == "temper-agent"))
            })
            .collect();
        // The parent's terminal answer can precede its last process-tree poll.
        let [agent] = agents.as_slice() else { return Ok(()) };
        let (bound, drain) = support::cluster::stop_bounds(worker.stderr())?;
        let timeout = bound.saturating_add(UNIT_MARGIN);
        self.unit.bound_until = now.saturating_add(bound);
        self.unit.drain_until = now.saturating_add(drain);
        self.deadline = self.unit.bound_until.saturating_add(support::WITHIN);
        self.members.stop_one_within("w1", timeout);
        // The next members.act queues TERM to the leader alone, before this
        // command signals the one agent found in its keeper's retained tree.
        self.calls(client, vec![Call::command(Path::new("/bin/kill"), &["-TERM", &agent.to_string()])])?;
        self.unit.phase = Phase::Signalled;
        Ok(())
    }
    fn held_run(&self, packet: &Value) -> Result<(), String> {
        let task = &packet["task"];
        if owner(task)["details"]["attempt"] != self.unit.attempt
            || activity(&packet["activity"])?
                .iter()
                .any(|row| row["event"]["owner"]["source"]["line"]["run_ended"].is_object())
        {
            return Err(format!("the signalled attempt remains live behind the unreleased gate: {packet}"));
        }
        Ok(())
    }
    fn unit_completed(&mut self, now: Time, host: &mut Root, calls: &[support::Ended]) -> Result<(), String> {
        let call = calls.last().ok_or("unit story's call")?;
        match self.unit.phase {
            Phase::Parent => {
                let parent = json(call)?;
                crate::complete_cluster_json(&parent)?;
                if !owner(&parent)["details"]["attempt"].is_null() {
                    self.calls(host, vec![Call::new(&["task", "show", &self.chat, "--json"])])?;
                    return Ok(());
                }
                self.unit.task =
                    one(&parent, "delegates")?["chip"]["task"].as_u64().ok_or("producer task")?.to_string();
                self.unit_read(host)?;
                self.unit.phase = Phase::Before;
            }
            Phase::Before => {
                let packet = json(call)?;
                self.unit.attempt = owner(&packet["task"])["details"]["attempt"].as_u64().ok_or("held attempt")?;
                self.held_run(&packet)?;
                self.unit.phase = Phase::Agent;
            }
            Phase::StillLive => {
                self.held_run(&json(call)?)?;
                if now >= self.unit.drain_until {
                    return Err("gate must be released inside the printed drain, not after its cancellation".into());
                }
                let capture = self.members.capture("llm", "address").ok_or("LLM ready line")?;
                let (_, control) = capture.split_once(' ').ok_or("LLM control address")?;
                let url = format!("http://{control}/release/gate");
                self.calls(
                    host,
                    vec![Call::command(
                        Path::new("/usr/bin/curl"),
                        &[
                            "--fail",
                            "--silent",
                            "--show-error",
                            "--request",
                            "POST",
                            "--write-out",
                            "%{http_code}",
                            &url,
                        ],
                    )],
                )?;
                self.unit.phase = Phase::Released;
            }
            Phase::Released => {
                if call.stdout != b"204" || now >= self.unit.drain_until {
                    return Err("gate release is acknowledged within the printed drain".into());
                }
                self.unit.phase = Phase::Exit;
            }
            Phase::Read => {
                let packet = json(call)?;
                let number = activity(&packet["activity"])?
                    .iter()
                    .filter_map(|row| row.pointer("/event/owner/source/line/turn"))
                    .filter(|turn| turn["attempt"] == self.unit.attempt)
                    .filter_map(|turn| turn["turn"].as_u64())
                    .max()
                    .ok_or("the drained attempt's last committed turn")?;
                self.calls(
                    host,
                    vec![Call::new(&[
                        "task",
                        "turn",
                        &self.unit.task,
                        &format!("{}.{number}", self.unit.attempt),
                        "--json",
                    ])],
                )?;
                self.unit.phase = Phase::Turn;
            }
            Phase::Turn => {
                committed_reply(call, REPLY)?;
                self.stop(now);
            }
            Phase::Gate | Phase::Agent | Phase::Signalled | Phase::Exit => {
                return Err("no unit calls in this phase".into());
            }
        }
        Ok(())
    }
}
