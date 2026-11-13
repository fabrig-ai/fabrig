//! Scriptable deployment shared by the redeploy and referee suites (redeploy.md, 8).
use crate::{
    description::{Description, Engine, Worker},
    hosts::{Hosts, Output, Part},
    stage::binaries,
};
use std::collections::{BTreeMap, VecDeque};

pub const OLD: &str = "0123456789ab";
pub const NEW: &str = "abcdef012345";
pub const WIRES: &str = "jig-worker-wire 13, smith-pieces 1, smith-charter 5, smith-transcript 3, temper-worker-wire 9";
#[derive(Clone, Debug)]
pub struct Machine {
    pub running: bool,
    /// Binary name to installed commit.
    pub installed: BTreeMap<String, String>,
    /// Full staged binary path to commit.
    pub staged: BTreeMap<String, String>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Call {
    pub part: Part,
    pub at: u64,
    pub line: String,
}
#[derive(Clone, Debug)]
pub enum Opening {
    After { nanoseconds: u64, wires: String },
    Refused { nanoseconds: u64, text: String },
    Never,
}
impl Default for Opening {
    fn default() -> Self {
        Self::After { nanoseconds: 0, wires: WIRES.into() }
    }
}
pub struct Deployment {
    pub description: Description,
    pub engine: Machine,
    pub workers: Vec<Machine>,
    pub at: u64,
    pub lines: Vec<Call>,
    pub log: Vec<(u64, String)>,
    pub check: Output,
    pub openings: Vec<Opening>,
    pub documents: Vec<Output>,
    /// A response queue per ctl argument line; the final response repeats for polling.
    pub ctl: BTreeMap<String, VecDeque<Output>>,
    pub failures: BTreeMap<(Part, String), Output>,
    /// Override a binary's version response, even after fetching it.
    pub versions: BTreeMap<(Part, String), Output>,
    pending: Vec<(usize, u64, String)>,
}
#[must_use]
pub fn output(code: i32, stdout: &str) -> Output {
    Output { code: Some(code), stdout: stdout.into(), stderr: String::new() }
}
#[must_use]
pub fn description(count: usize) -> Description {
    Description {
        source: "https://builds.example.com/temper/v0".into(),
        engine: Engine {
            run: vec!["sh".into(), "-c".into()],
            binaries: "/bin".into(),
            builds: "/builds".into(),
            stop: "stop engine".into(),
            start: "start engine".into(),
            log: "log engine {since}".into(),
            check: "check {binary}".into(),
            backup: "backup {name}".into(),
            swap: "swap {from} {to}".into(),
            ctl: "ctl engine".into(),
        },
        workers: (1..=count)
            .map(|index| {
                let name = format!("w{index}");
                Worker {
                    name: name.clone(),
                    run: vec!["sh".into(), "-c".into()],
                    binaries: "/bin".into(),
                    builds: "/builds".into(),
                    stop: format!("stop {name}"),
                    start: format!("start {name}"),
                    swap: "swap {from} {to}".into(),
                    document: format!("document {name}"),
                }
            })
            .collect(),
    }
}
impl Deployment {
    #[must_use]
    pub fn new(description: Description) -> Self {
        let machine = |part| Machine {
            running: true,
            installed: binaries(&description, part).2.into_iter().map(|name| (name.into(), OLD.into())).collect(),
            staged: BTreeMap::new(),
        };
        let engine = machine(Part::Engine);
        let workers = (0..description.workers.len()).map(|index| machine(Part::Worker(index))).collect();
        let documents = description
            .workers
            .iter()
            .map(|worker| {
                output(
                    0,
                    &serde_json::json!({"workers": [{"name": worker.name, "graces": {"grace_ms": 30000}}]}).to_string(),
                )
            })
            .collect();
        Self {
            openings: vec![Opening::default(); description.workers.len()],
            description,
            engine,
            workers,
            documents,
            at: 1_000_000_000,
            lines: Vec::new(),
            log: Vec::new(),
            check: output(0, "temper-engine upgrade: workers may stay: yes\n"),
            ctl: BTreeMap::new(),
            failures: BTreeMap::new(),
            versions: BTreeMap::new(),
            pending: Vec::new(),
        }
    }
    pub fn check_answer(&mut self, code: i32, lines: &str) {
        self.check = output(code, lines);
    }
    pub fn opening(&mut self, worker: usize, opening: Opening) {
        self.openings[worker] = opening;
    }
    pub fn ctl_answer(&mut self, arguments: &str, answer: Output) {
        self.ctl.entry(arguments.into()).or_default().push_back(answer);
    }
    pub fn fail_line(&mut self, part: Part, line: &str, code: i32) {
        self.failures.insert(
            (part, line.into()),
            Output { code: Some(code), stdout: String::new(), stderr: "scripted failure".into() },
        );
    }
    pub fn stage_all(&mut self, commit: &str) {
        for part in std::iter::once(Part::Engine).chain((0..self.workers.len()).map(Part::Worker)) {
            let (_, builds, names) = binaries(&self.description, part);
            let paths: Vec<_> = names.iter().map(|name| format!("{builds}/{commit}/{name}")).collect();
            for path in paths {
                self.machine_mut(part).staged.insert(path, commit.into());
            }
        }
    }
    #[must_use]
    pub fn machine(&self, part: Part) -> &Machine {
        match part {
            Part::Engine => &self.engine,
            Part::Worker(index) => &self.workers[index],
        }
    }
    pub fn machine_mut(&mut self, part: Part) -> &mut Machine {
        match part {
            Part::Engine => &mut self.engine,
            Part::Worker(index) => &mut self.workers[index],
        }
    }
    fn schedule(&mut self, index: usize) {
        self.pending.retain(|(worker, _, _)| *worker != index);
        let name = &self.description.workers[index].name;
        let (delay, line) = match &self.openings[index] {
            Opening::After { nanoseconds, wires } => {
                (*nanoseconds, format!("temper-engine serve: worker {name} opened: {wires}"))
            }
            Opening::Refused { nanoseconds, text } => {
                (*nanoseconds, format!("temper-engine serve: worker {name} refused at its opening: wires: {text}"))
            }
            Opening::Never => return,
        };
        self.pending.push((index, self.at + delay, line));
        self.flush();
    }
    fn flush(&mut self) {
        let mut pending = Vec::new();
        for (worker, at, line) in self.pending.drain(..) {
            if at <= self.at {
                self.log.push((at, line));
            } else {
                pending.push((worker, at, line));
            }
        }
        self.pending = pending;
        self.log.sort_by_key(|(at, _)| *at);
    }
    fn answer(&mut self, part: Part, line: &str) -> Output {
        if let Some(answer) = self.failures.get(&(part, line.into())) {
            return answer.clone();
        }
        let (stop, start, swap) = match part {
            Part::Engine => {
                (&self.description.engine.stop, &self.description.engine.start, &self.description.engine.swap)
            }
            Part::Worker(index) => {
                let worker = &self.description.workers[index];
                (&worker.stop, &worker.start, &worker.swap)
            }
        };
        if line == stop {
            self.machine_mut(part).running = false;
            match part {
                Part::Engine => self.pending.clear(),
                Part::Worker(index) => self.pending.retain(|(worker, _, _)| *worker != index),
            }
            return output(0, "");
        }
        if line == start {
            self.machine_mut(part).running = true;
            match part {
                Part::Engine => {
                    for index in 0..self.workers.len() {
                        if self.workers[index].running {
                            self.schedule(index);
                        }
                    }
                }
                Part::Worker(index) => {
                    if self.engine.running {
                        self.schedule(index);
                    }
                }
            }
            return output(0, "");
        }
        if let Some(values) = captures(swap, line) {
            return self.copy(part, &values["from"], &values["to"], false);
        }
        if let Some(values) = captures(&self.description.engine.log, line) {
            let since = values["since"].parse::<u64>().expect("log since seconds") * 1_000_000_000;
            let mut text = String::new();
            for (_, line) in self.log.iter().filter(|(at, _)| *at >= since) {
                text.push_str(line);
                text.push('\n');
            }
            return output(0, &text);
        }
        if captures(&self.description.engine.check, line).is_some() {
            return self.check.clone();
        }
        if captures(&self.description.engine.backup, line).is_some() {
            return output(0, "");
        }
        if let Some(arguments) = line.strip_prefix(&format!("{} ", self.description.engine.ctl)) {
            let arguments = shell_words(arguments.strip_suffix(" </dev/null").unwrap_or(arguments)).join(" ");
            return match self.ctl.get_mut(&arguments) {
                Some(answers) if answers.len() > 1 => answers.pop_front().expect("more than one answer"),
                Some(answers) => answers.front().cloned().expect("ctl answer exists"),
                None => Output { code: Some(1), stdout: String::new(), stderr: format!("unscripted ctl: {arguments}") },
            };
        }
        if let Part::Worker(index) = part
            && line == self.description.workers[index].document
        {
            return self.documents[index].clone();
        }
        self.composed(part, line)
    }
    fn composed(&mut self, part: Part, line: &str) -> Output {
        let words = shell_words(line);
        if let [binary, flag] = words.as_slice()
            && flag == "--version"
        {
            if let Some(answer) = self.versions.get(&(part, binary.clone())) {
                return answer.clone();
            }
            let (installed, _, _) = binaries(&self.description, part);
            let name = binary.rsplit('/').next().expect("binary name");
            let machine = self.machine(part);
            let commit = if binary == &format!("{installed}/{name}") {
                machine.installed.get(name)
            } else {
                machine.staged.get(binary)
            };
            return commit.map_or_else(|| output(127, ""), |commit| output(0, &format!("{name} 0.1.0 ({commit})\n")));
        }
        if words.iter().any(|word| word == "curl") {
            let directory = words.windows(2).find(|pair| pair[0] == "cd").expect("fetch directory")[1].clone();
            let commit = directory.rsplit('/').next().expect("commit directory");
            for pair in words.windows(2).filter(|pair| pair[0] == "-o") {
                let name = pair[1].rsplit('/').next().expect("download name");
                self.machine_mut(part).staged.insert(format!("{directory}/{name}"), commit.into());
            }
            return output(0, "");
        }
        if let Some(index) = words.iter().position(|word| word == "cp") {
            let arguments: Vec<_> = words[index + 1..].iter().filter(|word| !word.starts_with('-')).collect();
            if let [from, to] = arguments.as_slice() {
                return self.copy(part, from, to, true);
            }
        }
        if words.first().is_some_and(|word| word == "mkdir") {
            return output(0, "");
        }
        Output { code: Some(127), stdout: String::new(), stderr: format!("unrecognized fake line: {line}") }
    }
    fn copy(&mut self, part: Part, from: &str, to: &str, preserve: bool) -> Output {
        let (directory, _, _) = binaries(&self.description, part);
        let source_name = from.rsplit('/').next().expect("source name");
        let machine = self.machine(part);
        let commit = if from == format!("{directory}/{source_name}") {
            machine.installed.get(source_name)
        } else {
            machine.staged.get(from)
        }
        .cloned();
        let Some(commit) = commit else {
            return output(1, "missing source");
        };
        let installed = to.starts_with(&format!("{directory}/"));
        let name = to.rsplit('/').next().expect("target name");
        let machine = self.machine_mut(part);
        if installed {
            machine.installed.insert(name.into(), commit);
        } else if !preserve || !machine.staged.contains_key(to) {
            machine.staged.insert(to.into(), commit);
        }
        output(0, "")
    }
}
impl Hosts for Deployment {
    fn run(&mut self, part: Part, line: &str) -> Output {
        self.lines.push(Call { part, at: self.at, line: line.into() });
        self.answer(part, line)
    }
    fn run_each(&mut self, lines: &[(Part, String)]) -> Vec<Output> {
        lines.iter().map(|(part, line)| self.run(*part, line)).collect()
    }
    fn now(&mut self) -> u64 {
        self.at
    }
    fn sleep(&mut self, nanoseconds: u64) {
        self.at += nanoseconds;
        self.flush();
    }
}
fn captures(template: &str, line: &str) -> Option<BTreeMap<String, String>> {
    let pattern = shell_words(template);
    let words = shell_words(line);
    if pattern.len() != words.len() {
        return None;
    }
    let mut values = BTreeMap::new();
    for (pattern, word) in pattern.iter().zip(words) {
        if let Some(name) = pattern.strip_prefix('{').and_then(|name| name.strip_suffix('}')) {
            values.insert(name.into(), word);
        } else if pattern != &word {
            return None;
        }
    }
    Some(values)
}
/// Only the test description's POSIX words and the tool's composed lines, not a shell.
fn shell_words(line: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut word = String::new();
    let mut quoted = false;
    let mut escaped = false;
    let mut begun = false;
    for ch in line.chars() {
        if escaped {
            word.push(ch);
            escaped = false;
            begun = true;
        } else if ch == '\\' && !quoted {
            escaped = true;
            begun = true;
        } else if ch == '\'' {
            quoted = !quoted;
            begun = true;
        } else if ch.is_whitespace() && !quoted {
            if begun {
                words.push(std::mem::take(&mut word));
                begun = false;
            }
        } else {
            word.push(ch);
            begun = true;
        }
    }
    assert!(!quoted && !escaped, "closed fake shell word: {line}");
    if begun {
        words.push(word);
    }
    words
}
