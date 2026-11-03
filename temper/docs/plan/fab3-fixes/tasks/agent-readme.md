+++
id = "agent-readme"
part = "fixes"
kind = "docs"
needs = []
files = [
  "smith/crates/smith-agent-shell/README.md",
  "smith/crates/smith-agent-shell/src/config/tests/readme.rs",
  "smith/crates/smith-agent-shell/src/config/tests/mod.rs",
  ".config/gate/read-by-tests.txt",
  "smith/AGENTS.md",
]
effort = "medium"
minutes = 40
+++

# smith-agent-shell's README shows a configuration the agent accepts, and a test keeps it so

## Goal

`smith/crates/smith-agent-shell/README.md`'s fenced configuration (~8-37) names the Codex route
(provider `codex`, dialect 1, an endpoint `account`, `chatgpt.com`), and its optional fields
(~44-46) name `identity`; the agent refuses all of them at startup
(`smith/docs/design/protocol/agent.md`, 4). The README shows a current OpenAI endpoint instead,
drops `identity`, and a unit test decodes the README's fenced JSON with the agent's own parser so
it stays one the agent accepts.

Acceptance:

- The fenced block is OpenAI's public route: `"provider": "openai"`, `"dialect": 3`, no
  `"account"`, `"address": "api.openai.com:443"`, `"transport": "tls"`, `"server_name":
  "api.openai.com"`; the endpoint's name is `openai`; the profile, models, environment and trace
  stay as they are unless the parser refuses them.
- The optional fields' sentence names `authority`, `target` and `headers` (an array of `{ "name",
  "value" }`), and no `identity`. No other README text changes.
- A test reads the README at compile time and decodes each ` ```json ` fenced block (there is at
  least one) with `config::decode`, which checks every field, the profile, the endpoints and
  their dialect, the models and the derived limits, and neither resolves an address nor loads
  trust: no name lookup of `api.openai.com` (`notes/common.md`, no probe).
- The README is listed as Markdown read by tests: `.config/gate/read-by-tests.txt` and
  `smith/AGENTS.md`.
- The tests in Steps pass, each count not zero; clippy is clean.

## Inputs

- `temper/docs/plan/fab3-fixes/notes/common.md`.
- `smith/AGENTS.md`.
- `temper/docs/plan/fab3-fixes/inputs/request.md`, finding 3.
- `smith/docs/design/protocol/agent.md`, 4: an endpoint's fields; dialects `openai` 3 and
  `anthropic` 2; `account` and `identity` refused.
- `smith/crates/smith-agent-shell/src/config.rs`: `decode` (~97) against `parse` and
  `Decoded::resolve` (~238), which resolve addresses and load trust; the test uses `decode`.
- `smith/crates/smith-agent-shell/src/config/tests/mod.rs`: `shaped_document` (~255), an
  accepted document, and `withdrawn_endpoint_fields_are_refused_by_name` (~45).

## Owns and may change

- `smith/crates/smith-agent-shell/README.md`: the fenced block and the optional fields' sentence.
- `smith/crates/smith-agent-shell/src/config/tests/readme.rs`: new, the test below.
- `smith/crates/smith-agent-shell/src/config/tests/mod.rs`: its `mod readme;` line, beside
  `mod native;`, `mod programs;`, `mod prompt_line;`.
- `.config/gate/read-by-tests.txt`: one line, `smith/crates/smith-agent-shell/README.md`, after
  `smith/docs/design/**`.
- `smith/AGENTS.md`: its "Markdown read by tests" line gains `smith-agent-shell`'s README.

Pre-approved options:

- If `decode` refuses the block for a value the request did not name (a model's quantity, the
  memory, the trace), change that value to the nearest the parser accepts, keeping the README's
  prose true, and say which in the report.
- If the README's prose names another field the parser refuses, stop and ask; the request names
  only `identity`.

## Steps

1. Edit the README's fenced block and the optional fields' sentence.
2. `src/config/tests/readme.rs`:

   ```rust
   //! The README's configuration is one the agent accepts
   //! (smith/docs/design/protocol/agent.md, section 4).
   use super::*;

   const README: &str = include_str!("../../../README.md");

   /// Each ```json fenced block of `text`, in order.
   fn json_blocks(text: &str) -> Vec<&str> { ... }

   #[test]
   fn the_readme_configuration_decodes() {
       let blocks = json_blocks(README);
       assert!(!blocks.is_empty(), "the README shows a configuration");
       for block in blocks {
           decode(block.as_bytes()).expect("the README's configuration decodes");
       }
   }
   ```

   The crate's lints apply (no `unwrap`, assertion messages); `decode`'s refusal is a `String`,
   which `expect` prints, naming the field refused.
3. Narrow tests, one at a time:
   - `heavy cargo nextest run -p smith-agent-shell --lib -E 'test(/readme/)'`
   - `heavy cargo nextest run -p smith-agent-shell --lib -E 'test(/config::tests/)'`
4. `cargo fmt`; `python3 -I -B scripts/old_forms.py` from `smith/` (no count may rise; the new
   file takes no `#[path]` and no `/// Contract:` line);
   `heavy cargo clippy -p smith-agent-shell --all-targets -- -D warnings`;
   `python3 -I -B tools/citations/citations.py --base origin/main` from the repository root.
5. Commits: the README and its test; then the two read-by-tests lists. Push.

## Report

As the implementer's text says, and any value of the block changed beyond the request.
