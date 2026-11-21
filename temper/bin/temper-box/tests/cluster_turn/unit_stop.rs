//! A unit's mixed stop, sharing the cluster-turn setup (worker-restarts.md, 2 and 9).
use super::{Call, Cluster, Exit, Host, Judge, Path, Root, Stage, Time, Value, fs, json, one, setup, support};
use skein_lib::Duration;

const CUE: &[u8] = b"## Task\n\n@coding NOTICE";
const REPLY: &str = "The held answer settled.";
const SAVED: &str = "Saved work\nSettled turn\n";
// temper/docs/help/server.md, 5: the unit's timeout is the printed bound plus 34 s.
const UNIT_MARGIN: Duration = Duration::from_secs(34);
const ANSWERS: usize = 3; // read, write, then the held answer; no query after the drain.

pub(super) fn prepare(cluster: &mut Cluster) {
    cluster.record_llm();
    let path = cluster.dir("llm").join("llm.json");
    let mut llm = setup::read(&path);
    llm["control"] = serde_json::json!({"socket":"127.0.0.1:0"});
    llm["cued"][0]["turns"] = serde_json::json!([
        {"lines":[{"call":"read","arguments":{"path":"NOTICE"}}],"finish":"tool_calls","tokens":1},
        {"lines":[{"call":"write","arguments":{"path":"NOTICE","content":"Saved work\n"}}],"finish":"tool_calls","tokens":1},
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
    Activity,
    Turns,
    Branches,
    Saved,
}
#[derive(Debug)]
pub(super) struct Stop {
    phase: Phase,
    task: String,
    attempt: u64,
    until: Time,
    drain_until: Time,
    rows: Vec<Value>,
    task_read: Value,
}
impl Stop {
    pub(super) fn new() -> Self {
        Self {
            phase: Phase::Gate,
            task: String::new(),
            attempt: 0,
            until: Time::ZERO,
            drain_until: Time::ZERO,
            rows: Vec::new(),
            task_read: Value::Null,
        }
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
    pub(super) fn unit_queries(&self) -> Result<usize, String> {
        use skein_fake_llm_domain::api::Part;
        use temper_fake::llm::observations::Observation;
        let bytes = fs::read(self.members.dir("llm").join("observations.jsonl"))
            .map_err(|why| format!("LLM query records: {why}"))?;
        let mut queries = 0;
        for line in bytes.split_inclusive(|byte| *byte == b'\n').filter(|line| line.ends_with(b"\n")) {
            match temper_fake::llm::observations::decode(line).map_err(|why| format!("LLM query record: {why:?}"))? {
                Observation::Query(query) => {
                    if contains(&query.system, CUE)
                        || query
                            .messages
                            .iter()
                            .flat_map(|message| message.parts.iter())
                            .any(|part| matches!(part, Part::Text { text } if contains(text, CUE)))
                    {
                        queries += 1;
                    }
                }
                Observation::Checkpoint { lost: 0 } => {}
                Observation::Checkpoint { lost } => return Err(format!("LLM lost {lost} records")),
                Observation::Full { .. } => return Err("LLM records full".into()),
            }
        }
        if queries > ANSWERS {
            return Err(format!("one query per answer, none repeated or begun after drain: {queries}"));
        }
        Ok(queries)
    }
    pub(super) fn advance_unit(&mut self, now: Time, hosts: &mut [Root]) -> Result<(), String> {
        let (client, roots) = hosts.split_last_mut().ok_or("client root")?;
        self.tick = Some(now.saturating_add(Duration::from_millis(20)));
        match self.unit.phase {
            Phase::Gate if self.unit_queries()? == ANSWERS => {
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
                if now >= self.unit.until
                    || worker.exit_status() != Some(Exit::Code(0))
                    || !worker.tree_status().live_pids.is_empty()
                    || worker.tree_status().live_at_leader_exit.as_ref().is_none_or(|pids| !pids.is_empty())
                    || worker.tree_status().forced_cleanup
                {
                    return Err("unit stop exits 0 within its printed bound plus margin, with no tree left".into());
                }
                self.unit_read(client)?;
                self.unit.phase = Phase::Read;
            }
            Phase::Parent
            | Phase::Before
            | Phase::StillLive
            | Phase::Released
            | Phase::Read
            | Phase::Activity
            | Phase::Turns
            | Phase::Branches
            | Phase::Saved
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
            | Phase::Activity
            | Phase::Turns
            | Phase::Branches
            | Phase::Saved => {}
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
        self.unit.until = now.saturating_add(timeout);
        self.unit.drain_until = now.saturating_add(drain);
        self.deadline = self.unit.until.saturating_add(support::WITHIN);
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
            || self.unit_queries()? != ANSWERS
        {
            return Err(format!("the signalled attempt remains live behind the unreleased gate: {packet}"));
        }
        Ok(())
    }
    #[expect(
        clippy::too_many_lines,
        reason = "the unit's stop and readback keep their causal order in one transition table"
    )]
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
                if call.stdout != b"204" {
                    return Err("gate release is acknowledged once".into());
                }
                self.unit.phase = Phase::Exit;
            }
            Phase::Read => {
                let packet = json(call)?;
                crate::complete_cluster_json(&packet)?;
                self.unit.task_read = packet["task"].clone();
                self.unit_activity(host, &packet["activity"])?;
            }
            Phase::Activity => self.unit_activity(host, &json(call)?)?,
            Phase::Turns => {
                if calls.len() != ANSWERS || self.unit_queries()? != ANSWERS {
                    return Err("three queries, three committed turns, including the held answer once".into());
                }
                let mut replies = 0;
                for call in calls {
                    let turn = json(call)?;
                    crate::complete_cluster_json(&turn)?;
                    for message in turn["turn"]["decoded"]["messages"].as_array().ok_or("decoded turn messages")? {
                        if message["speaker"] == "assistant" {
                            for block in message["blocks"].as_array().ok_or("message blocks")? {
                                replies += block["text"]["text"].as_str().map_or(0, |text| text.matches(REPLY).count());
                            }
                        }
                    }
                }
                if replies != 1 {
                    return Err(format!("the engine holds the released answer exactly once: {replies}"));
                }
                self.calls(host, vec![self.api("/branches?limit=32&page=1")?])?;
                self.unit.phase = Phase::Branches;
            }
            Phase::Branches => {
                let branches = json(call)?;
                let suffix = format!("/s{}", self.unit.task);
                let saved: Vec<_> = branches
                    .as_array()
                    .ok_or("branches")?
                    .iter()
                    .filter_map(|branch| branch["name"].as_str())
                    .filter(|name| name.ends_with(&suffix))
                    .collect();
                let [branch] = saved.as_slice() else {
                    return Err(format!("one saved-work branch for the task: {branches}"));
                };
                self.calls(host, vec![self.api(&format!("/raw/NOTICE?ref={branch}"))?])?;
                self.unit.phase = Phase::Saved;
            }
            Phase::Saved => {
                if call.stdout != SAVED.as_bytes() {
                    return Err("shutdown saved the held turn's write to its saved-work branch".into());
                }
                self.members.stop();
                self.stage = Stage::Stopping;
            }
            Phase::Gate | Phase::Agent | Phase::Signalled | Phase::Exit => {
                return Err("no unit calls in this phase".into());
            }
        }
        Ok(())
    }
    fn unit_activity(&mut self, host: &mut Root, page: &Value) -> Result<(), String> {
        self.unit.rows.extend(activity(page)?.iter().cloned());
        if let Some(before) = page["before"]["value"].as_str() {
            self.calls(host, vec![Call::new(&["task", "activity", &self.unit.task, "--before", before, "--json"])])?;
            self.unit.phase = Phase::Activity;
            return Ok(());
        }
        let task = &self.unit.task_read;
        let details = &owner(task)["details"];
        let lines: Vec<_> = self.unit.rows.iter().map(|row| &row["event"]["owner"]["source"]["line"]).collect();
        let ends: Vec<_> = lines.iter().filter_map(|line| line.get("run_ended")).collect();
        // Recheck (Cancelled(Shutdown)) is named "refused" by ctl's RunEnd;
        // unlike a failure it leaves the task due, with no retry or failed try.
        if ends.len() != 1
            || ends[0]["attempt"] != self.unit.attempt
            || ends[0]["end"] != "refused"
            || owner(task)["phase"] != serde_json::json!({"active":"due"})
            || !details["attempt"].is_null()
            || !details["retry"].is_null()
            || !details["hold"].is_null()
            || lines.iter().filter(|line| line["run_began"].is_object()).count() != 1
        {
            return Err(format!(
                "one interrupted attempt, no failure or try spent: {task}; activity: {:?}",
                self.unit.rows
            ));
        }
        let calls: Vec<_> = lines
            .iter()
            .filter_map(|line| line.get("turn"))
            .map(|turn| {
                let attempt = turn["attempt"].as_u64().ok_or("turn attempt")?;
                let number = turn["turn"].as_u64().ok_or("turn number")?;
                if attempt != self.unit.attempt {
                    return Err("the held attempt owns every turn".into());
                }
                Ok(Call::new(&["task", "turn", &self.unit.task, &format!("{attempt}.{number}"), "--json"]))
            })
            .collect::<Result<_, String>>()?;
        self.calls(host, calls)?;
        self.unit.phase = Phase::Turns;
        Ok(())
    }
}

fn check_calls(host: &mut Root) -> Result<Box<support::ctl::Calls>, String> {
    let Root::Client { calls, .. } = host else { return Err("client root".into()) };
    let calls = calls.take().ok_or("completed unit calls")?;
    for ended in calls.ends() {
        if ended.exit != Some(Exit::Code(0)) {
            return Err(format!("unit call {:?}: {}", ended.exit, support::without_secrets(&ended.stderr)));
        }
    }
    Ok(calls)
}
