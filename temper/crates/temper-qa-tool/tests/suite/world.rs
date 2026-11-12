//! A git DAG, command-writing agents, and a clock moved only by the test (qa/README.md, 6).
use skein_scratch::Scratch;
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    fs,
    path::{Path, PathBuf},
};
use temper_qa_tool::{
    Exit, Home, Ledger, Refusal,
    faces::{
        ClockFace, CommandAnswer, Commit, Entry, Environment, GitFace, Moment, ProcessCommand, Processes,
        RealProcesses, Signal,
    },
    runner::Run,
};

pub fn id(number: u64) -> String {
    format!("{number:040x}")
}
#[derive(Default)]
pub struct Node {
    pub parents: Vec<String>,
    pub message: String,
    pub files: BTreeMap<String, Vec<u8>>,
    pub paths: Vec<String>,
}
#[derive(Default)]
pub struct Git {
    pub head: String,
    pub nodes: BTreeMap<String, Node>,
    order: Vec<String>,
    pub calls: Vec<String>,
    pub fail_head: bool,
    pub fail_add: bool,
    pub fail_remove: bool,
    pub prunes: usize,
}
impl Git {
    pub fn commit(&mut self, number: u64, parents: &[u64], message: &str) {
        let name = id(number);
        self.order.push(name.clone());
        self.nodes.insert(
            name,
            Node { parents: parents.iter().map(|n| id(*n)).collect(), message: message.into(), ..Node::default() },
        );
        self.head = id(number);
    }
    pub fn merge(&mut self, number: u64, parents: &[u64], goal: u64) {
        self.commit(number, parents, &format!("Goal {goal}'s title\n\nGoal: {goal}\nPull-request: 12\n"));
        for path in [
            "temper/docs/design/client/pages/work.md",
            "temper/docs/design/client/pages/README.md",
            "temper/docs/design/client/pages/work.status.md",
            "temper/docs/design/client/pages/chat/README.md",
            "temper/docs/design/client/tui/pages/chat.md",
        ] {
            self.file_at(number, path, b"# Page\n");
        }
    }
    pub fn file_at(&mut self, commit: u64, path: &str, bytes: &[u8]) {
        self.nodes.get_mut(&id(commit)).expect("commit").files.insert(path.into(), bytes.into());
    }
    fn held(&self, commit: &str) -> BTreeSet<String> {
        let mut held = BTreeSet::new();
        let mut todo = vec![commit.to_owned()];
        while let Some(name) = todo.pop() {
            if held.insert(name.clone()) {
                todo.extend(self.nodes.get(&name).expect("known commit").parents.clone());
            }
        }
        held
    }
    fn node(&self, name: &str) -> &Node {
        self.nodes.get(name).expect("known commit")
    }
}
impl GitFace for Git {
    fn fetch(&mut self, remote: &str, branch: &str) -> Result<(), Refusal> {
        self.calls.push(format!("fetch {remote} {branch}"));
        Ok(())
    }
    fn head(&mut self, reference: &str) -> Result<String, Refusal> {
        self.calls.push(format!("head {reference}"));
        if self.fail_head {
            return Err(Refusal("head refused".into()));
        }
        Ok(self.head.clone())
    }
    fn commits(&mut self, old: &str, new: &str) -> Result<Vec<Commit>, Refusal> {
        self.calls.push(format!("move {old} {new}"));
        let old = self.held(old);
        let new = self.held(new);
        Ok(self
            .order
            .iter()
            .filter(|name| new.contains(*name) && !old.contains(*name))
            .map(|name| Commit { name: name.clone(), parents: self.node(name).parents.clone() })
            .collect())
    }
    fn message(&mut self, commit: &str) -> Result<String, Refusal> {
        self.calls.push(format!("message {commit}"));
        Ok(self.node(commit).message.clone())
    }
    fn contains(&mut self, ancestor: &str, descendant: &str) -> Result<bool, Refusal> {
        self.calls.push(format!("contains {ancestor} {descendant}"));
        Ok(self.held(descendant).contains(ancestor))
    }
    fn count(&mut self, commit: &str) -> Result<u64, Refusal> {
        Ok(u64::try_from(self.held(commit).len()).expect("count"))
    }
    fn entries(&mut self, commit: &str, directory: &str) -> Result<Vec<Entry>, Refusal> {
        self.calls.push(format!("entries {commit} {directory}"));
        let prefix = format!("{directory}/");
        let mut entries = BTreeMap::new();
        for path in self.node(commit).files.keys() {
            if let Some(rest) = path.strip_prefix(&prefix) {
                let (name, directory) = rest.split_once('/').map_or((rest, false), |(first, _)| (first, true));
                entries.insert(name.to_owned(), directory);
            }
        }
        Ok(entries.into_iter().map(|(name, directory)| Entry { name, directory }).collect())
    }
    fn file(&mut self, commit: &str, path: &str) -> Result<Option<Vec<u8>>, Refusal> {
        self.calls.push(format!("file {commit} {path}"));
        Ok(self.node(commit).files.get(path).cloned())
    }
    fn diff(&mut self, parent: &str, commit: &str) -> Result<Vec<u8>, Refusal> {
        assert_eq!(parent, format!("{commit}^1"));
        Ok(b"diff --git a/page b/page\n+new page\n".into())
    }
    fn stat(&mut self, parent: &str, commit: &str) -> Result<String, Refusal> {
        assert_eq!(parent, format!("{commit}^1"));
        Ok(" page | 1 +\n 1 file changed, 1 insertion(+)\n".into())
    }
    fn changed_paths(&mut self, parent: &str, commit: &str) -> Result<Vec<String>, Refusal> {
        assert_eq!(parent, format!("{commit}^1"));
        Ok(self.node(commit).paths.clone())
    }
    fn qa_trailers(&mut self, parent: &str, commit: &str) -> Result<Vec<String>, Refusal> {
        assert_eq!(parent, format!("{commit}^1"));
        Ok(vec!["Work lists drafts".into(), "Work lists drafts".into()])
    }
    fn add_tree(&mut self, path: &Path, commit: &str) -> Result<(), Refusal> {
        self.calls.push(format!("add {commit}"));
        fs::create_dir_all(path).expect("fake tree");
        if self.fail_add {
            return Err(Refusal("worktree refused".into()));
        }
        Ok(())
    }
    fn remove_tree(&mut self, path: &Path) -> Result<(), Refusal> {
        self.calls.push(format!("remove {}", path.display()));
        if self.fail_remove {
            return Err(Refusal("remove refused".into()));
        }
        if path.exists() {
            fs::remove_dir_all(path).expect("fake tree removed");
        }
        Ok(())
    }
    fn prune(&mut self) -> Result<(), Refusal> {
        self.prunes += 1;
        Ok(())
    }
}

#[derive(Clone, Copy)]
pub enum Agent {
    None,
    Case,
    Empty,
    Never,
    IgnoreTerm,
    Unstarted,
}
struct Process {
    command: ProcessCommand,
    agent: Agent,
    ended: bool,
    notify: bool,
}
#[derive(Default)]
pub struct Agents {
    pub next: VecDeque<Agent>,
    pub commands: Vec<ProcessCommand>,
    pub notifications: Vec<Vec<u8>>,
    pub signals: Vec<(u64, Signal)>,
    pub bounded: Vec<(ProcessCommand, u64)>,
    pub command_bound: bool,
    processes: BTreeMap<u64, Process>,
}
impl Processes for Agents {
    fn run(&mut self, command: &ProcessCommand, seconds: u64) -> Result<CommandAnswer, Refusal> {
        self.bounded.push((command.clone(), seconds));
        if self.command_bound {
            return Ok(CommandAnswer { exit: Exit::Bound, last_stdout: None });
        }
        RealProcesses::new(command.environment.clone()).run(command, seconds)
    }
    fn start(&mut self, command: &ProcessCommand) -> Result<u64, Refusal> {
        self.commands.push(command.clone());
        assert!(command.stdin.is_file());
        let notify = command.arguments.first().is_some_and(|arg| arg == "notify");
        let agent = if notify { Agent::Empty } else { self.next.pop_front().unwrap_or(Agent::Never) };
        match agent {
            Agent::Unstarted => return Err(Refusal("agent not found".into())),
            Agent::None | Agent::Case | Agent::Empty | Agent::Never | Agent::IgnoreTerm => (),
        }
        let number = u64::try_from(self.commands.len()).expect("process number");
        if notify {
            self.notifications.push(fs::read(&command.stdin).expect("notify stdin"));
        }
        self.processes.insert(number, Process { command: command.clone(), agent, ended: false, notify });
        Ok(number)
    }
    fn poll(&mut self, process: u64) -> Result<Option<Exit>, Refusal> {
        let process = self.processes.get_mut(&process).expect("process");
        if process.ended {
            return Ok(Some(Exit::Signal(15)));
        }
        if process.notify {
            process.ended = true;
            return Ok(Some(Exit::Code(9)));
        }
        match process.agent {
            Agent::Never | Agent::IgnoreTerm => return Ok(None),
            Agent::Unstarted => unreachable!("not started"),
            Agent::None | Agent::Case | Agent::Empty => (),
        }
        let env = |key: &str| {
            process
                .command
                .environment
                .iter()
                .find(|(name, _)| name.as_ref() == key.as_bytes())
                .map(|(_, value)| String::from_utf8(value.to_vec()).expect("env"))
        };
        let home = Home::open(Path::new(&env("TEMPER_INTERNAL_QA_HOME").expect("home env"))).expect("agent's home");
        let number = env("TEMPER_INTERNAL_QA_SESSION").expect("session env").parse().expect("number");
        home.transact(|ledger| {
            match process.agent {
                Agent::None => {
                    ledger.answer_none(number, "internal refactor")?;
                }
                Agent::Case => {
                    ledger.add_case(number, &super::case(), 0)?;
                }
                Agent::Empty => (),
                Agent::Never | Agent::IgnoreTerm | Agent::Unstarted => unreachable!("not ending"),
            }
            Ok(())
        })?;
        fs::write(&process.command.output, "agent output\ntokens 1234\n\n").expect("fake output");
        process.ended = true;
        Ok(Some(Exit::Code(0)))
    }
    fn signal(&mut self, process: u64, signal: Signal) -> Result<(), Refusal> {
        self.signals.push((process, signal));
        let process = self.processes.get_mut(&process).expect("process");
        match (process.agent, signal) {
            (Agent::IgnoreTerm, Signal::Term) => (),
            (
                Agent::None | Agent::Case | Agent::Empty | Agent::Never | Agent::IgnoreTerm | Agent::Unstarted,
                Signal::Kill,
            )
            | (Agent::None | Agent::Case | Agent::Empty | Agent::Never | Agent::Unstarted, Signal::Term) => {
                process.ended = true;
            }
        }
        Ok(())
    }
}
#[derive(Default)]
pub struct Clock {
    pub time: u64,
}
impl ClockFace for Clock {
    fn now(&self) -> Moment {
        Moment { wall: self.time, monotonic: self.time }
    }
    fn sleep(&mut self, seconds: u64) {
        self.time += seconds;
    }
}
pub type Loop = Run<Git, Agents, Clock>;
pub struct World {
    pub scratch: Scratch,
    pub home: Home,
}
impl World {
    pub fn new() -> Self {
        let scratch = Scratch::new("qa-run");
        fs::write(scratch.path().join("qa.json"), br#"{"owner":"qa","git":{"directory":"repo"},"agent":{"command":["agent","--effort={effort}"],"triage":"cheap","pass":"high"},"request":{"command":["file"]},"bounds":{"poll":60,"triage":30,"build_wait":90},"notify":["notify"]}"#).expect("config");
        let home = Home::open(scratch.path()).expect("home");
        Self { scratch, home }
    }
    pub fn start(&self, git: Git, agents: &[Agent]) -> Loop {
        self.start_with_environment(git, agents, vec![(b"CAPTURED".as_slice().into(), b"kept".as_slice().into())])
    }
    pub fn start_with_environment(&self, git: Git, agents: &[Agent], environment: Environment) -> Loop {
        let processes = Agents { next: agents.iter().copied().collect(), ..Agents::default() };
        Run::start(self.home.clone(), git, processes, Clock::default(), environment).expect("run")
    }
    pub fn ledger(&self) -> Ledger {
        self.home.read(|ledger| Ok(ledger.clone())).expect("read")
    }
    pub fn build(&self, commit: u64) -> PathBuf {
        let directory = self.home.builds.join(id(commit));
        fs::create_dir_all(&directory).expect("build directory");
        self.home.transact(|ledger| ledger.keep_build(&id(commit), 0)).expect("kept");
        directory
    }
}
pub fn baseline() -> Git {
    let mut git = Git::default();
    git.commit(0, &[], "baseline");
    git
}
pub fn land(run: &mut Loop, commit: u64, goal: u64) {
    let parent = u64::from_str_radix(&run.git.head, 16).expect("short id");
    run.git.commit(commit - 1, &[parent], "goal tip");
    run.git.merge(commit, &[parent, commit - 1], goal);
}
