# claude-always: the shapes that cross tasks

Binding for this run. Design: `temper/docs/design/protocol/setup.md`, 6.6, 8.2 and 8.3; `temper/docs/design/protocol/configuration.md`, 5 and 12.1.

## The documents init writes

Every document, `--box` and bare, with `claude` on `PATH` or without it. All four templates in `temper/crates/temper-init/template/`.

`temper.json` and `bare.json`, in `engine.llm`, each entry straight after OpenAI's, ahead of `demo`'s where there is one:

```json
{"name":"anthropic","number":2,"signed_in_plan":false,"manage_usage":"https://claude.ai/settings/usage","models":["claude-opus-5-5"],"sources":{"programs":["claude_code"]}}
```
```json
{"name":"anthropic","number":2,"dialect":2,"provider":"anthropic","program":"claude_code","address":{"host":"api.anthropic.com","port":443},"transport":"tls","headers":[],"connect_ms":10000,"handshake_ms":10000,"server_name":"api.anthropic.com"}
```
```json
{"name":"claude-opus-5-5","endpoint":"anthropic","window":1048576,"output":8192,"prices":{"input":1,"cached":1,"output":1,"unit":1000000}}
```

`agent.json` and `bare-agent.json`, in `endpoints`, straight after OpenAI's:

```json
{"name":"anthropic","number":2,"dialect":2,"provider":"anthropic","program":"claude_code","address":"api.anthropic.com:443","transport":"tls","server_name":"api.anthropic.com","connect":10000,"handshake":10000,"headers":[],"models":[{"name":"claude-opus-5-5","prices":{"input":1,"cached":1,"output":1,"unit":1000000},"window":1048576,"output":8192,"reasoning_item":65536,"head":60000,"idle":30000}]}
```

`programs`, always:

```json
{"claude_code":{"path":"<the claude found on PATH, or \"fetched\">","directory":"<data>/programs"}}
```

`environment` stays `[]`: `DISABLE_AUTOUPDATER=1` is the launcher's. No `documents.lines` override is written: `standard`'s 16 holds the offer.

## `"fetched"`

- The word is temper's: `temper_init::FETCHED`, `b"fetched"`.
- smith never sees it. `temper`'s `agent::decoded(path)` reads `agent.json` and, when `programs.claude_code.path` is the word, puts the launcher's absolute path in its place before `smith_agent_shell::config::decode`.
- **The launcher's path** is `<directory's parent>/claude-code/claude`, where `directory` is the document's `programs.claude_code.directory`: `temper`'s `agent::launcher(directory: &Path) -> Option<PathBuf>`, none for a directory with no parent.

## smith's agent kit

```rust
// smith_agent_shell::config
pub fn read_bytes(path: &Path) -> Result<Box<[u8]>, String>; // read_decoded's read, its refusals unchanged
// smith_agent_shell
pub fn run_decoded(decoded: config::Decoded, errors: Box<dyn Write>) -> Result<Exit, String>; // run after its read
```

## The launcher's two links

Beside each other, in the launcher's directory, private (0o700):

- `claude`, a symbolic link to the running `temper`;
- `cache`, a symbolic link to the cache directory.

## The pin and its host

```rust
// temper_forge_setup::claude_code
pub const VERSION: &str = "2.1.311";
pub struct Build { pub platform: &'static str, pub digest: [u8; 32], pub bytes: u64 }
pub enum Libc { Gnu, Musl }
pub fn build(platform: Platform, machine: &str, libc: Libc) -> Result<&'static Build, Failure>;
pub fn file(cache: &[u8]) -> Box<[u8]>; // <cache>/claude-code/VERSION/claude
pub fn pin(cache: &[u8], origin: Origin, build: &Build) -> Pin;
// temper_forge_setup_shell::fetch
pub fn claude_code(limits, trust, file: &Path, pin: Pin, bytes: u64, out: &mut dyn io::Write) -> Result<(), Failed>;
```

`Platform` is `skein_forge::forgejo::pin::Platform`. The progress word is `claude-code`.
