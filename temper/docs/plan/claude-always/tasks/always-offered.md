+++
id = "always-offered"
part = "offered"
kind = "feature"
needs = ["run-decoded"]
files = ["temper/crates/temper-init/**", "temper/crates/temper-fake/src/backends.rs", "temper/crates/temper-fake/src/backends/tests.rs", "temper/crates/temper/src/agent.rs", "temper/crates/temper/src/lib.rs", "temper/crates/temper/src/worker_alone.rs", "temper/crates/temper/src/fake_backends/import.rs", "temper/crates/temper/src/fake_backends/json.rs", "temper/docs/help/box.md"]
effort = "high"
minutes = 75
+++

## Goal

Claude is offered on every fresh deployment. `temper init`, with `--box` and without, writes the Anthropic provider, its program, endpoint and model, whether or not `claude` is on `PATH`; the fake backends' rewrite finds them written and swaps in its own. A real, non-fake `temper box` with no `claude` on `PATH` starts, and its welcome's Model step shows ChatGPT and Claude. This fixes the user's bug; it fetches nothing. Choosing Claude with no `claude` on `PATH` stays unfinished until the `claude-fetch` task lands.

Acceptance:

- Both documents hold what `notes/contracts.md`, "The documents init writes", says, in all four cases: box and bare, `claude` found and not, the path being the found `claude` or `"fetched"`.
- `temper box` on empty directories with no `claude` on `PATH` reaches its link, and `temper serve`, the worker's agent and `temper program-login` all read a `"fetched"` document, smith never seeing the word.
- The rewrite swaps: exactly one provider `anthropic`, one endpoint numbered 2 and one model on it, `fake-claude`, no `claude-opus-5-5`; `temper box --fake-backends` behaves as before.
- A process story on a real box with no `claude` on `PATH` reads the welcome's Model step and sees ChatGPT's card and Claude's. It passes, and it bites.
- Every test of `temper-init`, `temper-fake`, `temper` (its `suite`, `process` is run for what you touch, below), `temper-service`, `temper-playground` and the worlds listed below passes.

## Inputs

- `temper/docs/plan/claude-always/notes/common.md`
- `temper/docs/plan/claude-always/notes/contracts.md`: "The documents init writes", "`"fetched"`", "smith's agent kit"
- `temper/docs/design/draft/claude-provider.md`, 1, 2 and 5
- `temper/docs/design/protocol/setup.md`, 8.2 and 8.3; 6.6, "Its launcher" alone
- `temper/docs/design/protocol/configuration.md`, 5 and 12.1
- `temper/docs/design/client/README.md`, 3.5, the Model step's cards

## Owns and may change

**Departs from your text:** you edit `temper/docs/help/box.md` (`Not yours` otherwise holds): see the steps. It is read by tests: run `heavy cargo nextest run -p temper-docs` and `-p temper --lib -E 'test(help)'`.

**Owns:** `temper/crates/temper-init/**` (its templates, `template.rs`, `lib.rs`, its tests); `temper/crates/temper-fake/src/backends.rs` and its tests; `temper/crates/temper/src/agent.rs`; the reads in `temper/crates/temper/src/lib.rs` and `worker_alone.rs`; `temper/crates/temper/src/fake_backends/import.rs`, and `json.rs`'s visibility.

**Readers you may change, each only as said:**

- Any test or test support that writes its own Anthropic provider, endpoint or model onto init's documents, or counts init's providers, endpoints or models: edit it to take init's, or to replace them, nothing more. Known: `temper/crates/temper/tests/support/forge_land.rs` (around line 1340: its `retain` keeps out `anthropic` too), `temper/crates/temper/tests/support/first_repository.rs`, `temper/crates/temper/tests/support/sign_in.rs`, `temper/crates/temper/tests/support/configuration.rs`, `temper/crates/temper/tests/support/owned.rs`, `temper/crates/temper/tests/suite/fake_opening.rs`, `temper/crates/temper/src/fake_backends/tests.rs`, `temper/crates/temper/src/tests/configuration.rs`, `temper/crates/temper/src/tests/reference_settings.rs`, `temper/crates/temper/src/init/tests.rs`, `temper/crates/temper-service/src/tests/{stock,welcome_policy,room,fixtures}.rs`, `temper/crates/temper-fake/src/tests/demo.rs`, `temper/crates/temper-fake/src/oauth/tests.rs`, `temper/crates/temper-playground/src/tests/stories.rs`, `temper/tests/engine/domain/tests/fixtures/stock.rs`, `temper/tests/engine/domain/src/forge_routes.rs`, `temper/tests/real-loop/src/llm.rs`, `temper/tests/real-loop/src/fixtures.rs`, `temper/tests/real-loop/tests/suite/template_bounds.rs`.
- `temper/crates/temper/tests/support/person.rs` and `temper/crates/temper/tests/suite/startup.rs`: the new script and story.

**Pre-approved options:**

- If a test pins the charters' model order and takes `demo` as second: the order is now `gpt-6-luna`, Anthropic's, `demo`, as `setup.md`, 8.3 has it (scripted last): change the test.
- If a test picks a charter's or provider's model by position and now picks Anthropic's: pick by name.
- If a stock limit turns out too small for a third model on a box (a startup refusal naming a limit): stop and ask; the fake box holds three today, so expect none.
- If a process test's story budget moves: report the measurement; do not move the test.

## Steps

1. **Init.** Add the entries to the four templates. In `template.rs`, `Value::Programs` always writes `claude_code`, its `path` the fact or `FETCHED`; add `pub const FETCHED: &[u8] = b"fetched";` to `lib.rs`. `Facts.claude` keeps its meaning; fix its comment.

   Init's tests: rewrite `the_agent_configuration_is_the_owned_examples_but_its_programs`, `the_default_is_the_owned_example_so_edited` and `the_bare_document_is_the_box_less_its_parts` for the new entries. Add a test over `fixtures::cases()` and the bare document: provider, endpoint, model and program, path by fact, and `jig_protocol_config::overrides::parse`'s `limits.core.documents.lines` at least 3.

2. **Reading `"fetched"`.** In `temper/crates/temper/src/agent.rs`:

   ```rust
   /// Claude Code's launcher, beside its programs directory (protocol/setup.md, 6.6).
   pub(crate) fn launcher(directory: &Path) -> Option<PathBuf>;
   /// agent.json as smith decodes it: a program `"fetched"` names its launcher.
   pub(crate) fn decoded(configuration: &Path) -> Result<smith_agent_shell::config::Decoded, String>;
   /// `decoded`, resolved with stderr's diagnostics, as `config::read` does.
   pub(crate) fn configuration(path: &Path) -> Result<smith_agent_shell::config::Configuration, String>;
   ```

   `decoded` is `config::read_bytes`, then, only when the document's path is the word, the crate's `Json` tree (widen `fake_backends/json.rs`'s `pub(super)` to what it needs), then `config::decode`. A document without the word goes to `decode` byte for byte. A `"fetched"` with no `directory`, or none with a parent, is refused naming `programs.claude_code.directory`.

   Replace the reads: `agent.rs`'s `read`, `worker_alone.rs:12`, `lib.rs:467` use `configuration`; `Command::Agent` uses `decoded`, then `smith_agent_shell::run_decoded`, keeping its `prefixed("temper internal agent", ..)`. A refusal of `decoded` there writes the same one diagnostic `run` writes today. Tests beside `agent.rs`: the word replaced, an absolute path left byte for byte, a refusal.

3. **The rewrite.** In `temper-fake/src/backends.rs`, `program_route` and `rewrite`'s agent half find and replace with `selected` and `edits.set`, never `append`: the provider and endpoint by `name` `anthropic`, the model by `endpoint` `anthropic`, each with the `fake-claude` value `program_endpoint`, `program_model` and the provider's bytes make today; the agent document's endpoint by `name`. `programs.claude_code` goes on through `object_member`, which replaces. Bring `EDITS`' count and comment up to date. `capacity` stays as it is. Missing entries are refused by `selected`'s `target missing`.

   `fake_backends/import.rs`: a source's `"fetched"` is `claude: None` for the regenerated documents; an absolute path as today.

   Rewrite's test: a written provider swapped, not doubled, as the acceptance says.

4. **The story.** `temper/crates/temper/tests/support/person.rs`, a script beginning `Begin::Link`, going to `Address::Welcome { step: Step::Model { way: None } }`, seeing the button "Continue with ChatGPT" and the "Use your Claude subscription" act (`welcome_sign_in` is the shape; check the role in `temper-client-view/src/pages/connect_model.rs`). `temper/crates/temper/tests/suite/startup.rs`, `a_fresh_box_offers_chatgpt_and_claude_with_no_claude_on_path`: `temper box` on empty scratch directories, on the empty environment `support/boxed.rs` gives (no `init` first, so the box's own init writes the documents), a person driving that script, then a stop. It also reads `agent.json`'s path, `"fetched"`. Reuse `support::boxed` and the person harness (`support/mod.rs`, around line 729; `run_ctl_with_face`); the choice is yours. Break it once: keep init's provider out, see it fail on Claude's act.

5. **The walk.** `temper/docs/help/box.md`, around lines 44-101: Claude's way in step 1 no longer needs the `jq` that adds Anthropic to the documents: init writes it. Replace that recipe with what stays true: `claude` on `PATH` when init runs names it, otherwise `temper` fetches its pinned build when first needed, and `me` lists provider 2, Anthropic, `claude_code`; `MODEL` goes. Keep the walk's other steps.

6. `cargo fmt`, `python3 -I -B temper/scripts/old_forms.py`, `heavy cargo clippy -p temper-init -p temper-fake -p temper --all-targets -- -D warnings`, then `heavy cargo nextest run -p temper-init -p temper-fake -p temper-service -p temper-playground -p temper-docs`, `-p temper --lib`, `-p temper --test suite`. This changes what a shared document means, so also run `heavy cargo nextest run -p temper --test process -E 'test(temper_box) | test(first_run) | test(fake)'`, the fake backends' walk among them, and `heavy cargo check -p temper-real-loop-world -p temper-engine-domain-world --tests`. Commit, push.

## Report

As your text says, and: the charters' model order as a test saw it; every reader you edited that the list above missed; how long the new story ran; anything the fake box does differently.

## Answers

2026-11-05 (fabrig/ui). Yes: regenerate temper/docs/help/reference/configuration-engine.md in this task. A generated file whose drift test the change makes red is always the changing task's to regenerate; never leave it red.
