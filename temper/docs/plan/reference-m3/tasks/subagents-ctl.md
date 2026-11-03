+++
id = "subagents-ctl"
part = "subagents"
kind = "feature"
needs = []
files = ["temper/crates/temper-client-tools/src/jig/runner/acts/choice.rs", "temper/crates/temper-client-tools/src/jig/runner/acts.rs", "temper/crates/temper-client-tools/src/jig/rows.rs", "temper/crates/temper-client-tools/src/jig/help.rs", "temper/crates/temper-client-tools/src/tests/help.rs", "temper/crates/temper-client-tools/src/tests/jig/choices.rs", "temper/crates/temper-client-tools/src/tests/golden/**", "temper/docs/help/reference/ctl-chat.md", "temper/docs/design/client/tools.status.md"]
effort = "high"
minutes = 75
+++

# subagents-ctl: `chat start --sub-agent MODEL[:EFFORT]`

## Goal

`temper ctl chat start` takes the chat's sub-agents in its LLM choice, so a person can start a
chat on Claude Code whose `sub_agent` calls run each child at its own model and effort
(temper's `client/tools.md`, 5.5). Today the choice names only the template's sub-agents.

Acceptance:

- `chat start` takes `--sub-agent MODEL[:EFFORT]`, repeatable, at most jig's
  `jig_domain_tasks::MODELS` (64) times: each one of the LLM choice's `subagents`, in the
  order given. MODEL is one `me` offers in the project, as `--model`'s is (`Failed::Unknown`
  naming `model` otherwise); EFFORT one that model declares (`Failed::Unknown` naming
  `effort`); a model that declares efforts and is given without `:EFFORT` fails as `--model`
  without `--effort` does (`Failed::EffortRequired`); a model given twice fails, naming it
  (`sub-agent named twice: MODEL`, exit 1, nothing sent). The text is split at its last `:`.
- `--sub-agent` needs `--model`: given without it, exit 1, `--sub-agent needs --model`,
  nothing sent. Given, the sub-agents replace the template's whole; not given, the choice keeps
  the template's (as now).
- `task amend` is unchanged (its `choice::llm` callers pass no sub-agent argument).
- `temper/docs/help/reference/ctl-chat.md` shows the flag in the usage line and its argument
  line, and a sentence in the notes: `--sub-agent` needs `--model` and replaces the template's
  sub-agents. The help tests and goldens that compile the page pass.
- Tests in `temper/crates/temper-client-tools/src/tests/jig/choices.rs` (or the file there
  that tests `chat start`'s `--model`): two sub-agents sent in order with their efforts; one
  unknown model; an effort the model lacks; a model that needs an effort given none; a model
  twice; `--sub-agent` without `--model`; none given keeping the template's sub-agents.
- `temper/docs/design/client/tools.status.md`: the Not yet text "5.5: `chat start` takes no
  `--sub-agent`, so a chat's choice names only its template's sub-agents
  (`temper-client-tools/src/jig/runner/acts/choice.rs`)." is removed.

## Inputs

- `temper/docs/design/client/tools.md`, 5.5 (the bullet "The task's model").
- jig's `jig/docs/design/protocol/configuration.md`, 3.1 (an LLM choice's `sub_agents`).
- `temper/docs/plan/reference-m3/notes/common.md`; contracts C6.

## Owns and may change

- Owns the files in the header. `rows.rs`: a new argument row for `chat start`
  (`flag(b"sub-agent", b"MODEL[:EFFORT]", Kind::Text, Occurs::Repeated { most: 64 }, ...)` or
  the table's equivalent for a repeatable flag, as `--dependency` is declared) after
  `--effort`.
- `choice.rs`: `llm(...)` keeps its signature for `task amend`; add
  `pub(crate) fn sub_agents(call: &Call, argument: u8, me: &c::Me, project: u32) ->
  Result<Option<List<c::ModelEffort>>, Failed>` (none when the flag is absent) and use it from
  `runner/acts.rs`'s chat start after `llm`, setting `subagents` on the choice.
- `jig/help.rs` and `src/tests/help.rs`: only where chat start's usage line or its help test needs the flag.
- Neighbours: `temper-client-tools`' `Failed` (a new arm or a `Fixed` message for "named twice"
  and "needs --model"; prefer `Fixed` where its words are fixed), its renderer of `Failed`.
- Options: if the argument table cannot declare a repeatable flag with a value, use the
  mechanism `--dependency TASK...` uses for `task amend` and say so. If a golden of `help` or
  `ctl chat` lists usage lines, regenerate it with the crate's own golden mechanism (the
  `UPDATE_GOLDEN`-style switch the tests name), never by hand.

## Steps

1. Read `runner/acts/choice.rs`, `runner/acts.rs` (chat start), `rows.rs` (`MODEL`, `EFFORT`,
   the chat start row) and how `argv.rs` reads `Occurs::Repeated`.
2. Add the row, `sub_agents`, and the wiring; update the help page.
3. Tests as above, in the crate's `src/tests/` world (its scripted engine and `me` fixtures).
4. `heavy cargo nextest run -p temper-client-tools -E 'test(/choice|chat/)'`, then the help
   tests (`-E 'test(/help/)'`); check the counts.
5. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p
   temper-client-tools --all-targets -- -D warnings`; `heavy cargo nextest run -p
   temper-client-tools`; the citations check (common.md). Commit and push.

## Report

As fab's implementer text says; name the golden files regenerated.
