# request-dump: contracts

The run's binding calls: what crosses between `dump-kit` and `dump-worker`, and between the kit's layers. Their design is `temper/docs/design/draft/dogfood-hacks.md`, 2.10.

## skein

```rust
// skein_llm::client
impl MeasuredRequest {
    /// Has the prepared client keep a copy of its rendered body until taken.
    pub fn keep_body(&mut self);
}
impl Client {
    /// The rendered body, before any compression, once, if `keep_body` asked.
    pub fn take_kept_body(&mut self) -> Option<Box<[u8]>>;
}

// skein_llm_connection, exported from lib.rs
/// One call's request body as its client rendered it.
pub struct Body { pub call: Token, pub provider: skein_llm::Provider, pub bytes: Box<[u8]> }
impl Component {
    /// Asks for every later prepared call's body. There is no way to stop.
    pub fn keep_bodies(&mut self);
    /// The next kept body, in preparation order.
    pub fn pop_body(&mut self) -> Option<Body>;
}

// skein_fake_llm_protocol::provider
impl Server {
    pub fn keep_body(&mut self);
    /// The last request's body as read, inflated if it came in gzip, once.
    pub fn take_kept_body(&mut self) -> Option<Box<[u8]>>;
}

// skein_fake_peers::llm
impl Peer {
    /// Keeps each query's body from here on (a builder, like `without_records`).
    pub fn keeping_bodies(self) -> Self;
    /// Each query's body as the peer read it, in the order they were read.
    pub fn bodies(&self) -> &[Box<[u8]>];
}
```

## smith

```rust
// smith_protocol_llm, delegating to the connection
impl Component {
    pub fn keep_bodies(&mut self);
    pub fn pop_body(&mut self) -> Option<skein_llm_connection::Body>;
}

// smith_protocol_program::backend, exported
/// What a program was given; never its environment or mcp.json.
pub enum Given {
    /// Its exact argv, excluding the program's name.
    Arguments(Box<[Box<[u8]>]>),
    /// The bytes stored as its instructions file.
    Instructions(Box<[u8]>),
    /// The bytes its MCP server answers `tools/list` with.
    Tools(Box<[u8]>),
    /// One whole line written to its stdin, once.
    Input(Box<[u8]>),
    /// Its `system/init` line, as it printed it.
    Init(Box<[u8]>),
}
impl Backend {
    pub fn keep_given(&mut self);
    pub fn pop_given(&mut self) -> Option<Given>;
}

// smith_agent_service, exported
/// One thing a run sent or was given, for the owner that asked.
pub struct Dump {
    /// The Start's identity, absent where agent admission draws it.
    pub identity: Option<[u8; 16]>,
    pub activation: u64,
    pub what: Dumped,
}
pub enum Dumped {
    /// What the Start's charter gave the run, past its text.
    Started(Started),
    /// One model request's body as rendered for its provider.
    Request { provider: skein_llm::Provider, body: Box<[u8]> },
    /// What a program was given.
    Program(Given),
}
impl Service {
    /// Asks for dumps of the LLM, of each program backend and of a child's when it starts.
    pub fn keep_dumps(&mut self);
    /// The next one: a Start's before its requests, then bodies, then what programs were given.
    pub fn pop_dump(&mut self) -> Option<Dump>;
}
```

`Started` holds owned copies of what `start.txt` needs. Its fields are the task's choice.

```rust
// smith_agent_shell::dump (a new public module), std allowed
/// Writes dumps to one directory, up to a total of `bound` bytes across every writer of it.
pub struct Dumps;
impl Dumps {
    pub fn new(directory: PathBuf, bound: u64) -> Dumps;
}

// smith_agent_shell
impl Agent {
    /// Has every later pass write what the service dumps. A hosted world calls this after `new`.
    pub fn dump_to(&mut self, dumps: Dumps);
}
/// `run_decoded`, dumping.
pub fn run_dumping(decoded: config::Decoded, dumps: Dumps, errors: Box<dyn Write>) -> Result<Exit, String>;
```

`run` and `run_decoded` keep their signatures and write nothing. `Dumps` is the writer of every file and every line below. A pass writes after `service::iterate` and before it moves the service's submissions, so that a request's file is whole before its first byte can leave.

## The directory

- **Made:** `program::private_directory(&directory)`: `Ok(false)` makes it and any missing parents, mode 0700 (`AlreadyExists` is fine, as two agents race), then checks again; `Ok(true)` goes on; `Err` is a failed write.
- **A file:** in `.dump-<pid>-<number>`, created new with mode 0600, whole, then renamed to its name; a failure removes the part.
- **Its name:** `<identity>-a<activation>-<number>-<what>`. `<identity>` is 32 lower-case hex digits, or `none`. `<activation>` is six digits at least. `<number>` counts this `Dumps`'s files from 0, six digits at least.
- **`<what>`:**

| `Dumped` | `<what>` | Its bytes |
|---|---|---|
| `Started` | `start.txt` | `name: value` lines, text as text: mode, resume, waiting, budget's turns, spend and time, reserve, main's model and effort, each sub-agent model and effort, inspect, modify, shell, agents, wait, deliver, host tools by name, the brief's titles |
| `Request`, `Provider::OpenAi` | `openai.json` | the body, unchanged |
| `Request`, `Provider::Anthropic` | `anthropic.json` | the body, unchanged |
| `Arguments` | `claude-code-argv.txt` | each argument, unchanged, on its own line |
| `Instructions` | `claude-code-instructions.md` | unchanged |
| `Tools` | `claude-code-tools.json` | unchanged |
| `Input` | `claude-code-stdin.json` | the line, unchanged |
| `Init` | `claude-code-init.json` | the line, unchanged |

- **The bound:** before each file, sum the sizes of what `read_dir` lists in the directory. If that sum and this file pass `bound`, write nothing more from this `Dumps`, ever, and log one line. A sum that fails is a failed write.
- **A line:** `diagnostic`'s `smith: request dump: <what happened>` on the agent's error output, and the same line appended to `dump.log` in the directory (mode 0600), whose own failure is ignored. One line per failed write; one line for the bound. A failed write's number is not reused, and the next dump tries again.

## temper

- **The variable:** `TEMPER_DUMP_LLM`, named in temper alone. Empty or absent is unset.
- **At the worker's start:** `std::path::absolute` of its value, read once, in both `prepare_with_limits` and `worker_alone::prepare`, beside `workspace_reuse`. A value `absolute` refuses counts as unset, with one line on the worker's standard error.
- **Carried:** `Service::dump_llm(&mut self, directory: &[u8])` makes the environment of every later `host::Launch` for an agent exactly `[b"TEMPER_DUMP_LLM=<directory>"]`. On an engine alone it does nothing.
- **In the agent:** `Command::Agent` reads the variable, and with a value calls `run_dumping(decoded, Dumps::new(directory, 4 GiB), errors)`; without one, `run_decoded` as today.
