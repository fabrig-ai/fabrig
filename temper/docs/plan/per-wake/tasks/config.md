+++
id = "config"
part = "formats"
kind = "feature"
needs = ["surface"]
files = ["jig/crates/jig-protocol-config/**", "jig/testing/jig-test-service/src/startup.rs", "jig/docs/design/protocol/configuration.status.md"]
effort = "high"
minutes = 100
+++

# per-wake: configuration gives a charter its parts and a template a later choice

## Goal

jig's configuration reads a charter's `parts` and a template's `later`, refusing what the
design refuses at its path (`contracts.md`, section 7), so a deployment's document can give
both. The limit `tasks.charter_parts` already exists (the surface's).

Acceptance:

- A charter's `parts` member parses as `jig/docs/design/protocol/configuration.md`,
  section 3 gives it: each `on` name through `jig_domain_tasks::Reasons::named`, each
  `instructions` a configuration path; absent, a charter has none. Each part's file goes
  through the same file list as the charter's own instructions (`charters::files`), its
  path `charters[i].parts[j].instructions`, read within the room the charter has left in
  `core.run_bytes`, and the startup that puts read files back
  (`jig/testing/jig-test-service/src/startup.rs`, `engine_startup`) puts each part's back
  as it does the charter's.
- Refused, each at its path and with the `What` the crate gives the like case today
  (`Unknown` or its equivalent for a name, `Unique` for a repeat, `Bound` for a count or a
  size): an unknown name, an always-offered kind (`ALWAYS_OFFERED_NAMES`) and a repeated
  name at `charters[i].parts[j].on[k]`; an empty `on` at `charters[i].parts[j].on`; a part
  past `tasks.charter_parts` at `charters[i].parts[j]`; parts on a charter with
  `"resume": true` at `charters[i].parts`; a charter whose policy and parts pass
  `core.run_bytes` (`policy_owned_bytes` plus `parts_owned_bytes`) at `charters[i]`.
- A template's `llm` takes an optional `later`, `{"main": ..., "sub_agents": [...]}`,
  named, checked and resolved as the choice's own `main` and `sub_agents` are
  (`src/templates.rs`, `src/llm.rs`), into `LlmChoice.later`; its own refusals at
  `templates[i].llm.later...` as the first choice's are at `templates[i].llm...`. Whether a
  `later` fits its template's charter (its models, a charter that resumes) is the core's
  check, the later task's.
- `members.rs`'s `MEMBERS` gains the rows: a charter's `parts`, a part's `on` and
  `instructions`, an LLM choice's `later`, each with its kind, bound and line; the
  table's agreement tests (`src/tests/members.rs`, `Example` and `EXTRAS`) pass with an
  example of each.
- Rows in the existing refusal table (`src/tests/charters.rs`, its `Case` list and its
  malformed-document rows): one per refusal above, each naming its path; one accepting a
  charter with two parts and reading back their bits, files and order. Rows in
  `src/tests/templates.rs`: a `later` accepted and resolved; one refused for an unknown
  model at its path.
- temper's configuration reference renders jig's `MEMBERS`
  (`temper/crates/temper/src/tests/configuration.rs`): regenerate
  `temper/docs/help/reference/` with that test and `SKEIN_CODEGEN_WRITE=1`, by its tool
  alone, and list what moved.
- Delete from `jig/docs/design/protocol/configuration.status.md` the bullet "3, 6: a charter
  has no `parts` member and there is no `tasks.charter_parts` limit ..." through
  "... the core's check of it against its charter is `../domain/tasks.status.md`, 3.", and
  nothing else (a departure from the implementer's text, which closes `docs/design/`: the
  status must say what is built when this lands).
- `heavy cargo nextest run -p jig-protocol-config` and `-p jig-test-service` pass, and the
  temper reference test.

## Inputs

- `temper/docs/plan/per-wake/notes/common.md`; `notes/contracts.md`, sections 1, 3, 5 and 7.
- `jig/docs/design/protocol/configuration.md`, section 3 (the charters member and "A
  charter's parts"), 3.1 (a template's LLM choice) and 6 (a charter's parts);
  `jig/docs/design/domain/tasks.md`, 8.6.

## Owns and may change

- Owns the files in the header.
- Readers: `temper/docs/help/reference/*.md`, regenerated only, as above; a JSON fixture of a
  world that must name a new member to keep a test's meaning (none expected: both members
  are optional).
- Pre-approved: if putting a read file back also lives in another jig startup than
  `jig-test-service`'s, do the same there and say so; temper passes its charters as
  values (`Defaults::Passed`) and reads no file here. If the crate has no `What` for an
  unknown name, use the one an unknown enumerated value takes elsewhere in it (an act's
  kind, `acts`) and say which.
- Not yours: the domain types, the core's check of a settings change and of a template's
  `later` (parts-core and later tasks), the store and the client wire.

## Steps

1. The refusal and acceptance rows, failing.
2. The `parts` member, its files, its refusals; the template's `later`; the member rows.
3. The rows pass; break one refusal by hand, see its row fail, restore.
4. Regenerate temper's reference; the status bullet.
5. `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py`; `heavy cargo clippy -p
   jig-protocol-config -p jig-test-service --all-targets -- -D warnings`; the focused tests
   of those packages and of `temper` (`--lib`); the citations check.
6. Commit (why: a deployment's document can give a charter its parts and a template a later
   choice); push your branch.

## Report

As the implementer's text says, and also: the `What` each refusal took; the reference pages
regenerated.
